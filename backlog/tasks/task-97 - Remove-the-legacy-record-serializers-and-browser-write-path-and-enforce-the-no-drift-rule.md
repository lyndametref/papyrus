---
id: TASK-97
title: >-
  Remove the legacy record serializers and browser write path and enforce the
  no-drift rule
status: Implemented
assignee: []
created_date: '2026-11-11T20:44:00Z'
updated_date: '2026-11-13T09:55:00Z'
labels:
  - native-layout
  - refactor
  - release-1.0
milestone: m-16
dependencies:
  - TASK-88
  - TASK-89
  - TASK-92
  - TASK-93
  - TASK-94
  - TASK-95
  - TASK-96
references:
  - >-
    docs/architecture/adr/ADR-0013-shared-record-envelope-without-a-storage-layer.md
  - docs/architecture/adr/ADR-0012-papyrus-owned-front-matter-metadata.md
  - >-
    docs/architecture/adr/ADR-0006-optimistic-concurrency-and-atomic-record-writes.md
  - >-
    docs/specifications/remove-legacy-record-serializers-and-enforce-the-no-drift-rule.md
  - docs/specifications/browser-requirements.md
  - docs/specifications/browser-technical-specifications.md
  - docs/requirements/business/REQ-05.011.md
  - docs/requirements/business/REQ-06.004.md
  - docs/requirements/business/REQ-01.005.md
  - docs/requirements/non-functional/NFR-04.001.md
  - docs/requirements/non-functional/NFR-04.002.md
  - docs/requirements/non-functional/NFR-04.004.md
documentation:
  - >-
    docs/specifications/remove-legacy-record-serializers-and-enforce-the-no-drift-rule.md
priority: high
type: feature
ordinal: 39140
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Remove the remaining legacy per-type record code and the browser's private record write path so that ADR-0013 Stage 7 is delivered: one parser (record::document), one mutation API (record::patch), one identity (record::identity), one writer (record::store), and one projection (record::view). ADR-0013 decision 7 then becomes mechanically enforced by tests instead of review convention.

In scope:
- Delete src/browser/records.rs's Doc/Section model, parse_doc, per-type load_* parsers, local index/resolution and ID helpers, local version token, all apply_*_edit serializers, and records::write_atomic.
- Rebuild browser reads, display, and edits on record::document, record::frontmatter, record::patch, record::resolve/descriptors, record::identity, record::store, and record::view (src/browser/mod.rs, src/browser/render.rs), including replacing the padded-ID example formatting in render.rs with record::identity.
- Remove the last module-local record parsing and formatting outside src/record/: the raw "## Release" section scan in src/task.rs (milestone_release), the duplicated decision-prefix matching in src/decision.rs (type_key_for_id and list filtering), and any equivalent helper surfaced by the no-drift checks.
- Extend tests/no_drift.rs to cover every ADR-0013 decision 7 condition and the ticket's mechanical checks: record fs::write/rename, front-matter delimiter construction, ID formatting/prefix parsing, storage-ID translation symbols and offsets, hardcoded record directories, Backlog-shaped stem detection, a second writer, and a second section parser.
- Keep every command output, JSON shape, HTML page, HTTP status code, validation message, and exit code unchanged; no UI redesign and no corpus migration command.

Out of scope: UI mockup HTML rendering/editing (ADR-0012 exception; writes already use the store), the import's Backlog.md body mapping (src/import/), search specification text (already revised), configuration/init writes, TASK-98 end-to-end verification and packaging.

Assumptions:
- Assumption: the browser's non-migrating write branch only serves pre-front-matter files; routing saves through record::store makes browser saves adopt front matter per REQ-05.011. Browser reads of legacy records then adopt too, which follows the requirement and differs from the current code.
- Assumption: the browser may keep a Record-shaped render-only view model, populated from Document + RecordView, with no parsing, ID, path, version, or write logic.
- Assumption: type-specific patch builders may live in one src/record/edit module instead of one module per type.
- Assumption: src/record/descriptors.rs does not exist; src/record/descriptor.rs is canonical.
- Assumption: no new dependency; only tests asserting removed internal APIs are rewritten.

Dependencies: all declared dependencies are complete (TASK-88, TASK-89, TASK-92, TASK-93, TASK-94, TASK-95, TASK-96). No new dependency. Blocks TASK-98 verification.

