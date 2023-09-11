use anyhow::Result;

use crate::file_manager::save_file_with_content;
use crate::project_info::{ProjectInfo, ProjectManager};

fn build_actions_python_test_versions(github_action_python_test_versions: &[String]) -> String {
    github_action_python_test_versions
        .iter()
        .map(|x| format!(r#""{x}""#))
        .collect::<Vec<String>>()
        .join(", ")
}

fn create_ci_testing_linux_only_file(
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
jobs:
  linting:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    - name: Set up Python
      uses: actions/setup-python@v4
      with:
        python-version: "{min_python_version}"
    - name: Get full Python version
      id: full-python-version
      run: echo version=$(python -c "import sys; print('-'.join(str(v) for v in sys.version_info))") >> $GITHUB_OUTPUT
    - name: Install Poetry
      run: |
        pip install -U pip
        pip install pipx
        pipx install poetry
    - name: Configure poetry
      run: |
        poetry config virtualenvs.create true
        poetry config virtualenvs.in-project true
    - name: Cache poetry venv
      uses: actions/cache@v3
      id: poetry-cache
      with:
        path: .venv
        key: venv-${{{{ runner.os }}}}-${{{{ steps.full-python-version.outputs.version }}}}-${{{{ hashFiles('**/poetry.lock') }}}}
    - name: Ensure cache is healthy
      if: steps.poetry-cache.outputs.cache-hit == 'true'
      shell: bash
      run: timeout 10s poetry run pip --version || rm -rf .venv
    - name: Install Dependencies
      run: poetry install
    - name: Black check
      run: |
        poetry run black {source_dir} tests --check
    - name: Lint with ruff
      run: |
        poetry run ruff check .
    - name: mypy check
      run: |
        poetry run mypy .
  testing:
    strategy:
      fail-fast: false
      matrix:
        python-version: [{python_versions}]
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    - name: Set up Python ${{{{ matrix.python-version }}}}
      uses: actions/setup-python@v4
      with:
        python-version: ${{{{ matrix.python-version }}}}
    - name: Get full Python version
      id: full-python-version
      run: echo version=$(python -c "import sys; print('-'.join(str(v) for v in sys.version_info))") >> $GITHUB_OUTPUT
    - name: Install Poetry
      run: |
        pip install -U pip
        pip install pipx
        pipx install poetry
    - name: Configure poetry
      run: |
        poetry config virtualenvs.create true
        poetry config virtualenvs.in-project true
    - name: Cache poetry venv
      uses: actions/cache@v3
      id: poetry-cache
      with:
        path: .venv
        key: venv-${{{{ runner.os }}}}-${{{{ steps.full-python-version.outputs.version }}}}-${{{{ hashFiles('**/poetry.lock') }}}}
    - name: Ensure cache is healthy
      if: steps.poetry-cache.outputs.cache-hit == 'true'
      shell: bash
      run: timeout 10s poetry run pip --version || rm -rf .venv
    - name: Install Dependencies
      run: poetry install
    - name: Test with pytest
      run: |
        poetry run pytest
"#
    )
}

fn create_ci_testing_linux_only_file_pyo3(
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
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1
  RUSTFLAGS: "-D warnings"
jobs:
  clippy:
    name: Clippy
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    - name: Install Rust
      run: |
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    - name: Cache dependencies
      uses: Swatinem/rust-cache@v2.6.2
    - name: Run cargo clippy
      run: cargo clippy --all-targets -- --deny warnings
  fmt:
    name: Rustfmt
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    - name: Install Rust
      run: |
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    - name: Cache dependencies
      uses: Swatinem/rust-cache@v2.6.2
    - name: Run cargo fmt
      run: cargo fmt --all -- --check
  python-linting:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    - name: Set up Python
      uses: actions/setup-python@v4
      with:
        python-version: "{min_python_version}"
    - name: Get full Python version
      id: full-python-version
      run: echo version=$(python -c "import sys; print('-'.join(str(v) for v in sys.version_info))") >> $GITHUB_OUTPUT
    - name: Install Poetry
      run: |
        pip install -U pip
        pip install pipx
        pipx install poetry
    - name: Configure poetry
      run: |
        poetry config virtualenvs.create true
        poetry config virtualenvs.in-project true
    - name: Cache poetry venv
      uses: actions/cache@v3
      id: poetry-cache
      with:
        path: .venv
        key: venv-${{{{ runner.os }}}}-${{{{ steps.full-python-version.outputs.version }}}}-${{{{ hashFiles('**/poetry.lock') }}}}
    - name: Ensure cache is healthy
      if: steps.poetry-cache.outputs.cache-hit == 'true'
      shell: bash
      run: timeout 10s poetry run pip --version || rm -rf .venv
    - name: Install Dependencies
      run: poetry install
    - name: Black check
      run: |
        poetry run black {source_dir} tests --check
    - name: Lint with ruff
      run: |
        poetry run ruff check .
    - name: mypy check
      run: |
        poetry run mypy .
  testing:
    strategy:
      fail-fast: false
      matrix:
        python-version: [{python_versions}]
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    - name: Set up Python ${{{{ matrix.python-version }}}}
      uses: actions/setup-python@v4
      with:
        python-version: ${{{{ matrix.python-version }}}}
    - name: Get full Python version
      id: full-python-version
      run: echo version=$(python -c "import sys; print('-'.join(str(v) for v in sys.version_info))") >> $GITHUB_OUTPUT
    - name: Install Poetry
      run: |
        pip install -U pip
        pip install pipx
        pipx install poetry
    - name: Configure poetry
      run: |
        poetry config virtualenvs.create true
        poetry config virtualenvs.in-project true
    - name: Cache poetry venv
      uses: actions/cache@v3
      id: poetry-cache
      with:
        path: .venv
        key: venv-${{{{ runner.os }}}}-${{{{ steps.full-python-version.outputs.version }}}}-${{{{ hashFiles('**/poetry.lock') }}}}
    - name: Ensure cache is healthy
      if: steps.poetry-cache.outputs.cache-hit == 'true'
      shell: bash
      run: timeout 10s poetry run pip --version || rm -rf .venv
    - name: Install Dependencies
      run: poetry install
    - name: Test with pytest
      run: |
        poetry run pytest
"#
    )
}

