use std::fs::create_dir_all;
use std::path::PathBuf;

use anyhow::Result;
use colored::*;
use minijinja::render;

use crate::file_manager::{save_empty_src_file, save_file_with_content};
use crate::github_actions::{
    save_ci_testing_linux_only_file, save_ci_testing_multi_os_file, save_dependabot_file,
    save_pypi_publish_file, save_release_drafter_file,
};
use crate::licenses::generate_license;
use crate::project_info::{LicenseType, ProjectInfo};
use crate::python_files::generate_python_files;
use crate::python_package_version::{
    LatestVersion, PreCommitHook, PreCommitHookVersion, PythonPackageVersion,
};

fn create_directories(
    project_slug: &str,
    source_dir: &str,
    project_root_dir: &Option<PathBuf>,
) -> Result<()> {
    let base = match project_root_dir {
        Some(root) => format!("{}/{}", root.display(), project_slug),
        None => project_slug.to_string(),
    };
    let src = format!("{base}/{source_dir}");
    create_dir_all(src)?;

    let github_dir = format!("{base}/.github/workflows");
    create_dir_all(github_dir)?;

    let test_dir = format!("{base}/tests");
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

fn save_gitigngore_file(project_slug: &str, project_root_dir: &Option<PathBuf>) -> Result<()> {
    let file_path = match project_root_dir {
        Some(root) => format!("{}/{project_slug}/.gitignore", root.display()),
        None => format!("{project_slug}/.gitignore"),
    };
    let content = create_gitigngore_file();
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
            rev: "v4.4.0".to_string(),
        },
        PreCommitHookVersion {
            id: PreCommitHook::Black,
            repo: "https://github.com/psf/black".to_string(),
            rev: "23.7.0".to_string(),
        },
        PreCommitHookVersion {
            id: PreCommitHook::MyPy,
            repo: "https://github.com/pre-commit/mirrors-mypy".to_string(),
            rev: "v1.5.1".to_string(),
        },
        PreCommitHookVersion {
            id: PreCommitHook::Ruff,
            repo: "https://github.com/astral-sh/ruff-pre-commit".to_string(),
            rev: "v0.0.287".to_string(),
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

fn create_pre_commit_file(max_line_length: &u8, download_latest_packages: bool) -> String {
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
            PreCommitHook::Black => {
                let info = format!(
                    "\n  - repo: {}\n    rev: {}\n    hooks:\n    - id: black\n      language_version: python3\n      args: [--line-length={max_line_length}]",
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
                    "\n  - repo: {}\n    rev: {}\n    hooks:\n    - id: ruff\n      args: [--fix, --exit-non-zero-on-fix]",
                    hook.repo, hook.rev
                );
                pre_commit_str.push_str(&info);
            }
        }
    }

    pre_commit_str.push('\n');
    pre_commit_str
}

fn save_pre_commit_file(
    project_slug: &str,
    max_line_length: &u8,
    download_latest_packages: bool,
    project_root_dir: &Option<PathBuf>,
) -> Result<()> {
    let file_path = match project_root_dir {
        Some(root) => format!("{}/{project_slug}/.pre-commit-config.yaml", root.display()),
        None => format!("{project_slug}/.pre-commit-config.yaml"),
    };
    let content = create_pre_commit_file(max_line_length, download_latest_packages);
    save_file_with_content(&file_path, &content)?;

    Ok(())
}

