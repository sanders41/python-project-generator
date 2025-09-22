use anyhow::Result;

use crate::{file_manager::save_file_with_content, project_info::ProjectInfo};

fn create_cache_file(project_info: &ProjectInfo) -> String {
    let module = &project_info.module_name();

    format!(
        r#"from __future__ import annotations

import valkey.asyncio as valkey

from {module}.core.config import settings


class Cache:
    def __init__(self) -> None:
        self._pool: valkey.ConnectionPool | None = None
        self.client: valkey.Valkey | None = None

    async def create_client(self, *, db: int = 0) -> None:
        self._pool = await self._create_pool(db)
        self.client = valkey.Valkey.from_pool(self._pool)

    async def close_client(self) -> None:
        if self.client:
            await self.client.aclose()

        if self._pool:
            await self._pool.aclose()

    async def _create_pool(self, db: int = 0) -> valkey.ConnectionPool:
        return valkey.ConnectionPool(
            host=settings.VALKEY_HOST,
            port=settings.VALKEY_PORT,
            password=settings.VALKEY_PASSWORD.get_secret_value(),
            db=db,
        )


cache = Cache()
"#
    )
}

pub fn save_cache_file(project_info: &ProjectInfo) -> Result<()> {
    let base = &project_info.source_dir_path();
    let file_path = base.join("core/cache.py");
    let file_content = create_cache_file(project_info);

    save_file_with_content(&file_path, &file_content)?;

    Ok(())
}

fn create_config_file(project_info: &ProjectInfo) -> String {
    let project_name = &project_info.project_name;

    format!(
        r#"from __future__ import annotations

import warnings
from typing import Annotated, Any, Literal, Self

from dotenv import find_dotenv, load_dotenv
from pydantic import (
    AnyUrl,
    BeforeValidator,
    EmailStr,
    SecretStr,
    computed_field,
    model_validator,
)
from pydantic_settings import BaseSettings, SettingsConfigDict

load_dotenv(find_dotenv(".env"))


def _parse_cors(v: Any) -> list[str] | str:
    if isinstance(v, str) and not v.startswith("["):
        return [i.strip() for i in v.split(",")]
    elif isinstance(v, list | str):
        return v
    raise ValueError(v)


class Settings(BaseSettings):
    model_config = SettingsConfigDict(env_file_encoding="utf-8", extra="ignore")

    API_V1_PREFIX: str = "/api/v1"
    TITLE: str = "{project_name}"
    PRODUCTION_MODE: bool = True
    SECRET_KEY: SecretStr
    # 60 minutes * 24 hours * 8 days = 8 days
    ACCESS_TOKEN_EXPIRE_MINUTES: int = 60 * 24 * 8
    ENVIRONMENT: Literal["local", "testing", "production"] = "local"
    DOMAIN: str = "127.0.0.1"
    FIRST_SUPERUSER_EMAIL: EmailStr
    FIRST_SUPERUSER_PASSWORD: SecretStr
    FIRST_SUPERUSER_NAME: str
    BACKEND_CORS_ORIGINS: Annotated[list[AnyUrl] | str, BeforeValidator(_parse_cors)] = []
    LOG_LEVEL: Literal["DEBUG", "INFO", "WARNING", "ERROR"] = "INFO"
    POSTGRES_HOST: str = "127.0.0.1"
    POSTGRES_PORT: int = 5432
    POSTGRES_USER: str
    POSTGRES_PASSWORD: SecretStr
    POSTGRES_DB: str
    POSTGRES_POOL_MIN_SIZE: int = 10
    POSTGRES_POOL_MAX_SIZE: int = 50
    POSTGRES_POOL_ACQUIRE_TIMEOUT: int = 30
    POSTGRES_POOL_MAX_LIFETIME: int = 3600
    VALKEY_HOST: str = "127.0.0.1"
    VALKEY_PASSWORD: SecretStr
    VALKEY_PORT: int = 6379

    @computed_field  # type: ignore[prop-decorator]
    @property
    def all_cors_origins(self) -> list[str]:
        return [str(origin).rstrip("/") for origin in self.BACKEND_CORS_ORIGINS]

    @computed_field  # type: ignore[prop-decorator]
    @property
    def server_host(self) -> str:
        # Use HTTPS for anything other than local development
        if self.ENVIRONMENT == "local":
            return f"http://{{self.DOMAIN}}"
        return f"https://{{self.DOMAIN}}"


    def _check_default_secret(self, var_name: str, value: str | None) -> None:
        if value == "changethis":
            message = (
                f'The value of {{var_name}} is "changethis", '
                "for security, please change it, at least for deployments."
            )
            if self.ENVIRONMENT == "local":
                warnings.warn(message, stacklevel=1)
            else:
                raise ValueError(message)

    @model_validator(mode="after")
    def _enforce_non_default_secrets(self) -> Self:
        self._check_default_secret("SECRET_KEY", self.SECRET_KEY.get_secret_value())
        self._check_default_secret(
            "FIRST_SUPERUSER_PASSWORD", self.FIRST_SUPERUSER_PASSWORD.get_secret_value()
        )
        self._check_default_secret("POSTGRES_PASSWORD", self.POSTGRES_PASSWORD.get_secret_value())

        return self


settings = Settings()  # type: ignore
"#
    )
}

