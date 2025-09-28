use std::fs::File;

use anyhow::{bail, Result};

use crate::{
    file_manager::save_file_with_content,
    project_info::{ProjectInfo, ProjectManager},
    utils::is_python_311_or_greater,
};

fn create_dunder_main_file(module: &str, is_async_project: bool) -> String {
    let mut file = "from __future__ import annotations  # pragma: no cover\n\n".to_string();

    if is_async_project {
        file.push_str("import asyncio\n\n");
    }

    file.push_str(&format!(
        r#"from {module}.main import main  #  pragma: no cover

if __name__ == "__main__":
"#
    ));

    if is_async_project {
        file.push_str("    raise SystemExit(asyncio.run(main()))\n");
    } else {
        file.push_str("    raise SystemExit(main())\n");
    }

    file
}

fn create_main_file(is_async_project: bool) -> String {
    if is_async_project {
        r#"from __future__ import annotations

import asyncio


async def main() -> int:
    # TODO: This is placeholder code, remove and replace with your code.
    await asyncio.sleep(1)
    print("Hello world!")  # noqa: T201

    return 0


if __name__ == "__main__":
    raise SystemExit(asyncio.run(main()))
"#
        .to_string()
    } else {
        r#"from __future__ import annotations


def main() -> int:
    print("Hello world!")  # noqa: T201

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
"#
        .to_string()
    }
}

fn save_main_files(project_info: &ProjectInfo) -> Result<()> {
    let module = project_info.module_name();
    let src = project_info.source_dir_path();
    let main = src.join("main.py");
    let main_content = create_main_file(project_info.is_async_project);

    save_file_with_content(&main, &main_content)?;

    let main_dunder = src.join("__main__.py");
    let main_dunder_content = create_dunder_main_file(&module, project_info.is_async_project);

    save_file_with_content(&main_dunder, &main_dunder_content)?;

    Ok(())
}

fn create_main_test_file(module: &str, is_async_project: bool) -> String {
    if is_async_project {
        format!(
            r#"from {module}.main import main


async def test_main():
    assert await main() == 0
"#
        )
    } else {
        format!(
            r#"from {module}.main import main


def test_main():
    assert main() == 0
"#
        )
    }
}

fn save_main_test_file(project_info: &ProjectInfo) -> Result<()> {
    let module = project_info.module_name();
    let file_path = project_info.base_dir().join("tests/test_main.py");
    let content = create_main_test_file(&module, project_info.is_async_project);

    save_file_with_content(&file_path, &content)?;

    Ok(())
}

fn create_pyo3_test_file(module: &str) -> String {
    format!(
        r#"from {module} import sum_as_string


def test_sum_as_string():
    assert sum_as_string(2, 4) == "6"
"#
    )
}

fn save_pyo3_test_file(project_info: &ProjectInfo) -> Result<()> {
    let module = project_info.module_name();
    let file_path = project_info
        .base_dir()
        .join(format!("tests/test_{}.py", &module));
    let content = create_pyo3_test_file(&module);

    save_file_with_content(&file_path, &content)?;

    Ok(())
}

fn create_project_init_file(module: &str, project_manager: &ProjectManager) -> String {
    match project_manager {
        ProjectManager::Maturin => {
            format!(
                r#"from {module}._{module} import __version__, sum_as_string

__all__ = ["__version__", "sum_as_string"]
"#
            )
        }
        _ => {
            format!(
                r#"from {module}._version import VERSION

__version__ = VERSION
"#
            )
        }
    }
}

fn save_test_init_file(project_info: &ProjectInfo) -> Result<()> {
    let file_path = project_info.base_dir().join("tests/__init__.py");
    File::create(file_path)?;

    Ok(())
}

fn save_project_init_file(project_info: &ProjectInfo) -> Result<()> {
    let module = project_info.module_name();
    let file_path = project_info
        .base_dir()
        .join(format!("{}/__init__.py", &module));
    let content = create_project_init_file(&module, &project_info.project_manager);

    save_file_with_content(&file_path, &content)?;

    Ok(())
}

