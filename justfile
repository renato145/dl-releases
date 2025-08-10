_default:
  @just --choose

checks:
  #!/usr/bin/env bash
  set -x
  cargo clippy --all-targets
  cargo fmt --all -- --check

clippy-pedantic:
  cargo clippy --workspace -- -W clippy::pedantic

deny-init:
  cargo deny init

deny-check:
  cargo deny check advisories
