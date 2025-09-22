use anyhow::Result;
use time::OffsetDateTime;

use crate::{file_manager::save_file_with_content, project_info::ProjectInfo};

fn create_initial_up_migration() -> String {
    r#"CREATE TABLE IF NOT EXISTS users (
  id TEXT PRIMARY KEY,
  email TEXT NOT NULL UNIQUE,
  full_name TEXT NOT NULL,
  hashed_password TEXT NOT NULL,
  is_active BOOLEAN NOT NULL DEFAULT true,
  is_superuser BOOLEAN NOT NULL DEFAULT false,
  last_login TIMESTAMP NOT NULL DEFAULT NOW()
);
"#
    .to_string()
}

fn create_initial_down_migration() -> String {
    "DROP TABLE IF EXISTS users;".to_string()
}

pub fn save_initial_migrations(project_info: &ProjectInfo) -> Result<()> {
    let now = OffsetDateTime::now_utc();
    let migration_prefix = format!(
        "{:04}{:02}{:02}{:02}{:02}{:02}",
        now.year(),
        now.month() as u8,
        now.day(),
        now.hour(),
        now.minute(),
        now.second()
    );
    let up_file_name = format!("{migration_prefix}_init.up.sql");
    let down_file_name = format!("{migration_prefix}_init.down.sql");

    let base = project_info.base_dir();
    let up_file_path = base.join(format!("migrations/{up_file_name}"));
    let up_file_content = create_initial_up_migration();

    save_file_with_content(&up_file_path, &up_file_content)?;

    let down_file_path = base.join(format!("migrations/{down_file_name}"));
    let down_file_content = create_initial_down_migration();

    save_file_with_content(&down_file_path, &down_file_content)?;

    Ok(())
}
