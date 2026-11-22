---
id: TASK-266
title: "Implement the generic edit form and form_post handler"
created_date: '2026-11-20T17:42:43Z'
updated_date: '2026-11-22T16:33:29Z'
labels: []
references:
  - ADR-039
  - ADR-0028
  - ADR-0013
  - REQ-07.003
  - TECH-068
  - REQ-06.014
  - TECH-040
  - TC-183
  - TC-184
status: ToDo
assignee:
  - rust-dev
milestone: MS-035
dependencies:
  - TASK-265
ordinal: 50
documentation:
  - src/browser/mod.rs
  - src/browser/render.rs
  - src/record/edit.rs
  - src/document.rs
  - src/browser/templates/edit.html
type: implementation
---

## Description
## Context
Edit forms and `POST /doc/{id}` handling are duplicated per document type. `src/browser/mod.rs` `edit_page` dispatches on `record.handler()` and `form_post` contains a nine-branch stale-version conflict chain and a long per-type validation and save chain. `src/browser/render.rs` has `edit_page` plus `edit_page_glossary`, `edit_page_ui_spec`, `edit_page_ui_mockup`, `edit_page_business_rule`, `edit_page_audit_record`, `edit_page_tech_spec`, `edit_page_use_case`, `edit_page_test_case`, and `edit_page_test_report`; `src/record/edit.rs` has eight document ops builders the browser calls. ADR-039 decisions 22-28 and TECH-068 sections 4.2-4.3 define the replacement. TASK-265 delivers `DocumentFieldMap` on `Record`.

## Scope
- One document edit builder using the TASK-265 field map: the fixed `title` control; one control per `ResolvedField` whose `editable` is true in descriptor order, control type from `kind` (text, multiline, list), `label` as the control label, `help` as the hint bound with `aria-describedby`, and the required marker from `required`; a field whose `verbs` exclude `edit` is rendered read-only (label and value, no control) and is never writable; the general `labels` and `references` controls; read-only `status`, `created_date`, and `updated_date`; and the hidden version token. Use `edit.html`; delete `DOCUMENT_EDIT_TEMPLATE` and `document_edit.html` once unused.
- `SingleMarkdown` uses `descriptor.fields`; `GroupedMarkdown` uses the role fields resolved from the record; the group link is read-only and never a control; membership is never a control. `SingleHtmlMockup` offers the title plus the list-valued `specs` and `refs` controls only; `status`, `task`, `references`, and `superseded_by` are read-only; there is no field sequence and no `status` control.
- One document `form_post` for `POST /doc/{id}` with the fixed order of ADR-039 decision 26 and TECH-068 section 4.3: `415` for a media type other than `application/x-www-form-urlencoded`; `400` for a missing or malformed `version`; `404` for an unknown, archived, or disabled ID through the one not-found builder; `409` for a stale token through the one conflict builder with the submitted values and the live token, writing nothing; `422` for an undeclared key (including `status`) or a validation error, re-rendering the edit page with the submitted values and the same token, writing nothing; `303` to `/doc/{id}` on success. Accepted keys are `title`, `labels`, `references`, a declared field key of the record's map, and `specs` and `refs` for `SingleHtmlMockup`.
- Title contract: single line, non-empty, at most `MAX_TITLE_CHARS` (`src/browser/records.rs`), and never clear an existing title on edit. A required field may change but not be cleared; an optional field may be cleared; `kind` rules apply. Build one patch from the field map and apply it once through `record::store`; refresh `updated_date` once; a no-op save leaves the file byte-identical.
- Reuse the field-to-`PatchOp` mapping of TECH-040 section 7 and the existing generic helpers in `src/document.rs` (`clear_field_ops`, `set_scalar_ops`, `list_field_ops`, and the title operations) through one shared record-layer helper rather than re-implementing the semantics. Do not call the eight per-type `edit::*_ops` builders from the document path.
- Keep the Backlog and Decision edit and POST behavior unchanged; route the document category to the new builder and handler. Do not add a route, dependency, asset, template, client script, or client-side script. The JSON `api_post` document path is handled by TASK-269.

## Grounding
ADR-039 decisions 22-28; ADR-0028 (template set); ADR-0006 and ADR-0013 (optimistic concurrency and single write); TECH-040 section 7; TECH-068 sections 4.2-4.3; REQ-07.003 and REQ-06.014. Symbols: `src/browser/{mod,render,records}.rs`, `src/record/{edit,document,store,patch}.rs`, `src/browser/templates/edit.html`, and `Record::MAX_TITLE_CHARS`.

## Definition of done
One field-map-driven edit form and one document `form_post` serve every document type and role; the status-code surface and optimistic concurrency are preserved; the document path no longer calls the eight ops builders; Backlog and Decision output is unchanged; tests, clippy -D warnings, and fmt are clean.

## Acceptance Criteria
- [ ] #1 One document edit builder renders, in order, the fixed title control, one control per editable ResolvedField in descriptor sequence order with control type from kind, label from label, help bound with aria-describedby, and the required marker from required, the general labels and references controls, read-only status, created_date, and updated_date, and the hidden version token; a field whose verbs exclude edit is rendered read-only and is never writable.
- [ ] #2 GroupedMarkdown group and member forms use the role fields resolved from the record; the group link is read-only and never a form control; membership is never a form control. SingleHtmlMockup offers title plus the list-valued specs and refs controls only, with status, task, references, and superseded_by read-only, no field sequence, and no status control.
- [ ] #3 No document edit form renders a name="status" control or accepts status as a writable key; title is not a field; labels and references are general metadata controls, not declared fields.
- [ ] #4 One document form_post implements the fixed order: 415 for a non-form media type; 400 for a missing or malformed version; 404 for an unknown, archived, or disabled ID through the one not-found builder; 409 for a stale token through the one conflict builder with the submitted values and the live token and no write; 422 for an undeclared key (including status) or a validation error with the submitted values and the same token and no write; 303 to /doc/{id} on success.
- [ ] #5 Accepted form keys are exactly title, labels, references, a declared field key of the record's field map, and specs and refs for SingleHtmlMockup; every other key, including status and a group key posted to a member record, is 422 and no write occurs.
- [ ] #6 The title contract is enforced: single line, non-empty, at most MAX_TITLE_CHARS, and an existing title cannot be cleared on edit. A required field may be changed but not cleared; an optional field may be cleared; text, multiline, and list kind rules apply; a validation failure writes nothing.
- [ ] #7 A successful save builds one patch from the field map, applies it once through record::store, and refreshes updated_date once; a no-op save leaves the record byte-identical; a stale-token conflict and a validation failure leave the record byte-identical.
- [ ] #8 The document edit and form_post path reuses the field-to-PatchOp mapping of TECH-040 section 7 and does not call generic_document_ops, requirement_ops, glossary_ops, ui_spec_ops, business_rule_ops, audit_ops, tech_spec_ops, or use_case_ops.
- [ ] #9 Backlog and Decision edit and POST behavior, including the board status submit and typed edit pages, is unchanged; the document category is routed to the new builder and handler.
- [ ] #10 document_edit.html and DOCUMENT_EDIT_TEMPLATE are deleted once unused; the template set remains shell.html, list.html, view.html, edit.html, and error.html; no new route, dependency, asset, client script, or template is added.
- [ ] #11 Tests cover each document model and role, the 415, 400, 404, 409, 422, and 303 outcomes, the title and required-field rules, and a no-op byte-identical save; cargo test, cargo clippy --all-targets -- -D warnings, and cargo fmt --check pass.
