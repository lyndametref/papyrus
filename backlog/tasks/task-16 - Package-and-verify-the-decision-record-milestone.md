---
id: TASK-16
title: Package and verify the decision-record milestone
status: Done
assignee: []
created_date: '2026-10-31T22:26:00Z'
updated_date: '2026-11-05T18:54:00Z'
labels: []
milestone: m-3
dependencies:
  - TASK-14
  - TASK-15
references:
  - docs/requirements/business/REQ-04.md
  - docs/requirements/business/REQ-06.001.md
  - docs/requirements/business/REQ-06.002.md
  - docs/requirements/non-functional/NFR-03.002.md
  - docs/requirements/non-functional/NFR-06.001.md
  - docs/requirements/non-functional/NFR-06.002.md
  - docs/business-rules/BR-008-decision-command-behavior.md
  - docs/specifications/decision-records-milestone-packaging-and-verification.md
  - docs/requirements/non-functional/NFR-04.001.md
  - docs/requirements/non-functional/NFR-05.001.md
  - docs/business-rules/BR-004-browser-command-behavior.md
  - docs/business-rules/BR-015-search-command-behavior.md
  - docs/business-rules/BR-016-configuration-keys-and-precedence.md
documentation:
  - docs/specifications/decision-records-milestone-packaging-and-verification.md
ordinal: 16000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Package and verify the decision-records milestone (m-3): rebuild the amd64 .deb from the post-TASK-15 source, verify its metadata and GPL-3.0-or-later licensing, install and uninstall it in an isolated Debian Forky environment, and run smoke tests proving the packaged binary can manage decisions (create/list/view/edit/archive/supersede), search them, and view/edit them in the offline browser. Add one milestone-level integration test and update docs/packaging.md with the Forky-only supported release, procedures, checklist, and observed results. No new product behavior is added; work is limited to packaging scripts usage, documentation, and tests.

Full behavior, scope, verification matrix, assumptions, and open questions: docs/specifications/decision-records-milestone-packaging-and-verification.md.

Scope (in):
- Rebuild target/release/papyrus and papyrus_<version>_amd64.deb via scripts/build-deb.sh from current source (after cargo build --release).
- Verify package metadata: name papyrus, Architecture amd64, Version == Cargo.toml, GPL-3.0-or-later copyright, documented Depends.
- Install and uninstall the .deb in an isolated Debian Forky environment (host untouched).
- Smoke-test installed binary: --help, --version, init, task/milestone/release/decision lifecycles (create/list/view/edit/archive; supersede where documented), search over decision records, browser over all six categories (view/edit round-trip, API, assets, 320 px markup, no external refs), offline operation, browser config keys, embedded assets.
- Re-run cargo build --release, cargo test, cargo clippy --all-targets -- -D warnings, cargo fmt --check with Rust 1.99.1.
- Add tests/decision_records_smoke.rs covering decision lifecycle + search + browser end-to-end.
- Update docs/packaging.md (Forky-only supported release per ADR-0003, install/uninstall, smoke-test checklist and results).

Scope (out):
- No new product features, command behavior, or record types (TASK-17+).
- No verification on Debian Trixie or any release other than Forky (ADR-0003).
- No real-GUI browser rendering checks (markup/asset-level only).
- No edits to requirements files or business rules.
- No refresh of the runtime dependency-license audit (TASK-1/TASK-4 audit remains valid).
- No UI changes or new mockups.
- No decision restore command (BR-008 defines none).

Assumptions:
- Assumption: Rust 1.99.1 and dpkg-deb are available in the build environment (ADR-0001, ADR-0003).
- Assumption: an isolated Debian Forky environment (fakeroot dpkg --instdir/--admindir under ./.tmp) is available; the host is never modified.
- Assumption: Trixie verification is not performed; supported-release claim is narrowed to Forky (ADR-0003).
- Assumption: Cargo package version is the single source of truth for the .deb version and --version output.
- Assumption: TASK-1/TASK-4 runtime dependency-license audit remains valid; no new runtime dependencies introduced by TASK-13..TASK-15.
- Assumption: automated integration tests use the built binary (CARGO_BIN_EXE_papyrus) and temp directories.
- Assumption: decision records have no restore command (BR-008); archived decisions are not restored by the CLI.

