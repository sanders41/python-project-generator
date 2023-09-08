use std::path::PathBuf;

use anyhow::Result;

use crate::file_manager::save_file_with_content;

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