Current state (verified 2026-11-13): CLI create/edit paths already use frontmatter/document/patch/store; browser/records.rs still owns the full legacy read/write path; browser/mod.rs still branches between store::write_atomic and records::write_atomic; src/decision.rs and src/browser/render.rs still format or match record prefixes outside record::identity; tests/no_drift.rs has only three checks and misses known violations such as format!("{}-", prefix) ID parsing in the browser. The feature specification is docs/specifications/remove-legacy-record-serializers-and-enforce-the-no-drift-rule.md (context and rationale; this ticket remains the implementation contract).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 src/browser/records.rs no longer defines Doc, Section, parse_doc, per-type load_* parsers, a duplicated index/resolve_id, valid_id, matches_*_id, form_for_stem, active_dir, record_canonical_id, version_token, apply_*_edit, or write_atomic; browser read, display, resolution, and edit paths call record::document, record::store, record::resolve plus descriptors, record::identity, record::frontmatter, record::patch, and record::view.
- [x] #2 Every browser save builds one PatchOp list (front-matter ops for metadata, section ops for free text, and a heading op for the body title where the type keeps one) and applies it through one record::patch::apply_patch call; a title edit patches front matter and the body heading in the same write.
- [x] #3 All browser record writes use record::store::write_atomic; the legacy has_front_matter write branch and records::write_atomic are deleted; a save that changes no byte performs no write and does not refresh updated_date.
- [x] #4 Browser concurrency and validation are unchanged: missing or malformed version is 400, a stale version is 409 with "record changed on disk; reload the form" and no write, field validation is 422, a path escape is 404, and a successful POST redirects and re-reads; unknown sections, preamble, and front-matter key order and quoting are preserved; line endings are LF with one trailing newline.
- [x] #5 A browser read or save of a managed record without front matter adopts front matter through the shared writer with the body unchanged apart from the added block, and a second read or save is idempotent; an adoption write failure is reported and leaves the file byte-identical.
- [x] #6 No module outside src/record/ splits a managed record into ## sections, detects a leading YAML block, formats a record ID or prefix, allocates a record number, or derives a record path from a hardcoded directory; the raw ## Release scan in src/task.rs is removed; import body mapping and mockup HTML remain documented exceptions.
- [x] #7 No module outside src/record/ writes a record file: no fs::write, fs::rename, or temporary-file plus rename sequence on a record path in production code; fs::write remains only in src/config.rs, src/init.rs, and cfg(test) fixtures.
- [x] #8 CLI and browser JSON, HTML, routes, and links expose only the Papyrus ID; metadata.id is the Papyrus ID; no storage-ID field, to_storage_id or to_display_id symbol, storage prefix, offset, or backlog/config.yml read remains in src/.
- [x] #9 tests/no_drift.rs fails on each condition in its documented table: record fs::write or rename; format!("--- outside src/record/; ID formatting and prefix-shaped ID parsing outside record::identity; storage-ID and offset symbols; hardcoded record-directory literals; reimplemented Backlog-shaped stem detection; a second record writer; and ## section splitting outside src/record/ and src/import/. Patterns and allowlists are documented in the test.
- [x] #10 Each no-drift check is proven able to fail: a representative violation reintroduced for each condition makes cargo test fail with file:line and the offending text; the proof is recorded in implementation notes and is not left in the tree. Checks run in the default test run with no feature flag and no #[ignore].
- [x] #11 All existing unit and integration tests pass, including tests/browser_cli.rs, migration, import, search, and no-drift tests; any test that asserted a removed internal API is rewritten to assert the same observable behavior; no corpus record file is modified.
- [x] #12 cargo fmt --check and cargo clippy --all-targets -- -D warnings pass; no new dependency is added; the feature specification matches the delivered behavior and is linked from the ticket.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Implementation plan completed; the legacy browser write path and module-local record parsing outside src/record/ are deleted in production code.
- [x] #2 cargo test passes in full, including tests/browser_cli.rs, migration, import, search, and no-drift tests; results are recorded in implementation notes.
- [x] #3 cargo fmt --check and cargo clippy --all-targets -- -D warnings pass.
- [x] #4 Every no-drift condition is proven to fail when a sample violation is reintroduced, and the proof is recorded in implementation notes.
- [x] #5 A manual grep audit for fs::write, fs::rename, format!("---, apply_*_edit, parse_doc, records::write_atomic, storage-ID symbols, and hardcoded record directories is recorded in implementation notes.
- [x] #6 The feature specification and ticket references match the delivered behavior; acceptance criteria are checked with evidence and the final summary is written.
- [x] #7 git status shows no modified managed record or corpus file, and Cargo.toml has no new dependency.
<!-- DOD:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Baseline and safety net: run cargo test, cargo fmt --check, and cargo clippy; record the current pass/fail state. Add characterization tests for every browser edit route (HTML and JSON) covering success redirect/200, 400/409/422/404, no-op saves writing nothing, preservation of unknown sections and preamble, front-matter key order and quoting, LF normalization, and the mockup HTML edit.
2. Browser read foundation: replace the browser Doc/Section model and parse_doc with record::document::Document; obtain version tokens from record::store; replace local index/resolution with the descriptors, record::resolve, and record::store::read; delete valid_id, matches_*_id, form_for_stem, active_dir, record_canonical_id, and the per-type load_* parsers; populate the existing render fields from Document plus RecordView and front matter.
3. Browser edit foundation: add type-specific patch builders under src/record/edit that turn the existing form fields into PatchOp lists (front-matter keys for metadata, section ops for free text, heading op for body titles); delete apply_*_edit, section_ops, set_section, remove_section, and render_list_items; keep the mockup HTML editor as the documented exception.
4. Rewire browser routes: validate form values, then check the version token, then build and apply one patch, compare with the stored source, and call record::store::write_atomic once; delete the has_front_matter dual branch and records::write_atomic; keep every status code, message, redirect, and JSON shape.
5. CLI and render cleanup: replace the raw ## Release section scan in src/task.rs (milestone_release) with the Document/front-matter API; replace decision-prefix matching in src/decision.rs with record::identity or the descriptor; replace the padded-ID example formatting in src/browser/render.rs with record::identity; sweep the remaining modules for local section parsing, ID formatting or parsing, hardcoded record paths, and dead legacy helpers, and remove them.
6. No-drift enforcement: extend tests/no_drift.rs with the eight-condition table, documented allowlists, cfg(test) exclusion, and file:line reporting; prove each check by temporarily reintroducing a representative violation.
7. Documentation: confirm the revised feature specification matches the delivered behavior, update ticket references, and confirm the search specifications already state that front matter is indexed.
8. Verification: run cargo fmt --check, cargo clippy --all-targets -- -D warnings, and the full cargo test suite; run the manual grep audit; confirm git status shows no corpus changes; record all evidence in implementation notes. If the browser change is too large for one review, land it as the ordered slices above with tests green after each slice; do not split the ticket.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
## Implementation notes (TASK-97)

