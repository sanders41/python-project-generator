use anyhow::Result;

use crate::{file_manager::save_file_with_content, project_info::ProjectInfo};

fn create_deps_file(project_info: &ProjectInfo) -> String {
    let module = &project_info.module_name();

    format!(
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

from {module}.core.config import settings
from {module}.core.db import db
from {module}.core.security import ALGORITHM
from {module}.models.token import TokenPayload
from {module}.models.users import UserInDb
from {module}.services.db.user_services import get_user_by_id

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
            scopes = {{}}
        flows = OAuthFlowsModel(password=cast(Any, {{"tokenUrl": tokenUrl, "scopes": scopes}}))
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
                        headers={{"WWW-Authenticate": "Bearer"}},
                    )

            scheme, param = get_authorization_scheme_param(auth_header)

        if scheme.lower() != "bearer":
            if self.auto_error:
                raise HTTPException(
                    status_code=HTTP_401_UNAUTHORIZED,
                    detail="Not authenticated",
                    headers={{"WWW-Authenticate": "Bearer"}},
                )
            else:  # pragma: no cover
                return None
        return param


reusable_oauth2 = OAuth2PasswordBearerWithCookie(
    tokenUrl=f"{{settings.API_V1_PREFIX}}/login/access-token"
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
        logger.debug(f"Error decoding token: {{e}}")
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
    )
}

pub fn save_deps_file(project_info: &ProjectInfo) -> Result<()> {
    let base = &project_info.source_dir_path();
    let file_path = base.join("api/deps.py");
    let file_content = create_deps_file(project_info);

    save_file_with_content(&file_path, &file_content)?;

    Ok(())
}

fn create_health_route(project_info: &ProjectInfo) -> String {
    let module = &project_info.module_name();

    format!(
        r#"from __future__ import annotations

from loguru import logger

from {module}.api.deps import DbPool
from {module}.core.config import settings
from {module}.core.utils import APIRouter
from {module}.services.db.db_services import ping

router = APIRouter(tags=["Health"], prefix=f"{{settings.API_V1_PREFIX}}/health")


@router.get("/")
async def health(*, pool: DbPool) -> dict[str, str]:
    """Check the health of the server."""

    logger.debug("Checking health")
    health = {{"server": "healthy"}}

    logger.debug("Checking db health")
    try:
        await ping(pool)
        health["db"] = "healthy"
    except Exception as e:
        logger.error(f"Unable to ping the database: {{e}}")
        health["db"] = "unhealthy"

    return health
"#
    )
}

pub fn save_health_route(project_info: &ProjectInfo) -> Result<()> {
    let base = &project_info.source_dir_path();
    let file_path = base.join("api/routes/health.py");
    let file_content = create_health_route(project_info);

    save_file_with_content(&file_path, &file_content)?;

    Ok(())
}

