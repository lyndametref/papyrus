---
id: TASK-98
title: >-
  Verify the native layout, import, relocation, and single ID space end to end
  and package the release build
status: Implemented
assignee: []
created_date: '2026-11-11T20:44:00Z'
updated_date: '2026-11-13T11:21:00Z'
labels:
  - verification
  - packaging
  - release-1.0
milestone: m-16
dependencies:
  - TASK-97
  - TASK-103
references:
  - docs/requirements/non-functional/NFR-04.004.md
  - docs/requirements/business/REQ-01.005.md
  - docs/requirements/business/REQ-01.002.md
  - docs/requirements/business/REQ-05.010.md
  - docs/requirements/business/REQ-05.011.md
  - docs/requirements/business/REQ-06.004.md
  - docs/requirements/business/REQ-06.013.md
  - docs/requirements/non-functional/NFR-04.001.md
  - docs/requirements/non-functional/NFR-04.002.md
  - docs/requirements/non-functional/NFR-03.002.md
  - >-
    docs/architecture/adr/ADR-0013-shared-record-envelope-without-a-storage-layer.md
  - docs/architecture/adr/ADR-0010-papyrus-native-on-disk-record-layout.md
  - docs/architecture/adr/ADR-0011-one-way-backlog-md-import.md
  - docs/architecture/adr/ADR-0012-papyrus-owned-front-matter-metadata.md
  - >-
    docs/architecture/adr/ADR-0006-optimistic-concurrency-and-atomic-record-writes.md
  - docs/architecture/adr/ADR-0003-supported-debian-release-target-forky-only.md
  - docs/specifications/release-1.0-native-layout-verification-and-packaging.md
  - docs/specifications/papyrus-native-record-layout-and-single-id-space.md
  - docs/specifications/import-backlog-md-records.md
  - >-
    docs/specifications/remove-legacy-record-serializers-and-enforce-the-no-drift-rule.md
  - docs/specifications/search-complete-unified.md
  - docs/packaging.md
priority: high
type: task
ordinal: 39150
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
ADR-0013 stage 8 is the gate on any release that claims NFR-04.004, and the release also needs a package that installs. Each implementation task tested its own behaviour; nothing yet proves the combined properties the renunciation depends on: adoption that is idempotent and byte-identical, no Backlog-shaped write anywhere, one Papyrus identity on every surface, import fidelity against a real source store, relocation that never loses a folder, and search that still indexes whole persisted text now including front matter.

This ticket produces that evidence and the installable amd64 Debian package. Deliverables: an isolated sample Backlog.md project used as the only import source, automated and scripted verification of the ADR-0013 stage 8 properties, one new integration test (tests/release_native_layout_smoke.rs), the built papyrus_<version>_amd64.deb verified install/uninstall/smoke on Debian Forky amd64, and the evidence recorded in docs/packaging.md and the ticket notes.

Boundaries: this repository own backlog store stays in the Backlog.md layout and is neither an import source nor modified (its one-time import was deferred out of Release 1.0; TASK-100 is archived); requirements, business rules, ADRs, packaging scripts, and upgrade notes are not edited here; product behaviour does not change except a minimal defect fix exposed and reported by verification; only Debian Forky amd64 is claimed or verified.

