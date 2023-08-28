use std::fs::{create_dir_all, File};
use std::io::prelude::*;

use anyhow::Result;
use colored::*;
use minijinja::render;

use crate::project_info::{LicenseType, ProjectInfo};

fn create_apache_license(project_slug: &str) -> Result<()> {
    let license_path = format!("{project_slug}/LICENSE");
    let license_content = r#"                               Apache License
                         Version 2.0, January 2004
                      http://www.apache.org/licenses/

TERMS AND CONDITIONS FOR USE, REPRODUCTION, AND DISTRIBUTION

1. Definitions.

   "License" shall mean the terms and conditions for use, reproduction,
   and distribution as defined by Sections 1 through 9 of this document.

   "Licensor" shall mean the copyright owner or entity authorized by
   the copyright owner that is granting the License.

   "Legal Entity" shall mean the union of the acting entity and all
   other entities that control, are controlled by, or are under common
   control with that entity. For the purposes of this definition,
   "control" means (i) the power, direct or indirect, to cause the
   direction or management of such entity, whether by contract or
   otherwise, or (ii) ownership of fifty percent (50%) or more of the
   outstanding shares, or (iii) beneficial ownership of such entity.

   "You" (or "Your") shall mean an individual or Legal Entity
   exercising permissions granted by this License.

   "Source" form shall mean the preferred form for making modifications,
   including but not limited to software source code, documentation
   source, and configuration files.

   "Object" form shall mean any form resulting from mechanical
   transformation or translation of a Source form, including but
   not limited to compiled object code, generated documentation,
   and conversions to other media types.

   "Work" shall mean the work of authorship, whether in Source or
   Object form, made available under the License, as indicated by a
   copyright notice that is included in or attached to the work
   (an example is provided in the Appendix below).

   "Derivative Works" shall mean any work, whether in Source or Object
   form, that is based on (or derived from) the Work and for which the
   editorial revisions, annotations, elaborations, or other modifications
   represent, as a whole, an original work of authorship. For the purposes
   of this License, Derivative Works shall not include works that remain
   separable from, or merely link (or bind by name) to the interfaces of,
   the Work and Derivative Works thereof.}

   "Contribution" shall mean any work of authorship, including
   the original version of the Work and any modifications or additions
   to that Work or Derivative Works thereof, that is intentionally
   submitted to Licensor for inclusion in the Work by the copyright owner
   or by an individual or Legal Entity authorized to submit on behalf of
   the copyright owner. For the purposes of this definition, "submitted"
   means any form of electronic, verbal, or written communication sent
   to the Licensor or its representatives, including but not limited to
   communication on electronic mailing lists, source code control systems,
   and issue tracking systems that are managed by, or on behalf of, the
   Licensor for the purpose of discussing and improving the Work, but
   excluding communication that is conspicuously marked or otherwise
   designated in writing by the copyright owner as "Not a Contribution."

   "Contributor" shall mean Licensor and any individual or Legal Entity
   on behalf of whom a Contribution has been received by Licensor and
   subsequently incorporated within the Work.

2. Grant of Copyright License. Subject to the terms and conditions of
   this License, each Contributor hereby grants to You a perpetual,
   worldwide, non-exclusive, no-charge, royalty-free, irrevocable
   copyright license to reproduce, prepare Derivative Works of,
   publicly display, publicly perform, sublicense, and distribute the
   Work and such Derivative Works in Source or Object form.

3. Grant of Patent License. Subject to the terms and conditions of
   this License, each Contributor hereby grants to You a perpetual,
   worldwide, non-exclusive, no-charge, royalty-free, irrevocable
   (except as stated in this section) patent license to make, have made,
   use, offer to sell, sell, import, and otherwise transfer the Work,
   where such license applies only to those patent claims licensable
   by such Contributor that are necessarily infringed by their
   Contribution(s) alone or by combination of their Contribution(s)
   with the Work to which such Contribution(s) was submitted. If You
   institute patent litigation against any entity (including a
   cross-claim or counterclaim in a lawsuit) alleging that the Work
   or a Contribution incorporated within the Work constitutes direct
   or contributory patent infringement, then any patent licenses
   granted to You under this License for that Work shall terminate
   as of the date such litigation is filed.