fn create_login_route(project_info: &ProjectInfo) -> String {
    let module = &project_info.module_name();

    format!(
        r#"from __future__ import annotations

from datetime import timedelta
from typing import Annotated

from fastapi import Depends, HTTPException, Response
from fastapi.security import OAuth2PasswordRequestForm
from loguru import logger
from starlette.status import (
    HTTP_400_BAD_REQUEST,
    HTTP_401_UNAUTHORIZED,
    HTTP_500_INTERNAL_SERVER_ERROR,
)

from {module}.api.deps import CurrentUser, DbPool
from {module}.core import security
from {module}.core.config import settings
from {module}.core.utils import APIRouter
from {module}.models.token import Token
from {module}.models.users import UserPublic
from {module}.services.db import user_services

router = APIRouter(tags=["Login"], prefix=f"{{settings.API_V1_PREFIX}}")


@router.post("/login/access-token")
async def login_access_token(
    *, response: Response, pool: DbPool, form_data: Annotated[OAuth2PasswordRequestForm, Depends()]
) -> Token:
    """OAuth2 compatible token login, get an access token for future requests."""

    logger.debug("Authenticating user")
    user = await user_services.authenticate(
        pool, email=form_data.username, password=form_data.password
    )

    if not user:
        logger.debug("Incorrect email or password")
        raise HTTPException(status_code=HTTP_400_BAD_REQUEST, detail="Incorrect email or password")
    elif not user.is_active:
        logger.debug("Inactive user")
        raise HTTPException(status_code=HTTP_401_UNAUTHORIZED, detail="Inactive user")
    access_token_expires = timedelta(minutes=settings.ACCESS_TOKEN_EXPIRE_MINUTES)

    access_token = security.create_access_token(
        str(user.id), user.is_superuser, expires_delta=access_token_expires
    )

    response.set_cookie(
        key="access_token",
        value=f"Bearer {{access_token}}",
        httponly=True,
        secure=settings.PRODUCTION_MODE,
    )

    return Token(access_token=access_token)


@router.post("/login/test-token")
async def test_token(*, db_pool: DbPool, current_user: CurrentUser) -> UserPublic:
    """Test access token."""

    try:
        user_public = await user_services.get_user_public_by_id(
            pool=db_pool, user_id=current_user.id
        )
    except Exception as e:  # pragma: no cover
        logger.error(f"An error occurred while testing the user token: {{e}}")
        raise HTTPException(
            status_code=HTTP_500_INTERNAL_SERVER_ERROR,
            detail="An error occurred while testing the user token",
        ) from e

    if user_public is None:  # pragma: no cover
        raise HTTPException(status_code=HTTP_401_UNAUTHORIZED, detail="Not authorized")

    return user_public
"#
    )
}

pub fn save_login_route(project_info: &ProjectInfo) -> Result<()> {
    let base = &project_info.source_dir_path();
    let file_path = base.join("api/routes/login.py");
    let file_content = create_login_route(project_info);

    save_file_with_content(&file_path, &file_content)?;

    Ok(())
}

fn create_router_file() -> String {
    r#"from {module}.api.routes import (
    health,
    login,
    users,
    version,
)
from {module}.core.utils import APIRouter

api_router = APIRouter()
api_router.include_router(health.router)
api_router.include_router(login.router)
api_router.include_router(users.router)
api_router.include_router(version.router)

"#
    .to_string()
}

pub fn save_router_file(project_info: &ProjectInfo) -> Result<()> {
    let base = &project_info.source_dir_path();
    let file_path = base.join("api/router.py");
    let file_content = create_router_file();

    save_file_with_content(&file_path, &file_content)?;

    Ok(())
}

