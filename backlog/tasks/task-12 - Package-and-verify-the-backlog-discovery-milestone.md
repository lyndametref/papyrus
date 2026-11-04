---
id: TASK-12
title: Package and verify the backlog-discovery milestone
status: Implemented
assignee: []
created_date: '2026-10-31T22:26:00Z'
updated_date: '2026-11-04T21:20:00Z'
labels:
  - packaging
  - verification
milestone: m-2
dependencies:
  - TASK-10
  - TASK-11
references:
  - docs/requirements/non-functional/NFR-03.002.md
  - docs/requirements/non-functional/NFR-04.001.md
  - docs/requirements/non-functional/NFR-05.001.md
  - docs/requirements/non-functional/NFR-06.001.md
  - docs/requirements/non-functional/NFR-06.002.md
  - docs/business-rules/BR-005-task-command-behavior.md
  - docs/business-rules/BR-006-milestone-command-behavior.md
  - docs/business-rules/BR-007-release-command-behavior.md
  - docs/business-rules/BR-004-browser-command-behavior.md
  - >-
    docs/specifications/backlog-discovery-milestone-packaging-and-verification.md
documentation:
  - >-
    docs/specifications/backlog-discovery-milestone-packaging-and-verification.md
type: chore
ordinal: 12000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Package and verify the backlog-discovery milestone (m-2): rebuild the amd64 .deb from the post-TASK-11 source, verify its metadata and GPL-3.0-or-later licensing, install and uninstall it in an isolated Debian Forky environment, and run smoke tests proving the packaged binary can manage tasks/milestones/releases, search them, and view/edit them in the offline browser. Add one milestone-level integration test and update docs/packaging.md with the Forky-only supported release, procedures, checklist, and observed results. No new product behavior is added; work is limited to packaging scripts usage, documentation, and tests.

Full behavior, scope, verification matrix, assumptions, and open questions: docs/specifications/backlog-discovery-milestone-packaging-and-verification.md.

Scope (in):
- Rebuild target/release/papyrus and papyrus_<version>_amd64.deb via scripts/build-deb.sh from current source (after cargo build --release).
- Verify package metadata: name papyrus, Architecture amd64, Version == Cargo.toml, GPL-3.0-or-later copyright, documented Depends.
- Install and uninstall the .deb in an isolated Debian Forky environment (host untouched).
- Smoke-test installed binary: --help, --version, init, task/milestone/release lifecycles (create/list/view/edit/archive/restore where documented), search over task/milestone/release records, browser over all five categories (view/edit round-trip, API, assets, 320 px markup, no external refs), offline operation, browser config keys, embedded assets.
- Re-run cargo build --release, cargo test, cargo clippy --all-targets -- -D warnings, cargo fmt --check with Rust 1.99.1.
- Add tests/backlog_discovery_smoke.rs covering task/milestone/release search + browser end-to-end.
- Update docs/packaging.md (Forky-only supported release per ADR-0003, install/uninstall, smoke-test checklist and results).

Scope (out):
- No new product features, command behavior, or record types (TASK-13+).
- No verification on Debian Trixie or any release other than Forky (ADR-0003).
- No real-GUI browser rendering checks (markup/asset-level only).
- No edits to requirements files or business rules.
- No refresh of the runtime dependency-license audit (TASK-1/TASK-4 audit remains valid).
- No UI changes or new mockups.

Assumptions:
- Assumption: Rust 1.99.1 and dpkg-deb are available in the build environment (ADR-0001, ADR-0003).
- Assumption: an isolated Debian Forky environment (fakeroot dpkg --instdir/--admindir under ./.tmp) is available; the host is never modified.
- Assumption: Trixie verification is not performed; supported-release claim is narrowed to Forky (ADR-0003).
- Assumption: Cargo package version is the single source of truth for the .deb version and --version output.
- Assumption: TASK-1/TASK-4 runtime dependency-license audit remains valid; no new runtime dependencies introduced by TASK-8..TASK-11.
- Assumption: automated integration tests use the built binary (CARGO_BIN_EXE_papyrus) and temp directories.
- Assumption: --editor smoke testing is environment-dependent and verified by review rather than automated test.

