---
id: TASK-092
title: >-
  Remove the legacy record serializers and browser write path and enforce the
  no-drift rule
status: Done
assignee: []
created_date: '2026-11-11T20:44:00Z'
updated_date: '2026-11-13T10:40:00Z'
labels:
  - native-layout
  - refactor
  - release-1.0
milestone: MS-017
dependencies:
  - TASK-083
  - TASK-084
  - TASK-087
  - TASK-088
  - TASK-089
  - TASK-090
  - TASK-091
references:
  - >-
    docs/architecture/adr/ADR-0013-shared-record-envelope-without-a-storage-layer.md
  - docs/architecture/adr/ADR-0012-papyrus-owned-front-matter-metadata.md
    docs/architecture/adr/ADR-0006-optimistic-concurrency-and-atomic-record-writes.md
    docs/specifications/remove-legacy-record-serializers-and-enforce-the-no-drift-rule.md
  - docs/specifications/browser-requirements.md
  - docs/specifications/browser-technical-specifications.md
  - docs/requirements/business/REQ-005.011.md
  - docs/requirements/business/REQ-006.004.md
  - docs/requirements/business/REQ-001.005.md
  - docs/requirements/non-functional/NFR-004.001.md
  - docs/requirements/non-functional/NFR-004.002.md
  - docs/requirements/non-functional/NFR-004.004.md
documentation:
    docs/specifications/remove-legacy-record-serializers-and-enforce-the-no-drift-rule.md
priority: high
type: feature
ordinal: 39140
---


## Description
Deliver ADR-0013 Stage 7: one parser (record::document), one mutation API (record::patch), one identity (record::identity), one writer (record::store), and one projection (record::view), with ADR-0013 decision 7 mechanically enforced by tests instead of review convention.

In scope:
- Delete src/browser/records.rs's Doc/Section model, parse_doc, per-type load_* parsers, its duplicated index/resolution algorithm, its local version token, all apply_*_edit serializers, and records::write_atomic. Thin category adapters that delegate to record::identity are permitted (decision D1); a second ID implementation is not.
- Rebuild browser reads, display, and edits on record::document, record::frontmatter, record::patch, record::resolve/descriptors, record::identity, record::store, and record::view (src/browser/mod.rs, src/browser/render.rs), including replacing the padded-ID example formatting in render.rs with record::identity.
- Normalize line endings on the browser read and save path only (decision D2): CRLF and lone CR become LF before parsing, a save writes LF with exactly one trailing newline, the first save of a CRLF file converts it, and the next identical save writes nothing. The normalization helper lives under src/record/ and is not reimplemented in src/browser/. CLI edits, import, and migration keep the shared byte-preserving contract.
- Remove the last module-local record parsing and formatting outside src/record/: the raw "## Release" section scan in src/task.rs (milestone_release), the duplicated decision-prefix matching in src/decision.rs (type_key_for_id and list filtering), and any equivalent helper surfaced by the no-drift checks.
- State the Papyrus ID only in src/record/view.rs documentation and tests (decision D3): metadata.id is the Papyrus ID, and no storage-ID wording or lowercase storage-shaped fixture remains.
- Extend tests/no_drift.rs to cover every ADR-0013 decision 7 condition and the ticket's mechanical checks: record fs::write/rename, front-matter delimiter construction, ID formatting/prefix parsing, storage-ID translation symbols and offsets, hardcoded record directories, Backlog-shaped stem detection, a second writer, and a second section parser. Exclusion is per #[cfg(test)] item with scanning resumed after it closes (decision D4), and a regression test proves a violation after a mid-file test module is reported.
- Keep every command output, JSON shape, HTML page, HTTP status code, validation message, and exit code unchanged; no UI redesign and no corpus migration command.

Out of scope: UI mockup HTML rendering/editing (ADR-0012 exception; writes already use the store), the import's Backlog.md body mapping (src/import/), search specification text (already revised), configuration/init writes, global LF normalization of every writer, TASK-98 end-to-end verification and packaging.

