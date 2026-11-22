---
id: TASK-259
title: "Delete the nine typed document modules and transitional document command tables"
created_date: '2026-11-20T17:42:17Z'
updated_date: '2026-11-22T14:49:24Z'
labels: []
references:
  - ADR-036
  - TECH-070
  - TECH-066
  - REQ-07.004
  - ADR-042
status: Done
assignee:
  - rust-dev
milestone: MS-034
dependencies:
  - TASK-256
  - TASK-257
  - TASK-258
  - TASK-308
ordinal: 70
documentation:
  - src/lib.rs
  - src/registry.rs
  - src/document.rs
  - src/record/edit.rs
  - src/browser/mod.rs
  - tests/document_type_no_drift.rs
type: implementation
---

## Description
## Context
Once TASK-255 and TASK-256 route every document type through the engine, the transitional per-type CLI code is unreachable duplication. The nine CLI modules are `src/test_case.rs`, `src/test_report.rs`, `src/use_case.rs`, `src/ui_spec.rs`, `src/glossary.rs`, `src/tech_spec.rs`, `src/requirement.rs`, `src/rule.rs`, and `src/audit.rs`; `src/ui_mockup.rs` stays. `src/registry.rs` still carries document `TypedHandler` variants, `TypedSchema`, `document_schema`, and the per-type `*_schema`/`*_verbs` builders. `src/record/edit.rs` carries the eight document patch builders, and `src/browser/mod.rs` consumes both the eight builders and the document `TypedHandler` variants; those cannot be deleted before MS-035 TASK-266 (generic edit form and form_post) and TASK-269 (remaining per-type browser branches) land.

## Scope
- Delete the nine module files and their `pub mod` declarations in `src/lib.rs`.
- Keep `src/ui_mockup.rs` as the `SingleHtmlMockup` surface.
- Remove the transitional document CLI schema and per-type verb tables from `src/registry.rs` (`document_schema`, the document `*_schema` and `*_verbs` builders) once TASK-255 derives command metadata from the seed and model. Keep `task_schema`, `milestone_schema`, `release_schema`, `decision_schema`, and their verb builders. Retain the document `TypedHandler` variants and the minimal handler mapping that `src/browser/mod.rs` reads until MS-035 TASK-269 deletes the browser branches.
- Do not delete the eight document ops builders from `src/record/edit.rs` in this task: `generic_document_ops`, `requirement_ops`, `glossary_ops`, `ui_spec_ops`, `business_rule_ops`, `audit_ops`, `tech_spec_ops`, and `use_case_ops`. They remain because `src/browser/mod.rs` calls them; their deletion is deferred to MS-035 TASK-266/TASK-269. Keep `task_status_ops`, `task_ops`, `milestone_ops`, `release_ops`, and `decision_ops`.
- Update the no-drift guards in `tests/document_type_no_drift.rs` (`TYPED_DOCUMENT_MODULES`, `REMOVED_DOCUMENT_ENUMS`, D-10, D-11) and any test that names the deleted modules; keep the ui_mockup guard and the guard that the deferred ops builders still exist until MS-035.
- Confirm no document type loses a command, option, field, heading, required flag, field verb, status value, or archive behavior. Record every fact found missing from the seed before deleting the module that provided it; the audit default is seed `field_defaults` (TASK-255) and verb about lines are engine-generated (TASK-260).

## Out of scope
The engine and field-derived command metadata (TASK-255), grouped and mockup surfaces (TASK-256), title contract (TASK-257), config selectors (TASK-258), help-golden regeneration (TASK-260), the browser, and deletion of the eight document ops builders or document `TypedHandler` variants (MS-035 TASK-266/TASK-269).

## Deferred to MS-035
The eight document ops builders and the document `TypedHandler` variants are retained. TASK-269 deletes them with the remaining per-type browser branches after TASK-266 lands the generic edit form and form_post. See the note on TASK-269.

## Grounding
ADR-036 decisions 8, 28, and 35 and downstream constraints 1 and 6; ADR-042 (browser dependency deferral); TECH-070 sections 2, 4, and 7; TECH-066; REQ-07.004.