Open questions (non-blocking, defaults in the specification):
- OQ-1: Real Firefox/Edge/Safari rendering is not verified in this headless task; default is markup/asset-level verification plus TASK-6/TASK-11 evidence.
- OQ-2: Container-level isolation is optional; default is fakeroot isolated dpkg install on the Forky build host.
- OQ-3: The milestone test lives in a new tests/backlog_discovery_smoke.rs rather than extending tests/milestone_smoke.rs.
- OQ-4: Keep version 0.1.0.

Dependencies: TASK-10 and TASK-11 (Done), and transitively TASK-8, TASK-9, and the whole m-1 chain TASK-1..TASK-7.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The amd64 Debian package built from the current source installs on Debian Forky (amd64) - the only supported release per ADR-0003 and NFR-03.002 - and the installed binary provides task, milestone, release, search, and browser (all five categories), including the backlog categories; Debian Trixie and any other release are not claimed or verified.
- [x] #2 cargo build --release succeeds with the pinned Rust 1.99.1 toolchain and target/release/papyrus --help lists task, milestone, release, search, and browser (guards against packaging a stale binary).
- [x] #3 cargo test, cargo clippy --all-targets -- -D warnings, and cargo fmt --check pass with Rust 1.99.1, and the test counts are recorded.
- [x] #4 scripts/build-deb.sh (unchanged) produces papyrus_<version>_amd64.deb with Version equal to the Cargo package version and Architecture: amd64.
- [x] #5 dpkg-deb metadata reports Package: papyrus, Architecture: amd64, Version equal to Cargo.toml, Depends: libc6 (>= 2.34), and a copyright file declaring GPL-3.0-or-later.
- [x] #6 The package contains only /usr/bin/papyrus (0755) and /usr/share/doc/papyrus/copyright, with no asset, template, or configuration files, and the installed binary serves /assets/app.css and /assets/app.js from assets compiled into the binary (no runtime asset file dependency).
- [x] #7 In an isolated Debian Forky environment (fakeroot dpkg --instdir/--admindir under ./.tmp, or a Forky container/chroot/VM), dpkg -i succeeds, /usr/bin/papyrus exists as root:root 0755, papyrus resolves on PATH, and the host package database and filesystem are unchanged.
- [x] #8 In the same isolated environment, dpkg -r (or dpkg --purge) papyrus removes the binary and papyrus no longer resolves on PATH.
- [x] #9 The installed binary smoke test passes: papyrus --help exits 0 and lists task, milestone, release, search, and browser, and papyrus --version exits 0 printing papyrus <version> matching the package version.
- [x] #10 The installed binary initializes a project and exercises release create/list/view/edit, milestone create/list/view/edit (with release assignment), and task create/list/view/edit/archive/restore with expected exit codes and persisted files, preserving IDs across archive/restore.
- [x] #11 Installed-binary search smoke tests pass over task, milestone, and release records: a quoted phrase matches contiguously, --type task|milestone|release restricts to that type, --limit caps results, --json returns the {id,type,title,path} array (and [] when empty), and an archived record is hidden until restored.
- [x] #12 Installed-binary browser smoke tests pass with --no-open on a free port: the startup line is printed, all five categories render one at a time with the selected sidebar entry marked aria-current, a task/milestone/release record view renders, an edit round-trip returns 303 and persists through both the UI and the CLI, the JSON API responds, both embedded assets are served, 320 px markup hooks are present, and no served page, stylesheet, or script references an external origin.
- [x] #13 Offline operation is verified with no internet access (isolated network namespace, or the documented fallback evidence): search and the browser view/edit checks pass using only loopback and locally served assets.
- [x] #14 Browser configuration keys work in the installed binary: config list/get/set/unset expose browser_host, browser_port, and browser_open, a configured port is used when --port is absent, --port overrides it, defaults are restored on unset, and invalid values exit 4 without changing the file.
- [x] #15 A milestone-level integration test in tests/backlog_discovery_smoke.rs exercises task/milestone/release search and browser together end-to-end against the built binary, passes under cargo test, and does not duplicate the contract coverage in tests/browser_cli.rs or tests/search_backlog_cli.rs.
- [x] #16 docs/packaging.md states Debian Forky only (amd64), explicitly disclaims Trixie and other releases, and records the isolated install/uninstall procedure, the extended smoke-test checklist, and observed results with environment and date including timezone.
- [x] #17 Verification evidence (environment, date/timezone, package file and size, quality-gate results with test counts, smoke-test outcomes, deviations) is recorded in the TASK-12 ticket notes.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 All acceptance criteria verified and recorded in the ticket notes and docs/packaging.md.
- [x] #2 cargo build --release, cargo test, cargo clippy --all-targets -- -D warnings, and cargo fmt --check pass with Rust 1.99.1.
- [x] #3 The .deb metadata and contents match the specification, and the package installs and uninstalls cleanly in an isolated Debian Forky environment with the host untouched.
- [x] #4 The installed-binary smoke checklist passes: baseline CLI, init, task/milestone/release lifecycles, search over task/milestone/release records, browser (all five categories, view, edit round-trip, API, assets, 320 px markup, no external references), offline operation, config keys, and embedded-asset independence.
- [x] #5 The new tests/backlog_discovery_smoke.rs passes and the full existing test suite remains green.
- [x] #6 docs/packaging.md is updated with the Forky-only claim, procedures, checklist, observed results (date with timezone), and any recorded limitation.
- [x] #7 No requirements, business rules, ADRs, packaging scripts, or UI were changed; any product bug fix is recorded separately in the notes.
<!-- DOD:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Preflight: confirm Debian Forky amd64, rustc/cargo 1.99.1 via rust-toolchain.toml, dpkg-deb, fakeroot, and unshare -rn availability; record versions and date (UTC) for the evidence log.
2. Quality gates: cargo build --release, cargo test (record counts), cargo clippy --all-targets -- -D warnings, cargo fmt --check. Confirm target/release/papyrus --help lists task, milestone, release, search, and browser.
3. Package: run scripts/build-deb.sh unchanged; verify dpkg-deb -I (Package, Version == Cargo.toml, Architecture amd64, Depends libc6 (>= 2.34), copyright GPL-3.0-or-later) and dpkg-deb -c (only /usr/bin/papyrus 0755 plus copyright; no asset files). Record file name and size.
4. Isolated install: create ./.tmp/inst and ./.tmp/adm, initialize an empty dpkg database, install with fakeroot dpkg --instdir/--admindir -i; assert /usr/bin/papyrus root:root 0755, PATH resolution, and that host dpkg -l papyrus is unchanged. Verify dpkg -r and --purge remove the binary and admin entry, then remove ./.tmp scratch.
5. Installed-binary smoke: run sections 3.1-3.6 of the specification against the installed binary in a ./.tmp project (help/version, init, task/milestone/release lifecycles, search quoted phrase/--type/--limit/--json/archival, browser --no-open on a free port, all five categories, view, edit round-trip, JSON API, assets, 320 px markup, no external references).
6. Offline check: repeat the search and browser smoke inside unshare -rn with lo up, first proving an outbound TCP connection fails; record the fallback evidence if the namespace is unavailable.
7. Config keys: config list/get/set/unset for browser_host/browser_port/browser_open, configured port honored without --port, --port override, defaults restored, invalid values exit 4.
8. Embedded assets: run the installed binary from an empty working directory against a project with no asset files; assert search and browser still work and assets are non-empty.
9. Tests: add tests/backlog_discovery_smoke.rs with the consolidated task/milestone/release search+browser flow and the no-runtime-asset-dependency test, using CARGO_BIN_EXE_papyrus, temp project dirs, a free loopback port, startup-line wait, and child kill on drop; avoid duplicating tests/browser_cli.rs or tests/search_backlog_cli.rs contract checks. Run cargo test and cargo fmt/clippy again.
10. Documentation: update docs/packaging.md with the Forky-only supported release and Trixie disclaimer, prerequisites, build/package commands, isolated install/uninstall using ./.tmp, the extended smoke-test checklist, and an observed-results table (environment, date/timezone, pass/fail, limitations).
11. Evidence and handoff: record environment, date/timezone, package name and size, gate results with test counts, smoke outcomes, and deviations in the ticket notes; report any bug fix separately.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
TASK-12 packaging and verification evidence (m-2).

