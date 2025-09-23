use anyhow::Result;

use crate::{file_manager::save_file_with_content, project_info::ProjectInfo};

fn create_config_test_file(project_info: &ProjectInfo) -> String {
    let module = &project_info.module_name();

    format!(
        r#"import pytest
from pydantic import AnyUrl, SecretStr

from {module}.core.config import Settings


def test_check_default_secret_production():
    with pytest.raises(ValueError):
        Settings(
            FIRST_SUPERUSER_EMAIL="user@email.com",
            FIRST_SUPERUSER_PASSWORD=SecretStr("Abc$123be"),
            FIRST_SUPERUSER_NAME="Some Name",
            POSTGRES_HOST="http://localhost",
            POSTGRES_USER="postgres",
            POSTGRES_PASSWORD=SecretStr("Somepassword!"),
            VALKEY_HOST="http://localhost",
            VALKEY_PASSWORD=SecretStr("Somepassword!"),
            ENVIRONMENT="production",
            SECRET_KEY=SecretStr("changethis"),
        )


def test_check_default_secret_testing():
    with pytest.raises(ValueError):
        Settings(
            FIRST_SUPERUSER_EMAIL="user@email.com",
            FIRST_SUPERUSER_PASSWORD=SecretStr("Abc$123be"),
            FIRST_SUPERUSER_NAME="Some Name",
            POSTGRES_HOST="http://localhost",
            POSTGRES_USER="postgres",
            POSTGRES_PASSWORD=SecretStr("Somepassword!"),
            VALKEY_HOST="http://localhost",
            VALKEY_PASSWORD=SecretStr("Somepassword!"),
            ENVIRONMENT="testing",
            SECRET_KEY=SecretStr("changethis"),
        )


def test_check_default_secret_local():
    with pytest.warns(
        UserWarning,
        match='The value of SECRET_KEY is "changethis", for security, please change it, at least for deployments.',
    ):
        Settings(
            FIRST_SUPERUSER_EMAIL="user@email.com",
            FIRST_SUPERUSER_PASSWORD=SecretStr("Abc$123be"),
            FIRST_SUPERUSER_NAME="Some Name",
            POSTGRES_HOST="http://localhost",
            POSTGRES_USER="postgres",
            POSTGRES_PASSWORD=SecretStr("Somepassword!"),
            VALKEY_HOST="http://localhost",
            VALKEY_PASSWORD=SecretStr("Somepassword!"),
            ENVIRONMENT="local",
            SECRET_KEY=SecretStr("changethis"),
        )


def test_serer_host_production():
    settings = Settings(
        FIRST_SUPERUSER_EMAIL="user@email.com",
        FIRST_SUPERUSER_PASSWORD=SecretStr("Abc$123be"),
        FIRST_SUPERUSER_NAME="Some Name",
        POSTGRES_HOST="http://localhost",
        POSTGRES_USER="postgres",
        POSTGRES_PASSWORD=SecretStr("Somepassword!"),
        VALKEY_HOST="http://localhost",
        VALKEY_PASSWORD=SecretStr("Somepassword!"),
        SECRET_KEY=SecretStr("Somesecretkey"),
        ENVIRONMENT="production",
    )

    assert settings.server_host == f"https://{{settings.DOMAIN}}"


def test_serer_host_testing():
    settings = Settings(
        FIRST_SUPERUSER_EMAIL="user@email.com",
        FIRST_SUPERUSER_PASSWORD=SecretStr("Abc$123be"),
        FIRST_SUPERUSER_NAME="Some Name",
        POSTGRES_HOST="http://localhost",
        POSTGRES_USER="postgres",
        POSTGRES_PASSWORD=SecretStr("Somepassword!"),
        VALKEY_HOST="http://localhost",
        VALKEY_PASSWORD=SecretStr("Somepassword!"),
        SECRET_KEY=SecretStr("Somesecretkey"),
        ENVIRONMENT="testing",
    )

    assert settings.server_host == f"https://{{settings.DOMAIN}}"


def test_serer_host_local():
    settings = Settings(
        FIRST_SUPERUSER_EMAIL="user@email.com",
        FIRST_SUPERUSER_PASSWORD=SecretStr("Abc$123be"),
        FIRST_SUPERUSER_NAME="Some Name",
        POSTGRES_HOST="http://localhost",
        POSTGRES_USER="postgres",
        POSTGRES_PASSWORD=SecretStr("Somepassword!"),
        VALKEY_HOST="http://localhost",
        VALKEY_PASSWORD=SecretStr("Somepassword!"),
        SECRET_KEY=SecretStr("Somesecretkey"),
        ENVIRONMENT="local",
    )

    assert settings.server_host == f"http://{{settings.DOMAIN}}"


def test_parse_cors_error():
    with pytest.raises(ValueError):
        Settings(
            FIRST_SUPERUSER_EMAIL="user@email.com",
            FIRST_SUPERUSER_PASSWORD=SecretStr("Abc$123be"),
            FIRST_SUPERUSER_NAME="Some Name",
            POSTGRES_HOST="http://localhost",
            POSTGRES_USER="postgres",
            POSTGRES_PASSWORD=SecretStr("Somepassword!"),
            VALKEY_HOST="http://localhost",
            VALKEY_PASSWORD=SecretStr("Somepassword!"),
            SECRET_KEY=SecretStr("Somesecretkey"),
            BACKEND_CORS_ORIGINS=1,  # type: ignore
        )


def test_parse_cors_string():
    settings = Settings(
        FIRST_SUPERUSER_EMAIL="user@email.com",
        FIRST_SUPERUSER_PASSWORD=SecretStr("Abc$123be"),
        FIRST_SUPERUSER_NAME="Some Name",
        POSTGRES_HOST="http://localhost",
        POSTGRES_USER="postgres",
        POSTGRES_PASSWORD=SecretStr("Somepassword!"),
        VALKEY_HOST="http://localhost",
        VALKEY_PASSWORD=SecretStr("Somepassword!"),
        SECRET_KEY=SecretStr("Somesecretkey"),
        BACKEND_CORS_ORIGINS="http://localhost, http://127.0.0.1",
    )

    assert settings.BACKEND_CORS_ORIGINS == [AnyUrl("http://localhost"), AnyUrl("http://127.0.0.1")]

"#
    )
}

