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
        save_dockercompose_file,
        save_dockercompose_override_file,
        save_dockercompose_traefik_file,
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

fn create_dockercompose_file(project_info: &ProjectInfo) -> String {
    let base_name = &project_info.project_slug;

    format!(
        r#"services:
  backend:
    image: {base_name}-backend:latest
    restart: unless-stopped
    networks:
      - traefik-public-{base_name}
      - default
    build:
      context: .
    container_name: {base_name}-backend
    healthcheck:
      test: ["CMD", "curl", "-f", "http://127.0.0.1:8000/api/v1/health"]
      interval: 10s
      retries: 5
      start_period: 30s
      timeout: 10s
    depends_on:
      db:
        condition: service_healthy
        restart: true
      migrations:
        condition: service_completed_successfully
    env_file:
      - .env
    environment:
      - POSTGRES_HOST=db
    labels:
      - traefik.enable=true
      - traefik.docker.network=traefik-public-{base_name}
      - traefik.constraint-label=traefik-public

      - traefik.http.services.${{STACK_NAME?Variable not set}}-backend.loadbalancer.server.port=8000

      # Rate limiting middleware
      - traefik.http.middlewares.${{STACK_NAME?Variable not set}}-api-rate-limit.ratelimit.burst=50
      - traefik.http.middlewares.${{STACK_NAME?Variable not set}}-api-rate-limit.ratelimit.average=25

      # Security headers middleware (backend-specific)
      - traefik.http.middlewares.${{STACK_NAME?Variable not set}}-security-headers.headers.contenttypenosniff=true
      - traefik.http.middlewares.${{STACK_NAME?Variable not set}}-security-headers.headers.referrerpolicy=strict-origin-when-cross-origin
      - traefik.http.middlewares.${{STACK_NAME?Variable not set}}-security-headers.headers.forcestsheader=true
      - traefik.http.middlewares.${{STACK_NAME?Variable not set}}-security-headers.headers.stsincludesubdomains=true
      - traefik.http.middlewares.${{STACK_NAME?Variable not set}}-security-headers.headers.stsseconds=31536000

      - traefik.http.routers.${{STACK_NAME?Variable not set}}-backend-http.rule=Host(`api.${{DOMAIN?Variable not set}}`)
      - traefik.http.routers.${{STACK_NAME?Variable not set}}-backend-http.entrypoints=http

      - traefik.http.routers.${{STACK_NAME?Variable not set}}-backend-https.rule=Host(`api.${{DOMAIN?Variable not set}}`) - traefik.http.routers.${{STACK_NAME?Variable not set}}-backend-https.entrypoints=https - traefik.http.routers.${{STACK_NAME?Variable not set}}-backend-https.tls=true
      - traefik.http.routers.${{STACK_NAME?Variable not set}}-backend-https.tls.certresolver=le

      # Enable redirection for HTTP and HTTPS
      - traefik.http.routers.${{STACK_NAME?Variable not set}}-backend-http.middlewares=https-redirect

      # Enable rate limiting and security headers
      - traefik.http.routers.${{STACK_NAME?Variable not set}}-backend-https.middlewares=${{STACK_NAME?Variable not set}}-api-rate-limit,${{STACK_NAME?Variable not set}}-security-headers

  db:
    image: postgres:17-alpine
    restart: unless-stopped
    container_name: {base_name}-db
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U $POSTGRES_USER -d $POSTGRES_DB"]
      interval: 10s
      retries: 5
      start_period: 30s
      timeout: 10s
    expose:
      - 5432
    env_file:
      - .env
    environment:
      - PGDATA=/var/lib/postgresql/data/pgdata
      - POSTGRES_PASSWORD=${{POSTGRES_PASSWORD?Variable not set}}
      - POSTGRES_USER=${{POSTGRES_USER?Variable not set}}
      - POSTGRES_DB=${{POSTGRES_DB?Variable not set}}
    volumes:
      - {base_name}-db-data:/var/lib/postgresql/data

  migrations:
    image: {base_name}-migrations:latest
    build:
      context: ./migration-runner
    container_name: {base_name}-migrations
    env_file:
      - .env
    environment:
      - POSTGRES_HOST=db
      - DATABASE_URL=postgresql://${{POSTGRES_USER}}:${{POSTGRES_PASSWORD}}@db:5432/${{POSTGRES_DB}}
    depends_on:
      db:
        condition: service_healthy
        restart: true
    volumes:
      - ./migrations:/migrations

volumes:
  {base_name}-db-data:

networks:
  traefik-public-{base_name}:
    name: traefik-public-{base_name}
    # Allow setting it to false for testing
    external: true
"#
    )
}