pub fn save_ci_testing_linux_only_file(project_info: &ProjectInfo) -> Result<()> {
    let file_path = project_info
        .base_dir()
        .join(".github/workflows/testing.yml");
    let content = match &project_info.project_manager {
        ProjectManager::Maturin => create_ci_testing_linux_only_file_pyo3(
            &project_info.source_dir,
            &project_info.min_python_version,
            &project_info.github_actions_python_test_versions,
        ),
        ProjectManager::Poetry => create_ci_testing_linux_only_file(
            &project_info.source_dir,
            &project_info.min_python_version,
            &project_info.github_actions_python_test_versions,
        ),
    };

    save_file_with_content(&file_path, &content)?;

    Ok(())
}

fn create_ci_testing_multi_os_file(
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
jobs:
  linting:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    - name: Set up Python
      uses: actions/setup-python@v4
      with:
        python-version: "{min_python_version}"
    - name: Get full Python version
      id: full-python-version
      run: echo version=$(python -c "import sys; print('-'.join(str(v) for v in sys.version_info))") >> $GITHUB_OUTPUT
    - name: Install Poetry
      run: |
        pip install -U pip
        pip install pipx
        pipx install poetry
    - name: Configure poetry
      run: |
        poetry config virtualenvs.create true
        poetry config virtualenvs.in-project true
    - name: Cache poetry venv
      uses: actions/cache@v3
      id: poetry-cache
      with:
        path: .venv
        key: venv-${{{{ runner.os }}}}-${{{{ steps.full-python-version.outputs.version }}}}-${{{{ hashFiles('**/poetry.lock') }}}}
    - name: Ensure cache is healthy
      if: steps.poetry-cache.outputs.cache-hit == 'true'
      shell: bash
      run: timeout 10s poetry run pip --version || rm -rf .venv
    - name: Install Dependencies
      run: poetry install
    - name: Black check
      run: |
        poetry run black {source_dir} tests --check
    - name: Lint with ruff
      run: |
        poetry run ruff check .
    - name: mypy check
      run: |
        poetry run mypy .
  testing:
    strategy:
      fail-fast: false
      matrix:
        python-version: [{python_versions}]
        os: [ubuntu-latest, windows-latest, macos-latest]
    runs-on: ${{{{ matrix.os }}}}
    steps:
    - uses: actions/checkout@v4
    - name: Set up Python ${{{{ matrix.python-version }}}}
      uses: actions/setup-python@v4
      with:
        python-version: ${{{{ matrix.python-version }}}}
    - name: Get full Python version
      id: full-python-version
      run: echo version=$(python -c "import sys; print('-'.join(str(v) for v in sys.version_info))") >> $GITHUB_OUTPUT
    - name: Install Poetry
      run: |
        pip install -U pip
        pip install pipx
        pipx install poetry
    - name: Configure poetry
      run: |
        poetry config virtualenvs.create true
        poetry config virtualenvs.in-project true
    - name: Cache poetry venv
      uses: actions/cache@v3
      id: poetry-cache
      with:
        path: .venv
        key: venv-${{{{ runner.os }}}}-${{{{ steps.full-python-version.outputs.version }}}}-${{{{ hashFiles('**/poetry.lock') }}}}
    - name: Ensure cache is healthy
      if: steps.poetry-cache.outputs.cache-hit == 'true'
      shell: bash
      run: timeout 10s poetry run pip --version || rm -rf .venv
    - name: Install Dependencies
      run: poetry install
    - name: Test with pytest
      run: |
        poetry run pytest
"#
    )
}

