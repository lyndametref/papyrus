---
id: ADR-001
title: "Three-Crate Workspace and Delivered Artifacts"
created_date: '2026-11-27T21:02:42Z'
updated_date: '2026-11-27T21:02:42Z'
labels: []
references: []
superseded_by: null
---
# ADR-001 Three-Crate Workspace and Delivered Artifacts

## Status
proposed

## Decision

Adopt one Cargo workspace rooted at `apps/` with three member crates:

- `papyrus-core` — library: record model, rules, configuration and Seed model, store, atomic writes, locking, serialization. It depends on the standard library and general-purpose crates only.
- `papyrus-cli` — binary `papyrus`: the command surface. It depends on `papyrus-core` only.
- `papyrus-browser` — binary `papyrus-browser` with embedded web assets: HTTP server, routes, and server-rendered views. It depends on `papyrus-core` only.

Allowed edges are `cli → core` and `browser → core`. `core` depends on no front end, `browser` does not depend on `cli`, and `cli` does not depend on `browser`.

Three Debian packages are delivered, one per crate, and their package dependencies mirror the crate edges:

- `papyrus-core_<version>_amd64.deb` — the library artifact for reuse; it installs no executable and depends on no other Papyrus package.
- `papyrus_<version>_amd64.deb` — installs `/usr/bin/papyrus` and depends on `papyrus-core` at the same version.
- `papyrus-browser_<version>_amd64.deb` — installs `/usr/bin/papyrus-browser` and depends on `papyrus-core` at the same version.

No package depends on the other front end: `papyrus` does not depend on `papyrus-browser`, and `papyrus-browser` does not depend on `papyrus`. Neither executable invokes the other. Each binary statically links `papyrus-core`; the package dependency records the shared library version even though no file from the core package is loaded at runtime.

The three packages carry the same version and are produced by one packaging run.

No `Cargo.toml`, `src/`, `target/`, `Cargo.lock`, or `rust-toolchain.toml` exists at the repository root. Cargo is always run from `apps/`, where one workspace manifest, one lockfile, and one toolchain file live.

## Context

One product capability is delivered by two front ends over one shared file-corpus engine. The repository root also holds documentation and organisational material and must stay free of Rust build files. The current single-crate assumption lets front-end code and corpus rules entangle, gives the browser server no boundary for in-process testing, and makes asset embedding and reuse unclear.

## Consequences

- Corpus rules exist once in `papyrus-core`; the CLI and browser contain no validation, storage, or serialization rule.
- Core is testable without a process or HTTP; one conformance suite covers the rules, and the front ends keep thin smoke tests.
- The browser router is testable in-process inside `papyrus-browser`, without spawning a server.
- Neither front end links the other, invokes the other, or depends on the other at package level.
- Packaging produces three `.deb` files, three file lists, and three smoke tests, all at one version.
- Both front-end packages declare `papyrus-core` at the same version, so an installed CLI or browser always records which core version it was built against.
- The core package is a development and reuse artifact; the executables do not load it at runtime because the library is statically linked.
- The workspace has one lockfile and one pinned toolchain under `apps/`; `cargo` commands run from `apps/`, so CI steps need `working-directory: apps` or `--manifest-path`.
- `rust-analyzer` opens `apps/` as the workspace root, and `.gitignore` covers `apps/target/`.
- Adding a root `Cargo.toml` later is forbidden: a parent manifest above `apps/Cargo.toml` would conflict with the workspace root.
- The source-tree, build, distribution, licensing, and command specifications must be updated from the single-crate, single-package, and single-command description to this workspace and two-command layout.
