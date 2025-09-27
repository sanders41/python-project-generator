@_default:
  just --list

@all:
  echo fmt
  just --justfile {{justfile()}} fmt
  echo check
  just --justfile {{justfile()}} check
  echo clippy
  just --justfile {{justfile()}} clippy
  echo test
  just --justfile {{justfile()}} test-review


@lint:
  echo fmt
  just --justfile {{justfile()}} fmt
  echo check
  just --justfile {{justfile()}} check
  echo check fastapi features
  just --justfile {{justfile()}} check-fastapi
  echo clippy
  just --justfile {{justfile()}} clippy
  echo clippy ffastapi features
  just --justfile {{justfile()}} clippy-fastapi

@clippy:
  cargo clippy

@clippy-fastapi:
  cargo clippy -F fastapi

@check:
  cargo check

@check-fastapi:
  cargo check -F fastapi

@fmt:
  cargo fmt --all

@test:
  cargo insta test

@test-fastapi:
  cargo insta test -F fastapi

@test-all:
  echo testing no fastapi feature
  just --justfile {{justfile()}} test
  echo testing with fastapi feature
  just --justfile {{justfile()}} test-fastapi

@test-review:
  cargo insta test --review

@test-review-fastapi:
  cargo insta test --review -F fastapi
