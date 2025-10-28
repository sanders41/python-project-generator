use anyhow::Result;

use crate::{file_manager::save_file_with_content, project_info::ProjectInfo};

fn create_cache_user_services_file(project_info: &ProjectInfo) -> String {
    let module = &project_info.module_name();

    format!(
        r#"from __future__ import annotations

from typing import TYPE_CHECKING

import orjson

from {module}.models.users import UserInDb, UserPublic, UsersPublic

if TYPE_CHECKING:  # pragma: no cover
    from valkey.asyncio import Valkey


async def delete_all_users_public(*, cache_client: Valkey) -> None:
    keys = [key async for key in cache_client.scan_iter("users:public:*")]

    if not keys:
        return None

    await cache_client.unlink(*keys)


async def get_users_public(*, cache_client: Valkey, offset: int, limit: int) -> UsersPublic | None:
    users = await cache_client.get(name=f"users:public:{{offset}}:{{limit}}")  # type: ignore[misc]
    if not users:
        return None

    json_data = orjson.loads(users)

    return UsersPublic(
        data=[UserPublic(**user) for user in json_data["data"]],
        count=json_data["count"],
        total_users=json_data["total_users"],
    )


async def cache_users_public(
    *, cache_client: Valkey, users_public: UsersPublic, offset: int, limit: int
) -> None:
    """Cache users by page, expire cache after 1 minutes."""

    await cache_client.setex(
        name=f"users:public:{{offset}}:{{limit}}",
        time=60,
        value=orjson.dumps(users_public.model_dump()),
    )


async def cache_user(*, cache_client: Valkey, user: UserInDb) -> None:
    """Cache user, expire cache after 1 minutes."""

    await cache_client.setex(name=f"user:{{user.id}}", time=60, value=orjson.dumps(user.model_dump()))


async def delete_cached_user(*, cache_client: Valkey, user_id: str) -> None:
    await cache_client.unlink(f"user:{{user_id}}")


async def get_cached_user(*, cache_client: Valkey, user_id: str) -> UserInDb | None:
    user = await cache_client.get(name=f"user:{{user_id}}")  # type: ignore[misc]

    if not user:
        return None

    return UserInDb(**orjson.loads(user))
"#
    )
}

pub fn save_cache_user_services_file(project_info: &ProjectInfo) -> Result<()> {
    let base = &project_info.source_dir_path();
    let file_path = base.join("services/cache/user_cache_services.py");
    let file_content = create_cache_user_services_file(project_info);

    save_file_with_content(&file_path, &file_content)?;
    Ok(())
}