Feature specification: docs/specifications/release-1.0-native-layout-verification-and-packaging.md
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 An isolated sample Backlog.md project fixture is generated for the run under ./.tmp/ with active and archived tasks and milestones, docs, drafts, decisions, config.yml, .locks, every body marker family, an unknown section, and free text; it is removed afterwards, and this repository own backlog/ store is neither used as an import source nor modified (unchanged git status and recursive hash before and after).
- [ ] #2 Adoption and no-op writes are byte-identical: adopting a record that already carries front matter twice, and applying a no-op or equal-value patch, change no byte, mode, or modification time and do not refresh updated_date; the same holds for a legacy record on first adoption.
- [ ] #3 No code path writes a Backlog.md shape: after the exercised create/edit/adopt/archive/restore/deprecate/import flows every written record path is native (<ID>.md, decisions <ID>-<slug>.md), no Backlog stem, storage ID, legacy ## Status metadata section, or storage-ID metadata key is written, and the eight tests/no_drift.rs checks pass in the same suite run.
- [ ] #4 An unimported Backlog.md store inside an initialised project is invisible to every surface (CLI list/view/search, --json forms, browser list, API, record views) while other families stay visible; after papyrus import backlog every in-scope record is readable with its allocated Papyrus ID and the source tree is byte-identical.
- [ ] #5 CLI text, CLI --json, browser JSON, browser HTML, and browser URLs expose only the Papyrus ID; no storage_id/display_id/storage_path field appears, and a Backlog ID passed as input (task view task-1, milestone view m-0) exits 3 as an unknown record.
- [ ] #6 Import fidelity matches ADR-0011 on the fixture: consecutive unique allocation from 1 in source-number order with milestones in the MS- space, field mapping including normalized status and translated milestone/dependency references, body mapping with acceptance-criteria ordinals dropped and unknown content preserved verbatim, archived destinations, and docs/drafts/decisions counted as excluded and not imported.
- [ ] #7 The documented one-time upgrade path executes as written against the isolated project (init, dry run writing nothing, import with plan/report kept, normal CLI and browser use), showing no automatic import, reallocated task IDs and milestone numbers, a kept source store, and a second run refused without --allow-existing and duplicating with it; any divergence is recorded as a finding for TASK-99/TASK-73 rather than edited here.
- [ ] #8 Relocation moves the whole folder including unmanaged files, removes the empty source, persists configuration only on success, and on a destination conflict or unwritable or out-of-project destination exits 4 moving nothing with the configuration file unchanged; the browser configuration view reaches the same config set path, and success and conflict are re-verified through the installed binary.
- [ ] #9 Search matches whole persisted text including front matter, so a recorded front-matter-only query (for example a label or assignee value) returns the record on CLI and browser; results match the documented search behaviour, and search writes no file (project hash unchanged).
- [ ] #10 cargo build --release, the full cargo test suite including the new tests/release_native_layout_smoke.rs, cargo clippy --all-targets -- -D warnings, and cargo fmt --check all pass on the pinned toolchain, with the test count recorded.
- [ ] #11 ./scripts/build-deb.sh, run unchanged after the release build, produces papyrus_<version>_amd64.deb whose control metadata shows Package: papyrus, the Cargo version, Architecture: amd64, Depends: libc6 (>= 2.34), Section: utils, Priority: optional, and GPL-3.0-or-later copyright, and whose contents are limited to /usr/bin/papyrus (0755) and the copyright file (0644).
- [ ] #12 On Debian Forky amd64 the package installs in an isolated dpkg root, the installed binary runs the import against the isolated sample and serves the browser over loopback offline (per-milestone backlog board, task record view, board status change persisted to the record, JSON API, embedded assets), then dpkg -r and --purge leave no papyrus file or admin entry and the host package state is unchanged.
- [ ] #13 docs/packaging.md and the ticket notes record the evidence: environment, date with timezone, isolation method, test count, package name/size/SHA-256, quality-gate table, install/uninstall and smoke results, and recorded limitations; the new tests/release_native_layout_smoke.rs covers the assertable portion without duplicating per-feature tests.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Feature specification docs/specifications/release-1.0-native-layout-verification-and-packaging.md is present and matches the delivered evidence.
- [ ] #2 tests/release_native_layout_smoke.rs is added and passes as part of the full suite; no per-feature contract test is duplicated.
- [ ] #3 cargo build --release, cargo test, cargo clippy --all-targets -- -D warnings, and cargo fmt --check pass on the pinned toolchain, with environment and test count recorded.
- [ ] #4 docs/packaging.md carries the TASK-98 section with procedures, gate results, install/uninstall and smoke results, package size and SHA-256, date with timezone, and recorded limitations.
- [ ] #5 Ticket notes record the same evidence plus fixture hash checks, the front-matter-only search query, relocation outcomes, and every finding raised against another ticket.
- [ ] #6 This repository own backlog/ store is verified untouched (git status and recursive hash unchanged) and no file under it was imported or rewritten.
- [ ] #7 No requirement, business rule, ADR, packaging script, or upgrade-notes file was modified; specification or requirement conflicts are escalated as open questions.
- [ ] #8 No new runtime dependency; the TASK-1/TASK-4 dependency-license audit remains the reference or a re-run is recorded.
- [ ] #9 All ./.tmp/ scratch directories, isolated dpkg roots, and staged package trees are removed; the built .deb and the recorded evidence remain.
<!-- DOD:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Create tests/release_native_layout_smoke.rs with a fixture builder under ./.tmp/: sample Backlog.md project (active/archived tasks and milestones, docs, drafts, decisions, config.yml, .locks, all body markers, unknown section, free text), helpers to init a project, hash a tree, capture CLI and browser output, and pick a free loopback port; assert the fixture inventory and repository backlog/ hash baseline.

