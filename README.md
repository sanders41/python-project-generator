# Python Project Generator

[![Tests Status](https://github.com/sanders41/python-project-generator/actions/workflows/testing.yml/badge.svg?branch=main&event=push)](https://github.com/sanders41/python-project-generator/actions?query=workflow%3ATesting+branch%3Amain+event%3Apush)
![crates.io](https://img.shields.io/crates/v/python-project-generator.svg?color=brightgreen)

Generates a Python project structure with github actions for continuous integration and continuous
deployment. Both pure Python projects and Python projects with Rust modules using PyO3 can be
created. Additionally FastAPI projects can be generated.

## Pure Python project included packages

For package management choose between:

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

## FastAPI projects include

- [asyncpg](https://github.com/MagicStack/asyncpg) for interacting with PostgreSQL
- [camel-converter](https://github.com/sanders41/camel-converter) for converting to and from
  camel/snake case in Pydantic models when serializing/deserializing JSON
- [fastapi](https://github.com/fastapi/fastapi)
- [granian](https://github.com/emmett-framework/granian) for handling the web requests
- [httptools](https://github.com/MagicStack/httptools) for faster http parsing
- [loguru](https://github.com/Delgan/loguru) for logging
- [orjson](https://github.com/ijl/orjson) for faster JSON serization/deserilization
- [pwdlib](https://github.com/frankie567/pwdlib) for password hashing
- [pydantic](https://github.com/pydantic/pydantic) for model validation
- [pydantic-settings](https://github.com/pydantic/pydantic-settings) for managing settings
- [uvloop](https://github.com/MagicStack/uvloop) for enhanced performance (not available on Windows)
- [postgresql](https://www.postgresql.org/) for the database layer
- [valkey](https://github.com/valkey-io/valkey) for the caching layer
- [traefik](https://github.com/traefik/traefik) for reverse proxy
- [sqlx](https://github.com/launchbadge/sqlx) for migrations

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

If you want to be able to generate FastAPI projects install with the fastapi feature

```sh
cargo install python-project-generator -F fastapi
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
generator will check pypi for the latest version of the included packages and use those while
generating the project.

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
  these files. FastAPI projects are automatically created as applications with a special FastAPI
  main.py

- Async Project

  Selecting yes for this option will add [pytest-asyncio](https://github.com/pytest-dev/pytest-asyncio)
  to the dev dependencies. Additionally if the project is an application the `main` function will
  be made async. This question is skipped with FastAPI projects and automatically set to an async
  project

- Max Line Length

  This controls how long the ruff formatter will use for line wrapping.

- Use Dependabot

  Dependabot can be used to keep dependencies up to date. If enabled dependabot will automatically
  create PRs to update dependencies when they are available.

- Dependabot Schedule

  When dependabot is enabled the schedule controls how often dependabot will check for updates and
  create PRs.

- Use Continuous Deployment

  This will create a GitHub Action to deploy the project to PyPI when a new release is created.
  Note that for this to work you will need to setup a
  [trusted publisher](https://docs.pypi.org/trusted-publishers/adding-a-publisher/) in PyPI with
  a workflow name of pypi_publish.yml.

  If the project is a FastAPI project this will create workflows to deploy to test and production
  servers using GitHub runners.

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

  This is skipped for FastAPI projects and defaults to Linux only. FastAPI projects use Docker
  with is only available in Linux in GitHub Actions.

- Include Docs

  Choosing yes will add additional packages and base setup for creating documents with mkdocs.

- Docs Site Name

  This question will only show if you chose `yes` for `Include Docs`. This value sets the site name
  field for mkdocs.

- Docs Site Description

  This question will only show if you chose `yes` for `Include Docs`. This value provides a
  a description of the repo to use in the docs.

- Docs Site URL

  This question will only show if you chose `yes` for `Include Docs`. This is the URL where the docs
  will be hosted.

- Docs Locale

  This question will only show if you chose `yes` for `Include Docs`, and controls the language of
  the docs.

- Repo Name

  This question will only show if you chose `yes` for `Include Docs`. This is the name of the repo
  the docs are referencing. For example in this repository the repo name would be
  `sanders41/python-project-generator`.

- Repo URL

  This question will only show if you chose `yes` for `Include Docs`. This is URL for the repo the
  docs are referencing. For example in this repository the repo url would be
  `https://github.com/sanders41/python-project-generator`

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

### FastAPI projects

Create a .env file with the needed variables. the .env-example file can be used as a starter
template. Then start the containers.

```sh
docker compose up
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
but has the advantage of being cross platform compatible and purposee build for running commands.

As an example, if you have the following in the `justfile` (this is included with the default file
generated by this project):

```just
@_default:
  just --list

@lint:
  echo mypy
  just --justfile {{justfile()}} mypy
  echo ruff-check
  just --justfile {{justfile()}} ruff-check
  echo ruff-format
  just --justfile {{justfile()}} ruff-format

@mypy:
  uv run mypy my_project tests

@ruff-check:
  uv run ruff check my_project tests

@ruff-format:
  uv run ruff format my_project tests

@test *args="":
  -uv run pytest {{args}}

@lock:
  uv lock

@lock-upgrade:
  uv lock --upgrade

@install:
  uv sync --frozen --all-extras
```

Then you can run `mypy`, `ruff` check, and `ruff` format with:

```sh
just mypy
just ruff-check
just ruff-format
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
formatting. pre-commit caches information and only runs on files that have changed so it is fast
and doesn't slow down your work flow will preventing you from forgetting to run checks.

### FastAPI migrations

[sqlx](https://github.com/launchbadge/sqlx) is used for migrations. A dedicated docker container
runs the migrations each time docker is started. For creating new migrations install `sqlx-cli`.
`sqlx-cli` also needs to be installed in order to run the generated test suite.

```sh
cargo install sqlx-cli --no-default-features --features native-tls,postgres
```

Then to add a new migration run:

```sh
sqlx migrate add -r my_migration
```

This will create new migration up and down files in the migrations directory.

## Contributing

If you are interested in contributing please see our [contributing guide](CONTRIBUTING.md)