fn create_ci_testing_multi_os_file_pyo3(
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
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1
  RUSTFLAGS: "-D warnings"
jobs:
  clippy:
    name: Clippy
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    - name: Install Rust
      run: |
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    - name: Cache dependencies
      uses: Swatinem/rust-cache@v2.6.2
    - name: Run cargo clippy
      run: cargo clippy --all-targets -- --deny warnings
  fmt:
    name: Rustfmt
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    - name: Install Rust
      run: |
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    - name: Cache dependencies
      uses: Swatinem/rust-cache@v2.6.2
    - name: Run cargo fmt
      run: cargo fmt --all -- --check
  python-linting:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    - name: Set up Python
      uses: actions/setup-python@v4
      with:
        python-version: "{min_python_version}"
    - name: Get full Python version
      id: full-python-version
      run: echo version=$(python -c "import sys; print('-'.join(str(v) for v in sys.version_info))") >> $GITHUB_OUTPUT
    - name: Install Poetry
      run: |
        pip install -U pip
        pip install pipx
        pipx install poetry
    - name: Configure poetry
      run: |
        poetry config virtualenvs.create true
        poetry config virtualenvs.in-project true
    - name: Cache poetry venv
      uses: actions/cache@v3
      id: poetry-cache
      with:
        path: .venv
        key: venv-${{{{ runner.os }}}}-${{{{ steps.full-python-version.outputs.version }}}}-${{{{ hashFiles('**/poetry.lock') }}}}
    - name: Ensure cache is healthy
      if: steps.poetry-cache.outputs.cache-hit == 'true'
      shell: bash
      run: timeout 10s poetry run pip --version || rm -rf .venv
    - name: Install Dependencies
      run: poetry install
    - name: Black check
      run: |
        poetry run black {source_dir} tests --check
    - name: Lint with ruff
      run: |
        poetry run ruff check .
    - name: mypy check
      run: |
        poetry run mypy .
  testing:
    strategy:
      fail-fast: false
      matrix:
        python-version: [{python_versions}]
        os: [ubuntu-latest, windows-latest, macos-latest]
    runs-on: ${{{{ matrix.os }}}}
    steps:
    - uses: actions/checkout@v4
    - name: Set up Python ${{{{ matrix.python-version }}}}
      uses: actions/setup-python@v4
      with:
        python-version: ${{{{ matrix.python-version }}}}
    - name: Get full Python version
      id: full-python-version
      run: echo version=$(python -c "import sys; print('-'.join(str(v) for v in sys.version_info))") >> $GITHUB_OUTPUT
    - name: Install Poetry
      run: |
        pip install -U pip
        pip install pipx
        pipx install poetry
    - name: Configure poetry
      run: |
        poetry config virtualenvs.create true
        poetry config virtualenvs.in-project true
    - name: Cache poetry venv
      uses: actions/cache@v3
      id: poetry-cache
      with:
        path: .venv
        key: venv-${{{{ runner.os }}}}-${{{{ steps.full-python-version.outputs.version }}}}-${{{{ hashFiles('**/poetry.lock') }}}}
    - name: Ensure cache is healthy
      if: steps.poetry-cache.outputs.cache-hit == 'true'
      shell: bash
      run: timeout 10s poetry run pip --version || rm -rf .venv
    - name: Install Dependencies
      run: poetry install
    - name: Test with pytest
      run: |
        poetry run pytest
"#
    )
}

pub fn save_ci_testing_multi_os_file(project_info: &ProjectInfo) -> Result<()> {
    let file_path = project_info
        .base_dir()
        .join(".github/workflows/testing.yml");
    let content = match &project_info.project_manager {
        ProjectManager::Maturin => create_ci_testing_multi_os_file_pyo3(
            &project_info.source_dir,
            &project_info.min_python_version,
            &project_info.github_actions_python_test_versions,
        ),
        ProjectManager::Poetry => create_ci_testing_multi_os_file(
            &project_info.source_dir,
            &project_info.min_python_version,
            &project_info.github_actions_python_test_versions,
        ),
    };

    save_file_with_content(&file_path, &content)?;

    Ok(())
}

fn create_dependabot_file() -> String {
    r#"version: 2
updates:
  - package-ecosystem: pip
    directory: "/"
    schedule:
      interval: daily
    labels:
    - skip-changelog
    - dependencies
  - package-ecosystem: github-actions
    directory: '/'
    schedule:
      interval: daily
    labels:
    - skip-changelog
    - dependencies
"#
    .to_string()
}

fn create_dependabot_file_pyo3() -> String {
    r#"version: 2
updates:
  - package-ecosystem: pip
    directory: "/"
    schedule:
      interval: daily
    labels:
    - skip-changelog
    - dependencies
  - package-ecosystem: cargo
    directory: "/"
    schedule:
      interval: daily
    labels:
    - skip-changelog
    - dependencies
  - package-ecosystem: github-actions
    directory: '/'
    schedule:
      interval: daily
    labels:
    - skip-changelog
    - dependencies
"#
    .to_string()
}

