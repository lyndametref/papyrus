---
id: TECH-028
title: "Build System"
created_date: '2026-11-27T20:35:43Z'
updated_date: '2026-11-27T20:35:43Z'
labels: []
references:
  - NFR-003
  - NFR-003.001
  - NFR-007.001
status: verified
---
# TECH-028 Build System

## Description
Covers the Cargo build, pinned toolchain, build profiles, build-time tools, and the checks that gate a build.

## Cargo

- Cargo is the only build system, dependency manager, and test runner.
- `apps/Cargo.toml` is a virtual workspace manifest declaring the members `cli`, `browser`, and `libs/core`; each member has its own `Cargo.toml`. `Cargo.lock` and `rust-toolchain.toml` are committed at `apps/` and pin the resolved dependency graph and toolchain.
- `cargo build`, `cargo test`, `cargo fmt --all`, and `cargo clippy --workspace` run from `apps/`; `cargo build -p papyrus-cli --release` writes `apps/target/release/papyrus` and `cargo build -p papyrus-browser --release` writes `apps/target/release/papyrus-browser`. CI runs every Cargo step with `working-directory: apps` or `--manifest-path apps/Cargo.toml`.
- The build uses a Cargo workspace (no parent manifest above `apps/Cargo.toml`) and no code generation before compilation.

## Toolchain

- Rust 1.99.1 via rustup is pinned by `apps/rust-toolchain.toml`.
- The pin is the reproducible baseline; a toolchain change is explicit.
- Build-time tools are rustc, Cargo, rustfmt, and Clippy. Python 3 and groff are used only for man-page generation and verification.
- No Node.js or npm step is part of the build.
- The build performs no network access. Runtime dependencies are limited to the compiled binary.

## Profiles

- The release profile uses LTO and strip for distributed binaries and for the binary used in performance measurement (NFR-007.001).
- The debug profile is a development artifact and is not distributed.
- Build output is confined to `apps/target/`, which `.gitignore` excludes.

## Checks

- `cargo fmt --check` passes.
- `cargo clippy` reports no new warning; any `allow` is local and justified.
- `cargo test` passes.
- The release build completes with the pinned toolchain.
- A build leaves tracked files unchanged.

## Reproducibility

- The committed `apps/Cargo.lock` and `apps/rust-toolchain.toml` define the dependency and toolchain baseline.
- The packaging build uses the local Cargo cache and performs no network access.
- The release binaries are `apps/target/release/papyrus` and `apps/target/release/papyrus-browser`; no runtime asset directory or generated file ships beside either.