Delivered ADR-0013 Stage 7 on branch task/TASK-97.

### Scope delivered

- `src/browser/records.rs` is now a render-only view model built from `record::document`, `record::view`, and front matter. Removed `Doc`, `Section`, `parse_doc`, per-type `load_*`, `valid_id`, `matches_*_id`, `form_for_stem`, `active_dir`, `record_canonical_id`, `version_token`, `apply_*_edit`, and `write_atomic`.
- Added `src/record/edit.rs`: one PatchOp builder per form and one `apply` helper. Browser handlers call one builder, then `record::patch::apply_patch`, then `record::store::write_atomic` once. Title edits patch front matter and the body heading in the same patch list.
- `src/browser/mod.rs`: one `save_ops`/`save_content` path; the `has_front_matter` legacy branch and `records::write_atomic` are gone. A no-op save performs no write and does not refresh `updated_date`.
- Browser reads use `record::store::read_document` with quiet adoption, so a pre-front-matter managed record adopts front matter on read/save and is idempotent; adoption write failure keeps the original file byte-identical through `record::store::write_atomic`.
- `src/task.rs` `milestone_release` uses `record::document::section_text`; `src/decision.rs` prefix matching uses `record::identity`; `src/browser/render.rs` `id_padding` examples use `record::identity::format_id`.
- All record-file renames outside `src/record/` moved to `record::store::move_record`; no `fs::write`/`fs::rename` remains on record paths outside config/init/store.
- `tests/no_drift.rs` has eight independent checks with documented allowlists and `file:line: text` reporting; checks run in the default test run, no feature flag, no `#[ignore]`.

