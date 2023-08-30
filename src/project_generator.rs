use std::fs::create_dir_all;

use anyhow::Result;
use colored::*;
use minijinja::render;

use crate::file_manager::{save_empty_src_file, save_file_with_content};
use crate::github_actions::{
    save_ci_testing_linux_only_file, save_ci_testing_multi_os_file, save_dependabot_file,
    save_pypi_publish_file, save_release_drafter_file,
};
use crate::licenses::generate_license;
use crate::project_info::ProjectInfo;
use crate::python_files::generate_python_files;
use crate::python_package_version::{PypiPackage, PythonPackageVersion};

fn create_directories(project_slug: &str, source_dir: &str) -> Result<()> {
    let src = format!("{project_slug}/{source_dir}");
    create_dir_all(src)?;

    let github_dir = format!("{project_slug}/.github/workflows");
    create_dir_all(github_dir)?;

    let test_dir = format!("{project_slug}/tests");
    create_dir_all(test_dir)?;

    Ok(())
}

fn create_gitigngore_file() -> String {
    r#"
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
    .to_string()
}

fn save_gitigngore_file(project_slug: &str) -> Result<()> {
    let file_path = format!("{project_slug}/.gitignore");
    let content = create_gitigngore_file();
    save_file_with_content(&file_path, &content)?;

    Ok(())
}

fn create_pre_commit_file(max_line_length: &u8) -> String {
    format!(
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
    rev: v0.0.286
    hooks:
    - id: ruff
      args: [--fix, --exit-non-zero-on-fix]
"#
    )
}

fn save_pre_commit_file(project_slug: &str, max_line_length: &u8) -> Result<()> {
    let file_path = format!("{project_slug}/.pre-commit-config.yml");
    let content = create_pre_commit_file(max_line_length);
    save_file_with_content(&file_path, &content)?;

    Ok(())
}

fn build_latest_dev_dependencies(is_application: bool, use_defaults: bool) -> String {
    let mut version_string = String::new();
    let packages = vec![
        PythonPackageVersion {
            name: "black".to_string(),
            version: "23.7.0".to_string(),
        },
        PythonPackageVersion {
            name: "mypy".to_string(),
            version: "1.5.1".to_string(),
        },
        PythonPackageVersion {
            name: "pre-commit".to_string(),
            version: "3.3.3".to_string(),
        },
        PythonPackageVersion {
            name: "pytest".to_string(),
            version: "7.4.0".to_string(),
        },
        PythonPackageVersion {
            name: "pytest-cov".to_string(),
            version: "4.1.0".to_string(),
        },
        PythonPackageVersion {
            name: "ruff".to_string(),
            version: "0.0.286".to_string(),
        },
        PythonPackageVersion {
            name: "tomli".to_string(),
            version: "2.0.1".to_string(),
        },
    ];

    for package in packages {
        let version: String;
        if !use_defaults {
            package.get_latest_version().unwrap();
            if let Ok(p) = package.get_latest_version() {
                if is_application {
                    version = p.version;
                } else {
                    version = format!(">={}", p.version);
                }
                if p.name == "tomli" {
                    version_string.push_str(&format!(
                        "{} = {{version = \"{}\", python = \"<3.11\"}}\n",
                        p.name, version
                    ));
                } else {
                    version_string.push_str(&format!("{} = \"{}\"\n", p.name, version));
                }
            } else {
                if is_application {
                    version = package.version;
                } else {
                    version = format!(">={}", package.version);
                }
                if package.name == "tomli" {
                    version_string.push_str(&format!(
                        "{} = {{version = \"{}\", python = \"<3.11\"}}\n",
                        package.name, version
                    ));
                } else {
                    version_string.push_str(&format!("{} = \"{}\"\n", package.name, version));
                }
            }
        } else {
            if is_application {
                version = package.version;
            } else {
                version = format!(">={}", package.version);
            }
            if package.name == "tomli" {
                version_string.push_str(&format!(
                    "{} = {{version = \"{}\", python = \"<3.11\"}}\n",
                    package.name, version
                ));
            } else {
                version_string.push_str(&format!("{} = \"{}\"\n", package.name, version));
            }
        }
    }

    println!("{version_string}");
    version_string.trim().to_string()
}