fn create_users_route(project_info: &ProjectInfo) -> String {
    let module = project_info.module_name();

    format!(
        r#"from __future__ import annotations

import asyncio

from fastapi import Depends, HTTPException
from loguru import logger
from starlette.status import (
    HTTP_204_NO_CONTENT
    HTTP_400_BAD_REQUEST,
    HTTP_401_UNAUTHORIZED,
    HTTP_403_FORBIDDEN,
    HTTP_404_NOT_FOUND,
    HTTP_409_CONFLICT,
    HTTP_500_INTERNAL_SERVER_ERROR,
)

from {module}.api.deps import CurrentUser, DbPool, get_current_active_superuser
from {module}.core.config import settings
from {module}.core.security import verify_password
from {module}.core.utils import APIRouter
from {module}.models.message import Message
from {module}.models.users import (
    UpdatePassword,
    UserCreate,
    UserPublic,
    UsersPublic,
    UserUpdate,
    UserUpdateMe,
)
from {module}.services.db import user_services
from {module}.types import ActiveFilter

router = APIRouter(tags=["Users"], prefix=f"{{settings.API_V1_PREFIX}}/users")


@router.get("/", dependencies=[Depends(get_current_active_superuser)])
async def read_users(
    *,
    db_pool: DbPool,
    active_filter: ActiveFilter = "active",
    offset: int = 0,
    limit: int = 100,
) -> UsersPublic:
    """Retrieve users.

    Administrator rights required.
    """

    logger.debug(f"Getting users with offset {{offset}} and limit {{limit}}")
    try:
        async with asyncio.TaskGroup() as tg:
            users_task = tg.create_task(
                user_services.get_users_public(
                    pool=db_pool,
                    active_filter=active_filter,
                    offset=offset,
                    limit=limit,
                )
            )
            total_user_count_task = tg.create_task(
                user_services.get_total_user_count(db_pool)
            )

        users = await users_task
        total_user_count = await total_user_count_task
    except* Exception as eg:  # pragma: no cover
        for e in eg.exceptions:
            logger.error(f"An error occurred while retrieving users: {{e}}")

        raise HTTPException(
            status_code=HTTP_500_INTERNAL_SERVER_ERROR,
            detail="An error occurred while retrieving users",
        ) from eg

    count = len(users) if users else 0
    data = users if users else []

    return UsersPublic(data=data, count=count, total_users=total_user_count)


@router.post("/")
async def create_user(
    *,
    active_filter: ActiveFilter = "active",
    db_pool: DbPool,
    user_in: UserCreate,
) -> UserPublic:
    """Create a new user."""

    logger.debug("Creating new user")
    try:
        user = await user_services.get_user_by_email(db_pool, email=user_in.email)
    except Exception as e:  # pragma: no cover
        logger.error(
            f"An error occurred while checking if the email {{user_in.email}} already exists for creating a user: {{e}}"
        )
        raise HTTPException(
            status_code=HTTP_500_INTERNAL_SERVER_ERROR,
            detail="An error occurred while creating the user.",
        ) from e

    if user:
        logger.debug(f"User with email address {{user_in.email}} already exists")
        raise HTTPException(
            status_code=HTTP_400_BAD_REQUEST,
            detail="A user with this email address already exists in the system",
        )

    try:
        created_user = await user_services.create_user(
            db_pool, user=user_in
        )
    except Exception as e:  # pragma: no cover
        logger.error(
            f"An error occurred while creating the user with email address {{user_in.email}}: {{e}}"
        )
        raise HTTPException(
            status_code=HTTP_500_INTERNAL_SERVER_ERROR,
            detail="An error occurred while creating the user",
        ) from e

    try:
        user_public = await user_services.get_user_public_by_id(
            pool=db_pool,
            user_id=created_user.id,
            active_filter=active_filter,
        )
    except Exception as e:  # pragma: no cover
        logger.error(f"An error occurred while creating the user: {{e}}")
        raise HTTPException(
            status_code=HTTP_500_INTERNAL_SERVER_ERROR,
            detail="An error occurred while creating the user",
        ) from e

    if user_public is None:  # pragma: no cover
        logger.error(f"User with id {{created_user.id}} not found after creation")
        raise HTTPException(
            status_code=HTTP_500_INTERNAL_SERVER_ERROR,
            detail="An error occurred while creating the user",
        )

    return user_public


@router.patch("/me")
async def update_user_me(
    *,
    active_filter: ActiveFilter = "active",
    db_pool: DbPool,
    user_in: UserUpdateMe,
    current_user: CurrentUser,
) -> UserPublic:
    """Update own user."""

    logger.debug("Updating current user")
    if user_in.email:
        try:
            existing_user = await user_services.get_user_by_email(db_pool, email=user_in.email)
        except Exception as e:  # pragma: no cover
            logger.error(
                f"An error occurred while updating me, checking if the email already exists: {{e}}"
            )
            raise HTTPException(
                status_code=HTTP_500_INTERNAL_SERVER_ERROR,
                detail="An error occurred while updating the user",
            ) from e

        if existing_user and existing_user.id != current_user.id:
            logger.debug(f"User with email address {{user_in.email}} already exists")
            raise HTTPException(
                status_code=HTTP_409_CONFLICT,
                detail="A user with this email address already exists",
            )

    try:
        updated_user = await user_services.update_user(
            pool=db_pool, db_user=current_user, user_in=user_in
        )
    except Exception as e:  # pragma: no cover
        logger.error(f"An error occurred while updating me: {{e}}")
        raise HTTPException(
            status_code=HTTP_500_INTERNAL_SERVER_ERROR,
            detail="An error occurred while updating the user",
        ) from e

    try:
        user_public = await user_services.get_user_public_by_id(
            pool=db_pool,
            user_id=updated_user.id,
            active_filter=active_filter,
        )
    except Exception as e:  # pragma: no cover
        logger.error(f"Error updating user: {{e}}")
        raise HTTPException(
            status_code=HTTP_500_INTERNAL_SERVER_ERROR,
            detail="An error occurred while updating the user",
        ) from e

    if user_public is None:  # pragma: no cover
        logger.error(f"User with id {{updated_user.id}} not found after update")
        raise HTTPException(
            status_code=HTTP_500_INTERNAL_SERVER_ERROR,
            detail="An error occurred while updating the user",
        )

    return user_public


@router.patch("/me/password", status_code=HTTP_204_NO_CONTENT)
async def update_password_me(
    *,
    db_pool: DbPool,
    user_in: UpdatePassword,
    current_user: CurrentUser,
) -> None:
    """Update own password."""

    if not verify_password(user_in.current_password, current_user.hashed_password):
        logger.debug("Passwords do not match")
        raise HTTPException(status_code=HTTP_400_BAD_REQUEST, detail="Incorrect password")
    if user_in.current_password == user_in.new_password:
        logger.debug("Password not changed")
        raise HTTPException(
            status_code=HTTP_400_BAD_REQUEST,
            detail="New password cannot be the same as the current one",
        )

    try:
        logger.debug("Updating password")
        await user_services.update_user(
            pool=db_pool, db_user=current_user, user_in=user_in
        )
    except Exception as e:  # pragma: no cover
        logger.error(f"An error occurred updating the password: {{e}}")
        raise HTTPException(
            status_code=HTTP_500_INTERNAL_SERVER_ERROR,
            detail="An error occurred while updating the password",
        ) from e


@router.get("/me")
async def read_user_me(
    *,
    db_pool: DbPool,
    current_user: CurrentUser,
    active_filter: ActiveFilter = "active",
) -> UserPublic:
    """Get current user."""

    try:
        user_public = await user_services.get_user_public_by_id(
            pool=db_pool,
            user_id=current_user.id,
            active_filter=active_filter,
        )
    except Exception as e:  # pragma: no cover
        logger.error(f"Error reading user me: {{e}}")
        raise HTTPException(
            status_code=HTTP_500_INTERNAL_SERVER_ERROR,
            detail="An error occurred while getting the user",
        ) from e

    # Fail safe, shouldn't be possible to get here
    if user_public is None:  # pragma: no cover
        logger.debug("User not found")
        raise HTTPException(status_code=HTTP_404_NOT_FOUND, detail="User not found")

    return user_public


@router.delete("/me", status_code=HTTP_204_NO_CONTENT)
async def delete_user_me(
    *, db_pool: DbPool, current_user: CurrentUser
) -> None:
    """Delete own user."""

    logger.debug("Deleting current user")
    if current_user.is_superuser:
        logger.debug("Super users are not allowed to delete themselves")
        raise HTTPException(
            status_code=HTTP_400_BAD_REQUEST,
            detail="Super users are not allowed to delete themselves",
        )

    try:
        await user_services.delete_user(pool=db_pool, user_id=current_user.id)
    except Exception as e:  # pragma: no cover
        logger.error(f"An error occurred while deleting the user: {{e}}")
        raise HTTPException(
            status_code=HTTP_500_INTERNAL_SERVER_ERROR,
            detail="An error occurred while deleting the user",
        ) from e


@router.get("/{{user_id}}")
async def read_user_by_id(
    *,
    db_pool: DbPool,
    user_id: str,
    current_user: CurrentUser,
    active_filter: ActiveFilter = "active",
) -> UserPublic:
    """Get a specific user by id."""

    stripped_user_id = user_id.strip()
    logger.debug(f"Getting user with id {{stripped_user_id}}")
    try:
        user = await user_services.get_user_public_by_id(
            pool=db_pool,
            user_id=stripped_user_id,
            active_filter=active_filter,
        )
    except Exception as e:  # pragma: no cover
        logger.error(f"An error occurred while retrieving user with id {{stripped_user_id}}: {{e}}")
        raise HTTPException(
            status_code=HTTP_500_INTERNAL_SERVER_ERROR,
            detail="An error occurred while retrieving the user",
        ) from e

    if user is None:
        logger.debug(f"User with id {{stripped_user_id}} not found")
        raise HTTPException(
            status_code=HTTP_404_NOT_FOUND,
            detail="The user with this id does not exist in the system",
        )

    if user.id == current_user.id:
        return user
    if not current_user.is_superuser:
        logger.debug("Current user is not an admin and does not have enough privileges to get user")
        raise HTTPException(
            status_code=HTTP_403_FORBIDDEN,
            detail="The user doesn't have enough privileges",
        )
    return user


@router.get("/name/{{name}}", dependencies=[Depends(get_current_active_superuser)])
async def read_user_by_name(
    *,
    db_pool: DbPool,
    name: str,
    active_filter: ActiveFilter = "active",
) -> list[UserPublic]:
    """Get users by name."""

    stripped_name = name.strip()
    logger.debug(f"Getting users with name {{stripped_name}}")
    try:
        users = await user_services.get_users_public_by_name(
            pool=db_pool,
            name=stripped_name,
            active_filter=active_filter,
        )
    except* Exception as eg:  # pragma: no cover
        for e in eg.exceptions:  # type: ignore[assignment]
            logger.error(f"An error occurred while retrieving users: {{e}}")

        raise HTTPException(
            status_code=HTTP_500_INTERNAL_SERVER_ERROR,
            detail="An error occurred while retrieving users",
        ) from eg

    if users is None:
        logger.debug(f"User with id {{stripped_name}} not found")
        raise HTTPException(
            status_code=HTTP_404_NOT_FOUND,
            detail="User with this name does not exist in the system",
        )

    return users


@router.patch(
    "/{{user_id}}",
    dependencies=[Depends(get_current_active_superuser)],
)
async def update_user(
    *,
    db_pool: DbPool,
    user_id: str,
    user_in: UserUpdate,
    active_filter: ActiveFilter = "active",
) -> UserPublic:
    """Update a user.

    Administrator rights required.
    """

    stripped_user_id = user_id.strip()
    logger.debug(f"Updating user {{user_id}}")
    try:
        db_user = await user_services.get_user_by_id(db_pool, user_id=stripped_user_id)
    except Exception as e:  # pragma: no cover
        logger.error(f"An error occurred while retrieving user {{user_id}} for updating: {{e}}")
        raise HTTPException(
            status_code=HTTP_500_INTERNAL_SERVER_ERROR,
            detail="An error occurred while retrieving the user for updating",
        ) from e

    if not db_user:
        logger.debug(f"User with id {{stripped_user_id}} not found")
        raise HTTPException(
            status_code=HTTP_404_NOT_FOUND,
            detail="The user with this id does not exist in the system",
        )
    if user_in.email:
        existing_user = await user_services.get_user_by_email(db_pool, email=user_in.email)
        if existing_user and existing_user.id != user_id:
            logger.debug(f"A user with email {{user_in.email}} already exists")
            raise HTTPException(
                status_code=HTTP_409_CONFLICT, detail="User with this email already exists"
            )

    try:
        if user_in.password:
            db_user = await user_services.update_user(
                pool=db_pool,
                db_user=db_user,
                user_in=user_in,
            )
        else:
            db_user = await user_services.update_user(
                pool=db_pool, db_user=db_user, user_in=user_in
            )
    except Exception as e:  # pragma: no cover
        logger.error(f"An error occurred while updating user {{stripped_user_id}}: {{e}}")
        raise HTTPException(
            status_code=HTTP_500_INTERNAL_SERVER_ERROR,
            detail="An error occurred while updating the user",
        ) from e

    try:
        user_public = await user_services.get_user_public_by_id(
            pool=db_pool, user_id=db_user.id, active_filter=active_filter
        )
    except Exception as e:  # pragma: no cover
        logger.error(f"Error updating the user: {{e}}")
        raise HTTPException(
            status_code=HTTP_500_INTERNAL_SERVER_ERROR,
            detail="An error occurred while updating the user",
        ) from e

    if user_public is None:  # pragma: no cover
        logger.error(f"User with id {{db_user.id}} not found after updating")
        raise HTTPException(
            status_code=HTTP_500_INTERNAL_SERVER_ERROR,
            detail="An error occurred while updating the user",
        )

    return user_public


@router.delete("/{{user_id}}", dependencies=[Depends(get_current_active_superuser)])
async def delete_user(
    *, db_pool: DbPool, current_user: CurrentUser, user_id: str
) -> Message:
    """Delete a user.

    Administrator rights required.
    """

    stripped_user_id = user_id.strip()
    logger.debug(f"Deleting user with id {{stripped_user_id}}")
    try:
        user = await user_services.get_user_by_id(db_pool, user_id=stripped_user_id)
    except Exception as e:  # pragma: no cover
        logger.error(f"An error occurred while retrieving user {{user_id}} for deleting: {{e}}")
        raise HTTPException(
            status_code=HTTP_500_INTERNAL_SERVER_ERROR,
            detail="An error occurred while retrieving the user for deleting",
        ) from e

    if not user:
        logger.debug(f"User with id {{stripped_user_id}} not found")
        raise HTTPException(status_code=HTTP_404_NOT_FOUND, detail="User not found")
    if user == current_user:
        logger.debug("Super users are not allowed to delete themselves")
        raise HTTPException(
            status_code=HTTP_403_FORBIDDEN,
            detail="Super users are not allowed to delete themselves",
        )
    try:
        await user_services.delete_user(pool=db_pool, user_id=stripped_user_id)
    except Exception as e:  # pragma: no cover
        logger.error(f"An error occurred while delete the user: {{e}}")
        raise HTTPException(
            status_code=HTTP_500_INTERNAL_SERVER_ERROR,
            detail="An error occurred while deleting the user",
        ) from e
    return Message(message="User deleted successfully")
"#
    )
}

pub fn save_users_route(project_info: &ProjectInfo) -> Result<()> {
    let base = &project_info.source_dir_path();
    let file_path = base.join("api/routes/users.py");
    let file_content = create_users_route(project_info);

    save_file_with_content(&file_path, &file_content)?;

    Ok(())
}

fn create_version_route(project_info: &ProjectInfo) -> String {
    let module = project_info.module_name();

    format!(
        r#"from __future__ import annotations

from {module} import __version__
from {module}.core.config import settings
from {module}.core.utils import APIRouter

router = APIRouter(tags=["Version"], prefix=f"{{settings.API_V1_PREFIX}}/version")


@router.get("/")
async def read_version() -> dict[str, str]:
    """Get the current backend software version."""

    return {{"version": __version__}}

"#
    )
}

pub fn save_version_route(project_info: &ProjectInfo) -> Result<()> {
    let base = &project_info.source_dir_path();
    let file_path = base.join("api/routes/version.py");
    let file_content = create_version_route(project_info);

    save_file_with_content(&file_path, &file_content)?;

    Ok(())
}
