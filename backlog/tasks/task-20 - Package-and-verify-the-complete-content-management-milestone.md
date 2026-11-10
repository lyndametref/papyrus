---
id: TASK-20
title: Package and verify the complete content-management milestone
status: Done
assignee: []
created_date: '2026-10-31T22:26:00Z'
updated_date: '2026-11-10T19:37:00Z'
labels:
  - packaging
milestone: m-12
dependencies:
  - TASK-18
  - TASK-19
  - TASK-59
  - TASK-60
  - TASK-69
  - TASK-70
  - TASK-71
  - TASK-72
  - TASK-74
  - TASK-75
  - TASK-77
references:
  - docs/requirements/business/REQ-01.md
  - docs/requirements/business/REQ-02.md
  - docs/requirements/business/REQ-03.md
  - docs/requirements/business/REQ-04.md
  - docs/requirements/business/REQ-05.md
  - docs/requirements/business/REQ-06.md
  - docs/requirements/non-functional/NFR-01.md
  - docs/requirements/non-functional/NFR-02.md
  - docs/requirements/non-functional/NFR-03.md
  - docs/requirements/non-functional/NFR-04.md
  - docs/requirements/non-functional/NFR-05.md
  - docs/requirements/non-functional/NFR-06.md
  - docs/business-rules/BR-011-glossary-command-behavior.md
  - docs/business-rules/BR-012-ui-spec-command-behavior.md
  - docs/business-rules/BR-013-rule-command-behavior.md
  - docs/business-rules/BR-014-audit-command-behavior.md
  - docs/business-rules/BR-004-browser-command-behavior.md
  - docs/business-rules/BR-009-req-command-behavior.md
  - docs/business-rules/BR-010-nfreq-command-behavior.md
  - docs/requirements/business/REQ-05.010.md
  - docs/requirements/non-functional/NFR-04.003.md
  - docs/requirements/business/REQ-01.006.md
  - docs/requirements/business/REQ-01.007.md
  - docs/requirements/business/REQ-06.005.md
  - >-
    docs/specifications/complete-content-management-milestone-packaging-and-verification.md
documentation:
  - >-
    docs/specifications/complete-content-management-milestone-packaging-and-verification.md
ordinal: 30500
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Package and verify the complete Papyrus content-management milestone (m-12): rebuild the amd64 Debian package from the post-m-12 source, verify its metadata and GPL-3.0-or-later licensing, install and uninstall it in an isolated Debian Forky environment, and run smoke tests proving the packaged binary provides every supported record family, unified search across all record types, and the offline browser over all Papyrus content (inventory, cross-type ID resolution, metadata projection, managed-artifact and skills pages, init walkthrough, ID padding, migration, Backlog.md round-trip). Add one milestone-level integration test and update docs/packaging.md. No new product behavior is added; work is limited to packaging scripts usage, documentation, and tests.