fn create_pyproject_toml(project_info: &ProjectInfo) -> String {
    let pyupgrade_version = &project_info.min_python_version.replace(['.', '^'], "");
    let pyproject = r#"[tool.poetry]
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

[tool.poetry.group.dev.dependencies]
{{ dev_dependencies }}

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

    render!(
        pyproject,
        project_slug => project_info.project_slug,
        version => project_info.version,
        project_description => project_info.project_description,
        creator => project_info.creator,
        creator_email => project_info.creator_email,
        license => format!("{:?}", project_info.license),
        min_python_version => project_info.min_python_version,
        dev_dependencies => build_latest_dev_dependencies(project_info.is_application, project_info.download_latest_packages),
        max_line_length => project_info.max_line_length,
        source_dir => project_info.source_dir,
        is_application => project_info.is_application,
        pyupgrade_version => pyupgrade_version,
    )
}

fn save_pyproject_toml(project_info: &ProjectInfo) -> Result<()> {
    let file_path = format!("{}/pyproject.toml", project_info.project_slug,);
    let content = create_pyproject_toml(project_info);

    save_file_with_content(&file_path, &content)?;

    Ok(())
}

fn create_readme(project_name: &str, project_description: &str) -> String {
    format!(
        r#"# {project_name}

{project_description}
"#
    )
}

fn save_readme(project_slug: &str, project_name: &str, project_description: &str) -> Result<()> {
    let readme_path = format!("{project_slug}/README.md");
    let readme_content = create_readme(project_name, project_description);
    save_file_with_content(&readme_path, &readme_content)?;

    Ok(())
}