fn create_pyi_file() -> String {
    r#"from __future__ import annotations

__version__: str

def sum_as_string(a: int, b: int) -> str: ...
"#
    .to_string()
}

pub fn save_pyi_file(project_info: &ProjectInfo) -> Result<()> {
    let module = project_info.module_name();
    let file_path = project_info
        .base_dir()
        .join(format!("{}/_{}.pyi", &module, &module));
    let content = create_pyi_file();

    save_file_with_content(&file_path, &content)?;

    Ok(())
}

fn create_version_file(version: &str) -> String {
    format!("VERSION = \"{version}\"\n")
}

fn save_version_file(project_info: &ProjectInfo) -> Result<()> {
    let module = project_info.module_name();
    let file_path = project_info
        .base_dir()
        .join(format!("{}/_version.py", &module));
    let content = create_version_file(&project_info.version);

    save_file_with_content(&file_path, &content)?;

    Ok(())
}

fn create_version_test_file(
    module: &str,
    project_manager: &ProjectManager,
    min_python_version: &str,
) -> Result<Option<String>> {
    let version_test: Option<&str> = match project_manager {
        ProjectManager::Poetry => Some(
            r#"def test_versions_match():
    pyproject = Path().absolute() / "pyproject.toml"
    with open(pyproject, "rb") as f:
        data = tomllib.load(f)
        pyproject_version = data["tool"]["poetry"]["version"]

    assert VERSION == pyproject_version"#,
        ),
        _ => None,
    };

    if let Some(v) = version_test {
        if is_python_311_or_greater(min_python_version)? {
            Ok(Some(format!(
                r#"import tomllib
from pathlib import Path

from {module}._version import VERSION


{v}
"#
            )))
        } else {
            Ok(Some(format!(
                r#"import sys
from pathlib import Path

from {module}._version import VERSION

if sys.version_info < (3, 11):
    import tomli as tomllib
else:
    import tomllib


{v}
"#
            )))
        }
    } else {
        Ok(None)
    }
}

fn save_version_test_file(project_info: &ProjectInfo) -> Result<()> {
    let module = project_info.module_name();
    let file_path = project_info.base_dir().join("tests/test_version.py");
    let content = create_version_test_file(
        &module,
        &project_info.project_manager,
        &project_info.min_python_version,
    )?;

    if let Some(c) = content {
        save_file_with_content(&file_path, &c)?;
    }

    Ok(())
}

