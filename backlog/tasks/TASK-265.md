---
id: TASK-265
title: "Implement the record envelope field map with generic list and view rendering"
created_date: '2026-11-20T17:42:31Z'
updated_date: '2026-11-22T17:06:20Z'
labels: []
references:
  - ADR-039
  - ADR-036
  - REQ-006.014
  - REQ-006.004
  - TECH-040
  - TECH-068
  - TC-184
status: Done
assignee:
  - rust-dev
milestone: MS-035
dependencies:
  - TASK-264
ordinal: 40
documentation:
  - src/browser/records.rs
  - src/browser/render.rs
  - src/browser/list.rs
  - src/record/view.rs
  - src/registry.rs
type: implementation
---

## Description
## Context
`Record` in `src/browser/records.rs` currently carries the union of every document type's content fields (`story`, `description`, `rationale`, `applicability`, `requirements`, `audit_type`, `definition`, `view`, `specs`, `mockup_task`, and equivalents), populated by a `match record.handler()` in `load_record`. `render::view_page` is one function with a `match record.handler()` and eight document branch clusters, and `render::document_detail_page` is a second, generic-only detail renderer. `list::ListPlan::for_descriptor` already builds one shared table but reads `routes::list_slots` and one plan-wide `show_references` flag. ADR-039 decisions 14-21 and TECH-040 sections 2-3 define the replacement; TECH-068 section 4.1 fixes the rendered structure.

## Scope
- Add the model-neutral field map from ADR-039 decision 14 and TECH-040 section 3 in the record layer (`src/record/`, exported through `src/record/mod.rs`): `DocumentFieldMap { role, fields: Vec<ResolvedField>, fixed_metadata, extra_metadata, reference_visibility }`, `ResolvedField { key, label, help, kind, required, editable, value }`, `FieldRole::{Single,Group,Member}`, and `FieldValue::{Text,Items,Absent}`. Build it once per loaded document record from `TypeDescriptor.fields`, `group`, `member`, and `document_model`, and carry it on `Record` in `src/browser/records.rs`.
- Resolve values deterministically per ADR-039 decision 16 and TECH-040 section 3.5: a `heading`-only field reads the body section; a `frontmatter_key`-only field reads that key; a field with both reads the body section as the display value and does not merge the two locations. An absent optional value is `Absent`; an absent required value is `Absent` and is flagged for the edit form. `audit_type` reads its declared `frontmatter_key` (`type`). `SingleMarkdown` uses `descriptor.fields` role `Single`; `GroupedMarkdown` uses `descriptor.group.fields` role `Group` for a group record and `descriptor.member.fields` role `Member` for a member; `SingleHtmlMockup` has `fields: []` and fixed metadata `status`, `task`, `specs`, `refs`, `superseded_by` from the HTML metadata comment.
- Add one document view builder for the document category, replacing the document branches of `render::view_page` and `render::document_detail_page`: the ADR-0033/TECH-068 section 4.1 head and chrome, labels, dates, one `<section>` per non-empty field in descriptor sequence order headed by the field `label`, prose pipeline for text/multiline and a list for `kind = list`, a required empty field as an empty section, `extra_metadata` as read-only rows, role-aware grouped rendering, and the mockup fixed metadata block instead of field sections. No per-type summary card and no repetition of the document type. Delete `DOCUMENT_DETAIL_TEMPLATE`/`document_detail.html` once unused; `view.html` remains the one template.
- Make document list rows role-aware from the field map: labels in the Title cell, the group row and member rows in the shared hierarchical table, and no document field becomes a list column. List-level presentation lookup (heading, count noun, empty state, slots, inventory, aliases) and reference visibility stay with TASK-268; do not add a new content-model or slug branch here.
- Preserve the Backlog and Decision surfaces in `render.rs`/`mod.rs` byte-compatibly. No document slug name match in the field-map/list/view path. Do not add a dependency, asset, template, client script, or route.

## Grounding
ADR-039 decisions 14-21 and 24-25; ADR-0033 (as amended by ADR-039 decisions 6-7); TECH-040 sections 2-3 and 7; TECH-068 sections 3.1 and 4.1; REQ-006.014 and REQ-006.004. Symbols: `src/browser/{records,render,list,shell,assets}.rs`, `src/record/{view,document,frontmatter}.rs`, `TypeDescriptor` fields `fields`/`group`/`member`/`document_model` in `src/registry.rs`, `src/browser/templates/{view,list}.html`, and `browser::assets::ASSETS`.

## Definition of done
The field map is the only document content projection for list and view; group, member, and mockup rendering is role-aware; the document detail template is gone; Backlog and Decision output is unchanged; tests, clippy -D warnings, and fmt are clean.

