---
name: cargo-build
description: 'Build, test, and inspect Rust projects with Cargo: common Cargo commands, test execution, linting with Clippy, formatting with rustfmt, and troubleshooting. Use when building or testing a Rust (Cargo) project.'
argument-hint: 'Examples: run unit tests; build release; check clippy; format code; resolve dependency issues.'
user-invocable: true
disable-model-invocation: false
---

# Cargo Build

## Outcome

Build, test, and diagnose Rust projects using Cargo and the standard toolchain, without breaking the build.

## When To Use

- Building or assembling a Rust crate/binary.
- Running unit, integration, or doc tests.
- Running lint/static analysis (Clippy) or formatting (rustfmt).
- Resolving dependency or build configuration issues.

Do not use this skill for non-Cargo projects (use the project's own build tool).

## Conventions

- Run from the crate/workspace root that contains `Cargo.toml`.
- Prefer the project's pinned toolchain if `rust-toolchain.toml` is present.
- Use `--offline` when dependencies are cached.
- Use `--verbose` / `RUST_BACKTRACE=1` only when diagnosing failures.

## Common tasks

- `cargo build` — build the crate (debug).
- `cargo build --release` — build optimized.
- `cargo check` — fast type-check without codegen.
- `cargo test` — run all tests (unit + integration + doc).
- `cargo test <name>` — run tests matching a name/filter.
- `cargo test -- --nocapture` — show stdout from tests.
- `cargo test --doc` — run doc tests only.
- `cargo clippy` — run Clippy lints.
- `cargo clippy -- -D warnings` — treat warnings as errors.
- `cargo fmt` — format code with rustfmt.
- `cargo fmt --check` — check formatting without writing.
- `cargo run` / `cargo run -- <args>` — build and run the binary.
- `cargo clean` — remove build artifacts.
- `cargo update` — update dependencies within lockfile constraints.
- `cargo tree` — dependency tree.
- `cargo audit` — check dependencies for known vulnerabilities (requires cargo-audit).

## Test execution

- Run the narrowest scope first: single test, then module, then full suite.
- Use `cargo test <filter>` to run a subset.
- Use `-- --nocapture` to see output on failure.
- Run doc tests with `cargo test --doc`.

## Troubleshooting

- **Stale build / corrupt target**: `cargo clean` then rebuild.
- **Dependency conflict**: inspect `cargo tree`; adjust versions in `Cargo.toml` or use `[patch]`.
- **Toolchain mismatch**: check `rust-toolchain.toml` and `rustup show`.
- **Clippy false positives**: allow specific lints with `#[allow(...)]` and a justification; prefer fixing.
- **Slow builds**: enable `sccache`, use `cargo check` during iteration.

## Completion checks

- Build succeeds (`Finished ...`).
- Target tests pass (no failures).
- `cargo clippy` reports no new errors (warnings acceptable unless configured otherwise).
- `cargo fmt --check` passes.
- No files outside the project folder were modified.
