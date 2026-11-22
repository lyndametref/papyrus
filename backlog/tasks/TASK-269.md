---
id: TASK-269
title: "Delete the remaining per-type browser branches and prove semantic equivalence"
created_date: '2026-11-20T17:42:52Z'
updated_date: '2026-11-22T16:44:34Z'
labels: []
references:
  - ADR-039
  - TECH-068
  - REQ-06.014
  - ADR-042
  - AUD-135
  - AUD-144
  - REQ-06.004
  - REQ-07.004
  - TECH-040
  - TC-183
  - TC-184
  - TASK-311
status: ToDo
assignee:
  - rust-dev
milestone: MS-035
dependencies:
  - TASK-311
ordinal: 80
documentation:
  - src/browser/mod.rs
  - src/browser/records.rs
  - src/browser/render.rs
  - src/record/edit.rs
  - src/registry.rs
  - src/config.rs
  - src/document.rs
  - tests/characterization_replay.rs
  - tests/support/characterization.rs
  - tests/document_type_no_drift.rs
  - tests/fixtures/characterization/corpus/EXPECTED-DELTAS.md
  - tests/ui_spec_cli.rs
type: implementation
---

## Description
## Context
After TASK-265 to TASK-268 and TASK-311, the document list, view, edit, conflict, and not-found surfaces are descriptor-driven, and the document JSON projections are generic. The per-type HTML branches, the eight document `edit::*_ops` builders that MS-034 TASK-259 deliberately retained, and the ten document `TypedHandler` variants are still compiled. Survivors before TASK-311 included: `breadcrumb_label` and the document arms of `view_page` in `src/browser/render.rs`; the document `match record.handler()` in `load_record` and the typed document fields on `Record` in `src/browser/records.rs`; the document arms of `edit_page`, `form_post`, `record_json`, `api_doc_list`, `api_get`, and `api_post` in `src/browser/mod.rs`; the per-type edit, conflict, and not-found functions in `src/browser/render.rs` (TASK-267 removes the conflict and not-found variants); the seed `handler` field and the document handler mapping in `src/config.rs` and `src/registry.rs`; and the eight builders in `src/record/edit.rs` (`generic_document_ops`, `requirement_ops`, `glossary_ops`, `ui_spec_ops`, `business_rule_ops`, `audit_ops`, `tech_spec_ops`, `use_case_ops`). TASK-311 (269a) removes the document JSON branches; this task (269b) removes the remaining HTML branches and the typed handlers. ADR-039 decisions 10-13, 17, and 35, ADR-042, MS-035 AC-2 and AC-7, and TECH-068 sections 4-8 define the target. The MS-032 characterization corpus in `tests/fixtures/characterization/corpus/` is the baseline: records, JSON payloads, and status codes are strict; rendered HTML is semantic, not byte-identical (TC-183 and TC-184).

