use std::fs::create_dir_all;

use anyhow::{bail, Result};
use colored::*;
use rayon::prelude::*;

use crate::{
    file_manager::{save_empty_src_file, save_file_with_content},
    github_actions::{
        save_ci_testing_linux_only_file, save_ci_testing_multi_os_file, save_dependabot_file,
        save_docs_publish_file, save_pypi_publish_file, save_release_drafter_file,
    },
    licenses::{generate_license, license_str},
    package_version::{
        LatestVersion, PreCommitHook, PreCommitHookVersion, PythonPackage, PythonPackageVersion,
    },
    project_info::{LicenseType, ProjectInfo, ProjectManager, Pyo3PythonManager},
    python_files::generate_python_files,
    rust_files::{save_cargo_toml_file, save_lib_file},
    utils::is_python_version_or_greater,
};

#[cfg(feature = "fastapi")]
use crate::github_actions::save_deploy_files;

fn create_directories(project_info: &ProjectInfo) -> Result<()> {
    let base = project_info.base_dir();
    let src = project_info.source_dir_path();
    create_dir_all(src)?;

    let github_dir = base.join(".github/workflows");
    create_dir_all(github_dir)?;

    let test_dir = base.join("tests");
    create_dir_all(test_dir)?;

    if let ProjectManager::Maturin = &project_info.project_manager {
        let rust_src = base.join("src");
        create_dir_all(rust_src)?;
    }

    if project_info.include_docs {
        let docs_css_dir = base.join("docs/css");
        create_dir_all(docs_css_dir)?;
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

# pixi environments
.pixi

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
        PreCommitHookVersion::new(PreCommitHook::PreCommit),
        PreCommitHookVersion::new(PreCommitHook::MyPy),
        PreCommitHookVersion::new(PreCommitHook::Ruff),
    ];

    if download_latest_packages {
        hooks.par_iter_mut().for_each(|hook| {
            if hook.get_latest_version().is_err() {
                let error_message = format!(
                    "Error retrieving latest pre-commit version for {}. Using default.",
                    hook.hook
                );
                println!("\n{}", error_message.yellow());
            }
        });
    }

    hooks
}

