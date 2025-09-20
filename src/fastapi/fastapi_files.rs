use std::{
    fs::{create_dir_all, File},
    path::Path,
};

use anyhow::Result;
use rayon::prelude::*;

use crate::{
    file_manager::save_file_with_content,
    project_info::{DatabaseManager, ProjectInfo, ProjectManager},
};

pub fn generate_fastapi(project_info: &ProjectInfo) -> Result<()> {
    create_directories(project_info)?;

    [
        save_example_env_file,
        save_dockerfileignore,
        save_dockerfile,
        save_main_file,
        save_config_file,
        save_core_utils_file,
        save_deps_file,
        save_health_route,
    ]
    .into_par_iter()
    .map(|f| f(project_info))
    .collect::<Result<Vec<_>, _>>()?;

    Ok(())
}

fn create_deps_file() -> String {
    r#"from __future__ import annotations

from collections.abc import AsyncGenerator, Generator
from typing import Annotated, Any, cast, TYPE_CHECKING

import jwt
from fastapi import Depends, HTTPException, Request
from fastapi.openapi.models import OAuthFlows as OAuthFlowsModel
from fastapi.security import OAuth2
from fastapi.security.utils import get_authorization_scheme_param
from jwt.exceptions import InvalidTokenError
from loguru import logger
from pydantic import ValidationError
from starlette.status import (
    HTTP_401_UNAUTHORIZED,
    HTTP_403_FORBIDDEN,
    HTTP_404_NOT_FOUND,
    HTTP_503_SERVICE_UNAVAILABLE,
)

from app.core.config import settings
from app.core.db import db
from app.core.security import ALGORITHM
from app.models.token import TokenPayload
from app.models.users import UserInDb
from app.services.db.user_services import get_user_by_id

if TYPE_CHECKING:
    import asyncpg


class OAuth2PasswordBearerWithCookie(OAuth2):
    def __init__(
        self,
        tokenUrl: str,
        scheme_name: str | None = None,
        scopes: dict[str, str] | None = None,
        description: str | None = None,
        auto_error: bool = True,
    ):
        if not scopes:
            scopes = {}
        flows = OAuthFlowsModel(password=cast(Any, {"tokenUrl": tokenUrl, "scopes": scopes}))
        super().__init__(
            flows=flows,
            scheme_name=scheme_name,
            description=description,
            auto_error=auto_error,
        )

    async def __call__(self, request: Request) -> str | None:
        authorization = request.cookies.get(  # changed to accept access token from httpOnly Cookie
            "access_token"
        )

        if authorization:
            scheme, param = get_authorization_scheme_param(authorization)
        else:  # Cookie not found, check headers.
            auth_header = request.headers.get("Authorization")
            if not auth_header:
                if self.auto_error:
                    raise HTTPException(
                        status_code=HTTP_401_UNAUTHORIZED,
                        detail="Not authenticated",
                        headers={"WWW-Authenticate": "Bearer"},
                    )

            scheme, param = get_authorization_scheme_param(auth_header)

        if scheme.lower() != "bearer":
            if self.auto_error:
                raise HTTPException(
                    status_code=HTTP_401_UNAUTHORIZED,
                    detail="Not authenticated",
                    headers={"WWW-Authenticate": "Bearer"},
                )
            else:  # pragma: no cover
                return None
        return param


reusable_oauth2 = OAuth2PasswordBearerWithCookie(
    tokenUrl=f"{settings.API_V1_PREFIX}/login/access-token"
)
TokenDep = Annotated[str, Depends(reusable_oauth2)]


async def get_db_pool() -> AsyncGenerator[asyncpg.Pool]:
    if db.db_pool is None:  # pragma: no cover
        logger.error("No database pool created")
        raise HTTPException(
            status_code=HTTP_503_SERVICE_UNAVAILABLE, detail="The database is currently unavailable"
        )

    yield db.db_pool


DbPool = Annotated[asyncpg.Pool, Depends(get_db_pool)]


async def get_current_user(pool: DbPool, token: TokenDep) -> UserInDb:
    try:
        logger.debug("Decoding JWT token")
        payload = jwt.decode(
            token, key=settings.SECRET_KEY.get_secret_value(), algorithms=[ALGORITHM]
        )
        token_data = TokenPayload(**payload)
    except (InvalidTokenError, ValidationError) as e:
        logger.debug(f"Error decoding token: {e}")
        raise HTTPException(
            status_code=HTTP_403_FORBIDDEN,
            detail="Could not validate credentials",
        ) from e
    if token_data.sub is None:  # pragma: no cover
        logger.debug("Token does not countain sub data")
        raise HTTPException(
            status_code=HTTP_403_FORBIDDEN, detail="Count not validate credientials"
        )
    user_id = token_data.sub
    user = await get_user_by_id(pool, user_id=user_id)
    if not user:  # pragma: no cover
        logger.debug("User not found")
        raise HTTPException(status_code=HTTP_404_NOT_FOUND, detail="User not found")
    if not user.is_active:
        logger.debug("User is inactive")
        raise HTTPException(status_code=HTTP_403_FORBIDDEN, detail="Inactive user")

    return user


CurrentUser = Annotated[UserInDb, Depends(get_current_user)]


def get_current_active_superuser(current_user: CurrentUser) -> UserInDb:
    if not current_user.is_superuser:
        logger.debug("The current user is not a super user")
        raise HTTPException(status_code=403, detail="The user doesn't have enough privileges")
    return current_user
"#
    .to_string()
}

