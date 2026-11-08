---
id: TASK-46
title: Package and verify the business rules milestone
status: Done
assignee: []
created_date: '2026-11-04T19:20:00Z'
updated_date: '2026-11-07T08:23:00Z'
labels:
  - packaging
milestone: m-8
dependencies:
  - TASK-43
  - TASK-44
  - TASK-25
references:
  - docs/specifications/business-rules-management.md
  - docs/specifications/search-business-rules.md
  - docs/specifications/browser-business-rules.md
  - docs/specifications/ui/browser-business-rules-ui.md
  - docs/specifications/cli-milestone-packaging-and-verification.md
  - docs/packaging.md
  - docs/requirements/business/REQ-05.005.md
  - docs/requirements/business/REQ-05.007.md
  - docs/requirements/business/REQ-06.001.md
  - docs/requirements/business/REQ-06.002.md
  - docs/requirements/non-functional/NFR-01.002.md
  - docs/requirements/non-functional/NFR-01.003.md
  - docs/requirements/non-functional/NFR-03.002.md
  - docs/requirements/non-functional/NFR-04.001.md
  - docs/requirements/non-functional/NFR-04.002.md
  - docs/requirements/non-functional/NFR-05.001.md
  - docs/requirements/non-functional/NFR-06.001.md
  - docs/requirements/non-functional/NFR-06.002.md
  - docs/business-rules/BR-004-browser-command-behavior.md
  - docs/business-rules/BR-013-rule-command-behavior.md
  - docs/business-rules/BR-016-configuration-keys-and-precedence.md
  - docs/architecture/adr/ADR-0001-implementation-language-rust.md
  - docs/architecture/adr/ADR-0003-supported-debian-release-target-forky-only.md
  - docs/architecture/adr/ADR-0004-repository-root-single-crate-layout.md
  - >-
    docs/architecture/adr/ADR-0005-embedded-dependency-free-http-server-and-server-rendered-ui.md
  - >-
    docs/architecture/adr/ADR-0006-optimistic-concurrency-and-atomic-record-writes.md
documentation:
  - docs/specifications/business-rules-management.md
  - docs/specifications/search-business-rules.md
  - docs/specifications/browser-business-rules.md
  - docs/specifications/ui/browser-business-rules-ui.md
  - docs/specifications/cli-milestone-packaging-and-verification.md
  - docs/packaging.md
priority: medium
type: chore
ordinal: 26400
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Package and verify the business-rules milestone (m-8) in an installable increment: rebuild the amd64 Debian package from the post-TASK-45 source, verify its metadata and GPL-3.0-or-later licensing, install and uninstall it in an isolated Debian Forky environment, and run smoke tests proving the packaged binary can manage business rules, search business rules, and serve the offline local browser business-rule category end to end. Add one milestone-level integration test and update docs/packaging.md with the Forky-only supported release, procedures, checklist, and observed results.

