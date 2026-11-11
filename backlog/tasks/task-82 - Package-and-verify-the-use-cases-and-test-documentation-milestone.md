---
id: TASK-82
title: Package and verify the use cases and test documentation milestone
status: Refined
assignee: []
created_date: '2026-11-10T20:16:00Z'
updated_date: '2026-11-11T08:45:00Z'
labels:
  - packaging
milestone: m-15
dependencies:
  - TASK-78
  - TASK-80
  - TASK-81
references:
  - docs/requirements/business/REQ-05.012.md
  - docs/requirements/business/REQ-05.013.md
  - docs/requirements/business/REQ-05.014.md
  - docs/requirements/business/REQ-05.007.md
  - docs/requirements/business/REQ-06.001.md
  - docs/requirements/business/REQ-06.002.md
  - docs/business-rules/BR-018-use-case-document-type-behavior.md
  - docs/business-rules/BR-019-test-case-document-type-behavior.md
  - docs/business-rules/BR-020-test-report-document-type-behavior.md
documentation:
  - >-
    docs/specifications/use-cases-and-test-documentation-milestone-packaging-and-verification.md
  - docs/packaging.md
priority: medium
type: chore
ordinal: 35400
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Package and verify the m-15 milestone (use cases and test documentation) in an installable increment: rebuild the amd64 Debian package from the post-m-15 source, verify metadata and GPL-3.0-or-later licensing, install/uninstall in an isolated Debian Forky environment, and run smoke tests proving the packaged binary manages use cases, test cases, and test reports (create/list/view/edit/deprecate/restore with IDs preserved), searches them, and serves the offline local browser categories end to end. Add one milestone-level integration test and update docs/packaging.md. No new product behavior.

Full behavior, scope, verification matrix, assumptions, and open questions: docs/specifications/use-cases-and-test-documentation-milestone-packaging-and-verification.md (TASK-82).

