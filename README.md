# Python Project Generator

[![Tests Status](https://github.com/sanders41/python-project-generator/workflows/Testing/badge.svg?branch=main&event=push)](https://github.com/sanders41/python-project-generator/actions?query=workflow%3ATesting+branch%3Amain+event%3Apush)
![crates.io](https://img.shields.io/crates/v/python-project-generator.svg?color=brightgreen)

Generates a Python project structure with github actions for continuous integration and continuous
deployment. Both pure Python projects and Python projects with Rust modules using PyO3 can be
created.

## Pure Python project included packages

For package managment choose between:

- [poetry](https://python-poetry.org/)
- [setuptools](https://github.com/pypa/setuptools)
- [uv](https://docs.astral.sh/uv/)
- [pixi](https://prefix.dev/)

Dev packages:

- [mypy](https://www.mypy-lang.org/) for static type checking
- [pytest](https://docs.pytest.org/en/latest/) for testing
- [pytest-cov](https://github.com/pytest-dev/pytest-cov) for test coverage reports
- [ruff](https://beta.ruff.rs/docs/) for linting and code formatting

## Python project with Rust modules included packages

- [maturin](https://github.com/PyO3/maturin) for package management
- [mypy](https://www.mypy-lang.org/) for static type checking
- [pytest](https://docs.pytest.org/en/latest/) for testing
- [pytest-cov](https://github.com/pytest-dev/pytest-cov) for test coverage reports
- [ruff](https://beta.ruff.rs/docs/) for linting and code formatting
- [PyO3](https://github.com/PyO3/pyo3) for managing the Rust/Python FFI
- [justfile](https://github.com/casey/just) for running commands (to use this you will need to
  install just)

## Docs

If you chose to include docs then additional dev packages will be included for docs.

- [mkdocs](index_md) for creating the docs
- [mkdocs-material](index_md) for theming the docs
- [mkdocstrings](index_md) for automatically creating API docs

Additionally the `pypi_publish.yml` workflow will also be setup to deploy the doc on release.

## Installation

Install with `cargo`:

```sh
cargo install python-project-generator
```

Install on Arch with the AUR:

```sh
paru -S python-project-generator-bin
```

Install on Debian/Ubuntu:

Note: Change the version to match the version you want to install.

```sh
curl -LO https://github.com/sanders41/python-project-generator/releases/download/v1.0.16/python-project-generator_1.0.16_amd64.deb
sudo dpkg -i python-project-generator_1.0.16_amd64.deb
```

Python Project Generator can also be installed with binaries provided with each release
[here](https://github.com/sanders41/python-project-generator/releases), or with cargo.

## How to use

### Create a new project

From your terminal run:

```sh
python-project create
```

You will be asked a series of questions that will be used to generate your project. The project
generator will check pypi for the lastest version of the included packages and use those while
generating the project. This feature can be disabled by using with either `-s` or
`--skip-download-latest-packages` when running the generator. If either there is an issue with
retrieving the latest versions or if you have decided to skip looking up the latest version, the
packages will be be created with default versions.

```sh
python-project create -s
```

#### Options

- License

  Choose from MIT, Apache 2, or no license.

- Python Version

  This will be the default Python version used. For example when releasing the project this is the
  version of Python that will be used.

- Minimum Python Version

  This is the minimum supported Python version for the project. This is also the version that is
  used for ruff's upgrade target version.

- Python Versions for Github Actions Testing

  Versions listed here will be the versions used for testing in CI.

- Project Manager

  Specifies how project dependencies and builds should be handled

- Application or Library

  Choosing application will create `main.py` and `__main__.py` files. Choosing library will omit
  these files.

- Async Project

  Selecting yes for this option will add [pytest-asyncio](https://github.com/pytest-dev/pytest-asyncio)
  to the dev dependencies. Additionally if the project is an application the `main` function will
  be maid async.

- Max Line Length

  This controls how long the ruff formatter will use for line wrapping.

- Use Dependabot

  Dependabot can be used to keep dependencies up to date. If enabled dependabot will automatically
  create PRs to update dependencies when they are available.

- Dependabot Schedule

  When dependabot is enabed the schedule controls how often dependabot will check for updates and
  create PRs.

- Use Continuous Deployment

  This will create a GitHub Action to deploy the project to pypi when a new release is created.
  Note that for this to work you will need to get an API token for the project from pypi and add it
  to as a new repsitory secret called `PYPI_API_KEY` in the GitHub project.

- Release Drafter

  Choosing yes will create a [release drafter](https://github.com/release-drafter/release-drafter)
  action automatically adds the tile of the PR, who created it, and it's PR number to a draft
  GitHub release. By default the release will get a patch version update. Adding a `bug` label will
  get a patch version update and add it to the `Bug` section of the release notes. Adding an
  `enhancement` label to a PR will create a release with a minor version bump, and a
  `breaking-change` label will create a major version bump. The draft release will get the release
  version tag for the highest label applied to the merged PRs in the release. PRs can be excluded
  from the release notes by applying a `skip-changelog` label to the PR.

- Use Multi OS CI

  Choosing yes will setup CI to run tests on Linux, Mac, and Windows. If no is chosen tests will
  only run on Linux in CI.

- Include Docs

  Choosing yes will add additional packages and base setup for creating documents with mkdocs.

- Docs Site Name

  This quesion will only show if you chose `yes` for `Include Docs`. This value sets the site name
  field for mkdocs.

- Docs Site Description

  This quesion will only show if you chose `yes` for `Include Docs`. This value provides a
  a description of the repo to use in the docs.

- Docs Site URL

  This quesion will only show if you chose `yes` for `Include Docs`. This is the URL where the docs
  will be hosted.

- Docs Locale

  This quesion will only show if you chose `yes` for `Include Docs`, and controls the language of
  the docs.

- Repo Name

  This quesion will only show if you chose `yes` for `Include Docs`. This is the name of the repo
  the docs are referencing. For example in this repository the repo name would be
  `sanders41/python-project-generator`.

- Repo URL

  This quesion will only show if you chose `yes` for `Include Docs`. This is URL for the repo the
  docs are referencing. For example in this repository the repo url would be
  `https://github.com/sanders41/python-project-generator`

- Extra Python Dependencies

  These are extra packages you want to include in the project that are not provided by default.
  For example specifying `fastapi, meilisearch-python-sdk` here will these two packages in the
  dependencies. If the project is an application the version will be pinned to the latest release
  of the packages, and if it is a library the latest release will be used for the minimum version.

After running the generator a new directory will be created with the name you used for the
`Project Slug`. Change to this directory then install the python packages and pre-commit hooks.

### Pure Python Projects

#### Install the Python dependencies when using Poetry.

```sh
poetry install
```

#### Install the Python dependencies when using setuptools.

First create a virtual environment and activate it.

```sh
python -m venv .venv
. .venv/bin/activate
```

```sh
python -m pip install -r requirements-dev.txt
```

#### Install the Python dependencies when using uv.

First create a virtual environment and activate it.

```sh
uv venv
. .venv/bin/activate
```

Next create a lock file

```sh
uv lock
```

Then install the dependencies

```sh
uv sync --frozen
```

Install the pre-commit hooks.

```sh
pre-commit install
```

### PyO3 projects

First create a virtual environment and activate it.

```sh
python -m venv .venv
. .venv/bin/activate
```

Install the dependencies and build the rust module.

```sh
just install
```

Install the pre-commit hooks.

```sh
pre-commit install
```

Now your project is ready to use.

### Save custom default values

You can specify default values for many of the project options. For example to save a default
creator:

```sh
python-project config creator "Wade Watts"
```

To see a full list of values that be set as defaults run:

```sh
python-project config --help
```

To view the current saved defaults:

```sh
python-project config show
```

To remove custom defaults:

```sh
python-project config reset
```

## Information

### just

[just](https://github.com/casey/just) allows you to add project specific commands to the project
that can be run from the command line. It is very similar to [make](https://github.com/mirror/make)
but has the advantage of being cross platform compatable and purposee build for running commands.

As an example, if you have the following in the `justfile` (this is included with the default file
generated by this project):

```just
@lint:
  echo mypy
  just --justfile {{justfile()}} mypy
  echo ruff
  just --justfile {{justfile()}} ruff
  echo ruff-format
  just --justfile {{justfile()}} ruff-format

@mypy:
  uv run mypy meilisearch_python_sdk tests

@ruff:
  uv run ruff check .

@ruff-format:
  uv run ruff format meilisearch_python_sdk tests examples
```

Then you can run `mypy`, `ruff` check, and `ruff` format with:

```sh
just mypy
just ruff
just ruff format
```

You can also run all 3 with 1 `just` command:

```sh
just lint
```

### pre-commit

[pre-commit](https://pre-commit.com/) runs linting and formatting on your code (as defined in the
provided .pre-commit-config.yaml file) every time you make a commit to your code. If any of the
lints fail pre-commit will cancel the commit. When possible, pre-commit will also automatically
fix any errors that fail. For example pre-commit can automatically apply changes that fail ruff
fromatting. pre-commit caches information and only runs on files that have changed so it is fast
and doesn't slow down your work flow will preventing you from forgetting to run checks.

## Contributing

If you are interested in contributing please see our [contributing guide](CONTRIBUTING.md)