Scope (in):
- Rebuild target/release/papyrus and papyrus_<version>_amd64.deb via scripts/build-deb.sh (unchanged); cargo build --release must run first (stale-binary guard via AC #2).
- Verify package metadata and contents (binary and copyright only; assets compiled into the binary).
- Isolated install/uninstall on Debian Forky via fakeroot dpkg --instdir/--admindir under ./.tmp; host package database and filesystem unchanged.
- Installed-binary smoke tests: business-rule lifecycle (create/list/view/edit/deprecate/restore, preserving IDs), requirements-tag interaction (--requirement/--no-requirement), search (quoted phrase, --type doc, --limit, --json, deprecated hidden), browser (?type=business-rule, view, field-preserving edit round-trip, JSON API, assets, 320 px markup, no external references).
- Offline operation, browser configuration keys, embedded assets/no runtime file dependency.
- Field-preserving browser edit: only edited sections change; all other bytes of the rule file are preserved (browser-business-rules spec).
- Add tests/business_rules_smoke.rs (milestone-level end-to-end test) and update docs/packaging.md; record verification evidence in ticket notes.

Scope (out):
- No new product behavior, commands, options, or UI changes; feature behavior delivered by TASK-43 (management), TASK-44 (search), TASK-25 (browser), TASK-45 (design review, MOCK-0007 approved).
- Business rules have no supersede command (BR-013 documents active -> deprecated only); lifecycle smoke covers create/list/view/edit/deprecate/restore only. No supersede step is claimed.
- No verification on Debian releases other than Forky (ADR-0003, NFR-03.002); no real-GUI browser checks (markup/asset-level only); no dependency-license audit re-run; no changes to requirements, business rules, ADRs, packaging scripts, or UI; product code changes only as a minimal bug fix found during verification.

Milestone m-8 scope vs packaging: this ticket packages and verifies the m-8 business-rules increment only; it does not re-deliver management, search, browser, or design-review behavior.

Assumptions:
- Assumption: the verification host is Debian Forky amd64 (forky/sid, glibc 2.43, dpkg 1.23.11), rustc/cargo 1.99.1, fakeroot and unshare -rn available.
- Assumption: isolation via fakeroot dpkg --instdir/--admindir is acceptable (open question OQ-2).
- Assumption: scratch directories live under ./.tmp/ and are removed before the task ends.
- Assumption: the package version stays 0.1.0 (open question OQ-4).
- Assumption: scripts/build-deb.sh is used unchanged and does not force a rebuild; cargo build --release must run first.
- Assumption: no new runtime dependency was added; the TASK-1/TASK-4 runtime dependency license audit remains valid (NFR-06.002).
- Assumption: no UI change ships in this ticket; MOCK-0001 through MOCK-0007 approved baselines remain the layout references and no new mockup is produced.
- Assumption: business-rule browser edit is field-preserving (browser-business-rules spec); smoke asserts only edited sections change.

Open questions (non-blocking, each with a default in the specification):
- OQ-1 (default: markup/asset-level verification only; no real-GUI browser run) - NFR-05.001 verification method.
- OQ-2 (default: fakeroot dpkg --instdir/--admindir on the Forky build host) - container/VM isolation.
- OQ-3 (default: new tests/business_rules_smoke.rs) - milestone test file location.
- OQ-4 (default: keep version 0.1.0).
- OQ-5 (flag for spec owner): no dedicated business-rules milestone-packaging-and-verification specification exists under docs/specifications/ (unlike glossary/ui-spec/ui-mockups which have one). This ticket currently references the generic docs/specifications/cli-milestone-packaging-and-verification.md plus the feature specs. A dedicated docs/specifications/business-rules-milestone-packaging-and-verification.md should be authored by the spec owner; until then, the generic spec plus feature specs are the references.

Specification: docs/specifications/cli-milestone-packaging-and-verification.md (generic; dedicated business-rules milestone-packaging spec absent - see OQ-5). Feature specs: docs/specifications/business-rules-management.md, docs/specifications/search-business-rules.md, docs/specifications/browser-business-rules.md, docs/specifications/ui/browser-business-rules-ui.md.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The amd64 Debian package built from the current source installs on Debian Forky (amd64) - the only supported release per ADR-0003 and NFR-03.002 - and the installed binary provides business-rule management, search, and browser (business-rule category); Debian Trixie and other releases are not claimed or verified.
- [x] #2 cargo build --release succeeds with the pinned Rust 1.99.1 toolchain and target/release/papyrus --help lists rule (guards against packaging a stale binary).
- [x] #3 cargo test, cargo clippy --all-targets -- -D warnings, and cargo fmt --check pass with Rust 1.99.1, and the test counts are recorded.
- [x] #4 scripts/build-deb.sh (unchanged) produces papyrus_<version>_amd64.deb with Version equal to the Cargo package version and Architecture: amd64.
- [x] #5 dpkg-deb metadata reports Package: papyrus, Architecture: amd64, Version equal to Cargo.toml, Depends: libc6 (>= 2.34), and a copyright file declaring GPL-3.0-or-later.
- [x] #6 The package contains only /usr/bin/papyrus (0755) and /usr/share/doc/papyrus/copyright, with no asset, template, or configuration files, and the installed binary serves /assets/app.css and /assets/app.js from assets compiled into the binary.
- [x] #7 In an isolated Debian Forky environment, dpkg -i succeeds, /usr/bin/papyrus exists as root:root 0755, papyrus resolves on PATH, and the host package database and filesystem are unchanged.
- [x] #8 In the same isolated environment, dpkg -r (or dpkg --purge) papyrus removes the binary and papyrus no longer resolves on PATH.
- [x] #9 The installed binary smoke test passes: papyrus --help exits 0 and lists rule, search, and browser, and papyrus --version exits 0 printing papyrus <version> matching the package version.
- [x] #10 The installed binary initializes a project and exercises the business-rule lifecycle (create/list/view/edit/deprecate/restore) with expected exit codes and persisted files, preserving IDs; requirements-tag interaction (--requirement add, --no-requirement remove/all) is exercised.
- [x] #11 Installed-binary search smoke tests pass over business rules: a quoted phrase matches contiguously, --type doc includes business rules, --limit caps results, --json returns the {id,type,title,path} array (and [] when empty), and deprecated rules are hidden until restored.
- [x] #12 Installed-binary browser smoke tests pass with --no-open on a free port: the startup line is printed, the business-rule category renders with the selected sidebar entry marked aria-current, a rule view renders, a field-preserving edit round-trip returns 303 and persists through both the UI and the CLI, the JSON API responds (/api/business-rules, /api/doc/<id>), both embedded assets are served, 320 px markup hooks are present, no served page/stylesheet/script references an external origin, and deprecated rules are not listed or reachable (404).
- [x] #13 Offline operation is verified with no internet access (isolated network namespace, or the documented fallback evidence): search and browser view/edit checks pass using only loopback and locally served assets.
- [x] #14 Browser configuration keys work in the installed binary: config list/get/set/unset expose browser_host, browser_port, and browser_open, a configured port is used when --port is absent, --port overrides it, defaults are restored on unset, and invalid values exit 4 without changing the file.
- [x] #15 A milestone-level integration test in tests/business_rules_smoke.rs exercises the business-rule lifecycle, search, and browser together end-to-end against the built binary, passes under cargo test, and does not duplicate the contract coverage in tests/business_rules_cli.rs, tests/search_cli.rs, or tests/browser_cli.rs.
- [x] #16 docs/packaging.md states Debian Forky only (amd64), explicitly disclaims Trixie and other releases, and records the isolated install/uninstall procedure, the extended smoke-test checklist, and observed results with environment and date including timezone.
- [x] #17 Verification evidence (environment, date/timezone, package file and size, quality-gate results with test counts, smoke-test outcomes, deviations) is recorded in the TASK-46 ticket notes.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 All acceptance criteria verified and recorded in the ticket notes and docs/packaging.md.
- [x] #2 cargo build --release, cargo test, cargo clippy --all-targets -- -D warnings, and cargo fmt --check pass with Rust 1.99.1.
- [x] #3 The .deb metadata and contents match the specification, and the package installs and uninstalls cleanly in an isolated Debian Forky environment with the host untouched.
- [x] #4 The installed-binary smoke checklist passes (business-rule lifecycle, search, browser, offline operation, config keys, embedded assets).
- [x] #5 The new tests/business_rules_smoke.rs passes and the full existing test suite remains green.
- [x] #6 docs/packaging.md is updated with the Forky-only claim, procedures, checklist, observed results (date with timezone), and any recorded limitation.
- [x] #7 No requirements, business rules, ADRs, packaging scripts, or UI were changed; any product bug fix is recorded separately in the notes.
<!-- DOD:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
TASK-46 implementation complete in working tree (no commit). Gates: cargo build PASS; cargo build --release PASS (target/release/papyrus --help lists rule/search/browser); cargo test PASS 322 (59 unit + 263 integration incl. business_rules_smoke 2); cargo clippy --all-targets -- -D warnings PASS 0 warnings; cargo fmt --check PASS. Package papyrus_0.1.0_amd64.deb 903368 bytes SHA-256 a287e67143a4064d36fc66bcf191e8511c4e12cce95e5cf82e42184df8afd724; metadata Package papyrus, Version 0.1.0, Architecture amd64, Depends libc6 (>= 2.34), copyright GPL-3.0-or-later; contents /usr/bin/papyrus 0755 and copyright only. Isolated fakeroot dpkg install/uninstall PASS; host unchanged. Installed smoke: help/version, business-rule lifecycle create/list/view/edit/deprecate/restore with IDs, requirements tags, search quoted phrase/--type doc/--limit/--json/deprecated hidden, browser ?type=business-rule aria-current/view/edit 303/API/assets/320px/no external refs/deprecated 404, offline unshare -rn, config keys. New test tests/business_rules_smoke.rs (2 tests). docs/packaging.md TASK-46 section added. No source changes outside packaging/tests/docs; no new deps. Business rules have no supersede command (BR-013).
<!-- SECTION:NOTES:END -->

## Comments

<!-- COMMENTS:BEGIN -->
created: 2026-11-07T08:23:00Z

---

Validated 2026-11-07 (UTC): quality-gatekeeper Pass with findings (17/17 AC, 7/7 DoD); cargo test 322 passed; isolated Forky install/uninstall + smoke verified; auto-closed under no-validation mode. Non-blocking: offline namespace/full installed smoke not independently rerun (documented), NFR-05.001 deferred, dedicated packaging spec absent (OQ-5).

---
<!-- COMMENTS:END -->
