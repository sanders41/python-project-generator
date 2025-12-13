use anyhow::{bail, Result};

use crate::{
    file_manager::save_file_with_content,
    project_info::{Day, DependabotSchedule, ProjectInfo, ProjectManager, Pyo3PythonManager},
};

fn build_actions_python_test_versions(github_action_python_test_versions: &[String]) -> String {
    github_action_python_test_versions
        .iter()
        .map(|x| format!(r#""{x}""#))
        .collect::<Vec<String>>()
        .join(", ")
}

fn create_poetry_ci_testing_linux_only_file(
    source_dir: &str,
    min_python_version: &str,
    github_action_python_test_versions: &[String],
) -> String {
    let python_versions = build_actions_python_test_versions(github_action_python_test_versions);

    format!(
        r#"name: Testing

on:
  push:
    branches:
    - main
  pull_request:
env:
  PYTHON_VERSION: "{min_python_version}"
jobs:
  linting:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v5
    - name: Install Poetry
      run: pipx install poetry
    - name: Configure poetry
      run: |
        poetry config virtualenvs.create true
        poetry config virtualenvs.in-project true
    - name: Set up Python
      uses: actions/setup-python@v6
      with:
        python-version: ${{{{ env.PYTHON_VERSION }}}}
        cache: "poetry"
    - name: Install Dependencies
      run: poetry install
    - name: Ruff format check
      run: poetry run ruff format {source_dir} tests --check
    - name: Lint with ruff
      run: poetry run ruff check .
    - name: mypy check
      run: poetry run mypy .
  testing:
    strategy:
      fail-fast: false
      matrix:
        python-version: [{python_versions}]
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v5
    - name: Install Poetry
      run: pipx install poetry
    - name: Configure poetry
      run: |
        poetry config virtualenvs.create true
        poetry config virtualenvs.in-project true
    - name: Set up Python ${{{{ matrix.python-version }}}}
      uses: actions/setup-python@v6
      with:
        python-version: ${{{{ matrix.python-version }}}}
        cache: "poetry"
    - name: Install Dependencies
      run: poetry install
    - name: Test with pytest
      run: poetry run pytest
"#
    )
}

#[cfg(feature = "fastapi")]
fn create_poetry_ci_testing_fastapi_file(
    source_dir: &str,
    min_python_version: &str,
    github_action_python_test_versions: &[String],
) -> String {
    let python_versions = build_actions_python_test_versions(github_action_python_test_versions);

    format!(
        r#"name: Testing

on:
  push:
    branches:
    - main
  pull_request:
env:
  PYTHON_VERSION: "{min_python_version}"
  SECRET_KEY: "someKey"
  PRODUCTION_MODE: false
  FIRST_SUPERUSER_EMAIL: "some@email.com"
  FIRST_SUPERUSER_PASSWORD: "somePassword1!"
  FIRST_SUPERUSER_NAME: "Super User"
  POSTGRES_HOST: "127.0.0.1"
  POSTGRES_USER: "postgres"
  POSTGRES_PASSWORD: "test_password"
  POSTGRES_DB: "test_db"
  VALKEY_HOST: "127.0.0.1"
  VALKEY_PASSWORD: "test_password"
  MEILISEARCH_HOST: http://127.0.0.1
  STACK_NAME: "test-stack"
  DOMAIN: "127.0.0.1"
  LOG_LEVEL: "DEBUG"
  CI: true
jobs:
  linting:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v5
    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable
    - name: Cache dependencies
      uses: Swatinem/rust-cache@v2
    - name: Install sqlx-cli
      run: cargo install sqlx-cli --no-default-features -F native-tls -F postgres
    - name: Install Poetry
      run: pipx install poetry
    - name: Configure poetry
      run: |
        poetry config virtualenvs.create true
        poetry config virtualenvs.in-project true
    - name: Set up Python
      uses: actions/setup-python@v6
      with:
        python-version: ${{{{ env.PYTHON_VERSION }}}}
        cache: "poetry"
    - name: Install Dependencies
      run: poetry install
    - name: Ruff format check
      run: poetry run ruff format {source_dir} tests --check
    - name: Lint with ruff
      run: poetry run ruff check .
    - name: mypy check
      run: poetry run mypy .
  testing:
    strategy:
      fail-fast: false
      matrix:
        python-version: [{python_versions}]
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v5
    - name: Install Poetry
      run: pipx install poetry
    - name: Configure poetry
      run: |
        poetry config virtualenvs.create true
        poetry config virtualenvs.in-project true
    - name: Set up Python ${{{{ matrix.python-version }}}}
      uses: actions/setup-python@v6
      with:
        python-version: ${{{{ matrix.python-version }}}}
        cache: "poetry"
    - name: Install Dependencies
      run: poetry install
    - name: make .env
      run: touch .env
    - name: Start docker containers
      run: docker compose up db valkey migrations -d
    - name: Test with pytest
      run: poetry run pytest -n auto
"#
    )
}

fn create_setuptools_ci_testing_linux_only_file(
    source_dir: &str,
    min_python_version: &str,
    github_action_python_test_versions: &[String],
) -> String {
    let python_versions = build_actions_python_test_versions(github_action_python_test_versions);

    format!(
        r#"name: Testing

on:
  push:
    branches:
    - main
  pull_request:
env:
  PYTHON_VERSION: "{min_python_version}"
jobs:
  linting:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v5
    - name: Set up Python
      uses: actions/setup-python@v6
      with:
        python-version: ${{{{ env.PYTHON_VERSION }}}}
        cache: "pip"
    - name: Install Dependencies
      run: |
        python -m pip install -U pip
        python -m pip install -r requirements-dev.txt
    - name: Ruff format check
      run: ruff format {source_dir} tests --check
    - name: Lint with ruff
      run: ruff check .
    - name: mypy check
      run: mypy .
  testing:
    strategy:
      fail-fast: false
      matrix:
        python-version: [{python_versions}]
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v5
    - name: Set up Python ${{{{ matrix.python-version }}}}
      uses: actions/setup-python@v6
      with:
        python-version: ${{{{ matrix.python-version }}}}
        cache: "pip"
    - name: Install Dependencies
      run: |
        python -m pip install -U pip
        python -m pip install -r requirements-dev.txt
    - name: Test with pytest
      run: pytest
"#
    )
}

#[cfg(feature = "fastapi")]
fn create_setuptools_ci_testing_fastapi_file(
    source_dir: &str,
    min_python_version: &str,
    github_action_python_test_versions: &[String],
) -> String {
    let python_versions = build_actions_python_test_versions(github_action_python_test_versions);

    format!(
        r#"name: Testing

on:
  push:
    branches:
    - main
  pull_request:
env:
  PYTHON_VERSION: "{min_python_version}"
  SECRET_KEY: "someKey"
  PRODUCTION_MODE: false
  FIRST_SUPERUSER_EMAIL: "some@email.com"
  FIRST_SUPERUSER_PASSWORD: "somePassword1!"
  FIRST_SUPERUSER_NAME: "Super User"
  POSTGRES_HOST: "127.0.0.1"
  POSTGRES_USER: "postgres"
  POSTGRES_PASSWORD: "test_password"
  POSTGRES_DB: "test_db"
  VALKEY_HOST: "127.0.0.1"
  VALKEY_PASSWORD: "test_password"
  MEILISEARCH_HOST: http://127.0.0.1
  STACK_NAME: "test-stack"
  DOMAIN: "127.0.0.1"
  LOG_LEVEL: "DEBUG"
  CI: true
jobs:
  linting:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v5
    - name: Set up Python
      uses: actions/setup-python@v6
      with:
        python-version: ${{{{ env.PYTHON_VERSION }}}}
        cache: "pip"
    - name: Install Dependencies
      run: |
        python -m pip install -U pip
        python -m pip install -r requirements-dev.txt
    - name: Ruff format check
      run: ruff format {source_dir} tests --check
    - name: Lint with ruff
      run: ruff check .
    - name: mypy check
      run: mypy .
  testing:
    strategy:
      fail-fast: false
      matrix:
        python-version: [{python_versions}]
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v5
    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable
    - name: Cache dependencies
      uses: Swatinem/rust-cache@v2
    - name: Install sqlx-cli
      run: cargo install sqlx-cli --no-default-features -F native-tls -F postgres
    - name: Set up Python ${{{{ matrix.python-version }}}}
      uses: actions/setup-python@v6
      with:
        python-version: ${{{{ matrix.python-version }}}}
        cache: "pip"
    - name: Install Dependencies
      run: |
        python -m pip install -U pip
        python -m pip install -r requirements-dev.txt
    - name: make .env
      run: touch .env
    - name: Start docker containers
      run: docker compose up db valkey migrations -d
    - name: Test with pytest
      run: python -m pytest -n auto
"#
    )
}

fn create_uv_ci_testing_linux_only_file(
    source_dir: &str,
    min_python_version: &str,
    github_action_python_test_versions: &[String],
) -> String {
    let python_versions = build_actions_python_test_versions(github_action_python_test_versions);

    format!(
        r#"name: Testing

on:
  push:
    branches:
    - main
  pull_request:
env:
  PYTHON_VERSION: "{min_python_version}"
jobs:
  linting:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v5
    - name: Install uv
      uses: astral-sh/setup-uv@v7
      with:
        enable-cache: true
    - name: Set up Python
      uses: actions/setup-python@v6
      with:
        python-version: ${{{{ env.PYTHON_VERSION }}}}
    - name: Install Dependencies
      run: uv sync --frozen
    - name: Ruff format check
      run: uv run ruff format {source_dir} tests --check
    - name: Lint with ruff
      run: uv run ruff check .
    - name: mypy check
      run: uv run mypy .
  testing:
    strategy:
      fail-fast: false
      matrix:
        python-version: [{python_versions}]
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v5
    - name: Install uv
      uses: astral-sh/setup-uv@v7
      with:
        enable-cache: true
    - name: Set up Python ${{{{ matrix.python-version }}}}
      uses: actions/setup-python@v6
      with:
        python-version: ${{{{ matrix.python-version }}}}
    - name: Install Dependencies
      run: uv sync --frozen
    - name: Test with pytest
      run: uv run pytest
"#
    )
}

#[cfg(feature = "fastapi")]
fn create_uv_ci_testing_fastapi_file(
    source_dir: &str,
    min_python_version: &str,
    github_action_python_test_versions: &[String],
) -> String {
    let python_versions = build_actions_python_test_versions(github_action_python_test_versions);

    format!(
        r#"name: Testing

on:
  push:
    branches:
    - main
  pull_request:
env:
  PYTHON_VERSION: "{min_python_version}"
  SECRET_KEY: "someKey"
  PRODUCTION_MODE: false
  FIRST_SUPERUSER_EMAIL: "some@email.com"
  FIRST_SUPERUSER_PASSWORD: "somePassword1!"
  FIRST_SUPERUSER_NAME: "Super User"
  POSTGRES_HOST: "127.0.0.1"
  POSTGRES_USER: "postgres"
  POSTGRES_PASSWORD: "test_password"
  POSTGRES_DB: "test_db"
  VALKEY_HOST: "127.0.0.1"
  VALKEY_PASSWORD: "test_password"
  MEILISEARCH_HOST: http://127.0.0.1
  STACK_NAME: "test-stack"
  DOMAIN: "127.0.0.1"
  LOG_LEVEL: "DEBUG"
  CI: true
jobs:
  linting:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v5
    - name: Install uv
      uses: astral-sh/setup-uv@v7
      with:
        enable-cache: true
    - name: Set up Python
      uses: actions/setup-python@v6
      with:
        python-version: ${{{{ env.PYTHON_VERSION }}}}
    - name: Install Dependencies
      run: uv sync --frozen
    - name: Ruff format check
      run: uv run ruff format {source_dir} tests --check
    - name: Lint with ruff
      run: uv run ruff check .
    - name: mypy check
      run: uv run mypy .
  testing:
    strategy:
      fail-fast: false
      matrix:
        python-version: [{python_versions}]
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v5
    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable
    - name: Cache dependencies
      uses: Swatinem/rust-cache@v2
    - name: Install sqlx-cli
      run: cargo install sqlx-cli --no-default-features -F native-tls -F postgres
    - name: Install uv
      uses: astral-sh/setup-uv@v7
      with:
        enable-cache: true
    - name: Set up Python ${{{{ matrix.python-version }}}}
      uses: actions/setup-python@v6
      with:
        python-version: ${{{{ matrix.python-version }}}}
    - name: Install Dependencies
      run: uv sync --frozen
    - name: make .env
      run: touch .env
    - name: Start docker containers
      run: docker compose up db valkey migrations -d
    - name: Test with pytest
      run: uv run pytest -n auto
"#
    )
}

fn create_ci_testing_linux_only_file_pyo3(
    source_dir: &str,
    min_python_version: &str,
    github_action_python_test_versions: &[String],
    pyo3_python_manager: &Pyo3PythonManager,
) -> String {
    let python_versions = build_actions_python_test_versions(github_action_python_test_versions);
    match pyo3_python_manager {
        Pyo3PythonManager::Uv => format!(
            r#"name: Testing

on:
  push:
    branches:
    - main
  pull_request:
env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1
  RUSTFLAGS: "-D warnings"
  PYTHON_VERSION: "{min_python_version}"
jobs:
  clippy:
    name: Clippy
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v5
    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable
      with:
        components: clippy
    - name: Cache dependencies
      uses: Swatinem/rust-cache@v2
    - name: Run cargo clippy
      run: cargo clippy --all-targets -- --deny warnings
  fmt:
    name: Rustfmt
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v5
    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable
      with:
        components: rustfmt
    - name: Cache dependencies
      uses: Swatinem/rust-cache@v2
    - name: Run cargo fmt
      run: cargo fmt --all -- --check
  python-linting:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v5
    - name: Install uv
      uses: astral-sh/setup-uv@v7
      with:
        enable-cache: true
    - name: Set up Python
      uses: actions/setup-python@v6
      with:
        python-version: ${{{{ env.PYTHON_VERSION }}}}
    - name: Install Dependencies
      run: |
        uv sync --frozen
        uv run maturin build
    - name: Ruff format check
      run: uv run ruff format {source_dir} tests --check
    - name: Lint with ruff
      run: uv run ruff check .
    - name: mypy check
      run: uv run mypy {source_dir} tests
  testing:
    strategy:
      fail-fast: false
      matrix:
        python-version: [{python_versions}]
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v5
    - name: Install uv
      uses: astral-sh/setup-uv@v7
      with:
        enable-cache: true
    - name: Set up Python ${{{{ matrix.python-version }}}}
      uses: actions/setup-python@v6
      with:
        python-version: ${{{{ matrix.python-version }}}}
    - name: Install Dependencies
      run: |
        uv sync --frozen
        uv run maturin build
    - name: Test with pytest
      run: uv run pytest
"#
        ),
        Pyo3PythonManager::Setuptools => format!(
            r#"name: Testing

on:
  push:
    branches:
    - main
  pull_request:
env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1
  RUSTFLAGS: "-D warnings"
  PYTHON_VERSION: "{min_python_version}"
jobs:
  clippy:
    name: Clippy
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v5
    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable
      with:
        components: clippy
    - name: Cache dependencies
      uses: Swatinem/rust-cache@v2
    - name: Run cargo clippy
      run: cargo clippy --all-targets -- --deny warnings
  fmt:
    name: Rustfmt
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v5
    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable
      with:
        components: rustfmt
    - name: Cache dependencies
      uses: Swatinem/rust-cache@v2
    - name: Run cargo fmt
      run: cargo fmt --all -- --check
  python-linting:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v5
    - name: Set up Python
      uses: actions/setup-python@v6
      with:
        python-version: ${{{{ env.PYTHON_VERSION }}}}
        cache: "pip"
    - name: Install Dependencies
      run: |
        python -m pip install -U pip
        python -m pip install -r requirements-dev.txt
        python -m pip install -e .
        maturin build --out dist
    - name: Ruff format check
      run: ruff format {source_dir} tests --check
    - name: Lint with ruff
      run: ruff check .
    - name: mypy check
      run: mypy .
  testing:
    strategy:
      fail-fast: false
      matrix:
        python-version: [{python_versions}]
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v5
    - name: Set up Python ${{{{ matrix.python-version }}}}
      uses: actions/setup-python@v6
      with:
        python-version: ${{{{ matrix.python-version }}}}
        cache: "pip"
    - name: Install Dependencies
      run: |
        python -m pip install -U pip
        python -m pip install -r requirements-dev.txt
        python -m pip install -e .
        maturin build --out dist
    - name: Test with pytest
      run: pytest
"#
        ),
    }
}

#[cfg(feature = "fastapi")]
fn create_ci_testing_fastapi_file_pyo3(
    source_dir: &str,
    min_python_version: &str,
    github_action_python_test_versions: &[String],
    pyo3_python_manager: &Pyo3PythonManager,
) -> String {
    let python_versions = build_actions_python_test_versions(github_action_python_test_versions);
    match pyo3_python_manager {
        Pyo3PythonManager::Uv => format!(
            r#"name: Testing

on:
  push:
    branches:
    - main
  pull_request:
env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1
  RUSTFLAGS: "-D warnings"
  PYTHON_VERSION: "{min_python_version}"
  SECRET_KEY: "someKey"
  PRODUCTION_MODE: false
  FIRST_SUPERUSER_EMAIL: "some@email.com"
  FIRST_SUPERUSER_PASSWORD: "somePassword1!"
  FIRST_SUPERUSER_NAME: "Super User"
  POSTGRES_HOST: "127.0.0.1"
  POSTGRES_USER: "postgres"
  POSTGRES_PASSWORD: "test_password"
  POSTGRES_DB: "test_db"
  VALKEY_HOST: "127.0.0.1"
  VALKEY_PASSWORD: "test_password"
  MEILISEARCH_HOST: http://127.0.0.1
  STACK_NAME: "test-stack"
  DOMAIN: "127.0.0.1"
  LOG_LEVEL: "DEBUG"
  CI: true
jobs:
  clippy:
    name: Clippy
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v5
    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable
      with:
        components: clippy
    - name: Cache dependencies
      uses: Swatinem/rust-cache@v2
    - name: Run cargo clippy
      run: cargo clippy --all-targets -- --deny warnings
  fmt:
    name: Rustfmt
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v5
    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable
      with:
        components: rustfmt
    - name: Cache dependencies
      uses: Swatinem/rust-cache@v2
    - name: Run cargo fmt
      run: cargo fmt --all -- --check
  python-linting:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v5
    - name: Install uv
      uses: astral-sh/setup-uv@v7
      with:
        enable-cache: true
    - name: Set up Python
      uses: actions/setup-python@v6
      with:
        python-version: ${{{{ env.PYTHON_VERSION }}}}
    - name: Install Dependencies
      run: |
        uv sync --frozen
        uv run maturin build
    - name: Ruff format check
      run: uv run ruff format {source_dir} tests --check
    - name: Lint with ruff
      run: uv run ruff check .
    - name: mypy check
      run: uv run mypy {source_dir} tests
  testing:
    strategy:
      fail-fast: false
      matrix:
        python-version: [{python_versions}]
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v5
    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable
    - name: Cache dependencies
      uses: Swatinem/rust-cache@v2
    - name: Install sqlx-cli
      run: cargo install sqlx-cli --no-default-features -F native-tls -F postgres
    - name: Install uv
      uses: astral-sh/setup-uv@v7
      with:
        enable-cache: true
    - name: Set up Python ${{{{ matrix.python-version }}}}
      uses: actions/setup-python@v6
      with:
        python-version: ${{{{ matrix.python-version }}}}
    - name: Install Dependencies
      run: |
        uv sync --frozen
        uv run maturin build
    - name: make .env
      run: touch .env
    - name: Start docker containers
      run: docker compose up db valkey migrations -d
    - name: Test with pytest
      run: uv run pytest -n auto
"#
        ),
        Pyo3PythonManager::Setuptools => format!(
            r#"name: Testing

on:
  push:
    branches:
    - main
  pull_request:
env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1
  RUSTFLAGS: "-D warnings"
  PYTHON_VERSION: "{min_python_version}"
  SECRET_KEY: "someKey"
  PRODUCTION_MODE: false
  FIRST_SUPERUSER_EMAIL: "some@email.com"
  FIRST_SUPERUSER_PASSWORD: "somePassword1!"
  FIRST_SUPERUSER_NAME: "Super User"
  POSTGRES_HOST: "127.0.0.1"
  POSTGRES_USER: "postgres"
  POSTGRES_PASSWORD: "test_password"
  POSTGRES_DB: "test_db"
  VALKEY_HOST: "127.0.0.1"
  VALKEY_PASSWORD: "test_password"
  MEILISEARCH_HOST: http://127.0.0.1
  STACK_NAME: "test-stack"
  DOMAIN: "127.0.0.1"
  LOG_LEVEL: "DEBUG"
  CI: true
jobs:
  clippy:
    name: Clippy
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v5
    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable
      with:
        components: clippy
    - name: Cache dependencies
      uses: Swatinem/rust-cache@v2
    - name: Run cargo clippy
      run: cargo clippy --all-targets -- --deny warnings
  fmt:
    name: Rustfmt
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v5
    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable
      with:
        components: rustfmt
    - name: Cache dependencies
      uses: Swatinem/rust-cache@v2
    - name: Run cargo fmt
      run: cargo fmt --all -- --check
  python-linting:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v5
    - name: Set up Python
      uses: actions/setup-python@v6
      with:
        python-version: ${{{{ env.PYTHON_VERSION }}}}
        cache: "pip"
    - name: Install Dependencies
      run: |
        python -m pip install -U pip
        python -m pip install -r requirements-dev.txt
        python -m pip install -e .
        maturin build --out dist
    - name: Ruff format check
      run: ruff format {source_dir} tests --check
    - name: Lint with ruff
      run: ruff check .
    - name: mypy check
      run: mypy .
  testing:
    strategy:
      fail-fast: false
      matrix:
        python-version: [{python_versions}]
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v5
    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable
    - name: Cache dependencies
      uses: Swatinem/rust-cache@v2
    - name: Set up Python ${{{{ matrix.python-version }}}}
      uses: actions/setup-python@v6
      with:
        python-version: ${{{{ matrix.python-version }}}}
        cache: "pip"
    - name: Install sqlx-cli
      run: cargo install sqlx-cli --no-default-features -F native-tls -F postgres
    - name: Install Dependencies
      run: |
        python -m pip install -U pip
        python -m pip install -r requirements-dev.txt
        python -m pip install -e .
        maturin build --out dist
    - name: make .env
      run: touch .env
    - name: Start docker containers
      run: docker compose up db valkey migrations -d
    - name: Test with pytest
      run: python -m pytest -n auto
"#
        ),
    }
}

pub fn save_ci_testing_linux_only_file(project_info: &ProjectInfo) -> Result<()> {
    let file_path = project_info
        .base_dir()
        .join(".github/workflows/testing.yml");
    let content = match &project_info.project_manager {
        ProjectManager::Maturin => {
            if let Some(pyo3_python_manager) = &project_info.pyo3_python_manager {
                #[cfg(feature = "fastapi")]
                if project_info.is_fastapi_project {
                    create_ci_testing_fastapi_file_pyo3(
                        &project_info.source_dir,
                        &project_info.min_python_version,
                        &project_info.github_actions_python_test_versions,
                        pyo3_python_manager,
                    )
                } else {
                    create_ci_testing_linux_only_file_pyo3(
                        &project_info.source_dir,
                        &project_info.min_python_version,
                        &project_info.github_actions_python_test_versions,
                        pyo3_python_manager,
                    )
                }

                #[cfg(not(feature = "fastapi"))]
                create_ci_testing_linux_only_file_pyo3(
                    &project_info.source_dir,
                    &project_info.min_python_version,
                    &project_info.github_actions_python_test_versions,
                    pyo3_python_manager,
                )
            } else {
                bail!("A PyO3 Python manager is required for maturin");
            }
        }
        ProjectManager::Poetry => {
            #[cfg(feature = "fastapi")]
            if project_info.is_fastapi_project {
                create_poetry_ci_testing_fastapi_file(
                    &project_info.source_dir,
                    &project_info.min_python_version,
                    &project_info.github_actions_python_test_versions,
                )
            } else {
                create_poetry_ci_testing_linux_only_file(
                    &project_info.source_dir,
                    &project_info.min_python_version,
                    &project_info.github_actions_python_test_versions,
                )
            }

            #[cfg(not(feature = "fastapi"))]
            create_poetry_ci_testing_linux_only_file(
                &project_info.source_dir,
                &project_info.min_python_version,
                &project_info.github_actions_python_test_versions,
            )
        }
        ProjectManager::Setuptools => {
            #[cfg(feature = "fastapi")]
            if project_info.is_fastapi_project {
                create_setuptools_ci_testing_fastapi_file(
                    &project_info.source_dir,
                    &project_info.min_python_version,
                    &project_info.github_actions_python_test_versions,
                )
            } else {
                create_setuptools_ci_testing_linux_only_file(
                    &project_info.source_dir,
                    &project_info.min_python_version,
                    &project_info.github_actions_python_test_versions,
                )
            }

            #[cfg(not(feature = "fastapi"))]
            create_setuptools_ci_testing_linux_only_file(
                &project_info.source_dir,
                &project_info.min_python_version,
                &project_info.github_actions_python_test_versions,
            )
        }
        ProjectManager::Uv => {
            #[cfg(feature = "fastapi")]
            if project_info.is_fastapi_project {
                create_uv_ci_testing_fastapi_file(
                    &project_info.source_dir,
                    &project_info.min_python_version,
                    &project_info.github_actions_python_test_versions,
                )
            } else {
                create_uv_ci_testing_linux_only_file(
                    &project_info.source_dir,
                    &project_info.min_python_version,
                    &project_info.github_actions_python_test_versions,
                )
            }

            #[cfg(not(feature = "fastapi"))]
            create_uv_ci_testing_linux_only_file(
                &project_info.source_dir,
                &project_info.min_python_version,
                &project_info.github_actions_python_test_versions,
            )
        }
    };

    save_file_with_content(&file_path, &content)?;

    Ok(())
}

fn create_poetry_ci_testing_multi_os_file(
    source_dir: &str,
    min_python_version: &str,
    github_action_python_test_versions: &[String],
) -> String {
    let python_versions = build_actions_python_test_versions(github_action_python_test_versions);

    format!(
        r#"name: Testing

on:
  push:
    branches:
    - main
  pull_request:
env:
  PYTHON_VERSION: "{min_python_version}"
jobs:
  linting:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v5
    - name: Install Poetry
      run: pipx install poetry
    - name: Configure poetry
      run: |
        poetry config virtualenvs.create true
        poetry config virtualenvs.in-project true
    - name: Set up Python
      uses: actions/setup-python@v6
      with:
        python-version: ${{{{ env.PYTHON_VERSION }}}}
        cache: "poetry"
    - name: Install Dependencies
      run: poetry install
    - name: Ruff format check
      run: poetry run ruff format {source_dir} tests --check
    - name: Lint with ruff
      run: poetry run ruff check .
    - name: mypy check
      run: poetry run mypy .
  testing:
    strategy:
      fail-fast: false
      matrix:
        python-version: [{python_versions}]
        os: [ubuntu-latest, windows-latest, macos-latest]
    runs-on: ${{{{ matrix.os }}}}
    steps:
    - uses: actions/checkout@v5
    - name: Install Poetry
      run: pipx install poetry
    - name: Configure poetry
      run: |
        poetry config virtualenvs.create true
        poetry config virtualenvs.in-project true
    - name: Set up Python ${{{{ matrix.python-version }}}}
      uses: actions/setup-python@v6
      with:
        python-version: ${{{{ matrix.python-version }}}}
        cache: "poetry"
    - name: Install Dependencies
      run: poetry install
    - name: Test with pytest
      run: poetry run pytest
"#
    )
}

fn create_setuptools_ci_testing_multi_os_file(
    source_dir: &str,
    min_python_version: &str,
    github_action_python_test_versions: &[String],
) -> String {
    let python_versions = build_actions_python_test_versions(github_action_python_test_versions);

    format!(
        r#"name: Testing

on:
  push:
    branches:
    - main
  pull_request:
env:
  PYTHON_VERSION: "{min_python_version}"
jobs:
  linting:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v5
    - name: Set up Python
      uses: actions/setup-python@v6
      with:
        python-version: ${{{{ env.PYTHON_VERSION }}}}
        cache: "pip"
    - name: Install Dependencies
      run: |
        python -m pip install -U pip
        python -m pip install -r requirements-dev.txt
    - name: Ruff format check
      run: ruff format {source_dir} tests --check
    - name: Lint with ruff
      run: ruff check .
    - name: mypy check
      run: mypy .
  testing:
    strategy:
      fail-fast: false
      matrix:
        python-version: [{python_versions}]
        os: [ubuntu-latest, windows-latest, macos-latest]
    runs-on: ${{{{ matrix.os }}}}
    steps:
    - uses: actions/checkout@v5
    - name: Set up Python ${{{{ matrix.python-version }}}}
      uses: actions/setup-python@v6
      with:
        python-version: ${{{{ matrix.python-version }}}}
        cache: "pip"
    - name: Install Dependencies
      run: |
        python -m pip install -U pip
        python -m pip install -r requirements-dev.txt
    - name: Test with pytest
      run: pytest
"#
    )
}

fn create_ci_testing_multi_os_file_pyo3(
    source_dir: &str,
    min_python_version: &str,
    github_action_python_test_versions: &[String],
    pyo3_python_manager: &Pyo3PythonManager,
) -> String {
    let python_versions = build_actions_python_test_versions(github_action_python_test_versions);
    match pyo3_python_manager {
        Pyo3PythonManager::Uv => format!(
            r#"name: Testing

on:
  push:
    branches:
    - main
  pull_request:
env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1
  RUSTFLAGS: "-D warnings"
  PYTHON_VERSION: "{min_python_version}"
jobs:
  clippy:
    name: Clippy
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v5
    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable
      with:
        components: clippy
    - name: Cache dependencies
      uses: Swatinem/rust-cache@v2
    - name: Run cargo clippy
      run: cargo clippy --all-targets -- --deny warnings
  fmt:
    name: Rustfmt
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v5
    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable
      with:
        components: rustfmt
    - name: Cache dependencies
      uses: Swatinem/rust-cache@v2
    - name: Run cargo fmt
      run: cargo fmt --all -- --check
  python-linting:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v5
    - name: Install uv
      uses: astral-sh/setup-uv@v7
      with:
        enable-cache: true
    - name: Set up Python
      uses: actions/setup-python@v6
      with:
        python-version: ${{{{ env.PYTHON_VERSION }}}}
    - name: Install Dependencies
      run: |
        uv sync --frozen
        uv run maturin build
    - name: Ruff format check
      run: uv run ruff format {source_dir} tests --check
    - name: Lint with ruff
      run: uv run ruff check .
    - name: mypy check
      run: uv run mypy {source_dir} tests
  testing:
    strategy:
      fail-fast: false
      matrix:
        python-version: [{python_versions}]
        os: [ubuntu-latest, windows-latest, macos-latest]
    runs-on: ${{{{ matrix.os }}}}
    steps:
    - uses: actions/checkout@v5
    - name: Install uv
      uses: astral-sh/setup-uv@v7
      with:
        enable-cache: true
    - name: Set up Python ${{{{ matrix.python-version }}}}
      uses: actions/setup-python@v6
      with:
        python-version: ${{{{ matrix.python-version }}}}
    - name: Install Dependencies
      run: |
        uv sync --frozen
        uv run maturin build
    - name: Test with pytest
      run: uv run pytest
"#
        ),
        Pyo3PythonManager::Setuptools => format!(
            r#"name: Testing

on:
  push:
    branches:
    - main
  pull_request:
env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1
  RUSTFLAGS: "-D warnings"
  PYTHON_VERSION: "{min_python_version}"
jobs:
  clippy:
    name: Clippy
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v5
    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable
      with:
        components: clippy
    - name: Cache dependencies
      uses: Swatinem/rust-cache@v2
    - name: Run cargo clippy
      run: cargo clippy --all-targets -- --deny warnings
  fmt:
    name: Rustfmt
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v5
    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable
      with:
        components: rustfmt
    - name: Cache dependencies
      uses: Swatinem/rust-cache@v2
    - name: Run cargo fmt
      run: cargo fmt --all -- --check
  python-linting:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v5
    - name: Set up Python
      uses: actions/setup-python@v6
      with:
        python-version: ${{{{ env.PYTHON_VERSION }}}}
        cache: "pip"
    - name: Install Dependencies
      run: |
        python -m pip install -U pip
        python -m pip install -r requirements-dev.txt
        python -m pip install -e .
        maturin build --out dist
    - name: Ruff format check
      run: ruff format {source_dir} tests --check
    - name: Lint with ruff
      run: ruff check .
    - name: mypy check
      run: mypy .
  testing:
    strategy:
      fail-fast: false
      matrix:
        python-version: [{python_versions}]
        os: [ubuntu-latest, windows-latest, macos-latest]
    runs-on: ${{{{ matrix.os }}}}
    steps:
    - uses: actions/checkout@v5
    - name: Set up Python ${{{{ matrix.python-version }}}}
      uses: actions/setup-python@v6
      with:
        python-version: ${{{{ matrix.python-version }}}}
        cache: "pip"
    - name: Install Dependencies
      run: |
        python -m pip install -U pip
        python -m pip install -r requirements-dev.txt
        python -m pip install -e .
        maturin build --out dist
    - name: Test with pytest
      run: pytest
"#
        ),
    }
}

fn create_uv_ci_testing_multi_os_file(
    source_dir: &str,
    min_python_version: &str,
    github_action_python_test_versions: &[String],
) -> String {
    let python_versions = build_actions_python_test_versions(github_action_python_test_versions);

    format!(
        r#"name: Testing

on:
  push:
    branches:
    - main
  pull_request:
env:
  UV_CACHE_DIR: /tmp/.uv-cache
  PYTHON_VERSION: "{min_python_version}"
jobs:
  linting:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v5
    - name: Install uv
      uses: astral-sh/setup-uv@v7
      with:
        enable-cache: true
    - name: Set up Python
      uses: actions/setup-python@v6
      with:
        python-version: ${{{{ env.PYTHON_VERSION }}}}
    - name: Install Dependencies
      run: uv sync --frozen
    - name: Ruff format check
      run: uv run ruff format {source_dir} tests --check
    - name: Lint with ruff
      run: uv run ruff check .
    - name: mypy check
      run: uv run mypy .
  testing:
    strategy:
      fail-fast: false
      matrix:
        python-version: [{python_versions}]
        os: [ubuntu-latest, windows-latest, macos-latest]
    runs-on: ${{{{ matrix.os }}}}
    steps:
    - uses: actions/checkout@v5
    - name: Install uv
      uses: astral-sh/setup-uv@v7
      with:
        enable-cache: true
    - name: Set up Python ${{{{ matrix.python-version }}}}
      uses: actions/setup-python@v6
      with:
        python-version: ${{{{ matrix.python-version }}}}
    - name: Install Dependencies
      run: uv sync --frozen
    - name: Test with pytest
      run: uv run pytest
"#
    )
}

pub fn save_ci_testing_multi_os_file(project_info: &ProjectInfo) -> Result<()> {
    let file_path = project_info
        .base_dir()
        .join(".github/workflows/testing.yml");
    let content = match &project_info.project_manager {
        ProjectManager::Maturin => {
            if let Some(pyo3_python_manager) = &project_info.pyo3_python_manager {
                create_ci_testing_multi_os_file_pyo3(
                    &project_info.source_dir,
                    &project_info.min_python_version,
                    &project_info.github_actions_python_test_versions,
                    pyo3_python_manager,
                )
            } else {
                bail!("A PyO3 Python Manager is required for maturin");
            }
        }
        ProjectManager::Poetry => create_poetry_ci_testing_multi_os_file(
            &project_info.source_dir,
            &project_info.min_python_version,
            &project_info.github_actions_python_test_versions,
        ),
        ProjectManager::Setuptools => create_setuptools_ci_testing_multi_os_file(
            &project_info.source_dir,
            &project_info.min_python_version,
            &project_info.github_actions_python_test_versions,
        ),
        ProjectManager::Uv => create_uv_ci_testing_multi_os_file(
            &project_info.source_dir,
            &project_info.min_python_version,
            &project_info.github_actions_python_test_versions,
        ),
    };

    save_file_with_content(&file_path, &content)?;

    Ok(())
}

fn create_dependabot_schedule(
    dependabot_schedule: &Option<DependabotSchedule>,
    dependabot_day: &Option<Day>,
) -> String {
    if let Some(schedule) = dependabot_schedule {
        match schedule {
            DependabotSchedule::Daily => r#"schedule:
      interval: daily"#
                .to_string(),
            DependabotSchedule::Weekly => {
                if let Some(day) = dependabot_day {
                    match day {
                        Day::Monday => r#"schedule:
      interval: weekly
      day: monday"#
                            .to_string(),
                        Day::Tuesday => r#"schedule:
      interval: weekly
      day: tuesday"#
                            .to_string(),
                        Day::Wednesday => r#"schedule:
      interval: weekly
      day: wednesday"#
                            .to_string(),
                        Day::Thursday => r#"schedule:
      interval: weekly
      day: thursday"#
                            .to_string(),
                        Day::Friday => r#"schedule:
      interval: weekly
      day: friday"#
                            .to_string(),
                        Day::Saturday => r#"schedule:
      interval: weekly
      day: saturday"#
                            .to_string(),
                        Day::Sunday => r#"schedule:
      interval: weekly
      day: sunday"#
                            .to_string(),
                    }
                } else {
                    r#"schedule:
      interval: weekly
      day: monday"#
                        .to_string()
                }
            }
            DependabotSchedule::Monthly => r#"schedule:
      interval: monthly"#
                .to_string(),
        }
    } else {
        r#"schedule:
      interval: daily"#
            .to_string()
    }
}

fn create_dependabot_file(
    project_manager: &ProjectManager,
    dependabot_schedule: &Option<DependabotSchedule>,
    dependabot_day: &Option<Day>,
) -> String {
    let schedule = create_dependabot_schedule(dependabot_schedule, dependabot_day);
    let package_ecosystem = if let ProjectManager::Uv = project_manager {
        "uv"
    } else {
        "pip"
    };

    format!(
        r#"version: 2
updates:
  - package-ecosystem: "{package_ecosystem}"
    directory: "/"
    {schedule}
    labels:
    - skip-changelog
    - dependencies
  - package-ecosystem: "github-actions"
    directory: '/'
    {schedule}
    labels:
    - skip-changelog
    - dependencies
"#
    )
}

fn create_dependabot_file_pyo3(
    pyo3_python_manager: &Pyo3PythonManager,
    dependabot_schedule: &Option<DependabotSchedule>,
    dependabot_day: &Option<Day>,
) -> String {
    let schedule = create_dependabot_schedule(dependabot_schedule, dependabot_day);
    let package_ecosystem = if let Pyo3PythonManager::Uv = pyo3_python_manager {
        "uv"
    } else {
        "pip"
    };

    format!(
        r#"version: 2
updates:
  - package-ecosystem: "{package_ecosystem}"
    directory: "/"
    {schedule}
    labels:
    - skip-changelog
    - dependencies
  - package-ecosystem: "cargo"
    directory: "/"
    {schedule}
    labels:
    - skip-changelog
    - dependencies
  - package-ecosystem: "github-actions"
    directory: '/'
    {schedule}
    labels:
    - skip-changelog
    - dependencies
"#
    )
}

pub fn save_dependabot_file(project_info: &ProjectInfo) -> Result<()> {
    let file_path = project_info.base_dir().join(".github/dependabot.yml");
    let content = match &project_info.project_manager {
        ProjectManager::Maturin => {
            if let Some(pyo3_python_manager) = &project_info.pyo3_python_manager {
                create_dependabot_file_pyo3(
                    pyo3_python_manager,
                    &project_info.dependabot_schedule,
                    &project_info.dependabot_day,
                )
            } else {
                bail!("A PyO3 Python manager is required for maturin projects");
            }
        }
        _ => create_dependabot_file(
            &project_info.project_manager,
            &project_info.dependabot_schedule,
            &project_info.dependabot_day,
        ),
    };

    save_file_with_content(&file_path, &content)?;

    Ok(())
}

fn create_poetry_pypi_publish_file(python_version: &str) -> String {
    format!(
        r#"name: PyPi Publish
on:
  release:
    types:
    - published
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v5
    - name: Install Poetry
      run: pipx install poetry
    - name: Set up Python
      uses: actions/setup-python@v6
      with:
        python-version: "{python_version}"
        cache: "poetry"
    - name: Install Dependencies
      run: |
        poetry install
    - name: Publish package
      run: poetry publish --build
"#
    )
}

fn create_pyo3_pypi_publish_file(python_version: &str) -> String {
    format!(
        r#"name: PyPi Publish
on:
  release:
    types:
    - published
permissions:
  contents: read
jobs:
  linux:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        target: [x86_64, x86, aarch64, armv7, s390x, ppc64le]
    steps:
      - uses: actions/checkout@v5
      - uses: actions/setup-python@v6
        with:
          python-version: "{python_version}"
      - name: Build wheels
        uses: PyO3/maturin-action@v1
        with:
          target: ${{{{ matrix.target }}}}
          args: --release --out dist --find-interpreter
          sccache: 'true'
          manylinux: auto
      - name: Upload wheels
        uses: actions/upload-artifact@v5
        with:
          name: wheels-linux-${{{{ matrix.target }}}}
          path: dist
  windows:
    runs-on: windows-latest
    strategy:
      matrix:
        target: [x64, x86]
    steps:
      - uses: actions/checkout@v5
      - uses: actions/setup-python@v6
        with:
          python-version: "{python_version}"
          architecture: ${{{{ matrix.target }}}}
      - name: Build wheels
        uses: PyO3/maturin-action@v1
        with:
          target: ${{{{ matrix.target }}}}
          args: --release --out dist --find-interpreter
          sccache: 'true'
      - name: Upload wheels
        uses: actions/upload-artifact@v5
        with:
          name: wheels-windows-${{{{ matrix.target }}}}
          path: dist
  macos:
    runs-on: macos-latest
    strategy:
      matrix:
        target: [x86_64, aarch64]
    steps:
      - uses: actions/checkout@v5
      - uses: actions/setup-python@v6
        with:
          python-version: "{python_version}"
      - name: Build wheels
        uses: PyO3/maturin-action@v1
        with:
          target: ${{{{ matrix.target }}}}
          args: --release --out dist --find-interpreter
          sccache: 'true'
      - name: Upload wheels
        uses: actions/upload-artifact@v5
        with:
          name: wheels-macos-${{{{ matrix.target }}}}
          path: dist
  sdist:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5
      - uses: actions/setup-python@v6
        with:
          python-version: "{python_version}"
      - name: Build sdist
        uses: PyO3/maturin-action@v1
        with:
          command: sdist
          args: --out dist
      - name: Upload sdist
        uses: actions/upload-artifact@v5
        with:
          name: wheels-sdist
          path: dist
  release:
    name: Release
    runs-on: ubuntu-latest
    permissions:
      # For PyPI's trusted publishing.
      id-token: write
    if: "startsWith(github.ref, 'refs/tags/')"
    needs: [linux, windows, macos, sdist]
    steps:
      - uses: actions/download-artifact@v6
      - uses: actions/setup-python@v6
        with:
          python-version: "{python_version}"
      - name: Publish to PyPI
        uses: PyO3/maturin-action@v1
        with:
          command: upload
          args: --non-interactive --skip-existing wheels-*/*
"#
    )
}

fn create_setuptools_pypi_publish_file(python_version: &str) -> String {
    format!(
        r#"name: PyPi Publish
on:
  release:
    types:
    - published
jobs:
  deploy:
    runs-on: ubuntu-latest
    permissions:
      # For PyPI's trusted publishing.
      id-token: write
    steps:
    - uses: actions/checkout@v5
    - name: Set up Python
      uses: actions/setup-python@v6
      with:
        python-version: "{python_version}"
        cache: "pip"
    - name: Install Dependencies
      run: |
        python -m pip install -U pip
        python -m pip install -r requirements-dev.txt
        python -m pip install build setuptools wheel twine
    - name: Build and publish package
      run: |
        python -m build
        twine upload dist/*
"#
    )
}

fn create_uv_pypi_publish_file(python_version: &str) -> String {
    format!(
        r#"name: PyPi Publish
on:
  release:
    types:
    - published
jobs:
  deploy:
    runs-on: ubuntu-latest
    permissions:
      # For PyPI's trusted publishing.
      id-token: write
    steps:
    - uses: actions/checkout@v5
    - name: Install uv
      uses: astral-sh/setup-uv@v7
      with:
        enable-cache: true
    - name: Set up Python
      uses: actions/setup-python@v6
      with:
        python-version: "{python_version}"
    - name: Install Dependencies
      run: uv sync --frozen
    - name: Build package
      run: uv build
    - name: Publish package
      run: uv publish
"#
    )
}

#[cfg(feature = "fastapi")]
fn create_testing_deploy_file() -> String {
    r#"name: Deploy to Testing
on:
  push:
    branches:
      - main
  workflow_dispatch:
jobs:
  deploy:
    runs-on:
      - self-hosted
      - testing
    env:
      ENVIRONMENT: testing
      DOMAIN: ${{ secrets.DOMAIN_TESTING }}
      STACK_NAME: ${{ secrets.STACK_NAME_TESTING }}
      SECRET_KEY: ${{ secrets.SECRET_KEY }}
      FIRST_SUPERUSER_EMAIL: ${{ secrets.FIRST_SUPERUSER_EMAIL }}
      FIRST_SUPERUSER_PASSWORD: ${{ secrets.FIRST_SUPERUSER_PASSWORD }}
      FIRST_SUPERUSER_NAME: ${{ secrets.FIRST_SUPERUSER_NAME }}
      POSTGRES_HOST: ${{ secrets.POSTGRES_HOST }}
      POSTGRES_USER: ${{ secrets.POSTGRES_USER }}
      POSTGRES_PASSWORD: ${{ secrets.POSTGRES_PASSWORD }}
      POSTGRES_DB: ${{ secrets.POSTGRES_DB }}
      VALKEY_HOST: ${{ secrets.VALKEY_HOST }}
      VALKEY_PASSWORD: ${{ secrets.VALKEY_PASSWORD }}
      HANDWRITING_OCR_TOKEN: ${{ secrets.HANDWRITING_OCR_TOKEN }}
      USERNAME: ${{ secrets.FIRST_SUPERUSER_EMAIL }}
      PASSWORD: ${{ secrets.FIRST_SUPERUSER_PASSWORD }}
      EMAIL: ${{ secrets.FIRST_SUPERUSER_EMAIL }}
      ERROR_NOTIFICATION_URL: ${{ secrets.ERROR_NOTIFICATION_URL_TESTING }}
      LOG_LEVEL: "DEBUG"
    steps:
      - name: Fix permissions
        run: |
          if [ -d "./data" ]; then
            sudo chown -R $USER:$USER ./data
          fi
      - name: Checkout
        uses: actions/checkout@v5
      - name: Create .env file
        run: |
          HASHED_PASSWORD=$(openssl passwd -apr1 "${PASSWORD}" | sed 's/\$/\$\$/g')
          cat > .env << EOF
          ENVIRONMENT=${ENVIRONMENT}
          DOMAIN=${DOMAIN}
          STACK_NAME=${STACK_NAME}
          SECRET_KEY=${SECRET_KEY}
          FIRST_SUPERUSER_EMAIL=${FIRST_SUPERUSER_EMAIL}
          FIRST_SUPERUSER_PASSWORD=${FIRST_SUPERUSER_PASSWORD}
          FIRST_SUPERUSER_NAME=${FIRST_SUPERUSER_NAME}
          POSTGRES_HOST=${POSTGRES_HOST}
          POSTGRES_USER=${POSTGRES_USER}
          POSTGRES_PASSWORD=${POSTGRES_PASSWORD}
          POSTGRES_DB=${POSTGRES_DB}
          VALKEY_HOST=${VALKEY_HOST}
          VALKEY_PASSWORD=${VALKEY_PASSWORD}
          HANDWRITING_OCR_TOKEN=${HANDWRITING_OCR_TOKEN}
          USERNAME=${FIRST_SUPERUSER_EMAIL}
          PASSWORD=${FIRST_SUPERUSER_PASSWORD}
          HASHED_PASSWORD=${HASHED_PASSWORD}
          EMAIL=${FIRST_SUPERUSER_EMAIL}
          ERROR_NOTIFICATION_URL=${ERROR_NOTIFICATION_URL}
          LOG_LEVEL=${LOG_LEVEL}
          EOF
      - name: Build and restart containers
        timeout-minutes: 15
        run: |
          docker compose -f docker-compose.yml --project-name ${{ secrets.STACK_NAME_TESTING }} build
          docker compose -f docker-compose.yml --project-name ${{ secrets.STACK_NAME_TESTING }} up -d
"#.to_string()
}

#[cfg(feature = "fastapi")]
fn create_production_deploy_file() -> String {
    r#"name: Deploy to Production
on:
  release:
    types:
      - published
jobs:
  deploy:
    runs-on:
      - self-hosted
      - production
    env:
      ENVIRONMENT: production
      DOMAIN: ${{ secrets.DOMAIN_PRODUCTION }}
      STACK_NAME: ${{ secrets.STACK_NAME_PRODUCTION }}
      SECRET_KEY: ${{ secrets.SECRET_KEY }}
      FIRST_SUPERUSER_EMAIL: ${{ secrets.FIRST_SUPERUSER_EMAIL }}
      FIRST_SUPERUSER_PASSWORD: ${{ secrets.FIRST_SUPERUSER_PASSWORD }}
      FIRST_SUPERUSER_NAME: ${{ secrets.FIRST_SUPERUSER_NAME }}
      POSTGRES_HOST: ${{ secrets.POSTGRES_HOST }}
      POSTGRES_USER: ${{ secrets.POSTGRES_USER }}
      POSTGRES_PASSWORD: ${{ secrets.POSTGRES_PASSWORD }}
      POSTGRES_DB: ${{ secrets.POSTGRES_DB }}
      VALKEY_HOST: ${{ secrets.VALKEY_HOST }}
      VALKEY_PASSWORD: ${{ secrets.VALKEY_PASSWORD }}
      USERNAME: ${{ secrets.FIRST_SUPERUSER_EMAIL }}
      PASSWORD: ${{ secrets.FIRST_SUPERUSER_PASSWORD }}
      EMAIL: ${{ secrets.FIRST_SUPERUSER_EMAIL }}
      LOG_LEVEL: "INFO"
    steps:
      - name: Fix permissions
        run: |
          if [ -d "./data" ]; then
            sudo chown -R $USER:$USER ./data
          fi
      - name: Checkout
        uses: actions/checkout@v5
      - name: Create .env file
        run: |
          HASHED_PASSWORD=$(openssl passwd -apr1 "${PASSWORD}" | sed 's/\$/\$\$/g')
          cat > .env << EOF
          ENVIRONMENT=${ENVIRONMENT}
          DOMAIN=${DOMAIN}
          STACK_NAME=${STACK_NAME}
          SECRET_KEY=${SECRET_KEY}
          FIRST_SUPERUSER_EMAIL=${FIRST_SUPERUSER_EMAIL}
          FIRST_SUPERUSER_PASSWORD=${FIRST_SUPERUSER_PASSWORD}
          FIRST_SUPERUSER_NAME=${FIRST_SUPERUSER_NAME}
          POSTGRES_HOST=${POSTGRES_HOST}
          POSTGRES_USER=${POSTGRES_USER}
          POSTGRES_PASSWORD=${POSTGRES_PASSWORD}
          POSTGRES_DB=${POSTGRES_DB}
          VALKEY_HOST=${VALKEY_HOST}
          VALKEY_PASSWORD=${VALKEY_PASSWORD}
          USERNAME=${FIRST_SUPERUSER_EMAIL}
          PASSWORD=${FIRST_SUPERUSER_PASSWORD}
          HASHED_PASSWORD=${HASHED_PASSWORD}
          EMAIL=${FIRST_SUPERUSER_EMAIL}
          LOG_LEVEL=${LOG_LEVEL}
          EOF
      - name: Build and restart containers
        timeout-minutes: 15
        run: |
          docker compose -f docker-compose.yml --project-name ${{ secrets.STACK_NAME_PRODUCTION }} build
          docker compose -f docker-compose.yml --project-name ${{ secrets.STACK_NAME_PRODUCTION }} up -d
"#.to_string()
}

#[cfg(feature = "fastapi")]
pub fn save_deploy_files(project_info: &ProjectInfo) -> Result<()> {
    let testing_file_path = project_info
        .base_dir()
        .join(".github/workflows/deploy_testing.yml");
    let testing_content = create_testing_deploy_file();

    save_file_with_content(&testing_file_path, &testing_content)?;

    let production_file_path = project_info
        .base_dir()
        .join(".github/workflows/deploy_production.yml");
    let production_content = create_production_deploy_file();

    save_file_with_content(&production_file_path, &production_content)?;

    Ok(())
}

pub fn save_pypi_publish_file(project_info: &ProjectInfo) -> Result<()> {
    let file_path = project_info
        .base_dir()
        .join(".github/workflows/pypi_publish.yml");
    let content = match &project_info.project_manager {
        ProjectManager::Maturin => create_pyo3_pypi_publish_file(&project_info.python_version),
        ProjectManager::Poetry => create_poetry_pypi_publish_file(&project_info.python_version),
        ProjectManager::Setuptools => {
            create_setuptools_pypi_publish_file(&project_info.python_version)
        }
        ProjectManager::Uv => create_uv_pypi_publish_file(&project_info.python_version),
    };

    save_file_with_content(&file_path, &content)?;

    Ok(())
}

fn create_poetry_docs_publish_file(python_version: &str) -> String {
    format!(
        r#"name: Docs Publish
on:
  release:
    types:
    - published
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v5
    - name: Install Poetry
      run: pipx install poetry
    - name: Set up Python
      uses: actions/setup-python@v6
      with:
        python-version: "{python_version}"
        cache: "poetry"
    - name: Install Dependencies
      run: |
        poetry install
    - name: Publish package
      run: poetry run mkdocs gh-deploy --force
"#
    )
}

fn create_setuptools_docs_publish_file(python_version: &str) -> String {
    format!(
        r#"name: Docs Publish
on:
  release:
    types:
    - published
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v5
    - name: Set up Python
      uses: actions/setup-python@v6
      with:
        python-version: "{python_version}"
        cache: "pip"
    - name: Install Dependencies
      run: |
        python -m pip install -U pip
        python -m pip install -r requirements-dev.txt
    - name: Publish docs
      run: mkdocs gh-deploy --force
"#
    )
}

fn create_uv_docs_publish_file(python_version: &str) -> String {
    format!(
        r#"name: Docs Publish
on:
  release:
    types:
    - published
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v5
    - name: Install uv
      uses: astral-sh/setup-uv@v7
      with:
        enable-cache: true
    - name: Set up Python
      uses: actions/setup-python@v6
      with:
        python-version: "{python_version}"
    - name: Install Dependencies
      run: uv sync --frozen
    - name: Deploy Docs
      run: uv run mkdocs gh-deploy --force
"#
    )
}

pub fn save_docs_publish_file(project_info: &ProjectInfo) -> Result<()> {
    let file_path = project_info
        .base_dir()
        .join(".github/workflows/docs_publish.yml");
    let content = match &project_info.project_manager {
        ProjectManager::Maturin => {
            if let Some(pyo3_python_manager) = &project_info.pyo3_python_manager {
                match pyo3_python_manager {
                    Pyo3PythonManager::Setuptools => {
                        create_setuptools_docs_publish_file(&project_info.python_version)
                    }
                    Pyo3PythonManager::Uv => {
                        create_uv_docs_publish_file(&project_info.python_version)
                    }
                }
            } else {
                bail!("No PyO3 Python project manager specified");
            }
        }
        ProjectManager::Poetry => create_poetry_docs_publish_file(&project_info.python_version),
        ProjectManager::Setuptools => {
            create_setuptools_docs_publish_file(&project_info.python_version)
        }
        ProjectManager::Uv => create_uv_docs_publish_file(&project_info.python_version),
    };

    save_file_with_content(&file_path, &content)?;

    Ok(())
}

fn create_release_drafter_file() -> String {
    r#"name: Release Drafter

on:
  push:
    branches:
      - main

jobs:
  update_release_draft:
    runs-on: ubuntu-latest
    steps:
      - uses: release-drafter/release-drafter@v6
        with:
          config-name: release_drafter_template.yml
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
"#
    .to_string()
}

fn create_release_drafter_template_file() -> String {
    r#"name-template: 'v$RESOLVED_VERSION'
tag-template: 'v$RESOLVED_VERSION'
exclude-labels:
  - 'dependencies'
  - 'skip-changelog'
version-resolver:
  major:
    labels:
      - 'breaking-change'
  minor:
    labels:
      - 'enhancement'
  default: patch
categories:
  - title: '⚠ Breaking changes'
    label: 'breaking-change'
  - title: 'Features'
    labels: 'enhancement'
  - title: 'Bug Fixes'
    labels: 'bug'
change-template: '- $TITLE @$AUTHOR (#$NUMBER)'
template: |
  ## Changes

  $CHANGES
"#
    .to_string()
}

pub fn save_release_drafter_file(project_info: &ProjectInfo) -> Result<()> {
    let base = project_info.base_dir().join(".github");
    let template_file_path = base.join("release_drafter_template.yml");
    let template_content = create_release_drafter_template_file();

    save_file_with_content(&template_file_path, &template_content)?;

    let file_path = base.join("workflows/release_drafter.yml");
    let content = create_release_drafter_file();

    save_file_with_content(&file_path, &content)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project_info::{
        DocsInfo, LicenseType, ProjectInfo, ProjectManager, Pyo3PythonManager,
    };
    use insta::assert_yaml_snapshot;
    use std::fs::create_dir_all;
    use tmp_path::tmp_path;

    #[cfg(feature = "fastapi")]
    use crate::project_info::DatabaseManager;

    #[tmp_path]
    fn project_info_dummy() -> ProjectInfo {
        ProjectInfo {
            project_name: "My project".to_string(),
            project_slug: "my-project".to_string(),
            source_dir: "my_project".to_string(),
            project_description: "This is a test".to_string(),
            creator: "Arthur Dent".to_string(),
            creator_email: "authur@heartofgold.com".to_string(),
            license: LicenseType::Mit,
            copyright_year: Some("2023".to_string()),
            version: "0.1.0".to_string(),
            python_version: "3.12".to_string(),
            min_python_version: "3.10".to_string(),
            project_manager: ProjectManager::Maturin,
            pyo3_python_manager: Some(Pyo3PythonManager::Uv),
            is_application: true,
            is_async_project: false,
            github_actions_python_test_versions: vec![
                "3.10".to_string(),
                "3.11".to_string(),
                "3.12".to_string(),
                "3.13".to_string(),
                "3.14".to_string(),
            ],
            max_line_length: 100,
            use_dependabot: true,
            dependabot_schedule: None,
            dependabot_day: None,
            use_continuous_deployment: true,
            use_release_drafter: true,
            use_multi_os_ci: true,
            include_docs: false,
            docs_info: None,
            project_root_dir: Some(tmp_path),

            #[cfg(feature = "fastapi")]
            is_fastapi_project: false,

            #[cfg(feature = "fastapi")]
            database_manager: None,
        }
    }

    fn docs_info_dummy() -> DocsInfo {
        DocsInfo {
            site_name: "Test Repo".to_string(),
            site_description: "Dummy data for testing".to_string(),
            site_url: "https://mytest.com".to_string(),
            locale: "en".to_string(),
            repo_name: "sanders41/python-project-generator".to_string(),
            repo_url: "https://github.com/sanders41/python-project-generator".to_string(),
        }
    }

    #[test]
    fn test_build_github_actions_test_versions() {
        assert_eq!(
            build_actions_python_test_versions(&[
                "3.10".to_string(),
                "3.11".to_string(),
                "3.12".to_string(),
                "3.13".to_string(),
                "3.14".to_string(),
            ]),
            r#""3.10", "3.11", "3.12", "3.13", "3.14""#.to_string()
        );
    }

    #[test]
    fn test_save_poetry_ci_testing_linux_only_file() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Poetry;
        project_info.use_multi_os_ci = true;
        let base = project_info.base_dir();
        create_dir_all(base.join(".github/workflows")).unwrap();
        let expected_file = base.join(".github/workflows/testing.yml");
        save_ci_testing_linux_only_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_yaml_snapshot!(content);
    }

    #[cfg(feature = "fastapi")]
    #[test]
    fn test_save_poetry_ci_testing_fastapi_file() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Poetry;
        project_info.is_fastapi_project = true;
        project_info.database_manager = Some(DatabaseManager::AsyncPg);
        let base = project_info.base_dir();
        create_dir_all(base.join(".github/workflows")).unwrap();
        let expected_file = base.join(".github/workflows/testing.yml");
        save_ci_testing_linux_only_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_yaml_snapshot!(content);
    }

    #[test]
    fn test_save_ci_testing_linux_only_file_pyo3() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Maturin;
        let base = project_info.base_dir();
        create_dir_all(base.join(".github/workflows")).unwrap();
        let expected_file = base.join(".github/workflows/testing.yml");
        save_ci_testing_linux_only_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_yaml_snapshot!(content);
    }

    #[cfg(feature = "fastapi")]
    #[test]
    fn test_save_ci_testing_fastapi_file_pyo3() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Maturin;
        project_info.is_fastapi_project = true;
        project_info.database_manager = Some(DatabaseManager::AsyncPg);
        let base = project_info.base_dir();
        create_dir_all(base.join(".github/workflows")).unwrap();
        let expected_file = base.join(".github/workflows/testing.yml");
        save_ci_testing_linux_only_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_yaml_snapshot!(content);
    }

    #[test]
    fn test_save_setuptools_ci_testing_linux_only_file() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Setuptools;
        project_info.use_multi_os_ci = false;
        let base = project_info.base_dir();
        create_dir_all(base.join(".github/workflows")).unwrap();
        let expected_file = base.join(".github/workflows/testing.yml");
        save_ci_testing_linux_only_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_yaml_snapshot!(content);
    }

    #[cfg(feature = "fastapi")]
    #[test]
    fn test_save_setuptools_ci_fastapi_file() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Setuptools;
        project_info.is_fastapi_project = true;
        project_info.database_manager = Some(DatabaseManager::AsyncPg);
        let base = project_info.base_dir();
        create_dir_all(base.join(".github/workflows")).unwrap();
        let expected_file = base.join(".github/workflows/testing.yml");
        save_ci_testing_linux_only_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_yaml_snapshot!(content);
    }

    #[test]
    fn test_save_uv_ci_testing_linux_only_file() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Uv;
        project_info.use_multi_os_ci = false;
        let base = project_info.base_dir();
        create_dir_all(base.join(".github/workflows")).unwrap();
        let expected_file = base.join(".github/workflows/testing.yml");
        save_ci_testing_linux_only_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_yaml_snapshot!(content);
    }

    #[cfg(feature = "fastapi")]
    #[test]
    fn test_save_uv_ci_testing_fastapi_file() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Uv;
        project_info.is_fastapi_project = true;
        project_info.database_manager = Some(DatabaseManager::AsyncPg);
        let base = project_info.base_dir();
        create_dir_all(base.join(".github/workflows")).unwrap();
        let expected_file = base.join(".github/workflows/testing.yml");
        save_ci_testing_linux_only_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_yaml_snapshot!(content);
    }

    #[test]
    fn test_save_poetry_ci_testing_multi_os_file() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Poetry;
        project_info.use_multi_os_ci = true;
        let base = project_info.base_dir();
        create_dir_all(base.join(".github/workflows")).unwrap();
        let expected_file = base.join(".github/workflows/testing.yml");
        save_ci_testing_multi_os_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_yaml_snapshot!(content);
    }

    #[test]
    fn test_save_setuptools_ci_testing_multi_os_file() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Setuptools;
        project_info.use_multi_os_ci = true;
        let base = project_info.base_dir();
        create_dir_all(base.join(".github/workflows")).unwrap();
        let expected_file = base.join(".github/workflows/testing.yml");
        save_ci_testing_multi_os_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_yaml_snapshot!(content);
    }

    #[test]
    fn test_save_ci_testing_multi_os_file_pyo3() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Maturin;
        project_info.use_multi_os_ci = true;
        let base = project_info.base_dir();
        create_dir_all(base.join(".github/workflows")).unwrap();
        let expected_file = base.join(".github/workflows/testing.yml");
        save_ci_testing_multi_os_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_yaml_snapshot!(content);
    }

    #[test]
    fn test_save_uv_ci_testing_multi_os_file() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Uv;
        project_info.use_multi_os_ci = true;
        let base = project_info.base_dir();
        create_dir_all(base.join(".github/workflows")).unwrap();
        let expected_file = base.join(".github/workflows/testing.yml");
        save_ci_testing_multi_os_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_yaml_snapshot!(content);
    }

    #[test]
    fn test_save_dependabot_file() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Poetry;
        project_info.use_dependabot = true;
        project_info.dependabot_schedule = None;
        project_info.dependabot_day = None;
        let base = project_info.base_dir();
        create_dir_all(base.join(".github")).unwrap();
        let expected_file = base.join(".github/dependabot.yml");

        save_dependabot_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_yaml_snapshot!(content);
    }

    #[test]
    fn test_save_dependabot_file_daily() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Poetry;
        project_info.use_dependabot = true;
        project_info.dependabot_schedule = Some(DependabotSchedule::Daily);
        project_info.dependabot_day = None;
        let base = project_info.base_dir();
        create_dir_all(base.join(".github")).unwrap();
        let expected_file = base.join(".github/dependabot.yml");

        save_dependabot_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_yaml_snapshot!(content);
    }

    #[test]
    fn test_save_dependabot_file_weekly_no_day() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Poetry;
        project_info.use_dependabot = true;
        project_info.dependabot_schedule = Some(DependabotSchedule::Weekly);
        project_info.dependabot_day = None;
        let base = project_info.base_dir();
        create_dir_all(base.join(".github")).unwrap();
        let expected_file = base.join(".github/dependabot.yml");

        save_dependabot_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_yaml_snapshot!(content);
    }

    #[test]
    fn test_save_dependabot_file_weekly_tuesday() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Poetry;
        project_info.use_dependabot = true;
        project_info.dependabot_schedule = Some(DependabotSchedule::Weekly);
        project_info.dependabot_day = Some(Day::Tuesday);
        let base = project_info.base_dir();
        create_dir_all(base.join(".github")).unwrap();
        let expected_file = base.join(".github/dependabot.yml");

        save_dependabot_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_yaml_snapshot!(content);
    }

    #[test]
    fn test_save_dependabot_file_monthly() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Poetry;
        project_info.use_dependabot = true;
        project_info.dependabot_schedule = Some(DependabotSchedule::Monthly);
        project_info.dependabot_day = None;
        let base = project_info.base_dir();
        create_dir_all(base.join(".github")).unwrap();
        let expected_file = base.join(".github/dependabot.yml");

        save_dependabot_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_yaml_snapshot!(content);
    }

    #[test]
    fn test_save_dependabot_file_pyo3() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Maturin;
        project_info.dependabot_schedule = None;
        project_info.dependabot_day = None;
        let base = project_info.base_dir();
        create_dir_all(base.join(".github")).unwrap();
        let expected_file = base.join(".github/dependabot.yml");

        save_dependabot_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_yaml_snapshot!(content);
    }

    #[test]
    fn test_save_dependabot_file_pyo3_daily() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Maturin;
        project_info.dependabot_schedule = Some(DependabotSchedule::Daily);
        project_info.dependabot_day = None;
        let base = project_info.base_dir();
        create_dir_all(base.join(".github")).unwrap();
        let expected_file = base.join(".github/dependabot.yml");

        save_dependabot_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_yaml_snapshot!(content);
    }

    #[test]
    fn test_save_dependabot_file_pyo3_weekly() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Maturin;
        project_info.dependabot_schedule = Some(DependabotSchedule::Weekly);
        project_info.dependabot_day = None;
        let base = project_info.base_dir();
        create_dir_all(base.join(".github")).unwrap();
        let expected_file = base.join(".github/dependabot.yml");

        save_dependabot_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_yaml_snapshot!(content);
    }

    #[test]
    fn test_save_dependabot_file_pyo3_weekly_wednesday() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Maturin;
        project_info.dependabot_schedule = Some(DependabotSchedule::Weekly);
        project_info.dependabot_day = Some(Day::Wednesday);
        let base = project_info.base_dir();
        create_dir_all(base.join(".github")).unwrap();
        let expected_file = base.join(".github/dependabot.yml");

        save_dependabot_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_yaml_snapshot!(content);
    }

    #[test]
    fn test_save_dependabot_file_pyo3_monthly() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Maturin;
        project_info.dependabot_schedule = Some(DependabotSchedule::Monthly);
        project_info.dependabot_day = None;
        let base = project_info.base_dir();
        create_dir_all(base.join(".github")).unwrap();
        let expected_file = base.join(".github/dependabot.yml");

        save_dependabot_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_yaml_snapshot!(content);
    }

    #[test]
    fn test_save_pypi_publish_file_poetry() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Poetry;
        let base = project_info.base_dir();
        create_dir_all(base.join(".github/workflows")).unwrap();
        let expected_file = base.join(".github/workflows/pypi_publish.yml");
        save_pypi_publish_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_yaml_snapshot!(content);
    }

    #[test]
    fn test_save_docs_publish_file_poetry() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Poetry;
        project_info.include_docs = true;
        project_info.docs_info = Some(docs_info_dummy());
        let base = project_info.base_dir();
        create_dir_all(base.join(".github/workflows")).unwrap();
        let expected_file = base.join(".github/workflows/docs_publish.yml");
        save_docs_publish_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_yaml_snapshot!(content);
    }

    #[test]
    fn test_save_pypi_publish_file_pyo3() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Maturin;
        let base = project_info.base_dir();
        create_dir_all(base.join(".github/workflows")).unwrap();
        let expected_file = base.join(".github/workflows/pypi_publish.yml");
        save_pypi_publish_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_yaml_snapshot!(content);
    }

    #[test]
    fn test_save_docs_publish_file_pyo3() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Maturin;
        project_info.include_docs = true;
        project_info.docs_info = Some(docs_info_dummy());
        let base = project_info.base_dir();
        create_dir_all(base.join(".github/workflows")).unwrap();
        let expected_file = base.join(".github/workflows/docs_publish.yml");
        save_docs_publish_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_yaml_snapshot!(content);
    }

    #[test]
    fn test_save_pypi_publish_file_setuptools() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Setuptools;
        let base = project_info.base_dir();
        create_dir_all(base.join(".github/workflows")).unwrap();
        let expected_file = base.join(".github/workflows/pypi_publish.yml");
        save_pypi_publish_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_yaml_snapshot!(content);
    }

    #[test]
    fn test_save_docs_publish_file_setuptools() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Setuptools;
        project_info.include_docs = true;
        project_info.docs_info = Some(docs_info_dummy());
        let base = project_info.base_dir();
        create_dir_all(base.join(".github/workflows")).unwrap();
        let expected_file = base.join(".github/workflows/docs_publish.yml");
        save_docs_publish_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_yaml_snapshot!(content);
    }

    #[test]
    fn test_save_pypi_publish_file_uv() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Uv;
        let base = project_info.base_dir();
        create_dir_all(base.join(".github/workflows")).unwrap();
        let expected_file = base.join(".github/workflows/pypi_publish.yml");
        save_pypi_publish_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_yaml_snapshot!(content);
    }

    #[test]
    fn test_save_docs_publish_file_uv() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Uv;
        project_info.include_docs = true;
        project_info.docs_info = Some(docs_info_dummy());
        let base = project_info.base_dir();
        create_dir_all(base.join(".github/workflows")).unwrap();
        let expected_file = base.join(".github/workflows/docs_publish.yml");
        save_docs_publish_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_yaml_snapshot!(content);
    }

    #[cfg(feature = "fastapi")]
    #[test]
    fn test_save_deploy_files() {
        let mut project_info = project_info_dummy();
        project_info.is_fastapi_project = true;
        project_info.database_manager = Some(DatabaseManager::AsyncPg);
        let base = project_info.base_dir();
        create_dir_all(base.join(".github/workflows")).unwrap();
        let expected_test_file = base.join(".github/workflows/deploy_testing.yml");
        let expected_production_file = base.join(".github/workflows/deploy_production.yml");
        save_deploy_files(&project_info).unwrap();

        assert!(expected_test_file.is_file());
        assert!(expected_production_file.is_file());

        let test_content = std::fs::read_to_string(expected_test_file).unwrap();
        let production_content = std::fs::read_to_string(expected_production_file).unwrap();

        assert_yaml_snapshot!(test_content);
        assert_yaml_snapshot!(production_content);
    }

    #[test]
    fn test_save_release_drafter_file() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Poetry;
        let base = project_info.base_dir();
        create_dir_all(base.join(".github/workflows")).unwrap();
        let expected_release_drafter_file = base.join(".github/workflows/release_drafter.yml");
        let expected_release_drafter_template_file =
            base.join(".github//release_drafter_template.yml");

        save_release_drafter_file(&project_info).unwrap();

        assert!(expected_release_drafter_file.is_file());
        assert!(expected_release_drafter_template_file.is_file());

        let release_drafter_file_content =
            std::fs::read_to_string(expected_release_drafter_file).unwrap();

        assert_yaml_snapshot!(release_drafter_file_content);

        let release_drafter_file_template_content =
            std::fs::read_to_string(expected_release_drafter_template_file).unwrap();

        assert_yaml_snapshot!(release_drafter_file_template_content);
    }
}