## Scope
- Delete every per-type document branch from the browser HTML path, including the `breadcrumb_label` document arms, the document clusters of `view_page`, all per-type document edit renderers, the `edit_page` dispatch arms, the `form_post` conflict and edit chains for documents, and the conditional document checks. Delete the legacy `document_detail_page` and `document_edit_page` helpers once the generic builders replace them. The only document builders left are one list, one view, one edit, one conflict, and one not-found builder.
- Delete the eight document ops builders from `src/record/edit.rs` and keep `task_status_ops`, `task_ops`, `milestone_ops`, `release_ops`, and `decision_ops`. The document HTML write path must use one shared field-to-`PatchOp` helper based on TECH-040 section 7 (the generic operations already in `src/document.rs`), not a per-type builder. The JSON write path is TASK-311's concern.
- Remove the ten document `TypedHandler` variants (`Requirement`, `Glossary`, `UiSpec`, `Rule`, `UiMockup`, `Audit`, `TechSpec`, `UseCase`, `TestCase`, `TestReport`) and the seed/descriptor handler mapping that carries them. `TypedHandler` is reduced to `Task`, `Milestone`, `Release`, and `Decision`; backlog and decision handlers keep their dispatch. `TypedSchema` and `ui_mockup_schema` carry no document `TypedHandler`, and `TypeDescriptor::handler()` and `SeedDocumentType.handler` no longer have a document path. Remove the `RequirementKind` use that only fed the document variants.
- Resolve the last non-browser document `TypedHandler` use in `src/document.rs::edit_document` with option (a): keep one documented, tested engine exception keyed on the descriptor or model, not on a slug literal or a name comparison. Express the ui-spec front-matter re-adoption through the descriptor (for example a descriptor flag set from the seed for the ui-spec type) and add or extend a test such as `tests/ui_spec_cli.rs::ui_spec_editor_rewrites_without_renaming` to prove the behaviour. Do not drop the quirk and do not add a document slug branch. If the descriptor-keyed rule changes output, record it as an accepted delta in `tests/fixtures/characterization/corpus/EXPECTED-DELTAS.md`; if it preserves output, record the no-byte-delta note there.
- Remove the document entries left in `PRESENTATION`, `routes::list_slots`, `NotFoundPage`, and any per-type lookup table; TASK-267 and TASK-268 should already have done this, and this task proves none remains. No document slug literal may select a behavior, string, list, or template anywhere outside the seed table and its tests.
- Add a test-only semantic HTML comparator in `tests/support/` using only the standard library and the `serde_json` already present: no new crate, no HTML parser dependency, no browser. Use the corpus manifest fields `kind`, `document_type`, `model`, and `renderer` to classify cases; do not classify by filename.
  - Strict comparison: every record file, `payload.json`, `stdout.txt`, `stderr.txt`, `exit.json`, asset `body.txt`, and every `http.json` status code and header stay byte-for-byte as captured. Backlog and Decision `page.html` cases stay byte-for-byte (MS-035 AC-8).
  - Semantic comparison for document HTML (list, view, edit, conflict, not-found): compare HTTP status and `Content-Type` exactly, and compare the canonical tag and attribute stream with attributes sorted; compare headings and section order, table structure and `td` classes, `data-row-id`, `.row-tags` and `.row-refs` presence and item text, `.actions-cell` links, `.record-section` and field labels, `.form`/`.field`/`.form-actions` structure, control tag and `name`, `type`, `value` or text, `required`, `aria-describedby`, label text, the hidden version token, `role=\"alert\"`, `.conflict-box`, `.readonly-block`, sidebar group and link text and `href`, `aria-current`, breadcrumb links, and the Edit/View/Cancel/Back link text and targets. Compare visible text with whitespace collapsed and entities decoded.
  - Ignore only attribute order, insignificant inter-tag whitespace, comments, the doctype, self-closing style, entity spelling, and optional quotes. A difference in any compared element, class, text, link, control, or field fails the gate.
  - Any non-ignored difference must be listed per case and per selector or text node with one reason code: `uniform-title-block` for the ADR-039 uniform document head/title structure, `seed-presentation` for a label, heading, help, order, slot, alias, or reference-visibility value that equals the seed, and `expected-delta-6` for the MS-032 conflict and not-found renderer collapse. No blanket or per-type allowlist; an unused allowlist entry fails the test.
  - Add negative tests proving that a one-character visible-text change, a selector or class change, an added, removed, or reordered form field, and a changed link target each fail, while attribute order, extra whitespace, and entity spelling alone pass.
- Tighten the no-slug-dispatch invariant in `tests/document_type_no_drift.rs`: D-05 must scan production lines (excluding `#[cfg(test)]`) of `src/browser/`, `src/cli/`, and `src/document.rs` for a document slug literal used in a match, condition, comparison, or lookup, allowing only the seed table in `src/config.rs` and its derivation in `src/registry.rs`; tests and docs are exempt. Flip D-15 from \"the eight builders remain\" to \"the eight builders are absent\" and D-16 from \"the document variants remain\" to \"only the four backlog/decision variants exist\". Add a guard that `PRESENTATION` has no document entry, that no document `TypedHandler` string exists in `src/` production, and that document render builders do not read the typed document fields on `Record`.
- Delete `document_detail.html` and `document_edit.html` if TASK-265 and TASK-266 have not already done so. Keep the template set `shell.html`, `list.html`, `view.html`, `edit.html`, and `error.html`. Keep `src/browser/assets.rs` and the embedded asset table unchanged.
- Do not change the status-code surface (415, 400, 404, 409, 422, 303, 500), the transport, headers, caching, `no-store`, the single writer, or the on-disk layout. Do not add a dependency, asset, template, client script, route, or configuration key.