pub fn save_dependabot_file(project_info: &ProjectInfo) -> Result<()> {
    let file_path = project_info.base_dir().join(".github/dependabot.yml");
    let content = match &project_info.project_manager {
        ProjectManager::Maturin => create_dependabot_file_pyo3(),
        ProjectManager::Poetry => create_dependabot_file(),
    };

    save_file_with_content(&file_path, &content)?;

    Ok(())
}

fn create_pypi_publish_file(python_version: &str) -> String {
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
    - uses: actions/checkout@v4
    - name: Set up Python
      uses: actions/setup-python@v4
      with:
        python-version: "{python_version}"
    - name: Install Poetry
      run: |
        pip install pipx
        pipx install poetry
    - name: Install Dependencies
      run: |
        poetry install
    - name: Add pypi token to Poetry
      run: |
        poetry config pypi-token.pypi {{{{ "${{{{ secrets.PYPI_API_KEY }}}}" }}}}
    - name: Publish package
      run: poetry publish --build
"#
    )
}

fn create_pypi_publish_file_pyo3(python_version: &str) -> String {
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
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v4
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
        uses: actions/upload-artifact@v3
        with:
          name: wheels
          path: dist
  windows:
    runs-on: windows-latest
    strategy:
      matrix:
        target: [x64, x86]
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v4
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
        uses: actions/upload-artifact@v3
        with:
          name: wheels
          path: dist
  macos:
    runs-on: macos-latest
    strategy:
      matrix:
        target: [x86_64, aarch64]
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v4
        with:
          python-version: "{python_version}"
      - name: Build wheels
        uses: PyO3/maturin-action@v1
        with:
          target: ${{{{ matrix.target }}}}
          args: --release --out dist --find-interpreter
          sccache: 'true'
      - name: Upload wheels
        uses: actions/upload-artifact@v3
        with:
          name: wheels
          path: dist
  sdist:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Build sdist
        uses: PyO3/maturin-action@v1
        with:
          command: sdist
          args: --out dist
      - name: Upload sdist
        uses: actions/upload-artifact@v3
        with:
          name: wheels
          path: dist
  release:
    name: Release
    runs-on: ubuntu-latest
    needs: [linux, windows, macos, sdist]
    steps:
      - uses: actions/download-artifact@v3
        with:
          name: wheels
      - name: Publish to PyPI
        uses: PyO3/maturin-action@v1
        env:
          MATURIN_PYPI_TOKEN: ${{{{ secrets.PYPI_API_TOKEN }}}}
        with:
          command: upload
          args: --skip-existing *
"#
    )
}