2. Add adoption and no-op tests: read/write an already-front-matter record twice and apply a no-op/equal-value patch; compare bytes, mode, and mtime, and confirm updated_date is not refreshed; repeat for a legacy record on first adoption, including the read-only failure path leaving the file byte-identical.

3. Add the no-Backlog-shape test: run create/edit/adopt/archive/restore/deprecate and import on a scratch project, assert every written path is native, no Backlog stem, no ## Status-style task/milestone metadata section, and no storage-ID key; confirm tests/no_drift.rs runs in the same cargo test invocation.

4. Add the invisibility and import test: initialise a project, copy the fixture store into its configured backlog/ directory, assert CLI list/view/search and their --json forms plus browser list/API/views report no tasks or milestones while other families stay visible; run papyrus import backlog --from that project and assert every in-scope record is readable and the source tree hash is unchanged.

5. Add the single-ID test: capture task list, task view, milestone list, search, --json, browser HTML, /api/tasks, /api/milestones, /api/doc/<ID> and rendered links; assert only Papyrus IDs appear, no storage-ID field exists, and task view task-1 / milestone view m-0 exit 3.

6. Add import fidelity assertions against the fixture: allocation order and uniqueness, milestones in the MS- space, field mapping (status normalization, timestamps, assignee list, labels, references, documentation, type, ordinal, unknown key), milestone and dependency translation with warnings, body mapping with AC ordinals dropped and unknown content verbatim, archived destinations, and excluded docs/drafts/decisions counts.

7. Verify the upgrade path literally on the isolated project: dry run writes nothing, import keeps the plan/report, subsequent CLI and browser use works without any automatic import, re-run is refused without --allow-existing and duplicates with it; record divergences from the TASK-99 upgrade notes as findings, not edits.

8. Re-verify relocation end to end: config set backlog_directory success (whole folder incl. unmanaged files, source removed when empty), destination conflict/unwritable/out-of-project exits 4 with no move and an unchanged configuration file, multi-segment destination accepted, browser configuration view producing the same on-disk result; rely on tests/config_relocation_cli.rs for the exhaustive matrix.

9. Add the search test: query a value that exists only in front matter (label or assignee) and assert the record is returned on CLI and browser, alongside quoted-phrase, --type, --limit, --json, empty-result and archived/deprecated visibility checks; assert the project hash is unchanged after search.

10. Run the quality gates with the pinned toolchain: cargo build --release (assert --help lists import and browser and --version matches Cargo.toml), cargo test, cargo clippy --all-targets -- -D warnings, cargo fmt --check; record versions and the final test count.

11. Build and inspect the package: ./scripts/build-deb.sh unchanged after the release build; record file name, size, SHA-256; check dpkg-deb -I metadata and dpkg-deb -c contents (binary and copyright only).

12. Isolated install and installed-binary smoke on Debian Forky amd64: fakeroot dpkg --instdir=./.tmp/task98/inst --admindir=./.tmp/task98/adm --force-depends -i, confirm root:root 0755 and --version, run init plus the invisibility/import flow, serve the browser and check board sections and status columns, a task record view, a board status change persisted to the record, JSON API and both assets over loopback, plus the unshare -rn offline check; then dpkg -r and --purge and confirm no files and an unchanged host dpkg state.

13. Record the evidence in docs/packaging.md (TASK-98 section: Forky-only statement, environment, date with timezone, isolation method, test count, gate table, install/uninstall/smoke table, package size and SHA-256, recorded limitations) and in the ticket notes; confirm git status and the recursive hash of backlog/ are unchanged, raise findings against owning tickets, and remove every ./.tmp/ scratch directory.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Scope note (planner, 2026-11-11): the user deferred the import of this repository own store out of Release 1.0. Verify the import against an isolated sample Backlog.md project fixture only; this repository store is not part of the verification evidence.

Scope note (planner, 2026-11-11): TASK-103 adds the per-milestone kanban board with card status change (REQ-06.013) in m-16 at ordinal 39145, after TASK-97 and before this task. This task now depends on it, so the verification and the packaged release build include the board; board behaviour is covered by TASK-103's own tests, and this task's acceptance criteria are unchanged.

Refinement (2026-11-13): execution-ready against ADR-0013 stage 8, ADR-0010/0011/0012, NFR-04.004, REQ-01.002, REQ-01.005, REQ-05.010/011, REQ-06.004, REQ-06.013, and the linked specifications. Feature specification written at docs/specifications/release-1.0-native-layout-verification-and-packaging.md. Scope boundaries confirmed by the caller: this repository own backlog store is out of scope and stays in the Backlog.md layout; the import is verified only against an isolated sample Backlog.md project fixture; the deliverable package is the Debian amd64 build. Acceptance criteria rewritten as 13 testable checks, Definition of Done added, implementation plan recorded, references expanded. Assumptions recorded: Debian Forky amd64 host with fakeroot/dpkg-deb/unshare; package version stays 0.1.0 (open question for release owner); upgrade notes (TASK-99) are cross-checked if present but are not a blocking dependency. No implementation performed.