pub fn save_config_test_file(project_info: &ProjectInfo) -> Result<()> {
    let base = &project_info.base_dir();
    let file_path = base.join("tests/core/test_config.py");
    let file_content = create_config_test_file(project_info);

    save_file_with_content(&file_path, &file_content)?;

    Ok(())
}

fn create_conftest_file(project_info: &ProjectInfo) -> String {
    let module = &project_info.module_name();

    format!(
        r#"from __future__ import annotations

import itertools
import os
import subprocess
from pathlib import Path
from unittest.mock import patch
from uuid import uuid4

import asyncpg
import pytest
from httpx import ASGITransport, AsyncClient

from {module}.api.deps import get_cache_client, get_db_pool
from {module}.core.cache import cache
from {module}.core.config import settings
from {module}.core.db import Database
from {module}.main import app
from {module}.models.users import UserCreate
from {module}.services.db import user_services
from tests.utils import (
    get_superuser_token_headers,
    random_email,
    random_lower_string,
    random_password,
)

ROOT_PATH = Path().absolute()
ASSETS_DIR = ROOT_PATH / "tests" / "assets"


async def user_authentication_headers(test_client, email, password):
    data = {{"username": email, "password": password}}

    result = await test_client.post("/login/access-token", data=data)
    response = result.json()
    auth_token = response["access_token"]
    return {{"Authorization": f"Bearer {{auth_token}}"}}


@pytest.fixture(scope="session")
def valkey_db_index(worker_id):
    if worker_id == "master":
        return 0
    else:
        return int(worker_id.lstrip("gw")) + 1


DBS_PER_WORKER = 5
MAX_DB_INDEX = 99
MAX_WORKERS = MAX_DB_INDEX // DBS_PER_WORKER
_db_counters: dict[int, itertools.count[int]] = {{}}


@pytest.fixture
def next_db(worker_id):
    """Calculate db number per worker so data doesn't clash in parallel tests."""
    if worker_id == "master":
        return 1

    worker_num = int(worker_id.lstrip("gw") or "0")

    if worker_num >= MAX_WORKERS:
        raise RuntimeError(
            f"Worker {{worker_id}} exceeds DB allocation limit (max {{MAX_WORKERS}} workers). "
            f"Either reduce number of workers or decrease DBS_PER_WORKER."
        )

    base = 1 + (worker_num * DBS_PER_WORKER)  # skip db=0
    if base + DBS_PER_WORKER - 1 > MAX_DB_INDEX:
        raise RuntimeError(f"Worker {{worker_id}} would exceed MAX_DB_INDEX with base {{base}}")

    if worker_id not in _db_counters:
        _db_counters[worker_id] = itertools.count(0)

    offset = next(_db_counters[worker_id]) % DBS_PER_WORKER
    db_index = base + offset

    return db_index


@pytest.fixture
def db_name(worker_id):
    base_name = "ae_reporter_test"
    unique_suffix = str(uuid4()).replace("-", "")[:8]
    if worker_id == "master":
        return f"{{base_name}}_{{unique_suffix}}"
    return f"{{base_name}}_{{worker_id}}_{{unique_suffix}}"


@pytest.fixture(autouse=True)
async def test_cache(next_db):
    await cache.create_client(db=next_db)
    yield cache
    await cache.client.flushdb()  # type: ignore
    await cache.close_client()


@pytest.fixture
def apply_migrations(db_name):
    test_db_url = f"postgresql://{{settings.POSTGRES_USER}}:{{settings.POSTGRES_PASSWORD.get_secret_value()}}@{{settings.POSTGRES_HOST}}:5432/{{db_name}}"
    migration_dir = ROOT_PATH.parent

    with patch.dict(os.environ, {{"DATABASE_URL": test_db_url}}):
        subprocess.run(["sqlx", "database", "create"], cwd=migration_dir)
        subprocess.run(["sqlx", "migrate", "run"], cwd=migration_dir)
    yield


@pytest.fixture
async def test_db(db_name, apply_migrations):
    test_db = Database(db_name=db_name)
    await test_db.create_pool(min_size=1, max_size=2)
    await test_db.create_first_superuser()
    yield test_db
    await test_db.close_pool()

    # Need to connect to "postgres" db instead of the db being dropped
    conn = await asyncpg.connect(
        database="postgres",
        user=settings.POSTGRES_USER,
        password=settings.POSTGRES_PASSWORD.get_secret_value(),
        host=settings.POSTGRES_HOST,
    )

    # Terminate any remaining connections to the test database
    await conn.execute(
        """
        SELECT pg_terminate_backend(pid)
        FROM pg_stat_activity
        WHERE datname = $1 AND pid <> pg_backend_pid()
    """,
        db_name,
    )

    await conn.execute(f'DROP DATABASE "{{db_name}}"')
    await conn.close()


@pytest.fixture
async def test_client(test_db, test_cache):
    app.dependency_overrides[get_cache_client] = lambda: test_cache.client
    app.dependency_overrides[get_db_pool] = lambda: test_db.db_pool
    async with AsyncClient(
        transport=ASGITransport(app=app), base_url=f"http://127.0.0.1{{settings.API_V1_PREFIX}}"
    ) as client:
        yield client
    app.dependency_overrides.clear()


@pytest.fixture
async def superuser_token_headers(test_client):
    return await get_superuser_token_headers(test_client)


@pytest.fixture
def normal_user_credentials():
    return {{
        "password": random_password(),
        "full_name": random_lower_string(),
        "email": random_email(),
    }}


@pytest.fixture
async def normal_user_token_headers(
    test_db, test_client, test_cache, normal_user_credentials
):
    user = await user_services.get_user_by_email(
        pool=test_db.db_pool, email=normal_user_credentials["email"]
    )
    if not user:
        user = await user_services.create_user(
            pool=test_db.db_pool,
            cache_client=test_cache.client,
            user=UserCreate(
                email=normal_user_credentials["email"],
                password=normal_user_credentials["password"],
                full_name=normal_user_credentials["full_name"],
            ),
        )

    return await user_authentication_headers(
        test_client=test_client,
        email=normal_user_credentials["email"],
        password=normal_user_credentials["password"],
    )
"#
    )
}

