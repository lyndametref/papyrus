# Feature Specification — Rust CLI and Debian Package Baseline

**Ticket:** TASK-1
**Milestone:** m-0 (Rust CLI Foundation and Requirements)
**Status:** Refined (execution-ready)
**UI mockups:** none

## Purpose

Establish the installable executable foundation for Papyrus: a Rust command-line binary that builds and runs with documented help and version output, packaged as an amd64 Debian `.deb` for Debian Trixie and Forky, licensed under GPL-3.0-or-later, with a runtime dependency-license audit. All later Papyrus capabilities depend on this baseline (TASK-2 … TASK-22).

## Scope

### In scope

- Create the Papyrus Rust crate (single crate, binary + library target) using Cargo, pinned to the Rust 1.99.1 toolchain (ADR-0001).
- Implement the minimal CLI surface: `papyrus --help` / `papyrus -h` and `papyrus --version`, plus the documented global option declarations needed for help rendering, exiting with status 0 on success (BR-001, README `papyrus(1)`).
- Declare GPL-3.0-or-later licensing: `LICENSE` file and crate/package metadata (`Cargo.toml` license field and `.deb` copyright/license metadata).
- Build an amd64 `.deb` package (`papyrus_<version>_amd64.deb`) using `dpkg-deb`, with control metadata (Package, Version, Architecture, Maintainer, Description, Section, Priority, and dependency declarations).
- Document the build, packaging, install, uninstall, and smoke-test procedure.
- Audit runtime dependency licenses and record the result.

### Out of scope

- Subcommands (`init`, `config`, `task`, …) and their behavior — TASK-2 onward.
- Global option runtime behavior beyond help/version rendering (e.g. `--dir`, `--json`, `--config`, `--no-color` semantics) — TASK-2.
- Multi-crate workspace or internal module architecture beyond a minimal `src/main.rs` + `src/lib.rs` split.
- Additional architectures (arm64, …) or non-Debian distribution formats — ADR-0002.
- Advertising support for Debian releases beyond Trixie and Forky — open question NFR-03.002 / ADR-0002 remains unresolved.

## Behavior

1. **Build:** `cargo build --release` (with the pinned toolchain) succeeds and produces an executable named `papyrus`.
2. **Help:** `papyrus --help` and `papyrus -h` print the documented usage/help text and exit 0.
3. **Version:** `papyrus --version` prints `papyrus <version>` (Cargo package version) and exits 0.
4. **Invalid usage:** unknown flags/arguments print usage to stderr and exit with status 2 (usage error), per BR-001 exit-status contract.
5. **Packaging:** the documented packaging command produces `papyrus_<version>_amd64.deb` with `Architecture: amd64`.
6. **Install:** `dpkg -i papyrus_<version>_amd64.deb` succeeds on Debian Trixie and Forky; the `papyrus` binary is on `PATH`.
7. **Uninstall:** `dpkg -r papyrus` (or `dpkg --purge`) removes the binary cleanly.
8. **Smoke test (installed):** `papyrus --help` and `papyrus --version` succeed on the installed binary.
9. **Licensing:** distributed artifacts declare GPL-3.0-or-later; runtime dependencies are open source and their licenses permit linking in a GPLv3-or-later application.

## Packaging details

- Package name: `papyrus`.
- Architecture: `amd64`.
- Supported suites declared: Debian Trixie and Forky only. Later Debian releases are not advertised until the NFR-03.002 open question is resolved (ADR-0002).
- Forward compatibility: the binary is built against a conservative glibc baseline so it is expected to run on later Debian releases, but this is not a supported/advertised claim.
- Control fields follow Debian policy; maintainer and homepage values are recorded in packaging metadata.

## Licensing and dependency audit

- License: GPL-3.0-or-later (SPDX `GPL-3.0-or-later`).
- Audit scope: **runtime dependencies only** (crates linked into the distributed binary). Dev/build-time dependencies are out of scope for this task; if a build-time dependency's license is later found incompatible, it is tracked separately.
- Audit output: a documented list of every runtime dependency with its license (SPDX identifier) and a compatibility verdict (compatible / incompatible). Stored with the packaging notes.
- Any incompatible runtime dependency blocks completion of this task.

## Assumptions

- Assumption: Rust 1.99.1 (via rustup) is available in the build environment, per ADR-0001.
- Assumption: `dpkg-deb` is available for packaging, per ADR-0002.
- Assumption: the Cargo package version is the single source of truth for the `.deb` version and the `--version` output.
- Assumption: Node.js/Python, if used, remain build-time/auxiliary tooling only and are never runtime dependencies (ADR-0001, ADR-0002).
- Assumption: minimal single-crate layout (`src/main.rs` + `src/lib.rs`); any move to a workspace or module boundaries beyond this requires a new ADR and is out of scope.
- Assumption: man-page installation in the `.deb` is optional and may be deferred; help text is the required documentation surface for this task.
- Assumption: package metadata values not specified by requirements (maintainer, section, priority) follow Debian policy defaults and are recorded in packaging files.

## Open questions

- OQ-1: The "later Debian releases" boundary (NFR-03.002) is unresolved; this task advertises only Trixie and Forky. Resolution is tracked against NFR-03.002 and ADR-0002.
- OQ-2: The exact interpretation of "open source" and "linking" for dev/build-time dependencies (NFR-06.002) remains unresolved; this task audits runtime dependencies only.

## Traceability

| Artifact | Reference |
|---|---|
| NFR | NFR-01.001, NFR-03.001, NFR-03.002, NFR-06.001, NFR-06.002 |
| Business rule | BR-001 (papyrus command behavior) |
| ADR | ADR-0001 (Rust), ADR-0002 (amd64 .deb) |
| Ticket | TASK-1 |
| UI mockups | none |
path: docs/specifications/rust-cli-and-debian-package-baseline.md