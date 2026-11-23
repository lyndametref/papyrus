---
id: TASK-115
title: Resolve record descriptors from the artifact registry as the single source
status: Done
assignee:
  - rust-dev
created_date: '2026-11-14T20:30:00Z'
updated_date: '2026-11-15T09:46:00Z'
labels:
  - rust
  - refactor
  - record
milestone: MS-019
dependencies:
  - TASK-114
references:
  - >-
    docs/architecture/adr/ADR-0017-artifact-category-registry-and-type-descriptors.md
documentation:
    docs/architecture/adr/ADR-0017-artifact-category-registry-and-type-descriptors.md
  - src/record/descriptor.rs
  - src/document.rs
  - docs/specifications/registry-sourced-record-descriptors.md
  - docs/requirements/business/REQ-007.001.md
  - docs/requirements/business/REQ-007.002.md
  - docs/requirements/business/REQ-007.003.md
  - docs/requirements/business/REQ-007.004.md
  - docs/requirements/business/REQ-005.007.md
  - docs/requirements/business/REQ-005.010.md
priority: high
ordinal: 41220
---


## Description
**Why.** After TASK-119 every document command is descriptor-driven, but the record layer still keeps a second, parallel descriptor table. `record::descriptor::descriptor_for_doc_type` is called from nine typed modules (26 call sites), `src/document.rs`, `src/init.rs`, and `src/record/migrate.rs`, and `src/registry.rs` carries a test that only asserts the two tables agree. Two hand-written sources can drift, and a config-defined type is described by a leaked runtime descriptor instead of its registry entry. TASK-117 recorded the duplication; the owner chose to refactor rather than amend ADR-0017.

**What.** Make the artifact registry the only lookup for a document type's record-level descriptor and delete the second lookup. Record data (record type, type name, label, display ID pattern, directory, subpath, doc-type key, front-matter keys, sections) is carried by the same `DescriptorTemplate` in `src/registry.rs` that already owns the type. `ArtifactRegistry::record_descriptor(slug)` projects it into the `record::descriptor::RecordDescriptor` the record layer consumes. The four backlog-family descriptors (task, milestone, release, decision) stay static and unchanged.

**In scope.**
- `src/registry.rs`: extend `DescriptorTemplate` with record fields, store a `RecordDescriptor` on every document `TypeDescriptor`, and add the single `record_descriptor(slug)` accessor.
- `src/record/descriptor.rs`: delete `descriptor_for_doc_type` and the ten static document entries; keep the four backlog entries and the generic constructor used until TASK-121.
- `src/document.rs`: `record_descriptor_for` resolves through the registry only and is removed once the registry lookup is inlined into `document::run`.
- Nine typed modules (`requirement`/req+nfr, `glossary`, `rule`, `ui_spec`, `audit`, `tech_spec`, `use_case`, `test_case`, `test_report`): receive the registry record descriptor from `dispatch_typed` instead of calling the deleted lookup. `ui_mockup` is unchanged.
- `src/init.rs`: create document-type directories from registry document descriptors.
- `src/record/migrate.rs`: `managed_descriptor` and `migrate_corpus` iterate the four backlog descriptors plus registry document descriptors; the `descriptor_for_doc_type` skip is removed.
- `src/registry.rs` tests: replace the agreement assertion with the single-source invariant tests.

**Out of scope.**
- Removing the process-global generic-descriptor cache and slug leak (TASK-121). `generic_document_descriptor` may remain as the registry-backed constructor for config-defined types.
- Rewriting backlog-family record descriptors or the modules that use them (`task.rs`, `milestone.rs`, `release.rs`, `import.rs`).
- Changing record file layout, front-matter keys, ID shapes, statuses, migration outcomes, CLI output, JSON, or help text.
- Editing requirements, business rules, specifications, or ADRs.

**Assumptions.**
- Assumption: record-level facts belong on the registry's `DescriptorTemplate`; no new module or parallel table is introduced. This is consistent with ADR-0017 decisions 1, 4, and 7.
- Assumption: `record::descriptor` keeps its `RecordDescriptor` type and the four backlog-family static values; only document entries move to the registry.
- Assumption: the legacy `deprecated/` folder is inert and is not read, scanned, or migrated (ADR-0022 decision 12; TASK-107). Existing behavior must be preserved, not expanded to read it.
- Assumption: `RecordDescriptor.name` is a migration key, not a label. Values are copied verbatim so `migrate.rs` branches, especially `"audit record"`, keep their exact behavior.
- Assumption: `ui-mockup` has no shared record descriptor today and keeps the TASK-119 archive/delete exception keyed on `ContentModel::HtmlMockup`.
- Assumption: no new dependency, configuration key, command, or user-visible behavior is introduced; ADR-0016/0017 remain unchanged.

