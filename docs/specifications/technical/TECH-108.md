---
id: TECH-108
title: "Build System"
created_date: '2026-11-26T07:13:33Z'
updated_date: '2026-11-27T18:46:00Z'
labels: []
references:
  - NFR-003
  - NFR-003.001
  - NFR-007.001
status: verified
---
# TECH-108 Build System

## Description
Covers the Cargo build, pinned toolchain, build profiles, build-time tools, and
the checks that gate a build. Source layout and the packaging entry point are
separate specifications.

## Cargo

- Cargo is the only build system, dependency manager, and test runner.
- `Cargo.toml` declares the crate and its direct dependencies; `Cargo.lock` is committed and pins the resolved dependency graph.
- `cargo build` and `cargo test` run from the repository root; `cargo build --release` writes `target/release/papyrus`.
- The build uses no Cargo workspace and no code generation before compilation.

## Toolchain

- Rust 1.99.1 via rustup is pinned by `rust-toolchain.toml`.
- The pin is the reproducible baseline; a toolchain change is explicit.
- Build-time tools are rustc, Cargo, rustfmt, and Clippy. Python 3 and groff are used only for man-page generation and verification.
- No Node.js or npm step is part of the build.
- The build performs no network access. Runtime dependencies are limited to the compiled binary.

## Profiles

- The release profile uses LTO and strip for distributed binaries and for the binary used in performance measurement (NFR-007.001).
- The debug profile is a development artifact and is not distributed.
- Build output is confined to `target/`.

## Size budget

- The release binary size budget is 12 MiB.

## Checks

- `cargo fmt --check` passes.
- `cargo clippy` reports no new warning; any `allow` is local and justified.
- `cargo test` passes.
- The release build completes with the pinned toolchain.
- A build leaves tracked files unchanged.

## Reproducibility

- The committed `Cargo.lock` and `rust-toolchain.toml` define the dependency and toolchain baseline.
- The packaging build uses the local Cargo cache and performs no network access.
- The release binary is `target/release/papyrus`; no runtime asset directory or generated file ships beside it.
