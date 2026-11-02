---
id: TASK-1
title: Establish Rust CLI and Debian package baseline
status: Refined
assignee: []
created_date: '2026-10-31T22:25:00Z'
updated_date: '2026-11-02T17:40:00Z'
labels:
  - foundation
milestone: m-0
dependencies: []
references:
  - docs/requirements/non-functional/NFR-01.001.md
  - docs/requirements/non-functional/NFR-03.001.md
  - docs/requirements/non-functional/NFR-03.002.md
  - docs/requirements/non-functional/NFR-06.001.md
  - docs/requirements/non-functional/NFR-06.002.md
  - docs/business-rules/BR-001-papyrus-command-behavior.md
  - docs/specifications/rust-cli-and-debian-package-baseline.md
  - docs/architecture/adr/ADR-0001-implementation-language-rust.md
  - docs/architecture/adr/ADR-0002-distribution-format-amd64-deb-packages.md
type: chore
ordinal: 1000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Establish the installable executable foundation for Papyrus before any project-management capability is delivered. The baseline must (a) implement Papyrus as a Rust command-line application that builds and runs with documented help and version output, (b) package it as an amd64 Debian `.deb` for Debian Trixie and Forky, and (c) declare GPL-3.0-or-later licensing with a runtime dependency-license audit satisfying the recorded compatibility constraint.

Scope (in): create the Papyrus Rust crate (single crate, binary + library) on the pinned Rust 1.99.1 toolchain (ADR-0001); implement the minimal CLI surface (`--help`/`-h`, `--version`) with correct exit codes (BR-001); declare GPL-3.0-or-later in source and package metadata; build an amd64 `.deb` via `dpkg-deb` with standard control metadata; document build/package/install/uninstall/smoke-test steps; audit runtime dependency licenses.

Scope (out): subcommands and their behavior (TASK-2+); runtime semantics of global options beyond help/version (TASK-2); multi-crate/workspace layout or internal module architecture beyond `src/main.rs` + `src/lib.rs`; additional architectures or non-Debian formats (ADR-0002); advertising support for Debian releases beyond Trixie and Forky (NFR-03.002 open question).

Assumptions:
- Assumption: Rust 1.99.1 via rustup and `dpkg-deb` are available in the build environment (ADR-0001, ADR-0002).
- Assumption: the Cargo package version is the single source of truth for the `.deb` version and `--version` output.
- Assumption: Node.js/Python, if used, remain build-time/auxiliary tooling only and are never runtime dependencies.
- Assumption: minimal single-crate layout (`src/main.rs` + `src/lib.rs`); any move to a workspace or module boundaries beyond this requires a new ADR.
- Assumption: man-page installation in the `.deb` is optional and may be deferred; help text is the required documentation surface.
- Assumption: package metadata values not specified by requirements (maintainer, section, priority) follow Debian policy defaults and are recorded in packaging files.

Open questions:
- OQ-1: The "later Debian releases" boundary (NFR-03.002) is unresolved; this task advertises only Trixie and Forky. Tracked against NFR-03.002 / ADR-0002.
- OQ-2: The interpretation of "open source" and "linking" for dev/build-time dependencies (NFR-06.002) is unresolved; this task audits runtime dependencies only.

Dependencies: none (foundational task). All other tasks (TASK-2..TASK-22) depend on this one.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Rust build: `cargo build --release` with the pinned Rust 1.99.1 toolchain succeeds and produces an executable named `papyrus`.
- [ ] #2 Help: `papyrus --help` and `papyrus -h` print the documented help/usage text and exit with status 0.
- [ ] #3 Version: `papyrus --version` prints `papyrus <version>` (Cargo package version) and exits with status 0.
- [ ] #4 Invalid usage: unknown flags/arguments print usage to stderr and exit with status 2.
- [ ] #5 Packaging: the documented packaging procedure produces `papyrus_<version>_amd64.deb` with `Architecture: amd64`.
- [ ] #6 Install: `dpkg -i papyrus_<version>_amd64.deb` succeeds on Debian Trixie and Forky and places `papyrus` on PATH.
- [ ] #7 Uninstall: `dpkg -r papyrus` (or purge) removes the binary cleanly.
- [ ] #8 Smoke test: `papyrus --help` and `papyrus --version` succeed on the installed binary.
- [ ] #9 Licensing: distributed artifacts declare GPL-3.0-or-later, and every runtime dependency's license permits linking in a GPLv3-or-later application (audit documented).
- [ ] #10 Release support declared: package metadata/docs advertise Debian Trixie and Forky only (no unverified later release is claimed).
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Rust source builds with the pinned toolchain with no warnings (cargo build --release, cargo test).
- [ ] #2 amd64 .deb installs and uninstalls cleanly on Debian Trixie and Forky.
- [ ] #3 Installed-binary help and version smoke test passes.
- [ ] #4 Runtime dependency-license audit lists every runtime dependency, its license, and a compatibility verdict; no incompatible runtime dependency remains.
- [ ] #5 Build, packaging, install, uninstall, and smoke-test steps are documented.
- [ ] #6 Licensing declaration (LICENSE plus crate and package metadata) reviewed for GPL-3.0-or-later.
- [ ] #7 Acceptance criteria reviewed and signed off.
<!-- DOD:END -->
