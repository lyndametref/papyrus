---
id: TASK-50
title: Package and verify the audit records milestone
status: Done
assignee: []
created_date: '2026-11-04T19:20:00Z'
updated_date: '2026-11-07T10:59:00Z'
labels:
  - packaging
milestone: m-9
dependencies:
  - TASK-47
  - TASK-48
  - TASK-26
references:
  - docs/specifications/audit-records-management.md
  - docs/specifications/search-audit-records.md
  - docs/specifications/browser-audit-records.md
  - docs/specifications/ui/browser-audit-records-ui.md
  - docs/specifications/cli-milestone-packaging-and-verification.md
  - docs/packaging.md
  - docs/requirements/business/REQ-05.006.md
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
  - docs/business-rules/BR-014-audit-command-behavior.md
  - docs/business-rules/BR-016-configuration-keys-and-precedence.md
  - docs/architecture/adr/ADR-0001-implementation-language-rust.md
  - docs/architecture/adr/ADR-0003-supported-debian-release-target-forky-only.md
  - docs/architecture/adr/ADR-0004-repository-root-single-crate-layout.md
  - >-
    docs/architecture/adr/ADR-0005-embedded-dependency-free-http-server-and-server-rendered-ui.md
  - >-
    docs/architecture/adr/ADR-0006-optimistic-concurrency-and-atomic-record-writes.md
documentation:
  - docs/specifications/audit-records-management.md
  - docs/specifications/search-audit-records.md
  - docs/specifications/browser-audit-records.md
  - docs/specifications/ui/browser-audit-records-ui.md
  - docs/specifications/cli-milestone-packaging-and-verification.md
  - docs/packaging.md
priority: medium
type: chore
ordinal: 27400
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Package and verify the audit-records milestone (m-9) in an installable increment: rebuild the amd64 Debian package from the post-TASK-49 source, verify its metadata and GPL-3.0-or-later licensing, install and uninstall it in an isolated Debian Forky environment, and run smoke tests proving the packaged binary can manage audit records, search audit records, and serve the offline local browser audit-record category end to end. Add one milestone-level integration test and update docs/packaging.md with the Forky-only supported release, procedures, checklist, and observed results.