Dependencies: TASK-78 (management), TASK-80 (search), TASK-81 (browser), TASK-79 (design review; MOCK-0013 approved).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The amd64 Debian package built from the current source installs on Debian Forky (amd64) - the only supported release per ADR-0003 and NFR-03.002 - and the installed binary provides use-case/test-case/test-report management, search, and browser access over all three categories; Debian Trixie and other releases are not claimed or verified.
- [x] #2 cargo build --release succeeds with the pinned Rust 1.99.1 toolchain and target/release/papyrus --help lists use-case, test-case, test-report, search, and browser, guarding against packaging a stale binary.
- [x] #3 cargo test, cargo clippy --all-targets -- -D warnings, and cargo fmt --check pass with Rust 1.99.1, and the test counts are recorded.
- [x] #4 scripts/build-deb.sh (unchanged) produces papyrus_<version>_amd64.deb with Version equal to the Cargo package version and Architecture: amd64.
- [x] #5 dpkg-deb metadata reports Package: papyrus, Architecture: amd64, Version equal to Cargo.toml, Depends: libc6 (>= 2.34), and a copyright file declaring GPL-3.0-or-later.
- [x] #6 The package contains only /usr/bin/papyrus (0755) and /usr/share/doc/papyrus/copyright, with no asset, template, configuration, or man files, and the installed binary serves /assets/app.css and /assets/app.js from assets compiled into the binary.
- [x] #7 In an isolated Debian Forky environment, dpkg -i succeeds, /usr/bin/papyrus exists as root:root 0755, papyrus resolves on PATH, and the host package database and filesystem are unchanged.
- [x] #8 In the same isolated environment, dpkg -r (or dpkg --purge) papyrus removes the binary and papyrus no longer resolves on PATH.
- [x] #9 The installed binary smoke test passes: papyrus --help exits 0 listing the full command set, papyrus --version exits 0 printing papyrus <version> matching the package version, and init plus the interactive walkthrough complete.
- [x] #10 The installed binary initializes a project and exercises the lifecycle of all three record families (use cases, test cases, test reports) with expected exit codes and persisted files, preserving IDs across deprecate/restore.
- [x] #11 Installed-binary unified search passes across all three types: quoted phrase matches contiguously, split words do not match, --type doc includes all three, other --type filters exclude them, --limit caps the combined set, --json returns the {id,type,title,path} array (and [] when empty), deprecated and disabled/absent types are hidden or contribute no results or error, and search performs no writes.
- [x] #12 Installed-binary browser smoke tests pass with --no-open on a free port: the startup line is printed, the three categories render with sidebar entries after Technical Specifications and aria-current, lists show count/rows/preview/metadata, record views show metadata block, title h1, description body without a Description heading, applicability section when present, single Edit action, an edit round-trip returns 303 and persists through both the UI and the CLI, field-preserving writes leave untouched sections byte-identical, the JSON API responds (/api/use-cases, /api/use-case, /api/test-cases, /api/test-case, /api/test-reports, /api/test-report, /api/doc/<id>), both embedded assets are served, 320 px markup hooks are present, and no served page/stylesheet/script references an external origin.
- [x] #13 Deprecated records of all three types are hidden consistently across list, search, and browser; after restore, they reappear in all three surfaces with the same ID and unchanged content, and return 404 on view, edit, POST, and API with the identical 404 page and recovery link.
- [x] #14 Offline operation is verified with no internet access (isolated network namespace, or documented fallback): search and browser list/view/edit checks pass using only loopback and locally served assets.
- [x] #15 Browser configuration keys work in the installed binary: config list/get/set/unset expose browser_host, browser_port, and browser_open, a configured port is used when --port is absent, --port overrides it, defaults are restored on unset, and invalid values exit 4 without changing the file.
- [x] #16 A milestone-level integration test in tests/use_cases_test_cases_test_reports_smoke.rs exercises the three-type lifecycle, search, and browser together end-to-end against the built binary, passes under cargo test, and does not duplicate the contract coverage in the CLI/search/browser contract tests.
- [x] #17 docs/packaging.md states Debian Forky only (amd64), explicitly disclaims Trixie and other releases, and records the isolated install/uninstall procedure, the extended smoke-test checklist, and observed results with environment and date including timezone.
- [x] #18 Verification evidence (environment, date/timezone, package file and size, quality-gate results with test counts, smoke-test outcomes, deviations) is recorded in the TASK-82 ticket notes.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 All acceptance criteria verified and recorded in the ticket notes and docs/packaging.md.
- [x] #2 cargo build --release, cargo test, cargo clippy --all-targets -- -D warnings, and cargo fmt --check pass with Rust 1.99.1.
- [x] #3 The .deb metadata and contents match the specification, and the package installs and uninstalls cleanly in an isolated Debian Forky environment with the host untouched.
- [x] #4 The installed-binary smoke checklist passes (all three record families, unified search, browser inventory and cross-type resolution, field-preserving edits, metadata projection, offline operation, config keys, embedded assets).
- [x] #5 The new tests/use_cases_test_cases_test_reports_smoke.rs passes and the full existing test suite remains green.
- [x] #6 docs/packaging.md is updated with the Forky-only claim, procedures, checklist, observed results (date with timezone), and any recorded limitation.
- [x] #7 No requirements, business rules, ADRs, packaging scripts, or UI were changed; any product bug fix is recorded separately in the notes.
<!-- DOD:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Preflight: confirm Debian Forky amd64, rustc/cargo 1.99.1 via rust-toolchain.toml, dpkg-deb, fakeroot, unshare -rn; record versions and date (UTC).
2. Quality gates: cargo build --release, cargo test (record counts), cargo clippy --all-targets -- -D warnings, cargo fmt --check. Confirm target/release/papyrus --help lists use-case, test-case, test-report, search, browser.
3. Package: run scripts/build-deb.sh unchanged; verify dpkg-deb -I (Package, Version == Cargo.toml, Architecture amd64, Depends libc6 (>= 2.34), copyright GPL-3.0-or-later) and dpkg-deb -c (only /usr/bin/papyrus 0755 plus copyright). Record file name and size.
4. Isolated install: create ./.tmp/inst and ./.tmp/adm, initialize an empty dpkg database, install with fakeroot dpkg --instdir/--admindir -i; assert /usr/bin/papyrus root:root 0755, PATH resolution, host unchanged. Verify dpkg -r and --purge remove the binary and admin entry, then remove scratch.
5. Installed-binary smoke: run sections 3.1-3.6 of the specification against the installed binary in a ./.tmp project (baseline, init/walkthrough, all three record families, unified search, browser inventory/sidebar/cross-type/edit/API/assets/320px/no external refs, metadata projection, offline over loopback, config keys, embedded assets).
6. Offline check: repeat search and browser smoke inside unshare -rn with lo up, first proving an outbound TCP connection fails; record fallback evidence if unavailable.
7. Config keys: config list/get/set/unset for browser_host/browser_port/browser_open, configured port honored without --port, --port override, defaults restored, invalid values exit 4.
8. Embedded assets: run installed binary from empty working directory against asset-free project; assert search and browser work and assets non-empty.
9. Tests: add tests/use_cases_test_cases_test_reports_smoke.rs with the consolidated cross-type flow and no-runtime-asset-dependency test, using CARGO_BIN_EXE_papyrus, temp project dirs, free loopback port, startup-line wait, child kill on drop; avoid duplicating per-type contract coverage. Run cargo test and cargo fmt/clippy again.
10. Documentation: update docs/packaging.md with Forky-only claim, procedures, extended checklist, observed results (environment, date/timezone), limitations.
11. Evidence and handoff: record environment, date/timezone, package name and size, gate results with test counts, smoke outcomes, deviations in ticket notes.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
TASK-82 verification evidence (2026-11-11 08:45:12 UTC; local CET 09:45:12).

