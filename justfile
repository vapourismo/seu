set ignore-comments := true

alias c := check
alias t := test
alias f := fix

export CARGO_TERM_QUIET := "true"

[default]
[parallel]
all: check test

[parallel]
check: check-cargo-build-no-std check-cargo-build check-clippy check-rustfmt check-taplo-format

check-cargo-build:
    # We use `build` over `check` to ensure we trigger const evaluation, which may raise panics that
    # aren't caught when only type-checking.
    cargo build --all-targets

check-cargo-build-no-std:
    # We use `build` over `check` to ensure we trigger const evaluation, which may raise panics that
    # aren't caught when only type-checking.
    cargo build --lib --no-default-features

check-clippy:
    cargo clippy --all-targets -- -D warnings

check-rustfmt:
    cargo fmt --all --check

check-taplo-format:
    taplo format --check

audit:
    cargo audit

[parallel]
fix: fix-rustfmt fix-taplo-format

fix-rustfmt:
    cargo fmt --all

fix-taplo-format:
    taplo format

fix-cargo-fetch:
    cargo fetch

test: test-nextest

test-nextest:
    cargo nextest run

coverage:
    cargo llvm-cov test

coverage-report:
    cargo llvm-cov test --html --open
