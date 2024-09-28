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
  echo clippy
  just --justfile {{justfile()}} clippy

@clippy:
  cargo clippy --all-targets

@check:
  cargo check --all-targets

@fmt:
  cargo fmt --all

@test:
  cargo insta test

@test-review:
  cargo insta test --review