Environment: Debian GNU/Linux forky/sid amd64, glibc 2.43, dpkg 1.23.11, rustc/cargo 1.99.1, clippy 0.1.99, rustfmt 1.99.1, fakeroot 2.1.4, unshare/ip available.

Package: papyrus_0.1.0_amd64.deb, 1136804 bytes, SHA-256 b5f01d4e33fe100232b39bbf2adfc1c4f2d5767cd64adeec4c11771fa2051592. Metadata: Package papyrus, Version 0.1.0, Architecture amd64, Depends libc6 (>= 2.34), copyright GPL-3.0-or-later. Contents: /usr/bin/papyrus (0755, 3817808 bytes) and /usr/share/doc/papyrus/copyright only.

Quality gates: cargo build --release PASS; cargo test PASS (565 tests); cargo clippy --all-targets -- -D warnings PASS (0 warnings); cargo fmt --check PASS. target/release/papyrus --help lists use-case, test-case, test-report, search, browser.

Isolated install/uninstall: fakeroot dpkg --instdir/--admindir under ./.tmp; install exit 0; binary root:root 0755; PATH resolution OK; remove/purge OK; host papyrus remained installed and unchanged.

Smoke: use-case/test-case/test-report lifecycle create/list/view/edit/deprecate/restore with IDs preserved; search quoted phrase, split words, --type doc includes all three, other --type filters exclude, --limit, --json, empty [], deprecated hidden until restored; --type use-case/test-case/test-report exit 2. Browser startup line, three categories with sidebar order and aria-current, list/view/edit round-trip 303, field-preserving write, JSON API aliases, embedded assets, 320px hooks, no external refs, deprecated 404 on view/edit/POST/API with identical page and recovery link. Offline via unshare -rn: outbound TCP failed; search/browser/assets passed over loopback. Config keys browser_host/browser_port/browser_open list/get/set/unset, configured port honored, --port override, defaults restored, invalid values exit 4 with unchanged file. Embedded assets work from empty working directory and asset-free project.

Tests: tests/use_cases_test_cases_test_reports_smoke.rs (2 tests) added and passing.

Documentation: docs/packaging.md updated with TASK-82 procedures, checklist, observed results, limitations.

Deviations: none. No product code changes. No requirements/BRs/ADRs/packaging scripts/UI changed.
<!-- SECTION:NOTES:END -->