Environment: Debian GNU/Linux forky/sid amd64, glibc 2.43, dpkg 1.23.11, rustc/cargo 1.99.1, clippy 0.1.99, fakeroot, unshare -rn available (loopback via `ip link set lo up`).
Date: 2026-11-04 (UTC).

Package: papyrus_0.1.0_amd64.deb, 735840 bytes, SHA-256 69a2152470fbec6a1f110fabad28ee2b5c40229a02c61bac7bd6bb49f422d3db.
Metadata: Package: papyrus, Version: 0.1.0, Architecture: amd64, Depends: libc6 (>= 2.34), Section: utils, Priority: optional, copyright GPL-3.0-or-later.
Contents: /usr/bin/papyrus (0755, 2112928 bytes) and /usr/share/doc/papyrus/copyright (0644, 865 bytes) only; no asset/template/config/man files; assets served from the binary.

Quality gates (Rust 1.99.1): cargo build --release PASS; cargo test PASS (160 tests); cargo clippy --all-targets -- -D warnings PASS (0 warnings); cargo fmt --check PASS. target/release/papyrus --help lists task, milestone, release, search, browser.

Isolated install/uninstall: fakeroot dpkg --instdir=./.tmp/inst --admindir=./.tmp/adm --force-depends; dpkg -i PASS, /usr/bin/papyrus root:root 0755, resolves on PATH, papyrus --version prints papyrus 0.1.0. dpkg -r and --purge PASS; binary removed and no longer on PATH. Host package status unchanged (host already reported papyrus 0.1.0 installed from prior work; not modified by this run).