fn build_latest_dev_dependencies(
    is_application: bool,
    download_latest_packages: bool,
    use_pyo3: bool,
) -> String {
    let mut version_string = String::new();
    let mut packages = vec![
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
            version: "7.4.2".to_string(),
        },
        PythonPackageVersion {
            name: "pytest-cov".to_string(),
            version: "4.1.0".to_string(),
        },
        PythonPackageVersion {
            name: "ruff".to_string(),
            version: "0.0.287".to_string(),
        },
    ];

    if use_pyo3 {
        packages.push(PythonPackageVersion {
            name: "maturin".to_string(),
            version: "1.2.3".to_string(),
        });
    } else {
        packages.push(PythonPackageVersion {
            name: "tomli".to_string(),
            version: "2.0.1".to_string(),
        })
    }

    for mut package in packages {
        if download_latest_packages && package.get_latest_version().is_err() {
            let error_message = format!(
                "Error retrieving latest python package version for {:?}. Using default.",
                package.name
            );
            println!("\n{}", error_message.yellow());
        }

        if use_pyo3 {
            if is_application {
                version_string.push_str(&format!("{}=={}\n", package.name, package.version));
            } else {
                version_string.push_str(&format!("{}>={}\n", package.name, package.version));
            };
        } else {
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
        }
    }

    if use_pyo3 {
        version_string
    } else {
        version_string.trim().to_string()
    }
}

fn create_pyproject_toml(project_info: &ProjectInfo) -> String {
    let pyupgrade_version = &project_info.min_python_version.replace(['.', '^'], "");
    let license_text = match &project_info.license {
        LicenseType::Mit => "MIT",
        LicenseType::Apache2 => "Apache-2.0",
        LicenseType::NoLicense => "NoLicense",
    };

    let mut pyproject = match &project_info.use_pyo3 {
        true => r#"[build-system]
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
        false => r#"[tool.poetry]
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
    };

    pyproject.push_str(
        r#"[tool.black]
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
        dev_dependencies => build_latest_dev_dependencies(project_info.is_application, project_info.download_latest_packages, project_info.use_pyo3),
        max_line_length => project_info.max_line_length,
        source_dir => project_info.source_dir,
        is_application => project_info.is_application,
        pyupgrade_version => pyupgrade_version,
    )
}

fn save_pyproject_toml_file(project_info: &ProjectInfo) -> Result<()> {
    let file_path = match &project_info.project_root_dir {
        Some(root) => format!(
            "{}/{}/pyproject.toml",
            root.display(),
            project_info.project_slug
        ),
        None => format!("{}/pyproject.toml", project_info.project_slug),
    };
    let content = create_pyproject_toml(project_info);

    save_file_with_content(&file_path, &content)?;

    Ok(())
}

fn save_pyo3_dev_requirements(
    project_slug: &str,
    is_application: bool,
    download_latest_packages: bool,
    project_root_dir: &Option<PathBuf>,
) -> Result<()> {
    let file_path = match project_root_dir {
        Some(root) => format!("{}/{project_slug}/requirements-dev.txt", root.display()),
        None => format!("{project_slug}/requirements-dev.txt"),
    };
    let content = build_latest_dev_dependencies(is_application, download_latest_packages, true);

    save_file_with_content(&file_path, &content)?;

    Ok(())
}

fn create_pyo3_justfile(source_dir: &str) -> String {
    format!(
        r#"@develop:
  maturin develop

@install: && develop
  pip install -r requirements-dev.txt

@lint:
  echo cargo check
  just --justfile {{{{justfile()}}}} check
  echo cargo clippy
  just --justfile {{{{justfile()}}}} clippy
  echo cargo fmt
  just --justfile {{{{justfile()}}}} fmt
  echo mypy
  just --justfile {{{{justfile()}}}} mypy
  echo black
  just --justfile {{{{justfile()}}}} black
  echo ruff
  just --justfile {{{{justfile()}}}} ruff

@check:
  cargo check

@clippy:
  cargo clippy

@fmt:
  cargo fmt

@black:
  black {} tests

@mypy:
  mypy .

@ruff:
  ruff check . --fix

@test:
  pytest
"#,
        source_dir
    )
}