fn create_pre_commit_file(download_latest_packages: bool) -> String {
    let mut pre_commit_str = "repos:".to_string();
    let hooks = build_latest_pre_commit_dependencies(download_latest_packages);
    for hook in hooks {
        match hook.hook {
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
                    "\n  - repo: {}\n    rev: {}\n    hooks:\n    - id: ruff-check\n      args: [--fix, --exit-non-zero-on-fix]\n    - id: ruff-format",
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

fn build_latest_dev_dependencies(project_info: &ProjectInfo) -> Result<String> {
    let mut version_string = String::new();
    let mut packages = if matches!(project_info.project_manager, ProjectManager::Maturin) {
        vec![PythonPackageVersion::new(PythonPackage::Maturin)]
    } else {
        Vec::new()
    };

    if project_info.include_docs {
        packages.push(PythonPackageVersion::new(PythonPackage::Mkdocs));
        packages.push(PythonPackageVersion::new(PythonPackage::MkdocsMaterial));
        packages.push(PythonPackageVersion::new(PythonPackage::Mkdocstrings));
    }

    packages.push(PythonPackageVersion::new(PythonPackage::MyPy));
    packages.push(PythonPackageVersion::new(PythonPackage::PreCommit));
    packages.push(PythonPackageVersion::new(PythonPackage::Pytest));

    if project_info.is_async_project {
        packages.push(PythonPackageVersion::new(PythonPackage::PytestAsyncio));
    }

    packages.push(PythonPackageVersion::new(PythonPackage::PytestCov));
    packages.push(PythonPackageVersion::new(PythonPackage::Ruff));

    if !is_python_version_or_greater(&project_info.min_python_version, 11)?
        && matches!(project_info.project_manager, ProjectManager::Poetry)
    {
        packages.push(PythonPackageVersion::new(PythonPackage::Tomli));
    }

    if project_info.download_latest_packages {
        packages.par_iter_mut().for_each(|package| {
            if package.get_latest_version().is_err() {
                let error_message = format!(
                    "Error retrieving latest python package version for {}. Using default.",
                    package.package
                );
                println!("\n{}", error_message.yellow());
            }
        })
    }

    if let ProjectManager::Uv | ProjectManager::Pixi = project_info.project_manager {
        version_string.push_str("[\n");
    }

    if let ProjectManager::Maturin = project_info.project_manager {
        if let Some(pyo3_python_manager) = &project_info.pyo3_python_manager {
            if pyo3_python_manager == &Pyo3PythonManager::Uv {
                version_string.push_str("[\n");
            }
        }
    }

    for package in packages {
        match project_info.project_manager {
            ProjectManager::Poetry => {
                if package.package == PythonPackage::MyPy {
                    version_string.push_str(&format!(
                        "{} = {{version = \"{}\", extras = [\"faster-cache\"]}}\n",
                        package.package, package.version
                    ));
                } else if package.package == PythonPackage::Tomli {
                    version_string.push_str(&format!(
                        "{} = {{version = \"{}\", python = \"<3.11\"}}\n",
                        package.package, package.version
                    ));
                } else if package.package == PythonPackage::Mkdocstrings {
                    version_string.push_str(&format!(
                        "{} = {{version = \"{}\", extras = [\"python\"]}}\n",
                        package.package, package.version
                    ));
                } else {
                    version_string
                        .push_str(&format!("{} = \"{}\"\n", package.package, package.version));
                }
            }
            ProjectManager::Uv | ProjectManager::Pixi => {
                if package.package == PythonPackage::MyPy {
                    version_string.push_str(&format!(
                        "  \"{}[faster-cache]=={}\",\n",
                        package.package, package.version
                    ));
                } else if package.package == PythonPackage::Mkdocstrings {
                    version_string.push_str(&format!(
                        "  \"{}[python]=={}\",\n",
                        package.package, package.version
                    ));
                } else {
                    version_string.push_str(&format!(
                        "  \"{}=={}\",\n",
                        package.package, package.version
                    ));
                }
            }
            ProjectManager::Maturin => {
                if let Some(pyo3_python_manager) = &project_info.pyo3_python_manager {
                    match pyo3_python_manager {
                        Pyo3PythonManager::Uv => {
                            if package.package == PythonPackage::MyPy {
                                version_string.push_str(&format!(
                                    "  \"{}[faster-cache]=={}\",\n",
                                    package.package, package.version
                                ));
                            } else if package.package == PythonPackage::Mkdocstrings {
                                version_string.push_str(&format!(
                                    "  \"{}[python]=={}\",\n",
                                    package.package, package.version
                                ));
                            } else {
                                version_string.push_str(&format!(
                                    "  \"{}=={}\",\n",
                                    package.package, package.version
                                ));
                            }
                        }
                        Pyo3PythonManager::Setuptools => {
                            if package.package == PythonPackage::MyPy {
                                version_string.push_str(&format!(
                                    "{}[faster-cachw]=={}\n",
                                    package.package, package.version
                                ));
                            } else if package.package == PythonPackage::Mkdocstrings {
                                version_string.push_str(&format!(
                                    "{}[python]=={}\n",
                                    package.package, package.version
                                ));
                            } else {
                                version_string.push_str(&format!(
                                    "{}=={}\n",
                                    package.package, package.version
                                ));
                            }
                        }
                    }
                } else {
                    bail!("A PyO3 Python manager is required with maturin");
                }
            }
            ProjectManager::Setuptools => {
                if package.package == PythonPackage::MyPy {
                    version_string.push_str(&format!(
                        "{}[faster-cache]=={}\n",
                        package.package, package.version
                    ));
                } else if package.package == PythonPackage::Mkdocstrings {
                    version_string.push_str(&format!(
                        "{}[python]=={}\n",
                        package.package, package.version
                    ));
                } else {
                    version_string.push_str(&format!("{}=={}\n", package.package, package.version));
                }
            }
        }
    }

    match project_info.project_manager {
        ProjectManager::Poetry => Ok(version_string.trim().to_string()),
        ProjectManager::Uv => {
            version_string.push(']');
            Ok(version_string)
        }
        ProjectManager::Pixi => {
            version_string.push(']');
            Ok(version_string)
        }
        ProjectManager::Maturin => {
            if let Some(pyo3_python_manager) = &project_info.pyo3_python_manager {
                match pyo3_python_manager {
                    Pyo3PythonManager::Uv => {
                        version_string.push(']');
                        Ok(version_string)
                    }
                    Pyo3PythonManager::Setuptools => {
                        version_string.push_str("-e .\n");
                        Ok(version_string)
                    }
                }
            } else {
                bail!("A PyO3 Python manager is required for maturin");
            }
        }
        ProjectManager::Setuptools => {
            version_string.push_str("-e .\n");
            Ok(version_string)
        }
    }
}

fn create_pyproject_toml(project_info: &ProjectInfo) -> Result<String> {
    let module = project_info.module_name();
    let min_python_version = &project_info.min_python_version;
    let pyupgrade_version = &project_info.min_python_version.replace(['.', '^'], "");
    let project_name = &module.replace('_', "-");
    let project_description = &project_info.project_description;
    let creator = &project_info.creator;
    let creator_email = &project_info.creator_email;
    let version = &project_info.version;
    let license = &project_info.license;
    let license_text = license_str(&project_info.license);
    let dev_dependencies = build_latest_dev_dependencies(project_info)?;
    let max_line_length = &project_info.max_line_length;
    let include_docs = project_info.include_docs;

    let mut pyproject = match &project_info.project_manager {
        ProjectManager::Maturin => {
            if let Some(pyo3_python_manager) = &project_info.pyo3_python_manager {
                match pyo3_python_manager {
                    Pyo3PythonManager::Uv => {
                        let mut pyproject = format!(
                            r#"[build-system]
requires = ["maturin>=1.5,<2.0"]
build-backend = "maturin"

[project]
name = "{project_name}"
description = "{project_description}"
authors = [
  {{ name = "{creator}", email = "{creator_email}" }},
]"#,
                        );

                        if license != &LicenseType::NoLicense {
                            pyproject.push_str(
                                r#"
license = { file = "LICENSE" }"#,
                            );
                        }

                        pyproject.push_str(&format!(
                            r#"
readme = "README.md"
dynamic = ["version"]
requires-python = ">={min_python_version}"
dependencies = []

[dependency-groups]
dev = {dev_dependencies}

[tool.maturin]
module-name = "{module}._{module}"
binding = "pyo3"
features = ["pyo3/extension-module"]

"#,
                        ));
                        pyproject
                    }
                    Pyo3PythonManager::Setuptools => {
                        let mut pyproject = format!(
                            r#"[build-system]
requires = ["maturin>=1.5,<2.0"]
build-backend = "maturin"

[project]
name = "{project_name}"
description = "{project_description}"
authors = [{{name = "{creator}", email = "{creator_email}"}}]"#,
                        );
                        if license != &LicenseType::NoLicense {
                            pyproject.push_str(&format!(
                                r#"
license = "{license_text}""#,
                            ));
                        }

                        pyproject.push_str(&format!(
                            r#"
readme = "README.md"
dynamic = ["version"]
dependencies = []

[tool.maturin]
module-name = "{module}._{module}"
binding = "pyo3"
features = ["pyo3/extension-module"]

"#,
                        ));
                        pyproject
                    }
                }
            } else {
                bail!("A PyO3 Python manager is required for maturin projects");
            }
        }
        ProjectManager::Poetry => {
            let mut pyproject = format!(
                r#"[tool.poetry]
name = "{project_name}"
version = "{version}"
description = "{project_description}"
authors = ["{creator} <{creator_email}>"]
"#
            );

            if license != &LicenseType::NoLicense {
                pyproject.push_str(&format!(
                    "license = \"{license_text}\"
"
                ));
            }

            pyproject.push_str(&format!(
                r#"readme = "README.md"

[tool.poetry.dependencies]
python = "^{min_python_version}"

[tool.poetry.group.dev.dependencies]
{dev_dependencies}

[build-system]
requires = ["poetry-core>=1.0.0"]
build-backend = "poetry.core.masonry.api"

"#
            ));
            pyproject
        }
        ProjectManager::Setuptools => {
            let mut pyproject = format!(
                r#"[build-system]
requires = ["setuptools", "wheel"]
build-backend = "setuptools.build_meta"

[project]
name = "{project_name}"
description = "{project_description}"
authors = [
  {{ name = "{creator}", email = "{creator_email}" }}
]
"#
            );

            if license != &LicenseType::NoLicense {
                pyproject.push_str(&format!(
                    "license = {{ text = \"{license_text}\" }}
"
                ));
            }

            pyproject.push_str(&format!(
                r#"requires-python = ">={min_python_version}"
dynamic = ["version", "readme"]
dependencies = []

[tool.setuptools.dynamic]
version = {{attr = "{module}.__version__"}}
readme = {{file = ["README.md"]}}

[tool.setuptools.packages.find]
include = ["{module}*"]

[tool.setuptools.package-data]
{module} = ["py.typed"]

"#,
            ));
            pyproject
        }
        ProjectManager::Uv => {
            let mut pyproject = format!(
                r#"[build-system]
requires = ["hatchling"]
build-backend = "hatchling.build"

[project]
name = "{project_name}"
description = "{project_description}"
authors = [
  {{ name = "{creator}", email = "{creator_email}" }}
]
"#
            );

            if license != &LicenseType::NoLicense {
                pyproject.push_str(
                    "license = { file = \"LICENSE\" }
",
                );
            }

            pyproject.push_str(&format!(
                r#"readme = "README.md"
requires-python = ">={min_python_version}"
dynamic = ["version"]
dependencies = []

[dependency-groups]
dev = {dev_dependencies}

[tool.hatch.version]
path = "{module}/_version.py"

"#,
            ));
            pyproject
        }
        ProjectManager::Pixi => {
            let mut pyproject = format!(
                r#"[build-system]
requires = ["hatchling"]
build-backend = "hatchling.build"

[project]
name = "{project_name}"
description = "{project_description}"
authors = [
  {{ name = "{creator}", email = "{creator_email}" }}
]
"#
            );

            if license != &LicenseType::NoLicense {
                pyproject.push_str(
                    "license = { file = \"LICENSE\" }
",
                );
            }

            pyproject.push_str(&format!(
                r#"readme = "README.md"
requires-python = ">={min_python_version}"
dynamic = ["version"]
dependencies = []

[tool.pixi.project]
channels = ["conda-forge", "bioconda"]
platforms = ["linux-64", "osx-arm64", "osx-64", "win-64"]

[tool.pixi.feature.dev.tasks]
run-mypy = "mypy {module} tests"
run-ruff-check = "ruff check {module} tests"
run-ruff-format = "ruff format {module} tests"
run-pytest = "pytest -x"
"#,
            ));

            if include_docs {
                pyproject.push_str(
                    "run-deploy-docs = \"mkdocs gh-deploy --force\"
",
                );
            }

            pyproject.push_str(&format!(
                r#"
[project.optional-dependencies]
dev = {dev_dependencies}

[tool.pixi.environments]
default = {{features = [], solve-group = "default"}}
dev = {{features = ["dev"], solve-group = "default"}}

[tool.hatch.version]
path = "{module}/_version.py"

"#,
            ));
            pyproject
        }
    };

    pyproject.push_str(
        r#"[tool.mypy]
check_untyped_defs = true
disallow_untyped_defs = true

[[tool.mypy.overrides]]
module = ["tests.*"]
disallow_untyped_defs = false
"#,
    );

    #[cfg(feature = "fastapi")]
    if project_info.is_fastapi_project {
        pyproject.push_str(
            r#"
[[tool.mypy.overrides]]
module = ["asyncpg.*"]
ignore_missing_imports = true
"#,
        );
    }

    pyproject.push_str(&format!(
        r#"
[tool.pytest.ini_options]
minversion = "6.0"
addopts = "--cov={module} --cov-report term-missing --no-cov-on-fail"
"#,
    ));

    if project_info.is_async_project {
        pyproject.push_str(
            r#"asyncio_mode = "auto"
asyncio_default_fixture_loop_scope = "function"
asyncio_default_test_loop_scope = "function"
"#,
        );
    }

    pyproject.push_str(&format!(
        r#"
[tool.coverage.report]
exclude_lines = ["if __name__ == .__main__.:", "pragma: no cover"]

[tool.ruff]
line-length = {max_line_length}
target-version = "py{pyupgrade_version}"
fix = true

[tool.ruff.lint]
select = [
  "E",  # pycodestyle
  "B",  # flake8-bugbear
  "W",  # Warning
  "F",  # pyflakes
  "UP",  # pyupgrade
  "I001",  # unsorted-imports
  "T201",  # print found
  "T203",  # pprint found
  "RUF022",  # Unsorted __all__
  "RUF023",  # Unforted __slots__
"#,
    ));

    if project_info.is_async_project {
        pyproject.push_str(
            r#"  "ASYNC",  # flake8-async
"#,
        );
    }
    pyproject.push_str(
        r#"]
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
"#,
    );

    if project_info.project_manager == ProjectManager::Uv && project_info.is_application {
        pyproject.push_str(
            r#"
[tool.uv]
add-bounds = "exact"
"#,
        );
    }

    Ok(pyproject)
}

fn save_pyproject_toml_file(project_info: &ProjectInfo) -> Result<()> {
    let file_path = project_info.base_dir().join("pyproject.toml");
    let content = create_pyproject_toml(project_info)?;

    save_file_with_content(&file_path, &content)?;

    Ok(())
}

fn save_dev_requirements(project_info: &ProjectInfo) -> Result<()> {
    let file_path = project_info.base_dir().join("requirements-dev.txt");
    let content = build_latest_dev_dependencies(project_info)?;

    save_file_with_content(&file_path, &content)?;

    Ok(())
}

fn build_mkdocs_yaml(project_info: &ProjectInfo) -> Result<String> {
    if let Some(docs_info) = &project_info.docs_info {
        Ok(format!(
            r#"site_name: {}
site_description: {}
site_url: {}

theme:
  name: material
  locale: {}
  icon:
    repo: fontawesome/brands/github
  palette:
    - scheme: slate
      primary: green
      accent: blue
      toggle:
        icon: material/lightbulb-outline
        name: Switch to dark mode
    - scheme: default
      primary: green
      accent: blue
      toggle:
        icon: material/lightbulb
        name: Switch to light mode
  features:
    - search.suggest
    - search.highlight
repo_name: {}
repo_url: {}

nav:
  - Home: index.md

plugins:
  - mkdocstrings
  - search
"#,
            docs_info.site_name,
            docs_info.site_description,
            docs_info.site_url,
            docs_info.locale,
            docs_info.repo_name,
            docs_info.repo_url,
        ))
    } else {
        bail!("No docs info provided");
    }
}

fn save_mkdocs_yaml(project_info: &ProjectInfo) -> Result<()> {
    let file_path = project_info.base_dir().join("mkdocs.yml");
    let content = build_mkdocs_yaml(project_info)?;

    save_file_with_content(&file_path, &content)?;

    Ok(())
}

fn save_docs_cname(project_info: &ProjectInfo) -> Result<()> {
    if let Some(docs_info) = &project_info.docs_info {
        let file_path = project_info.base_dir().join("docs/CNAME");
        let content = format!("{}\n", &docs_info.site_url);

        save_file_with_content(&file_path, &content)?;

        Ok(())
    } else {
        bail!("No doc info provided");
    }
}

fn save_docs_index_md(project_info: &ProjectInfo) -> Result<()> {
    if let Some(docs_info) = &project_info.docs_info {
        let file_path = project_info.base_dir().join("docs/index.md");
        let content = format!("# {}\n", docs_info.site_description);

        save_file_with_content(&file_path, &content)?;

        Ok(())
    } else {
        bail!("No doc info provided");
    }
}

fn build_docs_css() -> String {
    r#".md-source__repository {
  overflow: visible;
}

div.autodoc-docstring {
  padding-left: 20px;
  margin-bottom: 30px;
  border-left: 5px solid rgba(230, 230, 230);
}

div.autodoc-members {
  padding-left: 20px;
  margin-bottom: 15px;
}
"#
    .to_string()
}

fn save_docs_css(project_info: &ProjectInfo) -> Result<()> {
    let file_path = project_info.base_dir().join("docs/css/custom.css");
    let content = build_docs_css();

    save_file_with_content(&file_path, &content)?;

    Ok(())
}

fn create_poetry_justfile(project_info: &ProjectInfo) -> String {
    let module = project_info.module_name();

    #[cfg_attr(not(feature = "fastapi"), allow(unused_mut))]
    let mut justfile = format!(
        r#"@_default:
  just --list

@lint:
  echo mypy
  just --justfile {{{{justfile()}}}} mypy
  echo ruff-check
  just --justfile {{{{justfile()}}}} ruff-check
  echo ruff-format
  just --justfile {{{{justfile()}}}} ruff-format

@mypy:
  poetry run mypy {module} tests

@ruff-check:
  poetry run ruff check {module} tests

@ruff-format:
  poetry run ruff format {module} tests

@install:
  poetry install

@test *args="":
  poetry run pytest {{{{args}}}}
"#
    );

    #[cfg(feature = "fastapi")]
    if project_info.is_fastapi_project {
        justfile.push_str(
            r#"
granian_cmd := if os() != "windows" {
  "poetry run granian app.main:app --host 127.0.0.1 --port 8000 --interface asgi --no-ws --runtime-mode st --loop uvloop --reload"
} else {
  "poetry run granian app.main:app --host 127.0.0.1 --port 8000 --interface asgi --no-ws --runtime-mode st --reload"
}

@backend-server:
  {{granian_cmd}}

@test-parallel *args="":
  poetry run pytest -n auto {{args}}

@docker-up:
  docker compose up --build

@docker-up-detached:
  docker compose up --build -d

@docker-up-services:
  docker compose up db valkey migrations

@docker-up-services-detached:
  docker compose up db valkey migrations -d

@docker-down:
  docker compose down

@docker-down-volumes:
  docker compose down --volumes

@docker-pull:
  docker compose pull db valkey migrations

@docker-build:
  docker compose build
"#,
        )
    }

    justfile
}

fn create_pyo3_justfile(project_info: &ProjectInfo) -> Result<String> {
    let module = project_info.module_name();
    let pyo3_python_manager = if let Some(manager) = &project_info.pyo3_python_manager {
        manager
    } else {
        bail!("A PyO3 Python manager is required for maturin");
    };
    #[cfg_attr(not(feature = "fastapi"), allow(unused_mut))]
    let mut justfile = match pyo3_python_manager {
        Pyo3PythonManager::Uv => {
            let mut file = format!(
                r#"@_default:
  just --list

@lint:
  echo cargo check
  just --justfile {{{{justfile()}}}} check
  echo cargo clippy
  just --justfile {{{{justfile()}}}} clippy
  echo cargo fmt
  just --justfile {{{{justfile()}}}} fmt
  echo mypy
  just --justfile {{{{justfile()}}}} mypy
  echo ruff check
  just --justfile {{{{justfile()}}}} ruff-check
  echo ruff formatting
  just --justfile {{{{justfile()}}}} ruff-format

@lock:
  uv lock

@lock-upgrade:
  uv lock --upgrade

@develop:
  uv run maturin develop --uv

@develop-release:
  uv run maturin develop -r --uv

@install:
  uv run maturin develop --uv && \
  uv sync --frozen --all-extras

@install-release:
  uv run maturin develop -r --uv && \
  uv sync --frozen --all-extras

@check:
  cargo check

@clippy:
  cargo clippy --all-targets

@fmt:
  cargo fmt --all -- --check

@mypy:
  uv run mypy {module} tests

@ruff-check:
  uv run ruff check {module} tests --fix

@ruff-format:
  uv run ruff format {module} tests

@test *args="":
  uv run pytest {{{{args}}}}
"#
            );

            #[cfg(feature = "fastapi")]
            if project_info.is_fastapi_project {
                file.push_str(
                    r#"
@test-parallel *args="":
  uv run pytest -n auto {{args}}

granian_cmd := if os() != "windows" {
  "uv run granian app.main:app --host 127.0.0.1 --port 8000 --interface asgi --no-ws --runtime-mode st --loop uvloop --reload"
} else {
  "uv run granian app.main:app --host 127.0.0.1 --port 8000 --interface asgi --no-ws --runtime-mode st --reload"
}

@backend-server:
  {{granian_cmd}}
"#,
                );
            }

            file
        }
        Pyo3PythonManager::Setuptools => {
            let mut file = format!(
                r#"@_default:
  just --list

@lint:
  echo cargo check
  just --justfile {{{{justfile()}}}} check
  echo cargo clippy
  just --justfile {{{{justfile()}}}} clippy
  echo cargo fmt
  just --justfile {{{{justfile()}}}} fmt
  echo mypy
  just --justfile {{{{justfile()}}}} mypy
  echo ruff check
  just --justfile {{{{justfile()}}}} ruff-check
  echo ruff formatting
  just --justfile {{{{justfile()}}}} ruff-format

@develop:
  python -m maturin develop

@develop-release:
  python -m maturin develop -r

@install:
  python -m maturin develop && \
  python -m pip install -r requirements-dev.txt

@install-release:
  python -m maturin develop -r && \
  python -m pip install -r requirements-dev.txt

@check:
  cargo check

@clippy:
  cargo clippy --all-targets

@fmt:
  cargo fmt --all -- --check

@mypy:
  python -m mypy {module} tests

@ruff-check:
  python -m ruff check {module} tests --fix

@ruff-format:
  python -m ruff format {module} tests

@test *arg="":
  python -m pytest {{{{args}}}}
"#
            );

            #[cfg(feature = "fastapi")]
            if project_info.is_fastapi_project {
                file.push_str(
                    r#"
@test-parallel *args="":
  python -m pytest -n auto {{{{args}}}}

granian_cmd := if os() != "windows" {
  "python -m granian app.main:app --host 127.0.0.1 --port 8000 --interface asgi --no-ws --runtime-mode st --loop uvloop --reload"
} else {
  "python -m granian app.main:app --host 127.0.0.1 --port 8000 --interface asgi --no-ws --runtime-mode st --reload"
}

@backend-server:
  {{granian_cmd}}
"#,
                );
            }

            file
        }
    };

    #[cfg(feature = "fastapi")]
    if project_info.is_fastapi_project {
        justfile.push_str(
            r#"
@docker-up:
  docker compose up --build

@docker-up-detached:
  docker compose up --build -d

@docker-up-services:
  docker compose up db valkey migrations

@docker-up-services-detached:
  docker compose up db valkey migrations -d

@docker-down:
  docker compose down

@docker-down-volumes:
  docker compose down --volumes

@docker-pull:
  docker compose pull db valkey migrations

@docker-build:
  docker compose build
}}}}
"#,
        )
    }

    Ok(justfile)
}