pub fn save_config_file(project_info: &ProjectInfo) -> Result<()> {
    let base = project_info.source_dir_path();
    let file_path = base.join("core/config.py");
    let file_content = create_config_file(project_info);

    save_file_with_content(&file_path, &file_content)?;

    Ok(())
}

fn create_core_utils_file() -> String {
    r#"from __future__ import annotations

from collections.abc import Callable
from typing import Any
from uuid import uuid4

from fastapi import APIRouter as FastAPIRouter
from fastapi.types import DecoratedCallable


class APIRouter(FastAPIRouter):
    """This resolves both paths that end in a / slash and those that don't.

    For example https://my_site and https://my_site/ will be routed to the same place.
    """

    def api_route(
        self, path: str, *, include_in_schema: bool = True, **kwargs: Any
    ) -> Callable[[DecoratedCallable], DecoratedCallable]:
        """Updated api_route function that automatically configures routes to have 2 versions.

        One without a trailing slash and another with it.
        """
        if path.endswith("/"):
            path = path[:-1]

        add_path = super().api_route(path, include_in_schema=include_in_schema, **kwargs)

        alternate_path = f"{path}/"
        add_alternate_path = super().api_route(alternate_path, include_in_schema=False, **kwargs)

        def decorator(func: DecoratedCallable) -> DecoratedCallable:
            add_alternate_path(func)
            return add_path(func)

        return decorator


def create_db_primary_key() -> str:
    return str(uuid4())
"#
    .to_string()
}

pub fn save_core_utils_file(project_info: &ProjectInfo) -> Result<()> {
    let base = project_info.source_dir_path();
    let file_path = base.join("core/utils.py");
    let file_content = create_core_utils_file();

    save_file_with_content(&file_path, &file_content)?;

    Ok(())
}

