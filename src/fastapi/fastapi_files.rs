use std::fs::create_dir_all;

use anyhow::{bail, Result};

use crate::{
    project_info::{DatabaseManager, ProjectInfo, ProjectManager},
    utils::source_path,
};

pub fn generate_fastapi(project_info: &ProjectInfo) -> Result<()> {
    create_migrations_dir(project_info)?;
    Ok(())
}

fn create_migrations_dir(project_info: &ProjectInfo) -> Result<()> {
    let base = project_info.base_dir();
    let migrations_dir = base.join("migrations");
    create_dir_all(migrations_dir)?;

    Ok(())
}