Assumptions:
- Assumption: the browser's non-migrating write branch only serves pre-front-matter files; routing saves through record::store makes browser saves adopt front matter per REQ-005.011. Browser reads of legacy records then adopt too, which follows the requirement and differs from the current code.
- Assumption: the browser may keep a Record-shaped render-only view model, populated from Document + RecordView, with no parsing, ID, path, version, or write logic. Category adapter functions over record::identity are permitted (D1).
- Assumption: type-specific patch builders may live in one src/record/edit module instead of one module per type.
- Assumption: src/record/descriptors.rs does not exist; src/record/descriptor.rs is canonical.
- Assumption: no new dependency; only tests asserting removed internal APIs are rewritten.
- Assumption: no corpus record is stored with CRLF, so LF normalization is verified with a test fixture and does not migrate the corpus.

Dependencies: all declared dependencies are complete (TASK-88, TASK-89, TASK-92, TASK-93, TASK-94, TASK-95, TASK-96). No new dependency. Blocks TASK-98 verification.

Rework required (quality gate 2026-11-13):
The ticket was marked Implemented, but four findings invalidate part of that claim. Decisions D1-D4 are recorded in docs/specifications/remove-legacy-record-serializers-and-enforce-the-no-drift-rule.md.
1. Adapters (D1, accepted): form_for_stem, canonical_id, canonical_key, matches_stem, parse_id_for, build_index, and resolve_id remain in src/browser/records.rs but delegate to record::identity, record::descriptor, record::resolve, and record::store. They are acceptable; AC1 is reworded to forbid reimplemented identity rather than wrapper functions. No code change is required for this finding.
2. Line endings (D2, defect): the delivered browser path does not normalize CRLF, so a CRLF record is parsed with a trailing carriage return in every heading, a section edit appends a duplicate section, line endings are mixed, and a repeated no-op save is not idempotent. Code change required.
3. Projection wording (D3, defect): src/record/view.rs line 11 and the test at line 272 still describe and assert metadata["id"] as a Backlog.md storage ID, contradicting AC8. Code, doc comment, and test change required.
4. No-drift coverage (D4, defect): tests/no_drift.rs stops scanning a file at its first #[cfg(test)] attribute, so all production lines after a mid-file test module are invisible; src/browser/render.rs keeps about 570 production lines after its test module and its tail was never scanned. Scanner and regression-test change required, and the can-fail proofs must be repeated for a post-test-module region.

Current state (verified 2026-11-13, after the quality gate): CLI create/edit paths already use frontmatter/document/patch/store; the browser has one save path through record::patch and record::store::write_atomic; src/task.rs, src/decision.rs, and src/browser/render.rs use record::document and record::identity. Open defects: browser CRLF normalization missing, src/record/view.rs storage-ID wording, item-scoped no-drift scanning missing. The feature specification is docs/specifications/remove-legacy-record-serializers-and-enforce-the-no-drift-rule.md (context and rationale; this ticket remains the implementation contract).