fn save_dockercompose_file(project_info: &ProjectInfo) -> Result<()> {
    let base = &project_info.base_dir();
    let file_path = base.join("docker-compose.yml");
    let file_content = create_dockercompose_file(project_info);

    save_file_with_content(&file_path, &file_content)?;

    Ok(())
}

fn create_dockercompose_override_file(project_info: &ProjectInfo) -> String {
    let base_name = &project_info.project_slug;
    let db_name = &project_info.module_name();

    format!(
        r#"services:
  proxy:
    image: traefik:3
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
    ports:
      - "80:80"
      - "8090:8080"
    # Duplicate the command from docker-compose.yml to add --api.insecure=true
    command:
      # Enable Docker in Traefik, so that it reads labels from Docker services
      - --providers.docker
      # Add a constraint to only use services with the label for this stack
      - --providers.docker.constraints=Label(`traefik.constraint-label`, `traefik-public`)
      # Do not expose all Docker services, only the ones explicitly exposed
      - --providers.docker.exposedbydefault=false
      # Create an entrypoint "http" listening on port 80
      - --entrypoints.http.address=:80
      # Enable the access log, with HTTP requests
      - --accesslog
      # Enable the Traefik log, for configurations and errors
      - --log
      # Enable debug logging for local development
      - --log.level=DEBUG
      # Enable the Dashboard and API
      - --api
      # Enable the Dashboard and API in insecure mode for local development
      - --api.insecure=true
    labels:
      # Enable Traefik for this service, to make it available in the public network
      - traefik.enable=true
      - traefik.constraint-label=traefik-public
    networks:
      - traefik-public-{base_name}
      - default

  backend:
    image: {base_name}-backend
    restart: no
    healthcheck:
      test: ["CMD", "curl", "-f", "http://127.0.0.1:8000/api/v1/health"]
      interval: 10s
      retries: 5
      start_period: 10s
      timeout: 5s
    ports:
      - "8000:8000"
    networks:
      - traefik-public-{base_name}
      - default
    build:
      context: ./backend
    container_name: {base_name}-backend
    depends_on:
      db:
        condition: service_healthy
        restart: true
    env_file:
      - .env
    environment:
      - SECRET_KEY=someKey
      - POSTGRES_HOST=db
      - POSTGRES_USER=postgres
      - POSTGRES_PASSWORD=test_password
      - POSTGRES_DB={db_name}
      - ENVIRONMENT=local
    labels:
      - traefik.enable=true
      - traefik.docker.network=traefik-public-{base_name}
      - traefik.constraint-label=traefik-public
      - traefik.http.services.${{STACK_NAME:-{base_name}}}-backend.loadbalancer.server.port=8000
      - traefik.http.routers.${{STACK_NAME:-{base_name}}}-backend-http.rule=Host(`api.127.0.0.1`)
      - traefik.http.routers.${{STACK_NAME:-{base_name}}}-backend-http.entrypoints=http
      - traefik.http.routers.${{STACK_NAME:-{base_name}}}-backend-https.rule=
      - traefik.http.routers.${{STACK_NAME:-{base_name}}}-backend-https.entrypoints=
      - traefik.http.routers.${{STACK_NAME:-{base_name}}}-backend-https.tls=
      - traefik.http.routers.${{STACK_NAME:-{base_name}}}-backend-https.tls.certresolver=
      - traefik.http.routers.${{STACK_NAME:-{base_name}}}-backend-http.middlewares=

  db:
    restart: "no"
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U $POSTGRES_USER -d $POSTGRES_DB"]
      interval: 10s
      retries: 5
      start_period: 30s
      timeout: 10s
    ports:
      - "5432:5432"

networks:
  traefik-public-{base_name}:
    # For local dev, don't expect an external Traefik network
    external: false
"#
    )
}

fn save_dockercompose_override_file(project_info: &ProjectInfo) -> Result<()> {
    let base = &project_info.base_dir();
    let file_path = base.join("docker-compose.override.yml");
    let file_content = create_dockercompose_override_file(project_info);

    save_file_with_content(&file_path, &file_content)?;

    Ok(())
}