**Canonical design.** `docs/specifications/registry-sourced-record-descriptors.md` contains the field-source matrix, call-site migration table, invariant tests, test plan, and risks. The plan below is self-contained for implementation.

## Acceptance Criteria
- [x] No production call site uses `record::descriptor::descriptor_for_doc_type`; the function is removed. The nine typed modules (`requirement`/req+nfr, `glossary`, `rule`, `ui_spec`, `audit`, `tech_spec`, `use_case`, `test_case`, `test_report`), `src/document.rs`, `src/init.rs`, and `src/record/migrate.rs` obtain a document type record descriptor from `ArtifactRegistry::record_descriptor`; `rg descriptor_for_doc_type src/` returns no matches.
- [x] `record_descriptor_for` is deleted from `src/document.rs`; `document::run` resolves the record once through the registry and passes it to generic and typed handlers. `generic_document_descriptor` remains only as the registry-backed constructor for config-defined types and is not called by a module that has a registry.
- [x] Registry descriptors carry or derive every record-level field a typed module consumes: record name/type, type name, label, display ID pattern, directory, subpath, doc-type key, front-matter keys, and sections. The field-source matrix in the specification is implemented, and `src/record/descriptor.rs` keeps no document entry or per-slug literal parallel to the registry.
- [x] `record::descriptor::DESCRIPTORS` contains only the four backlog families (task, milestone, release, decision); no entry has `doc_type_key.is_some()`. The registry test asserts totality (every document descriptor has a record), field agreement, and absence of a parallel document table, rather than comparing two independent tables.
- [x] Existing behavior is preserved: native/archived reads, inert `deprecated/` folders (not read, scanned, or migrated), files without front matter, disabled types, migration outcomes, CLI text/JSON/exit codes, and search/browser results are unchanged. `tests/migrate_records_cli.rs`, `tests/release_native_layout_smoke.rs`, `tests/document_type_no_drift.rs`, `tests/no_drift.rs`, the typed CLI suites, and the search/browser suites pass.
- [x] A test declares a config-defined generic type (for example `note`) and asserts that it and a built-in typed type (for example `glossary`) both resolve their own record fields through `ArtifactRegistry::record_descriptor`: doc-type key, prefix, directory, status front-matter key, and sections.
- [x] `src/init.rs` creates document-type directories from enabled registry document descriptors, and `src/record/migrate.rs` iterates the four backlog descriptors plus registry document records in `managed_descriptor` and `migrate_corpus`. `src/ui_mockup.rs` is unchanged and keeps the `ContentModel::HtmlMockup` archive/delete exception.
- [x] `cargo build`, the full `cargo test`, and `cargo clippy --all-targets -- -D warnings` pass. No dependency, configuration key, command, verb, status, record layout, or user-visible behavior is added or removed; ADR-0016/0017 and existing specifications are unedited.

## Definition of Done
- [x] #1 `cargo build`, the full `cargo test`, and `cargo clippy --all-targets -- -D warnings` are clean.
- [x] #2 No new dependency, configuration key, command, verb, status, record layout, or user-visible behavior is introduced; ADR-0016/0017 and existing specifications are not edited.
- [x] #3 The final summary lists every replaced call site (file + function), every new `DescriptorTemplate` record field, the removed static document entries, the new registry accessor, and states that only `generic_document_descriptor` remains as a non-registry constructor pending TASK-121.
- [x] #4 `tests/document_type_no_drift.rs`, `tests/migrate_records_cli.rs`, `tests/release_native_layout_smoke.rs`, `tests/no_drift.rs`, and the new single-source invariant tests pass.
- [x] #5 `rg descriptor_for_doc_type src/` returns no matches and `record::descriptor::DESCRIPTORS` has exactly four entries, all without `doc_type_key`.
- [x] #6 `docs/specifications/registry-sourced-record-descriptors.md` matches the implemented field-source matrix and call-site migration; no requirement, business rule, or ADR is edited.


## Implementation Notes
# Implementation Plan

Canonical design: `docs/specifications/registry-sourced-record-descriptors.md`. Requirements: REQ-007.001/002/003/004, REQ-005.007/010. ADRs: 0017, 0018, 0022, 0013.

## 1. Capture the current contract

