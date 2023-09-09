use std::path::PathBuf;

use anyhow::Result;
use colored::*;

use crate::file_manager::save_file_with_content;
use crate::licenses::license_str;
use crate::package_version::{LatestVersion, RustPackageVersion};
use crate::project_info::LicenseType;

fn build_latest_dependencies(min_python_version: &str, download_latest_packages: bool) -> String {
    let mut version_string = String::new();
    let abi = format!("abi3-py{}", min_python_version.replace('.', ""));
    let packages = vec![RustPackageVersion {
        name: "pyo3".to_string(),
        version: "0.19.2".to_string(),
        features: Some(vec!["extension-module".to_string(), abi]),
    }];

    for mut package in packages {
        if download_latest_packages && package.get_latest_version().is_err() {
            let error_message = format!(
                "Error retrieving latest crate version for {:?}. Using default.",
                package.name
            );
            println!("\n{}", error_message.yellow());
        }

        if let Some(features) = &package.features {
            let mut feature_str = "[".to_string();
            for feature in features {
                feature_str.push_str(&format!(r#""{feature}", "#));
            }

            feature_str.truncate(feature_str.len() - 2);
            feature_str.push(']');

            version_string.push_str(&format!(
                "{} = {{ version = \"{}\", features = {} }}\n",
                package.name, package.version, feature_str
            ));
        } else {
            version_string.push_str(&format!(
                "{} = {{ version = \"{}\" }}\n",
                package.name, package.version
            ));
        }
    }

    version_string.trim().to_string()
}

fn create_cargo_toml_file(
    project_slug: &str,
    project_description: &str,
    source_dir: &str,
    license_type: &LicenseType,
    min_python_version: &str,
    download_latest_packages: bool,
) -> String {
    let versions = build_latest_dependencies(min_python_version, download_latest_packages);
    let license = license_str(license_type);

    format!(
        r#"[package]
name = "{project_slug}"
version = "0.1.0"
description = "{project_description}"
edition = "2021"
license = "{license}"
readme = "README.md"

[lib]
name = "_{source_dir}"
crate-type = ["cdylib"]

[dependencies]
{versions}
"#
    )
}

pub fn save_cargo_toml_file(
    project_slug: &str,
    source_dir: &str,
    project_description: &str,
    license_type: &LicenseType,
    min_python_version: &str,
    download_latest_packages: bool,
    project_root_dir: &Option<PathBuf>,
) -> Result<()> {
    let file_path = match project_root_dir {
        Some(root) => format!("{}/{project_slug}/Cargo.toml", root.display()),
        None => format!("{project_slug}/Cargo.toml"),
    };
    let content = create_cargo_toml_file(
        project_slug,
        project_description,
        source_dir,
        license_type,
        min_python_version,
        download_latest_packages,
    );

    save_file_with_content(&file_path, &content)?;

    Ok(())
}

fn create_lib_file(source_dir: &str) -> String {
    format!(
        r#"use pyo3::prelude::*;

#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {{
    Ok((a + b).to_string())
}}

#[pymodule]
fn _{source_dir}(_py: Python, m: &PyModule) -> PyResult<()> {{
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    Ok(())
}}
"#
    )
}

pub fn save_lib_file(
    project_slug: &str,
    source_dir: &str,
    project_root_dir: &Option<PathBuf>,
) -> Result<()> {
    let file_path = match project_root_dir {
        Some(root) => format!("{}/{project_slug}/src/lib.rs", root.display()),
        None => format!("{project_slug}/src/lib.rs"),
    };
    let content = create_lib_file(source_dir);

    save_file_with_content(&file_path, &content)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::create_dir_all;
    use tempfile::tempdir;

    #[test]
    fn test_save_cargo_toml_file() {
        let project_slug = "test-project";
        let source_dir = "test_project";
        let project_description = "Test description";
        let expected = format!(
            r#"[package]
name = "{project_slug}"
version = "0.1.0"
description = "{project_description}"
edition = "2021"
license = "MIT"
readme = "README.md"

[lib]
name = "_{source_dir}"
crate-type = ["cdylib"]

[dependencies]
pyo3 = {{ version = "0.19.2", features = ["extension-module", "abi3-py38"] }}
"#
        );

        let base = tempdir().unwrap().path().to_path_buf();
        create_dir_all(base.join("test-project")).unwrap();
        let expected_file = base.join("test-project/Cargo.toml");
        save_cargo_toml_file(
            project_slug,
            source_dir,
            project_description,
            &LicenseType::Mit,
            "3.8",
            false,
            &Some(base),
        )
        .unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_eq!(content, expected);
    }

    #[test]
    fn test_save_lib_file() {
        let source_dir = "test_project";
        let expected = format!(
            r#"use pyo3::prelude::*;

#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {{
    Ok((a + b).to_string())
}}

#[pymodule]
fn _{source_dir}(_py: Python, m: &PyModule) -> PyResult<()> {{
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    Ok(())
}}
"#
        )
        .to_string();

        let base = tempdir().unwrap().path().to_path_buf();
        create_dir_all(base.join("test-project/src")).unwrap();
        let expected_file = base.join("test-project/src/lib.rs");
        save_lib_file("test-project", source_dir, &Some(base)).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_eq!(content, expected);
    }
}