4. Redistribution. You may reproduce and distribute copies of the
   Work or Derivative Works thereof in any medium, with or without
   modifications, and in Source or Object form, provided that You
   meet the following conditions:

   (a) You must give any other recipients of the Work or
       Derivative Works a copy of this License; and

   (b) You must cause any modified files to carry prominent notices
       stating that You changed the files; and

   (c) You must retain, in the Source form of any Derivative Works
       that You distribute, all copyright, patent, trademark, and
       attribution notices from the Source form of the Work,
       excluding those notices that do not pertain to any part of
       the Derivative Works; and

   (d) If the Work includes a "NOTICE" text file as part of its
       distribution, then any Derivative Works that You distribute must
       include a readable copy of the attribution notices contained
       within such NOTICE file, excluding those notices that do not
       pertain to any part of the Derivative Works, in at least one
       of the following places: within a NOTICE text file distributed
       as part of the Derivative Works; within the Source form or
       documentation, if provided along with the Derivative Works; or,
       within a display generated by the Derivative Works, if and
       wherever such third-party notices normally appear. The contents
       of the NOTICE file are for informational purposes only and
       do not modify the License. You may add Your own attribution
       notices within Derivative Works that You distribute, alongside
       or as an addendum to the NOTICE text from the Work, provided
       that such additional attribution notices cannot be construed
       as modifying the License.

   You may add Your own copyright statement to Your modifications and
   may provide additional or different license terms and conditions
   for use, reproduction, or distribution of Your modifications, or
   for any such Derivative Works as a whole, provided Your use,
   reproduction, and distribution of the Work otherwise complies with
   the conditions stated in this License.

5. Submission of Contributions. Unless You explicitly state otherwise,
   any Contribution intentionally submitted for inclusion in the Work
   by You to the Licensor shall be under the terms and conditions of
   this License, without any additional terms or conditions.
   Notwithstanding the above, nothing herein shall supersede or modify
   the terms of any separate license agreement you may have executed
   with Licensor regarding such Contributions.

6. Trademarks. This License does not grant permission to use the trade
   names, trademarks, service marks, or product names of the Licensor,
   except as required for reasonable and customary use in describing the
   origin of the Work and reproducing the content of the NOTICE file.

7. Disclaimer of Warranty. Unless required by applicable law or
   agreed to in writing, Licensor provides the Work (and each
   Contributor provides its Contributions) on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
   implied, including, without limitation, any warranties or conditions
   of TITLE, NON-INFRINGEMENT, MERCHANTABILITY, or FITNESS FOR A
   PARTICULAR PURPOSE. You are solely responsible for determining the
   appropriateness of using or redistributing the Work and assume any
   risks associated with Your exercise of permissions under this License.

8. Limitation of Liability. In no event and under no legal theory,
   whether in tort (including negligence), contract, or otherwise,
   unless required by applicable law (such as deliberate and grossly
   negligent acts) or agreed to in writing, shall any Contributor be
   liable to You for damages, including any direct, indirect, special,
   incidental, or consequential damages of any character arising as a
   result of this License or out of the use or inability to use the
   Work (including but not limited to damages for loss of goodwill,
   work stoppage, computer failure or malfunction, or any and all
   other commercial damages or losses), even if such Contributor
   has been advised of the possibility of such damages.

9. Accepting Warranty or Additional Liability. While redistributing
   the Work or Derivative Works thereof, You may choose to offer,
   and charge a fee for, acceptance of support, warranty, indemnity,
   or other liability obligations and/or rights consistent with this
   License. However, in accepting such obligations, You may act only
   on Your own behalf and on Your sole responsibility, not on behalf
   of any other Contributor, and only if You agree to indemnify,
   defend, and hold each Contributor harmless for any liability
   incurred by, or claims asserted against, such Contributor by reason
   of your accepting any such warranty or additional liability."#;

    create_file_with_content(&license_path, license_content)?;

    Ok(())
}

