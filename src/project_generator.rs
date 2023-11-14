use std::fs::create_dir_all;

use anyhow::{bail, Result};
use colored::*;
use minijinja::render;

use crate::file_manager::{save_empty_src_file, save_file_with_content};
use crate::github_actions::{
    save_ci_testing_linux_only_file, save_ci_testing_multi_os_file, save_dependabot_file,
    save_pypi_publish_file, save_release_drafter_file,
};
use crate::licenses::{generate_license, license_str};
use crate::package_version::{
    LatestVersion, PreCommitHook, PreCommitHookVersion, PythonPackageVersion,
};
use crate::project_info::{ProjectInfo, ProjectManager};
use crate::python_files::generate_python_files;
use crate::rust_files::{save_cargo_toml_file, save_lib_file};

fn create_directories(project_info: &ProjectInfo) -> Result<()> {
    let base = project_info.base_dir();
    let src = base.join(&project_info.source_dir);
    create_dir_all(src)?;

    let github_dir = base.join(".github/workflows");
    create_dir_all(github_dir)?;

    let test_dir = base.join("tests");
    create_dir_all(test_dir)?;

    if let ProjectManager::Maturin = &project_info.project_manager {
        let rust_src = base.join("src");
        create_dir_all(rust_src)?;
    }

    Ok(())
}

fn create_gitigngore_file(project_manager: &ProjectManager) -> String {
    let mut gitignore = r#"
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
"#
    .to_string();

    if let ProjectManager::Maturin = project_manager {
        gitignore.push_str(
            r#"
# Rust
/target
"#,
        );
    }

    gitignore
}

fn save_gitigngore_file(project_info: &ProjectInfo) -> Result<()> {
    let file_path = project_info.base_dir().join(".gitignore");
    let content = create_gitigngore_file(&project_info.project_manager);
    save_file_with_content(&file_path, &content)?;

    Ok(())
}

fn build_latest_pre_commit_dependencies(
    download_latest_packages: bool,
) -> Vec<PreCommitHookVersion> {
    let mut hooks = vec![
        PreCommitHookVersion {
            id: PreCommitHook::PreCommit,
            repo: "https://github.com/pre-commit/pre-commit-hooks".to_string(),
            rev: "v4.5.0".to_string(),
        },
        PreCommitHookVersion {
            id: PreCommitHook::MyPy,
            repo: "https://github.com/pre-commit/mirrors-mypy".to_string(),
            rev: "v1.6.1".to_string(),
        },
        PreCommitHookVersion {
            id: PreCommitHook::Ruff,
            repo: "https://github.com/astral-sh/ruff-pre-commit".to_string(),
            rev: "v0.1.5".to_string(),
        },
    ];

    if download_latest_packages {
        for hook in &mut hooks {
            if hook.get_latest_version().is_err() {
                let error_message = format!(
                    "Error retrieving latest pre-commit version for {:?}. Using default.",
                    hook.id
                );
                println!("\n{}", error_message.yellow());
            }
        }
    }

    hooks
}

fn create_pre_commit_file(download_latest_packages: bool) -> String {
    let mut pre_commit_str = "repos:".to_string();
    let hooks = build_latest_pre_commit_dependencies(download_latest_packages);
    for hook in hooks {
        match hook.id {
            PreCommitHook::PreCommit => {
                let info = format!(
                    "\n  - repo: {}\n    rev: {}\n    hooks:\n    - id: check-added-large-files\n    - id: check-toml\n    - id: check-yaml\n    - id: debug-statements\n    - id: end-of-file-fixer\n    - id: trailing-whitespace",
                    hook.repo, hook.rev
                );
                pre_commit_str.push_str(&info);
            }
            PreCommitHook::MyPy => {
                let info = format!(
                    "\n  - repo: {}\n    rev: {}\n    hooks:\n    - id: mypy",
                    hook.repo, hook.rev
                );
                pre_commit_str.push_str(&info);
            }
            PreCommitHook::Ruff => {
                let info = format!(
                    "\n  - repo: {}\n    rev: {}\n    hooks:\n    - id: ruff\n      args: [--fix, --exit-non-zero-on-fix]\n    - id: ruff-format",
                    hook.repo, hook.rev
                );
                pre_commit_str.push_str(&info);
            }
        }
    }

    pre_commit_str.push('\n');
    pre_commit_str
}

fn save_pre_commit_file(project_info: &ProjectInfo) -> Result<()> {
    let file_path = project_info.base_dir().join(".pre-commit-config.yaml");
    let content = create_pre_commit_file(project_info.download_latest_packages);
    save_file_with_content(&file_path, &content)?;

    Ok(())
}