Full behavior, scope, verification matrix, assumptions, and open questions: docs/specifications/complete-content-management-milestone-packaging-and-verification.md.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The amd64 Debian package built from the current source installs on Debian Forky (amd64) - the only supported release per ADR-0003 and NFR-03.002 - and the installed binary provides all implemented Papyrus commands, unified search across all record types, and browser access over all Papyrus content; Debian Trixie and other releases are not claimed or verified.
- [ ] #2 cargo build --release succeeds with the pinned Rust 1.99.1 toolchain and target/release/papyrus --help lists the full command set (init, config, task, milestone, release, req, nfr, decision, glossary, ui-spec, ui-mockup, rule, audit, tech-spec, search, browser), guarding against packaging a stale binary.
- [ ] #3 cargo test, cargo clippy --all-targets -- -D warnings, and cargo fmt --check pass with Rust 1.99.1, and the test counts are recorded.
- [ ] #4 scripts/build-deb.sh (unchanged) produces papyrus_<version>_amd64.deb with Version equal to the Cargo package version and Architecture: amd64.
- [ ] #5 dpkg-deb metadata reports Package: papyrus, Architecture: amd64, Version equal to Cargo.toml, Depends: libc6 (>= 2.34), and a copyright file declaring GPL-3.0-or-later.
- [ ] #6 The package contains only /usr/bin/papyrus (0755) and /usr/share/doc/papyrus/copyright, with no asset, template, configuration, or man files, and the installed binary serves /assets/app.css and /assets/app.js from assets compiled into the binary.
- [ ] #7 In an isolated Debian Forky environment, dpkg -i succeeds, /usr/bin/papyrus exists as root:root 0755, papyrus resolves on PATH, and the host package database and filesystem are unchanged.
- [ ] #8 In the same isolated environment, dpkg -r (or dpkg --purge) papyrus removes the binary and papyrus no longer resolves on PATH.
- [ ] #9 The installed binary smoke test passes: papyrus --help exits 0 listing the full command set, papyrus --version exits 0 printing papyrus <version> matching the package version, and init plus the interactive walkthrough complete.
- [ ] #10 The installed binary initializes a project and exercises the lifecycle of all twelve record families (business requirements, non-functional requirements, tasks, milestones, releases, decisions, glossary terms, UI specifications, UI mockups, business rules, audit records, technical specifications) with expected exit codes and persisted files, preserving IDs across deprecate/restore and archive/restore.
- [ ] #11 ID padding is constrained to 1-6 (invalid values exit 4 without changing the file) and Backlog.md round-trip interoperability holds in the installed binary with no drift.
- [ ] #12 Installed-binary unified search passes across all record types: quoted phrase matches contiguously, --type doc aggregates document families while task/milestone/release/decision restrict to their store, --limit caps the combined set, --json returns the {id,type,title,path} array (and [] when empty), deprecated and archived records are hidden until restored, disabled/absent types contribute no results or error, and search performs no writes.
- [ ] #13 Installed-binary browser smoke tests pass with --no-open on a free port: the startup line is printed, the inventory hub renders with one card per supported type and correct counts, the sidebar lists entries in final order with aria-current, per-type lists and record views render, an edit round-trip returns 303 and persists through both the UI and the CLI, the JSON API responds, both embedded assets are served, 320 px markup hooks are present, and no served page/stylesheet/script references an external origin.
- [ ] #14 Cross-type ID resolution behaves correctly: an ID in exactly one category resolves, an ID in more than one category returns 500, and deprecated and unknown IDs return 404; disabled types contribute no records and no error; empty states render for missing stores.
- [ ] #15 Front-matter metadata projection (REQ-06.004) holds in the installed binary: list and detail views read id/title/created_date/updated_date/labels/references and type-specific keys from YAML front matter, created_date and updated_date are read-only in the browser, and a save writes changed values, refreshes updated_date, and preserves all other keys and the file body.
- [ ] #16 The managed artifacts and lifecycles page (/lifecycle) and the agent skills and workflows page (/skills) render with the shared shell and sidebar in final order.
- [ ] #17 Lazy front-matter migration and Backlog.md round-trip hold in the installed binary: migrated records remain searchable and viewable, unsafe migrations leave the file untouched, and Backlog.md-edited task/milestone files are read and written without drift.
- [ ] #18 Offline operation is verified with no internet access (isolated network namespace, or documented fallback): search and browser list/view/edit checks pass using only loopback and locally served assets.
- [ ] #19 Browser configuration keys work in the installed binary: config list/get/set/unset expose browser_host, browser_port, and browser_open, a configured port is used when --port is absent, --port overrides it, defaults are restored on unset, and invalid values exit 4 without changing the file.
- [ ] #20 A milestone-level integration test in tests/content_management_smoke.rs exercises the cross-type flow end-to-end against the built binary, passes under cargo test, and does not duplicate the contract coverage in tests/*_cli.rs or tests/*_smoke.rs.
- [ ] #21 docs/packaging.md states Debian Forky only (amd64), explicitly disclaims Trixie and other releases, and records the isolated install/uninstall procedure, the extended smoke-test checklist, and observed results with environment and date including timezone.
- [ ] #22 Verification evidence (environment, date/timezone, package file and size, quality-gate results with test counts, smoke-test outcomes, deviations) is recorded in the TASK-20 ticket notes.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 All acceptance criteria verified and recorded in the ticket notes and docs/packaging.md.
- [ ] #2 cargo build --release, cargo test, cargo clippy --all-targets -- -D warnings, and cargo fmt --check pass with Rust 1.99.1.
- [ ] #3 The .deb metadata and contents match the specification, and the package installs and uninstalls cleanly in an isolated Debian Forky environment with the host untouched.
- [ ] #4 The installed-binary smoke checklist passes (all record families, unified search, browser inventory and cross-type resolution, metadata projection, managed artifacts and skills pages, migration and round-trip, offline operation, config keys, embedded assets).
- [ ] #5 The new tests/content_management_smoke.rs passes and the full existing test suite remains green.
- [ ] #6 docs/packaging.md is updated with the Forky-only claim, procedures, checklist, observed results (date with timezone), and any recorded limitation.
- [ ] #7 No requirements, business rules, ADRs, packaging scripts, or UI were changed; any product bug fix is recorded separately in the notes.
<!-- DOD:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Preflight: confirm Debian Forky amd64, rustc/cargo 1.99.1 via rust-toolchain.toml, dpkg-deb, fakeroot, unshare -rn; record versions and date (UTC).
2. Quality gates: cargo build --release, cargo test (record counts), cargo clippy --all-targets -- -D warnings, cargo fmt --check. Confirm target/release/papyrus --help lists the full command set.
3. Package: run scripts/build-deb.sh unchanged; verify dpkg-deb -I (Package, Version == Cargo.toml, Architecture amd64, Depends libc6 (>= 2.34), copyright GPL-3.0-or-later) and dpkg-deb -c (only /usr/bin/papyrus 0755 plus copyright). Record file name and size.
4. Isolated install: create ./.tmp/inst and ./.tmp/adm, initialize an empty dpkg database, install with fakeroot dpkg --instdir/--admindir -i; assert /usr/bin/papyrus root:root 0755, PATH resolution, host unchanged. Verify dpkg -r and --purge remove the binary and admin entry, then remove scratch.
5. Installed-binary smoke: run sections 3.1-3.8 of the specification against the installed binary in a ./.tmp project (baseline, init/walkthrough, all twelve record families, ID padding, unified search, browser inventory/sidebar/cross-type/edit/API/assets/320px/no external refs, metadata projection, managed artifacts and skills pages, migration and round-trip).
6. Offline check: repeat search and browser smoke inside unshare -rn with lo up, first proving an outbound TCP connection fails; record fallback evidence if unavailable.
7. Config keys: config list/get/set/unset for browser_host/browser_port/browser_open, configured port honored without --port, --port override, defaults restored, invalid values exit 4.
8. Embedded assets: run installed binary from empty working directory against asset-free project; assert search and browser work and assets non-empty.
9. Tests: add tests/content_management_smoke.rs with the consolidated cross-type flow and no-runtime-asset-dependency test, using CARGO_BIN_EXE_papyrus, temp project dirs, free loopback port, startup-line wait, child kill on drop; avoid duplicating per-type contract coverage. Run cargo test and cargo fmt/clippy again.
10. Documentation: update docs/packaging.md with Forky-only claim, procedures, extended checklist, observed results (environment, date/timezone), limitations.
11. Evidence and handoff: record environment, date/timezone, package name and size, gate results with test counts, smoke outcomes, deviations in ticket notes.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Verification evidence (2026-11-10 UTC): rustc/cargo 1.99.1; cargo build --release PASS; cargo test 506 passed; cargo clippy --all-targets -- -D warnings 0 warnings; cargo fmt --check PASS. Packaging: ./scripts/build-deb.sh -> papyrus_0.1.0_amd64.deb (1,100,860 bytes); dpkg-deb metadata Version 0.1.0, Architecture amd64, Depends libc6 (>=2.34), copyright GPL-3.0-or-later; contents only /usr/bin/papyrus + copyright; embedded assets served from binary. Isolated install/uninstall via fakeroot dpkg --instdir/--admindir PASS; host unchanged. Installed smoke: --help/--version, all twelve record families, id_padding 1-6 (exit 4 on invalid), unified search, browser startup/assets/inventory/sidebar, metadata projection, lifecycle+skills pages, lazy migration+round-trip, offline over loopback (unshare -rn: outbound TCP failed, search/browser/assets succeeded). New test: tests/content_management_smoke.rs (4 tests). Offline limitation wording corrected in docs/packaging.md.
<!-- SECTION:NOTES:END -->

## Comments

<!-- COMMENTS:BEGIN -->
author: rust-dev
created: 2026-11-10T19:15:00Z

---

TASK-20 verification evidence (2026-11-10 UTC): cargo build --release PASS; cargo test PASS (506 tests); clippy PASS 0 warnings; fmt PASS. Package papyrus_0.1.0_amd64.deb 1100976 bytes, metadata verified (Package/Version/Arch/Depends/copyright GPL-3.0-or-later), contents only binary+copyright. Isolated install/uninstall PASS; host unchanged. New tests/content_management_smoke.rs (4 tests) PASS. docs/packaging.md updated.

---
<!-- COMMENTS:END -->
