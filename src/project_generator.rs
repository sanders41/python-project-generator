use std::fs::create_dir_all;

use anyhow::Result;
use colored::*;
use minijinja::render;

use crate::file_manager::{create_empty_src_file, create_file_with_content};
use crate::github_actions::{
    create_ci_testing_linux_only_file, create_ci_testing_multi_os_file, create_dependabot_file,
    create_pypi_publish_file, create_release_drafter_file,
};
use crate::licenses::generate_license;
use crate::project_info::ProjectInfo;
use crate::python_files::generate_python_files;

fn create_directories(project_slug: &str, source_dir: &str) -> Result<()> {
    let src = format!("{project_slug}/{source_dir}");
    create_dir_all(src)?;

    let github_dir = format!("{project_slug}/.github/workflows");
    create_dir_all(github_dir)?;

    let test_dir = format!("{project_slug}/tests");
    create_dir_all(test_dir)?;

    Ok(())
}

fn create_gitigngore_file(project_slug: &str) -> Result<()> {
    let file_path = format!("{project_slug}/.gitignore");
    let content = r#"
# Byte-compiled / optimized / DLL files
__pycache__/
*.py[cod]
*$py.class

# OS Files
*.swp
*.DS_Store

# C extensions
*.so

# Distribution / packaging
.Python
build/
develop-eggs/
dist/
downloads/
eggs/
.eggs/
lib/
lib64/
parts/
sdist/
var/
wheels/
pip-wheel-metadata/
share/python-wheels/
*.egg-info/
.installed.cfg
*.egg
MANIFEST

# PyInstaller
#  Usually these files are written by a python script from a template
#  before PyInstaller builds the exe, so as to inject date/other infos into it.
*.manifest
*.spec

# Installer logs
pip-log.txt
pip-delete-this-directory.txt

# Unit test / coverage reports
htmlcov/
.tox/
.nox/
.coverage
.coverage.*
.cache
nosetests.xml
coverage.xml
*.cover
*.py,cover
.hypothesis/
.pytest_cache/

# Translations
*.mo
*.pot

# Django stuff:
*.log
local_settings.py
db.sqlite3
db.sqlite3-journal

# Flask stuff:
instance/
.webassets-cache

# Scrapy stuff:
.scrapy

# Sphinx documentation
docs/_build/

# PyBuilder
target/

# Jupyter Notebook
.ipynb_checkpoints

# IPython
profile_default/
ipython_config.py

# pyenv
.python-version

# pipenv
#   According to pypa/pipenv#598, it is recommended to include Pipfile.lock in version control.
#   However, in case of collaboration, if having platform-specific dependencies or dependencies
#   having no cross-platform support, pipenv may install dependencies that don't work, or not
#   install all needed dependencies.
#Pipfile.lock

# PEP 582; used by e.g. github.com/David-OConnor/pyflow
__pypackages__/

# Celery stuff
celerybeat-schedule
celerybeat.pid

# SageMath parsed files
*.sage.py

# Environments
.env
.venv
env/
venv/
ENV/
env.bak/
venv.bak/

# Spyder project settings
.spyderproject
.spyproject

# Rope project settings
.ropeproject

# mkdocs documentation
/site

# mypy
.mypy_cache/
.dmypy.json
dmypy.json

# Pyre type checker
.pyre/

# editors
.idea
.vscode

"#;

    create_file_with_content(&file_path, content)?;

    Ok(())
}

fn create_pre_commit_file(project_slug: &str, max_line_length: &u8) -> Result<()> {
    let file_path = format!("{project_slug}/.pre-commit-config.yml");
    let content = format!(
        r#"repos:
  - repo: https://github.com/pre-commit/pre-commit-hooks
    rev: v4.4.0
    hooks:
    - id: check-added-large-files
    - id: check-toml
    - id: check-yaml
    - id: debug-statements
    - id: end-of-file-fixer
    - id: trailing-whitespace
  - repo: https://github.com/psf/black
    rev: 23.7.0
    hooks:
    - id: black
      language_version: python3
      args: [--line-length={max_line_length}]
  - repo: https://github.com/pre-commit/mirrors-mypy
    rev: v1.5.1
    hooks:
    - id: mypy
  - repo: https://github.com/astral-sh/ruff-pre-commit
    rev: v0.0.285
    hooks:
    - id: ruff
      args: [--fix, --exit-non-zero-on-fix]
"#
    );

    create_file_with_content(&file_path, &content)?;

    Ok(())
}

