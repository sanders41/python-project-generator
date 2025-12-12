use anyhow::Result;

use crate::{
    file_manager::save_file_with_content,
    licenses::license_str,
    project_info::{LicenseType, ProjectInfo},
};

fn create_cargo_toml_file(
    project_slug: &str,
    project_description: &str,
    source_dir: &str,
    license_type: &LicenseType,
) -> String {
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
    );

    save_file_with_content(&file_path, &content)?;

    Ok(())
}

pub fn cargo_add_pyo3(project_info: &ProjectInfo) -> Result<()> {
    use anyhow::bail;

    let output = std::process::Command::new("cargo")
        .args(["add", "pyo3", "--features", "extension-module"])
        .current_dir(project_info.base_dir())
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to add pyo3 dependency: {stderr}");
    }

    Ok(())
}

fn create_lib_file(project_info: &ProjectInfo) -> String {
    let module = project_info.module_name();
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
    let content = create_lib_file(project_info);

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
            min_python_version: "3.10".to_string(),
            project_manager: ProjectManager::Maturin,
            pyo3_python_manager: Some(Pyo3PythonManager::Uv),
            is_application: true,
            is_async_project: false,
            github_actions_python_test_versions: vec![
                "3.10".to_string(),
                "3.11".to_string(),
                "3.12".to_string(),
                "3.13".to_string(),
                "3.14".to_string(),
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
            project_root_dir: Some(tmp_path),

            #[cfg(feature = "fastapi")]
            is_fastapi_project: false,

            #[cfg(feature = "fastapi")]
            database_manager: None,
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