## Definition of done
The nine modules and the transitional document CLI verb tables are gone; `ui_mockup.rs` remains; the eight document ops builders and the document `TypedHandler` variants are retained and their deletion is recorded on MS-035 TASK-269; no document behavior is lost; the no-drift guards are updated; the full build, tests, clippy, and fmt are clean; backlog and decision behavior is unchanged.

## Acceptance Criteria
- [ ] #1 The nine typed document modules are absent: test_case, test_report, use_case, ui_spec, glossary, tech_spec, requirement, rule, and audit; ui_mockup remains as the SingleHtmlMockup surface. (MS-034 AC-4)
- [ ] #2 No transitional document CLI schema or per-type verb builder remains in production code; document command metadata comes from DocumentModel, the field sequence, and the seed. The document TypedHandler variants and the minimal handler mapping required by src/browser/mod.rs remain until MS-035 TASK-269. (MS-034 AC-4; ADR-036 downstream constraint 1; ADR-042)
- [ ] #3 The eight document ops builders remain in record/edit.rs and are explicitly deferred to MS-035 TASK-266/TASK-269: generic_document_ops, requirement_ops, glossary_ops, ui_spec_ops, business_rule_ops, audit_ops, tech_spec_ops, and use_case_ops. The task, milestone, release, and decision ops builders remain. No document ops builder is deleted in MS-034. (MS-034 AC-4; ADR-042)
- [ ] #4 Backlog and decision modules, commands, help, and behavior are unchanged; the only allowed surface changes are the MS-034 title, option, derived-verb, and config deltas. (MS-034 AC-4)
- [ ] #5 No document type loses a command, option, field, heading, required flag, field verb, status value, or archive behavior; every fact the seed cannot express is recorded with its owner before the providing module is deleted; the audit_type security default is seed field_defaults (TASK-255) and per-type verb about lines are engine-generated prose (TASK-260). (MS-034 AC-4, AC-6)
- [ ] #6 The full build, test suite, clippy with -D warnings, and fmt check are clean after deletion, and the browser still compiles and its existing tests pass. (DoD)
## Comments
- **task-refiner**: Grounding (browser dependency): src/browser/mod.rs calls edit::requirement_ops, edit::glossary_ops, edit::ui_spec_ops, edit::business_rule_ops, edit::audit_ops, edit::tech_spec_ops, edit::use_case_ops, and edit::generic_document_ops, and selects document behavior through the document TypedHandler variants. This is why the builders and variants remain in MS-034.
- **planner**: ADR-042 planner alignment applied: TASK-259 is narrowed to the nine typed CLI modules and the transitional document CLI schema/verb tables. The eight document ops builders and the document TypedHandler variants are consumed by src/browser/mod.rs, so they remain in MS-034 and their deletion is deferred to MS-035 TASK-266/TASK-269; a deferral note is added to TASK-269. No dependency, ordinal, or milestone change.
- **planner**: The audit_type default and verb-about-line escalations are resolved by ADR-042 rulings 1 and 2 in TASK-255 and TASK-260; no seed fact is missing for module deletion.
- **rust-dev**: TASK-259: deleted src audit.rs, glossary.rs, requirement.rs, rule.rs, tech_spec.rs, test_case.rs, test_report.rs, ui_spec.rs, use_case.rs and their lib.rs mods; removed registry document_schema, the nine *_verbs and ten *_schema builders while retaining ui_mockup.rs and ui_mockup_schema and ui_mockup_verbs, the eight document ops builders, and document TypedHandler variants via document_handler; Markdown docs now derive metadata from DocumentModel and seed (schema None), browser is_generic and api_doc_list use handler().is_none(). Quirks: glossary, req, nfr view trailing newline retained (new handler-mapping test plus corpus); ui-spec post-editor frontmatter adoption retained (ui_spec_cli test); nfr --story is the documented corrected delta (req_cli and document_dispatch tests). Added D-13 to D-16 no-drift guards. env -u EDITOR -u VISUAL cargo test: 1260 passed, 0 failed; clippy -D warnings and fmt clean. LOC excluding task file: +404 and -6089 net -5685. Branch task-TASK-259, working tree (no commit per instruction).
- **orchestrator**: Acceptance met: nine typed document modules and transitional schemas and verb tables deleted with a net reduction of 5685 lines; ui_mockup retained; eight ops builders and TypedHandler variants kept for MS-035; three quirks preserved with guards; 1260 tests pass.