1. Before editing, record the exact values the ten document entries in `src/record/descriptor.rs` expose for `name`, `record_type`, `type_name`, `label`, `display_id_pattern`, `frontmatter_keys`, and `sections`. The migration branches in `src/record/migrate.rs` depend on `desc.name`, especially `"audit record"`, `"release"`, `"task"`, and `"milestone"`.
2. Confirm the section arrays in `src/registry.rs` (`REQUIREMENT_SECTIONS`, `GLOSSARY_SECTIONS`, `RULE_SECTIONS`, `UI_SPEC_SECTIONS`, `AUDIT_SECTIONS`, `TECH_SPEC_SECTIONS`, `USE_CASE_SECTIONS`, `TEST_CASE_SECTIONS`, `TEST_REPORT_SECTIONS`) are byte-identical to the arrays in `src/record/descriptor.rs`; they are the replacement source for `RecordDescriptor.sections`.

## 2. Extend the registry template and descriptor

1. In `src/registry.rs`, add to `DescriptorTemplate`: `record_name`, `record_type`, `type_name`, `display_id_pattern`, and `record_frontmatter_keys`.
2. Fill them from the current static document entries, including:
   - `req`: name `"business requirement"`, record_type `"requirement"`, type_name `"requirement"`, pattern `"REQ-###.###"`, keys `&["status"]`.
   - `nfr`: name `"non-functional requirement"`, record_type `"nfr"`, type_name `"nfr"`, pattern `"NFR-###.###"`, keys `&["status"]`.
   - `audit`: name `"audit record"`, keys `&["status", "type"]`.
   - `ui-mockup`: keys `&[]`; sections `&[]`; it still receives a record descriptor for registry totality.
3. Add `record: Option<RecordDescriptor>` to `TypeDescriptor`. Backlog and decision descriptors set `None`; document descriptors set `Some`.
4. In `document_descriptor()`, build the record for a template with: `name = template.record_name`, `record_type`, `type_name`, `label = template.label`, `display_id_pattern`, `directory = "doc"` (the `doc_directory` key), `subpath = ""`, `doc_type_key = Some(template.slug)`, `frontmatter_keys = template.record_frontmatter_keys`, `sections = schema.sections` (`ContentModel::Typed` or `HtmlMockup`).
5. For a config-defined type with no template, build the generic record with `generic_document_descriptor(slug)` (copy it into the descriptor), `doc_type_key = Some(slug)`, keys `&["status"]`, sections `&["## Description"]`. Do not remove the generic cache/leak; TASK-121 owns that.
6. Add `ArtifactRegistry::record_descriptor(&self, slug: &str) -> Option<&RecordDescriptor>` returning the stored record. This is the only slug-to-record accessor.

## 3. Delete the parallel document table

1. Remove `descriptor_for_doc_type` from `src/record/descriptor.rs`.
2. Remove the ten document entries from `DESCRIPTORS`; keep task, milestone, release, and decision. Delete the now-unused document section constants and any imports they alone used.
3. Update `descriptor_count_excludes_ui_mockup` to assert four backlog entries and that no entry has `doc_type_key.is_some()`.
4. Keep `generic_document_descriptor` as the documented constructor for config-defined types; do not change its cache or leak.

## 4. Migrate the call sites (one module per change unit)

Recommended order: simplest typed module first, requirement last, then shared plumbing.

1. `src/document.rs`: resolve the record once in `document::run` via `registry.record_descriptor(&descriptor.slug)`. Pass `&RecordDescriptor` into `create_generic`, `list_records`, `view_record`, `edit_generic`, `archive_record`, `delete_record`, and `status_record`; delete `record_descriptor_for`. A missing record is an internal runtime error (`AppError::Runtime`).
2. `src/document.rs` `dispatch_typed`: resolve the record once and pass it to every typed handler alongside `descriptor`, `scope`, `verb`, and `args`.
3. For each of `tech_spec`, `use_case`, `test_case`, `test_report`, `glossary`, `ui_spec`, `audit`, `rule`, `requirement`: add the record parameter to `handle`, delete the module-local `*_dotted()` helper, and replace its body/callers with `record.is_dotted_id()`. Keep all section mapping, validation, body parsing, and patch construction untouched.
4. `src/ui_mockup.rs`: no change. HTML records have no shared record descriptor; archive/delete keep the TASK-119 exception keyed on `ContentModel::HtmlMockup`.
5. `src/init.rs`: build the registry from the effective config, iterate enabled `ArtifactCategory::Document` descriptors, and create each resolved record directory. Keep `descriptor_for` for task/milestone/release/decision directories.
6. `src/record/migrate.rs`: remove the `descriptor_for_doc_type` skip. In `managed_descriptor`, iterate the four backlog descriptors plus `registry` document record descriptors; return an owned `RecordDescriptor` (it is `Copy`) so the registry can be dropped. Adapt callers in `store.rs`, `resolve.rs`, and `migrate.rs` to borrow the owned value. In `migrate_corpus`, iterate the same two sets. Preserve the existing enabled-type and stem/directory matching rules.
7. `src/search.rs` and `src/browser/*`: no change required; they already iterate registry descriptors and call `descriptor_for` only for backlog families.