## Grounding
ADR-039 decisions 10-13, 17, 22-28, 35, and downstream constraints 1-10; ADR-042; ADR-0006 and ADR-0013; ADR-0035 as amended; ADR-0038; REQ-06.014, REQ-06.019, REQ-06.004, and REQ-07.004; TECH-040 section 7; TECH-068 sections 4-8; TC-183 and TC-184; MS-032 `EXPECTED-DELTAS.md` item 6; TASK-311 (269a) for the generic JSON projection. Symbols: `src/browser/{mod,records,render,list,routes,shell,assets}.rs`, `src/record/edit.rs`, `src/registry.rs`, `src/config.rs`, `src/document.rs`, `tests/characterization_replay.rs`, `tests/support/characterization.rs`, `tests/document_type_no_drift.rs`, `tests/ui_spec_cli.rs`, `tests/fixtures/characterization/corpus/`, and `tests/fixtures/characterization/corpus/EXPECTED-DELTAS.md`.

## Definition of done
No document `TypedHandler` variant, per-type document browser branch, per-type document ops builder, document `PRESENTATION` entry, or document slug dispatch remains outside the seed and its tests; list, view, edit, conflict, not-found, and JSON are served by descriptor-driven code (JSON delivered by TASK-311); the ui-spec front-matter re-adoption is a descriptor-keyed, tested engine rule with no slug literal; the semantic corpus replay passes with only the three attributed reason codes; the record, JSON, and status-code comparison is strict; Backlog and Decision output is byte-identical; the full suite, clippy -D warnings, and fmt are clean; evidence records the deleted symbols, corpus case counts, the semantic allowlist, before/after production lines, and the ui-spec exception and its test.