pub fn save_pypi_publish_file(project_info: &ProjectInfo) -> Result<()> {
    let file_path = project_info
        .base_dir()
        .join(".github/workflows/pypi_publish.yml");
    let content = match &project_info.project_manager {
        ProjectManager::Maturin => create_pypi_publish_file_pyo3(&project_info.python_version),
        ProjectManager::Poetry => create_pypi_publish_file(&project_info.python_version),
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
      - uses: release-drafter/release-drafter@v5
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
  minor:
    labels:
      - 'breaking-change'
      - 'enhancement'
  default: patch
categories:
  - title: 'Features'
    labels:
      - 'enhancement'
  - title: 'Bug Fixes'
    labels:
      - 'bug'
  - title: '⚠ Breaking changes'
    label: 'breaking-change'
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
    use crate::project_info::{LicenseType, ProjectInfo, ProjectManager};
    use std::fs::create_dir_all;
    use tempfile::tempdir;

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
            python_version: "3.11".to_string(),
            min_python_version: "3.8".to_string(),
            project_manager: ProjectManager::Maturin,
            is_application: true,
            github_actions_python_test_versions: vec![
                "3.8".to_string(),
                "3.9".to_string(),
                "3.10".to_string(),
                "3.11".to_string(),
            ],
            max_line_length: 100,
            use_dependabot: true,
            use_continuous_deployment: true,
            use_release_drafter: true,
            use_multi_os_ci: true,
            download_latest_packages: false,
            project_root_dir: Some(tempdir().unwrap().path().to_path_buf()),
        }
    }

    #[test]
    fn test_build_github_actions_test_versions() {
        assert_eq!(
            build_actions_python_test_versions(&[
                "3.8".to_string(),
                "3.9".to_string(),
                "3.10".to_string(),
                "3.11".to_string()
            ]),
            r#""3.8", "3.9", "3.10", "3.11""#.to_string()
        );
    }

    #[test]
    fn test_save_ci_testing_linux_only_file() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Poetry;
        project_info.use_multi_os_ci = true;
        let base = project_info.base_dir();
        create_dir_all(base.join(".github/workflows")).unwrap();
        let expected_file = base.join(".github/workflows/testing.yml");
        let expected = format!(
            r#"name: Testing

on:
  push:
    branches:
    - main
  pull_request:
jobs:
  linting:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    - name: Set up Python
      uses: actions/setup-python@v4
      with:
        python-version: "{}"
    - name: Get full Python version
      id: full-python-version
      run: echo version=$(python -c "import sys; print('-'.join(str(v) for v in sys.version_info))") >> $GITHUB_OUTPUT
    - name: Install Poetry
      run: |
        pip install -U pip
        pip install pipx
        pipx install poetry
    - name: Configure poetry
      run: |
        poetry config virtualenvs.create true
        poetry config virtualenvs.in-project true
    - name: Cache poetry venv
      uses: actions/cache@v3
      id: poetry-cache
      with:
        path: .venv
        key: venv-${{{{ runner.os }}}}-${{{{ steps.full-python-version.outputs.version }}}}-${{{{ hashFiles('**/poetry.lock') }}}}
    - name: Ensure cache is healthy
      if: steps.poetry-cache.outputs.cache-hit == 'true'
      shell: bash
      run: timeout 10s poetry run pip --version || rm -rf .venv
    - name: Install Dependencies
      run: poetry install
    - name: Black check
      run: |
        poetry run black {} tests --check
    - name: Lint with ruff
      run: |
        poetry run ruff check .
    - name: mypy check
      run: |
        poetry run mypy .
  testing:
    strategy:
      fail-fast: false
      matrix:
        python-version: ["3.8", "3.9", "3.10", "3.11"]
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    - name: Set up Python ${{{{ matrix.python-version }}}}
      uses: actions/setup-python@v4
      with:
        python-version: ${{{{ matrix.python-version }}}}
    - name: Get full Python version
      id: full-python-version
      run: echo version=$(python -c "import sys; print('-'.join(str(v) for v in sys.version_info))") >> $GITHUB_OUTPUT
    - name: Install Poetry
      run: |
        pip install -U pip
        pip install pipx
        pipx install poetry
    - name: Configure poetry
      run: |
        poetry config virtualenvs.create true
        poetry config virtualenvs.in-project true
    - name: Cache poetry venv
      uses: actions/cache@v3
      id: poetry-cache
      with:
        path: .venv
        key: venv-${{{{ runner.os }}}}-${{{{ steps.full-python-version.outputs.version }}}}-${{{{ hashFiles('**/poetry.lock') }}}}
    - name: Ensure cache is healthy
      if: steps.poetry-cache.outputs.cache-hit == 'true'
      shell: bash
      run: timeout 10s poetry run pip --version || rm -rf .venv
    - name: Install Dependencies
      run: poetry install
    - name: Test with pytest
      run: |
        poetry run pytest
"#,
            &project_info.min_python_version, &project_info.source_dir
        );

        save_ci_testing_linux_only_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_eq!(content, expected);
    }

    #[test]
    fn test_save_ci_testing_linux_only_file_pyo3() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Maturin;
        let base = project_info.base_dir();
        create_dir_all(base.join(".github/workflows")).unwrap();
        let expected_file = base.join(".github/workflows/testing.yml");
        let expected = format!(
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
jobs:
  clippy:
    name: Clippy
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    - name: Install Rust
      run: |
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    - name: Cache dependencies
      uses: Swatinem/rust-cache@v2.6.2
    - name: Run cargo clippy
      run: cargo clippy --all-targets -- --deny warnings
  fmt:
    name: Rustfmt
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    - name: Install Rust
      run: |
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    - name: Cache dependencies
      uses: Swatinem/rust-cache@v2.6.2
    - name: Run cargo fmt
      run: cargo fmt --all -- --check
  python-linting:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    - name: Set up Python
      uses: actions/setup-python@v4
      with:
        python-version: "{}"
    - name: Get full Python version
      id: full-python-version
      run: echo version=$(python -c "import sys; print('-'.join(str(v) for v in sys.version_info))") >> $GITHUB_OUTPUT
    - name: Install Poetry
      run: |
        pip install -U pip
        pip install pipx
        pipx install poetry
    - name: Configure poetry
      run: |
        poetry config virtualenvs.create true
        poetry config virtualenvs.in-project true
    - name: Cache poetry venv
      uses: actions/cache@v3
      id: poetry-cache
      with:
        path: .venv
        key: venv-${{{{ runner.os }}}}-${{{{ steps.full-python-version.outputs.version }}}}-${{{{ hashFiles('**/poetry.lock') }}}}
    - name: Ensure cache is healthy
      if: steps.poetry-cache.outputs.cache-hit == 'true'
      shell: bash
      run: timeout 10s poetry run pip --version || rm -rf .venv
    - name: Install Dependencies
      run: poetry install
    - name: Black check
      run: |
        poetry run black {} tests --check
    - name: Lint with ruff
      run: |
        poetry run ruff check .
    - name: mypy check
      run: |
        poetry run mypy .
  testing:
    strategy:
      fail-fast: false
      matrix:
        python-version: ["3.8", "3.9", "3.10", "3.11"]
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    - name: Set up Python ${{{{ matrix.python-version }}}}
      uses: actions/setup-python@v4
      with:
        python-version: ${{{{ matrix.python-version }}}}
    - name: Get full Python version
      id: full-python-version
      run: echo version=$(python -c "import sys; print('-'.join(str(v) for v in sys.version_info))") >> $GITHUB_OUTPUT
    - name: Install Poetry
      run: |
        pip install -U pip
        pip install pipx
        pipx install poetry
    - name: Configure poetry
      run: |
        poetry config virtualenvs.create true
        poetry config virtualenvs.in-project true
    - name: Cache poetry venv
      uses: actions/cache@v3
      id: poetry-cache
      with:
        path: .venv
        key: venv-${{{{ runner.os }}}}-${{{{ steps.full-python-version.outputs.version }}}}-${{{{ hashFiles('**/poetry.lock') }}}}
    - name: Ensure cache is healthy
      if: steps.poetry-cache.outputs.cache-hit == 'true'
      shell: bash
      run: timeout 10s poetry run pip --version || rm -rf .venv
    - name: Install Dependencies
      run: poetry install
    - name: Test with pytest
      run: |
        poetry run pytest
"#,
            &project_info.min_python_version, &project_info.source_dir
        );

        save_ci_testing_linux_only_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_eq!(content, expected);
    }

    #[test]
    fn test_save_ci_testing_multi_os_file() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Poetry;
        project_info.use_multi_os_ci = true;
        let base = project_info.base_dir();
        create_dir_all(base.join(".github/workflows")).unwrap();
        let expected_file = base.join(".github/workflows/testing.yml");
        let expected = format!(
            r#"name: Testing

on:
  push:
    branches:
    - main
  pull_request:
jobs:
  linting:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    - name: Set up Python
      uses: actions/setup-python@v4
      with:
        python-version: "{}"
    - name: Get full Python version
      id: full-python-version
      run: echo version=$(python -c "import sys; print('-'.join(str(v) for v in sys.version_info))") >> $GITHUB_OUTPUT
    - name: Install Poetry
      run: |
        pip install -U pip
        pip install pipx
        pipx install poetry
    - name: Configure poetry
      run: |
        poetry config virtualenvs.create true
        poetry config virtualenvs.in-project true
    - name: Cache poetry venv
      uses: actions/cache@v3
      id: poetry-cache
      with:
        path: .venv
        key: venv-${{{{ runner.os }}}}-${{{{ steps.full-python-version.outputs.version }}}}-${{{{ hashFiles('**/poetry.lock') }}}}
    - name: Ensure cache is healthy
      if: steps.poetry-cache.outputs.cache-hit == 'true'
      shell: bash
      run: timeout 10s poetry run pip --version || rm -rf .venv
    - name: Install Dependencies
      run: poetry install
    - name: Black check
      run: |
        poetry run black {} tests --check
    - name: Lint with ruff
      run: |
        poetry run ruff check .
    - name: mypy check
      run: |
        poetry run mypy .
  testing:
    strategy:
      fail-fast: false
      matrix:
        python-version: ["3.8", "3.9", "3.10", "3.11"]
        os: [ubuntu-latest, windows-latest, macos-latest]
    runs-on: ${{{{ matrix.os }}}}
    steps:
    - uses: actions/checkout@v4
    - name: Set up Python ${{{{ matrix.python-version }}}}
      uses: actions/setup-python@v4
      with:
        python-version: ${{{{ matrix.python-version }}}}
    - name: Get full Python version
      id: full-python-version
      run: echo version=$(python -c "import sys; print('-'.join(str(v) for v in sys.version_info))") >> $GITHUB_OUTPUT
    - name: Install Poetry
      run: |
        pip install -U pip
        pip install pipx
        pipx install poetry
    - name: Configure poetry
      run: |
        poetry config virtualenvs.create true
        poetry config virtualenvs.in-project true
    - name: Cache poetry venv
      uses: actions/cache@v3
      id: poetry-cache
      with:
        path: .venv
        key: venv-${{{{ runner.os }}}}-${{{{ steps.full-python-version.outputs.version }}}}-${{{{ hashFiles('**/poetry.lock') }}}}
    - name: Ensure cache is healthy
      if: steps.poetry-cache.outputs.cache-hit == 'true'
      shell: bash
      run: timeout 10s poetry run pip --version || rm -rf .venv
    - name: Install Dependencies
      run: poetry install
    - name: Test with pytest
      run: |
        poetry run pytest
"#,
            &project_info.min_python_version, &project_info.source_dir
        );

        save_ci_testing_multi_os_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_eq!(content, expected);
    }

    #[test]
    fn test_save_ci_testing_multi_os_file_pyo3() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Maturin;
        project_info.use_multi_os_ci = true;
        let base = project_info.base_dir();
        create_dir_all(base.join(".github/workflows")).unwrap();
        let expected_file = base.join(".github/workflows/testing.yml");
        let expected = format!(
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
jobs:
  clippy:
    name: Clippy
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    - name: Install Rust
      run: |
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    - name: Cache dependencies
      uses: Swatinem/rust-cache@v2.6.2
    - name: Run cargo clippy
      run: cargo clippy --all-targets -- --deny warnings
  fmt:
    name: Rustfmt
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    - name: Install Rust
      run: |
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    - name: Cache dependencies
      uses: Swatinem/rust-cache@v2.6.2
    - name: Run cargo fmt
      run: cargo fmt --all -- --check
  python-linting:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    - name: Set up Python
      uses: actions/setup-python@v4
      with:
        python-version: "{}"
    - name: Get full Python version
      id: full-python-version
      run: echo version=$(python -c "import sys; print('-'.join(str(v) for v in sys.version_info))") >> $GITHUB_OUTPUT
    - name: Install Poetry
      run: |
        pip install -U pip
        pip install pipx
        pipx install poetry
    - name: Configure poetry
      run: |
        poetry config virtualenvs.create true
        poetry config virtualenvs.in-project true
    - name: Cache poetry venv
      uses: actions/cache@v3
      id: poetry-cache
      with:
        path: .venv
        key: venv-${{{{ runner.os }}}}-${{{{ steps.full-python-version.outputs.version }}}}-${{{{ hashFiles('**/poetry.lock') }}}}
    - name: Ensure cache is healthy
      if: steps.poetry-cache.outputs.cache-hit == 'true'
      shell: bash
      run: timeout 10s poetry run pip --version || rm -rf .venv
    - name: Install Dependencies
      run: poetry install
    - name: Black check
      run: |
        poetry run black {} tests --check
    - name: Lint with ruff
      run: |
        poetry run ruff check .
    - name: mypy check
      run: |
        poetry run mypy .
  testing:
    strategy:
      fail-fast: false
      matrix:
        python-version: ["3.8", "3.9", "3.10", "3.11"]
        os: [ubuntu-latest, windows-latest, macos-latest]
    runs-on: ${{{{ matrix.os }}}}
    steps:
    - uses: actions/checkout@v4
    - name: Set up Python ${{{{ matrix.python-version }}}}
      uses: actions/setup-python@v4
      with:
        python-version: ${{{{ matrix.python-version }}}}
    - name: Get full Python version
      id: full-python-version
      run: echo version=$(python -c "import sys; print('-'.join(str(v) for v in sys.version_info))") >> $GITHUB_OUTPUT
    - name: Install Poetry
      run: |
        pip install -U pip
        pip install pipx
        pipx install poetry
    - name: Configure poetry
      run: |
        poetry config virtualenvs.create true
        poetry config virtualenvs.in-project true
    - name: Cache poetry venv
      uses: actions/cache@v3
      id: poetry-cache
      with:
        path: .venv
        key: venv-${{{{ runner.os }}}}-${{{{ steps.full-python-version.outputs.version }}}}-${{{{ hashFiles('**/poetry.lock') }}}}
    - name: Ensure cache is healthy
      if: steps.poetry-cache.outputs.cache-hit == 'true'
      shell: bash
      run: timeout 10s poetry run pip --version || rm -rf .venv
    - name: Install Dependencies
      run: poetry install
    - name: Test with pytest
      run: |
        poetry run pytest
"#,
            &project_info.min_python_version, &project_info.source_dir
        );

        save_ci_testing_multi_os_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_eq!(content, expected);
    }

    #[test]
    fn test_save_dependabot_file() {
        let expected = r#"version: 2
updates:
  - package-ecosystem: pip
    directory: "/"
    schedule:
      interval: daily
    labels:
    - skip-changelog
    - dependencies
  - package-ecosystem: github-actions
    directory: '/'
    schedule:
      interval: daily
    labels:
    - skip-changelog
    - dependencies
"#
        .to_string();

        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Poetry;
        let base = project_info.base_dir();
        create_dir_all(base.join(".github")).unwrap();
        let expected_file = base.join(".github/dependabot.yml");

        save_dependabot_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_eq!(content, expected);
    }

    #[test]
    fn test_save_dependabot_file_pyo3() {
        let expected = r#"version: 2
updates:
  - package-ecosystem: pip
    directory: "/"
    schedule:
      interval: daily
    labels:
    - skip-changelog
    - dependencies
  - package-ecosystem: cargo
    directory: "/"
    schedule:
      interval: daily
    labels:
    - skip-changelog
    - dependencies
  - package-ecosystem: github-actions
    directory: '/'
    schedule:
      interval: daily
    labels:
    - skip-changelog
    - dependencies
"#
        .to_string();

        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Maturin;
        let base = project_info.base_dir();
        create_dir_all(base.join(".github")).unwrap();
        let expected_file = base.join(".github/dependabot.yml");

        save_dependabot_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_eq!(content, expected);
    }

    #[test]
    fn test_save_pypi_publish_file() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Poetry;
        let base = project_info.base_dir();
        create_dir_all(base.join(".github/workflows")).unwrap();
        let expected_file = base.join(".github/workflows/pypi_publish.yml");
        let expected = format!(
            r#"name: PyPi Publish
on:
  release:
    types:
    - published
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    - name: Set up Python
      uses: actions/setup-python@v4
      with:
        python-version: "{}"
    - name: Install Poetry
      run: |
        pip install pipx
        pipx install poetry
    - name: Install Dependencies
      run: |
        poetry install
    - name: Add pypi token to Poetry
      run: |
        poetry config pypi-token.pypi {{{{ "${{{{ secrets.PYPI_API_KEY }}}}" }}}}
    - name: Publish package
      run: poetry publish --build
"#,
            &project_info.python_version
        );

        save_pypi_publish_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_eq!(content, expected);
    }

    #[test]
    fn test_save_pypi_publish_file_pyo3() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Maturin;
        let base = project_info.base_dir();
        create_dir_all(base.join(".github/workflows")).unwrap();
        let expected_file = base.join(".github/workflows/pypi_publish.yml");
        let expected = format!(
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
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v4
        with:
          python-version: "{}"
      - name: Build wheels
        uses: PyO3/maturin-action@v1
        with:
          target: ${{{{ matrix.target }}}}
          args: --release --out dist --find-interpreter
          sccache: 'true'
          manylinux: auto
      - name: Upload wheels
        uses: actions/upload-artifact@v3
        with:
          name: wheels
          path: dist
  windows:
    runs-on: windows-latest
    strategy:
      matrix:
        target: [x64, x86]
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v4
        with:
          python-version: "{}"
          architecture: ${{{{ matrix.target }}}}
      - name: Build wheels
        uses: PyO3/maturin-action@v1
        with:
          target: ${{{{ matrix.target }}}}
          args: --release --out dist --find-interpreter
          sccache: 'true'
      - name: Upload wheels
        uses: actions/upload-artifact@v3
        with:
          name: wheels
          path: dist
  macos:
    runs-on: macos-latest
    strategy:
      matrix:
        target: [x86_64, aarch64]
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v4
        with:
          python-version: "{}"
      - name: Build wheels
        uses: PyO3/maturin-action@v1
        with:
          target: ${{{{ matrix.target }}}}
          args: --release --out dist --find-interpreter
          sccache: 'true'
      - name: Upload wheels
        uses: actions/upload-artifact@v3
        with:
          name: wheels
          path: dist
  sdist:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Build sdist
        uses: PyO3/maturin-action@v1
        with:
          command: sdist
          args: --out dist
      - name: Upload sdist
        uses: actions/upload-artifact@v3
        with:
          name: wheels
          path: dist
  release:
    name: Release
    runs-on: ubuntu-latest
    needs: [linux, windows, macos, sdist]
    steps:
      - uses: actions/download-artifact@v3
        with:
          name: wheels
      - name: Publish to PyPI
        uses: PyO3/maturin-action@v1
        env:
          MATURIN_PYPI_TOKEN: ${{{{ secrets.PYPI_API_TOKEN }}}}
        with:
          command: upload
          args: --skip-existing *
"#,
            &project_info.python_version,
            &project_info.python_version,
            &project_info.python_version
        );

        save_pypi_publish_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_eq!(content, expected);
    }

    #[test]
    fn test_save_release_drafter_file() {
        let release_drafer_file_expected = r#"name: Release Drafter

on:
  push:
    branches:
      - main

jobs:
  update_release_draft:
    runs-on: ubuntu-latest
    steps:
      - uses: release-drafter/release-drafter@v5
        with:
          config-name: release_drafter_template.yml
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
"#
        .to_string();

        let expected_release_drafter_template = r#"name-template: 'v$RESOLVED_VERSION'
tag-template: 'v$RESOLVED_VERSION'
exclude-labels:
  - 'dependencies'
  - 'skip-changelog'
version-resolver:
  minor:
    labels:
      - 'breaking-change'
      - 'enhancement'
  default: patch
categories:
  - title: 'Features'
    labels:
      - 'enhancement'
  - title: 'Bug Fixes'
    labels:
      - 'bug'
  - title: '⚠ Breaking changes'
    label: 'breaking-change'
change-template: '- $TITLE @$AUTHOR (#$NUMBER)'
template: |
  ## Changes

  $CHANGES
"#
        .to_string();

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

        assert_eq!(release_drafter_file_content, release_drafer_file_expected);

        let release_drafter_file_template_content =
            std::fs::read_to_string(expected_release_drafter_template_file).unwrap();

        assert_eq!(
            release_drafter_file_template_content,
            expected_release_drafter_template
        );
    }
}