fn save_deps_file(project_info: &ProjectInfo) -> Result<()> {
    let base = &project_info.source_dir_path();
    let file_path = base.join("api/deps.py");
    let file_content = create_deps_file();

    save_file_with_content(&file_path, &file_content)?;

    Ok(())
}

fn create_dockerfile(project_info: &ProjectInfo) -> String {
    let python_version = &project_info.python_version;
    let source_dir = &project_info.source_dir;
    match project_info.project_manager {
        ProjectManager::Uv => format!(
            r#"# syntax=docker/dockerfile:1

FROM ubuntu:24.04 AS builder

WORKDIR /app

ENV \
  PYTHONUNBUFFERED=true \
  UV_PYTHON_INSTALL_DIR=/opt/uv/python \
  UV_LINK_MODE=copy

# Install uv
ADD https://astral.sh/uv/install.sh /uv-installer.sh

RUN sh /uv-installer.sh && rm /uv-installer.sh

ENV PATH="/root/.local/bin:$PATH"

COPY pyproject.toml uv.lock ./

RUN --mount=type=cache,target=/root/.cache/uv \
  uv venv -p {python_version} \
  && uv sync --locked --no-dev --no-install-project --no-editable

COPY . /app

RUN --mount=type=cache,target=/root/.cache/uv \
  uv sync --locked --no-dev --no-editable


# Build production stage
FROM ubuntu:24.04 AS prod

RUN useradd appuser

WORKDIR /app

RUN chown appuser:appuser /app

ENV \
  PYTHONUNBUFFERED=true \
  PATH="/app/.venv/bin:$PATH" \
  PORT="8000"

COPY --from=builder /app/.venv /app/.venv
COPY --from=builder /app/{source_dir} /app/{source_dir}
COPY --from=builder /opt/uv/python /opt/uv/python
COPY ./scripts/entrypoint.sh /{source_dir}

RUN chmod +x /app/entrypoint.sh

EXPOSE 8000

USER appuser

ENTRYPOINT ["./entrypoint.sh"]
"#,
        ),
        _ => todo!("Implement this"),
    }
}

fn save_dockerfile(project_info: &ProjectInfo) -> Result<()> {
    let base = &project_info.base_dir();
    let file_path = base.join("Dockerfile");
    let file_content = create_dockerfile(project_info);

    save_file_with_content(&file_path, &file_content)?;

    Ok(())
}

fn create_dockerignore(project_info: &ProjectInfo) -> String {
    let mut info = r#"__pycache__
app.egg-info
*.pyc
.mypy_cache
.pytest_cache
.ruff_cache
.coverage
htmlcov
.cache
.venv
.env*
*.log
Dockerfile
.dockerignore
.git
tests
tests-results
"#
    .to_string();

    if project_info.project_manager == ProjectManager::Maturin {
        info.push_str("target\n");
    }

    info
}

fn save_dockerfileignore(project_info: &ProjectInfo) -> Result<()> {
    let base = &project_info.base_dir();
    let file_path = base.join(".dockerignore");
    let file_content = create_dockerignore(project_info);

    save_file_with_content(&file_path, &file_content)?;

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

fn create_health_route() -> String {
    r#"from __future__ import annotations

from loguru import logger

from app.api.deps import DbPool
from app.core.config import settings
from app.core.utils import APIRouter
from app.services.db.db_services import ping

router = APIRouter(tags=["Health"], prefix=f"{settings.API_V1_PREFIX}/health")


@router.get("/")
async def health(*, pool: DbPool) -> dict[str, str]:
    """Check the health of the server."""

    logger.debug("Checking health")
    health = {"server": "healthy"}

    logger.debug("Checking db health")
    try:
        await ping(pool)
        health["db"] = "healthy"
    except Exception as e:
        logger.error(f"Unable to ping the database: {e}")
        health["db"] = "unhealthy"

    return health
"#
    .to_string()
}

fn save_health_route(project_info: &ProjectInfo) -> Result<()> {
    let base = &project_info.source_dir_path();
    let file_path = base.join("api/routes/health.py");
    let file_content = create_health_route();

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

fn create_config_file() -> String {
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

    @computed_field  # type: ignore[prop-decorator]
    @property
    def all_cors_origins(self) -> list[str]:
        return [str(origin).rstrip("/") for origin in self.BACKEND_CORS_ORIGINS] + [
            self.FRONTEND_HOST
        ]

    @computed_field  # type: ignore[prop-decorator]
    @property
    def server_host(self) -> str:
        # Use HTTPS for anything other than local development
        if self.ENVIRONMENT == "local":
            return f"http://{self.DOMAIN}"
        return f"https://{self.DOMAIN}"


    def _check_default_secret(self, var_name: str, value: str | None) -> None:
        if value == "changethis":
            message = (
                f'The value of {var_name} is "changethis", '
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
    .to_string()
}

fn save_config_file(project_info: &ProjectInfo) -> Result<()> {
    let base = project_info.source_dir_path();
    let file_path = base.join("core/config.py");
    let file_content = create_config_file();

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

fn save_core_utils_file(project_info: &ProjectInfo) -> Result<()> {
    let base = project_info.source_dir_path();
    let file_path = base.join("core/utils.py");
    let file_content = create_core_utils_file();

    save_file_with_content(&file_path, &file_content)?;

    Ok(())
}

fn save_init_file(path: &Path) -> Result<()> {
    let file_path = path.join("__init__.py");
    File::create(file_path)?;

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