## Acceptance Criteria
- [ ] #1 A DocumentFieldMap and ResolvedField are built once per loaded document record from the descriptor and carried on Record in src/browser/records.rs; SingleMarkdown (fields, role Single), GroupedMarkdown group (group.fields, role Group), GroupedMarkdown member (member.fields, role Member), and SingleHtmlMockup (fields empty plus fixed metadata) resolve per ADR-039 decision 14 and TECH-040 section 3.
- [ ] #2 Value resolution is deterministic: heading-only reads Document.sections by heading; frontmatter_key-only reads record::frontmatter; both reads the body section as the display value without merging; audit_type reads frontmatter key type; an absent optional value is Absent; an absent required value is Absent and flagged for the edit form; a divergent body and frontmatter pair displays the body value and is not repaired on read.
- [ ] #3 One generic document view renders the field map: the shared head and chrome, labels, dates, one section per non-empty field in descriptor sequence order headed by the field label (never the storage heading), prose for text or multiline, a list for kind=list, a required empty field as an empty section, and extra_metadata as read-only rows; there is no per-type summary card and no repetition of the document type.
- [ ] #4 Grouped rendering is role-aware: a group record renders group.fields sections then a read-only member list (ID, title, link) in ID order; a member record renders member.fields sections then a read-only group link; group is not a field and membership is never rendered as a field or control.
- [ ] #5 SingleHtmlMockup view uses the fixed metadata block (status, task, specs, refs, superseded_by) read from the HTML metadata comment and renders no field sections, through the same view.html and shell.
- [ ] #6 Document list rows are built from the descriptor and field map: labels render in the Title cell; grouped group and member rows render through the existing shared hierarchical row variant; no document field becomes a list column and the structural column order is unchanged. List-level presentation lookup, slots, and reference visibility remain TASK-268.
- [ ] #7 No document slug name match, per-type handler branch, ContentModel branch, or document-type lookup table is added in the field-map, list, and view path; Backlog and Decision renderers and their output remain unchanged.
- [ ] #8 document_detail.html and DOCUMENT_DETAIL_TEMPLATE are deleted once unused; view.html, list.html, and shell.html remain; render::page_assets and browser::assets::ASSETS are unchanged and math and diagram asset links still render.
- [ ] #9 cargo test, cargo clippy --all-targets -- -D warnings, and cargo fmt --check pass; no new dependency, asset, template, client script, or route is added.
## Comments
- **rust-dev**: test comment
- **rust-dev**: TASK-265 evidence: field map in src-record-field_map.rs (DocumentFieldMap, ResolvedField, FieldRole Single, Group, Member; FieldValue Text, Items, Absent; role, fields, fixed_metadata, extra_metadata, reference_visibility; ResolvedField carries key, heading, label, help, kind, required, editable, frontmatter_key, value plus missing_required). Built once in browser records load_record and load_ui_mockup from the descriptor and carried on Record.field_map. SingleMarkdown uses fields with role Single; GroupedMarkdown uses group.fields with role Group or member.fields with role Member; SingleHtmlMockup uses fields empty plus fixed status, task, specs, refs, superseded_by. Heading-only reads Document.sections; frontmatter-only reads RecordView; both display the body with frontmatter fallback when the body is absent, never merged or repaired; audit_type reads frontmatter type when the body is absent; optional absent equals Absent; required absent equals Absent plus edit-form flag; undeclared frontmatter keys become extra_metadata. Renderers: one render::document_view_page for every Document record using view.html head and chrome, ID, status, title, labels, dates, one record-section per non-empty field in descriptor order headed by the field label, prose for text and multiline, ul.req-list for kind list, empty required section, extra metadata rows, role-aware group member list and member group link, mockup fixed metadata block, and reference, dependency, dependent disclosures. render::document_detail_page, document_detail.html and DOCUMENT_DETAIL_TEMPLATE are deleted. mod.rs selects the new builder for all Document records; Backlog and Decision keep view_page. List slot_cell reads document slot values from the field map (group remains engine-owned); no document field becomes a column and row structure is unchanged. No new slug literal, TypedHandler or ContentModel branch, lookup table, dependency, asset, template, client script, or route. Checks: env -u EDITOR -u VISUAL cargo test passes; cargo clippy --all-targets -- -D warnings passes; cargo fmt --check passes; MS-032 characterization replay passes after regenerating only the 11 document view page.html and http.json baselines (audit, glossary, nfr, req, rule, tech-spec, test-case, test-report, ui-mockup, ui-spec, use-case). Equivalence deltas are consolidation-only: field labels head every section (adds Description for tech-spec, use-case, test-case, test-report; Audit type and Content for audit), shared taglist markup, mockup fixed metadata plus shared head and dates, and rule Requirements moves from a disclosure to a field list. Remaining semantic note for TASK-269: rule requirement list items are plain text because field list items are model-neutral values, so the old resolved requirement link is gone; no other links, navigation, breadcrumbs, row labels or references, or Backlog and Decision output changed. Branch task-TASK-265; no commit, merge, checkout, or push per instruction. Open: semantic HTML comparator and EXPECTED-DELTAS attribution remain TASK-269; list presentation, slot, and reference-visibility lookup remains TASK-268.
- **orchestrator**: Acceptance met: DocumentFieldMap with role-aware resolved fields built once per record; generic document view and list consume it with no slug or per-type branch; grouped role rendering and mockup fixed metadata; detail template deleted; semantic checks pass. NOTE: the eleven view baselines were regenerated, so TASK-311 and TASK-269 must compare against the pre-MS-035 pages from git history.
