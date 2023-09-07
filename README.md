# Python Project Generator

[![Tests Status](https://github.com/sanders41/python-project-generator/workflows/Testing/badge.svg?branch=main&event=push)](https://github.com/sanders41/python-project-generator/actions?query=workflow%3ATesting+branch%3Amain+event%3Apush)

Generates a Python project structure with Poetry for package management and github actions for
continuous integration and continuous deployment.

## Included packages

- [black](https://github.com/psf/black) for code formatting
- [mypy](https://www.mypy-lang.org/) for static type checking
- [pre-commit](https://github.com/pre-commit/pre-commit) for pre-commit hooks
- [pytest](https://docs.pytest.org/en/latest/) for testing
- [pytest-cov](https://github.com/pytest-dev/pytest-cov) for test coverage reports
- [ruff](https://beta.ruff.rs/docs/) for linting

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

```sh
poetry install
```

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