pub fn save_conftest_file(project_info: &ProjectInfo) -> Result<()> {
    let base = &project_info.base_dir();
    let file_path = base.join("tests/conftest.py");
    let file_content = create_conftest_file(project_info);

    save_file_with_content(&file_path, &file_content)?;

    Ok(())
}

fn create_test_uitls_file(project_info: &ProjectInfo) -> String {
    let module = &project_info.module_name();

    format!(
        r#"import random
import string

from {module}.core.config import settings


def random_email() -> str:
    return f"{{random_lower_string()}}@{{random_lower_string()}}.com"


def random_lower_string() -> str:
    return "".join(random.choices(string.ascii_lowercase, k=32))


def random_password() -> str:
    password = "".join(random.choices(string.ascii_lowercase, k=32))
    return f"A{{password}}1_"


async def get_superuser_token_headers(test_client):
    login_data = {{
        "username": settings.FIRST_SUPERUSER_EMAIL,
        "password": settings.FIRST_SUPERUSER_PASSWORD.get_secret_value(),
    }}
    response = await test_client.post("/login/access-token", data=login_data)
    tokens = response.json()
    a_token = tokens["access_token"]
    headers = {{"Authorization": f"Bearer {{a_token}}"}}
    return headers
"#
    )
}

pub fn save_test_utils_file(project_info: &ProjectInfo) -> Result<()> {
    let base = &project_info.base_dir();
    let file_path = base.join("tests/utils.py");
    let file_content = create_test_uitls_file(project_info);

    save_file_with_content(&file_path, &file_content)?;

    Ok(())
}

