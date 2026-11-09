---
id: TASK-59
title: Package and verify the technical specifications milestone
status: Done
assignee: []
created_date: '2026-11-04T19:21:00Z'
updated_date: '2026-11-07T17:41:00Z'
labels:
  - packaging
milestone: m-11
dependencies:
  - TASK-55
  - TASK-56
  - TASK-57
references:
  - >-
    docs/specifications/technical-specifications-milestone-packaging-and-verification.md
  - docs/specifications/technical-specifications-management.md
  - docs/specifications/search-technical-specifications.md
  - docs/specifications/browser-technical-specifications.md
  - docs/specifications/ui/browser-technical-specifications-ui.md
  - docs/specifications/cli-milestone-packaging-and-verification.md
  - docs/packaging.md
  - docs/requirements/business/REQ-05.007.md
  - docs/requirements/business/REQ-05.009.md
  - docs/requirements/business/REQ-06.001.md
  - docs/requirements/business/REQ-06.002.md
  - docs/requirements/non-functional/NFR-01.002.md
  - docs/requirements/non-functional/NFR-01.003.md
  - docs/requirements/non-functional/NFR-02.002.md
  - docs/requirements/non-functional/NFR-03.002.md
  - docs/requirements/non-functional/NFR-04.001.md
  - docs/requirements/non-functional/NFR-04.002.md
  - docs/requirements/non-functional/NFR-05.001.md
  - docs/requirements/non-functional/NFR-06.001.md
  - docs/requirements/non-functional/NFR-06.002.md
  - docs/business-rules/BR-004-browser-command-behavior.md
  - docs/business-rules/BR-015-search-command-behavior.md
  - docs/business-rules/BR-016-configuration-keys-and-precedence.md
  - docs/business-rules/BR-017-tech-spec-command-behavior.md
  - docs/architecture/adr/ADR-0001-implementation-language-rust.md
  - docs/architecture/adr/ADR-0003-supported-debian-release-target-forky-only.md
  - docs/architecture/adr/ADR-0004-repository-root-single-crate-layout.md
  - >-
    docs/architecture/adr/ADR-0005-embedded-dependency-free-http-server-and-server-rendered-ui.md
  - >-
    docs/architecture/adr/ADR-0006-optimistic-concurrency-and-atomic-record-writes.md
  - docs/domain-model/business-glossary/technical-specification.md
documentation:
  - >-
    docs/specifications/technical-specifications-milestone-packaging-and-verification.md
  - docs/specifications/technical-specifications-management.md
  - docs/specifications/search-technical-specifications.md
  - docs/specifications/browser-technical-specifications.md
  - docs/specifications/ui/browser-technical-specifications-ui.md
  - docs/packaging.md
priority: medium
type: chore
ordinal: 29400
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Package and verify the technical-specifications milestone (m-11) in an installable increment: rebuild the amd64 Debian package from the post-TASK-58 source, verify its metadata and GPL-3.0-or-later licensing, install and uninstall it in an isolated Debian Forky environment, and run smoke tests proving the packaged binary can manage technical specifications, search technical specifications, and serve the offline local browser Technical Specifications category end to end. Add one milestone-level integration test and update docs/packaging.md with the Forky-only supported release, procedures, checklist, and observed results.

Pattern: follows the completed milestone-packaging tasks TASK-33 (glossary), TASK-37 (UI specifications), TASK-42 (UI mockups), TASK-46 (business rules), and TASK-50 (audit records).