fn create_db_user_services_file(project_info: &ProjectInfo) -> String {
    let module = &project_info.module_name();

    format!(
        r#"from __future__ import annotations

import asyncio
from typing import TYPE_CHECKING

from loguru import logger

from {module}.core.security import get_password_hash, verify_password
from {module}.exceptions import DbInsertError, DbUpdateError, UserNotFoundError
from {module}.models.users import (
    UpdatePassword,
    UserCreate,
    UserInDb,
    UserPublic,
    UsersPublic,
    UserUpdate,
    UserUpdateMe,
)
from {module}.services.cache import user_cache_services

if TYPE_CHECKING:  # pragma: no cover
    from asyncpg import Pool
    from valkey.asyncio import Valkey

    from {module}.types import ActiveFilter


async def authenticate(*, pool: Pool, email: str, password: str) -> UserInDb | None:
    db_user = await get_user_by_email(pool=pool, email=email)

    if not db_user or not verify_password(password, db_user.hashed_password):
        return None

    return db_user


async def create_user(*, pool: Pool, cache_client: Valkey, user: UserCreate) -> UserInDb:
    query = """
    INSERT INTO users (
        email,
        full_name,
        hashed_password,
        is_active,
        is_superuser
    )
    VALUES ($1, $2, $3, $4, $5)
    RETURNING
        id::text,
        email,
        full_name,
        hashed_password,
        is_active,
        is_superuser,
        last_login
    """

    async with pool.acquire() as conn:
        async with conn.transaction():
            result = await conn.fetchrow(
                query,
                user.email,
                user.full_name,
                get_password_hash(user.password),
                user.is_active,
                user.is_superuser,
            )

    # failsafe: this shouldn't happen
    if not result:  # pragma: no cover
        raise DbInsertError("Unable to find user after inserting")

    logger.debug("Deleting cached users public")
    await user_cache_services.delete_all_users_public(cache_client=cache_client)

    return UserInDb(**dict(result))


async def delete_user(*, pool: Pool, cache_client: Valkey, user_id: str) -> None:
    query = "DELETE FROM users WHERE id::text = $1"
    async with pool.acquire() as conn:
        async with conn.transaction():
            result = await conn.execute(query, user_id)

    if result == "DELETE 0":  # pragma: no cover
        raise UserNotFoundError(f"User with id {{user_id}} not found")

    logger.debug("Deleting cached user")
    user_cache_services.delete_cached_user(cache_client=cache_client, user_id=user_id)


async def get_users(*, pool: Pool, offset: int = 0, limit: int = 100) -> list[UserInDb] | None:
    query = """
    SELECT id::text,
        email,
        full_name,
        hashed_password,
        is_active,
        is_superuser,
        last_login
    FROM users
    OFFSET $1
    LIMIT $2
    """

    async with pool.acquire() as conn:
        results = await conn.fetch(query, offset, limit)

    # Failsafe: this shouldn't happen because the first superuser always gets added at startup
    if not results:  # pragma: no cover
        return None

    return [UserInDb(**x) for x in results]


async def get_users_public(
    *,
    pool: Pool,
    cache_client: Valkey,
    offset: int = 0,
    limit: int = 100,
) -> UsersPublic:
    cached_users = await user_cache_services.get_users_public(
        cache_client=cache_client,
        offset=offset,
        limit=limit,
    )

    if cached_users:
        logger.debug("Users page found in cache, returning")
        return cached_users

    async with asyncio.TaskGroup() as tg:
        users_task = tg.create_task(get_users(pool=pool, offset=offset, limit=limit))
        total_task = tg.create_task(get_total_user_count(pool=pool))

    db_users = await users_task
    total = await total_task
    data = [UserPublic(**user.model_dump()) for user in db_users] if db_users else []
    users_public = UsersPublic(data=data, count=len(data), total_users=total)

    logger.debug("Caching users public")
    await user_cache_services.cache_users_public(
        cache_client=cache_client, users_public=users_public, offset=offset, limit=limit
    )

    return users_public


async def get_user_by_email(*, pool: Pool, email: str) -> UserInDb | None:
    query = """
    SELECT id::text,
        email,
        full_name,
        hashed_password,
        is_active,
        is_superuser,
        last_login
    FROM users
    WHERE email = $1
    """
    async with pool.acquire() as conn:
        db_user = await conn.fetchrow(query, email)

    if not db_user:
        return None

    return UserInDb(**dict(db_user))


async def get_user_public_by_email(*, pool: Pool, email: str) -> UserPublic | None:
    user = await get_user_by_email(pool=pool, email=email)
    if not user:
        return None

    return UserPublic(**user.model_dump())


async def get_user_by_id(*, pool: Pool, cache_client: Valkey, user_id: str) -> UserInDb | None:
    cached_user = await user_cache_services.get_cached_user(
        cache_client=cache_client, user_id=user_id
    )

    if cached_user:
        logger.debug("User found in cache, returning")
        return cached_user

    query = """
    SELECT id::text,
        email,
        full_name,
        hashed_password,
        is_active,
        is_superuser,
        last_login
    FROM users
    WHERE id::text = $1
    """

    async with pool.acquire() as conn:
        db_user = await conn.fetchrow(query, user_id)

    if not db_user:
        return None

    user = UserInDb(**db_user)

    logger.debug("Caching user")
    await user_cache_services.cache_user(cache_client=cache_client, user=user)

    return user


async def get_user_public_by_id(
    *,
    pool: Pool,
    cache_client: Valkey,
    user_id: str,
    active_filter: ActiveFilter = "all",
) -> UserPublic | None:
    user = await get_user_by_id(pool=pool, cache_client=cache_client, user_id=user_id)

    if not user:
        return None

    return UserPublic(**user.model_dump())


async def get_total_user_count(*, pool: Pool) -> int:
    query = """
    SELECT COUNT(*) as total_count
    FROM users
    """

    async with pool.acquire() as conn:
        result = await conn.fetchrow(query)

    return result["total_count"]


async def update_user(
    *,
    pool: Pool,
    cache_client: Valkey,
    db_user: UserInDb,
    user_in: UserUpdate | UserUpdateMe | UpdatePassword,
) -> UserInDb:
    if isinstance(user_in, UpdatePassword):
        query = """
        UPDATE users
        SET hashed_password=$1
        WHERE id::text = $2
        RETURNING
            id::text,
            email,
            full_name,
            hashed_password,
            is_active,
            is_superuser,
            last_login
        """

        async with pool.acquire() as conn:
            async with conn.transaction():
                result = await conn.fetchrow(
                    query, get_password_hash(user_in.new_password), db_user.id
                )
    else:
        user_data = user_in.model_dump(exclude_unset=True)
        if "password" in user_data:
            user_data["hashed_password"] = get_password_hash(user_data.pop("password"))
        set_clause = ", ".join([f"{{key}} = ${{i + 2}}" for i, key in enumerate(user_data.keys())])
        query = f"""
        UPDATE users
        SET {{set_clause}}
        WHERE id::text = $1
        RETURNING
            id::text,
            email,
            full_name,
            hashed_password,
            is_active,
            is_superuser,
            last_login
        """

        async with pool.acquire() as conn:
            async with conn.transaction():
                result = await conn.fetchrow(query, db_user.id, *user_data.values())

    if not result or result == "UPDATE 0":  # pragma: no cover
        raise DbUpdateError("Error updating user")

    await user_cache_services.delete_cached_user(cache_client=cache_client, user_id=db_user.id)

    return UserInDb(**dict(result))
"#
    )
}

pub fn save_db_user_services_file(project_info: &ProjectInfo) -> Result<()> {
    let base = &project_info.source_dir_path();
    let file_path = base.join("services/db/user_services.py");
    let file_content = create_db_user_services_file(project_info);

    save_file_with_content(&file_path, &file_content)?;

    Ok(())
}