Open questions (non-blocking, defaults in the specification):
- OQ-1: Real Firefox/Edge/Safari rendering is not verified in this headless task; default is markup/asset-level verification plus TASK-6/TASK-11/TASK-15 evidence.
- OQ-2: Container-level isolation is optional; default is fakeroot isolated dpkg install on the Forky build host.
- OQ-3: The milestone test lives in a new tests/decision_records_smoke.rs rather than extending an existing smoke file.
- OQ-4: Keep version 0.1.0.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The amd64 Debian package built from the current source installs on Debian Forky (amd64) - the only supported release per ADR-0003 and NFR-03.002 - and the installed binary provides decision creation, list, view, edit, archive, and supersession, search (including decision records), and browser access (all six categories including decision); Debian Trixie and any other release are not claimed or verified.
- [ ] #2 cargo build --release succeeds with the pinned Rust 1.99.1 toolchain and target/release/papyrus --help lists decision, task, milestone, release, search, and browser (guards against packaging a stale binary).
- [ ] #3 cargo test, cargo clippy --all-targets -- -D warnings, and cargo fmt --check pass with Rust 1.99.1, and the test counts are recorded.
- [ ] #4 scripts/build-deb.sh (unchanged) produces papyrus_<version>_amd64.deb with Version equal to the Cargo package version and Architecture: amd64.
- [ ] #5 dpkg-deb metadata reports Package: papyrus, Architecture: amd64, Version equal to Cargo.toml, Depends: libc6 (>= 2.34), and a copyright file declaring GPL-3.0-or-later.
- [ ] #6 The package contains only /usr/bin/papyrus (0755) and /usr/share/doc/papyrus/copyright, with no asset, template, or configuration files, and the installed binary serves /assets/app.css and /assets/app.js from assets compiled into the binary (no runtime asset file dependency).
- [ ] #7 In an isolated Debian Forky environment (fakeroot dpkg --instdir/--admindir under ./.tmp, or a Forky container/chroot/VM), dpkg -i succeeds, /usr/bin/papyrus exists as root:root 0755, papyrus resolves on PATH, and the host package database and filesystem are unchanged.
- [ ] #8 In the same isolated environment, dpkg -r (or dpkg --purge) papyrus removes the binary and papyrus no longer resolves on PATH.
- [ ] #9 The installed binary smoke test passes: papyrus --help exits 0 and lists decision, task, milestone, release, search, and browser, and papyrus --version exits 0 printing papyrus <version> matching the package version.
- [ ] #10 The installed binary initializes a project and exercises release, milestone, and task lifecycles (create/list/view/edit/archive/restore) and the decision lifecycle (create/list/view/edit/archive/supersede) with expected exit codes and persisted files, preserving IDs; disabled decision types are rejected.
- [ ] #11 Installed-binary search smoke tests pass over decision (and existing) records: a quoted phrase matches contiguously, --type decision restricts to decision records, --limit caps results, --json returns the {id,type,title,path} array (and [] when empty), archived decisions are hidden, and disabled decision types are excluded.
- [ ] #12 Installed-binary browser smoke tests pass with --no-open on a free port: the startup line is printed, all six categories render one at a time with the selected sidebar entry marked aria-current, a decision record view renders its Nygard fields and supersession, a decision edit round-trip returns 303 and persists through both the UI and the CLI, the JSON API responds (/api/decisions, /api/doc/<id>), both embedded assets are served, 320 px markup hooks are present, and no served page, stylesheet, or script references an external origin; archived decisions are not listed or reachable (404).
- [ ] #13 Offline operation is verified with no internet access (isolated network namespace, or the documented fallback evidence): search and the browser view/edit checks pass using only loopback and locally served assets.
- [ ] #14 Browser configuration keys work in the installed binary: config list/get/set/unset expose browser_host, browser_port, and browser_open, a configured port is used when --port is absent, --port overrides it, defaults are restored on unset, and invalid values exit 4 without changing the file.
- [ ] #15 A milestone-level integration test in tests/decision_records_smoke.rs exercises the decision lifecycle, search, and browser together end-to-end against the built binary, passes under cargo test, and does not duplicate the contract coverage in tests/decision_cli.rs, tests/search_backlog_cli.rs, or tests/browser_cli.rs.
- [ ] #16 docs/packaging.md states Debian Forky only (amd64), explicitly disclaims Trixie and other releases, and records the isolated install/uninstall procedure, the extended smoke-test checklist, and observed results with environment and date including timezone.
- [ ] #17 Verification evidence (environment, date/timezone, package file and size, quality-gate results with test counts, smoke-test outcomes, deviations) is recorded in the TASK-16 ticket notes.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 All acceptance criteria verified and recorded in the ticket notes and docs/packaging.md.
- [ ] #2 cargo build --release, cargo test, cargo clippy --all-targets -- -D warnings, and cargo fmt --check pass with Rust 1.99.1.
- [ ] #3 The .deb metadata and contents match the specification, and the package installs and uninstalls cleanly in an isolated Debian Forky environment with the host untouched.
- [ ] #4 The installed-binary smoke checklist passes: baseline CLI, init, task/milestone/release/decision lifecycles, search over decision records, browser (all six categories, view, edit round-trip, API, assets, 320 px markup, no external references), offline operation, config keys, and embedded-asset independence.
- [ ] #5 The new tests/decision_records_smoke.rs passes and the full existing test suite remains green.
- [ ] #6 docs/packaging.md is updated with the Forky-only claim, procedures, checklist, observed results (date with timezone), and any recorded limitation.
- [ ] #7 No requirements, business rules, ADRs, packaging scripts, or UI were changed; any product bug fix is recorded separately in the notes.
<!-- DOD:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
TASK-16 verification complete (2026-11-05 UTC). cargo build --release, cargo test (all pass incl. decision_records_smoke 3/3), clippy, fmt clean. build-deb.sh produced papyrus_0.1.0_amd64.deb. Package metadata verified: Package papyrus, Version 0.1.0, Architecture amd64, Depends libc6 (>= 2.34), contents only /usr/bin/papyrus + copyright. Isolated install/uninstall via fakeroot dpkg under ./.tmp/ succeeded; host untouched. Smoke tests: decision lifecycle, search decisions, browser decision category, offline loopback, browser config keys all pass. docs/packaging.md updated.
<!-- SECTION:NOTES:END -->
