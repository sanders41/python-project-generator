use std::path::PathBuf;

use anyhow::Result;
use colored::*;

use crate::file_manager::{save_empty_src_file, save_file_with_content};

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

fn save_main_files(
    project_slug: &str,
    source_dir: &str,
    project_root_dir: &Option<PathBuf>,
) -> Result<()> {
    let src = match project_root_dir {
        Some(root) => format!("{}/{project_slug}/{source_dir}", root.display()),
        None => format!("{project_slug}/{source_dir}"),
    };
    let main = format!("{src}/main.py");
    let main_content = create_main_file();

    save_file_with_content(&main, &main_content)?;

    let main_dunder = format!("{src}/__main__.py");
    let main_dunder_content = create_dunder_main_file(source_dir);

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

fn save_main_test_file(
    project_slug: &str,
    source_dir: &str,
    project_root_dir: &Option<PathBuf>,
) -> Result<()> {
    let file_path = match project_root_dir {
        Some(root) => format!("{}/{project_slug}/tests/test_main.py", root.display()),
        None => format!("{project_slug}/tests/test_main.py"),
    };
    let content = create_main_test_file(source_dir);

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

fn save_pyo3_test_file(
    project_slug: &str,
    source_dir: &str,
    project_root_dir: &Option<PathBuf>,
) -> Result<()> {
    let file_path = match project_root_dir {
        Some(root) => format!(
            "{}/{project_slug}/tests/test_{source_dir}.py",
            root.display()
        ),
        None => format!("{project_slug}/tests/test_{source_dir}.py"),
    };
    let content = create_pyo3_test_file(source_dir);

    save_file_with_content(&file_path, &content)?;

    Ok(())
}

fn create_project_init_file(source_dir: &str, use_pyo3: bool) -> String {
    if use_pyo3 {
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
    } else {
        format!(
            r#"from {source_dir}._version import VERSION

__version__ = VERSION
"#
        )
    }
}

fn save_project_init_file(
    project_slug: &str,
    source_dir: &str,
    use_pyo3: bool,
    project_root_dir: &Option<PathBuf>,
) -> Result<()> {
    let file_path = match project_root_dir {
        Some(root) => format!("{}/{project_slug}/{source_dir}/__init__.py", root.display()),
        None => format!("{project_slug}/{source_dir}/__init__.py"),
    };
    let content = create_project_init_file(source_dir, use_pyo3);

    save_file_with_content(&file_path, &content)?;

    Ok(())
}

fn create_pyi_file() -> String {
    r#"from __future__ import annotations

def sum_as_string(a: int, b: int) -> str: ..."#
        .to_string()
}

pub fn save_pyi_file(
    project_slug: &str,
    source_dir: &str,
    project_root_dir: &Option<PathBuf>,
) -> Result<()> {
    let file_path = match project_root_dir {
        Some(root) => format!(
            "{}/{project_slug}/{source_dir}/_{source_dir}.pyi",
            root.display()
        ),
        None => format!("{project_slug}/{source_dir}/_{source_dir}.pyi"),
    };
    let content = create_pyi_file();

    save_file_with_content(&file_path, &content)?;

    Ok(())
}

fn create_version_file(version: &str) -> String {
    format!("VERSION = \"{version}\"\n")
}

fn save_version_file(
    project_slug: &str,
    source_dir: &str,
    version: &str,
    project_root_dir: &Option<PathBuf>,
) -> Result<()> {
    let file_path = match project_root_dir {
        Some(root) => format!("{}/{project_slug}/{source_dir}/_version.py", root.display()),
        None => format!("{project_slug}/{source_dir}/_version.py"),
    };
    let content = create_version_file(version);

    save_file_with_content(&file_path, &content)?;

    Ok(())
}

fn create_version_test_file(source_dir: &str, use_pyo3: bool) -> String {
    let version_test: &str = if use_pyo3 {
        r#"def test_versions_match():
    cargo = Path().absolute() / "Cargo.toml"
    with open(cargo, "rb") as f:
        data = tomllib.load(f)
        cargo_version = data["package"]["version"]

    assert VERSION == cargo_version"#
    } else {
        r#"def test_versions_match():
    pyproject = Path().absolute() / "pyproject.toml"
    with open(pyproject, "rb") as f:
        data = tomllib.load(f)
        pyproject_version = data["tool"]["poetry"]["version"]

    assert VERSION == pyproject_version"#
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

fn save_version_test_file(
    project_slug: &str,
    source_dir: &str,
    use_pyo3: bool,
    project_root_dir: &Option<PathBuf>,
) -> Result<()> {
    let file_path = match project_root_dir {
        Some(root) => format!("{}/{project_slug}/tests/test_version.py", root.display()),
        None => format!("{project_slug}/tests/test_version.py"),
    };
    let content = create_version_test_file(source_dir, use_pyo3);

    save_file_with_content(&file_path, &content)?;

    Ok(())
}

pub fn generate_python_files(
    is_application: &bool,
    project_slug: &str,
    source_dir: &str,
    version: &str,
    use_pyo3: bool,
    project_root_dir: &Option<PathBuf>,
) {
    if save_project_init_file(project_slug, source_dir, use_pyo3, project_root_dir).is_err() {
        let error_message = "Error creating __init__.py file";
        println!("\n{}", error_message.red());
        std::process::exit(1);
    }

    if save_empty_src_file(project_slug, "tests", "__init__.py", project_root_dir).is_err() {
        let error_message = "Error creating test __init__.py file";
        println!("\n{}", error_message.red());
        std::process::exit(1);
    }

    if *is_application {
        if save_main_files(project_slug, source_dir, project_root_dir).is_err() {
            let error_message = "Error creating main files";
            println!("\n{}", error_message.red());
            std::process::exit(1);
        }

        if save_main_test_file(project_slug, source_dir, project_root_dir).is_err() {
            let error_message = "Error creating main test file";
            println!("\n{}", error_message.red());
            std::process::exit(1);
        }
    }

    if save_version_file(project_slug, source_dir, version, project_root_dir).is_err() {
        let error_message = "Error creating version file";
        println!("\n{}", error_message.red());
        std::process::exit(1);
    }

    if save_version_test_file(project_slug, source_dir, use_pyo3, project_root_dir).is_err() {
        let error_message = "Error creating version test file";
        println!("\n{}", error_message.red());
        std::process::exit(1);
    }

    if use_pyo3 {
        if save_pyi_file(project_slug, source_dir, project_root_dir).is_err() {
            let error_message = "Error creating pyi file";
            println!("\n{}", error_message.red());
            std::process::exit(1);
        }

        if save_pyo3_test_file(project_slug, source_dir, project_root_dir).is_err() {
            let error_message = "Error creating pyo3 test file";
            println!("\n{}", error_message.red());
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::create_dir_all;
    use tempfile::tempdir;

    #[test]
    fn test_save_project_init_file() {
        let expected = r#"from src._version import VERSION

__version__ = VERSION
"#
        .to_string();

        let base = tempdir().unwrap().path().to_path_buf();
        let project_slug = "test-project";
        create_dir_all(base.join(format!("{project_slug}/src"))).unwrap();
        let expected_file = base.join(format!("{project_slug}/src/__init__.py"));
        save_project_init_file(project_slug, "src", false, &Some(base)).unwrap();

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

        let base = tempdir().unwrap().path().to_path_buf();
        let project_slug = "test-project";
        create_dir_all(base.join(format!("{project_slug}/{source_dir}"))).unwrap();
        let expected_file = base.join(format!("{project_slug}/{source_dir}/__init__.py"));
        save_project_init_file(project_slug, source_dir, true, &Some(base)).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_eq!(content, expected);
    }

    #[test]
    fn test_save_project_init_file_pyo3_last() {
        let source_dir = "z_my_project";
        let expected = format!(
            r#"from {source_dir}._version import VERSION
from {source_dir}._{source_dir} import sum_as_string

__version__ = VERSION


__all__ = ["sum_as_string"]
"#
        )
        .to_string();

        let base = tempdir().unwrap().path().to_path_buf();
        let project_slug = "test-project";
        create_dir_all(base.join(format!("{project_slug}/{source_dir}"))).unwrap();
        let expected_file = base.join(format!("{project_slug}/{source_dir}/__init__.py"));
        save_project_init_file(project_slug, source_dir, true, &Some(base)).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_eq!(content, expected);
    }

    #[test]
    fn test_save_main_files() {
        let expected_dunder_main = r#"from src.main import main  #  pragma: no cover

if __name__ == "__main__":
    raise SystemExit(main())
"#
        .to_string();

        let expected_main = r#"def main() -> int:
    print("Hello world!")  # noqa: T201

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
"#
        .to_string();

        let base = tempdir().unwrap().path().to_path_buf();
        let project_slug = "test-project";
        create_dir_all(base.join(format!("{project_slug}/src"))).unwrap();
        let expected_dunder_main_file = base.join(format!("{project_slug}/src/__main__.py"));
        let expected_main_file = base.join(format!("{project_slug}/src/main.py"));
        save_main_files(project_slug, "src", &Some(base)).unwrap();

        assert!(expected_dunder_main_file.is_file());
        assert!(expected_main_file.is_file());

        let dunder_main_content = std::fs::read_to_string(expected_dunder_main_file).unwrap();

        assert_eq!(dunder_main_content, expected_dunder_main);

        let main_content = std::fs::read_to_string(expected_main_file).unwrap();

        assert_eq!(main_content, expected_main);
    }

    #[test]
    fn test_save_main_test_file() {
        let expected = r#"from src.main import main


def test_main():
    assert main() == 0
"#
        .to_string();

        let base = tempdir().unwrap().path().to_path_buf();
        let project_slug = "test-project";
        create_dir_all(base.join(format!("{project_slug}/tests"))).unwrap();
        let expected_file = base.join(format!("{project_slug}/tests/test_main.py"));
        save_main_test_file(project_slug, "src", &Some(base)).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_eq!(content, expected);
    }

    #[test]
    fn test_save_pyo3_test_file() {
        let project_slug = "test-project";
        let source_dir = "test_project";
        let expected = format!(
            r#"from {source_dir} import sum_as_string


def test_sum_as_string():
    assert sum_as_string(2, 4) == "6"
"#
        );

        let base = tempdir().unwrap().path().to_path_buf();
        create_dir_all(base.join(format!("{project_slug}/tests"))).unwrap();
        let expected_file = base.join(format!("{project_slug}/tests/test_{source_dir}.py"));
        save_pyo3_test_file(project_slug, source_dir, &Some(base)).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_eq!(content, expected);
    }

    #[test]
    fn test_save_pyi_file() {
        let expected = r#"from __future__ import annotations

def sum_as_string(a: int, b: int) -> str: ..."#
            .to_string();
        let base = tempdir().unwrap().path().to_path_buf();
        let project_slug = "test-project";
        let source_dir = "test_project";
        create_dir_all(base.join(format!("{project_slug}/{source_dir}"))).unwrap();
        let expected_file = base.join(format!("{project_slug}/{source_dir}/_{source_dir}.pyi"));
        save_pyi_file(project_slug, source_dir, &Some(base)).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_eq!(content, expected);
    }

    #[test]
    fn test_save_version_file() {
        let expected = "VERSION = \"1.2.3\"\n".to_string();

        let base = tempdir().unwrap().path().to_path_buf();
        let project_slug = "test-project";
        create_dir_all(base.join(format!("{project_slug}/src"))).unwrap();
        let expected_file = base.join(format!("{project_slug}/src/_version.py"));
        save_version_file(project_slug, "src", "1.2.3", &Some(base)).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_eq!(content, expected);
    }

    #[test]
    fn test_save_version_test_file() {
        let expected = r#"import sys
from pathlib import Path

from src._version import VERSION

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
"#
        .to_string();

        let base = tempdir().unwrap().path().to_path_buf();
        let project_slug = "test-project";
        create_dir_all(base.join(format!("{project_slug}/tests"))).unwrap();
        let expected_file = base.join(format!("{project_slug}/tests/test_version.py"));
        save_version_test_file(project_slug, "src", false, &Some(base)).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_eq!(content, expected);
    }

    #[test]
    fn test_save_version_test_file_pyo3() {
        let expected = r#"import sys
from pathlib import Path

from src._version import VERSION

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
"#
        .to_string();

        let base = tempdir().unwrap().path().to_path_buf();
        let project_slug = "test-project";
        create_dir_all(base.join(format!("{project_slug}/tests"))).unwrap();
        let expected_file = base.join(format!("{project_slug}/tests/test_version.py"));
        save_version_test_file(project_slug, "src", true, &Some(base)).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_eq!(content, expected);
    }
}