Scope (in):
- Rebuild target/release/papyrus and papyrus_<version>_amd64.deb via scripts/build-deb.sh (unchanged); cargo build --release must run first (stale-binary guard via AC #2).
- Verify package metadata and contents (binary and copyright only; assets compiled into the binary).
- Isolated install/uninstall on Debian Forky via fakeroot dpkg --instdir/--admindir under ./.tmp; host package database and filesystem unchanged.
- Installed-binary smoke tests: technical-specification lifecycle (create/list/view/edit/deprecate/restore, preserving IDs; title and description required; edits change only supplied fields; deprecated specifications stay viewable by ID), search (quoted phrase, split words, --type doc, other type filters, --limit, --json, empty result, deprecated hidden until restored), and browser (?type=tech-spec, sidebar entry after Audit Records with aria-current, list with count/rows/preview, view with description body without a "Description" heading, field-preserving edit round-trip, JSON API /api/tech-specs and alias /api/tech-spec plus /api/doc/<id>, assets, 320 px markup, no external references, deprecated 404 on view/edit/POST/API with the identical 404 page and recovery link).
- Offline operation, browser configuration keys, embedded assets/no runtime file dependency.
- Field-preserving browser edit: only the heading title and the ## Description section change; preamble and every other section/line are preserved byte-for-byte (browser-technical-specifications spec).
- Add tests/technical_specifications_smoke.rs (milestone-level end-to-end test) and update docs/packaging.md; record verification evidence in ticket notes.

Scope (out):
- No new product behavior, commands, options, or UI changes; feature behavior delivered by TASK-55 (management), TASK-56 (search), TASK-57 (browser), TASK-58 (design review, MOCK-0009 approved).
- Technical specifications have no supersede or delete command (BR-017 documents create/list/view/edit/deprecate/restore only); the lifecycle smoke covers those commands only and claims no supersede step.
- No dedicated --type tech-spec search filter: the documented vocabulary is task/milestone/release/decision/doc; technical specifications are searched through --type doc and default search.
- No structured references between technical specifications and NFRs/tasks (BR-017 defines no reference fields); such IDs remain literal text in the title or description.
- No verification on Debian releases other than Forky (ADR-0003, NFR-03.002); no real-GUI browser checks (markup/asset-level only); no dependency-license audit re-run; no changes to requirements, business rules, ADRs, packaging scripts, or UI; product code changes only as a minimal bug fix found during verification.
- Final cross-type completeness verification is TASK-20 (m-12), not this ticket.

Milestone m-11 scope vs packaging: this ticket packages and verifies the m-11 technical-specifications increment only; it does not re-deliver management, search, browser, or design-review behavior.

Assumptions:
- Assumption: the verification host is Debian Forky amd64 (forky/sid, glibc 2.43, dpkg 1.23.11), rustc/cargo 1.99.1, fakeroot and unshare -rn available.
- Assumption: isolation via fakeroot dpkg --instdir/--admindir is acceptable (open question OQ-2).
- Assumption: scratch directories live under ./.tmp/ and are removed before the task ends.
- Assumption: the package version stays 0.1.0 (open question OQ-4).
- Assumption: scripts/build-deb.sh is used unchanged and does not force a rebuild; cargo build --release must run first.
- Assumption: no new runtime dependency was added; the TASK-1/TASK-4 runtime dependency license audit remains valid (NFR-06.002).
- Assumption: no UI change ships in this ticket; MOCK-0001 through MOCK-0009 approved baselines remain the layout references and no new mockup is produced.
- Assumption: the technical-specification browser edit is field-preserving (browser-technical-specifications spec); smoke asserts only the heading title and ## Description change.
- Assumption: the browser requires both title and description; an on-disk record with an absent/empty description is viewable as (none) but must be completed before saving from the browser.
- Assumption: --description remains a single-line CLI input and --editor remains the CLI route for multi-line bodies; --editor is not exercised in the isolated install (environment-dependent) and remains covered by tests/technical_specifications_cli.rs.

Open questions (non-blocking, each with a default in the specification):
- OQ-1 (default: markup/asset-level verification only; no real-GUI browser run) - NFR-05.001 verification method.
- OQ-2 (default: fakeroot dpkg --instdir/--admindir on the Forky build host) - container/VM isolation.
- OQ-3 (default: new tests/technical_specifications_smoke.rs) - milestone test file location.
- OQ-4 (default: keep version 0.1.0).
- OQ-5 (default: repeat the lifecycle smoke in this milestone package even though TASK-62 verified it at m-4) - end-to-end proof of the complete m-11 increment.

Specification: docs/specifications/technical-specifications-milestone-packaging-and-verification.md (dedicated; authored during this refinement). Feature specs: docs/specifications/technical-specifications-management.md, docs/specifications/search-technical-specifications.md, docs/specifications/browser-technical-specifications.md, docs/specifications/ui/browser-technical-specifications-ui.md (MOCK-0009 approved).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The amd64 Debian package built from the current source installs on Debian Forky (amd64) - the only supported release per ADR-0003 and NFR-03.002 - and the installed binary provides technical-specification management, search, and browser (Technical Specifications category); Debian Trixie and other releases are not claimed or verified.
- [x] #2 cargo build --release succeeds with the pinned Rust 1.99.1 toolchain and target/release/papyrus --help lists tech-spec (guards against packaging a stale binary).
- [x] #3 cargo test, cargo clippy --all-targets -- -D warnings, and cargo fmt --check pass with Rust 1.99.1, and the test counts are recorded.
- [x] #4 scripts/build-deb.sh (unchanged) produces papyrus_<version>_amd64.deb with Version equal to the Cargo package version and Architecture: amd64.
- [x] #5 dpkg-deb metadata reports Package: papyrus, Architecture: amd64, Version equal to Cargo.toml, Depends: libc6 (>= 2.34), and a copyright file declaring GPL-3.0-or-later.
- [x] #6 The package contains only /usr/bin/papyrus (0755) and /usr/share/doc/papyrus/copyright, with no asset, template, configuration, or man files, and the installed binary serves /assets/app.css and /assets/app.js from assets compiled into the binary.
- [x] #7 In an isolated Debian Forky environment, dpkg -i succeeds, /usr/bin/papyrus exists as root:root 0755, papyrus resolves on PATH, and the host package database and filesystem are unchanged.
- [x] #8 In the same isolated environment, dpkg -r (or dpkg --purge) papyrus removes the binary and papyrus no longer resolves on PATH.
- [x] #9 The installed binary smoke test passes: papyrus --help exits 0 and lists tech-spec, search, and browser, and papyrus --version exits 0 printing papyrus <version> matching the package version.
- [x] #10 The installed binary initializes a project and exercises the technical-specification lifecycle (create/list/view/edit/deprecate/restore) with expected exit codes and persisted files, preserving IDs; create requires a title and --description; edit changes only supplied fields; deprecated specifications remain viewable by ID; unknown IDs exit 3, already-deprecated/already-active conflicts and sanitization failures exit 4, missing required arguments exit 2, and a disabled tech-spec document type rejects create/list with exit 4.
- [x] #11 Installed-binary search smoke tests pass over technical specifications: a quoted phrase matches contiguously, split words do not match, --type doc includes technical specifications, other --type filters exclude them, --limit caps results, --json returns the {id,type,title,path} array (and [] when empty), deprecated specifications are hidden until restored, a missing directory yields an empty success, and --type tech-spec exits 2 as an undocumented value.
- [x] #12 Installed-binary browser smoke tests pass with --no-open on a free port: the startup line is printed, ?type=tech-spec renders the category with the Technical Specifications sidebar entry after Audit Records and aria-current, the list shows the count/rows/first-paragraph preview in ID order, a record view shows ID/form/category/title/description as body text with pre-wrap and no Description heading (or (none) when empty) and exactly one Edit action, an edit round-trip returns 303 and persists through both the UI and the CLI, a field-preserving edit changes only the heading title and ## Description while retaining the preamble and unknown sections byte-for-byte, the JSON API responds (/api/tech-specs, its /api/tech-spec alias, /api/doc/<id>), both embedded assets are served, 320 px markup hooks are present, no served page/stylesheet/script references an external origin, and deprecated specifications are not listed or counted and return 404 on view, edit, POST, and API with the identical 404 page, help text, and single recovery link.
- [x] #13 Deprecated technical specifications are hidden consistently across list, search, and browser; after restore, they reappear in all three surfaces with the same ID and unchanged content.
- [x] #14 Offline operation is verified with no internet access (isolated network namespace, or the documented fallback evidence): search and browser list/view/edit checks pass using only loopback and locally served assets.
- [x] #15 Browser configuration keys work in the installed binary: config list/get/set/unset expose browser_host, browser_port, and browser_open, a configured port is used when --port is absent, --port overrides it, defaults are restored on unset, and invalid values exit 4 without changing the file.
- [x] #16 A milestone-level integration test in tests/technical_specifications_smoke.rs exercises the technical-specification lifecycle, search, and browser together end-to-end against the built binary, passes under cargo test, and does not duplicate the contract coverage in tests/technical_specifications_cli.rs, tests/search_cli.rs, or tests/browser_cli.rs.
- [x] #17 docs/packaging.md states Debian Forky only (amd64), explicitly disclaims Trixie and other releases, and records the isolated install/uninstall procedure, the extended smoke-test checklist, and observed results with environment and date including timezone.
- [x] #18 Verification evidence (environment, date/timezone, package file and size, quality-gate results with test counts, smoke-test outcomes, deviations) is recorded in the TASK-59 ticket notes.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 All acceptance criteria verified and recorded in the ticket notes and docs/packaging.md.
- [x] #2 cargo build --release, cargo test, cargo clippy --all-targets -- -D warnings, and cargo fmt --check pass with Rust 1.99.1.
- [x] #3 The .deb metadata and contents match the specification, and the package installs and uninstalls cleanly in an isolated Debian Forky environment with the host untouched.
- [x] #4 The installed-binary smoke checklist passes (technical-specification lifecycle, search, browser, deprecated visibility, offline operation, config keys, embedded assets).
- [x] #5 The new tests/technical_specifications_smoke.rs passes and the full existing test suite remains green.
- [x] #6 docs/packaging.md is updated with the Forky-only claim, procedures, checklist, observed results (date with timezone), and any recorded limitation.
- [x] #7 No requirements, business rules, ADRs, packaging scripts, or UI were changed; any product bug fix is recorded separately in the notes.
<!-- DOD:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Run cargo build --release with Rust 1.99.1 and confirm target/release/papyrus --help lists tech-spec (stale-binary guard).
2. Run cargo test, cargo clippy --all-targets -- -D warnings, and cargo fmt --check; record test counts.
3. Run scripts/build-deb.sh unchanged; verify .deb metadata, contents, and copyright with dpkg-deb -I/-c.
4. Create an isolated dpkg root/admin under ./.tmp; install, verify /usr/bin/papyrus root:root 0755 and PATH resolution, uninstall/purge, and confirm the host is unchanged; remove scratch directories.
5. Smoke the installed binary: init; tech-spec lifecycle and exit codes; search over technical specifications; browser list/view/edit/field preservation/API/assets/320 px/no external refs/deprecated 404; config keys; embedded assets from an empty working directory.
6. Repeat the search and browser checks inside unshare -rn (or record the documented fallback evidence).
7. Add tests/technical_specifications_smoke.rs with the two milestone-level end-to-end tests and run cargo test.
8. Update docs/packaging.md with the TASK-59 Forky-only section, procedures, checklist, and observed results.
9. Record evidence in the ticket notes, verify every AC and DoD item, and move the ticket to Validated for quality review.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
### Refinement (2026-11-07 UTC)
- Completed scope (in/out), assumptions, dependencies, 18 testable acceptance criteria, 7 definition-of-done items, implementation plan, references, and open questions.
- Authored the dedicated specification: docs/specifications/technical-specifications-milestone-packaging-and-verification.md (fills the missing-dedicated-spec gap observed in TASK-46/TASK-50).
- Dependencies verified complete: TASK-55 (management), TASK-56 (search), TASK-57 (browser), TASK-58 (design review; MOCK-0009 approved).
- Scope boundary: TASK-62 (m-4 CLI packaging) already verified technical-specification management at the CLI level but explicitly deferred search and browser to TASK-59; TASK-59 re-verifies the lifecycle in the installed m-11 package for end-to-end proof.
- OQ-1 (markup/asset-level browser verification), OQ-2 (fakeroot isolated install), OQ-3 (new smoke test file), OQ-4 (keep version 0.1.0), and OQ-5 (repeat lifecycle smoke) are non-blocking and have defaults in the specification.
- No implementation performed; no source, packaging script, docs/packaging.md, or UI changes made during refinement.

### Verification (2026-11-07 UTC)
- Environment: Debian GNU/Linux forky/sid amd64, glibc 2.43-7, dpkg/dpkg-deb 1.23.11, rustc/cargo 1.99.1, clippy 0.1.99, fakeroot 2.1.4; `unshare -rn` with `ip link set lo up` available.
- Package: `papyrus_0.1.0_amd64.deb`, 918176 bytes, SHA-256 `eaabd7cc16a2bce1ea1c3675d3e955d2abd33f7756fe67133ffa0a3c12eb3498`, built with `cargo build --release` then unchanged `scripts/build-deb.sh`.
- Metadata: `Package: papyrus`, `Version: 0.1.0`, `Architecture: amd64`, `Depends: libc6 (>= 2.34)`, `Section: utils`, `Priority: optional`, copyright `GPL-3.0-or-later`.
- Contents: `/usr/bin/papyrus` (0755, 2888128 bytes) and `/usr/share/doc/papyrus/copyright` (0644, 865 bytes) only; no asset, template, configuration, or man files. `/assets/app.css` and `/assets/app.js` are embedded and served from the binary.
- Quality gates: `cargo build` PASS; `cargo build --release` PASS; `target/release/papyrus --help` lists `tech-spec`, `search`, `browser`; `cargo test` PASS — 383 tests (70 unit + 313 integration, including new `tests/technical_specifications_smoke.rs` 2); `cargo clippy --all-targets -- -D warnings` PASS — 0 warnings; `cargo fmt --check` PASS.
- New test file: `tests/technical_specifications_smoke.rs` (2 tests): `milestone_technical_specifications_flow` and `installed_layout_has_no_runtime_asset_dependency`, both passing.
- Isolated install: `fakeroot dpkg --instdir=./.tmp/task59/inst --admindir=./.tmp/task59/adm --force-depends -i` exit 0; installed binary `root:root` 0755; resolves on the installed PATH and prints `papyrus 0.1.0`.
- Isolated uninstall: `dpkg -r papyrus` exit 0, binary removed and not resolvable with the isolated-only PATH; `dpkg --purge papyrus` exit 0, admin entry gone, no papyrus files remain under the isolated root.
- Host unchanged: host package status was `install ok installed 0.1.0` before and after (pre-existing install from prior work). Scratch directories under `./.tmp/task59/` were removed after verification.
- Installed-binary smoke: lifecycle create/list/view/edit/deprecate/restore with ID preservation; exit codes 3 (unknown), 4 (already-deprecated/already-active, sanitization, disabled type), 2 (missing arguments); search quoted phrase, split words no match, `--type doc`, other type filters, `--limit`, `--json`, `[]`, deprecated hidden/restored, missing directory empty success, `--type tech-spec` exit 2 — all PASS.
- Installed-binary browser (`--no-open`, free port): exact startup line; `/?type=tech-spec` with count/rows/preview, sidebar position after Audit Records and `aria-current`; view without a Description heading; `(none)` for an empty description; single Edit action; edit round-trip `303` + `Location` persisted through UI and CLI; byte-identical preamble/unknown sections after a field-preserving edit; `/api/tech-specs`, `/api/tech-spec`, `/api/doc/TECH-001`; both embedded assets non-empty with `Cache-Control: no-store`; 320 px hooks; no external references; CSP present — all PASS.
- Deprecated specification: hidden from list, search, and count; `404` on view, edit, form POST, and API; deprecated and unknown HTML 404s byte-identical, with the documented help text and exactly one recovery link — PASS.
- Offline: inside `unshare -rn` with loopback up, an outbound TCP connection failed; search, browser list/view, and both assets passed over loopback — PASS.
- Config keys: `browser_host`, `browser_port`, `browser_open` list/get; configured port used without `--port`; `--port` overrides; `browser_open=false` suppresses opening; unset restores defaults; invalid values (0, 70000, `maybe`) exit 4 with an unchanged file — PASS.
- Embedded assets: search and browser worked from an empty working directory and an asset-free project; both assets non-empty — PASS.
- Documentation: `docs/packaging.md` updated with the Forky-only claim, TASK-59 build/package and isolated install/uninstall procedures, extended smoke checklist, observed results, limitations, and package artifact.
- Deviations: none in product behavior. The host already had `papyrus 0.1.0` installed, so the post-removal PATH check used the isolated-only PATH; the host status was recorded before and after and was unchanged. No product bug fix was required. No requirements, business rules, ADRs, packaging scripts, or UI files were changed.
- Open limitation: OQ-1 remains markup/asset-level browser verification only (no real-GUI browser run).

- Offline edit follow-up: a second isolated run inside `unshare -rn` confirmed the installed binary's browser edit round-trip offline (GET edit, POST returned 303 with `Location: /doc/TECH-001`, and the on-disk title/description were updated) — PASS.
<!-- SECTION:NOTES:END -->