fn create_pyproject_toml(project_info: &ProjectInfo) -> Result<()> {
    let pyproject_path = format!("{}/pyproject.toml", project_info.project_slug,);
    let pyupgrade_version = &project_info.min_python_version.replace(['.', '^'], "");

    let pyprpoject = r#"[tool.poetry]
name = "{{ project_slug }}"
version = "{{ version }}"
description = "{{ project_description }}"
authors = ["{{ creator }} <{{ creator_email }}>"]
{% if license != "NoLicense" -%}
license = "{{ license }}"
{% endif -%}
readme = "README.md"

[tool.poetry.dependencies]
python = "^{{ min_python_version }}"

{% if is_application -%}
[tool.poetry.group.dev.dependencies]
black = "23.7.0"
mypy = "1.5.1"
pre-commit = "3.3.3"
pytest = "7.4.0"
pytest-cov = "4.1.0"
ruff = "0.0.285"
tomli = {version = "2.0.1", python = "<3.11"}
{% else %}
[tool.poetry.group.dev.dependencies]
black = ">=23.7.0"
mypy = ">=1.5.1"
pre-commit = ">=3.3.3"
pytest = ">=7.4.0"
pytest-cov = ">=4.1.0"
ruff = ">=0.0.285"
tomli = {>=version = "2.0.1", python = "<3.11"}
{% endif %}

[build-system]
requires = ["poetry-core>=1.0.0"]
build-backend = "poetry.core.masonry.api"

[tool.black]
line-length = {{ max_line_length }}
include = '\.pyi?$'
exclude = '''
/(
    \.egg
  | \.git
  | \.hg
  | \.mypy_cache
  | \.nox
  | \.tox
  | \.venv
  | \venv
  | _build
  | buck-out
  | build
  | dist
  | setup.py
)/
'''

[tool.mypy]
check_untyped_defs = true
disallow_untyped_defs = true

[[tool.mypy.overrides]]
module = ["tests.*"]
disallow_untyped_defs = false

[tool.pytest.ini_options]
minversion = "6.0"
addopts = "--cov={{ source_dir }} --cov-report term-missing --no-cov-on-fail"

[tool.coverage.report]
exclude_lines = ["if __name__ == .__main__.:", "pragma: no cover"]

[tool.ruff]
select = ["E", "F", "UP", "I001", "T201", "T203"]
ignore = ["E501"]
line-length = {{ max_line_length }}
target-version = "py{{ pyupgrade_version }}"
fix = true
"#;

    let pyproject_toml = render!(
        pyprpoject,
        project_slug => project_info.project_slug,
        version => project_info.version,
        project_description => project_info.project_description,
        creator => project_info.creator,
        creator_email => project_info.creator_email,
        license => format!("{:?}", project_info.license),
        min_python_version => project_info.min_python_version,
        max_line_length => project_info.max_line_length,
        source_dir => project_info.source_dir,
        is_application => project_info.is_application,
        pyupgrade_version => pyupgrade_version,
    );

    create_file_with_content(&pyproject_path, &pyproject_toml)?;

    Ok(())
}

fn create_readme(project_slug: &str, project_name: &str, project_description: &str) -> Result<()> {
    let readme_path = format!("{project_slug}/README.md");
    let readme_content = format!(
        r#"# {}

{}
"#,
        project_name, project_description
    );

    create_file_with_content(&readme_path, &readme_content)?;

    Ok(())
}

pub fn generate_project(project_info: &ProjectInfo) {
    if create_directories(&project_info.project_slug, &project_info.source_dir).is_err() {
        let error_message = "Error creating project directories";
        println!("\n{}", error_message.red());
        std::process::exit(1);
    }

    if create_gitigngore_file(&project_info.project_slug).is_err() {
        let error_message = "Error creating .gitignore file";
        println!("\n{}", error_message.red());
        std::process::exit(1);
    }

    if create_pre_commit_file(&project_info.project_slug, &project_info.max_line_length).is_err() {
        let error_message = "Error creating .gitignore file";
        println!("\n{}", error_message.red());
        std::process::exit(1);
    }

    if create_readme(
        &project_info.project_slug,
        &project_info.project_name,
        &project_info.project_description,
    )
    .is_err()
    {
        let error_message = "Error creating README.md file";
        println!("\n{}", error_message.red());
        std::process::exit(1);
    }

    generate_license(
        &project_info.license,
        &project_info.copywright_year,
        &project_info.project_slug,
        &project_info.creator,
    );

    if create_empty_src_file(
        &project_info.project_slug,
        &project_info.source_dir,
        "py.typed",
    )
    .is_err()
    {
        let error_message = "Error creating py.typed file";
        println!("\n{}", error_message.red());
        std::process::exit(1);
    }

    generate_python_files(
        &project_info.is_application,
        &project_info.project_slug,
        &project_info.source_dir,
        &project_info.version,
    );

    if create_pyproject_toml(project_info).is_err() {
        let error_message = "Error creating pyproject.toml file";
        println!("\n{}", error_message.red());
        std::process::exit(1);
    }

    if create_pypi_publish_file(&project_info.project_slug).is_err() {
        let error_message = "Error creating PYPI publish file";
        println!("\n{}", error_message.red());
        std::process::exit(1);
    }

    if project_info.use_multi_os_ci {
        if create_ci_testing_multi_os_file(
            &project_info.project_slug,
            &project_info.source_dir,
            &project_info.min_python_version,
            &project_info.github_action_python_test_versions,
        )
        .is_err()
        {
            let error_message = "Error creating CI teesting file";
            println!("\n{}", error_message.red());
            std::process::exit(1);
        }
    } else if create_ci_testing_linux_only_file(
        &project_info.project_slug,
        &project_info.source_dir,
        &project_info.min_python_version,
        &project_info.github_action_python_test_versions,
    )
    .is_err()
    {
        let error_message = "Error creating CI teesting file";
        println!("\n{}", error_message.red());
        std::process::exit(1);
    }

    if project_info.use_dependabot && create_dependabot_file(&project_info.project_slug).is_err() {
        let error_message = "Error creating dependabot file";
        println!("\n{}", error_message.red());
        std::process::exit(1);
    }

    if project_info.use_release_drafter
        && create_release_drafter_file(&project_info.project_slug).is_err()
    {
        let error_message = "Error creating release drafter file";
        println!("\n{}", error_message.red());
        std::process::exit(1);
    }
}
