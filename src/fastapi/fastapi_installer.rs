use anyhow::{bail, Result};

use crate::project_info::{DatabaseManager, ProjectInfo, ProjectManager};

const FASTAPI_BASE_DEPENDENCIES: &[&str] = &[
    "asyncpg",
    "camel-converter[pydantic]",
    "fastapi",
    "granian[pname,reload]",
    "httptools",
    "loguru",
    "orjson",
    "pwdlib[argon2]",
    "pydantic[email]",
    "pydantic-settings",
    "pyjwt",
    "python-multipart",
    "valkey",
];

const FASTAPI_BASE_UNIX_DEPENDENCIES: &[&str] = &["uvloop"];

const FASTAPI_BASE_DEV_DEPENDENCIES: &[&str] = &["httpx"];

pub fn install_fastapi_dependencies(project_info: &ProjectInfo) -> Result<()> {
    match project_info.project_manager {
        ProjectManager::Uv => uv_fastapi_depencency_installer(project_info)?,
        ProjectManager::Poetry => poetry_fastapi_depencency_installer(project_info)?,
        ProjectManager::Setuptools => setuptools_fastapi_depencency_installer(project_info)?,
        ProjectManager::Pixi => bail!("Pixi is not currently supported for FastAPI projects"),
        ProjectManager::Maturin => maturin_fastapi_depencency_installer(project_info)?,
    };

    Ok(())
}

fn get_fastapi_base_dependencies() -> Vec<&'static str> {
    let mut base_dependencies = FASTAPI_BASE_DEPENDENCIES.to_vec();
    if cfg!(unix) {
        base_dependencies.extend_from_slice(FASTAPI_BASE_UNIX_DEPENDENCIES);
    }
    base_dependencies
}

fn uv_fastapi_depencency_installer(project_info: &ProjectInfo) -> Result<()> {
    let mut dependencies = get_fastapi_base_dependencies();
    if project_info.database_manager == Some(DatabaseManager::SqlAlchemy) {
        dependencies.push("sqlalchemy");
        dependencies.push("alembic");
    }
    let mut args = vec!["add"];
    args.extend(dependencies);
    let output = std::process::Command::new("uv")
        .args(args)
        .current_dir(project_info.base_dir())
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to install FastAPI dependencies: {stderr}");
    }

    let dev_dependencies = FASTAPI_BASE_DEV_DEPENDENCIES.to_vec();
    let mut dev_args = vec!["add", "--group=dev"];
    dev_args.extend(dev_dependencies);
    let output = std::process::Command::new("uv")
        .args(dev_args)
        .current_dir(project_info.base_dir())
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to install FastAPI dependencies: {stderr}");
    }

    Ok(())
}

fn poetry_fastapi_depencency_installer(project_info: &ProjectInfo) -> Result<()> {
    let mut dependencies = get_fastapi_base_dependencies();
    if project_info.database_manager == Some(DatabaseManager::SqlAlchemy) {
        dependencies.push("sqlalchemy");
        dependencies.push("alembic");
    }
    let mut args = vec!["add"];
    args.extend(dependencies);
    let output = std::process::Command::new("poetry")
        .args(args)
        .current_dir(project_info.base_dir())
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to install FastAPI dependencies: {stderr}");
    }

    let dev_dependencies = FASTAPI_BASE_DEV_DEPENDENCIES.to_vec();
    let mut dev_args = vec!["add", "--group=dev"];
    dev_args.extend(dev_dependencies);
    let output = std::process::Command::new("poetry")
        .args(dev_args)
        .current_dir(project_info.base_dir())
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to install FastAPI dependencies: {stderr}");
    }

    Ok(())
}

fn setuptools_fastapi_depencency_installer(project_info: &ProjectInfo) -> Result<()> {
    let venv_output = std::process::Command::new("python")
        .args(["-m", "venv", ".venv"])
        .current_dir(project_info.base_dir())
        .output()?;

    if !venv_output.status.success() {
        let stderr = String::from_utf8_lossy(&venv_output.stderr);
        bail!("Failed to create virtual environment: {stderr}");
    }

    let mut dependencies = get_fastapi_base_dependencies();
    let dev_dependencies = FASTAPI_BASE_DEV_DEPENDENCIES.to_vec();
    if project_info.database_manager == Some(DatabaseManager::SqlAlchemy) {
        dependencies.push("sqlalchemy");
        dependencies.push("alembic");
    }
    let mut args = vec!["-m", "pip", "install"];
    args.extend(dependencies);
    args.extend(dev_dependencies);
    let output = std::process::Command::new(".venv/bin/python")
        .args(args)
        .current_dir(project_info.base_dir())
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to install FastAPI dependencies: {stderr}");
    }

    Ok(())
}

fn maturin_fastapi_depencency_installer(project_info: &ProjectInfo) -> Result<()> {
    use crate::project_info::Pyo3PythonManager;

    if let Some(pyo3_python_manager) = &project_info.pyo3_python_manager {
        match pyo3_python_manager {
            Pyo3PythonManager::Uv => uv_fastapi_depencency_installer(project_info),
            Pyo3PythonManager::Setuptools => setuptools_fastapi_depencency_installer(project_info),
        }
    } else {
        bail!("No Python project mangager provided for PyO3 project");
    }
}