fn create_ci_testing_linux_only_file(
    project_slug: &str,
    source_dir: &str,
    min_python_version: &str,
    github_action_python_test_versions: &[String],
) -> Result<()> {
    let file_path = format!("{project_slug}/.github/workflows/testing.yml");
    let python_versions = github_action_python_test_versions
        .iter()
        .map(|x| format!(r#""{x}""#))
        .collect::<Vec<String>>()
        .join(", ");
    let content = format!(
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
    );

    create_file_with_content(&file_path, &content)?;

    Ok(())
}

fn create_ci_testing_multi_os_file(
    project_slug: &str,
    source_dir: &str,
    min_python_version: &str,
    github_action_python_test_versions: &[String],
) -> Result<()> {
    let file_path = format!("{project_slug}/.github/workflows/testing.yml");
    let python_versions = github_action_python_test_versions
        .iter()
        .map(|x| format!(r#""{x}""#))
        .collect::<Vec<String>>()
        .join(", ");
    let content = format!(
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
    );

    create_file_with_content(&file_path, &content)?;

    Ok(())
}

fn create_dependabot_file(project_slug: &str) -> Result<()> {
    let file_path = format!("{project_slug}/.github/dependabot.yml");
    let content = r#"version: 2
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
"#;

    create_file_with_content(&file_path, content)?;

    Ok(())
}

fn create_directories(project_slug: &str, source_dir: &str) -> Result<()> {
    let src = format!("{project_slug}/{source_dir}");
    create_dir_all(src)?;

    let github_dir = format!("{project_slug}/.github/workflows");
    create_dir_all(github_dir)?;

    let test_dir = format!("{project_slug}/tests");
    create_dir_all(test_dir)?;

    Ok(())
}

fn create_empty_src_file(project_slug: &str, source_dir: &str, file_name: &str) -> Result<()> {
    let init_file = format!("{project_slug}/{source_dir}/{file_name}");
    File::create(init_file)?;

    Ok(())
}

fn create_file_with_content(file_path: &str, file_content: &str) -> Result<()> {
    let mut file = File::create(file_path)?;
    file.write_all(file_content.as_bytes())?;

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

fn create_main_files(project_slug: &str, source_dir: &str) -> Result<()> {
    let src = format!("{project_slug}/{source_dir}");
    let main = format!("{src}/main.py");
    let main_content = r#"def main() -> int:
    print("Hello world!")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
"#;

    create_file_with_content(&main, main_content)?;

    let main_dunder = format!("{src}/__main__.py");
    let main_dunder_content = format!(
        r#"from {source_dir}.main import main  #  pragma: no cover

if __name__ == "__main__":
    raise SystemExit(main())
"#
    );

    create_file_with_content(&main_dunder, &main_dunder_content)?;

    Ok(())
}

fn create_main_test_file(project_slug: &str, source_dir: &str) -> Result<()> {
    let main_test_path = format!("{project_slug}/tests/test_main.py");
    let main_test_content = format!(
        r#"from {source_dir}.main import main


def test_main():
    assert main() == 0
"#
    );

    create_file_with_content(&main_test_path, &main_test_content)?;

    Ok(())
}

fn create_mit_license(project_slug: &str, copyright_year: &str, creator: &str) -> Result<()> {
    let license_path = format!("{project_slug}/LICENSE");
    let license_content = format!(
        r#"MIT License

Copyright (c) {copyright_year} {creator}

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE."#
    );

    create_file_with_content(&license_path, &license_content)?;

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

fn create_project_init_file(project_slug: &str, source_dir: &str) -> Result<()> {
    let file_path = format!("{project_slug}/{source_dir}/__init__.py");
    let content = format!(
        r#"from {source_dir}._version import VERSION


__version__ = VERSION
"#
    );

    create_file_with_content(&file_path, &content)?;

    Ok(())
}

fn create_pypi_publish_file(project_slug: &str) -> Result<()> {
    let file_path = format!("{project_slug}/.github/workflows/pypi_publish.yml");
    let content = r#"name: PyPi Publish
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
"#;

    create_file_with_content(&file_path, content)?;

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

fn create_release_drafter_file(project_slug: &str) -> Result<()> {
    let template_file_path = format!("{project_slug}/.github/release_drafter_template.yml");
    let template_content = r#"name-template: 'v$RESOLVED_VERSION'
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
"#;

    create_file_with_content(&template_file_path, template_content)?;

    let file_path = format!("{project_slug}/.github/workflows/release_drafter.yml");
    let content = r#"name: Release Drafter

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
"#;

    create_file_with_content(&file_path, content)?;

    Ok(())
}

fn create_version_file(project_slug: &str, source_dir: &str, version: &str) -> Result<()> {
    let version_file_path = format!("{project_slug}/{source_dir}/_version.py");
    let version_content = format!(r#"VERSION = "{version}""#);

    create_file_with_content(&version_file_path, &version_content)?;

    Ok(())
}

fn create_version_test_file(project_slug: &str, source_dir: &str) -> Result<()> {
    let file_path = format!("{project_slug}/tests/test_version.py");
    let content = format!(
        r#"import sys
from pathlib import Path

from {source_dir}._version import VERSION

if sys.version_info < (3, 11):
    import tomli as tomllib
else:
    import tomllib


def test_versions_match():
    pyproject = Path().absolute() / "pyproject.toml"
    with open(pyproject, "rb") as f:
        data = tomllib.load(f)
        pyproject_version = data["tool"]["poetry"]["version"]

    assert VERSION == pyproject_version

"#
    );

    create_file_with_content(&file_path, &content)?;

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

    if create_project_init_file(&project_info.project_slug, &project_info.source_dir).is_err() {
        let error_message = "Error creating __init__.py file";
        println!("\n{}", error_message.red());
        std::process::exit(1);
    }

    if create_empty_src_file(&project_info.project_slug, "tests", "__init__.py").is_err() {
        let error_message = "Error creating test __init__.py file";
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

    match project_info.license {
        LicenseType::Mit => {
            if let Some(year) = &project_info.copywright_year {
                if create_mit_license(&project_info.project_slug, year, &project_info.creator)
                    .is_err()
                {
                    let error_message = "Error creating MIT license file";
                    println!("\n{}", error_message.red());
                    std::process::exit(1);
                };
            } else {
                let error_message = "Error creating MIT license file: copywright year missing";
                println!("\n{}", error_message.red());
                std::process::exit(1);
            }
        }
        LicenseType::Apache2 => {
            if create_apache_license(&project_info.project_slug).is_err() {
                let error_message = "Error creating Apache2 license file";
                println!("\n{}", error_message.red());
                std::process::exit(1);
            };
        }
        _ => (),
    }

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

    if create_version_file(
        &project_info.project_slug,
        &project_info.source_dir,
        &project_info.version,
    )
    .is_err()
    {
        let error_message = "Error creating version file";
        println!("\n{}", error_message.red());
        std::process::exit(1);
    }

    if create_version_test_file(&project_info.project_slug, &project_info.source_dir).is_err() {
        let error_message = "Error creating version test file";
        println!("\n{}", error_message.red());
        std::process::exit(1);
    }

    if project_info.is_application {
        if create_main_files(&project_info.project_slug, &project_info.source_dir).is_err() {
            let error_message = "Error creating main files";
            println!("\n{}", error_message.red());
            std::process::exit(1);
        }

        if create_main_test_file(&project_info.project_slug, &project_info.source_dir).is_err() {
            let error_message = "Error creating main test file";
            println!("\n{}", error_message.red());
            std::process::exit(1);
        }
    }

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
