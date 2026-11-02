# Feature Specification — Requirements-Management Milestone Packaging and Verification

**Ticket:** TASK-4
**Milestone:** m-0 (Rust CLI Foundation and Requirements)
**Status:** Refined (execution-ready pending open question OQ-1)
**UI mockups:** none

## Purpose

Package the current Papyrus feature set (TASK-1 foundation + TASK-2 init/config + TASK-3 req/nfreq) as an installable amd64 Debian package and verify, in an isolated environment, that the packaged binary installs, uninstalls, and can initialize a project and exercise the full business/non-functional requirement lifecycle. This is a packaging/verification task: no new product behavior is added. It closes milestone m-0's "each increment is packaged and usable" goal for the requirements-management capability.

## Scope

### In scope

- Rebuild the release binary and `.deb` from the current source using `scripts/build-deb.sh`.
- Verify package metadata: package name `papyrus`, `Architecture: amd64`, `Version` equal to `Cargo.toml`, GPL-3.0-or-later copyright, and `Depends` as documented.
- Verify the package installs and uninstalls in an isolated Debian Forky environment (no host modification).
- Run the installation smoke test against the installed binary in that isolated environment: `--help`, `--version`, project `init`, and the req/nfreq lifecycle (create-group, create, list, view, edit, deprecate, restore).
- Re-run the Rust quality gates: `cargo build --release`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check` with the pinned Rust 1.99.1 toolchain.
- Add/extend a Rust integration test in `tests/` that exercises the req/nfreq lifecycle end-to-end against the built binary, closing any coverage gap not already present in `tests/req_cli.rs`.
- Update `docs/packaging.md` to reflect the verified supported release (Forky only), install/uninstall steps, and the smoke-test checklist with observed results.
- Record verification results (pass/fail, environment, date) in `docs/packaging.md` or the task notes.

### Out of scope

- New product features, command behavior, or record types (TASK-5 onward).
- Changes to req/nfreq implementation except bug fixes found during verification.
- Verification on Debian Trixie or any release other than Forky (see OQ-1).
- Advertising support for any Debian release beyond Forky (NFR-03.002 amendment pending).
- UI mockups or web UI.
- Changes to requirements files (`docs/requirements/`) or business rules (`docs/business-rules/`).
- Refreshing the runtime dependency-license audit table (TASK-1 audit remains valid; no new runtime dependencies were added by TASK-2/TASK-3 beyond those already audited).

## Behavior

### 1. Build and package

1. `cargo build --release` succeeds with the pinned Rust 1.99.1 toolchain and produces `target/release/papyrus`.
2. `cargo test`, `cargo clippy --all-targets -- -D warnings`, and `cargo fmt --check` pass.
3. `./scripts/build-deb.sh` produces `papyrus_<version>_amd64.deb` in the project root.
4. `dpkg-deb -I` reports `Package: papyrus`, `Architecture: amd64`, `Version` equal to the Cargo package version, and GPL-3.0-or-later copyright.
5. `dpkg-deb -c` lists `/usr/bin/papyrus` and the copyright file.

### 2. Install and uninstall (isolated Debian Forky environment)

1. `dpkg -i papyrus_<version>_amd64.deb` succeeds in the isolated environment.
2. After install, `papyrus` is on `PATH`; `which papyrus` resolves to `/usr/bin/papyrus`.
3. `dpkg -r papyrus` (or `dpkg --purge papyrus`) removes the binary; `papyrus` is no longer on `PATH`.
4. No host system is modified; all install tests run in an isolated environment (container/chroot/VM).

### 3. Smoke test (installed binary, isolated Forky environment)

1. `papyrus --help` exits 0 and prints usage.
2. `papyrus --version` exits 0 and prints `papyrus <version>`.
3. `papyrus --dir <tmp> init <name>` creates `papyrus.config.yml` and the configured directory structure.
4. `papyrus req create-group --title <title>` creates a group and prints its ID (`REQ-<n>`).
5. `papyrus req create <title> --group <id>` creates a requirement and prints its ID (`<group>.<seq>`).
6. `papyrus req list` shows the active group and requirement by ID and title, in ID order, excluding deprecated records.
7. `papyrus req view <id>` prints the full persisted text; an unknown ID exits 3.
8. `papyrus req edit <id>` updates only supplied fields; `--editor` is not exercised in the automated smoke test (environment-dependent).
9. `papyrus req deprecate <id>` moves the file to `deprecated/`; the record is hidden from `list`.
10. `papyrus req restore <id>` returns the file with the same ID; the record reappears in `list`.
11. The same lifecycle holds for `nfreq` (create-group, create with `--group`, list, view, edit, deprecate, restore).
12. Exit statuses follow the TASK-2 contract: 0 success, 1 IO/boundary, 2 usage, 3 not found, 4 validation.

### 4. Automated integration test

1. A Rust integration test in `tests/` exercises the req/nfreq lifecycle end-to-end against the built binary (`CARGO_BIN_EXE_papyrus`) and passes under `cargo test`.
2. The test covers ID format/padding, group requirement, list ordering, deprecate/restore ID preservation, and exit codes for missing group / unknown ID / validation.

### 5. Documentation

1. `docs/packaging.md` states the supported release as Debian Forky only (amd64), per ADR-0003.
2. `docs/packaging.md` contains the install, uninstall, and smoke-test procedures with the verification results for this task.
3. The runtime dependency-license audit table from TASK-1 is referenced (not re-run) and confirmed unchanged.

## Inputs and outputs

- Inputs: current source tree (post TASK-3), `scripts/build-deb.sh`, `packaging/deb/**`.
- Outputs: `papyrus_<version>_amd64.deb`, updated `docs/packaging.md`, verification notes, optional integration test in `tests/`.

## Validation and error behavior

- Missing/invalid usage in smoke-test commands → exit code 2.
- Unknown record/group ID → exit code 3.
- Validation/conflict (missing `--group`, type mismatch, deprecated-state conflict, unnormalizable input) → exit code 4.
- IO failure or project-directory boundary violation → exit code 1.
- Any quality-gate failure or install failure blocks completion.

## Assumptions

- Assumption: Rust 1.99.1 via rustup and `dpkg-deb` are available in the build environment (ADR-0001, ADR-0003).
- Assumption: an isolated Debian Forky environment (container/chroot/VM) is available for install/uninstall/smoke tests; the host is never modified.
- Assumption: Trixie verification is not performed in this task; supported-release claim is narrowed to Forky (ADR-0003). Amending NFR-03.002 to match is pending and out of this ticket's scope (see OQ-1).
- Assumption: the Cargo package version remains the single source of truth for the `.deb` version and `--version` output.
- Assumption: the TASK-1 runtime dependency-license audit remains valid; no new runtime dependencies were introduced by TASK-2/TASK-3 that are absent from that audit (yaml_serde and serde_json were already audited).
- Assumption: automated integration tests use the built binary (`CARGO_BIN_EXE_papyrus`) and temp directories, consistent with existing `tests/req_cli.rs` and `tests/cli.rs`.
- Assumption: `--editor` smoke testing is environment-dependent and is verified by code review/manual check rather than automated test.

## Open questions

- OQ-1: NFR-03.002 still states "Debian Trixie, Forky, and later Debian releases". This task and ADR-0003 narrow the verified/advertised target to Forky only. Amending NFR-03.002 (and any linked docs) requires the requirements-engineer and is out of this ticket's write scope. Until amended, the ticket's supported-release claim conflicts with NFR-03.002 as written. **Blocker for marking execution-ready if strict requirement alignment is required.**
- OQ-2: Whether the new integration test duplicates existing `tests/req_cli.rs` coverage or adds a consolidated end-to-end lifecycle test; implementation should avoid redundant tests and prefer filling gaps (exit-code matrix, ID preservation).

## Traceability

| Artifact | Reference |
|---|---|
| Requirement | REQ-01, REQ-05 |
| Non-functional | NFR-03.002, NFR-06.001, NFR-06.002 |
| Business rule | BR-009 (req), BR-010 (nfreq) |
| ADR | ADR-0001 (Rust), ADR-0003 (Forky-only supported release; supersedes ADR-0002) |
| Ticket | TASK-4 |
| UI mockups | none |
| Related specification | docs/specifications/rust-cli-and-debian-package-baseline.md (TASK-1) |
| Related specification | docs/specifications/project-initialization-and-configuration.md (TASK-2) |
| Related specification | docs/specifications/business-and-non-functional-requirements-management.md (TASK-3) |
path: docs/specifications/requirements-management-milestone-packaging-and-verification.md