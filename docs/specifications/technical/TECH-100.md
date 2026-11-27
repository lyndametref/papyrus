---
id: TECH-100
title: "Source Tree"
created_date: '2026-11-25T20:53:42Z'
updated_date: '2026-11-27T19:40:35Z'
labels: []
references:
  - NFR-003.001
status: draft
---
# TECH-100 Source Tree

## Description
Covers the repository source layout and the Cargo workspace that holds the three crates. Build profiles and checks, packaging, and command behavior are separate specifications.

## Repository layout

```
repo/
├── README.md
├── .gitignore                 # apps/target/
├── backlog/
├── decisions/
├── docs/
└── apps/
    ├── Cargo.toml             # virtual workspace manifest
    ├── Cargo.lock
    ├── rust-toolchain.toml
    ├── rustfmt.toml
    ├── .cargo/config.toml
    ├── cli/                   # package papyrus-cli
    │   ├── Cargo.toml
    │   └── src/main.rs
    ├── browser/               # package papyrus-browser
    │   ├── Cargo.toml
    │   ├── src/{lib.rs,main.rs,routes/,render/,assets.rs}
    │   └── web/               # embedded browser assets
    └── libs/
        └── core/              # package papyrus-core
            ├── Cargo.toml
            └── src/{lib.rs,model.rs,rules/,store.rs,lock.rs}
```

- The repository root holds documentation, decisions, and backlog records; it holds no `Cargo.toml`, `src/`, `target/`, `Cargo.lock`, or `rust-toolchain.toml`.
- Rust source and build files live only under `apps/`.

## Workspace

- `apps/Cargo.toml` is a virtual workspace manifest: it declares `[workspace]` and members only, with no `[package]`.
- The members are `cli`, `browser`, and `libs/core`.
- One `Cargo.lock`, one `rust-toolchain.toml`, one `rustfmt.toml`, and one `.cargo/config.toml` live at `apps/`.
- Every Cargo command runs from `apps/`; a parent manifest above `apps/Cargo.toml` is forbidden because it would conflict with the workspace root.
- Build output is confined to `apps/target/`, which `.gitignore` excludes.

## Crates

- `libs/core` builds package `papyrus-core` as a library. It holds the record model, rules, configuration and Seed model, store, atomic writes, locking, and serialization.
- `cli` builds package `papyrus-cli` as the binary `papyrus`. It holds the command surface.
- `browser` builds package `papyrus-browser` as the binary `papyrus-browser`. It holds the HTTP server, routes, server-rendered views, and the embedded `web/` assets.

## Dependency edges

- `papyrus-cli` depends on `papyrus-core` only.
- `papyrus-browser` depends on `papyrus-core` only.
- `papyrus-core` depends on no front end.
- `papyrus-cli` and `papyrus-browser` do not depend on each other.
- No path dependency points upward or sideways between the two front ends.

## Browser assets

- Static assets live under `apps/browser/web/` and are embedded into the binary at build time.
- There is no Node.js, npm, or separate front-end build step.
- The browser binary is self-contained; it serves no file from disk for its own assets.

## Commands

- `cargo build -p papyrus-cli` and `cargo build -p papyrus-browser` build one front end each.
- `cargo test --workspace`, `cargo fmt --all`, and `cargo clippy --workspace` run from `apps/`.
- CI runs every Cargo step with `working-directory: apps` or `--manifest-path apps/Cargo.toml`.
- `rust-analyzer` opens `apps/` as the workspace root.