Scope (in):
- Rebuild target/release/papyrus and papyrus_<version>_amd64.deb via scripts/build-deb.sh (unchanged); cargo build --release must run first (stale-binary guard via AC #2).
- Verify package metadata and contents (binary and copyright only; assets compiled into the binary).
- Isolated install/uninstall on Debian Forky via fakeroot dpkg --instdir/--admindir under ./.tmp; host package database and filesystem unchanged.
- Installed-binary smoke tests: audit-record lifecycle (create/list/view/edit/deprecate/restore, preserving IDs), with --title/--content/--audit-type (default security; built-in security/licence/architecture plus free-form sanitized values), search (quoted phrase, --type doc, --limit, --json, deprecated hidden), browser (?type=audit-record, view with audit type shown and content as body text without a "Content" heading, field-preserving edit round-trip, JSON API, assets, 320 px markup, no external references).
- Offline operation, browser configuration keys, embedded assets/no runtime file dependency.
- Field-preserving browser edit: only edited fields change; all other bytes of the record file are preserved (browser-audit-records spec); audit type shown in the card.
- Add tests/audit_records_smoke.rs (milestone-level end-to-end test) and update docs/packaging.md; record verification evidence in ticket notes.

Scope (out):
- No new product behavior, commands, options, or UI changes; feature behavior delivered by TASK-47 (management), TASK-48 (search), TASK-26 (browser), TASK-49 (design review, MOCK-0008 approved).
- Audit records have no supersede command (BR-014 documents active -> deprecated only); lifecycle smoke covers create/list/view/edit/deprecate/restore only. No supersede step is claimed. No requirements-tag interaction (that is business rules, BR-013).
- No verification on Debian releases other than Forky (ADR-0003, NFR-03.002); no real-GUI browser checks (markup/asset-level only); no dependency-license audit re-run; no changes to requirements, business rules, ADRs, packaging scripts, or UI; product code changes only as a minimal bug fix found during verification.

Milestone m-9 scope vs packaging: this ticket packages and verifies the m-9 audit-records increment only; it does not re-deliver management, search, browser, or design-review behavior.

Assumptions:
- Assumption: the verification host is Debian Forky amd64 (forky/sid, glibc 2.43, dpkg 1.23.11), rustc/cargo 1.99.1, fakeroot and unshare -rn available.
- Assumption: isolation via fakeroot dpkg --instdir/--admindir is acceptable (open question OQ-2).
- Assumption: scratch directories live under ./.tmp/ and are removed before the task ends.
- Assumption: the package version stays 0.1.0 (open question OQ-4).
- Assumption: scripts/build-deb.sh is used unchanged and does not force a rebuild; cargo build --release must run first.
- Assumption: no new runtime dependency was added; the TASK-1/TASK-4 runtime dependency license audit remains valid (NFR-06.002).
- Assumption: no UI change ships in this ticket; MOCK-0001 through MOCK-0008 approved baselines remain the layout references and no new mockup is produced.
- Assumption: audit-record browser edit is field-preserving (browser-audit-records spec); smoke asserts only edited fields change.
- Assumption: audit record content may be empty on browser edit, which clears the ## Content section; the CLI handles an empty/absent content on subsequent reads (browser-audit-records spec OQ, adopted default).

Open questions (non-blocking, each with a default in the specification):
- OQ-1 (default: markup/asset-level verification only; no real-GUI browser run) - NFR-05.001 verification method.
- OQ-2 (default: fakeroot dpkg --instdir/--admindir on the Forky build host) - container/VM isolation.
- OQ-3 (default: new tests/audit_records_smoke.rs) - milestone test file location.
- OQ-4 (default: keep version 0.1.0).
- OQ-5 (flag for spec owner): no dedicated audit-records milestone-packaging-and-verification specification exists under docs/specifications/ (unlike glossary/ui-spec/ui-mockups/business-rules which have one). This ticket currently references the generic docs/specifications/cli-milestone-packaging-and-verification.md plus the feature specs. A dedicated docs/specifications/audit-records-milestone-packaging-and-verification.md should be authored by the spec owner; until then, the generic spec plus feature specs are the references.

Specification: docs/specifications/cli-milestone-packaging-and-verification.md (generic; dedicated audit-records milestone-packaging spec absent - see OQ-5). Feature specs: docs/specifications/audit-records-management.md, docs/specifications/search-audit-records.md, docs/specifications/browser-audit-records.md, docs/specifications/ui/browser-audit-records-ui.md.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The amd64 Debian package built from the current source installs on Debian Forky (amd64) - the only supported release per ADR-0003 and NFR-03.002 - and the installed binary provides audit-record management, search, and browser (audit-record category); Debian Trixie and other releases are not claimed or verified.
- [x] #2 cargo build --release succeeds with the pinned Rust 1.99.1 toolchain and target/release/papyrus --help lists audit (guards against packaging a stale binary).
- [x] #3 cargo test, cargo clippy --all-targets -- -D warnings, and cargo fmt --check pass with Rust 1.99.1, and the test counts are recorded.
- [x] #4 scripts/build-deb.sh (unchanged) produces papyrus_<version>_amd64.deb with Version equal to the Cargo package version and Architecture: amd64.
- [x] #5 dpkg-deb metadata reports Package: papyrus, Architecture: amd64, Version equal to Cargo.toml, Depends: libc6 (>= 2.34), and a copyright file declaring GPL-3.0-or-later.
- [x] #6 The package contains only /usr/bin/papyrus (0755) and /usr/share/doc/papyrus/copyright, with no asset, template, or configuration files, and the installed binary serves /assets/app.css and /assets/app.js from assets compiled into the binary.
- [x] #7 In an isolated Debian Forky environment, dpkg -i succeeds, /usr/bin/papyrus exists as root:root 0755, papyrus resolves on PATH, and the host package database and filesystem are unchanged.
- [x] #8 In the same isolated environment, dpkg -r (or dpkg --purge) papyrus removes the binary and papyrus no longer resolves on PATH.
- [x] #9 The installed binary smoke test passes: papyrus --help exits 0 and lists audit, search, and browser, and papyrus --version exits 0 printing papyrus <version> matching the package version.
- [x] #10 The installed binary initializes a project and exercises the audit-record lifecycle (create/list/view/edit/deprecate/restore) with expected exit codes and persisted files, preserving IDs; create supports --title, --content, and --audit-type (default security; built-in security/licence/architecture plus free-form sanitized values); edit preserves omitted fields; the audit type is shown on view; empty content clears the section.
- [x] #11 Installed-binary search smoke tests pass over audit records: a quoted phrase matches contiguously, --type doc includes audit records, --limit caps results, --json returns the {id,type,title,path} array (and [] when empty), and deprecated audit records are hidden until restored.
- [x] #12 Installed-binary browser smoke tests pass with --no-open on a free port: the startup line is printed, the audit-record category renders with the selected sidebar entry marked aria-current, a record view renders with the audit type shown (badge next to title/ID plus an Audit type row) and content rendered as body text without a Content heading, a field-preserving edit round-trip returns 303 and persists through both the UI and the CLI, the JSON API responds (/api/audit-records, /api/audit-record, /api/doc/<id>), both embedded assets are served, 320 px markup hooks are present, no served page/stylesheet/script references an external origin, and deprecated audit records are not listed or reachable (404).
- [x] #13 Offline operation is verified with no internet access (isolated network namespace, or the documented fallback evidence): search and browser view/edit checks pass using only loopback and locally served assets.
- [x] #14 Browser configuration keys work in the installed binary: config list/get/set/unset expose browser_host, browser_port, and browser_open, a configured port is used when --port is absent, --port overrides it, defaults are restored on unset, and invalid values exit 4 without changing the file.
- [x] #15 A milestone-level integration test in tests/audit_records_smoke.rs exercises the audit-record lifecycle, search, and browser together end-to-end against the built binary, passes under cargo test, and does not duplicate the contract coverage in tests/audit_records_cli.rs, tests/search_cli.rs, or tests/browser_cli.rs.
- [x] #16 docs/packaging.md states Debian Forky only (amd64), explicitly disclaims Trixie and other releases, and records the isolated install/uninstall procedure, the extended smoke-test checklist, and observed results with environment and date including timezone.
- [x] #17 Verification evidence (environment, date/timezone, package file and size, quality-gate results with test counts, smoke-test outcomes, deviations) is recorded in the TASK-50 ticket notes.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 All acceptance criteria verified and recorded in the ticket notes and docs/packaging.md.
- [x] #2 cargo build --release, cargo test, cargo clippy --all-targets -- -D warnings, and cargo fmt --check pass with Rust 1.99.1.
- [x] #3 The .deb metadata and contents match the specification, and the package installs and uninstalls cleanly in an isolated Debian Forky environment with the host untouched.
- [x] #4 The installed-binary smoke checklist passes (audit-record lifecycle, search, browser, offline operation, config keys, embedded assets).
- [x] #5 The new tests/audit_records_smoke.rs passes and the full existing test suite remains green.
- [x] #6 docs/packaging.md is updated with the Forky-only claim, procedures, checklist, observed results (date with timezone), and any recorded limitation.
- [x] #7 No requirements, business rules, ADRs, packaging scripts, or UI were changed; any product bug fix is recorded separately in the notes.
<!-- DOD:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Package artifact: papyrus_0.1.0_amd64.deb, 910548 bytes, SHA-256 fcc0fd05c9684ffc11a3b770af9f94eab2f7bb6928cff44335662407de7b605d. Metadata: Package papyrus, Version 0.1.0, Architecture amd64, Depends libc6 (>= 2.34), Section utils, Priority optional, copyright GPL-3.0-or-later. Contents: /usr/bin/papyrus (0755, 2858256 bytes) and /usr/share/doc/papyrus/copyright (0644, 865 bytes) only. Quality gates: cargo build, cargo build --release, cargo test (352 passed), cargo clippy (0 warnings), cargo fmt --check (pass). Isolated install/uninstall PASS; host unchanged. Offline (unshare -rn) PASS. Config keys PASS. Smoke test audit_records_smoke 2/2 PASS. Deviations: no real-GUI browser run (OQ-1); no supersede command (BR-014); dedicated packaging spec absent (OQ-5). Date 2026-11-07 (UTC).
<!-- SECTION:NOTES:END -->

## Comments

<!-- COMMENTS:BEGIN -->
created: 2026-11-07T10:57:00Z

---

Corrected package size/hash in docs/packaging.md and consolidated notes to 910548 bytes / fcc0fd05... (actual artifact). Previous 910352/769bb000 values were erroneous.

---

created: 2026-11-07T10:59:00Z

---

Validated 2026-11-07 (UTC): quality-gatekeeper Pass with findings (17/17 AC, 7/7 DoD); cargo test 352 passed; isolated Forky install/uninstall + smoke verified; package evidence reconciled to 910548/fcc0fd05; auto-closed under no-validation mode.

---
<!-- COMMENTS:END -->