## Acceptance Criteria
- [x] src/browser/records.rs no longer defines Doc, Section, parse_doc, per-type load_* parsers, a duplicated version token, a duplicated index or resolution algorithm, apply_*_edit, or write_atomic; it keeps at most thin category-to-descriptor, category-to-prefix, stem-to-Form, and Index/Resolved adapters that delegate to record::identity, record::descriptor, record::resolve, or record::store and that contain no digit slicing, zero-padding, prefix string construction, or local detection predicate; browser read, display, resolution, and edit paths call record::document, record::store, record::resolve plus descriptors, record::identity, record::frontmatter, record::patch, and record::view.
- [x] Every browser save builds one PatchOp list (front-matter ops for metadata, section ops for free text, and a heading op for the body title where the type keeps one) and applies it through one record::patch::apply_patch call; a title edit patches front matter and the body heading in the same write.
- [x] All browser record writes use record::store::write_atomic; the legacy has_front_matter write branch and records::write_atomic are deleted; a save that changes no byte performs no write and does not refresh updated_date, apart from the one-time LF conversion of a CRLF record required by the line-ending criterion.
- [x] Browser concurrency and validation are unchanged: missing or malformed version is 400, a stale version is 409 with "record changed on disk; reload the form" and no write, field validation is 422, a path escape is 404, and a successful POST redirects and re-reads; unknown sections, preamble, and front-matter key order and quoting are preserved; a CRLF or lone-CR record is read with LF-normalized content so a section edit updates the existing section instead of appending a duplicate, the first save writes LF with exactly one trailing newline, and a repeated identical save writes nothing; normalization exists once under src/record/ and is not reimplemented in src/browser/.
- [x] A browser read or save of a managed record without front matter adopts front matter through the shared writer with the body unchanged apart from the added block, and a second read or save is idempotent, including after line-ending normalization; an adoption write failure is reported and leaves the file byte-identical.
- [x] No module outside src/record/ splits a managed record into ## sections, detects a leading YAML block, formats a record ID or prefix, allocates a record number, or derives a record path from a hardcoded directory; the raw ## Release scan in src/task.rs is removed; import body mapping and mockup HTML remain documented exceptions.
- [x] No module outside src/record/ writes a record file: no fs::write, fs::rename, or temporary-file plus rename sequence on a record path in production code; fs::write remains only in src/config.rs, src/init.rs, and cfg(test) fixtures.
- [x] CLI and browser JSON, HTML, routes, and links expose only the Papyrus ID; metadata.id is the Papyrus ID; src/record/view.rs module documentation, doc comments, and tests state the Papyrus ID and contain no storage-ID claim or lowercase storage-shaped metadata.id fixture; no storage-ID field, to_storage_id or to_display_id symbol, storage prefix, offset, or backlog/config.yml read remains in src/.
- [x] tests/no_drift.rs fails on each condition in its documented table: record fs::write or rename; format!("--- outside src/record/; ID formatting and prefix-shaped ID parsing outside record::identity; storage-ID and offset symbols; hardcoded record-directory literals; reimplemented Backlog-shaped stem detection; a second record writer; and ## section splitting outside src/record/ and src/import/. Patterns and allowlists are documented in the test. Exclusion applies per cfg(test) item and scanning resumes after its closing brace, so production code after a mid-file test module, including the src/browser/render.rs lines after its test module, is scanned; a regression test proves a violation located after a test module is reported and that nested cfg(test) items inside a test module do not resume scanning early.
- [x] Each no-drift check is proven able to fail: a representative violation reintroduced for each condition makes cargo test fail with file:line and the offending text, including at least one violation placed after a cfg(test) module in a file that has production code after it; the proof is recorded in implementation notes and is not left in the tree. Checks run in the default test run with no feature flag and no #[ignore].
- [x] All existing unit and integration tests pass, including tests/browser_cli.rs, migration, import, search, and no-drift tests; any test that asserted a removed internal API is rewritten to assert the same observable behavior; no corpus record file is modified.
- [x] A CRLF record fixture edited through a browser form saves as LF with exactly one trailing newline and exactly one copy of each section heading, and an immediately repeated identical save leaves the file byte-identical without refreshing updated_date.
- [x] cargo fmt --check and cargo clippy --all-targets -- -D warnings pass; no new dependency is added; the feature specification matches the delivered behavior, records decisions D1-D4, and is linked from the ticket.

## Definition of Done
- [x] #1 Implementation plan completed; the legacy browser write path and module-local record parsing outside src/record/ are deleted in production code.
- [x] #2 cargo test passes in full, including tests/browser_cli.rs, migration, import, search, and no-drift tests; results are recorded in implementation notes.
- [x] #3 cargo fmt --check and cargo clippy --all-targets -- -D warnings pass.
- [x] #4 Every no-drift condition is proven to fail when a sample violation is reintroduced, and the proof is recorded in implementation notes.
- [x] #5 A manual grep audit for fs::write, fs::rename, format!("---, apply_*_edit, parse_doc, records::write_atomic, storage-ID symbols, and hardcoded record directories is recorded in implementation notes.
- [x] #6 The feature specification and ticket references match the delivered behavior; acceptance criteria are checked with evidence and the final summary is written.
- [x] #7 git status shows no modified managed record or corpus file, and Cargo.toml has no new dependency.
- [x] #8 A browser integration test covers a CRLF fixture: the first save normalizes to LF with exactly one trailing newline and no duplicate section, and an immediately repeated save performs no write.
- [x] #9 tests/no_drift.rs proves its own coverage: a violation placed after a cfg(test) module is reported, and a production line after the src/browser/render.rs test module is scanned.


## Implementation Notes
1. Baseline and safety net: run cargo test, cargo fmt --check, and cargo clippy; record the current pass/fail state. Add characterization tests for every browser edit route (HTML and JSON) covering success redirect/200, 400/409/422/404, no-op saves writing nothing, preservation of unknown sections and preamble, front-matter key order and quoting, LF normalization, and the mockup HTML edit.
2. Browser read foundation: replace the browser Doc/Section model and parse_doc with record::document::Document; obtain version tokens from record::store; replace local index/resolution with the descriptors, record::resolve, and record::store::read; delete valid_id, matches_*_id, active_dir, record_canonical_id, and the per-type load_* parsers; keep form_for_stem, canonical_id, canonical_key, matches_stem, build_index, and resolve_id only as thin delegations to record::identity, record::descriptor, record::resolve, and record::store with no digit or prefix logic of their own (D1); populate the existing render fields from Document plus RecordView and front matter.
3. Browser edit foundation: add type-specific patch builders under src/record/edit that turn the existing form fields into PatchOp lists (front-matter keys for metadata, section ops for free text, heading op for body titles); delete apply_*_edit, section_ops, set_section, remove_section, and render_list_items; keep the mockup HTML editor as the documented exception.
4. Rewire browser routes: validate form values, then check the version token, then build and apply one patch, compare with the stored source, and call record::store::write_atomic once; delete the has_front_matter dual branch and records::write_atomic; keep every status code, message, redirect, and JSON shape.
5. CLI and render cleanup: replace the raw ## Release section scan in src/task.rs (milestone_release) with the Document/front-matter API; replace decision-prefix matching in src/decision.rs with record::identity or the descriptor; replace the padded-ID example formatting in src/browser/render.rs with record::identity; sweep the remaining modules for local section parsing, ID formatting or parsing, hardcoded record paths, and dead legacy helpers, and remove them.
6. No-drift enforcement: extend tests/no_drift.rs with the eight-condition table, documented allowlists, item-scoped cfg(test) exclusion, and file:line reporting; prove each check by temporarily reintroducing a representative violation.
7. Documentation: confirm the revised feature specification matches the delivered behavior, update ticket references, and confirm the search specifications already state that front matter is indexed.
8. Verification: run cargo fmt --check, cargo clippy --all-targets -- -D warnings, and the full cargo test suite; run the manual grep audit; confirm git status shows no corpus changes; record all evidence in implementation notes. If the browser change is too large for one review, land it as the ordered slices above with tests green after each slice; do not split the ticket.
9. Rework D2 (line endings): add one normalization helper under src/record/ that converts CRLF and lone CR to LF and guarantees exactly one trailing newline; use it on the browser read path before parsing and on the browser save path before the no-op comparison, and compare against the raw bytes read from disk so a line-ending-only difference produces exactly one write. Do not change record::document::parse, record::patch, or record::store::write_atomic for every caller. Add browser integration coverage for a CRLF fixture (first save converts, second save writes nothing, one copy of each section heading).
10. Rework D3 (projection): rewrite the src/record/view.rs module documentation and the test at line 272 so metadata.id is the Papyrus ID and no storage-ID wording or lowercase storage-shaped fixture remains.
11. Rework D4 (no-drift coverage): make production_lines exclude per cfg(test) item by tracking brace depth from the annotated item, resume scanning after it closes, and add a regression test proving a violation after a mid-file test module is reported and that the src/browser/render.rs tail is scanned. Repeat the can-fail proofs with at least one violation placed after a test module.
12. Re-verify: cargo test, cargo fmt --check, cargo clippy --all-targets -- -D warnings, the manual grep audit including storage-ID wording in src/record/view.rs, and git status; record the results and the D1-D4 outcomes in implementation notes before re-checking the acceptance criteria.


## Notes
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

- Browser saves of pre-front-matter records now adopt front matter (REQ-005.011). `tests/browser_cli.rs` `t57e_field_preserving_write` and `t81f_field_preserving_write` were updated to assert the documented adoption while keeping the body byte-identical apart from the edited fields.
- `record::migrate::ensure_front_matter_quiet` was added so adoption from a protocol stdout stream (the browser) does not print a progress line; CLI read adoption still prints.
- No new dependency; `Cargo.toml`/`Cargo.lock` unchanged. No corpus record changed; `git status` shows only source, tests, spec, and this task file.

### Final verification update

- `cargo test`: 606 passed, 0 failed, 0 ignored across 38 suites (browser_cli is now 159 tests, including the new TASK-97 adoption idempotence test).
- `cargo fmt --check`: pass.
- `cargo clippy --all-targets -- -D warnings`: pass.
- `tests/no_drift.rs`: 8 passed after all refactors.
- Browser discovery/resolution calls `record::resolve::resolve_in_dirs` per descriptor family, with the documented UI-mockup HTML exception and requirement group fallback.

### Quality-gate rework (2026-11-13)

Status moved from Implemented back to Refined. Four findings from the quality gate and their decisions:

1. Delegating adapters accepted (D1). form_for_stem, canonical_id, canonical_key, matches_stem, parse_id_for, build_index, and resolve_id stay in src/browser/records.rs as thin wrappers over record::identity, record::descriptor, record::resolve, and record::store. The earlier claim that all local index/ID helpers were deleted was inaccurate; AC1 now forbids reimplemented identity instead of wrapper functions. No code change for this finding.
2. Browser LF normalization is missing (D2, defect). record::document::parse keeps \r in headings on CRLF input, so exact heading comparison in record::patch appends a duplicate section, output mixes line endings, and a repeated no-op save is not idempotent. Decision: normalize on the browser read and save path only, with one helper under src/record/; the shared parser, patch, and store keep their byte-preserving contract.
3. src/record/view.rs still documents and asserts metadata["id"] as a storage ID (D3, defect) at line 11 and in the test at line 272. Decision: the projection states the Papyrus ID only.
4. tests/no_drift.rs stops scanning at the first #[cfg(test)] attribute, so the roughly 570 production lines after the src/browser/render.rs test module (lifecycle_page, config_page, inventory_page, change, and others) were never scanned (D4, defect). Decision: item-scoped exclusion with resumed scanning plus a regression test, and the can-fail proofs repeated for a post-test-module region.

Acceptance criteria: #2, #3, #6, #7, #11 remain satisfied by the delivered work. #1, #4, #5, #8, #9, #10, #12, #13 were unchecked as reworded or unmet. DoD #1-#6 unchecked pending rework evidence; DoD #8 and #9 added.

### Rework D2-D4 (2026-11-13)

**D2 - browser LF normalization**

- Added `src/record/line_endings.rs`: `unify` (CRLF and lone CR to LF) and `normalize` (unify plus exactly one trailing newline), with 5 unit tests.
- Added `record::store::read_document_normalized`, returning `NormalizedRead { document, version, stored_source }`. `record::document::parse`, `record::patch`, and `record::store::write_atomic` are unchanged for CLI/import/migration.
- `src/browser/records.rs` loads through the normalized read; `Record.stored_source` keeps the bytes on disk. `src/browser/mod.rs` `save_ops` normalizes the patch output and `save_content` compares it with `stored_source`, so the first CRLF save writes once and the next identical save writes nothing.
- `validate_multiline` and `validate_list_items` call `line_endings::unify`, so no browser-local CRLF replacement remains.
- Regression test `tests/browser_cli.rs::t57_crlf_save_normalizes_to_lf_once`: a CRLF fixture saves as LF with one trailing newline and one `## Description`; the repeated identical save leaves the file byte-identical and the mtime unchanged.
- Can-fail proof: reverting `load_record` to `read_document` and removing the save normalization made the test fail with "read normalization strips carriage returns from headings and bodies" at tests/browser_cli.rs:5587; code restored.

**D3 - projection wording**

- `src/record/view.rs` module documentation states that `metadata.id` is the Papyrus ID.
- The fixture id is `TASK-001`, and the test is renamed `metadata_id_is_the_papyrus_id`; no storage-ID wording or lowercase storage-shaped fixture remains.

**D4 - item-scoped no-drift scanning**

- `tests/no_drift.rs` `production_lines_in` excludes each `#[cfg(test)]` item by brace depth using a string/comment/raw-string-aware brace counter and resumes after the closing brace.
- Regression tests: `scanner_reports_violations_after_a_mid_file_test_module` proves a post-module `fs::write` is reported at `fixture.rs:14` and that a nested `#[cfg(test)]` item does not resume scanning early; `scanner_covers_production_after_the_render_test_module` proves `lifecycle_page`, `config_page`, and `inventory_page` after the `src/browser/render.rs` test module are scanned.
- All 8 checks were re-proven in a post-test-module region by inserting markers in `src/browser/render.rs::lifecycle_page` after the test module at line 3614: `fs::write` 3638, `format!("---` 3639, zero-padded ID 3640, `to_storage_id` 3641, `backlog/tasks` 3642, `is_backlog_shaped` 3643, `fn write_atomic` 3644, `split("## ")` 3639. Every check failed with `file:line: text`; markers were removed (`grep "no-drift proof"` is clean).

### Verification (2026-11-13 rework)

- `cargo test`: 614 passed, 0 failed, 0 ignored across 38 suites (browser_cli 160, no_drift 10).
- `cargo fmt --check`: pass.
- `cargo clippy --all-targets -- -D warnings`: pass.
- Manual production-line audit (same item-scoped scanner as no_drift): 0 `fs::write`/`fs::rename` on record paths outside `src/config.rs`, `src/init.rs`, `src/record/`; 0 `format!("---` outside `src/record/`; 0 storage-ID symbols; 0 hardcoded record directories outside the allowlist; 0 second writer or section parser. `src/record/view.rs` contains no storage-ID wording or `task-1` fixture.
- No new dependency; `Cargo.toml`/`Cargo.lock` unchanged; `git status` shows only source, tests, spec, ticket, and the new `src/record/line_endings.rs`.


## Comments
author: @refiner
created: 2026-11-13T10:19:00Z

---

Quality-gate decisions applied (D1-D4): (1) category adapters over record::identity are acceptable, AC1/spec reworded to forbid reimplemented identity rather than wrappers; (2) LF normalization required on the browser path only, behavior specified as first save converts CRLF and second save writes nothing; (3) src/record/view.rs must state the Papyrus ID only; (4) no-drift scanning excludes per cfg(test) item with resumed scanning and a regression test, can-fail proofs repeated for a post-test-module region. Spec updated at docs/specifications/remove-legacy-record-serializers-and-enforce-the-no-drift-rule.md.

---

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Reworked TASK-97 for quality-gate decisions D2-D4. D2: added src/record/line_endings.rs (unify and normalize) and record::store::read_document_normalized; the browser parses LF-normalized content and compares a save against the stored bytes, so the first CRLF save converts once and the next identical save writes nothing; shared document/patch/store byte preservation is unchanged; a CRLF browser integration test was added. D3: src/record/view.rs now states the Papyrus ID only, with a Papyrus-ID fixture and test. D4: tests/no_drift.rs excludes each cfg(test) item by brace depth with a string/comment-aware counter and resumes scanning; regression tests prove a post-test-module violation is reported and the src/browser/render.rs tail is scanned; all eight checks were re-proven after the render test module. Verified: cargo test 614 passed, 0 failed across 38 suites; cargo fmt --check; cargo clippy --all-targets -- -D warnings. No new dependency and no corpus file changed.