fn build_latest_dev_dependencies(
    is_application: bool,
    download_latest_packages: bool,
    project_manager: &ProjectManager,
) -> String {
    let mut version_string = String::new();
    let mut packages = vec![
        PythonPackageVersion {
            name: "mypy".to_string(),
            version: "1.6.1".to_string(),
        },
        PythonPackageVersion {
            name: "pre-commit".to_string(),
            version: "3.5.0".to_string(),
        },
        PythonPackageVersion {
            name: "pytest".to_string(),
            version: "7.4.2".to_string(),
        },
        PythonPackageVersion {
            name: "pytest-cov".to_string(),
            version: "4.1.0".to_string(),
        },
        PythonPackageVersion {
            name: "ruff".to_string(),
            version: "0.1.5".to_string(),
        },
    ];

    match project_manager {
        ProjectManager::Maturin => packages.push(PythonPackageVersion {
            name: "maturin".to_string(),
            version: "1.3.2".to_string(),
        }),
        ProjectManager::Poetry => packages.push(PythonPackageVersion {
            name: "tomli".to_string(),
            version: "2.0.1".to_string(),
        }),
        ProjectManager::Setuptools => (),
    };

    for mut package in packages {
        if download_latest_packages && package.get_latest_version().is_err() {
            let error_message = format!(
                "Error retrieving latest python package version for {:?}. Using default.",
                package.name
            );
            println!("\n{}", error_message.yellow());
        }

        if let ProjectManager::Poetry = project_manager {
            let version: String = if is_application {
                package.version
            } else {
                format!(">={}", package.version)
            };

            if package.name == "tomli" {
                version_string.push_str(&format!(
                    "{} = {{version = \"{}\", python = \"<3.11\"}}\n",
                    package.name, version
                ));
            } else {
                version_string.push_str(&format!("{} = \"{}\"\n", package.name, version));
            }
        } else if is_application {
            version_string.push_str(&format!("{}=={}\n", package.name, package.version));
        } else {
            version_string.push_str(&format!("{}>={}\n", package.name, package.version));
        }
    }

    if let ProjectManager::Poetry = project_manager {
        version_string.trim().to_string()
    } else {
        version_string.push_str("-e .\n");
        version_string
    }
}

fn create_pyproject_toml(project_info: &ProjectInfo) -> String {
    let pyupgrade_version = &project_info.min_python_version.replace(['.', '^'], "");
    let license_text = license_str(&project_info.license);
    let mut pyproject = match &project_info.project_manager {
        ProjectManager::Maturin => r#"[build-system]
requires = ["maturin>=1.0.0"]
build-backend = "maturin"

[project]
name = "{{ project_name }}"
description = "{{ project_description }}"
authors = [{name = "{{ creator }}", email =  "{{ creator_email }}"}]
{% if license != "NoLicense" -%}
license = "{{ license }}"
{% endif -%}
readme = "README.md"

[tool.maturin]
module-name = "{{ source_dir }}._{{ source_dir }}"
binding = "pyo3"
features = ["pyo3/extension-module"]

"#
        .to_string(),
        ProjectManager::Poetry => r#"[tool.poetry]
name = "{{ project_name }}"
version = "{{ version }}"
description = "{{ project_description }}"
authors = ["{{ creator }} <{{ creator_email }}>"]
{% if license != "NoLicense" -%}
license = "{{ license }}"
{% endif -%}
readme = "README.md"

[tool.poetry.dependencies]
python = "^{{ min_python_version }}"

[tool.poetry.group.dev.dependencies]
{{ dev_dependencies }}

[build-system]
requires = ["poetry-core>=1.0.0"]
build-backend = "poetry.core.masonry.api"

"#
        .to_string(),
        ProjectManager::Setuptools => r#"[build-system]
requires = ["setuptools", "wheel"]
build-backend = "setuptools.build_meta"

[project]
name = "{{ project_name }}"
description = "{{ project_description }}"
authors = [
  { name = "{{ creator }}", email = "{{ creator_email }}" }
]
{% if license != "NoLicense" -%}
license = { text = "{{ license }}" }
{% endif -%}
requires-python = ">={{ min_python_version }}"
dynamic = ["version", "readme"]

[tool.setuptools.dynamic]
version = {attr = "{{ source_dir }}.__version__"}
readme = {file = ["README.md"]}

[tool.setuptools.packages.find]
include = ["{{ source_dir }}*"]

[tool.setuptools.package-data]
{{ source_dir }} = ["py.typed"]

"#
        .to_string(),
    };

    pyproject.push_str(
        r#"[tool.mypy]
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
ignore=[
  # Recommended ignores by ruff when using formatter
  "E501",
  "W191",
  "E111",
  "E114",
  "E117",
  "D206",
  "D300",
  "Q000",
  "Q001",
  "Q002",
  "Q003",
  "COM812",
  "COM819",
  "ISC001",
  "ISC002",
]
line-length = {{ max_line_length }}
target-version = "py{{ pyupgrade_version }}"
fix = true

"#,
    );

    render!(
        &pyproject,
        project_name => project_info.source_dir.replace('_', "-"),
        version => project_info.version,
        project_description => project_info.project_description,
        creator => project_info.creator,
        creator_email => project_info.creator_email,
        license => license_text,
        min_python_version => project_info.min_python_version,
        dev_dependencies => build_latest_dev_dependencies(project_info.is_application, project_info.download_latest_packages, &project_info.project_manager),
        max_line_length => project_info.max_line_length,
        source_dir => project_info.source_dir,
        is_application => project_info.is_application,
        pyupgrade_version => pyupgrade_version,
    )
}

fn save_pyproject_toml_file(project_info: &ProjectInfo) -> Result<()> {
    let file_path = project_info.base_dir().join("pyproject.toml");
    let content = create_pyproject_toml(project_info);

    save_file_with_content(&file_path, &content)?;

    Ok(())
}

fn save_dev_requirements(project_info: &ProjectInfo) -> Result<()> {
    let file_path = project_info.base_dir().join("requirements-dev.txt");
    let content = build_latest_dev_dependencies(
        project_info.is_application,
        project_info.download_latest_packages,
        &project_info.project_manager,
    );

    save_file_with_content(&file_path, &content)?;

    Ok(())
}

fn create_pyo3_justfile(source_dir: &str) -> String {
    format!(
        r#"@develop:
  maturin develop

@install: && develop
  python -m pip install -r requirements-dev.txt

@lint:
  echo cargo check
  just --justfile {{{{justfile()}}}} check
  echo cargo clippy
  just --justfile {{{{justfile()}}}} clippy
  echo cargo fmt
  just --justfile {{{{justfile()}}}} fmt
  echo mypy
  just --justfile {{{{justfile()}}}} mypy
  echo ruff linting
  just --justfile {{{{justfile()}}}} ruff
  echo ruff formatting
  just --justfile {{{{justfile()}}}} ruff-format

@check:
  cargo check

@clippy:
  cargo clippy --all-targets

@fmt:
  cargo fmt --all -- --check

@mypy:
  mypy .

@ruff:
  ruff check . --fix

@ruff-format:
  ruff format {} tests

@test:
  pytest
"#,
        source_dir
    )
}