pub fn generate_project(project_info: &ProjectInfo) {
    if create_directories(&project_info.project_slug, &project_info.source_dir).is_err() {
        let error_message = "Error creating project directories";
        println!("\n{}", error_message.red());
        std::process::exit(1);
    }

    if save_gitigngore_file(&project_info.project_slug).is_err() {
        let error_message = "Error creating .gitignore file";
        println!("\n{}", error_message.red());
        std::process::exit(1);
    }

    if save_pre_commit_file(&project_info.project_slug, &project_info.max_line_length).is_err() {
        let error_message = "Error creating .gitignore file";
        println!("\n{}", error_message.red());
        std::process::exit(1);
    }

    if save_readme(
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
        &project_info.copyright_year,
        &project_info.project_slug,
        &project_info.creator,
    );

    if save_empty_src_file(
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

    if save_pyproject_toml(project_info).is_err() {
        let error_message = "Error creating pyproject.toml file";
        println!("\n{}", error_message.red());
        std::process::exit(1);
    }

    if save_pypi_publish_file(&project_info.project_slug).is_err() {
        let error_message = "Error creating PYPI publish file";
        println!("\n{}", error_message.red());
        std::process::exit(1);
    }

    if project_info.use_multi_os_ci {
        if save_ci_testing_multi_os_file(
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
    } else if save_ci_testing_linux_only_file(
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

    if project_info.use_dependabot && save_dependabot_file(&project_info.project_slug).is_err() {
        let error_message = "Error creating dependabot file";
        println!("\n{}", error_message.red());
        std::process::exit(1);
    }

    if project_info.use_release_drafter
        && save_release_drafter_file(&project_info.project_slug).is_err()
    {
        let error_message = "Error creating release drafter file";
        println!("\n{}", error_message.red());
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project_info::{LicenseType, ProjectInfo};

    #[test]
    fn test_create_gitigngore_file() {
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

        assert_eq!(create_gitigngore_file(), expected);
    }

    #[test]
    fn test_create_pre_commit_file() {
        let max_line_length: u8 = 100;
        let expected = format!(
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
    rev: v0.0.286
    hooks:
    - id: ruff
      args: [--fix, --exit-non-zero-on-fix]
"#
        );

        assert_eq!(create_pre_commit_file(&max_line_length), expected);
    }

    #[test]
    fn test_create_pyproject_toml_mit_application() {
        let project_info = ProjectInfo {
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
            is_application: true,
            github_action_python_test_versions: vec![
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
        };
        let pyupgrade_version = &project_info.min_python_version.replace(['.', '^'], "");
        let expected = format!(
            r#"[tool.poetry]
name = "{}"
version = "{}"
description = "{}"
authors = ["{} <{}>"]
license = "Mit"
readme = "README.md"

[tool.poetry.dependencies]
python = "^{}"

[tool.poetry.group.dev.dependencies]
black = "23.7.0"
mypy = "1.5.1"
pre-commit = "3.3.3"
pytest = "7.4.0"
pytest-cov = "4.1.0"
ruff = "0.0.286"
tomli = {{version = "2.0.1", python = "<3.11"}}

[build-system]
requires = ["poetry-core>=1.0.0"]
build-backend = "poetry.core.masonry.api"

[tool.black]
line-length = {}
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
addopts = "--cov={} --cov-report term-missing --no-cov-on-fail"

[tool.coverage.report]
exclude_lines = ["if __name__ == .__main__.:", "pragma: no cover"]

[tool.ruff]
select = ["E", "F", "UP", "I001", "T201", "T203"]
ignore = ["E501"]
line-length = {}
target-version = "py{}"
fix = true"#,
            project_info.project_slug,
            project_info.version,
            project_info.project_description,
            project_info.creator,
            project_info.creator_email,
            project_info.min_python_version,
            project_info.max_line_length,
            project_info.source_dir,
            project_info.max_line_length,
            pyupgrade_version,
        );

        println!("{expected}");

        assert_eq!(create_pyproject_toml(&project_info), expected);
    }

    #[test]
    fn test_create_pyproject_toml_apache_application() {
        let project_info = ProjectInfo {
            project_name: "My project".to_string(),
            project_slug: "my-project".to_string(),
            source_dir: "my_project".to_string(),
            project_description: "This is a test".to_string(),
            creator: "Arthur Dent".to_string(),
            creator_email: "authur@heartofgold.com".to_string(),
            license: LicenseType::Apache2,
            copyright_year: Some("2023".to_string()),
            version: "0.1.0".to_string(),
            python_version: "3.11".to_string(),
            min_python_version: "3.8".to_string(),
            is_application: true,
            github_action_python_test_versions: vec![
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
        };
        let pyupgrade_version = &project_info.min_python_version.replace(['.', '^'], "");
        let expected = format!(
            r#"[tool.poetry]
name = "{}"
version = "{}"
description = "{}"
authors = ["{} <{}>"]
license = "Apache2"
readme = "README.md"

[tool.poetry.dependencies]
python = "^{}"

[tool.poetry.group.dev.dependencies]
black = "23.7.0"
mypy = "1.5.1"
pre-commit = "3.3.3"
pytest = "7.4.0"
pytest-cov = "4.1.0"
ruff = "0.0.286"
tomli = {{version = "2.0.1", python = "<3.11"}}

[build-system]
requires = ["poetry-core>=1.0.0"]
build-backend = "poetry.core.masonry.api"

[tool.black]
line-length = {}
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
addopts = "--cov={} --cov-report term-missing --no-cov-on-fail"

[tool.coverage.report]
exclude_lines = ["if __name__ == .__main__.:", "pragma: no cover"]

[tool.ruff]
select = ["E", "F", "UP", "I001", "T201", "T203"]
ignore = ["E501"]
line-length = {}
target-version = "py{}"
fix = true"#,
            project_info.project_slug,
            project_info.version,
            project_info.project_description,
            project_info.creator,
            project_info.creator_email,
            project_info.min_python_version,
            project_info.max_line_length,
            project_info.source_dir,
            project_info.max_line_length,
            pyupgrade_version,
        );

        println!("{expected}");

        assert_eq!(create_pyproject_toml(&project_info), expected);
    }

    #[test]
    fn test_create_pyproject_toml_no_license_application() {
        let project_info = ProjectInfo {
            project_name: "My project".to_string(),
            project_slug: "my-project".to_string(),
            source_dir: "my_project".to_string(),
            project_description: "This is a test".to_string(),
            creator: "Arthur Dent".to_string(),
            creator_email: "authur@heartofgold.com".to_string(),
            license: LicenseType::NoLicense,
            copyright_year: Some("2023".to_string()),
            version: "0.1.0".to_string(),
            python_version: "3.11".to_string(),
            min_python_version: "3.8".to_string(),
            is_application: true,
            github_action_python_test_versions: vec![
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
        };
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

[tool.poetry.group.dev.dependencies]
black = "23.7.0"
mypy = "1.5.1"
pre-commit = "3.3.3"
pytest = "7.4.0"
pytest-cov = "4.1.0"
ruff = "0.0.286"
tomli = {{version = "2.0.1", python = "<3.11"}}

[build-system]
requires = ["poetry-core>=1.0.0"]
build-backend = "poetry.core.masonry.api"

[tool.black]
line-length = {}
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
addopts = "--cov={} --cov-report term-missing --no-cov-on-fail"

[tool.coverage.report]
exclude_lines = ["if __name__ == .__main__.:", "pragma: no cover"]

[tool.ruff]
select = ["E", "F", "UP", "I001", "T201", "T203"]
ignore = ["E501"]
line-length = {}
target-version = "py{}"
fix = true"#,
            project_info.project_slug,
            project_info.version,
            project_info.project_description,
            project_info.creator,
            project_info.creator_email,
            project_info.min_python_version,
            project_info.max_line_length,
            project_info.source_dir,
            project_info.max_line_length,
            pyupgrade_version,
        );

        println!("{expected}");

        assert_eq!(create_pyproject_toml(&project_info), expected);
    }

    #[test]
    fn test_create_pyproject_toml_mit_lib() {
        let project_info = ProjectInfo {
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
            is_application: false,
            github_action_python_test_versions: vec![
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
        };
        let pyupgrade_version = &project_info.min_python_version.replace(['.', '^'], "");
        let expected = format!(
            r#"[tool.poetry]
name = "{}"
version = "{}"
description = "{}"
authors = ["{} <{}>"]
license = "Mit"
readme = "README.md"

[tool.poetry.dependencies]
python = "^{}"

[tool.poetry.group.dev.dependencies]
black = ">=23.7.0"
mypy = ">=1.5.1"
pre-commit = ">=3.3.3"
pytest = ">=7.4.0"
pytest-cov = ">=4.1.0"
ruff = ">=0.0.286"
tomli = {{version = ">=2.0.1", python = "<3.11"}}

[build-system]
requires = ["poetry-core>=1.0.0"]
build-backend = "poetry.core.masonry.api"

[tool.black]
line-length = {}
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
addopts = "--cov={} --cov-report term-missing --no-cov-on-fail"

[tool.coverage.report]
exclude_lines = ["if __name__ == .__main__.:", "pragma: no cover"]

[tool.ruff]
select = ["E", "F", "UP", "I001", "T201", "T203"]
ignore = ["E501"]
line-length = {}
target-version = "py{}"
fix = true"#,
            project_info.project_slug,
            project_info.version,
            project_info.project_description,
            project_info.creator,
            project_info.creator_email,
            project_info.min_python_version,
            project_info.max_line_length,
            project_info.source_dir,
            project_info.max_line_length,
            pyupgrade_version,
        );

        println!("{expected}");

        assert_eq!(create_pyproject_toml(&project_info), expected);
    }

    #[test]
    fn test_create_readme() {
        let project_name = "My Project";
        let project_description = "Some test project";
        let expected = format!(
            r#"# {project_name}

{project_description}
"#
        );

        assert_eq!(create_readme(project_name, project_description), expected);
    }
}