fn create_test_deps_file(project_info: &ProjectInfo) -> String {
    let module = &project_info.module_name();

    format!(
        r#"from unittest.mock import Mock

import pytest
from fastapi import HTTPException, Request

from {module}.api.deps import get_cache_client, get_current_user, get_db_pool
from {module}.core.cache import cache
from {module}.core.db import db


async def test_auth_no_authorization_in_header(test_client, normal_user_token_headers):
    del normal_user_token_headers["Authorization"]
    test_client.cookies.clear()
    response = await test_client.get(
        "/users/me",
        headers=normal_user_token_headers,
    )

    assert response.status_code == 401


async def test_auth_no_bearer(test_client, normal_user_token_headers):
    normal_user_token_headers["Authorization"] = normal_user_token_headers[
        "Authorization"
    ].removeprefix("Bearer ")
    test_client.cookies.clear()
    response = await test_client.get(
        "/users/me",
        headers=normal_user_token_headers,
    )

    assert response.status_code == 401


async def test_get_current_user_invalid_token(test_db):
    mock_request = Mock(spec=Request)
    mock_request.url.path = "/api/v1/users/me"

    with pytest.raises(HTTPException) as ex:
        await get_current_user(
            test_db.db_pool,
            "e",
            mock_request,
        )

    assert ex.value.status_code == 403


async def test_get_current_user_inactive(
    test_client, test_cache, normal_user_token_headers, superuser_token_headers, test_db
):
    mock_request = Mock(spec=Request)
    mock_request.url.path = "/api/v1/users/me"

    user = await get_current_user(
        test_db.db_pool,
        test_cache.client,
        normal_user_token_headers["Authorization"].split(" ", 1)[1],
    )

    test_client.cookies.clear()
    response = await test_client.patch(
        f"/users/{{user.id}}",
        headers=superuser_token_headers,
        json={{"fullName": user.full_name, "isActive": False}},
    )

    assert response.status_code == 200

    with pytest.raises(HTTPException) as ex:
        await get_current_user(
            test_db.db_pool,
            normal_user_token_headers["Authorization"].split(" ", 1)[1],
            mock_request,
        )

    assert ex.value.status_code == 403


@pytest.fixture
async def temp_db_pool():
    await db.create_pool()
    yield
    await db.close_pool()


@pytest.mark.usefixtures("temp_db_pool")
async def test_get_db_pool_success():
    async for pool in get_db_pool():
        assert pool is not None


@pytest.fixture
async def temp_cache_client():
    await cache.create_client()
    yield
    await cache.close_client()


@pytest.mark.usefixtures("temp_cache_client")
async def test_get_cache_client_success():
    async for client in get_cache_client():
        assert client is not None

"#
    )
}

pub fn save_test_deps_file(project_info: &ProjectInfo) -> Result<()> {
    let base = &project_info.base_dir();
    let file_path = base.join("tests/api/test_deps.py");
    let file_content = create_test_deps_file(project_info);

    save_file_with_content(&file_path, &file_content)?;

    Ok(())
}
