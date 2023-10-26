# Python Project Generator

[![Tests Status](https://github.com/sanders41/python-project-generator/workflows/Testing/badge.svg?branch=main&event=push)](https://github.com/sanders41/python-project-generator/actions?query=workflow%3ATesting+branch%3Amain+event%3Apush)

Generates a Python project structure with github actions for continuous integration and continuous
deployment. Both pure Python projects and Python projects with Rust modules using PyO3 can be
created.

## Pure Python project included packages

For package managment choose between:

- [poetry](https://python-poetry.org/)
- [setuptools](https://github.com/pypa/setuptools)

Dev packages:

- [mypy](https://www.mypy-lang.org/) for static type checking
- [pre-commit](https://github.com/pre-commit/pre-commit) for pre-commit hooks
- [pytest](https://docs.pytest.org/en/latest/) for testing
- [pytest-cov](https://github.com/pytest-dev/pytest-cov) for test coverage reports
- [ruff](https://beta.ruff.rs/docs/) for linting and code formatting

## Python project with Rust modules included packages

- [maturin](https://github.com/PyO3/maturin) for package management
- [mypy](https://www.mypy-lang.org/) for static type checking
- [pre-commit](https://github.com/pre-commit/pre-commit) for pre-commit hooks
- [pytest](https://docs.pytest.org/en/latest/) for testing
- [pytest-cov](https://github.com/pytest-dev/pytest-cov) for test coverage reports
- [ruff](https://beta.ruff.rs/docs/) for linting and code formatting
- [PyO3](https://github.com/PyO3/pyo3) for managing the Rust/Python FFI
- [justfile](https://github.com/casey/just) for running commands (to use this you will need to
  install just)

## Installation

First install [Rust](https://www.rust-lang.org/tools/install). Then to install the package run:

```sh
cargo install python-project-generator
```

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

Install the Python dependencies when using Poetry.

```sh
poetry install
```

Install the Python dependencies when using setuptools.

First create a virtual environment and activate it.

```sh
python -m venv .venv
. .venv/bin/activate
```

```sh
python -m pip install -r requirements-dev.txt
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

## Contributing

If you are interested in contributing please see our [contributing guide](CONTRIBUTING.md)