### Verification

- `cargo test`: 605 passed, 0 failed, 0 ignored across 38 suites (including browser_cli, migrate_records_cli, import_backlog_cli, search_*, no_drift, record_view_projection).
- `cargo fmt --check`: pass.
- `cargo clippy --all-targets -- -D warnings`: pass.
- `tests/no_drift.rs`: 8 passed.

### No-drift can-fail proof

Each check was proven by temporarily inserting a representative comment in `src/browser/records.rs`, running `cargo test --test no_drift`, observing the failure with file:line and text, then restoring the file (`grep 'no-drift proof'` is clean):
1. `fs::write` -> `src/browser/records.rs:1295: // no-drift proof: fs::write`
2. `format!("---` -> `src/browser/records.rs:1295` with offending delimiter text
3. `format!("{}-{:0` -> `src/browser/records.rs:1295` with zero-padded ID pattern
4. `to_storage_id` -> `src/browser/records.rs:1295`
5. `backlog/tasks` -> `src/browser/records.rs:1295`
6. `is_backlog_shaped` -> `src/browser/records.rs:1295`
7. `fn write_atomic` -> `src/browser/records.rs:1295`
8. `split("## ")` -> `src/browser/records.rs:1295`

No proof violation remains in the tree.

### Manual grep audit (production lines only, cfg(test) excluded)

- `fs::write`: 0 outside `src/config.rs`, `src/init.rs`, `src/record/`.
- `fs::rename`: 0 outside `src/record/`.
- `format!("---`: 0 outside `src/record/`.
- `apply_*_edit`, `parse_doc`, `records::write_atomic`: 0.
- `to_storage_id`, `to_display_id`, `storage_prefix`, `storage_id`, `backlog/config.yml`: 0.
- Hardcoded record directories (`backlog/tasks`, `backlog/milestones`, `backlog/releases`, archives, `deprecated/`): 0 outside the documented allowlist.
- Zero-padded ID formatting and prefix-shaped ID parsing: 0 outside `src/record/identity.rs`.
- `split("## ")`, `strip_prefix("## ")`, `split_once("## "): 0 outside `src/record/` and `src/import.rs`.

### Behavior notes

- Browser saves of pre-front-matter records now adopt front matter (REQ-05.011). `tests/browser_cli.rs` `t57e_field_preserving_write` and `t81f_field_preserving_write` were updated to assert the documented adoption while keeping the body byte-identical apart from the edited fields.
- `record::migrate::ensure_front_matter_quiet` was added so adoption from a protocol stdout stream (the browser) does not print a progress line; CLI read adoption still prints.
- No new dependency; `Cargo.toml`/`Cargo.lock` unchanged. No corpus record changed; `git status` shows only source, tests, spec, and this task file.

### Final verification update

- `cargo test`: 606 passed, 0 failed, 0 ignored across 38 suites (browser_cli is now 159 tests, including the new TASK-97 adoption idempotence test).
- `cargo fmt --check`: pass.
- `cargo clippy --all-targets -- -D warnings`: pass.
- `tests/no_drift.rs`: 8 passed after all refactors.
- Browser discovery/resolution calls `record::resolve::resolve_in_dirs` per descriptor family, with the documented UI-mockup HTML exception and requirement group fallback.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Removed the legacy browser record read/write path and the remaining out-of-record parsing/formatting. Browser saves now build one PatchOp list in src/record/edit.rs, apply it through record::patch, and write once through record::store; browser reads use record::document, record::resolve, descriptors, and record::view. Fixed task.rs milestone_release, decision.rs prefix matching, and render.rs padded-ID formatting. tests/no_drift.rs now enforces eight ADR-0013 decision 7 conditions and each check was proven able to fail. Verified: cargo test 606 passed/0 failed across 38 suites, cargo fmt --check, cargo clippy --all-targets -- -D warnings, and the manual grep audit. No new dependency or corpus record change; browser save adoption of pre-front-matter records is the documented behavior change.
<!-- SECTION:FINAL_SUMMARY:END -->
