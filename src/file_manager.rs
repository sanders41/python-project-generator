use std::fs::File;
use std::io::prelude::*;

use anyhow::Result;

pub fn save_file_with_content(file_path: &str, file_content: &str) -> Result<()> {
    let mut file = File::create(file_path)?;
    file.write_all(file_content.as_bytes())?;

    Ok(())
}

pub fn save_empty_src_file(project_slug: &str, source_dir: &str, file_name: &str) -> Result<()> {
    let init_file = format!("{project_slug}/{source_dir}/{file_name}");
    File::create(init_file)?;

    Ok(())
}