fn create_setuptools_justfile(project_info: &ProjectInfo) -> String {
    let module = project_info.module_name();

    #[cfg_attr(not(feature = "fastapi"), allow(unused_mut))]
    let mut justfile = format!(
        r#"@_default:
  just --list

@lint:
  echo mypy
  just --justfile {{{{justfile()}}}} mypy
  echo ruff-check
  just --justfile {{{{justfile()}}}} ruff-check
  echo ruff-format
  just --justfile {{{{justfile()}}}} ruff-format

@mypy:
  python -m mypy {module} tests

@ruff-check:
  python -m ruff check {module} tests

@ruff-format:
  python -m ruff format {module} tests

@install:
  python -m pip install -r requirements-dev.txt

@test *args="":
  python -m pytest {{{{args}}}}
"#
    );

    #[cfg(feature = "fastapi")]
    if project_info.is_fastapi_project {
        justfile.push_str(
            r#"
@test-parallel *args="":
  python -m pytest -n auto {{args}}

granian_cmd := if os() != "windows" {
  "python -m granian app.main:app --host 127.0.0.1 --port 8000 --interface asgi --no-ws --runtime-mode st --loop uvloop --reload"
} else {
  "python -m granian app.main:app --host 127.0.0.1 --port 8000 --interface asgi --no-ws --runtime-mode st --reload"
}

@backend-server:
  {{granian_cmd}}

@docker-up:
  docker compose up --build

@docker-up-detached:
  docker compose up --build -d

@docker-up-services:
  docker compose up db valkey migrations

@docker-up-services-detached:
  docker compose up db valkey migrations -d

@docker-down:
  docker compose down

@docker-down-volumes:
  docker compose down --volumes

@docker-pull:
  docker compose pull db valkey migrations

@docker-build:
  docker compose build
"#,
        )
    }

    justfile
}

