use anyhow::Result;

use crate::{file_manager::save_file_with_content, project_info::ProjectInfo};

fn create_db_user_services_file() -> String {
    r#"from __future__ import annotations

from typing import TYPE_CHECKING

from loguru import logger

from app.core.security import get_password_hash, verify_password
from app.core.utils import create_db_primary_key
from app.exceptions import DbInsertError, DbUpdateError, UserNotFoundError
from app.models.users import (
    UpdatePassword,
    UserCreate,
    UserInDb,
    UserPublic,
    UserUpdate,
    UserUpdateMe,
)

if TYPE_CHECKING:  # pragma: no cover
    from asyncpg import Pool

    from app.types import ActiveFilter, SortOrder


async def authenticate(*, pool: Pool, email: str, password: str) -> UserInDb | None:
    db_user = await get_user_by_email(pool, email=email)

    if not db_user or not verify_password(password, db_user.hashed_password):
        return None

    return db_user


async def create_user(*, pool: Pool, user: UserCreate) -> UserInDb:
    query = """
    INSERT INTO users (
        id,
        email,
        full_name,
        hashed_password,
        is_active,
        is_superuser
    )
    VALUES ($1, $2, $3, $4, $5, $6)
    RETURNING
        id,
        email,
        full_name,
        hashed_password,
        is_active,
        is_superuser,
        last_login
    """

    async with pool.acquire() as conn:
        result = await conn.fetchrow(
            query,
            create_db_primary_key(),
            user.email,
            user.full_name,
            get_password_hash(user.password),
            user.is_active,
            user.is_superuser,
        )

    # failsafe: this shouldn't happen
    if not result:  # pragma: no cover
        raise DbInsertError("Unable to find user after inserting")

    return UserInDb(**dict(result))


async def delete_user(*, pool: Pool, user_id: str) -> None:
    query = "DELETE FROM users WHERE id = $1"
    async with pool.acquire() as conn:
        result = await conn.execute(query, user_id)

    if result == "DELETE 0":
        raise UserNotFoundError(f"Study with id {user_id} not found")


async def get_users(
    *, pool: Pool, offset: int = 0, limit: int = 100
) -> list[UserInDb] | None:
    if sort_order not in ("asc", "desc"):
        raise ValueError("Invalid sort order")

    query = f"""
    SELECT id,
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
    active_filter: ActiveFilter = "all",
    offset: int = 0,
    limit: int = 100,
) -> list[UserPublic] | None:
    db_users = await get_users(pool, offset=offset, limit=limit)
    if not db_users:
        return None

    return [UserPublic(**users.model_dump()) for user in db_users]


async def get_user_by_email(*, pool: Pool, email: str) -> UserInDb | None:
    query = """
    SELECT id,
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


async def get_user_public_by_email(
    *, pool: Pool, email: str, active_filter: ActiveFilter = "all"
) -> UserPublic | None:
    user = await get_user_by_email(pool=pool, email=email)
    if not user:
        return None

    return UserPublic(**user.model_dump())


async def get_user_by_id(*, pool: Pool, user_id: str) -> UserInDb | None:
    query = """
    SELECT id,
        email,
        full_name,
        hashed_password,
        is_active,
        is_superuser,
        last_login
    FROM users
    WHERE id = $1
    """

    async with pool.acquire() as conn:
        db_user = await conn.fetchrow(query, user_id)

    if not db_user:
        return None

    return UserInDb(**db_user)


async def get_user_public_by_id(
    pool: Pool,
    *,
    user_id: str,
    active_filter: ActiveFilter = "all",
) -> UserPublic | None:
    user = await get_user_by_id(pool=pool, user_id=user_id)

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
    pool: Pool,
    *,
    db_user: UserInDb,
    user_in: UserUpdate | UserUpdateMe | UpdatePassword,
) -> UserInDb:
    if isinstance(user_in, UpdatePassword):
        query = """
        UPDATE users
        SET hashed_password=$1
        WHERE id = $2
        RETURNING
            id,
            email,
            full_name,
            hashed_password,
            is_active,
            is_superuser,
            last_login
        """

        async with pool.acquire() as conn:
            result = await conn.fetchrow(
                query, get_password_hash(user_in.new_password), db_user.id
            )
    else:
        user_data = user_in.model_dump(exclude_unset=True)
        if "password" in user_data:
            user_data["hashed_password"] = get_password_hash(user_data.pop("password"))
        set_clause = ", ".join([f"{key} = ${i + 2}" for i, key in enumerate(user_data.keys())])
        query = f"""
        UPDATE users
        SET {set_clause}
        WHERE id = $1
        RETURNING
            id,
            email,
            full_name,
            hashed_password,
            is_active,
            is_superuser,
            last_login
        """

        async with pool.acquire() as conn:
            result = await conn.fetchrow(query, db_user.id, *user_data.values())

    if not result or result == "UPDATE 0":  # pragma: no cover
        raise DbUpdateError("Error updating user")

    return UserInDb(**dict(result))
"#
    .to_string()
}

pub fn save_db_user_services_file(project_info: &ProjectInfo) -> Result<()> {
    let base = &project_info.source_dir_path();
    let file_path = base.join("services/db/user_services.py");
    let file_content = create_db_user_services_file();

    save_file_with_content(&file_path, &file_content)?;

    Ok(())
}
