use anyhow::Result;

use crate::file_manager::save_file_with_content;

fn create_ci_testing_linux_only_file(
    source_dir: &str,
    min_python_version: &str,
    github_action_python_test_versions: &[String],
) -> String {
    let python_versions = github_action_python_test_versions
        .iter()
        .map(|x| format!(r#""{x}""#))
        .collect::<Vec<String>>()
        .join(", ");

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
    - uses: actions/checkout@v3
    - name: Set up Python
      uses: actions/setup-python@v4
      with:
        python-version: "{min_python_version}"
    - name: Get full Python version
      id: full-python-version
      run: echo version=$(python -c "import sys; print('-'.join(str(v) for v in sys.version_info))") >> $GITHUB_OUTPUT
    - name: Install Poetry
      run: |
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
    - uses: actions/checkout@v3
    - name: Set up Python {{{{ matrix.python-version }}}}
      uses: actions/setup-python@v4
      with:
        python-version: {{{{ matrix.python-version }}}}
    - name: Get full Python version
      id: full-python-version
      run: echo version=$(python -c "import sys; print('-'.join(str(v) for v in sys.version_info))") >> $GITHUB_OUTPUT
    - name: Install Poetry
      run: |
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

pub fn save_ci_testing_linux_only_file(
    project_slug: &str,
    source_dir: &str,
    min_python_version: &str,
    github_action_python_test_versions: &[String],
) -> Result<()> {
    let file_path = format!("{project_slug}/.github/workflows/testing.yml");
    let content = create_ci_testing_linux_only_file(
        source_dir,
        min_python_version,
        github_action_python_test_versions,
    );

    save_file_with_content(&file_path, &content)?;

    Ok(())
}

fn create_ci_testing_multi_os_file(
    source_dir: &str,
    min_python_version: &str,
    github_action_python_test_versions: &[String],
) -> String {
    let python_versions = github_action_python_test_versions
        .iter()
        .map(|x| format!(r#""{x}""#))
        .collect::<Vec<String>>()
        .join(", ");

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
    - uses: actions/checkout@v3
    - name: Set up Python
      uses: actions/setup-python@v4
      with:
        python-version: "{min_python_version}"
    - name: Get full Python version
      id: full-python-version
      run: echo version=$(python -c "import sys; print('-'.join(str(v) for v in sys.version_info))") >> $GITHUB_OUTPUT
    - name: Install Poetry
      run: |
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
    runs-on: {{{{matrix.os}}}}
    steps:
    - uses: actions/checkout@v3
    - name: Set up Python {{{{ matrix.python-version }}}}
      uses: actions/setup-python@v4
      with:
        python-version: {{{{ matrix.python-version }}}}
    - name: Get full Python version
      id: full-python-version
      run: echo version=$(python -c "import sys; print('-'.join(str(v) for v in sys.version_info))") >> $GITHUB_OUTPUT
    - name: Install Poetry
      run: |
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

pub fn save_ci_testing_multi_os_file(
    project_slug: &str,
    source_dir: &str,
    min_python_version: &str,
    github_action_python_test_versions: &[String],
) -> Result<()> {
    let file_path = format!("{project_slug}/.github/workflows/testing.yml");
    let content = create_ci_testing_multi_os_file(
        source_dir,
        min_python_version,
        github_action_python_test_versions,
    );

    save_file_with_content(&file_path, &content)?;

    Ok(())
}

fn create_dependabot_file() -> String {
    r#"version: 2
updates:
  - package-ecosystem: "pip"
    directory: "/"
    schedule:
      interval: "daily"
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

pub fn save_dependabot_file(project_slug: &str) -> Result<()> {
    let file_path = format!("{project_slug}/.github/dependabot.yml");
    let content = create_dependabot_file();

    save_file_with_content(&file_path, &content)?;

    Ok(())
}

fn create_pypi_publish_file() -> String {
    r#"name: PyPi Publish
on:
  release:
    types:
    - published
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v2
    - name: Set up Python
      uses: actions/setup-python@v4
      with:
        python-version: "{{ cookiecutter.python_version }}"
    - name: Install Poetry
      run: |
        pip install pipx
        pipx install poetry
    - name: Install Dependencies
      run: |
        poetry install
    - name: Add pypi token to Poetry
      run: |
        poetry config pypi-token.pypi {{ "${{ secrets.PYPI_API_KEY }}" }}
    - name: Publish package
      run: poetry publish --build
"#
    .to_string()
}

pub fn save_pypi_publish_file(project_slug: &str) -> Result<()> {
    let file_path = format!("{project_slug}/.github/workflows/pypi_publish.yml");
    let content = create_pypi_publish_file();

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

pub fn save_release_drafter_file(project_slug: &str) -> Result<()> {
    let template_file_path = format!("{project_slug}/.github/release_drafter_template.yml");
    let template_content = create_release_drafter_template_file();

    save_file_with_content(&template_file_path, &template_content)?;

    let file_path = format!("{project_slug}/.github/workflows/release_drafter.yml");
    let content = create_release_drafter_file();

    save_file_with_content(&file_path, &content)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_ci_testing_linux_only_file() {
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
    - uses: actions/checkout@v3
    - name: Set up Python
      uses: actions/setup-python@v4
      with:
        python-version: "3.8"
    - name: Get full Python version
      id: full-python-version
      run: echo version=$(python -c "import sys; print('-'.join(str(v) for v in sys.version_info))") >> $GITHUB_OUTPUT
    - name: Install Poetry
      run: |
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
        poetry run black src tests --check
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
    - uses: actions/checkout@v3
    - name: Set up Python {{{{ matrix.python-version }}}}
      uses: actions/setup-python@v4
      with:
        python-version: {{{{ matrix.python-version }}}}
    - name: Get full Python version
      id: full-python-version
      run: echo version=$(python -c "import sys; print('-'.join(str(v) for v in sys.version_info))") >> $GITHUB_OUTPUT
    - name: Install Poetry
      run: |
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
        );

        assert_eq!(
            create_ci_testing_linux_only_file(
                "src",
                "3.8",
                &[
                    "3.8".to_string(),
                    "3.9".to_string(),
                    "3.10".to_string(),
                    "3.11".to_string()
                ]
            ),
            expected
        );
    }

    #[test]
    fn test_create_ci_testing_multi_os_file() {
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
    - uses: actions/checkout@v3
    - name: Set up Python
      uses: actions/setup-python@v4
      with:
        python-version: "3.8"
    - name: Get full Python version
      id: full-python-version
      run: echo version=$(python -c "import sys; print('-'.join(str(v) for v in sys.version_info))") >> $GITHUB_OUTPUT
    - name: Install Poetry
      run: |
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
        poetry run black src tests --check
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
    runs-on: {{{{matrix.os}}}}
    steps:
    - uses: actions/checkout@v3
    - name: Set up Python {{{{ matrix.python-version }}}}
      uses: actions/setup-python@v4
      with:
        python-version: {{{{ matrix.python-version }}}}
    - name: Get full Python version
      id: full-python-version
      run: echo version=$(python -c "import sys; print('-'.join(str(v) for v in sys.version_info))") >> $GITHUB_OUTPUT
    - name: Install Poetry
      run: |
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
        );

        assert_eq!(
            create_ci_testing_multi_os_file(
                "src",
                "3.8",
                &[
                    "3.8".to_string(),
                    "3.9".to_string(),
                    "3.10".to_string(),
                    "3.11".to_string()
                ]
            ),
            expected
        );
    }

    #[test]
    fn test_create_dependabot_file() {
        let expected = r#"version: 2
updates:
  - package-ecosystem: "pip"
    directory: "/"
    schedule:
      interval: "daily"
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

        assert_eq!(create_dependabot_file(), expected);
    }

    #[test]
    fn test_create_pypi_publish_file() {
        let expected = r#"name: PyPi Publish
on:
  release:
    types:
    - published
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v2
    - name: Set up Python
      uses: actions/setup-python@v4
      with:
        python-version: "{{ cookiecutter.python_version }}"
    - name: Install Poetry
      run: |
        pip install pipx
        pipx install poetry
    - name: Install Dependencies
      run: |
        poetry install
    - name: Add pypi token to Poetry
      run: |
        poetry config pypi-token.pypi {{ "${{ secrets.PYPI_API_KEY }}" }}
    - name: Publish package
      run: poetry publish --build
"#
        .to_string();

        assert_eq!(create_pypi_publish_file(), expected);
    }

    #[test]
    fn test_create_release_drafter_file() {
        let expected = r#"name: Release Drafter

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

        assert_eq!(create_release_drafter_file(), expected);
    }

    #[test]
    fn test_create_release_drafter_template_file() {
        let expected = r#"name-template: 'v$RESOLVED_VERSION'
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

        assert_eq!(create_release_drafter_template_file(), expected);
    }
}