## Acceptance Criteria
- [ ] #1 Every per-type document HTML branch is deleted: the breadcrumb_label document arms, the view_page document clusters, all per-type document edit renderers, the edit_page document dispatch arms, the form_post document conflict and edit chains, and the conditional document checks. Only one document list, view, edit, conflict, and not-found builder remain.
- [ ] #2 The eight document ops builders are deleted from src/record/edit.rs: generic_document_ops, requirement_ops, glossary_ops, ui_spec_ops, business_rule_ops, audit_ops, tech_spec_ops, and use_case_ops. task_status_ops, task_ops, milestone_ops, release_ops, and decision_ops remain, and the document HTML write path uses one shared field-to-PatchOp helper based on TECH-040 section 7.
- [ ] #3 TypedHandler has exactly four variants (Task, Milestone, Release, Decision). The ten document variants, the seed handler field, the document branch of TypeDescriptor::handler, and the ui_mockup_schema document handler are removed, and RequirementKind is removed or retained only for a non-document use.
- [ ] #4 The last document TypedHandler use in src/document.rs::edit_document, the ui-spec editor front-matter re-adoption quirk, is replaced by option (a): one documented, tested engine rule keyed on the descriptor or model (for example a descriptor flag set from the seed for ui-spec), with no document slug literal and no name comparison. The existing ui_spec_editor_rewrites_without_renaming test or a new test proves the behaviour. If the rule changes output, it is recorded as an accepted delta in tests/fixtures/characterization/corpus/EXPECTED-DELTAS.md; if it preserves output, the no-byte-delta note is recorded there.
- [ ] #5 Document Record values no longer populate or read the typed document content fields for document types; the field map is the only document content projection, and a source guard fails if the document render builders read story, rationale, applicability, requirements, audit_type, definition, view, specs, or mockup_task. JSON projection is delivered by TASK-311.
- [ ] #6 No document slug literal selects a behavior, string, list, or template outside the seed table and its tests; no document entry remains in PRESENTATION; no NotFoundPage, list_slots(document), or per-type lookup table remains; TypedHandler contains no document variant string in src/ production code.
- [ ] #7 A test-only semantic HTML comparator using only the standard library and serde_json is added in tests/support and used by the characterization replay. It compares document HTML status, Content-Type, canonical tags and attributes, headings, sections, table and cell hooks, labels, visible text, row tags and references, links, navigation and aria-current, breadcrumbs, form control tag, name, type, value or text, required, aria-describedby, label text, and hidden version token.
- [ ] #8 The semantic comparator ignores only attribute order, insignificant inter-tag whitespace, comments, doctype, self-closing style, entity spelling, and optional quotes. Any other difference fails unless it is listed per case and per selector or text node under uniform-title-block, seed-presentation, or expected-delta-6; no blanket or per-type allowlist is accepted and an unused allowlist entry fails.
- [ ] #9 Negative tests prove that a one-character visible-text change, a selector or class change, an added, removed, or reordered form field, and a changed link target each fail the semantic comparison, while attribute order, extra whitespace, and entity spelling alone pass.
- [ ] #10 Every MS-032 corpus case replays: record files, payload.json, stdout, stderr, exit, asset bodies, HTTP status and headers, and Backlog and Decision page.html are byte-identical; document HTML list, view, edit, conflict, and not-found pages pass the semantic comparison with only the attributed differences. The JSON and API cases delivered by TASK-311 (269a) remain strict and byte-identical.
- [ ] #11 tests/document_type_no_drift.rs D-05 is tightened to fail any document slug literal dispatch in src/browser, src/cli, and src/document.rs outside the seed and its tests; D-15 asserts the eight builders are absent; D-16 asserts only the four backlog/decision variants exist; new guards assert no document PRESENTATION entry, no document TypedHandler production string, and no typed document field read in document builders.
- [ ] #12 The full suite, cargo clippy --all-targets -- -D warnings, cargo fmt --check, and the no-drift and no-descriptor-leak guards pass; no new dependency, asset, template, client script, route, or configuration key is added; the on-disk layout, record bytes, status-code surface, transport, headers, caching, and single-writer contract are unchanged.
- [ ] #13 Evidence is recorded in the task: the deleted functions, builders, and variants with counts; the number of corpus cases compared strictly and semantically; the complete semantic allowlist with reason codes; before and after production line counts; and the descriptor-keyed ui-spec exception with its test.

## Comments
- **planner**: MS-034 TASK-259 deferral (ADR-042): TASK-259 retains the eight document edit ops builders and the document TypedHandler variants because the browser consumes them. Delete the eight builders here together with the remaining per-type browser branches after TASK-266 lands the generic edit form and form_post. Builders: generic_document_ops, requirement_ops, glossary_ops, ui_spec_ops, business_rule_ops, audit_ops, tech_spec_ops, use_case_ops.
- **task-refiner**: Refinement note: not execution-ready until the ui-spec editor quirk in document.rs edit_document has a descriptor-driven expression or an approved exception, and the generic document JSON projection is accepted in scope. Do not add a slug literal or drop the quirk. See the TASK-264 planner escalation comments.
- **planner**: planner: MS-035 TASK-269 split applied by the planner. This task is 269b: it keeps the HTML branch deletion, the eight ops builders, the document TypedHandler variants, the ui-spec editor exception, the semantic comparator, the no-slug guards, and the replay. It depends on TASK-311 (269a), which owns the generic document JSON projections. The ui-spec editor quirk is resolved by option (a): one descriptor-keyed, tested engine rule with no slug literal, recorded in EXPECTED-DELTAS.md as a no-byte-delta note. TASK-309 depends on this deletion task; TASK-270 depends on this task and TASK-309.
