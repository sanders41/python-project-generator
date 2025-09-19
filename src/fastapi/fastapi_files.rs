use std::fs::create_dir_all;

use anyhow::{bail, Result};
use dirs::data_dir;

use crate::{
    file_manager::save_file_with_content,
    project_info::{DatabaseManager, ProjectInfo, ProjectManager},
    utils::{module_name, source_path},
};

pub fn generate_fastapi(project_info: &ProjectInfo) -> Result<()> {
    create_migrations_dir(project_info)?;
    save_example_env_file(project_info)?;
    Ok(())
}

pub fn create_example_env_file(project_info: &ProjectInfo) -> String {
    let mut info = r#"SECRET_KEY=someKey
FIRST_SUPERUSER_EMAIL=some@email.com
FIRST_SUPERUSER_PASSWORD=changethis
FIRST_SUPERUSER_NAME="Wade Watts"
POSTGRES_HOST=127.0.0.1
POSTGRES_PORT=5432
POSTGRES_USER=postgres
POSTGRES_PASSWORD=some_password
POSTGRES_DB=changethis
STACK_NAME=changethis
DOMAIN=127.0.0.1"#
        .to_string();

    if let Some(database_manager) = &project_info.database_manager {
        if database_manager == &DatabaseManager::AsyncPg {
            info.push_str("DATABASE_URL=postgresql://${POSTGRES_USER}:${POSTGRES_PASSWORD}@${POSTGRES_HOST}:{POSTGRES_PORT}/${POSTGRES_DB}");
        }
    }

    info
}

fn save_example_env_file(project_info: &ProjectInfo) -> Result<()> {
    let base = &project_info.base_dir();
    let file_path = base.join(".env-example");
    let file_content = create_example_env_file(project_info);

    save_file_with_content(&file_path, &file_content)?;

    Ok(())
}

fn create_migrations_dir(project_info: &ProjectInfo) -> Result<()> {
    let base = project_info.base_dir();
    let migrations_dir = base.join("migrations");
    create_dir_all(migrations_dir)?;

    Ok(())
}
