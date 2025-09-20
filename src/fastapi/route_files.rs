use anyhow::Result;

use crate::{file_manager::save_file_with_content, project_info::ProjectInfo};

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

pub fn save_deps_file(project_info: &ProjectInfo) -> Result<()> {
    let base = &project_info.source_dir_path();
    let file_path = base.join("api/deps.py");
    let file_content = create_deps_file();

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

pub fn save_health_route(project_info: &ProjectInfo) -> Result<()> {
    let base = &project_info.source_dir_path();
    let file_path = base.join("api/routes/health.py");
    let file_content = create_health_route();

    save_file_with_content(&file_path, &file_content)?;

    Ok(())
}
