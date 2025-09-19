use std::{fs::File, io::prelude::*, path::PathBuf};

use anyhow::Result;

use crate::project_info::ProjectInfo;

pub fn save_file_with_content(file_path: &PathBuf, file_content: &str) -> Result<()> {
    let mut file = File::create(file_path)?;
    file.write_all(file_content.as_bytes())?;

    Ok(())
}

pub fn save_empty_src_file(project_info: &ProjectInfo, file_name: &str) -> Result<()> {
    let module = project_info.source_dir.replace([' ', '-'], "_");
    let file_path = project_info
        .base_dir()
        .join(format!("{}/{}", &module, file_name));
    File::create(file_path)?;

    Ok(())
}
