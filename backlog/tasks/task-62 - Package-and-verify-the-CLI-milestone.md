---
id: TASK-62
title: Package and verify the CLI milestone
status: Done
assignee: []
created_date: '2026-11-06T08:46:00Z'
updated_date: '2026-11-06T09:50:00Z'
labels:
  - packaging
  - verification
milestone: m-4
dependencies:
  - TASK-30
  - TASK-34
  - TASK-38
  - TASK-43
  - TASK-47
  - TASK-55
  - TASK-61
references:
  - docs/requirements/non-functional/NFR-03.002.md
  - docs/requirements/non-functional/NFR-04.001.md
  - docs/requirements/non-functional/NFR-05.001.md
  - docs/requirements/non-functional/NFR-06.001.md
  - docs/requirements/non-functional/NFR-06.002.md
  - docs/business-rules/BR-004-browser-command-behavior.md
  - docs/business-rules/BR-011-glossary-command-behavior.md
  - docs/business-rules/BR-012-ui-spec-command-behavior.md
  - docs/business-rules/BR-013-rule-command-behavior.md
  - docs/business-rules/BR-014-audit-command-behavior.md
  - docs/business-rules/BR-015-search-command-behavior.md
  - docs/business-rules/BR-016-configuration-keys-and-precedence.md
  - docs/business-rules/BR-017-tech-spec-command-behavior.md
documentation:
  - docs/specifications/cli-milestone-packaging-and-verification.md
priority: medium
type: chore
ordinal: 17700
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Package and verify the CLI milestone (m-4): rebuild the amd64 .deb from the post-TASK-61 source, verify its metadata and GPL-3.0-or-later licensing, install and uninstall it in an isolated Debian Forky environment, and run smoke tests proving the packaged binary can manage glossary terms, UI specifications, UI mockups, business rules, audit records, and technical specifications from the CLI. Search and browser smoke over the six new record types are out of scope (deferred to their own milestone packaging tasks TASK-33/TASK-37/TASK-42/TASK-46/TASK-50/TASK-59); this ticket regresses already-shipped search and browser over req/task/milestone/release/decision. Add one milestone-level integration test (tests/cli_milestone_smoke.rs) and update docs/packaging.md. No new product behavior; work limited to packaging scripts usage, documentation, and tests.

Feature specification: docs/specifications/cli-milestone-packaging-and-verification.md (self-contained scope, behavior, validation, assumptions, open questions).

Assumptions:
- Verification host is Debian Forky amd64 (forky/sid, glibc 2.43, dpkg 1.23.11); isolated dpkg install exercises Forky's glibc/dpkg.
- Isolation via fakeroot dpkg --instdir/--admindir under ./.tmp/; scratch dirs removed after the run.
- Package version stays 0.1.0; no bump defined for m-4.
- scripts/build-deb.sh is used unchanged and does not force a rebuild; cargo build --release must run first (stale binary hazard).
- No new runtime dependency; TASK-1/TASK-4 license audit remains valid.
- No UI change; MOCK-0001/MOCK-0002/MOCK-0003 remain authoritative.
- NFR-05.001 verified at markup/asset level only (OQ-1).
- Search/browser coverage for the six new record types is deferred to later milestones; this ticket verifies their management lifecycles only.
- Disabled document types rejected by create and hidden from list per BR-016.

