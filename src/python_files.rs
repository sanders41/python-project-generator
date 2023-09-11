use std::fs::File;

use anyhow::{bail, Result};

use crate::file_manager::save_file_with_content;
use crate::project_info::{ProjectInfo, ProjectManager};

fn create_dunder_main_file(source_dir: &str) -> String {
    format!(
        r#"from {source_dir}.main import main  #  pragma: no cover

if __name__ == "__main__":
    raise SystemExit(main())
"#
    )
}

fn create_main_file() -> String {
    r#"def main() -> int:
    print("Hello world!")  # noqa: T201

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
"#
    .to_string()
}

fn save_main_files(project_info: &ProjectInfo) -> Result<()> {
    let src = project_info.base_dir().join(&project_info.source_dir);
    let main = src.join("main.py");
    let main_content = create_main_file();

    save_file_with_content(&main, &main_content)?;

    let main_dunder = src.join("__main__.py");
    let main_dunder_content = create_dunder_main_file(&project_info.source_dir);

    save_file_with_content(&main_dunder, &main_dunder_content)?;

    Ok(())
}

fn create_main_test_file(source_dir: &str) -> String {
    format!(
        r#"from {source_dir}.main import main


def test_main():
    assert main() == 0
"#
    )
}

fn save_main_test_file(project_info: &ProjectInfo) -> Result<()> {
    let file_path = project_info.base_dir().join("tests/test_main.py");
    let content = create_main_test_file(&project_info.source_dir);

    save_file_with_content(&file_path, &content)?;

    Ok(())
}

fn create_pyo3_test_file(source_dir: &str) -> String {
    format!(
        r#"from {source_dir} import sum_as_string


def test_sum_as_string():
    assert sum_as_string(2, 4) == "6"
"#
    )
}

fn save_pyo3_test_file(project_info: &ProjectInfo) -> Result<()> {
    let file_path = project_info
        .base_dir()
        .join(format!("tests/test_{}.py", &project_info.source_dir));
    let content = create_pyo3_test_file(&project_info.source_dir);

    save_file_with_content(&file_path, &content)?;

    Ok(())
}

