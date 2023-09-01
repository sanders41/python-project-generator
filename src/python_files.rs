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
    print("Hello world!")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
"#
    .to_string()
}

fn save_main_files(project_slug: &str, source_dir: &str) -> Result<()> {
    let src = format!("{project_slug}/{source_dir}");
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

fn save_main_test_file(project_slug: &str, source_dir: &str) -> Result<()> {
    let main_test_path = format!("{project_slug}/tests/test_main.py");
    let main_test_content = create_main_test_file(source_dir);

    save_file_with_content(&main_test_path, &main_test_content)?;

    Ok(())
}

fn create_project_init_file(source_dir: &str) -> String {
    format!(
        r#"from {source_dir}._version import VERSION


__version__ = VERSION
"#
    )
}

fn save_project_init_file(project_slug: &str, source_dir: &str) -> Result<()> {
    let file_path = format!("{project_slug}/{source_dir}/__init__.py");
    let content = create_project_init_file(source_dir);

    save_file_with_content(&file_path, &content)?;

    Ok(())
}

fn create_version_file(version: &str) -> String {
    format!(r#"VERSION = "{version}""#)
}

fn save_version_file(project_slug: &str, source_dir: &str, version: &str) -> Result<()> {
    let version_file_path = format!("{project_slug}/{source_dir}/_version.py");
    let version_content = create_version_file(version);

    save_file_with_content(&version_file_path, &version_content)?;

    Ok(())
}

fn create_version_test_file(source_dir: &str) -> String {
    format!(
        r#"import sys
from pathlib import Path

from {source_dir}._version import VERSION

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
    )
}

fn save_version_test_file(project_slug: &str, source_dir: &str) -> Result<()> {
    let file_path = format!("{project_slug}/tests/test_version.py");
    let content = create_version_test_file(source_dir);

    save_file_with_content(&file_path, &content)?;

    Ok(())
}

pub fn generate_python_files(
    is_application: &bool,
    project_slug: &str,
    source_dir: &str,
    version: &str,
) {
    if save_project_init_file(project_slug, source_dir).is_err() {
        let error_message = "Error creating __init__.py file";
        println!("\n{}", error_message.red());
        std::process::exit(1);
    }

    if save_empty_src_file(project_slug, "tests", "__init__.py").is_err() {
        let error_message = "Error creating test __init__.py file";
        println!("\n{}", error_message.red());
        std::process::exit(1);
    }

    if *is_application {
        if save_main_files(project_slug, source_dir).is_err() {
            let error_message = "Error creating main files";
            println!("\n{}", error_message.red());
            std::process::exit(1);
        }

        if save_main_test_file(project_slug, source_dir).is_err() {
            let error_message = "Error creating main test file";
            println!("\n{}", error_message.red());
            std::process::exit(1);
        }
    }

    if save_version_file(project_slug, source_dir, version).is_err() {
        let error_message = "Error creating version file";
        println!("\n{}", error_message.red());
        std::process::exit(1);
    }

    if save_version_test_file(project_slug, source_dir).is_err() {
        let error_message = "Error creating version test file";
        println!("\n{}", error_message.red());
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_project_init_file() {
        let expected = r#"from src._version import VERSION


__version__ = VERSION
"#
        .to_string();

        assert_eq!(create_project_init_file("src"), expected);
    }

    #[test]
    fn test_create_dunder_main_file() {
        let expected = r#"from src.main import main  #  pragma: no cover

if __name__ == "__main__":
    raise SystemExit(main())
"#
        .to_string();

        assert_eq!(create_dunder_main_file("src"), expected);
    }

    #[test]
    fn test_create_main_file() {
        let expected = r#"def main() -> int:
    print("Hello world!")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
"#
        .to_string();

        assert_eq!(create_main_file(), expected);
    }

    #[test]
    fn test_create_main_test_file() {
        let expected = r#"from src.main import main


def test_main():
    assert main() == 0
"#
        .to_string();

        assert_eq!(create_main_test_file("src"), expected);
    }

    #[test]
    fn test_create_version_file() {
        let expected = r#"VERSION = "1.2.3""#.to_string();
        assert_eq!(create_version_file("1.2.3"), expected);
    }

    #[test]
    fn test_create_version_test_file() {
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

        assert_eq!(create_version_test_file("src"), expected);
    }
}