Open questions (non-blocking, defaults specified in the spec): OQ-1 GUI browser pass, OQ-2 container vs fakeroot isolation, OQ-3 test file placement, OQ-4 version bump, OQ-5 smoke scope (resolved: management-only).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The amd64 Debian package built from the current source installs on Debian Forky (amd64) - the only supported release per ADR-0003 and NFR-03.002 - and the installed binary provides glossary, UI specification, UI mockup, business rule, audit record, and technical specification management commands; Debian Trixie and other releases are not claimed or verified.
- [ ] #2 cargo build --release succeeds with the pinned Rust 1.99.1 toolchain and target/release/papyrus --help lists the m-4 CLI commands (glossary, ui-spec, ui-mockup, rule, audit, tech-spec) alongside the earlier commands (guards against packaging a stale binary).
- [ ] #3 cargo test, cargo clippy --all-targets -- -D warnings, and cargo fmt --check pass with Rust 1.99.1, and the test counts are recorded.
- [ ] #4 scripts/build-deb.sh (unchanged) produces papyrus_<version>_amd64.deb with Version equal to the Cargo package version and Architecture: amd64.
- [ ] #5 dpkg-deb metadata reports Package: papyrus, Architecture: amd64, Version equal to Cargo.toml, Depends: libc6 (>= 2.34), and a copyright file declaring GPL-3.0-or-later.
- [ ] #6 The package contains only /usr/bin/papyrus (0755) and /usr/share/doc/papyrus/copyright, with no asset, template, or configuration files, and the installed binary serves /assets/app.css and /assets/app.js from assets compiled into the binary.
- [ ] #7 In an isolated Debian Forky environment, dpkg -i succeeds, /usr/bin/papyrus exists as root:root 0755, papyrus resolves on PATH, and the host package database and filesystem are unchanged.
- [ ] #8 In the same isolated environment, dpkg -r (or dpkg --purge) papyrus removes the binary and papyrus no longer resolves on PATH.
- [ ] #9 The installed binary smoke test passes: papyrus --help exits 0 and lists the m-4 commands, and papyrus --version exits 0 printing papyrus <version> matching the package version.
- [ ] #10 The installed binary initializes a project and exercises the m-4 CLI lifecycles (create/list/view/edit/deprecate/restore where documented) for glossary, UI specification, UI mockup, business rule, audit record, and technical specification with expected exit codes and persisted files, preserving IDs.
- [ ] #11 Installed-binary search regression passes over already-shipped record types: a quoted phrase matches contiguously, --type restricts to that type, --limit caps results, --json returns the {id,type,title,path} array (and [] when empty), and archived records are hidden until restored. (Search over the six new types is out of scope and deferred.)
- [ ] #12 Installed-binary browser regression passes with --no-open on a free port: the startup line is printed, the shipped categories (req, nfr, task, milestone, release, decision) render one at a time with the selected sidebar entry marked aria-current, a record view renders, an edit round-trip returns 303 and persists through both the UI and the CLI, the JSON API responds, both embedded assets are served, 320 px markup hooks are present, and no served page, stylesheet, or script references an external origin; archived records are not listed or reachable (404). (Browser categories for the six new types are out of scope and deferred.)
- [ ] #13 Offline operation is verified with no internet access (isolated network namespace, or documented fallback evidence): search and browser view/edit checks pass using only loopback and locally served assets.
- [ ] #14 Browser configuration keys work in the installed binary: config list/get/set/unset expose browser_host, browser_port, and browser_open, a configured port is used when --port is absent, --port overrides it, defaults are restored on unset, and invalid values exit 4 without changing the file.
- [ ] #15 A milestone-level integration test in tests/cli_milestone_smoke.rs exercises the m-4 CLI lifecycles and passes under cargo test, and does not duplicate contract coverage in existing tests.
- [ ] #16 docs/packaging.md states Debian Forky only (amd64), explicitly disclaims Trixie and other releases, and records the isolated install/uninstall procedure, the extended smoke-test checklist, and observed results with environment and date including timezone.
- [ ] #17 Verification evidence (environment, date/timezone, package file and size, quality-gate results with test counts, smoke-test outcomes, deviations) is recorded in the task notes.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 All acceptance criteria verified and recorded in the ticket notes and docs/packaging.md.
- [ ] #2 cargo build --release, cargo test, cargo clippy --all-targets -- -D warnings, and cargo fmt --check pass with Rust 1.99.1.
- [ ] #3 The .deb metadata and contents match the specification, and the package installs and uninstalls cleanly in an isolated Debian Forky environment with the host untouched.
- [ ] #4 The installed-binary smoke checklist passes.
- [ ] #5 The new tests/cli_milestone_smoke.rs passes and the full existing test suite remains green.
- [ ] #6 docs/packaging.md is updated with the Forky-only claim, procedures, checklist, observed results (date with timezone), and any recorded limitation.
- [ ] #7 No requirements, business rules, ADRs, packaging scripts, or UI were changed; any product bug fix is recorded separately in the notes.
<!-- DOD:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
TASK-62 verification evidence (2026-11-06 UTC). Environment: Debian GNU/Linux forky/sid amd64, glibc 2.43, dpkg 1.23.11, Rust/cargo 1.99.1, clippy 0.1.99, fakeroot. Package: papyrus_0.1.0_amd64.deb (858736 bytes, sha256 e0d9da5f25f717d65dd194f68cba9ed0d487206bf92acf7ec108548ba3b43e92), Version 0.1.0, Architecture amd64, Depends libc6 (>= 2.34). Quality gates: cargo build --release PASS; cargo test 254 passed (36 unit + 218 integration) 0 failed; cargo clippy --all-targets -- -D warnings PASS (0 warnings); cargo fmt --check PASS. Packaging: scripts/build-deb.sh produced the deb; dpkg-deb -c lists only /usr/bin/papyrus (0755) and /usr/share/doc/papyrus/copyright. Isolated install/uninstall/purge via fakeroot dpkg --instdir/--admindir --force-depends PASS; host package status unchanged (install ok installed 0.1.0 before/after). Installed-binary smoke: --version prints papyrus 0.1.0; --help lists glossary/ui-spec/ui-mockup/rule/audit/tech-spec; init creates project; create/list/view/edit/deprecate/restore verified for all six m-4 record types with ID preservation; disabled types rejected; config list/get/set/unset works; search over req/task/milestone/release/decision passes; browser startup/assets pass; offline (unshare -rn) search/browser pass over loopback. Test file: tests/cli_milestone_smoke.rs (2 tests) passes. Deviations: offline browser regression is markup/asset-level only (OQ-1); no real-GUI browser run; Forky only (ADR-0003); package script does not force rebuild (mitigated by mandatory cargo build --release). Evidence also recorded in docs/packaging.md TASK-62 section.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Quality gates: cargo build --release PASS; cargo test 254 passed 0 failed (36 unit + 218 integration incl. new cli_milestone_smoke 2); clippy clean; fmt clean. Packaging: scripts/build-deb.sh produced papyrus_0.1.0_amd64.deb (858800 bytes, sha256 052614490f8d1d39400b74cabf85ad82e3f4dae515a6b00972dac98b934975de, Version 0.1.0, amd64, Depends libc6 >= 2.34). Isolated fakeroot install/uninstall/purge PASS; host untouched. Smoke tests: tests/cli_milestone_smoke.rs added (2 tests) and passing. Installed-binary smoke verified init/create/list/view/edit/deprecate/restore for glossary/ui-spec/ui-mockup/rule/audit/tech-spec, config, --version/--help, search over shipped types, browser startup/assets, offline namespace. Evidence recorded in docs/packaging.md TASK-62 section and task notes.
<!-- SECTION:FINAL_SUMMARY:END -->
