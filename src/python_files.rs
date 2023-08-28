use anyhow::Result;
use colored::*;

use crate::file_manager::{create_empty_src_file, create_file_with_content};

fn create_main_files(project_slug: &str, source_dir: &str) -> Result<()> {
    let src = format!("{project_slug}/{source_dir}");
    let main = format!("{src}/main.py");
    let main_content = r#"def main() -> int:
    print("Hello world!")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
"#;

    create_file_with_content(&main, main_content)?;

    let main_dunder = format!("{src}/__main__.py");
    let main_dunder_content = format!(
        r#"from {source_dir}.main import main  #  pragma: no cover

if __name__ == "__main__":
    raise SystemExit(main())
"#
    );

    create_file_with_content(&main_dunder, &main_dunder_content)?;

    Ok(())
}

fn create_main_test_file(project_slug: &str, source_dir: &str) -> Result<()> {
    let main_test_path = format!("{project_slug}/tests/test_main.py");
    let main_test_content = format!(
        r#"from {source_dir}.main import main


def test_main():
    assert main() == 0
"#
    );

    create_file_with_content(&main_test_path, &main_test_content)?;

    Ok(())
}

fn create_project_init_file(project_slug: &str, source_dir: &str) -> Result<()> {
    let file_path = format!("{project_slug}/{source_dir}/__init__.py");
    let content = format!(
        r#"from {source_dir}._version import VERSION


__version__ = VERSION
"#
    );

    create_file_with_content(&file_path, &content)?;

    Ok(())
}

fn create_version_file(project_slug: &str, source_dir: &str, version: &str) -> Result<()> {
    let version_file_path = format!("{project_slug}/{source_dir}/_version.py");
    let version_content = format!(r#"VERSION = "{version}""#);

    create_file_with_content(&version_file_path, &version_content)?;

    Ok(())
}

fn create_version_test_file(project_slug: &str, source_dir: &str) -> Result<()> {
    let file_path = format!("{project_slug}/tests/test_version.py");
    let content = format!(
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
    );

    create_file_with_content(&file_path, &content)?;

    Ok(())
}

pub fn generate_python_files(
    is_application: &bool,
    project_slug: &str,
    source_dir: &str,
    version: &str,
) {
    if create_project_init_file(project_slug, source_dir).is_err() {
        let error_message = "Error creating __init__.py file";
        println!("\n{}", error_message.red());
        std::process::exit(1);
    }

    if create_empty_src_file(project_slug, "tests", "__init__.py").is_err() {
        let error_message = "Error creating test __init__.py file";
        println!("\n{}", error_message.red());
        std::process::exit(1);
    }

    if *is_application {
        if create_main_files(project_slug, source_dir).is_err() {
            let error_message = "Error creating main files";
            println!("\n{}", error_message.red());
            std::process::exit(1);
        }

        if create_main_test_file(project_slug, source_dir).is_err() {
            let error_message = "Error creating main test file";
            println!("\n{}", error_message.red());
            std::process::exit(1);
        }
    }

    if create_version_file(project_slug, source_dir, version).is_err() {
        let error_message = "Error creating version file";
        println!("\n{}", error_message.red());
        std::process::exit(1);
    }

    if create_version_test_file(project_slug, source_dir).is_err() {
        let error_message = "Error creating version test file";
        println!("\n{}", error_message.red());
        std::process::exit(1);
    }
}