fn save_pyo3_justfile(project_info: &ProjectInfo) -> Result<()> {
    let file_path = project_info.base_dir().join("justfile");
    let content = create_pyo3_justfile(&project_info.source_dir);

    save_file_with_content(&file_path, &content)?;

    Ok(())
}

fn create_readme_file(project_name: &str, project_description: &str) -> String {
    format!(
        r#"# {project_name}

{project_description}
"#
    )
}

fn save_readme_file(project_info: &ProjectInfo) -> Result<()> {
    let file_path = project_info.base_dir().join("README.md");
    let content = create_readme_file(
        &project_info.project_name,
        &project_info.project_description,
    );
    save_file_with_content(&file_path, &content)?;

    Ok(())
}

pub fn generate_project(project_info: &ProjectInfo) -> Result<()> {
    if create_directories(project_info).is_err() {
        bail!("Error creating project directories");
    }

    if save_gitigngore_file(project_info).is_err() {
        bail!("Error creating .gitignore file");
    }

    if save_pre_commit_file(project_info).is_err() {
        bail!("Error creating .gitignore file");
    }

    if save_readme_file(project_info).is_err() {
        bail!("Error creating README.md file");
    }

    generate_license(project_info)?;

    if save_empty_src_file(project_info, "py.typed").is_err() {
        bail!("Error creating py.typed file");
    }

    generate_python_files(project_info)?;

    if save_pyproject_toml_file(project_info).is_err() {
        bail!("Error creating pyproject.toml file");
    }

    if let ProjectManager::Maturin = &project_info.project_manager {
        if save_dev_requirements(project_info).is_err() {
            bail!("Error creating requirements-dev.txt file");
        }

        if save_pyo3_justfile(project_info).is_err() {
            bail!("Error creating justfile");
        }

        if save_lib_file(project_info).is_err() {
            bail!("Error creating Rust lib.rs file");
        }

        if save_cargo_toml_file(project_info).is_err() {
            bail!("Error creating Rust lib.rs file");
        }
    }

    if let ProjectManager::Setuptools = &project_info.project_manager {
        if save_dev_requirements(project_info).is_err() {
            bail!("Error creating requirements-dev.txt file");
        }
    }

    if project_info.use_continuous_deployment && save_pypi_publish_file(project_info).is_err() {
        bail!("Error creating PYPI publish file");
    }

    if project_info.use_multi_os_ci {
        if save_ci_testing_multi_os_file(project_info).is_err() {
            bail!("Error creating CI teesting file");
        }
    } else if save_ci_testing_linux_only_file(project_info).is_err() {
        bail!("Error creating CI teesting file");
    }

    if project_info.use_dependabot && save_dependabot_file(project_info).is_err() {
        bail!("Error creating dependabot file");
    }

    if project_info.use_release_drafter && save_release_drafter_file(project_info).is_err() {
        bail!("Error creating release drafter file");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project_info::{LicenseType, ProjectInfo};
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
            version: "0.1.5".to_string(),
            python_version: "3.11".to_string(),
            min_python_version: "3.8".to_string(),
            project_manager: ProjectManager::Poetry,
            is_application: true,
            github_actions_python_test_versions: vec![
                "3.8".to_string(),
                "3.9".to_string(),
                "3.10".to_string(),
                "3.11".to_string(),
            ],
            max_line_length: 100,
            use_dependabot: true,
            dependabot_schedule: None,
            dependabot_day: None,
            use_continuous_deployment: true,
            use_release_drafter: true,
            use_multi_os_ci: true,
            download_latest_packages: false,
            project_root_dir: Some(tempdir().unwrap().path().to_path_buf()),
        }
    }

    fn pinned_poetry_dependencies() -> String {
        r#"[tool.poetry.group.dev.dependencies]
mypy = "1.6.1"
pre-commit = "3.5.0"
pytest = "7.4.2"
pytest-cov = "4.1.0"
ruff = "0.1.5"
tomli = {version = "2.0.1", python = "<3.11"}"#
            .to_string()
    }

    fn min_poetry_dependencies() -> String {
        r#"[tool.poetry.group.dev.dependencies]
mypy = ">=1.6.1"
pre-commit = ">=3.5.0"
pytest = ">=7.4.2"
pytest-cov = ">=4.1.0"
ruff = ">=0.1.5"
tomli = {version = ">=2.0.1", python = "<3.11"}"#
            .to_string()
    }

    fn pinned_requirments_file() -> String {
        r#"mypy==1.6.1
pre-commit==3.5.0
pytest==7.4.2
pytest-cov==4.1.0
maturin==1.3.2
ruff==0.1.5
-e .
"#
        .to_string()
    }

    fn min_requirments_file() -> String {
        r#"mypy>=1.6.1
pre-commit>=3.5.0
pytest>=7.4.2
pytest-cov>=4.1.0
maturin>=1.3.2
ruff>=0.1.5
-e .
"#
        .to_string()
    }

    #[test]
    fn test_save_gitigngore_file() {
        let expected = r#"
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
"#
        .to_string();

        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Poetry;
        let base = project_info.base_dir();
        create_dir_all(&base).unwrap();
        let expected_file = base.join(".gitignore");
        save_gitigngore_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_eq!(content, expected);
    }

    #[test]
    fn test_save_gitigngore_pyo3_file() {
        let expected = r#"
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

# Rust
/target
"#
        .to_string();

        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Maturin;
        let base = project_info.base_dir();
        create_dir_all(&base).unwrap();
        let expected_file = base.join(".gitignore");
        save_gitigngore_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_eq!(content, expected);
    }

    #[test]
    fn test_save_pre_commit_file() {
        let expected = r#"repos:
  - repo: https://github.com/pre-commit/pre-commit-hooks
    rev: v4.5.0
    hooks:
    - id: check-added-large-files
    - id: check-toml
    - id: check-yaml
    - id: debug-statements
    - id: end-of-file-fixer
    - id: trailing-whitespace
  - repo: https://github.com/pre-commit/mirrors-mypy
    rev: v1.6.1
    hooks:
    - id: mypy
  - repo: https://github.com/astral-sh/ruff-pre-commit
    rev: v0.1.5
    hooks:
    - id: ruff
      args: [--fix, --exit-non-zero-on-fix]
    - id: ruff-format
"#;

        let project_info = project_info_dummy();
        let base = project_info.base_dir();
        create_dir_all(&base).unwrap();
        let expected_file = base.join(".pre-commit-config.yaml");
        save_pre_commit_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_eq!(content, expected);
    }

    #[test]
    fn test_save_poetry_pyproject_toml_file_mit_application() {
        let mut project_info = project_info_dummy();
        project_info.license = LicenseType::Mit;
        project_info.project_manager = ProjectManager::Poetry;
        project_info.is_application = true;
        let base = project_info.base_dir();
        create_dir_all(&base).unwrap();
        let expected_file = base.join("pyproject.toml");
        let pyupgrade_version = &project_info.min_python_version.replace(['.', '^'], "");
        let expected = format!(
            r#"[tool.poetry]
name = "{}"
version = "{}"
description = "{}"
authors = ["{} <{}>"]
license = "MIT"
readme = "README.md"

[tool.poetry.dependencies]
python = "^{}"

{}

[build-system]
requires = ["poetry-core>=1.0.0"]
build-backend = "poetry.core.masonry.api"

[tool.mypy]
check_untyped_defs = true
disallow_untyped_defs = true

[[tool.mypy.overrides]]
module = ["tests.*"]
disallow_untyped_defs = false

[tool.pytest.ini_options]
minversion = "6.0"
addopts = "--cov={} --cov-report term-missing --no-cov-on-fail"

[tool.coverage.report]
exclude_lines = ["if __name__ == .__main__.:", "pragma: no cover"]

[tool.ruff]
select = ["E", "F", "UP", "I001", "T201", "T203"]
ignore=[
  # Recommended ignores by ruff when using formatter
  "E501",
  "W191",
  "E111",
  "E114",
  "E117",
  "D206",
  "D300",
  "Q000",
  "Q001",
  "Q002",
  "Q003",
  "COM812",
  "COM819",
  "ISC001",
  "ISC002",
]
line-length = {}
target-version = "py{}"
fix = true
"#,
            project_info.source_dir.replace('_', "-"),
            project_info.version,
            project_info.project_description,
            project_info.creator,
            project_info.creator_email,
            project_info.min_python_version,
            pinned_poetry_dependencies(),
            project_info.source_dir,
            project_info.max_line_length,
            pyupgrade_version,
        );

        save_pyproject_toml_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_eq!(content, expected);
    }

    #[test]
    fn test_save_poetry_pyproject_toml_file_apache_application() {
        let mut project_info = project_info_dummy();
        project_info.license = LicenseType::Apache2;
        project_info.project_manager = ProjectManager::Poetry;
        project_info.is_application = true;
        let base = project_info.base_dir();
        create_dir_all(&base).unwrap();
        let expected_file = base.join("pyproject.toml");
        let pyupgrade_version = &project_info.min_python_version.replace(['.', '^'], "");
        let expected = format!(
            r#"[tool.poetry]
name = "{}"
version = "{}"
description = "{}"
authors = ["{} <{}>"]
license = "Apache-2.0"
readme = "README.md"

[tool.poetry.dependencies]
python = "^{}"

{}

[build-system]
requires = ["poetry-core>=1.0.0"]
build-backend = "poetry.core.masonry.api"

[tool.mypy]
check_untyped_defs = true
disallow_untyped_defs = true

[[tool.mypy.overrides]]
module = ["tests.*"]
disallow_untyped_defs = false

[tool.pytest.ini_options]
minversion = "6.0"
addopts = "--cov={} --cov-report term-missing --no-cov-on-fail"

[tool.coverage.report]
exclude_lines = ["if __name__ == .__main__.:", "pragma: no cover"]

[tool.ruff]
select = ["E", "F", "UP", "I001", "T201", "T203"]
ignore=[
  # Recommended ignores by ruff when using formatter
  "E501",
  "W191",
  "E111",
  "E114",
  "E117",
  "D206",
  "D300",
  "Q000",
  "Q001",
  "Q002",
  "Q003",
  "COM812",
  "COM819",
  "ISC001",
  "ISC002",
]
line-length = {}
target-version = "py{}"
fix = true
"#,
            project_info.source_dir.replace('_', "-"),
            project_info.version,
            project_info.project_description,
            project_info.creator,
            project_info.creator_email,
            project_info.min_python_version,
            pinned_poetry_dependencies(),
            project_info.source_dir,
            project_info.max_line_length,
            pyupgrade_version,
        );

        save_pyproject_toml_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_eq!(content, expected);
    }

    #[test]
    fn test_save_poetry_pyproject_toml_file_no_license_application() {
        let mut project_info = project_info_dummy();
        project_info.license = LicenseType::NoLicense;
        project_info.project_manager = ProjectManager::Poetry;
        project_info.is_application = true;
        let base = project_info.base_dir();
        create_dir_all(&base).unwrap();
        let expected_file = base.join("pyproject.toml");
        let pyupgrade_version = &project_info.min_python_version.replace(['.', '^'], "");
        let expected = format!(
            r#"[tool.poetry]
name = "{}"
version = "{}"
description = "{}"
authors = ["{} <{}>"]
readme = "README.md"

[tool.poetry.dependencies]
python = "^{}"

{}

[build-system]
requires = ["poetry-core>=1.0.0"]
build-backend = "poetry.core.masonry.api"

[tool.mypy]
check_untyped_defs = true
disallow_untyped_defs = true

[[tool.mypy.overrides]]
module = ["tests.*"]
disallow_untyped_defs = false

[tool.pytest.ini_options]
minversion = "6.0"
addopts = "--cov={} --cov-report term-missing --no-cov-on-fail"

[tool.coverage.report]
exclude_lines = ["if __name__ == .__main__.:", "pragma: no cover"]

[tool.ruff]
select = ["E", "F", "UP", "I001", "T201", "T203"]
ignore=[
  # Recommended ignores by ruff when using formatter
  "E501",
  "W191",
  "E111",
  "E114",
  "E117",
  "D206",
  "D300",
  "Q000",
  "Q001",
  "Q002",
  "Q003",
  "COM812",
  "COM819",
  "ISC001",
  "ISC002",
]
line-length = {}
target-version = "py{}"
fix = true
"#,
            project_info.source_dir.replace('_', "-"),
            project_info.version,
            project_info.project_description,
            project_info.creator,
            project_info.creator_email,
            project_info.min_python_version,
            pinned_poetry_dependencies(),
            project_info.source_dir,
            project_info.max_line_length,
            pyupgrade_version,
        );

        save_pyproject_toml_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_eq!(content, expected);
    }

    #[test]
    fn test_create_poetry_pyproject_toml_mit_lib() {
        let mut project_info = project_info_dummy();
        project_info.license = LicenseType::Mit;
        project_info.project_manager = ProjectManager::Poetry;
        project_info.is_application = false;
        let base = project_info.base_dir();
        create_dir_all(&base).unwrap();
        let expected_file = base.join("pyproject.toml");
        let pyupgrade_version = &project_info.min_python_version.replace(['.', '^'], "");
        let expected = format!(
            r#"[tool.poetry]
name = "{}"
version = "{}"
description = "{}"
authors = ["{} <{}>"]
license = "MIT"
readme = "README.md"

[tool.poetry.dependencies]
python = "^{}"

{}

[build-system]
requires = ["poetry-core>=1.0.0"]
build-backend = "poetry.core.masonry.api"

[tool.mypy]
check_untyped_defs = true
disallow_untyped_defs = true

[[tool.mypy.overrides]]
module = ["tests.*"]
disallow_untyped_defs = false

[tool.pytest.ini_options]
minversion = "6.0"
addopts = "--cov={} --cov-report term-missing --no-cov-on-fail"

[tool.coverage.report]
exclude_lines = ["if __name__ == .__main__.:", "pragma: no cover"]

[tool.ruff]
select = ["E", "F", "UP", "I001", "T201", "T203"]
ignore=[
  # Recommended ignores by ruff when using formatter
  "E501",
  "W191",
  "E111",
  "E114",
  "E117",
  "D206",
  "D300",
  "Q000",
  "Q001",
  "Q002",
  "Q003",
  "COM812",
  "COM819",
  "ISC001",
  "ISC002",
]
line-length = {}
target-version = "py{}"
fix = true
"#,
            project_info.source_dir.replace('_', "-"),
            project_info.version,
            project_info.project_description,
            project_info.creator,
            project_info.creator_email,
            project_info.min_python_version,
            min_poetry_dependencies(),
            project_info.source_dir,
            project_info.max_line_length,
            pyupgrade_version,
        );

        save_pyproject_toml_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_eq!(content, expected);
    }

    #[test]
    fn test_save_pyproject_toml_file_mit_pyo3() {
        let mut project_info = project_info_dummy();
        project_info.license = LicenseType::Mit;
        project_info.project_manager = ProjectManager::Maturin;
        project_info.is_application = true;
        let base = project_info.base_dir();
        create_dir_all(&base).unwrap();
        let expected_file = base.join("pyproject.toml");
        let pyupgrade_version = &project_info.min_python_version.replace(['.', '^'], "");
        let expected = format!(
            r#"[build-system]
requires = ["maturin>=1.0.0"]
build-backend = "maturin"

[project]
name = "{}"
description = "{}"
authors = [{{name = "{}", email =  "{}"}}]
license = "MIT"
readme = "README.md"

[tool.maturin]
module-name = "{}._{}"
binding = "pyo3"
features = ["pyo3/extension-module"]

[tool.mypy]
check_untyped_defs = true
disallow_untyped_defs = true

[[tool.mypy.overrides]]
module = ["tests.*"]
disallow_untyped_defs = false

[tool.pytest.ini_options]
minversion = "6.0"
addopts = "--cov={} --cov-report term-missing --no-cov-on-fail"

[tool.coverage.report]
exclude_lines = ["if __name__ == .__main__.:", "pragma: no cover"]

[tool.ruff]
select = ["E", "F", "UP", "I001", "T201", "T203"]
ignore=[
  # Recommended ignores by ruff when using formatter
  "E501",
  "W191",
  "E111",
  "E114",
  "E117",
  "D206",
  "D300",
  "Q000",
  "Q001",
  "Q002",
  "Q003",
  "COM812",
  "COM819",
  "ISC001",
  "ISC002",
]
line-length = {}
target-version = "py{}"
fix = true
"#,
            project_info.source_dir.replace('_', "-"),
            project_info.project_description,
            project_info.creator,
            project_info.creator_email,
            project_info.source_dir,
            project_info.source_dir,
            project_info.source_dir,
            project_info.max_line_length,
            pyupgrade_version,
        );

        save_pyproject_toml_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_eq!(content, expected);
    }

    #[test]
    fn test_save_pyproject_toml_file_apache_pyo3() {
        let mut project_info = project_info_dummy();
        project_info.license = LicenseType::Apache2;
        project_info.project_manager = ProjectManager::Maturin;
        project_info.is_application = true;
        let base = project_info.base_dir();
        create_dir_all(&base).unwrap();
        let expected_file = base.join("pyproject.toml");
        let pyupgrade_version = &project_info.min_python_version.replace(['.', '^'], "");
        let expected = format!(
            r#"[build-system]
requires = ["maturin>=1.0.0"]
build-backend = "maturin"

[project]
name = "{}"
description = "{}"
authors = [{{name = "{}", email =  "{}"}}]
license = "Apache-2.0"
readme = "README.md"

[tool.maturin]
module-name = "{}._{}"
binding = "pyo3"
features = ["pyo3/extension-module"]

[tool.mypy]
check_untyped_defs = true
disallow_untyped_defs = true

[[tool.mypy.overrides]]
module = ["tests.*"]
disallow_untyped_defs = false

[tool.pytest.ini_options]
minversion = "6.0"
addopts = "--cov={} --cov-report term-missing --no-cov-on-fail"

[tool.coverage.report]
exclude_lines = ["if __name__ == .__main__.:", "pragma: no cover"]

[tool.ruff]
select = ["E", "F", "UP", "I001", "T201", "T203"]
ignore=[
  # Recommended ignores by ruff when using formatter
  "E501",
  "W191",
  "E111",
  "E114",
  "E117",
  "D206",
  "D300",
  "Q000",
  "Q001",
  "Q002",
  "Q003",
  "COM812",
  "COM819",
  "ISC001",
  "ISC002",
]
line-length = {}
target-version = "py{}"
fix = true
"#,
            project_info.source_dir.replace('_', "-"),
            project_info.project_description,
            project_info.creator,
            project_info.creator_email,
            project_info.source_dir,
            project_info.source_dir,
            project_info.source_dir,
            project_info.max_line_length,
            pyupgrade_version,
        );

        save_pyproject_toml_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_eq!(content, expected);
    }

    #[test]
    fn test_save_pyproject_toml_file_no_license_pyo3() {
        let mut project_info = project_info_dummy();
        project_info.license = LicenseType::NoLicense;
        project_info.project_manager = ProjectManager::Maturin;
        project_info.is_application = true;
        let base = project_info.base_dir();
        create_dir_all(&base).unwrap();
        let expected_file = base.join("pyproject.toml");
        let pyupgrade_version = &project_info.min_python_version.replace(['.', '^'], "");
        let expected = format!(
            r#"[build-system]
requires = ["maturin>=1.0.0"]
build-backend = "maturin"

[project]
name = "{}"
description = "{}"
authors = [{{name = "{}", email =  "{}"}}]
readme = "README.md"

[tool.maturin]
module-name = "{}._{}"
binding = "pyo3"
features = ["pyo3/extension-module"]

[tool.mypy]
check_untyped_defs = true
disallow_untyped_defs = true

[[tool.mypy.overrides]]
module = ["tests.*"]
disallow_untyped_defs = false

[tool.pytest.ini_options]
minversion = "6.0"
addopts = "--cov={} --cov-report term-missing --no-cov-on-fail"

[tool.coverage.report]
exclude_lines = ["if __name__ == .__main__.:", "pragma: no cover"]

[tool.ruff]
select = ["E", "F", "UP", "I001", "T201", "T203"]
ignore=[
  # Recommended ignores by ruff when using formatter
  "E501",
  "W191",
  "E111",
  "E114",
  "E117",
  "D206",
  "D300",
  "Q000",
  "Q001",
  "Q002",
  "Q003",
  "COM812",
  "COM819",
  "ISC001",
  "ISC002",
]
line-length = {}
target-version = "py{}"
fix = true
"#,
            project_info.source_dir.replace('_', "-"),
            project_info.project_description,
            project_info.creator,
            project_info.creator_email,
            project_info.source_dir,
            project_info.source_dir,
            project_info.source_dir,
            project_info.max_line_length,
            pyupgrade_version,
        );

        save_pyproject_toml_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_eq!(content, expected);
    }

    #[test]
    fn test_save_setuptools_pyproject_toml_file_mit_application() {
        let mut project_info = project_info_dummy();
        project_info.license = LicenseType::Mit;
        project_info.project_manager = ProjectManager::Setuptools;
        project_info.is_application = true;
        let base = project_info.base_dir();
        create_dir_all(&base).unwrap();
        let expected_file = base.join("pyproject.toml");
        let pyupgrade_version = &project_info.min_python_version.replace(['.', '^'], "");
        let expected = format!(
            r#"[build-system]
requires = ["setuptools", "wheel"]
build-backend = "setuptools.build_meta"

[project]
name = "{}"
description = "{}"
authors = [
  {{ name = "{}", email = "{}" }}
]
license = {{ text = "MIT" }}
requires-python = ">={}"
dynamic = ["version", "readme"]

[tool.setuptools.dynamic]
version = {{attr = "{}.__version__"}}
readme = {{file = ["README.md"]}}

[tool.setuptools.packages.find]
include = ["{}*"]

[tool.setuptools.package-data]
{} = ["py.typed"]

[tool.mypy]
check_untyped_defs = true
disallow_untyped_defs = true

[[tool.mypy.overrides]]
module = ["tests.*"]
disallow_untyped_defs = false

[tool.pytest.ini_options]
minversion = "6.0"
addopts = "--cov={} --cov-report term-missing --no-cov-on-fail"

[tool.coverage.report]
exclude_lines = ["if __name__ == .__main__.:", "pragma: no cover"]

[tool.ruff]
select = ["E", "F", "UP", "I001", "T201", "T203"]
ignore=[
  # Recommended ignores by ruff when using formatter
  "E501",
  "W191",
  "E111",
  "E114",
  "E117",
  "D206",
  "D300",
  "Q000",
  "Q001",
  "Q002",
  "Q003",
  "COM812",
  "COM819",
  "ISC001",
  "ISC002",
]
line-length = {}
target-version = "py{}"
fix = true
"#,
            project_info.source_dir.replace('_', "-"),
            project_info.project_description,
            project_info.creator,
            project_info.creator_email,
            project_info.min_python_version,
            project_info.source_dir,
            project_info.source_dir,
            project_info.source_dir,
            project_info.source_dir,
            project_info.max_line_length,
            pyupgrade_version,
        );

        save_pyproject_toml_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_eq!(content, expected);
    }

    #[test]
    fn test_save_setuptools_pyproject_toml_file_apache_application() {
        let mut project_info = project_info_dummy();
        project_info.license = LicenseType::Apache2;
        project_info.project_manager = ProjectManager::Setuptools;
        project_info.is_application = true;
        let base = project_info.base_dir();
        create_dir_all(&base).unwrap();
        let expected_file = base.join("pyproject.toml");
        let pyupgrade_version = &project_info.min_python_version.replace(['.', '^'], "");
        let expected = format!(
            r#"[build-system]
requires = ["setuptools", "wheel"]
build-backend = "setuptools.build_meta"

[project]
name = "{}"
description = "{}"
authors = [
  {{ name = "{}", email = "{}" }}
]
license = {{ text = "Apache-2.0" }}
requires-python = ">={}"
dynamic = ["version", "readme"]

[tool.setuptools.dynamic]
version = {{attr = "{}.__version__"}}
readme = {{file = ["README.md"]}}

[tool.setuptools.packages.find]
include = ["{}*"]

[tool.setuptools.package-data]
{} = ["py.typed"]

[tool.mypy]
check_untyped_defs = true
disallow_untyped_defs = true

[[tool.mypy.overrides]]
module = ["tests.*"]
disallow_untyped_defs = false

[tool.pytest.ini_options]
minversion = "6.0"
addopts = "--cov={} --cov-report term-missing --no-cov-on-fail"

[tool.coverage.report]
exclude_lines = ["if __name__ == .__main__.:", "pragma: no cover"]

[tool.ruff]
select = ["E", "F", "UP", "I001", "T201", "T203"]
ignore=[
  # Recommended ignores by ruff when using formatter
  "E501",
  "W191",
  "E111",
  "E114",
  "E117",
  "D206",
  "D300",
  "Q000",
  "Q001",
  "Q002",
  "Q003",
  "COM812",
  "COM819",
  "ISC001",
  "ISC002",
]
line-length = {}
target-version = "py{}"
fix = true
"#,
            project_info.source_dir.replace('_', "-"),
            project_info.project_description,
            project_info.creator,
            project_info.creator_email,
            project_info.min_python_version,
            project_info.source_dir,
            project_info.source_dir,
            project_info.source_dir,
            project_info.source_dir,
            project_info.max_line_length,
            pyupgrade_version,
        );

        save_pyproject_toml_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_eq!(content, expected);
    }

    #[test]
    fn test_save_setuptools_pyproject_toml_file_no_license_application() {
        let mut project_info = project_info_dummy();
        project_info.license = LicenseType::NoLicense;
        project_info.project_manager = ProjectManager::Setuptools;
        project_info.is_application = true;
        let base = project_info.base_dir();
        create_dir_all(&base).unwrap();
        let expected_file = base.join("pyproject.toml");
        let pyupgrade_version = &project_info.min_python_version.replace(['.', '^'], "");
        let expected = format!(
            r#"[build-system]
requires = ["setuptools", "wheel"]
build-backend = "setuptools.build_meta"

[project]
name = "{}"
description = "{}"
authors = [
  {{ name = "{}", email = "{}" }}
]
requires-python = ">={}"
dynamic = ["version", "readme"]

[tool.setuptools.dynamic]
version = {{attr = "{}.__version__"}}
readme = {{file = ["README.md"]}}

[tool.setuptools.packages.find]
include = ["{}*"]

[tool.setuptools.package-data]
{} = ["py.typed"]

[tool.mypy]
check_untyped_defs = true
disallow_untyped_defs = true

[[tool.mypy.overrides]]
module = ["tests.*"]
disallow_untyped_defs = false

[tool.pytest.ini_options]
minversion = "6.0"
addopts = "--cov={} --cov-report term-missing --no-cov-on-fail"

[tool.coverage.report]
exclude_lines = ["if __name__ == .__main__.:", "pragma: no cover"]

[tool.ruff]
select = ["E", "F", "UP", "I001", "T201", "T203"]
ignore=[
  # Recommended ignores by ruff when using formatter
  "E501",
  "W191",
  "E111",
  "E114",
  "E117",
  "D206",
  "D300",
  "Q000",
  "Q001",
  "Q002",
  "Q003",
  "COM812",
  "COM819",
  "ISC001",
  "ISC002",
]
line-length = {}
target-version = "py{}"
fix = true
"#,
            project_info.source_dir.replace('_', "-"),
            project_info.project_description,
            project_info.creator,
            project_info.creator_email,
            project_info.min_python_version,
            project_info.source_dir,
            project_info.source_dir,
            project_info.source_dir,
            project_info.source_dir,
            project_info.max_line_length,
            pyupgrade_version,
        );

        save_pyproject_toml_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_eq!(content, expected);
    }

    #[test]
    fn test_create_setuptools_pyproject_toml_mit_lib() {
        let mut project_info = project_info_dummy();
        project_info.license = LicenseType::Mit;
        project_info.project_manager = ProjectManager::Setuptools;
        project_info.is_application = false;
        let base = project_info.base_dir();
        create_dir_all(&base).unwrap();
        let expected_file = base.join("pyproject.toml");
        let pyupgrade_version = &project_info.min_python_version.replace(['.', '^'], "");
        let expected = format!(
            r#"[build-system]
requires = ["setuptools", "wheel"]
build-backend = "setuptools.build_meta"

[project]
name = "{}"
description = "{}"
authors = [
  {{ name = "{}", email = "{}" }}
]
license = {{ text = "MIT" }}
requires-python = ">={}"
dynamic = ["version", "readme"]

[tool.setuptools.dynamic]
version = {{attr = "{}.__version__"}}
readme = {{file = ["README.md"]}}

[tool.setuptools.packages.find]
include = ["{}*"]

[tool.setuptools.package-data]
{} = ["py.typed"]

[tool.mypy]
check_untyped_defs = true
disallow_untyped_defs = true

[[tool.mypy.overrides]]
module = ["tests.*"]
disallow_untyped_defs = false

[tool.pytest.ini_options]
minversion = "6.0"
addopts = "--cov={} --cov-report term-missing --no-cov-on-fail"

[tool.coverage.report]
exclude_lines = ["if __name__ == .__main__.:", "pragma: no cover"]

[tool.ruff]
select = ["E", "F", "UP", "I001", "T201", "T203"]
ignore=[
  # Recommended ignores by ruff when using formatter
  "E501",
  "W191",
  "E111",
  "E114",
  "E117",
  "D206",
  "D300",
  "Q000",
  "Q001",
  "Q002",
  "Q003",
  "COM812",
  "COM819",
  "ISC001",
  "ISC002",
]
line-length = {}
target-version = "py{}"
fix = true
"#,
            project_info.source_dir.replace('_', "-"),
            project_info.project_description,
            project_info.creator,
            project_info.creator_email,
            project_info.min_python_version,
            project_info.source_dir,
            project_info.source_dir,
            project_info.source_dir,
            project_info.source_dir,
            project_info.max_line_length,
            pyupgrade_version,
        );

        save_pyproject_toml_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_eq!(content, expected);
    }

    #[test]
    fn test_save_pyo3_dev_requirements_application_file() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Maturin;
        project_info.is_application = true;
        let base = project_info.base_dir();
        create_dir_all(&base).unwrap();
        let expected_file = base.join("requirements-dev.txt");
        save_dev_requirements(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_eq!(content, pinned_requirments_file());
    }

    #[test]
    fn test_save_pyo3_dev_requirements_lib_file() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Maturin;
        project_info.is_application = false;
        let base = project_info.base_dir();
        create_dir_all(&base).unwrap();
        let expected_file = base.join("requirements-dev.txt");
        save_dev_requirements(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_eq!(content, min_requirments_file());
    }

    #[test]
    fn test_save_setuptools_dev_requirements_application_file() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Maturin;
        project_info.is_application = true;
        let base = project_info.base_dir();
        create_dir_all(&base).unwrap();
        let expected_file = base.join("requirements-dev.txt");
        save_dev_requirements(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_eq!(content, pinned_requirments_file());
    }

    #[test]
    fn test_save_setuptools_dev_requirements_lib_file() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Maturin;
        project_info.is_application = false;
        let base = project_info.base_dir();
        create_dir_all(&base).unwrap();
        let expected_file = base.join("requirements-dev.txt");
        save_dev_requirements(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_eq!(content, min_requirments_file());
    }

    #[test]
    fn test_save_pyo3_justfile() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Maturin;
        project_info.is_application = false;
        let base = project_info.base_dir();
        create_dir_all(&base).unwrap();
        let expected_file = base.join("justfile");
        let expected = format!(
            r#"@develop:
  maturin develop

@install: && develop
  python -m pip install -r requirements-dev.txt

@lint:
  echo cargo check
  just --justfile {{{{justfile()}}}} check
  echo cargo clippy
  just --justfile {{{{justfile()}}}} clippy
  echo cargo fmt
  just --justfile {{{{justfile()}}}} fmt
  echo mypy
  just --justfile {{{{justfile()}}}} mypy
  echo ruff linting
  just --justfile {{{{justfile()}}}} ruff
  echo ruff formatting
  just --justfile {{{{justfile()}}}} ruff-format

@check:
  cargo check

@clippy:
  cargo clippy --all-targets

@fmt:
  cargo fmt --all -- --check

@mypy:
  mypy .

@ruff:
  ruff check . --fix

@ruff-format:
  ruff format {} tests

@test:
  pytest
"#,
            &project_info.source_dir
        );

        save_pyo3_justfile(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_eq!(content, expected);
    }

    #[test]
    fn test_save_readme_file() {
        let project_info = project_info_dummy();
        let expected = format!(
            r#"# {}

{}
"#,
            &project_info.project_name, &project_info.project_description
        );

        let base = project_info.base_dir();
        create_dir_all(&base).unwrap();
        let expected_file = base.join("README.md");
        save_readme_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_eq!(content, expected);
    }
}