fn create_dockercompose_traefik_file(project_info: &ProjectInfo) -> String {
    let base_name = &project_info.project_slug;

    format!(
        r#"services:
  traefik:
    image: traefik:3
    container_name: {base_name}-traefik
    ports:
      # Listen on port 80, default for HTTP, necessary to redirect to HTTPS
      - 80:80
      # Listen on port 443, default for HTTPS
      - 443:443
    restart: unless-stopped
    env_file:
      - .env
    labels:
      # Enable Traefik for this service, to make it available in the public network
      - traefik.enable=true
      # Use the traefik-public network (declared below)
      - traefik.docker.network=traefik-public
      # Define the port inside of the Docker service to use
      - traefik.http.services.traefik-dashboard.loadbalancer.server.port=8080
      # Make Traefik use this domain (from an environment variable) in HTTP
      - traefik.http.routers.traefik-dashboard-http.entrypoints=http
      - traefik.http.routers.traefik-dashboard-http.rule=Host(`traefik.${{DOMAIN?Variable not set}}`)
      # traefik-https the actual router using HTTPS
      - traefik.http.routers.traefik-dashboard-https.entrypoints=https
      - traefik.http.routers.traefik-dashboard-https.rule=Host(`traefik.${{DOMAIN?Variable not set}}`)
      - traefik.http.routers.traefik-dashboard-https.tls=true
      # Use the "le" (Let's Encrypt) resolver created below
      - traefik.http.routers.traefik-dashboard-https.tls.certresolver=le
      # Use the special Traefik service api@internal with the web UI/Dashboard
      - traefik.http.routers.traefik-dashboard-https.service=api@internal
      # https-redirect middleware to redirect HTTP to HTTPS
      - traefik.http.middlewares.https-redirect.redirectscheme.scheme=https
      - traefik.http.middlewares.https-redirect.redirectscheme.permanent=true
      # traefik-http set up only to use the middleware to redirect to https
      - traefik.http.routers.traefik-dashboard-http.middlewares=https-redirect
      # admin-auth middleware with HTTP Basic auth
      # Using the environment variables USERNAME and HASHED_PASSWORD
      - traefik.http.middlewares.admin-auth.basicauth.users=${{USERNAME?Variable not set}}:${{HASHED_PASSWORD?Variable not set}}
      # Enable HTTP Basic auth, using the middleware created above
      - traefik.http.routers.traefik-dashboard-https.middlewares=admin-auth
    volumes:
      # Add Docker as a mounted volume, so that Traefik can read the labels of other services
      - /var/run/docker.sock:/var/run/docker.sock:ro
      # Mount the volume to store the certificates
      - {base_name}-traefik-public-certificates:/certificates
    command:
      # Enable Docker in Traefik, so that it reads labels from Docker services
      - --providers.docker
      # Do not expose all Docker services, only the ones explicitly exposed
      - --providers.docker.exposedbydefault=false
      # Create an entrypoint "http" listening on port 80
      - --entrypoints.http.address=:80
      # Create an entrypoint "https" listening on port 443
      - --entrypoints.https.address=:443
      # Create the certificate resolver "le" for Let's Encrypt, uses the environment variable EMAIL
      - --certificatesresolvers.le.acme.email=${{EMAIL?Variable not set}}
      # Store the Let's Encrypt certificates in the mounted volume
      - --certificatesresolvers.le.acme.storage=/certificates/acme.json
      # Use the TLS Challenge for Let's Encrypt
      - --certificatesresolvers.le.acme.tlschallenge=true
      # Enable the access log, with HTTP requests
      - --accesslog
      # Enable the Traefik log, for configurations and errors
      - --log
      # Enable the Dashboard and API
      - --api
    networks:
      # Use the public network created to be shared between Traefik and
      # any other service that needs to be publicly available with HTTPS
      - traefik-public

volumes:
  # Create a volume to store the certificates, even if the container is recreated
  {base_name}-traefik-public-certificates:

networks:
  # Use the previously created public network "traefik-public", shared with other
  # services that need to be publicly available via this Traefik
  traefik-public:
    name: traefik-public
    external: true
"#
    )
}

fn save_dockercompose_traefik_file(project_info: &ProjectInfo) -> Result<()> {
    let base = &project_info.base_dir();
    let file_path = base.join("docker-compose.treafik.yml");
    let file_content = create_dockercompose_traefik_file(project_info);

    save_file_with_content(&file_path, &file_content)?;

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