fn create_db_file(project_info: &ProjectInfo) -> String {
    let module = &project_info.module_name();

    format!(
        r#"from __future__ import annotations

import asyncpg
from loguru import logger

from {module}.core.config import settings
from {module}.core.security import get_password_hash
from {module}.core.utils import create_db_primary_key
from {module}.exceptions import NoDbPoolError
from {module}.services.db.user_services import get_user_by_email


class Database:
    def __init__(self, db_name: str | None = None) -> None:
        self.db_name = db_name or settings.POSTGRES_DB
        self.db_pool: asyncpg.Pool | None = None

    async def create_pool(self, min_size: int | None = None, max_size: int | None = None) -> None:
        min_size = min_size or settings.POSTGRES_POOL_MIN_SIZE
        max_size = max_size or settings.POSTGRES_POOL_MAX_SIZE

        self.db_pool = await asyncpg.create_pool(
            user=settings.POSTGRES_USER,
            password=settings.POSTGRES_PASSWORD.get_secret_value(),
            database=self.db_name,
            host=settings.POSTGRES_HOST,
            port=settings.POSTGRES_PORT,
            min_size=min_size,
            max_size=max_size,
            max_inactive_connection_lifetime=settings.POSTGRES_POOL_MAX_LIFETIME,
        )

    async def close_pool(self) -> None:
        if self.db_pool:
            await self.db_pool.close()

    async def create_first_superuser(self) -> None:
        if self.db_pool is None:  # pragma: no cover
            logger.error("No db pool created")
            raise NoDbPoolError("No db pool created")

        db_user = await get_user_by_email(pool=self.db_pool, email=settings.FIRST_SUPERUSER_EMAIL)

        if db_user:  # pragma: no cover
            if db_user.is_active and db_user.is_superuser:
                logger.debug("First super user already exists, skipping.")
                return None
            else:
                logger.info(
                    f"User with email {{settings.FIRST_SUPERUSER_EMAIL}} found, but is not active or is not a superuser, updating."
                )
                update_query = """
                UPDATE users
                SET is_active = true, is_superuser = true
                WHERE email = $1
                """

                async with self.db_pool.acquire() as conn:
                    try:
                        await conn.execute(update_query, settings.FIRST_SUPERUSER_EMAIL)
                    except asyncpg.exceptions.UniqueViolationError:
                        logger.info("first superuser already added, skipping")

                return None

        logger.debug(f"User with email {{settings.FIRST_SUPERUSER_EMAIL}} not found, adding")
        query = """
            INSERT INTO users (
              id, email, full_name, hashed_password, is_active, is_superuser
            )
            VALUES ($1, $2, $3, $4, $5, $6)
        """

        hashed_password = get_password_hash(settings.FIRST_SUPERUSER_PASSWORD.get_secret_value())
        async with self.db_pool.acquire() as conn:
            try:
                await conn.execute(
                    query,
                    create_db_primary_key(),
                    settings.FIRST_SUPERUSER_EMAIL,
                    settings.FIRST_SUPERUSER_NAME,
                    hashed_password,
                    True,
                    True,
                )
            # Check this because there could be a race condition between workers where the user wasn't
            # found by multiple workers and they all try to add it at the same time
            except asyncpg.exceptions.UniqueViolationError:  # pragma: no cover
                logger.info("First superuser already added, skipping")


db = Database()
"#
    )
}

pub fn save_db_file(project_info: &ProjectInfo) -> Result<()> {
    let base = &project_info.source_dir_path();
    let file_path = base.join("core/db.py");
    let file_content = create_db_file(project_info);

    save_file_with_content(&file_path, &file_content)?;

    Ok(())
}

fn create_security_file(project_info: &ProjectInfo) -> String {
    let module = &project_info.module_name();

    format!(
        r#"from __future__ import annotations

from datetime import UTC, datetime, timedelta

import jwt
from fastapi import Request
from pwdlib import PasswordHash
from pwdlib.hashers.argon2 import Argon2Hasher

from {module}.core.config import settings

password_hash = PasswordHash((Argon2Hasher(),))


ALGORITHM = "HS256"
_ALLOWED_PATHS = {{
    f"{{settings.API_V1_PREFIX}}/login/access-token",
    f"{{settings.API_V1_PREFIX}}/login/test-token",
    f"{{settings.API_V1_PREFIX}}/users/me/password",
    f"{{settings.API_V1_PREFIX}}/users/me",
}}


def create_access_token(subject: str, is_superuser: bool, expires_delta: timedelta) -> str:
    expire = datetime.now(UTC) + expires_delta
    to_encode = {{"exp": expire, "sub": subject, "is_superuser": is_superuser}}
    encoded_jwt = jwt.encode(
        to_encode, key=settings.SECRET_KEY.get_secret_value(), algorithm=ALGORITHM
    )
    return encoded_jwt


def verify_password(plain_password: str, hashed_password: str) -> bool:
    return password_hash.verify(plain_password, hashed_password)


def get_password_hash(password: str) -> str:
    return password_hash.hash(password)


def verify_password_changed(password_changed: bool, request: Request) -> bool:
    if not password_changed:
        if request.url.path not in _ALLOWED_PATHS:
            return False

    return True

"#
    )
}

pub fn save_security_file(project_info: &ProjectInfo) -> Result<()> {
    let base = project_info.source_dir_path();
    let file_path = base.join("core/security.py");
    let file_content = create_security_file(project_info);

    save_file_with_content(&file_path, &file_content)?;

    Ok(())
}