Installed-binary smoke: baseline help/version PASS; init PASS; release/milestone/task lifecycles (create/list/view/edit/archive/restore) PASS with IDs preserved; search quoted phrase/--type task|milestone|release/--limit/--json/[]/archival visibility PASS; browser startup line, all five categories, aria-current, record views, task edit round-trip 303 + persisted UI/CLI, JSON API (/api/tasks, /api/milestones, /api/releases, /api/doc/<id>), embedded assets (non-empty, no-store), 320 px hooks, no external references, CSP present PASS; offline via unshare -rn (outbound TCP blocked) search over loopback PASS; config list/get/set/unset (browser_host, browser_port, browser_open), configured port honored, --port override, defaults restored, invalid values exit 4 with unchanged file PASS; embedded assets from empty working dir and asset-free project PASS.

New test: tests/backlog_discovery_smoke.rs (milestone_backlog_discovery_flow, installed_layout_has_no_runtime_asset_dependency), 2 tests, passing; no duplication of tests/browser_cli.rs or tests/search_backlog_cli.rs contract coverage.

Documentation: docs/packaging.md updated with Forky-only claim, procedures, extended checklist, observed results (environment, date/timezone), and limitations.

Limitations/deviations: OQ-1 no real-GUI browser run (markup/asset level only). Forky only; no Trixie verification. Host baseline already had papyrus installed (unchanged, not absent). scripts/build-deb.sh unchanged. No requirements/BR/ADRs/UI changed; no product code changes.
<!-- SECTION:NOTES:END -->