pub fn generate_python_files(project_info: &ProjectInfo) -> Result<()> {
    if save_project_init_file(project_info).is_err() {
        bail!("Error creating __init__.py file");
    }

    if save_test_init_file(project_info).is_err() {
        bail!("Error creating __init__.py file");
    }

    #[cfg(not(feature = "fastapi"))]
    if project_info.is_application {
        if save_main_files(project_info).is_err() {
            bail!("Error creating main files");
        }

        if save_main_test_file(project_info).is_err() {
            bail!("Error creating main test file");
        }
    }

    #[cfg(feature = "fastapi")]
    if project_info.is_application && !project_info.is_fastapi_project {
        if save_main_files(project_info).is_err() {
            bail!("Error creating main files");
        }

        if save_main_test_file(project_info).is_err() {
            bail!("Error creating main test file");
        }
    }

    if project_info.project_manager != ProjectManager::Maturin
        && save_version_file(project_info).is_err()
    {
        bail!("Error creating version file");
    }

    if save_version_test_file(project_info).is_err() {
        bail!("Error creating version test file")
    }

    if let ProjectManager::Maturin = project_info.project_manager {
        if save_pyi_file(project_info).is_err() {
            bail!("Error creating pyi file");
        }

        if save_pyo3_test_file(project_info).is_err() {
            bail!("Error creating pyo3 test file");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project_info::{LicenseType, ProjectInfo, ProjectManager, Pyo3PythonManager};
    use insta::assert_yaml_snapshot;
    use std::fs::create_dir_all;
    use tmp_path::tmp_path;

    #[tmp_path]
    fn project_info_dummy() -> ProjectInfo {
        ProjectInfo {
            project_name: "My project".to_string(),
            project_slug: "my-project".to_string(),
            source_dir: "my_project".to_string(),
            project_description: "This is a test".to_string(),
            creator: "Arthur Dent".to_string(),
            creator_email: "authur@heartofgold.com".to_string(),
            license: LicenseType::Mit,
            copyright_year: Some("2023".to_string()),
            version: "0.1.0".to_string(),
            python_version: "3.12".to_string(),
            min_python_version: "3.9".to_string(),
            project_manager: ProjectManager::Maturin,
            pyo3_python_manager: Some(Pyo3PythonManager::Uv),
            is_application: true,
            is_async_project: false,
            github_actions_python_test_versions: vec![
                "3.9".to_string(),
                "3.10".to_string(),
                "3.11".to_string(),
                "3.12".to_string(),
            ],
            max_line_length: 100,
            use_dependabot: true,
            dependabot_schedule: None,
            dependabot_day: None,
            use_continuous_deployment: true,
            use_release_drafter: true,
            use_multi_os_ci: true,
            include_docs: false,
            docs_info: None,
            download_latest_packages: false,
            project_root_dir: Some(tmp_path),

            #[cfg(feature = "fastapi")]
            is_fastapi_project: false,

            #[cfg(feature = "fastapi")]
            database_manager: None,
        }
    }

    #[test]
    fn test_save_project_init_file() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Poetry;
        let base = project_info.base_dir();
        create_dir_all(base.join(&project_info.source_dir)).unwrap();
        let expected_file = base.join(format!("{}/__init__.py", &project_info.source_dir));

        save_project_init_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_yaml_snapshot!(content);
    }

    #[test]
    fn test_save_project_init_file_pyo3_first() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Maturin;
        let base = project_info.base_dir();
        create_dir_all(base.join(&project_info.source_dir)).unwrap();
        let expected_file = base.join(format!("{}/__init__.py", &project_info.source_dir));
        save_project_init_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_yaml_snapshot!(content);
    }

    #[test]
    fn test_save_project_init_file_pyo3_last() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Maturin;
        project_info.source_dir = "z_my_project".to_string();
        let base = project_info.base_dir();
        create_dir_all(base.join(&project_info.source_dir)).unwrap();
        let expected_file = base.join(format!("{}/__init__.py", &project_info.source_dir));
        save_project_init_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_yaml_snapshot!(content);
    }

    #[test]
    fn test_save_main_files() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Poetry;
        project_info.is_application = true;
        let base = project_info.base_dir();
        create_dir_all(base.join(&project_info.source_dir)).unwrap();
        let expected_dunder_main_file =
            base.join(format!("{}/__main__.py", &project_info.source_dir));
        let expected_main_file = base.join(format!("{}/main.py", &project_info.source_dir));
        save_main_files(&project_info).unwrap();

        assert!(expected_dunder_main_file.is_file());
        assert!(expected_main_file.is_file());

        let dunder_main_content = std::fs::read_to_string(expected_dunder_main_file).unwrap();

        assert_yaml_snapshot!(dunder_main_content);

        let main_content = std::fs::read_to_string(expected_main_file).unwrap();

        assert_yaml_snapshot!(main_content);
    }

    #[test]
    fn test_save_main_files_is_async_project() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Poetry;
        project_info.is_application = true;
        project_info.is_async_project = true;
        let base = project_info.base_dir();
        create_dir_all(base.join(&project_info.source_dir)).unwrap();
        let expected_dunder_main_file =
            base.join(format!("{}/__main__.py", &project_info.source_dir));
        let expected_main_file = base.join(format!("{}/main.py", &project_info.source_dir));
        save_main_files(&project_info).unwrap();

        assert!(expected_dunder_main_file.is_file());
        assert!(expected_main_file.is_file());

        let dunder_main_content = std::fs::read_to_string(expected_dunder_main_file).unwrap();

        assert_yaml_snapshot!(dunder_main_content);

        let main_content = std::fs::read_to_string(expected_main_file).unwrap();

        assert_yaml_snapshot!(main_content);
    }

    #[test]
    fn test_save_main_test_file() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Poetry;
        project_info.is_application = true;
        let base = project_info.base_dir();
        create_dir_all(base.join("tests")).unwrap();
        let expected_file = base.join("tests/test_main.py");
        save_main_test_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_yaml_snapshot!(content);
    }

    #[test]
    fn test_save_main_test_file_is_async_project() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Poetry;
        project_info.is_application = true;
        project_info.is_async_project = true;
        let base = project_info.base_dir();
        create_dir_all(base.join("tests")).unwrap();
        let expected_file = base.join("tests/test_main.py");
        save_main_test_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_yaml_snapshot!(content);
    }

    #[test]
    fn test_save_pyo3_test_file() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Maturin;
        let base = project_info.base_dir();
        create_dir_all(base.join("tests")).unwrap();
        let expected_file = base.join(format!("tests/test_{}.py", &project_info.source_dir));

        save_pyo3_test_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_yaml_snapshot!(content);
    }

    #[test]
    fn test_save_pyi_file() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Maturin;
        project_info.is_application = true;
        let base = project_info.base_dir();
        create_dir_all(base.join(&project_info.source_dir)).unwrap();
        let expected_file = base.join(format!(
            "{}/_{}.pyi",
            &project_info.source_dir, &project_info.source_dir
        ));

        save_pyi_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_yaml_snapshot!(content);
    }

    #[test]
    fn test_save_version_file() {
        let project_info = project_info_dummy();
        let base = project_info.base_dir();
        create_dir_all(base.join(&project_info.source_dir)).unwrap();
        let expected_file = base.join(format!("{}/_version.py", &project_info.source_dir));
        save_version_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_yaml_snapshot!(content);
    }

    #[test]
    fn test_save_version_test_file_poetry_tomli() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Poetry;
        let base = project_info.base_dir();
        create_dir_all(base.join("tests")).unwrap();
        let expected_file = base.join("tests/test_version.py");
        save_version_test_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_yaml_snapshot!(content);
    }

    #[test]
    fn test_save_version_test_file_poetry_no_tomli() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Poetry;
        project_info.min_python_version = "3.12".to_string();
        let base = project_info.base_dir();
        create_dir_all(base.join("tests")).unwrap();
        let expected_file = base.join("tests/test_version.py");
        save_version_test_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_yaml_snapshot!(content);
    }

    #[test]
    fn test_save_version_test_file_setuptools() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Setuptools;
        let base = project_info.base_dir();
        create_dir_all(base.join("tests")).unwrap();
        let expected_file = base.join("tests/test_version.py");
        save_version_test_file(&project_info).unwrap();

        assert!(!expected_file.is_file());
    }

    #[test]
    fn test_save_version_test_file_uv() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Uv;
        let base = project_info.base_dir();
        create_dir_all(base.join("tests")).unwrap();
        let expected_file = base.join("tests/test_version.py");
        save_version_test_file(&project_info).unwrap();

        assert!(!expected_file.is_file());
    }

    #[test]
    fn test_save_version_test_file_pixi() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Pixi;
        let base = project_info.base_dir();
        create_dir_all(base.join("tests")).unwrap();
        let expected_file = base.join("tests/test_version.py");
        save_version_test_file(&project_info).unwrap();

        assert!(!expected_file.is_file());
    }
}
