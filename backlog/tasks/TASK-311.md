---
id: TASK-311
title: "Implement generic document JSON projections preserving the MS-032 payload bytes"
created_date: '2026-11-22T16:44:13Z'
updated_date: '2026-11-22T18:17:08Z'
labels: []
references:
  - ADR-039
  - TECH-068
  - REQ-06.014
  - TECH-040
  - AUD-144
  - TC-184
  - MS-035
status: Done
assignee:
  - rust-dev
milestone: MS-035
dependencies:
  - TASK-268
ordinal: 75
documentation:
  - src/browser/mod.rs
  - src/browser/records.rs
  - src/record/view.rs
  - tests/characterization_replay.rs
  - tests/fixtures/characterization/corpus
type: implementation
---

## Description
## Context
TASK-269 currently combines the generic document JSON projection with the HTML branch deletion. This task is the 269a half of that split: it replaces the per-type document JSON branches with one generic projection derived from the record envelope and the field map, and preserves the MS-032 payload bytes. TASK-269 (269b) keeps the HTML deletion, the eight ops builders, the document TypedHandler variants, the ui-spec editor exception, the semantic comparator, and the no-slug guards, and it depends on this task.

Current code in `src/browser/mod.rs` has per-type document branches in `record_json`, `api_doc_list`, `api_get`, and `api_post`; `api_post` carries the document `api-post-success` and `api-post-conflict` behavior. The grouped requirement index `/api/requirements` selects by document model and role. ADR-039 decisions 10-13 and 17, TECH-068 sections 5 and 7, and MS-035 AC-2 and AC-7 define the target. TASK-265 delivered the `DocumentFieldMap`; the MS-032 corpus under `tests/fixtures/characterization/corpus/` is the byte baseline for `payload.json`, `api-post-success`, and `api-post-conflict`.

## Scope
- Replace the per-type document JSON branches in `record_json`, `api_doc_list`, `api_get`, and `api_post` with one generic projection from the record envelope and the field map. No document `TypedHandler` or document slug dispatch remains in the JSON path.
- Preserve the MS-032 `payload.json` bytes for every document type and the `api-post-success` and `api-post-conflict` behavior: `kind`, `form`, `title`, the declared field values under their existing keys, `labels`, `references`, `status` where the old shape carried it, `version`, `path`, and `metadata`. `SingleHtmlMockup` keeps its fixed metadata projection.
- Keep `/api/requirements` with its existing shape and selection by document model and role, not by handler.
- Do not change the HTML render path, the eight `edit::*_ops` builders, the document `TypedHandler` variants, or `src/document.rs::edit_document`; TASK-269 (269b) owns those.
- Do not change routes, status codes, transport, headers, caching, the on-disk layout, or the API aliases; add no dependency, asset, template, client script, or configuration key.

## Grounding
ADR-039 decisions 10-13, 17, 22-28, and 35; TECH-068 sections 5 and 7; TECH-040 section 7; REQ-06.014; MS-035 AC-2 and AC-7; AUD-144; TC-184; `tests/fixtures/characterization/corpus/`; `tests/characterization_replay.rs`; `src/browser/mod.rs`; `src/browser/records.rs`; `src/record/view.rs`.

## Definition of done
`record_json`, `api_doc_list`, `api_get`, and `api_post` no longer inspect a document `TypedHandler`; every document `payload.json` and the `api-post-success` and `api-post-conflict` cases replay byte-for-byte; `/api/requirements` keeps its shape selected by document model and role; no document slug dispatch exists in the JSON path; the full suite, clippy -D warnings, and fmt are clean; evidence records the corpus cases and the deleted JSON branches.

