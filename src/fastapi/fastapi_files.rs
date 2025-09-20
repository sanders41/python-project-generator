use std::{
    fs::{create_dir_all, File},
    path::Path,
};

use anyhow::Result;
use rayon::prelude::*;

use crate::{
    fastapi::{
        core_files::{save_config_file, save_core_utils_file, save_db_file, save_security_file},
        docker_files::{
            save_dockercompose_file, save_dockercompose_override_file,
            save_dockercompose_traefik_file, save_dockerfile, save_dockerfileignore,
        },
        model_files::save_user_models_file,
        route_files::{save_deps_file, save_health_route},
        service_files::save_db_user_services_file,
    },
    file_manager::save_file_with_content,
    project_info::{DatabaseManager, ProjectInfo},
};

pub fn generate_fastapi(project_info: &ProjectInfo) -> Result<()> {
    create_directories(project_info)?;

    [
        save_db_file,
        save_db_user_services_file,
        save_dockercompose_file,
        save_dockercompose_override_file,
        save_dockercompose_traefik_file,
        save_dockerfileignore,
        save_dockerfile,
        save_example_env_file,
        save_exceptions_file,
        save_main_file,
        save_config_file,
        save_core_utils_file,
        save_deps_file,
        save_health_route,
        save_security_file,
        save_types_file,
        save_user_models_file,
    ]
    .into_par_iter()
    .map(|f| f(project_info))
    .collect::<Result<Vec<_>, _>>()?;

    Ok(())
}

fn create_example_env_file(project_info: &ProjectInfo) -> String {
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
DOMAIN=127.0.0.1
"#
    .to_string();

    if let Some(database_manager) = &project_info.database_manager {
        if database_manager == &DatabaseManager::AsyncPg {
            info.push_str("DATABASE_URL=postgresql://${POSTGRES_USER}:${POSTGRES_PASSWORD}@${POSTGRES_HOST}:{POSTGRES_PORT}/${POSTGRES_DB}\n");
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

fn create_exceptions_file() -> String {
    r#"class DbInsertError(Exception):
    pass


class DbUpdateError(Exception):
    pass


class NoDbPoolError(Exception):
    pass


class UserNotFoundError(Exception):
    pass
"#
    .to_string()
}

fn save_exceptions_file(project_info: &ProjectInfo) -> Result<()> {
    let base = &project_info.source_dir_path();
    let file_path = base.join("exceptions.py");
    let file_content = create_exceptions_file();

    save_file_with_content(&file_path, &file_content)?;

    Ok(())
}

fn create_main_file(project_info: &ProjectInfo) -> String {
    let module = &project_info.module_name();
    format!(
        r#"from __future__ import annotations

import sys
from collections.abc import AsyncGenerator
from contextlib import asynccontextmanager

from fastapi import FastAPI
from fastapi.responses import ORJSONResponse
from loguru import logger
from starlette.middleware.cors import CORSMiddleware

from {module}.api.router import api_router
from {module}.core.config import settings
from {module}.core.db import db
from {module}.exceptions import NoDbPoolError

logger.remove()  # Remove the default logger so log level can be set
logger.add(sys.stderr, level=settings.LOG_LEVEL)


@asynccontextmanager
async def lifespan(_: FastAPI) -> AsyncGenerator:  # pragma: no cover
    logger.info("Initalizing database connection pool")
    try:
        await db.create_pool()
    except Exception as e:
        logger.error(f"Error creating db connection pool: {{e}}")
        raise

    logger.info("Saving first superuser")
    try:
        await db.create_first_superuser()
    except Exception as e:
        logger.error(f"Error creating first superuser: {{e}}")
        raise e

    yield

    logger.info("Closing database connection pool")
    try:
        await db.close_pool()
    except Exception as e:
        logger.error(f"Error closing db connection pool: {{e}}")
        raise


openapi_url = f"{{settings.API_V1_PREFIX}}/openapi.json"

app = FastAPI(
    title=settings.TITLE,
    lifespan=lifespan,
    openapi_url=openapi_url,
    default_response_class=ORJSONResponse,
)


if settings.all_cors_origins:
    app.add_middleware(
        CORSMiddleware,
        allow_origins=settings.all_cors_origins,
        allow_credentials=True,
        allow_methods=["GET", "POST", "PUT", "PATCH", "DELETE", "OPTIONS"],
        allow_headers=["Authorization", "Content-Type"],
    )

app.include_router(api_router)
"#
    )
}

fn save_main_file(project_info: &ProjectInfo) -> Result<()> {
    let base = project_info.source_dir_path();
    let file_path = base.join("main.py");
    let file_content = create_main_file(project_info);

    save_file_with_content(&file_path, &file_content)?;

    Ok(())
}

fn create_types_file() -> String {
    r#" from typing import Any, BinaryIO, Literal, NamedTuple

import asyncpg

type ActiveFilter = Literal["all", "active", "inactive"]
type Json = dict[str, Any]
"#
    .to_string()
}

fn save_types_file(project_info: &ProjectInfo) -> Result<()> {
    let base = project_info.source_dir_path();
    let file_path = base.join("types.py");
    let file_content = create_types_file();

    save_file_with_content(&file_path, &file_content)?;

    Ok(())
}

fn create_directories(project_info: &ProjectInfo) -> Result<()> {
    [
        create_api_dir,
        create_core_dir,
        create_migrations_dir,
        create_models_dir,
        create_services_dir,
    ]
    .into_par_iter()
    .map(|f| f(project_info))
    .collect::<Result<Vec<_>, _>>()?;

    Ok(())
}

fn create_api_dir(project_info: &ProjectInfo) -> Result<()> {
    let src = &project_info.source_dir_path();
    let api_dir = src.join("api");
    let routes_dir = api_dir.join("routes");
    create_dir_all(&routes_dir)?;
    save_init_file(&api_dir)?;
    save_init_file(&routes_dir)?;

    Ok(())
}

fn create_core_dir(project_info: &ProjectInfo) -> Result<()> {
    let src = &project_info.source_dir_path();
    let core_dir = src.join("core");
    create_dir_all(&core_dir)?;
    save_init_file(&core_dir)?;

    Ok(())
}

fn create_migrations_dir(project_info: &ProjectInfo) -> Result<()> {
    let base = project_info.base_dir();
    let migrations_dir = base.join("migrations");
    create_dir_all(migrations_dir)?;

    Ok(())
}

fn create_models_dir(project_info: &ProjectInfo) -> Result<()> {
    let src = &project_info.source_dir_path();
    let models_dir = src.join("models");
    create_dir_all(&models_dir)?;
    save_init_file(&models_dir)?;

    Ok(())
}

fn create_services_dir(project_info: &ProjectInfo) -> Result<()> {
    let src = &project_info.source_dir_path();
    let services_dir = src.join("services");
    let services_db_dir = services_dir.join("db");
    create_dir_all(&services_db_dir)?;
    save_init_file(&services_dir)?;
    save_init_file(&services_db_dir)?;

    Ok(())
}

fn save_init_file(path: &Path) -> Result<()> {
    let file_path = path.join("__init__.py");
    File::create(file_path)?;

    Ok(())
}