## 5. Preserve legacy and no-front-matter reads

1. Do not add a `deprecated/` read path. `resolve_read_directories` remains active + `archived/` (+ task/milestone legacy alias), per ADR-0022 decision 12.
2. Verify `managed_descriptor` still returns the same record for: a native file in the active directory, a file in `archived/`, a file with no front matter, and a file in a configured generic-type directory.
3. Verify `ensure_front_matter`/`adoption_ops` seed the same keys and order as today: general keys, then `status` (audit also `type`), using the config default or `active`. The audit branch must still key on `record_name == "audit record"`.
4. Disabled document types remain excluded from init directory creation and migration; command dispatch already rejects them before any record lookup.

## 6. Single-source invariant tests

Replace `registry_agrees_with_record_descriptor_for_default_documents` with:

1. Totality: every document descriptor has `record_descriptor(slug).is_some()`; count equals document descriptor count; `deploy` (undeclared) and an unknown slug return `None`.
2. Field agreement per document descriptor: `doc_type_key == Some(slug)`, `label` matches, `configured_prefixes(cfg) == [descriptor.prefix]`, `is_dotted_id()` matches `id_shape != Plain`, `status` appears in `frontmatter_keys` exactly when the status model is front-matter-backed, `sections` equals the typed schema sections (or the generic/HTML default), and `resolve_directory` equals `doc_directory.join(descriptor.directory.relative)`.
3. No parallel document table: `record::descriptor::DESCRIPTORS` has no `doc_type_key.is_some()` entry; add a no-drift scan that forbids `descriptor_for_doc_type` and `doc_type_key: Some(` in production lines of `src/record/descriptor.rs` (mirroring `tests/document_type_no_drift.rs` style).
4. Built-in/config-defined parity: declare `note` as a generic type in a test config; assert `glossary` and `note` each resolve through `record_descriptor` with their own slug, prefix, directory, status key, and sections.

## 7. Verification

1. `cargo build`.
2. `cargo test` (full suite), specifically:
   - `tests/document_dispatch.rs`
   - typed CLI suites: `req_cli`, `glossary_cli`, `business_rules_cli`, `ui_spec_cli`, `audit_records_cli`, `technical_specifications_cli`, `use_cases_cli`, `test_cases_cli`, `test_reports_cli`, `ui_mockup_cli`
   - `tests/migrate_records_cli.rs`, `tests/release_native_layout_smoke.rs`, `tests/document_type_no_drift.rs`, `tests/no_drift.rs`
   - search/browser suites (`tests/search_*`, `tests/browser_*`)
3. `cargo clippy --all-targets -- -D warnings`.
4. `rg descriptor_for_doc_type src/` returns no matches.
5. Add a focused regression test for a document record without front matter and for an inert `deprecated/` file remaining unmanaged.

## 8. Risks and rollback

- Migration branch drift: copy `record_name` verbatim and run migration/native-layout suites before deleting the static entries.
- Registry rebuild cost in `managed_descriptor`: build once per command/loop where practical; per-record builds are small. Caching is TASK-121.
- Hidden consumers of `DESCRIPTORS`/`RECORD_DESCRIPTORS`: the compiler finds them; update or remove the re-export with its last user.
- Typed signature churn: one module per unit, no changes to validation/section logic, rely on each module's suite.
- `ui-mockup`: leave untouched; do not route it through the shared record descriptor.
- Rollback: restore the ten static entries and `descriptor_for_doc_type`; the registry accessor is additive until the call sites are migrated, so the change can be reverted per module.

## 9. Final summary requirements

List every replaced call site (file + function), the five new template fields, the removed static entries, the new accessor, the invariant tests, and confirm that only `generic_document_descriptor` remains as a non-registry record constructor pending TASK-121.

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Registry is now the single source of document record descriptors. TypeDescriptor.record plus DescriptorTemplate/RecordFacts (record_name, record_type, type_name, display_id_pattern, frontmatter_keys) feed ArtifactRegistry::record_descriptor; generic/config-defined types use generic_document_descriptor only as the registry-backed constructor (cache/leak removal is TASK-121). Deleted descriptor_for_doc_type and the ten static document entries (DESCRIPTORS is now the four backlog families, all doc_type_key None); migrated all 26 call sites in the nine typed modules plus document.rs, init.rs and record/migrate.rs. Behavior preserved: full suite 815 passed / 0 failed / 22 ignored, legacy no-front-matter adoption, inert deprecated/, disabled types, ui-mockup untouched, golden help unchanged. Added totality, field-agreement, no-parallel-table and built-in-vs-config parity tests plus D-12 no-drift guard.
