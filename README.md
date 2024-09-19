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

### Relase Drafter

The [release drafter](https://github.com/release-drafter/release-drafter) action automatically adds
the tile of the PR, who created it, and it's PR number to a draft GitHub release. By default the
release will get a patch version update. Adding a `bug` label will get a patch version update and
add it to the `Bug` section of the release notes. Adding an `enhancement` label to a PR will create
a release with a minor version bump, and a `breaking-change` label will create a major version bump.
The draft release will get the release version tag for the highest label applied to the merged PRs
in the release. PRs can be excluded from the release notes by applying a `skip-changelog` label to
the PR.

## Contributing

If you are interested in contributing please see our [contributing guide](CONTRIBUTING.md)
