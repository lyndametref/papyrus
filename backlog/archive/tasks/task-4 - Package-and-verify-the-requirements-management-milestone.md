---
id: TASK-4
title: Package and verify the requirements-management milestone
status: Done
assignee: []
created_date: '2026-10-31T22:25:00Z'
updated_date: '2026-11-02T19:26:00Z'
labels:
  - packaging
  - verification
milestone: m-0
dependencies:
  - TASK-3
references:
  - docs/requirements/non-functional/NFR-03.002.md
  - docs/requirements/non-functional/NFR-06.001.md
  - docs/requirements/non-functional/NFR-06.002.md
  - docs/requirements/business/REQ-01.md
  - docs/requirements/business/REQ-05.md
  - docs/business-rules/BR-009-req-command-behavior.md
  - docs/business-rules/BR-010-nfreq-command-behavior.md
  - >-
    docs/specifications/requirements-management-milestone-packaging-and-verification.md
  - docs/architecture/adr/ADR-0003-supported-debian-release-target-forky-only.md
type: chore
ordinal: 4000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Package and verify the requirements-management milestone (m-0): rebuild the amd64 .deb from the post-TASK-3 source, verify its metadata and GPL-3.0-or-later licensing, install and uninstall it in an isolated Debian Forky environment, and run a smoke test proving the packaged binary can initialize a project and exercise the full req/nfreq lifecycle (create-group, create, list, view, edit, deprecate, restore). Add or extend a Rust integration test covering that lifecycle. Update docs/packaging.md with the Forky-only supported release and the verification results. No new product behavior is added; work is limited to packaging scripts, documentation, and tests.

Scope (in):
- Rebuild the release binary and .deb via scripts/build-deb.sh from the current source.
- Verify package metadata: name papyrus, Architecture amd64, Version == Cargo.toml, GPL-3.0-or-later copyright, documented Depends.
- Install and uninstall the .deb in an isolated Debian Forky environment (host untouched).
- Smoke-test the installed binary: --help, --version, init, and the req/nfreq lifecycle with expected exit codes and persisted files.
- Re-run cargo build --release, cargo test, cargo clippy --all-targets -- -D warnings, cargo fmt --check with Rust 1.99.1.
- Add/extend a Rust integration test in tests/ for the req/nfreq lifecycle against the built binary.
- Update docs/packaging.md (Forky-only supported release per ADR-0003, install/uninstall, smoke-test checklist and results).

Scope (out):
- No new product features, command behavior, or record types (TASK-5+).
- No req/nfreq implementation changes beyond bug fixes found during verification.
- No verification on Debian Trixie or any release other than Forky (see OQ-1).
- No advertising of support beyond Forky (NFR-03.002 amendment pending).
- No UI, search, or browser work.
- No edits to requirements files or business rules.
- No refresh of the runtime dependency-license audit (TASK-1 audit remains valid).

Assumptions:
- Assumption: Rust 1.99.1 and dpkg-deb are available in the build environment (ADR-0001, ADR-0003).
- Assumption: an isolated Debian Forky environment (container/chroot/VM) is available; the host is never modified.
- Assumption: Trixie verification is not performed; the supported-release claim is narrowed to Forky (ADR-0003). Amending NFR-03.002 to match is pending and out of this ticket's write scope (OQ-1).
- Assumption: the Cargo package version is the single source of truth for the .deb version and --version output.
- Assumption: the TASK-1 runtime dependency-license audit remains valid; no new runtime dependencies were introduced by TASK-2/TASK-3 beyond those already audited.
- Assumption: automated integration tests use the built binary (CARGO_BIN_EXE_papyrus) and temp directories, consistent with tests/req_cli.rs and tests/cli.rs.
- Assumption: --editor smoke testing is environment-dependent and is verified by review rather than automated test.

Open questions:
- OQ-1: NFR-03.002 still states "Debian Trixie, Forky, and later Debian releases". This task and ADR-0003 narrow the verified/advertised target to Forky only. Amending NFR-03.002 requires the requirements-engineer and is out of this ticket's write scope. Until amended, the ticket's supported-release claim conflicts with NFR-03.002 as written.
- OQ-2: Whether the new integration test duplicates existing tests/req_cli.rs coverage or adds a consolidated end-to-end lifecycle test; implementation should avoid redundancy and prefer filling gaps (exit-code matrix, ID preservation).

Dependencies: TASK-3 (Done). Also depends on requirements-engineer (NFR-03.002 amendment) and architect (ADR-0003) for the supported-release claim.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 cargo build --release with the pinned Rust 1.99.1 toolchain succeeds and produces target/release/papyrus.
- [ ] #2 cargo test, cargo clippy --all-targets -- -D warnings, and cargo fmt --check all pass.
- [ ] #3 scripts/build-deb.sh produces papyrus_<version>_amd64.deb with Architecture: amd64 and Version equal to the Cargo package version.
- [ ] #4 dpkg-deb metadata reports Package: papyrus, Architecture: amd64, and the copyright file declares GPL-3.0-or-later.
- [ ] #5 In an isolated Debian Forky environment, dpkg -i papyrus_<version>_amd64.deb succeeds and places papyrus on PATH (/usr/bin/papyrus).
- [ ] #6 In the same isolated environment, dpkg -r (or dpkg --purge) papyrus removes the binary and papyrus is no longer on PATH.
- [ ] #7 The installed binary smoke test passes: papyrus --help exits 0 with usage, and papyrus --version exits 0 printing papyrus <version>.
- [ ] #8 The installed binary can init a project and exercise req and nfreq create-group, create, list, view, edit, deprecate, and restore with expected exit codes and persisted files, including ID preservation across deprecate/restore.
- [ ] #9 A Rust integration test in tests/ exercises the req/nfreq lifecycle end-to-end against the built binary and passes under cargo test.
- [ ] #10 docs/packaging.md is updated to state Debian Forky only (per ADR-0003) and records install, uninstall, and smoke-test procedures with verification results.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 All acceptance criteria verified and recorded in task notes or docs/packaging.md.
- [ ] #2 cargo build --release, cargo test, cargo clippy --all-targets -- -D warnings, and cargo fmt --check pass with Rust 1.99.1.
- [ ] #3 amd64 .deb builds and installs/uninstalls cleanly in an isolated Debian Forky environment.
- [ ] #4 Installed-binary smoke test (help, version, init, req/nfreq lifecycle) passes.
- [ ] #5 Rust integration test for the req/nfreq lifecycle passes.
- [ ] #6 docs/packaging.md updated with Forky-only claim and verification results.
- [ ] #7 No host system modification occurred during verification.
- [ ] #8 Acceptance criteria reviewed and signed off.
<!-- DOD:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Rebuild release binary and .deb; verify metadata.
2. Run Rust quality gates (build, test, clippy, fmt).
3. Install/uninstall in isolated Forky environment.
4. Run installation smoke test (help, version, init, req/nfreq lifecycle).
5. Add/extend Rust integration test for req/nfreq lifecycle.
6. Update docs/packaging.md with Forky-only claim and verification results.
7. Record results in task notes.
<!-- SECTION:PLAN:END -->