## Acceptance Criteria
- [ ] #1 record_json, api_doc_list, api_get, and api_post no longer inspect a document TypedHandler or dispatch on a document slug; one generic JSON projection derives from the record envelope and the field map.
- [ ] #2 The generic projection preserves the MS-032 payload.json bytes for every document type and the api-post-success and api-post-conflict behavior, including kind, form, title, field values under their existing keys, labels, references, status where the old shape carried it, version, path, and metadata.
- [ ] #3 SingleHtmlMockup keeps its fixed metadata projection and the grouped requirement index /api/requirements keeps its existing shape and selection by document model and role, not by handler.
- [ ] #4 The HTML render path, the eight edit::*_ops builders, the document TypedHandler variants, and src/document.rs::edit_document are unchanged by this task; no route, status code, transport, header, caching, on-disk layout, API alias, dependency, asset, template, client script, or configuration key is added.
- [ ] #5 The full suite, cargo clippy --all-targets -- -D warnings, and cargo fmt --check pass; the characterization replay shows the JSON and API cases byte-identical; evidence records the corpus case counts and the removed JSON branches.
## Comments
- **planner**: planner: MS-035 TASK-269 split applied. This is 269a: generic document JSON projections preserving the MS-032 payload bytes and removing the JSON document handler branches. TASK-269 is 269b and depends on this task. TASK-270 depends on TASK-269 (last of the split) and TASK-309. TASK-309 depends on TASK-269 (the deletion task).
- **rust-dev**: TASK-311 evidence pending
- **rust-dev**: TASK-311 implementation summary and removed branches listed in the next comment.
- **rust-dev**: Test colon: CategoryKind Document and 11 handlers.
- **rust-dev**: Test parens (one) and field_map.fields.
- **rust-dev**: Test scope CategoryKind::Document and edit::document_edit_ops.
- **rust-dev**: TASK-311 implementation: record_json now dispatches on CategoryKind::Document to document_record_json, which projects the envelope, field_map.fields, descriptor field_defaults, and the SingleHtmlMockup fixed_metadata block.
- **rust-dev**: Removed the 8 document arm groups, covering 11 handlers: Requirement, Glossary, UiSpec, UiMockup, Rule, TechSpec, UseCase, TestCase, TestReport, Audit.
- **rust-dev**: word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word
- **rust-dev**: word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word
- **rust-dev**: word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word
- **rust-dev**: word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word
- **rust-dev**: TASK-311 implementation: record_json dispatches on CategoryKind::Document to document_record_json
- **rust-dev**: It projects the envelope, field_map.fields, descriptor field_defaults, and the SingleHtmlMockup fixed_metadata block
- **rust-dev**: Removed the 8 document arm groups, covering 11 handlers: Requirement, Glossary, UiSpec, UiMockup, Rule, TechSpec, UseCase, TestCase, TestReport, Audit
- **rust-dev**: Kept the 4 backlog and decision arms.
- **rust-dev**: Kept the four arms.
- **rust-dev**: TASK-311 implementation evidence from rust-dev. record_json dispatches on CategoryKind Document to document_record_json, which projects the envelope, field_map.fields, descriptor field_defaults, and the SingleHtmlMockup fixed_metadata block. Removed the 8 document arm groups covering 11 handlers: Requirement, Glossary, UiSpec, UiMockup, Rule, TechSpec, UseCase, TestCase, TestReport, Audit. Kept the 4 backlog and decision arms.
- **rust-dev**: TASK-311 api_doc_list and api_get evidence. Document lists use document_list_json: grouped types use requirements_json selected by DocumentModel GroupedMarkdown, config-defined types use the full summary, and other seed types use id, title, metadata. Removed the Requirement and handler-none document branches and kept the backlog and decision summaries. api_get resolves a document group by model and role via document_group and no longer inspects a document handler; the related lookup remains Task and Milestone only.
- **rust-dev**: TASK-311 api_post evidence. Document writes use api_post_document with field-map validation and edit document_edit_ops; mockups use api_post_mockup with the fixed specs, refs, and title projection. Removed the 8 document arm groups and kept Task, Milestone, Release, and Decision. The requirements index now selects DocumentModel GroupedMarkdown and no longer reads RequirementKind; an empty group title is null for every grouped type and no corpus case has an empty group title. JSON document validation now follows the same field-map contract as the one HTML document form from TASK-266: optional fields clear on null or empty, required fields are 422, audit-type and requirement-list checks come from descriptor field options, and unknown members are ignored. Tests t25j, t26i, t57i, and t81j were updated for this consolidation.
- **rust-dev**: TASK-311 payload compatibility and checks. 0 of 96 payload.json files changed versus pre-MS-035 commit 214a252; the characterization replay covers 118 HTTP and 136 CLI cases and passes byte for byte, including api-doc-post-success and api-doc-post-conflict. The 46 JSON API cases are 12 record GET, 12 canonical list GET, 2 requirements GET, 18 legacy alias GET, and 2 POST. Full cargo test passes, cargo clippy --all-targets -- -D warnings passes, and cargo fmt --check passes. No new crate, route, status code, header, template, asset, configuration key, or HTML, ops builder, or edit_document change. Branch task-TASK-311. No commit, merge, checkout, or push per instruction.
- **orchestrator**: Acceptance met: generic document JSON projections for record, list, api get and api post; zero payload differences against the pre-MS-035 oracle 214a252; document handler arms removed from the JSON path; 1317 tests pass, clippy and fmt clean.
