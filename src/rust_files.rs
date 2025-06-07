use anyhow::Result;
use colored::*;
use rayon::prelude::*;

use crate::file_manager::save_file_with_content;
use crate::licenses::license_str;
use crate::package_version::{LatestVersion, RustPackageVersion};
use crate::project_info::{LicenseType, ProjectInfo};

fn build_latest_dependencies(download_latest_packages: bool) -> String {
    let mut version_string = String::new();
    let mut packages = vec![RustPackageVersion {
        name: "pyo3".to_string(),
        version: "0.25.0".to_string(),
        features: Some(vec!["extension-module".to_string()]),
    }];

    if download_latest_packages {
        packages.par_iter_mut().for_each(|package| {
            if package.get_latest_version().is_err() {
                let error_message = format!(
                    "Error retrieving latest crate version for {}. Using default.",
                    package.name
                );
                println!("\n{}", error_message.yellow());
            }
        })
    }

    for package in packages {
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
    download_latest_packages: bool,
) -> String {
    let versions = build_latest_dependencies(download_latest_packages);
    let license = license_str(license_type);
    let name = source_dir.replace([' ', '-'], "_");

    format!(
        r#"[package]
name = "{project_slug}"
version = "0.1.0"
description = "{project_description}"
edition = "2021"
license = "{license}"
readme = "README.md"

[lib]
name = "_{name}"
crate-type = ["cdylib"]

[dependencies]
{versions}
"#
    )
}

pub fn save_cargo_toml_file(project_info: &ProjectInfo) -> Result<()> {
    let file_path = project_info.base_dir().join("Cargo.toml");
    let content = create_cargo_toml_file(
        &project_info.project_slug,
        &project_info.project_description,
        &project_info.source_dir,
        &project_info.license,
        project_info.download_latest_packages,
    );

    save_file_with_content(&file_path, &content)?;

    Ok(())
}

fn create_lib_file(source_dir: &str) -> String {
    let module = source_dir.replace([' ', '-'], "_");
    format!(
        r#"use pyo3::prelude::*;

#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {{
    Ok((a + b).to_string())
}}

#[pymodule]
fn _{module}(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {{
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}}
"#
    )
}

pub fn save_lib_file(project_info: &ProjectInfo) -> Result<()> {
    let file_path = project_info.base_dir().join("src/lib.rs");
    let content = create_lib_file(&project_info.source_dir);

    save_file_with_content(&file_path, &content)?;

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
        }
    }

    #[test]
    fn test_save_cargo_toml_file() {
        let project_info = project_info_dummy();
        let base = project_info.base_dir();
        create_dir_all(base.join(&project_info.project_slug)).unwrap();
        let expected_file = base.join("Cargo.toml");
        save_cargo_toml_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        insta::with_settings!({filters => vec![
            (r"\d+\.\d+\.\d+", "1.0.0"),
        ]}, { assert_yaml_snapshot!(content)});
    }

    #[test]
    fn test_save_lib_file() {
        let project_info = project_info_dummy();
        let base = project_info.base_dir();
        create_dir_all(base.join("src")).unwrap();
        let expected_file = base.join("src/lib.rs");
        save_lib_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_yaml_snapshot!(content);
    }
}
