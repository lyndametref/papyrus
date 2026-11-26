---
id: TECH-100
title: "Source Tree and Build Entry Point"
created_date: '2026-11-25T20:53:42Z'
updated_date: '2026-11-25T20:53:42Z'
labels: []
references:
  - NFR-003.001
  - NFR-003.002
  - NFR-003.003
status: draft
---
# TECH-100 Source Tree and Build Entry Point

## Description
Covers the repository layout of the Papyrus crate and the packaging entry point
`scripts/build-package.sh` with its argument, version, output, and exit
contracts. The Cargo build, toolchain, profiles, and checks are a separate
build-system specification. Command behavior and release scheduling are out of
scope.

## Repository Layout

- `Cargo.toml`, `src/`, `tests/`, `rust-toolchain.toml`, `scripts/`, and
  `packaging/` are top-level entries of the repository root.
- `src/` holds `src/main.rs` and `src/lib.rs`; the crate has a binary target and
  a library target.
- Papyrus is a single crate. The repository contains no Cargo workspace and no
  `apps/<name>/` nesting of the crate.
- Code and build assets live at the repository root; the generic
  multi-application `apps/**` write scopes do not apply to Papyrus.

## Packaging Entry Point

- `scripts/build-package.sh` at the repository root is the packaging entry point
  for the deployable component. No per-component script is introduced.
- File properties: mode `755`, first line `#!/usr/bin/env bash`,
  `set -euo pipefail`.
- Fixed interface:
  - `scripts/build-package.sh --version <version> --out <dir>`
  - `scripts/build-package.sh --help`
- `--version` is the version exactly as supplied by the caller, for example
  `0.2.0-rc.1` or `0.2.0`. It is not read from Git tags and not re-derived from
  the package manifest.
- `--out` is the directory that receives the deliverables.
- `--help` prints usage and exits `0`. A missing or unknown argument exits
  non-zero with a clear message on stderr.
- The script is non-interactive, commits, tags, pushes, uploads, and deploys
  nothing, and is the only command invoked to produce a package; bare
  `cargo build` plus `dpkg-deb` invocation is not a substitute.

## Delegation to the Debian Assembly Step

- `scripts/build-package.sh` invokes `scripts/build-deb.sh`; it does not
  assemble the `.deb` itself.
- `scripts/build-deb.sh` remains the Debian assembly step and the single source
  of truth for package content: the staged tree, the `amd64` architecture, and
  the asset-version verification.
- The entry point contains no `dpkg-deb --build`, no control rendering, and no
  asset-version enumeration; two assembly implementations are not permitted.
- `scripts/build-deb.sh` keeps its direct developer invocation: with no version
  supplied it derives the version from the package manifest and writes the
  `.deb` to the project root. That direct path publishes nothing.

## Version Injection

- The `--version` value is authoritative for the produced package. It appears
  in the `Version:` field of `DEBIAN/control` and in the artifact file name
  `papyrus_<version>_amd64.deb`.
- Injection never rewrites a tracked file: `Cargo.toml`, `Cargo.lock`,
  `packaging/deb/DEBIAN/control`, and every other committed file stay
  byte-identical.
- The carrying mechanism (flag, environment variable, or generated staging file)
  is free, provided an explicitly supplied version overrides the manifest
  derivation, and `git status --porcelain` is empty before and after a run.
- The package version is not the asset version. The assembly step keeps deriving
  the asset version from the vendor directories and the manifest, and the
  injected package version appears in no asset URL.

## Output Isolation and Cleanup

- `--out` receives deliverables only: the `.deb` and, when needed, its
  `.sha256`. The script cleans its own destination subtree before writing, so a
  rerun into the same directory is idempotent.
- The permitted publish location is the git-ignored `builds/<version>/` below
  the repository root; scratch staging stays under the repository-local `.tmp/`.
- The output guard works on the canonicalised path (`realpath -m`): an
  in-repository `--out` is accepted only when it lies strictly below
  `<root>/.tmp/`, or strictly below `<root>/builds/` with at least one further
  path segment. `<root>`, `<root>/builds`, and every other in-repository path
  are refused with exit code `2`.
- Outside the repository, the filesystem root, the system directories, the user
  home directory and its subtree, the repository root, and every parent of the
  repository root are refused.
- After a run, no package file is left in the project root; the Debian staging
  directory under `target/` and the normal tool cache remain the only writes
  outside `--out`. The scripts state this rule in their usage text and diagnostics.

## Stdout and Exit Contract

- stdout carries one produced file path per line; diagnostics go to stderr.
- Exit is non-zero when the build fails, the toolchain is missing, or a promised
  artifact is absent or empty; the script never exits `0` with incomplete output.

## Package Layout

- The deliverable is an `amd64` `.deb` for Debian Forky (NFR-003.002), with the
  binary at `usr/bin/papyrus`, the copyright file at
  `usr/share/doc/papyrus/copyright`, and metadata in `DEBIAN/control`.
- Man pages are installed under the standard man paths, generated at build time
  from the Markdown sources under `docs/man/` (NFR-003.003).

## Build Environment

- The build needs only `cargo` and `dpkg-deb`; no other toolchain is added, and
  no network access occurs at build time.