fn create_project_init_file(source_dir: &str, project_manager: &ProjectManager) -> String {
    match project_manager {
        ProjectManager::Maturin => {
            let v_ascii: u8 = 118;
            if let Some(first_char) = source_dir.chars().next() {
                if (first_char as u8) < v_ascii {
                    format!(
                        r#"from {source_dir}._{source_dir} import sum_as_string
from {source_dir}._version import VERSION

__version__ = VERSION


__all__ = ["sum_as_string"]
"#
                    )
                } else {
                    format!(
                        r#"from {source_dir}._version import VERSION
from {source_dir}._{source_dir} import sum_as_string

__version__ = VERSION


__all__ = ["sum_as_string"]
"#
                    )
                }
            } else {
                format!(
                    r#"from {source_dir}._{source_dir} import sum_as_string
r#"from {source_dir}._version import VERSION

__version__ = VERSION


__all__ = ["sum_as_string"]
"#
                )
            }
        }
        ProjectManager::Poetry => {
            format!(
                r#"from {source_dir}._version import VERSION

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
    let file_path = project_info
        .base_dir()
        .join(format!("{}/__init__.py", &project_info.source_dir));
    let content = create_project_init_file(&project_info.source_dir, &project_info.project_manager);

    save_file_with_content(&file_path, &content)?;

    Ok(())
}

fn create_pyi_file() -> String {
    r#"from __future__ import annotations

def sum_as_string(a: int, b: int) -> str: ...
"#
    .to_string()
}

pub fn save_pyi_file(project_info: &ProjectInfo) -> Result<()> {
    let file_path = project_info.base_dir().join(format!(
        "{}/_{}.pyi",
        &project_info.source_dir, &project_info.source_dir
    ));
    let content = create_pyi_file();

    save_file_with_content(&file_path, &content)?;

    Ok(())
}

fn create_version_file(version: &str) -> String {
    format!("VERSION = \"{version}\"\n")
}

fn save_version_file(project_info: &ProjectInfo) -> Result<()> {
    let file_path = project_info
        .base_dir()
        .join(format!("{}/_version.py", &project_info.source_dir));
    let content = create_version_file(&project_info.version);

    save_file_with_content(&file_path, &content)?;

    Ok(())
}

fn create_version_test_file(source_dir: &str, project_manager: &ProjectManager) -> String {
    let version_test: &str = match project_manager {
        ProjectManager::Maturin => {
            r#"def test_versions_match():
    cargo = Path().absolute() / "Cargo.toml"
    with open(cargo, "rb") as f:
        data = tomllib.load(f)
        cargo_version = data["package"]["version"]

    assert VERSION == cargo_version"#
        }
        ProjectManager::Poetry => {
            r#"def test_versions_match():
    pyproject = Path().absolute() / "pyproject.toml"
    with open(pyproject, "rb") as f:
        data = tomllib.load(f)
        pyproject_version = data["tool"]["poetry"]["version"]

    assert VERSION == pyproject_version"#
        }
    };

    format!(
        r#"import sys
from pathlib import Path

from {source_dir}._version import VERSION

if sys.version_info < (3, 11):
    import tomli as tomllib
else:
    import tomllib


{version_test}
"#
    )
}

fn save_version_test_file(project_info: &ProjectInfo) -> Result<()> {
    let file_path = project_info.base_dir().join("tests/test_version.py");
    let content = create_version_test_file(&project_info.source_dir, &project_info.project_manager);

    save_file_with_content(&file_path, &content)?;

    Ok(())
}

pub fn generate_python_files(project_info: &ProjectInfo) -> Result<()> {
    if save_project_init_file(project_info).is_err() {
        bail!("Error creating __init__.py file");
    }

    if save_test_init_file(&project_info).is_err() {
        bail!("Error creating __init__.py file");
    }

    if project_info.is_application {
        if save_main_files(project_info).is_err() {
            bail!("Error creating main files");
        }

        if save_main_test_file(project_info).is_err() {
            bail!("Error creating main test file");
        }
    }

    if save_version_file(project_info).is_err() {
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
    use crate::project_info::{LicenseType, ProjectInfo, ProjectManager};
    use std::fs::create_dir_all;
    use tempfile::tempdir;

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
            python_version: "3.11".to_string(),
            min_python_version: "3.8".to_string(),
            project_manager: ProjectManager::Maturin,
            is_application: true,
            github_actions_python_test_versions: vec![
                "3.8".to_string(),
                "3.9".to_string(),
                "3.10".to_string(),
                "3.11".to_string(),
            ],
            max_line_length: 100,
            use_dependabot: true,
            use_continuous_deployment: true,
            use_release_drafter: true,
            use_multi_os_ci: true,
            download_latest_packages: false,
            project_root_dir: Some(tempdir().unwrap().path().to_path_buf()),
        }
    }

    #[test]
    fn test_save_project_init_file() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Poetry;
        let base = project_info.base_dir();
        create_dir_all(base.join(&project_info.source_dir)).unwrap();
        let expected_file = base.join(format!("{}/__init__.py", &project_info.source_dir));
        let expected = format!(
            r#"from {}._version import VERSION

__version__ = VERSION
"#,
            &project_info.source_dir
        );

        save_project_init_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_eq!(content, expected);
    }

    #[test]
    fn test_save_project_init_file_pyo3_first() {
        let source_dir = "my_project";
        let expected = format!(
            r#"from {source_dir}._{source_dir} import sum_as_string
from {source_dir}._version import VERSION

__version__ = VERSION


__all__ = ["sum_as_string"]
"#
        )
        .to_string();

        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Maturin;
        let base = project_info.base_dir();
        create_dir_all(base.join(&project_info.source_dir)).unwrap();
        let expected_file = base.join(format!("{}/__init__.py", &project_info.source_dir));
        save_project_init_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_eq!(content, expected);
    }

    #[test]
    fn test_save_project_init_file_pyo3_last() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Maturin;
        project_info.source_dir = "z_my_project".to_string();
        let base = project_info.base_dir();
        create_dir_all(base.join(&project_info.source_dir)).unwrap();
        let expected_file = base.join(format!("{}/__init__.py", &project_info.source_dir));
        let expected = format!(
            r#"from {}._version import VERSION
from {}._{} import sum_as_string

__version__ = VERSION


__all__ = ["sum_as_string"]
"#,
            &project_info.source_dir, &project_info.source_dir, &project_info.source_dir
        );

        save_project_init_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_eq!(content, expected);
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
        let expected_dunder_main = format!(
            r#"from {}.main import main  #  pragma: no cover

if __name__ == "__main__":
    raise SystemExit(main())
"#,
            &project_info.source_dir
        );

        let expected_main = r#"def main() -> int:
    print("Hello world!")  # noqa: T201

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
"#
        .to_string();

        save_main_files(&project_info).unwrap();

        assert!(expected_dunder_main_file.is_file());
        assert!(expected_main_file.is_file());

        let dunder_main_content = std::fs::read_to_string(expected_dunder_main_file).unwrap();

        assert_eq!(dunder_main_content, expected_dunder_main);

        let main_content = std::fs::read_to_string(expected_main_file).unwrap();

        assert_eq!(main_content, expected_main);
    }

    #[test]
    fn test_save_main_test_file() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Poetry;
        project_info.is_application = true;
        let base = project_info.base_dir();
        create_dir_all(base.join("tests")).unwrap();
        let expected_file = base.join("tests/test_main.py");
        let expected = format!(
            r#"from {}.main import main


def test_main():
    assert main() == 0
"#,
            &project_info.source_dir
        );

        save_main_test_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_eq!(content, expected);
    }

    #[test]
    fn test_save_pyo3_test_file() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Maturin;
        let base = project_info.base_dir();
        create_dir_all(base.join("tests")).unwrap();
        let expected_file = base.join(format!("tests/test_{}.py", &project_info.source_dir));
        let expected = format!(
            r#"from {} import sum_as_string


def test_sum_as_string():
    assert sum_as_string(2, 4) == "6"
"#,
            &project_info.source_dir
        );

        save_pyo3_test_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_eq!(content, expected);
    }

    #[test]
    fn test_save_pyi_file() {
        let expected = r#"from __future__ import annotations

def sum_as_string(a: int, b: int) -> str: ...
"#
        .to_string();

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

        assert_eq!(content, expected);
    }

    #[test]
    fn test_save_version_file() {
        let project_info = project_info_dummy();
        let base = project_info.base_dir();
        create_dir_all(base.join(&project_info.source_dir)).unwrap();
        let expected_file = base.join(format!("{}/_version.py", &project_info.source_dir));
        let expected = format!("VERSION = \"{}\"\n", &project_info.version);

        save_version_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_eq!(content, expected);
    }

    #[test]
    fn test_save_version_test_file() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Poetry;
        let base = project_info.base_dir();
        create_dir_all(base.join("tests")).unwrap();
        let expected_file = base.join("tests/test_version.py");
        let expected = format!(
            r#"import sys
from pathlib import Path

from {}._version import VERSION

if sys.version_info < (3, 11):
    import tomli as tomllib
else:
    import tomllib


def test_versions_match():
    pyproject = Path().absolute() / "pyproject.toml"
    with open(pyproject, "rb") as f:
        data = tomllib.load(f)
        pyproject_version = data["tool"]["poetry"]["version"]

    assert VERSION == pyproject_version
"#,
            &project_info.source_dir
        );

        save_version_test_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_eq!(content, expected);
    }

    #[test]
    fn test_save_version_test_file_pyo3() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Maturin;
        let base = project_info.base_dir();
        create_dir_all(base.join("tests")).unwrap();
        let expected_file = base.join("tests/test_version.py");
        let expected = format!(
            r#"import sys
from pathlib import Path

from {}._version import VERSION

if sys.version_info < (3, 11):
    import tomli as tomllib
else:
    import tomllib


def test_versions_match():
    cargo = Path().absolute() / "Cargo.toml"
    with open(cargo, "rb") as f:
        data = tomllib.load(f)
        cargo_version = data["package"]["version"]

    assert VERSION == cargo_version
"#,
            &project_info.source_dir
        );

        save_version_test_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_eq!(content, expected);
    }
}