fn create_uv_justfile(project_info: &ProjectInfo) -> String {
    let module = project_info.module_name();

    #[cfg_attr(not(feature = "fastapi"), allow(unused_mut))]
    let mut justfile = format!(
        r#"@_default:
  just --list

@lint:
  echo mypy
  just --justfile {{{{justfile()}}}} mypy
  echo ruff-check
  just --justfile {{{{justfile()}}}} ruff-check
  echo ruff-format
  just --justfile {{{{justfile()}}}} ruff-format

@mypy:
  uv run mypy {module} tests

@ruff-check:
  uv run ruff check {module} tests

@ruff-format:
  uv run ruff format {module} tests

@lock:
  uv lock

@lock-upgrade:
  uv lock --upgrade

@install:
  uv sync --frozen --all-extras

@test *args="":
  uv run pytest {{{{args}}}}
"#
    );

    #[cfg(feature = "fastapi")]
    if project_info.is_fastapi_project {
        justfile.push_str(
            r#"
@test-parallel *args="":
  uv run pytest -n auto {{args}}

granian_cmd := if os() != "windows" {
  "uv run granian app.main:app --host 127.0.0.1 --port 8000 --interface asgi --no-ws --runtime-mode st --loop uvloop --reload"
} else {
  "uv run granian app.main:app --host 127.0.0.1 --port 8000 --interface asgi --no-ws --runtime-mode st --reload"
}

@backend-server:
  {{granian_cmd}}

@docker-up:
  docker compose up --build

@docker-up-detached:
  docker compose up --build -d

@docker-up-services:
  docker compose up db valkey migrations

@docker-up-services-detached:
  docker compose up db valkey migrations -d

@docker-down:
  docker compose down

@docker-down-volumes:
  docker compose down --volumes

@docker-pull:
  docker compose pull db valkey migrations

@docker-build:
  docker compose build
"#,
        )
    }

    justfile
}