fn save_pyo3_justfile(
    project_slug: &str,
    source_dir: &str,
    project_root_dir: &Option<PathBuf>,
) -> Result<()> {
    let file_path = match project_root_dir {
        Some(root) => format!("{}/{project_slug}/justfile", root.display()),
        None => format!("{project_slug}/justfile"),
    };
    let content = create_pyo3_justfile(source_dir);

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

fn save_readme_file(
    project_slug: &str,
    project_name: &str,
    project_description: &str,
    project_root_dir: &Option<PathBuf>,
) -> Result<()> {
    let file_path = match project_root_dir {
        Some(root) => format!("{}/{project_slug}/README.md", root.display()),
        None => format!("{project_slug}/README.md"),
    };
    let content = create_readme_file(project_name, project_description);
    save_file_with_content(&file_path, &content)?;

    Ok(())
}

pub fn generate_project(project_info: &ProjectInfo) {
    if create_directories(
        &project_info.project_slug,
        &project_info.source_dir,
        &project_info.project_root_dir,
    )
    .is_err()
    {
        let error_message = "Error creating project directories";
        println!("\n{}", error_message.red());
        std::process::exit(1);
    }

    if save_gitigngore_file(&project_info.project_slug, &project_info.project_root_dir).is_err() {
        let error_message = "Error creating .gitignore file";
        println!("\n{}", error_message.red());
        std::process::exit(1);
    }

    if save_pre_commit_file(
        &project_info.project_slug,
        &project_info.max_line_length,
        project_info.download_latest_packages,
        &project_info.project_root_dir,
    )
    .is_err()
    {
        let error_message = "Error creating .gitignore file";
        println!("\n{}", error_message.red());
        std::process::exit(1);
    }

    if save_readme_file(
        &project_info.project_slug,
        &project_info.project_name,
        &project_info.project_description,
        &project_info.project_root_dir,
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
        &project_info.project_root_dir,
    );

    if save_empty_src_file(
        &project_info.project_slug,
        &project_info.source_dir,
        "py.typed",
        &project_info.project_root_dir,
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
        &project_info.project_root_dir,
    );

    if save_pyproject_toml_file(project_info).is_err() {
        let error_message = "Error creating pyproject.toml file";
        println!("\n{}", error_message.red());
        std::process::exit(1);
    }

    if project_info.use_pyo3 {
        if save_pyo3_dev_requirements(
            &project_info.project_slug,
            project_info.is_application,
            project_info.download_latest_packages,
            &project_info.project_root_dir,
        )
        .is_err()
        {
            let error_message = "Error creating requirements-dev.txt file";
            println!("\n{}", error_message.red());
            std::process::exit(1);
        }

        if save_pyo3_justfile(
            &project_info.project_slug,
            &project_info.source_dir,
            &project_info.project_root_dir,
        )
        .is_err()
        {
            let error_message = "Error creating justfile";
            println!("\n{}", error_message.red());
            std::process::exit(1);
        }
    }

    if save_pypi_publish_file(&project_info.project_slug, &project_info.project_root_dir).is_err() {
        let error_message = "Error creating PYPI publish file";
        println!("\n{}", error_message.red());
        std::process::exit(1);
    }

    if project_info.use_multi_os_ci {
        if save_ci_testing_multi_os_file(
            &project_info.project_slug,
            &project_info.source_dir,
            &project_info.min_python_version,
            &project_info.github_actions_python_test_versions,
            &project_info.project_root_dir,
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
        &project_info.github_actions_python_test_versions,
        &project_info.project_root_dir,
    )
    .is_err()
    {
        let error_message = "Error creating CI teesting file";
        println!("\n{}", error_message.red());
        std::process::exit(1);
    }

    if project_info.use_dependabot
        && save_dependabot_file(&project_info.project_slug, &project_info.project_root_dir).is_err()
    {
        let error_message = "Error creating dependabot file";
        println!("\n{}", error_message.red());
        std::process::exit(1);
    }

    if project_info.use_release_drafter
        && save_release_drafter_file(&project_info.project_slug, &project_info.project_root_dir)
            .is_err()
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
    use tempfile::tempdir;

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

        let base = tempdir().unwrap().path().to_path_buf();
        let project_slug = "test-project";
        create_dir_all(base.join(project_slug)).unwrap();
        let expected_file = base.join(format!("{project_slug}/.gitignore"));
        save_gitigngore_file(project_slug, &Some(base)).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_eq!(content, expected);
    }

    #[test]
    fn test_save_pre_commit_file() {
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
    rev: v0.0.287
    hooks:
    - id: ruff
      args: [--fix, --exit-non-zero-on-fix]
"#
        );

        let base = tempdir().unwrap().path().to_path_buf();
        let project_slug = "test-project";
        create_dir_all(base.join(project_slug)).unwrap();
        let expected_file = base.join(format!("{project_slug}/.pre-commit-config.yaml"));
        save_pre_commit_file(project_slug, &max_line_length, false, &Some(base)).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_eq!(content, expected);
    }

    #[test]
    fn test_save_pyproject_toml_file_mit_application() {
        let base = tempdir().unwrap().path().to_path_buf();
        let project_slug = "test-project";
        create_dir_all(base.join(project_slug)).unwrap();
        let expected_file = base.join(format!("{project_slug}/pyproject.toml"));

        let project_info = ProjectInfo {
            project_name: "My project".to_string(),
            project_slug: project_slug.to_string(),
            source_dir: "my_project".to_string(),
            project_description: "This is a test".to_string(),
            creator: "Arthur Dent".to_string(),
            creator_email: "authur@heartofgold.com".to_string(),
            license: LicenseType::Mit,
            copyright_year: Some("2023".to_string()),
            version: "0.1.0".to_string(),
            python_version: "3.11".to_string(),
            min_python_version: "3.8".to_string(),
            use_pyo3: false,
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
            project_root_dir: Some(base),
        };
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

[tool.poetry.group.dev.dependencies]
black = "23.7.0"
mypy = "1.5.1"
pre-commit = "3.3.3"
pytest = "7.4.2"
pytest-cov = "4.1.0"
ruff = "0.0.287"
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
fix = true
"#,
            project_info.source_dir.replace('_', "-"),
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

        save_pyproject_toml_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_eq!(content, expected);
    }

    #[test]
    fn test_save_pyproject_toml_file_apache_application() {
        let base = tempdir().unwrap().path().to_path_buf();
        let project_slug = "test-project";
        create_dir_all(base.join(project_slug)).unwrap();
        let expected_file = base.join(format!("{project_slug}/pyproject.toml"));

        let project_info = ProjectInfo {
            project_name: "My project".to_string(),
            project_slug: project_slug.to_string(),
            source_dir: "my_project".to_string(),
            project_description: "This is a test".to_string(),
            creator: "Arthur Dent".to_string(),
            creator_email: "authur@heartofgold.com".to_string(),
            license: LicenseType::Apache2,
            copyright_year: Some("2023".to_string()),
            version: "0.1.0".to_string(),
            python_version: "3.11".to_string(),
            min_python_version: "3.8".to_string(),
            use_pyo3: false,
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
            project_root_dir: Some(base),
        };
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

[tool.poetry.group.dev.dependencies]
black = "23.7.0"
mypy = "1.5.1"
pre-commit = "3.3.3"
pytest = "7.4.2"
pytest-cov = "4.1.0"
ruff = "0.0.287"
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
fix = true
"#,
            project_info.source_dir.replace('_', "-"),
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

        save_pyproject_toml_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_eq!(content, expected);
    }

    #[test]
    fn test_save_pyproject_toml_file_no_license_application() {
        let base = tempdir().unwrap().path().to_path_buf();
        let project_slug = "test-project";
        create_dir_all(base.join(project_slug)).unwrap();
        let expected_file = base.join(format!("{project_slug}/pyproject.toml"));

        let project_info = ProjectInfo {
            project_name: "My project".to_string(),
            project_slug: project_slug.to_string(),
            source_dir: "my_project".to_string(),
            project_description: "This is a test".to_string(),
            creator: "Arthur Dent".to_string(),
            creator_email: "authur@heartofgold.com".to_string(),
            license: LicenseType::NoLicense,
            copyright_year: Some("2023".to_string()),
            version: "0.1.0".to_string(),
            python_version: "3.11".to_string(),
            min_python_version: "3.8".to_string(),
            use_pyo3: false,
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
            project_root_dir: Some(base),
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
pytest = "7.4.2"
pytest-cov = "4.1.0"
ruff = "0.0.287"
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
fix = true
"#,
            project_info.source_dir.replace('_', "-"),
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

        save_pyproject_toml_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_eq!(content, expected);
    }

    #[test]
    fn test_create_pyproject_toml_mit_lib() {
        let base = tempdir().unwrap().path().to_path_buf();
        let project_slug = "test-project";
        create_dir_all(base.join(project_slug)).unwrap();
        let expected_file = base.join(format!("{project_slug}/pyproject.toml"));

        let project_info = ProjectInfo {
            project_name: "My project".to_string(),
            project_slug: project_slug.to_string(),
            source_dir: "my_project".to_string(),
            project_description: "This is a test".to_string(),
            creator: "Arthur Dent".to_string(),
            creator_email: "authur@heartofgold.com".to_string(),
            license: LicenseType::Mit,
            copyright_year: Some("2023".to_string()),
            version: "0.1.0".to_string(),
            python_version: "3.11".to_string(),
            min_python_version: "3.8".to_string(),
            use_pyo3: false,
            is_application: false,
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
            project_root_dir: Some(base),
        };
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

[tool.poetry.group.dev.dependencies]
black = ">=23.7.0"
mypy = ">=1.5.1"
pre-commit = ">=3.3.3"
pytest = ">=7.4.2"
pytest-cov = ">=4.1.0"
ruff = ">=0.0.287"
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
fix = true
"#,
            project_info.source_dir.replace('_', "-"),
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

        save_pyproject_toml_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_eq!(content, expected);
    }

    #[test]
    fn test_save_pyproject_toml_file_mit_pyo3() {
        let base = tempdir().unwrap().path().to_path_buf();
        let project_slug = "test-project";
        create_dir_all(base.join(project_slug)).unwrap();
        let expected_file = base.join(format!("{project_slug}/pyproject.toml"));

        let project_info = ProjectInfo {
            project_name: "My project".to_string(),
            project_slug: project_slug.to_string(),
            source_dir: "my_project".to_string(),
            project_description: "This is a test".to_string(),
            creator: "Arthur Dent".to_string(),
            creator_email: "authur@heartofgold.com".to_string(),
            license: LicenseType::Mit,
            copyright_year: Some("2023".to_string()),
            version: "0.1.0".to_string(),
            python_version: "3.11".to_string(),
            min_python_version: "3.8".to_string(),
            use_pyo3: true,
            is_application: false,
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
            project_root_dir: Some(base),
        };
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
fix = true
"#,
            project_info.source_dir.replace('_', "-"),
            project_info.project_description,
            project_info.creator,
            project_info.creator_email,
            project_info.source_dir,
            project_info.source_dir,
            project_info.max_line_length,
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
        let base = tempdir().unwrap().path().to_path_buf();
        let project_slug = "test-project";
        create_dir_all(base.join(project_slug)).unwrap();
        let expected_file = base.join(format!("{project_slug}/pyproject.toml"));

        let project_info = ProjectInfo {
            project_name: "My project".to_string(),
            project_slug: project_slug.to_string(),
            source_dir: "my_project".to_string(),
            project_description: "This is a test".to_string(),
            creator: "Arthur Dent".to_string(),
            creator_email: "authur@heartofgold.com".to_string(),
            license: LicenseType::Apache2,
            copyright_year: Some("2023".to_string()),
            version: "0.1.0".to_string(),
            python_version: "3.11".to_string(),
            min_python_version: "3.8".to_string(),
            use_pyo3: true,
            is_application: false,
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
            project_root_dir: Some(base),
        };
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
fix = true
"#,
            project_info.source_dir.replace('_', "-"),
            project_info.project_description,
            project_info.creator,
            project_info.creator_email,
            project_info.source_dir,
            project_info.source_dir,
            project_info.max_line_length,
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
        let base = tempdir().unwrap().path().to_path_buf();
        let project_slug = "test-project";
        create_dir_all(base.join(project_slug)).unwrap();
        let expected_file = base.join(format!("{project_slug}/pyproject.toml"));

        let project_info = ProjectInfo {
            project_name: "My project".to_string(),
            project_slug: project_slug.to_string(),
            source_dir: "my_project".to_string(),
            project_description: "This is a test".to_string(),
            creator: "Arthur Dent".to_string(),
            creator_email: "authur@heartofgold.com".to_string(),
            license: LicenseType::NoLicense,
            copyright_year: Some("2023".to_string()),
            version: "0.1.0".to_string(),
            python_version: "3.11".to_string(),
            min_python_version: "3.8".to_string(),
            use_pyo3: true,
            is_application: false,
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
            project_root_dir: Some(base),
        };
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
fix = true
"#,
            project_info.source_dir.replace('_', "-"),
            project_info.project_description,
            project_info.creator,
            project_info.creator_email,
            project_info.source_dir,
            project_info.source_dir,
            project_info.max_line_length,
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
        let expected = format!(
            r#"black==23.7.0
mypy==1.5.1
pre-commit==3.3.3
pytest==7.4.2
pytest-cov==4.1.0
ruff==0.0.287
maturin==1.2.3
"#
        );

        let base = tempdir().unwrap().path().to_path_buf();
        let project_slug = "test-project";
        create_dir_all(base.join(project_slug)).unwrap();
        let expected_file = base.join(format!("{project_slug}/requirements-dev.txt"));
        save_pyo3_dev_requirements(project_slug, true, false, &Some(base)).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_eq!(content, expected);
    }

    #[test]
    fn test_save_pyo3_dev_requirements_lib_file() {
        let expected = format!(
            r#"black>=23.7.0
mypy>=1.5.1
pre-commit>=3.3.3
pytest>=7.4.2
pytest-cov>=4.1.0
ruff>=0.0.287
maturin>=1.2.3
"#
        );

        let base = tempdir().unwrap().path().to_path_buf();
        let project_slug = "test-project";
        create_dir_all(base.join(project_slug)).unwrap();
        let expected_file = base.join(format!("{project_slug}/requirements-dev.txt"));
        save_pyo3_dev_requirements(project_slug, false, false, &Some(base)).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_eq!(content, expected);
    }

    #[test]
    fn test_save_pyo3_justfile() {
        let source_dir = "my_src";
        let expected = format!(
            r#"@develop:
  maturin develop

@install: && develop
  pip install -r requirements-dev.txt

@lint:
  echo cargo check
  just --justfile {{{{justfile()}}}} check
  echo cargo clippy
  just --justfile {{{{justfile()}}}} clippy
  echo cargo fmt
  just --justfile {{{{justfile()}}}} fmt
  echo mypy
  just --justfile {{{{justfile()}}}} mypy
  echo black
  just --justfile {{{{justfile()}}}} black
  echo ruff
  just --justfile {{{{justfile()}}}} ruff

@check:
  cargo check

@clippy:
  cargo clippy

@fmt:
  cargo fmt

@black:
  black {source_dir} tests

@mypy:
  mypy .

@ruff:
  ruff check . --fix

@test:
  pytest
"#
        );

        let base = tempdir().unwrap().path().to_path_buf();
        let project_slug = "test-project";
        create_dir_all(base.join(project_slug)).unwrap();
        let expected_file = base.join(format!("{project_slug}/justfile"));
        save_pyo3_justfile(project_slug, &source_dir, &Some(base)).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_eq!(content, expected);
    }

    #[test]
    fn test_save_readme_file() {
        let project_name = "My Project";
        let project_description = "Some test project";
        let expected = format!(
            r#"# {project_name}

{project_description}
"#
        );

        let base = tempdir().unwrap().path().to_path_buf();
        let project_slug = "test-project";
        create_dir_all(base.join(project_slug)).unwrap();
        let expected_file = base.join(format!("{project_slug}/README.md"));
        save_readme_file(project_slug, project_name, project_description, &Some(base)).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_eq!(content, expected);
    }
}