TASK-98 verification evidence (2026-11-13 12:21 CET, Debian Forky amd64, glibc 2.43, rustc/cargo 1.99.1, clippy 0.1.99, dpkg 1.23.11).
New test: tests/release_native_layout_smoke.rs, 9 tests, all PASS: t98_01 fixture inventory + repository-store protection; t98_02 adoption/no-op byte identity; t98_03 no Backlog-shaped write; t98_04 invisibility before import/readability after; t98_05 single ID space; t98_06 import fidelity; t98_07 documented upgrade path; t98_08 relocation atomicity/conflict/browser path; t98_09 front-matter search with no write.
Quality gates: cargo build --release PASS (32.31s); target/release/papyrus --version = papyrus 0.1.0; --help lists task/import/browser/search; cargo test PASS = 633 tests, 0 failed, 0 ignored (includes 9 new and tests/no_drift.rs eight checks); cargo clippy --all-targets -- -D warnings PASS 0 warnings; cargo fmt --check PASS.
Package: ./scripts/build-deb.sh unchanged produced papyrus_0.1.0_amd64.deb, 1,197,168 bytes, SHA-256 96acf96b672522bbd718dab94552e1e817b1743745a11ad42a5d75744a0985b2. dpkg-deb -I shows Package papyrus, Version 0.1.0, amd64, Depends libc6 (>= 2.34), Section utils, Priority optional, Maintainer Papyrus Maintainers. dpkg-deb -c lists only /usr/bin/papyrus 0755 and /usr/share/doc/papyrus/copyright 0644 (GPL-3.0-or-later).
Isolation: fixtures and scratch projects under ./.tmp/task98-* with Scratch guard cleanup; installed package under ./.tmp/task98/{inst,adm} using fakeroot dpkg --instdir/--admindir --force-depends. Installed binary root:root 0755 (fakeroot view) and prints papyrus 0.1.0. Installed init/import/browser: pre-import task list empty; import created TASK-001 and MS-001 with source kept; board rendered MS-001 and six status columns; record view HTTP 200; board status change HTTP 303 persisted status: Initiated to TASK-001.md; /api/tasks and /api/milestones 200; app.css 14,151 bytes and app.js 7,795 bytes 200. Offline unshare -rn with loopback up: outbound TCP failed; list/view/status-change(303, status: Refined)/API/assets all 200. dpkg -r removed the binary; dpkg --purge removed the admin entry with 0 files left; host dpkg -l papyrus unchanged before/after.
Fixture/source checks: fixture under ./.tmp with 5 tasks (4 active + 1 archived), 3 milestones (2 active + 1 archived), docs/drafts/decisions/config.yml/.locks, all marker families, unknown section, free text; source tree hash unchanged before import, after import, and after CLI/browser checks. Dry run wrote nothing (project hash unchanged). Repository backlog was never an import source and was not rewritten; its 116-file hash (excluding the TASK-98 ticket metadata) remains 050892ef900e7f76486f709bbcd3d1875b429ca4b4dc53be2900a9b1b191b117, with git status showing only the expected TASK-98 ticket file.
Front-matter-only search query: fm-only-label (label only in TASK-001 front matter) returned TASK-001 on CLI and in browser record view/API; quoted phrase, --type, --limit, --json, empty, archived-hidden, deprecated-hidden, restored-visible checks passed; project hash unchanged after search.
Relocation: multi-segment native/store moved unmanaged files/subfolders and removed the empty source; browser /config produced the same result; conflict, read-only destination, and out-of-project destination all exited 4 with nothing moved and byte-identical config.
Findings: F-98-1 for TASK-73/TASK-99: release spec 2.2 says a legacy record equal-value write is byte-identical, but the documented TASK-89 first-write retirement removes the legacy heading/metadata section; the test asserts documented retirement plus post-retirement identity. F-98-2 for TASK-73: no browser search endpoint exists; search evidence is CLI plus browser record view/API. Limitations: no real-GUI browser run (OQ-1); Forky amd64 only (ADR-0003); package version stays 0.1.0 (OQ-2). All ./.tmp/task98* scratch, isolated dpkg roots, and staged package trees removed; built .deb remains.
<!-- SECTION:NOTES:END -->