fn create_pixi_justfile() -> String {
    (r#"@_default:
  just --list

@lint:
  echo mypy
  just --justfile {{{{justfile()}}}} mypy
  echo ruff-check
  just --justfile {{{{justfile()}}}} ruff-check
  echo ruff-format
  just --justfile {{{{justfile()}}}} ruff-format

@mypy:
  pixi run run-mypy

@ruff-check:
  pixi run run-ruff-check

@ruff-format:
  pixi run run-ruff-format

@test:
  pixi run run-pytest

@install:
  pixi install
"#)
    .to_string()
}

fn save_justfile(project_info: &ProjectInfo) -> Result<()> {
    let file_path = project_info.base_dir().join("justfile");
    let content = match &project_info.project_manager {
        ProjectManager::Poetry => create_poetry_justfile(project_info),
        ProjectManager::Maturin => create_pyo3_justfile(project_info)?,
        ProjectManager::Setuptools => create_setuptools_justfile(project_info),
        ProjectManager::Uv => create_uv_justfile(project_info),
        ProjectManager::Pixi => create_pixi_justfile(),
    };

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

    if save_justfile(project_info).is_err() {
        bail!("Error creating justfile");
    }

    match &project_info.project_manager {
        ProjectManager::Maturin => {
            if let Some(pyo3_python_manager) = &project_info.pyo3_python_manager {
                if pyo3_python_manager == &Pyo3PythonManager::Setuptools
                    && save_dev_requirements(project_info).is_err()
                {
                    bail!("Error creating requirements-dev.txt file");
                }

                if save_lib_file(project_info).is_err() {
                    bail!("Error creating Rust lib.rs file");
                }

                if save_cargo_toml_file(project_info).is_err() {
                    bail!("Error creating Rust lib.rs file");
                }
            } else {
                bail!("A PyO3 Python Manager is required with Maturin");
            }
        }
        ProjectManager::Setuptools => {
            if save_dev_requirements(project_info).is_err() {
                bail!("Error creating requirements-dev.txt file");
            }
        }
        _ => (),
    }

    #[cfg(feature = "fastapi")]
    if project_info.use_continuous_deployment {
        if project_info.is_fastapi_project {
            if save_deploy_files(project_info).is_err() {
                bail!("Error creating deploy files");
            }
        } else if save_pypi_publish_file(project_info).is_err() {
            bail!("Error creating PyPI publish file");
        }
    }

    #[cfg(not(feature = "fastapi"))]
    if project_info.use_continuous_deployment && save_pypi_publish_file(project_info).is_err() {
        bail!("Error creating PyPI publish file");
    }

    if project_info.include_docs && save_docs_publish_file(project_info).is_err() {
        bail!("Error creating docs publish file");
    }

    if project_info.use_multi_os_ci {
        if save_ci_testing_multi_os_file(project_info).is_err() {
            bail!("Error creating CI teesting file");
        }
    } else if save_ci_testing_linux_only_file(project_info).is_err() {
        bail!("Error creating CI teesting file");
    }

    if project_info.include_docs {
        if save_mkdocs_yaml(project_info).is_err() {
            bail!("Error creating mkdocs.yml file");
        }

        if save_docs_cname(project_info).is_err() {
            bail!("Error creating CNAME file for docs");
        }

        if save_docs_index_md(project_info).is_err() {
            bail!("Error index.md file for docs");
        }

        if save_docs_css(project_info).is_err() {
            bail!("Error saving docs css file");
        }
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
    use crate::project_info::{DocsInfo, LicenseType, ProjectInfo, Pyo3PythonManager};
    use insta::assert_yaml_snapshot;
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
            python_version: "3.11".to_string(),
            min_python_version: "3.9".to_string(),
            project_manager: ProjectManager::Poetry,
            pyo3_python_manager: Some(Pyo3PythonManager::Uv),
            is_application: true,
            is_async_project: false,
            github_actions_python_test_versions: vec![
                "3.9".to_string(),
                "3.10".to_string(),
                "3.11".to_string(),
                "3.12".to_string(),
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
            download_latest_packages: false,
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
    fn test_save_gitigngore_file() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Poetry;
        let base = project_info.base_dir();
        create_dir_all(&base).unwrap();
        let expected_file = base.join(".gitignore");
        save_gitigngore_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_yaml_snapshot!(content);
    }

    #[test]
    fn test_save_gitigngore_pyo3_file() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Maturin;
        let base = project_info.base_dir();
        create_dir_all(&base).unwrap();
        let expected_file = base.join(".gitignore");
        save_gitigngore_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_yaml_snapshot!(content);
    }

    #[test]
    fn test_save_pre_commit_file() {
        let project_info = project_info_dummy();
        let base = project_info.base_dir();
        create_dir_all(&base).unwrap();
        let expected_file = base.join(".pre-commit-config.yaml");
        save_pre_commit_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        insta::with_settings!({filters => vec![
            (r": v\d+\.\d+\.\d+", ": v1.0.0"),
        ]}, { assert_yaml_snapshot!(content)});
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
        save_pyproject_toml_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        insta::with_settings!({filters => vec![
            (r#""\d+\.\d+\.\d+"#, "\"1.0.0"),
        ]}, { assert_yaml_snapshot!(content)});
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
        save_pyproject_toml_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        insta::with_settings!({filters => vec![
            (r#""\d+\.\d+\.\d+"#, "\"1.0.0"),
            (r#"">=\d+\.\d+\.\d+"#, "\">=1.0.0"),
        ]}, { assert_yaml_snapshot!(content)});
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
        save_pyproject_toml_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        insta::with_settings!({filters => vec![
            (r#""\d+\.\d+\.\d+"#, "\"1.0.0"),
            (r#"">=\d+\.\d+\.\d+"#, "\">=1.0.0"),
        ]}, { assert_yaml_snapshot!(content)});
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
        save_pyproject_toml_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        insta::with_settings!({filters => vec![
            (r#""\d+\.\d+\.\d+"#, "\"1.0.0"),
            (r#"">=\d+\.\d+\.\d+"#, "\">=1.0.0"),
        ]}, { assert_yaml_snapshot!(content)});
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
        save_pyproject_toml_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        insta::with_settings!({filters => vec![
            (r"==\d+\.\d+\.\d+", "==1.0.0"),
            (r">=\d+\.\d+\.\d+", ">=1.0.0"),
        ]}, { assert_yaml_snapshot!(content)});
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
        save_pyproject_toml_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        insta::with_settings!({filters => vec![
            (r"==\d+\.\d+\.\d+", "==1.0.0"),
            (r">=\d+\.\d+\.\d+", ">=1.0.0"),
        ]}, { assert_yaml_snapshot!(content)});
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
        save_pyproject_toml_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        insta::with_settings!({filters => vec![
            (r"==\d+\.\d+\.\d+", "==1.0.0"),
            (r">=\d+\.\d+\.\d+", ">=1.0.0"),
        ]}, { assert_yaml_snapshot!(content)});
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
        save_pyproject_toml_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        insta::with_settings!({filters => vec![
            (r"==\d+\.\d+\.\d+", "==1.0.0"),
            (r">=\d+\.\d+\.\d+", ">=1.0.0"),
        ]}, { assert_yaml_snapshot!(content)});
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
        save_pyproject_toml_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        insta::with_settings!({filters => vec![
            (r"==\d+\.\d+\.\d+", "==1.0.0"),
            (r">=\d+\.\d+\.\d+", ">=1.0.0"),
        ]}, { assert_yaml_snapshot!(content)});
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
        save_pyproject_toml_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        insta::with_settings!({filters => vec![
            (r"==\d+\.\d+\.\d+", "==1.0.0"),
            (r">=\d+\.\d+\.\d+", ">=1.0.0"),
        ]}, { assert_yaml_snapshot!(content)});
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
        save_pyproject_toml_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        insta::with_settings!({filters => vec![
            (r"==\d+\.\d+\.\d+", "==1.0.0"),
            (r">=\d+\.\d+\.\d+", ">=1.0.0"),
        ]}, { assert_yaml_snapshot!(content)});
    }

    #[test]
    fn test_save_uv_pyproject_toml_file_mit_application() {
        let mut project_info = project_info_dummy();
        project_info.license = LicenseType::Mit;
        project_info.project_manager = ProjectManager::Uv;
        project_info.is_application = true;
        let base = project_info.base_dir();
        create_dir_all(&base).unwrap();
        let expected_file = base.join("pyproject.toml");
        save_pyproject_toml_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        insta::with_settings!({filters => vec![
            (r"==\d+\.\d+\.\d+", "==1.0.0"),
            (r">=\d+\.\d+\.\d+", ">=1.0.0"),
        ]}, { assert_yaml_snapshot!(content)});
    }

    #[test]
    fn test_save_uv_pyproject_toml_file_apache_application() {
        let mut project_info = project_info_dummy();
        project_info.license = LicenseType::Apache2;
        project_info.project_manager = ProjectManager::Uv;
        project_info.is_application = true;
        let base = project_info.base_dir();
        create_dir_all(&base).unwrap();
        let expected_file = base.join("pyproject.toml");
        save_pyproject_toml_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        insta::with_settings!({filters => vec![
            (r"==\d+\.\d+\.\d+", "==1.0.0"),
            (r">=\d+\.\d+\.\d+", ">=1.0.0"),
        ]}, { assert_yaml_snapshot!(content)});
    }

    #[test]
    fn test_save_uv_pyproject_toml_file_no_license_application() {
        let mut project_info = project_info_dummy();
        project_info.license = LicenseType::NoLicense;
        project_info.project_manager = ProjectManager::Uv;
        project_info.is_application = true;
        let base = project_info.base_dir();
        create_dir_all(&base).unwrap();
        let expected_file = base.join("pyproject.toml");
        save_pyproject_toml_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        insta::with_settings!({filters => vec![
            (r"==\d+\.\d+\.\d+", "==1.0.0"),
            (r">=\d+\.\d+\.\d+", ">=1.0.0"),
        ]}, { assert_yaml_snapshot!(content)});
    }

    #[test]
    fn test_create_uv_pyproject_toml_mit_lib() {
        let mut project_info = project_info_dummy();
        project_info.license = LicenseType::Mit;
        project_info.project_manager = ProjectManager::Uv;
        project_info.is_application = false;
        let base = project_info.base_dir();
        create_dir_all(&base).unwrap();
        let expected_file = base.join("pyproject.toml");
        save_pyproject_toml_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        insta::with_settings!({filters => vec![
            (r"==\d+\.\d+\.\d+", "==1.0.0"),
            (r">=\d+\.\d+\.\d+", ">=1.0.0"),
        ]}, { assert_yaml_snapshot!(content)});
    }

    #[test]
    fn test_save_pixi_pyproject_toml_file_mit_application() {
        let mut project_info = project_info_dummy();
        project_info.license = LicenseType::Mit;
        project_info.project_manager = ProjectManager::Pixi;
        project_info.is_application = true;
        let base = project_info.base_dir();
        create_dir_all(&base).unwrap();
        let expected_file = base.join("pyproject.toml");
        save_pyproject_toml_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        insta::with_settings!({filters => vec![
            (r"==\d+\.\d+\.\d+", "==1.0.0"),
            (r">=\d+\.\d+\.\d+", ">=1.0.0"),
        ]}, { assert_yaml_snapshot!(content)});
    }

    #[test]
    fn test_save_pixi_pyproject_toml_file_apache_application() {
        let mut project_info = project_info_dummy();
        project_info.license = LicenseType::Apache2;
        project_info.project_manager = ProjectManager::Pixi;
        project_info.is_application = true;
        let base = project_info.base_dir();
        create_dir_all(&base).unwrap();
        let expected_file = base.join("pyproject.toml");
        save_pyproject_toml_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        insta::with_settings!({filters => vec![
            (r"==\d+\.\d+\.\d+", "==1.0.0"),
            (r">=\d+\.\d+\.\d+", ">=1.0.0"),
        ]}, { assert_yaml_snapshot!(content)});
    }

    #[test]
    fn test_save_pixi_pyproject_toml_file_no_license_application() {
        let mut project_info = project_info_dummy();
        project_info.license = LicenseType::NoLicense;
        project_info.project_manager = ProjectManager::Pixi;
        project_info.is_application = true;
        let base = project_info.base_dir();
        create_dir_all(&base).unwrap();
        let expected_file = base.join("pyproject.toml");
        save_pyproject_toml_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        insta::with_settings!({filters => vec![
            (r"==\d+\.\d+\.\d+", "==1.0.0"),
            (r">=\d+\.\d+\.\d+", ">=1.0.0"),
        ]}, { assert_yaml_snapshot!(content)});
    }

    #[test]
    fn test_create_pixi_pyproject_toml_mit_lib() {
        let mut project_info = project_info_dummy();
        project_info.license = LicenseType::Mit;
        project_info.project_manager = ProjectManager::Pixi;
        project_info.is_application = false;
        let base = project_info.base_dir();
        create_dir_all(&base).unwrap();
        let expected_file = base.join("pyproject.toml");
        save_pyproject_toml_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        insta::with_settings!({filters => vec![
            (r"==\d+\.\d+\.\d+", "==1.0.0"),
            (r">=\d+\.\d+\.\d+", ">=1.0.0"),
        ]}, { assert_yaml_snapshot!(content)});
    }

    #[test]
    fn test_create_pyproject_toml_async_project() {
        let mut project_info = project_info_dummy();
        project_info.is_async_project = true;
        let base = project_info.base_dir();
        create_dir_all(&base).unwrap();
        let expected_file = base.join("pyproject.toml");
        save_pyproject_toml_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        insta::with_settings!({filters => vec![
            (r"\d+\.\d+\.\d+", "1.0.0"),
        ]}, { assert_yaml_snapshot!(content)});
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

        insta::with_settings!({filters => vec![
            (r"==\d+\.\d+\.\d+", "==1.0.0"),
            (r">=\d+\.\d+\.\d+", ">=1.0.0"),
        ]}, { assert_yaml_snapshot!(content)});
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

        insta::with_settings!({filters => vec![
            (r"==\d+\.\d+\.\d+", "==1.0.0"),
            (r">=\d+\.\d+\.\d+", ">=1.0.0"),
        ]}, { assert_yaml_snapshot!(content)});
    }

    #[test]
    fn test_save_setuptools_dev_requirements_application_file() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Maturin;
        project_info.pyo3_python_manager = Some(Pyo3PythonManager::Setuptools);
        project_info.is_application = true;
        let base = project_info.base_dir();
        create_dir_all(&base).unwrap();
        let expected_file = base.join("requirements-dev.txt");
        save_dev_requirements(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        insta::with_settings!({filters => vec![
            (r"==\d+\.\d+\.\d+", "==1.0.0"),
            (r">=\d+\.\d+\.\d+", ">=1.0.0"),
        ]}, { assert_yaml_snapshot!(content)});
    }

    #[test]
    fn test_save_setuptools_dev_requirements_lib_file() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Maturin;
        project_info.pyo3_python_manager = Some(Pyo3PythonManager::Setuptools);
        project_info.is_application = false;
        let base = project_info.base_dir();
        create_dir_all(&base).unwrap();
        let expected_file = base.join("requirements-dev.txt");
        save_dev_requirements(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        insta::with_settings!({filters => vec![
            (r"==\d+\.\d+\.\d+", "==1.0.0"),
            (r">=\d+\.\d+\.\d+", ">=1.0.0"),
        ]}, { assert_yaml_snapshot!(content)});
    }

    #[test]
    fn test_save_mkdocs_yaml() {
        let mut project_info = project_info_dummy();
        project_info.include_docs = true;
        project_info.docs_info = Some(docs_info_dummy());
        let base = project_info.base_dir();
        create_dir_all(&base).unwrap();
        let expected_file = base.join("mkdocs.yml");
        save_mkdocs_yaml(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();
        assert_yaml_snapshot!(content);
    }

    #[test]
    fn test_save_cname_file() {
        let mut project_info = project_info_dummy();
        project_info.include_docs = true;
        project_info.docs_info = Some(docs_info_dummy());
        let base = project_info.base_dir().join("docs");
        create_dir_all(&base).unwrap();
        let expected_file = base.join("CNAME");
        save_docs_cname(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();
        assert_yaml_snapshot!(content);
    }

    #[test]
    fn test_save_index_md_file() {
        let mut project_info = project_info_dummy();
        project_info.include_docs = true;
        project_info.docs_info = Some(docs_info_dummy());
        let base = project_info.base_dir().join("docs");
        create_dir_all(&base).unwrap();
        let expected_file = base.join("index.md");
        save_docs_index_md(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();
        assert_yaml_snapshot!(content);
    }

    #[test]
    fn test_save_docs_css_file() {
        let mut project_info = project_info_dummy();
        project_info.include_docs = true;
        project_info.docs_info = Some(docs_info_dummy());
        let base = project_info.base_dir().join("docs/css");
        create_dir_all(&base).unwrap();
        let expected_file = base.join("custom.css");
        save_docs_css(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();
        assert_yaml_snapshot!(content);
    }

    #[test]
    fn test_save_justfile_poetry() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Poetry;
        let base = project_info.base_dir();
        create_dir_all(&base).unwrap();
        let expected_file = base.join("justfile");
        save_justfile(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_yaml_snapshot!(content);
    }

    #[cfg(feature = "fastapi")]
    #[test]
    fn test_save_justfile_poetry_fastapi_project() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Poetry;
        project_info.is_fastapi_project = true;
        project_info.database_manager = Some(DatabaseManager::AsyncPg);
        let base = project_info.base_dir();
        create_dir_all(&base).unwrap();
        let expected_file = base.join("justfile");
        save_justfile(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_yaml_snapshot!(content);
    }

    #[test]
    fn test_save_justfile_uv() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Uv;
        let base = project_info.base_dir();
        create_dir_all(&base).unwrap();
        let expected_file = base.join("justfile");
        save_justfile(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_yaml_snapshot!(content);
    }

    #[cfg(feature = "fastapi")]
    #[test]
    fn test_save_justfile_uv_fastapi_project() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Uv;
        project_info.is_fastapi_project = true;
        project_info.database_manager = Some(DatabaseManager::AsyncPg);
        let base = project_info.base_dir();
        create_dir_all(&base).unwrap();
        let expected_file = base.join("justfile");
        save_justfile(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_yaml_snapshot!(content);
    }

    #[test]
    fn test_save_justfile_setuptools() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Setuptools;
        let base = project_info.base_dir();
        create_dir_all(&base).unwrap();
        let expected_file = base.join("justfile");
        save_justfile(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_yaml_snapshot!(content);
    }

    #[cfg(feature = "fastapi")]
    #[test]
    fn test_save_justfile_setuptools_fastapi_project() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Setuptools;
        project_info.is_fastapi_project = true;
        project_info.database_manager = Some(DatabaseManager::AsyncPg);
        let base = project_info.base_dir();
        create_dir_all(&base).unwrap();
        let expected_file = base.join("justfile");
        save_justfile(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_yaml_snapshot!(content);
    }

    #[test]
    fn test_save_justfile_maturin() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Maturin;
        project_info.is_application = false;
        let base = project_info.base_dir();
        create_dir_all(&base).unwrap();
        let expected_file = base.join("justfile");
        save_justfile(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_yaml_snapshot!(content);
    }

    #[cfg(feature = "fastapi")]
    #[test]
    fn test_save_justfile_maturin_fastapi_project() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Maturin;
        project_info.is_fastapi_project = true;
        project_info.database_manager = Some(DatabaseManager::AsyncPg);
        let base = project_info.base_dir();
        create_dir_all(&base).unwrap();
        let expected_file = base.join("justfile");
        save_justfile(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_yaml_snapshot!(content);
    }

    #[test]
    fn test_save_readme_file() {
        let project_info = project_info_dummy();
        let base = project_info.base_dir();
        create_dir_all(&base).unwrap();
        let expected_file = base.join("README.md");
        save_readme_file(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_yaml_snapshot!(content);
    }
}
