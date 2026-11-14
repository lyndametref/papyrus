---
id: TASK-120
title: Resolve record descriptors from the artifact registry as the single source
status: To Do
assignee:
  - rust-dev
created_date: '2026-11-14T20:30:00Z'
labels:
  - rust
  - refactor
  - record
milestone: m-18
dependencies:
  - TASK-119
references:
  - >-
    docs/architecture/adr/ADR-0017-artifact-category-registry-and-type-descriptors.md
documentation:
  - >-
    docs/architecture/adr/ADR-0017-artifact-category-registry-and-type-descriptors.md
  - src/record/descriptor.rs
  - src/document.rs
priority: high
ordinal: 41220
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why.** After TASK-119 every document command is descriptor-driven, but the record layer still keeps a second, parallel descriptor table: `record::descriptor::descriptor_for_doc_type` is called from ten typed modules, `src/document.rs`, `src/init.rs`, and the legacy migration path, and `src/registry.rs` carries a test that merely asserts the two tables agree. Two sources that are only checked by a test can drift, and a config-defined type is described by a leaked runtime descriptor instead of its registry entry. TASK-117's architecture checkpoint listed this duplication; the owner chose to refactor rather than amend ADR-0017.

**What.** Make the artifact registry the only lookup for a document type's record-level descriptor and delete the second lookup. The registry descriptor must supply (or derive) the record fields the typed modules need: record type, type name, label, display ID pattern, directory, subpath, front-matter keys, and content sections. Where a static record descriptor is still needed to read legacy files, it is generated from or checked against the registry entry in one place, not selected by a second lookup.

**In scope.**

- `src/record/descriptor.rs`: remove the runtime lookup (or reduce it to migration-only data) and expose one registry-backed accessor.
- `src/document.rs`: `record_descriptor_for` resolves built-in and config-defined slugs through the registry only.
- Typed modules (`req`, `glossary`, `rule`, `ui_spec`, `audit`, `tech_spec`, `use_case`, `test_case`, `test_report`, `ui_mockup`) and `src/init.rs`: call the registry-backed accessor.
- `src/registry.rs`: replace the agreement assertion with the single-source invariant test.

**Out of scope.**

- Removing the process-global cache and slug leak (TASK-121); this task may keep `generic_document_descriptor` as the registry-backed constructor.
- Changing record file layout, front-matter keys, ID shapes, statuses, or migration outcomes.
- Editing requirements, business rules, specifications, or ADRs.

**Assume.** Reading pre-existing records (native layout, legacy `deprecated/` folders, missing front matter) must keep working unchanged; migration behavior is verified by the existing migration and native-layout suites.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 No production call site uses `record::descriptor::descriptor_for_doc_type`; the ten typed modules, `src/document.rs`, and `src/init.rs` obtain a document type record descriptor from the artifact registry
- [ ] #2 `record_descriptor_for` in `src/document.rs` resolves both built-in and config-defined slugs through registry descriptors only, and `generic_document_descriptor` is used at most as the registry-backed constructor
- [ ] #3 Registry descriptors carry or derive every record-level field a typed module consumes (record type, type name, label, display ID pattern, directory, subpath, front-matter keys, sections), so no module keeps a parallel literal for the same slug
- [ ] #4 The registry test asserts the single-source invariant — every document descriptor is reachable by slug and no record descriptor exists for a slug the registry does not declare — rather than comparing two independent tables
- [ ] #5 Existing behavior is preserved: every CLI, search, browser, migration, native-layout, and document-type no-drift test passes, including reads of legacy `deprecated/` folders and files without front matter
- [ ] #6 A config-defined type and a built-in typed type both resolve the same record descriptor fields through the registry in tests
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 `cargo build`, `cargo test`, and `cargo clippy` are clean.
- [ ] #2 No new dependency or configuration key is introduced; ADR-0016/0017 are unchanged.
- [ ] #3 The final summary lists every replaced call site and states what, if anything, remains migration-only.
- [ ] #4 `tests/document_type_no_drift.rs` and `tests/migrate_records_cli.rs` pass.
<!-- DOD:END -->
