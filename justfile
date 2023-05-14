#!/usr/bin/env just --justfile
# Adapted from the excellent justfile of HASH
# <https://github.com/hashintel/hash/blob/0bff2d6f6bd55825400967efd541ab3d155cc8ad/.justfile>

set dotenv-load := true

repo := `git rev-parse --show-toplevel`
profile := env_var_or_default('PROFILE', "dev")

######################################################################
## Helper to print a message when calling `just`
######################################################################

[private]
default:
  @echo "Usage: just <recipe>"
  @just list-recipes
  @echo "For further information, run 'just --help'"

# List recipes in this file and from the calling directory
[private]
usage:
  @echo "Usage: just <recipe>"
  @just list-recipes
  @echo "For further information, run 'just --help'"

[private]
list-recipes:
  @echo "\nRepository recipes:"
  @just --list --unsorted --list-heading ''

######################################################################
## Helper to run a command on an environmental condition
######################################################################

# Runs the provided command if `PROFILE` starts with `"dev"`
[private]
in-dev +command:
  #!/usr/bin/env bash
  set -euo pipefail
  if [[ {{ profile }} =~ dev.* ]]; then
    echo "{{command}}" >&2
    {{command}}
  fi

######################################################################
## Install scripts
######################################################################

[private]
install-cargo-tool tool install version:
  @`{{tool}} --version | grep -q "{{version}}" || cargo install "{{install}}" --version "{{version}}" --locked --force`

[private]
install-cargo-hack:
  @just install-cargo-tool 'cargo hack' cargo-hack 0.5.26

[private]
install-cargo-nextest:
  @just install-cargo-tool 'cargo nextest' cargo-nextest 0.9.37

[private]
install-llvm-cov:
  @just install-cargo-tool 'cargo llvm-cov' cargo-llvm-cov 0.5.9


######################################################################
## Predefined commands
######################################################################

# Runs all linting commands and fails if the CI would fail
lint:
  @just format --check
  @just clippy -- -D warnings
  @RUSTDOCFLAGS='-Z unstable-options --check' just doc
  @RUSTDOCFLAGS='-Z unstable-options --check' just doc --document-private-items

# Format the code using `rustfmt`
format *arguments:
  cargo fmt --all {{arguments}}

# Lint the code using `clippy`
clippy *arguments: install-cargo-hack
  cargo hack --workspace --optional-deps --feature-powerset clippy --profile {{profile}} --all-targets --no-deps {{arguments}}

# Creates the documentation for the crate
doc *arguments:
  dot -Tsvg < "{{repo}}/doc/graph-example.dot" > "{{repo}}/doc/graph-example.svg"
  svgo "{{repo}}/doc/graph-example.svg"

  RUSTDOCFLAGS="--extend-css {{repo}}/doc/custom.css" cargo doc --workspace --all-features --no-deps -Zunstable-options -Zrustdoc-scrape-examples {{arguments}}


# Builds the crate
build *arguments:
  cargo build --profile {{profile}} {{arguments}}

# Run the test suite
test *arguments: install-cargo-nextest install-cargo-hack
  cargo hack --workspace --optional-deps --feature-powerset nextest run --cargo-profile {{profile}} {{arguments}}

  @just in-dev cargo test --profile {{profile}} --workspace --all-features --doc

# Run the test suite with `miri`
miri *arguments:
  cargo miri test --workspace --all-features --all-targets {{arguments}}

# Runs the benchmarks
bench *arguments:
  cargo bench --workspace --all-features --all-targets {{arguments}}

# Run the test suite and generate a coverage report
coverage *arguments: install-llvm-cov
  cargo llvm-cov nextest --workspace --all-features --all-targets {{arguments}}
