---
id: TASK-255
title: "Implement the field-driven document handler with baseline verbs and status"
created_date: '2026-11-20T17:41:57Z'
updated_date: '2026-11-22T13:31:27Z'
labels: []
references:
  - ADR-036
  - ADR-0016
  - REQ-07.003
  - REQ-05.011
  - TECH-070
  - TECH-066
  - REQ-07.004
  - ADR-0017
  - ADR-042
  - REQ-05.006
status: In-Progress
assignee:
  - rust-dev
milestone: MS-034
dependencies:
  - TASK-254
ordinal: 30
documentation:
  - src/document.rs
  - src/registry.rs
  - src/record/patch.rs
  - src/record/store.rs
  - src/record/document.rs
  - tests/document_dispatch.rs
  - tests/descriptor_equivalence.rs
  - src/config.rs
type: implementation
---

## Description
## Context
`src/document.rs` still dispatches typed content verbs through `dispatch_typed` (line ~354) to the nine per-type modules (`requirement`, `glossary`, `rule`, `ui_spec`, `audit`, `tech_spec`, `use_case`, `test_case`, `test_report`) and keeps `create_generic`/`edit_generic` for user-defined types. `src/registry.rs` still supplies per-type `VerbSpec` tables (`document_schema`, `requirement_verbs` at 771, `glossary_verbs` at 941, `audit_verbs` at 1265, `ui_mockup_verbs` at 1800, and the rest) and `TypeDescriptor::verb_specs`/`ordered_verb_specs` (256-290) read them. MS-033 added the seed table `SEED_DOCUMENT_TYPES` (`src/config.rs:614`), `DocumentModel` (`src/registry.rs:232`), `FieldSpec` (`src/config.rs:196`), and the resolved `fields`/`group`/`member` on each descriptor, but the CLI still uses the transitional `TypedSchema`.

## Scope
- Replace `dispatch_typed`, `create_generic`, and `edit_generic` in `src/document.rs` with one engine:
  - resolve the descriptor, one `DocumentScope`, and the `RecordDescriptor`;
  - load the record for edit, status, archive, and delete;
  - build one model-neutral field map for the record role: `Single` from `fields`, `Group` from `group.fields`, `Member` from `member.fields`;
  - build one `Vec<PatchOp>` for the verb from the field sequence and model: `SetFrontMatter`/`SetFrontMatterValue` for `frontmatter_key`, `SetSection`/`ClearSection` for `heading`, `SetHeadingTitle` for the title, and the model-specific group-link, HTML-metadata, and supersession operations owned by TASK-256;
  - apply through `record::patch::apply_patch` and write once through `record::store::write_atomic`; a no-op patch performs no write.
- Derive the command metadata from the model and field sequence in `src/registry.rs` (`TypeDescriptor::verb_specs`/`ordered_verb_specs`, `field_options`): baseline verbs, model-derived status/create-group/supersede placement, the title contract (TASK-257), model-fixed options for `SingleHtmlMockup` (TASK-256), `--label`/`--reference`, and `--editor` on edit. Stop reading the transitional `document_schema`/`*_verbs` for command construction; TASK-259 deletes them.
- Seed defaults (ADR-042 ruling 1): add `field_defaults` to `SeedDocumentType`, empty for every seed type except `audit`, whose entry carries `field_defaults: { audit_type: security }`; the resolved descriptor carries the same map. On `create`, when a field that has a seed default is absent, write the default through the ordinary `PatchOp` path; on `edit`, an absent field stays absent. `field_defaults` is not a `FieldSpec` key, not a configuration key, and not accepted in a project entry; a user-defined type has no field defaults; no slug literal in the CLI, engine, or browser supplies the value.
- Preserve create/edit ID allocation (`record::identity::next_number`, `format_id`), required-field validation, list rendering, section order, list/view rendering, and the existing archive/delete/status record semantics.
- Update `tests/document_dispatch.rs`, `tests/descriptor_equivalence.rs`, and `tests/document_type_no_drift.rs` to the new engine; TASK-260 regenerates the help goldens.

## Out of scope
GroupedMarkdown create-group, group-link validation, and SingleHtmlMockup supersede (TASK-256); title-contract changes and `--term` removal (TASK-257); config selectors (TASK-258); deleting the nine modules or the transitional schema (TASK-259), and deleting the per-type ops builders or document TypedHandler variants (MS-035 TASK-266/TASK-269); help-golden regeneration (TASK-260); the browser.

## Grounding
ADR-036 decisions 7, 8, 15-24, 35-37 and downstream constraints 1-6; ADR-042 ruling 1; TECH-070 sections 1-5 and 7; TECH-066; REQ-07.003; REQ-05.006; REQ-05.011; REQ-07.004.

## Definition of done
One engine drives every document model; the baseline verbs and status behavior are unchanged; field options and required-field validation are field-derived and deterministic; the `audit` seed default is applied on create and not on edit without a `FieldSpec` key, configuration key, or slug literal; the engine emits only PatchOps and uses the single writer; the targeted suites pass; `cargo test`, `cargo clippy --all-targets -- -D warnings`, and `cargo fmt --check` are clean. Any seed fact the engine cannot express is recorded before module deletion, not hard-coded by slug.

## Acceptance Criteria
- [ ] #1 Every document model dispatches through one handler: `dispatch_typed` and the nine per-type `handle` calls are gone from the document engine; no document slug literal selects a verb, option, field, or status behavior. (MS-034 AC-1, AC-6)
- [ ] #2 The baseline verbs create, list, view, edit, archive, and delete behave identically for every document type; status exists only for SingleMarkdown and GroupedMarkdown; SingleHtmlMockup has no status verb. (MS-034 AC-1)
- [ ] #3 Create and edit options are derived from the record role field sequence: each field contributes one option on the verbs listed in its `verbs`, in sequence order, with its `option` or kebab-case key. (MS-034 AC-6; ADR-036 d15, d19, d20)
- [ ] #4 A create that omits required fields fails with one validation outcome listing every missing field key, exit 4, and writes nothing; an edit cannot clear a required field; an optional field can be cleared. (MS-034 AC-1; ADR-036 d18)
- [ ] #5 The `audit` seed entry carries `field_defaults: { audit_type: security }` and no other seed type has a field default; `create` writes the default when `audit_type` is absent, `edit` leaves an absent `audit_type` unchanged, and no `FieldSpec` key, configuration key, or document slug literal supplies the value. (ADR-042 ruling 1; REQ-05.006; MS-034 AC-4)
- [ ] #6 Field storage follows the field spec: heading-only writes the section; frontmatter_key-only writes the front-matter key; both writes both; a list field in a section writes `- item` lines; a list field with a frontmatter_key writes a YAML sequence including a single element and an empty list; field sequence order determines section order. (ADR-036 d17, d21, d35)
- [ ] #7 The engine emits only PatchOp values, applies them through record::patch, and writes once through record::store::write_atomic; it never formats a `---` block, re-renders a body, or calls fs::write; a patch that changes nothing performs no write. (ADR-036 d35-d37; TECH-070 section 4)
- [ ] #8 Archive and delete keep the existing semantics: archive moves to the type archived subfolder, is idempotent, and never cascades to grouped members; delete follows archive-then-remove; status rejects `archived` and archived records and does not move files. (MS-034 AC-1; TECH-066)
- [ ] #9 list and view render through the same model-neutral field map with frontmatter_key fallback to heading; the list text and JSON keys are unchanged for every existing record. (MS-034 AC-1)
- [ ] #10 No document type loses a field, heading, option, required flag, field verb, status value, or archive behavior that its module supplied; every fact the seed cannot express is recorded with its owner before the module is deleted, and no per-type default is hard-coded in the engine. (MS-034 AC-4, AC-6; ADR-036 downstream constraint 1)
- [ ] #11 The document dispatch, descriptor equivalence, and document no-drift suites pass on the new engine; cargo test, cargo clippy --all-targets -- -D warnings, and cargo fmt --check are clean. (DoD)
## Comments
- **task-refiner**: NOTE: FieldSpec has no short option, so the current -d shorts on description (for example req create -d, req edit -d, note create -d) cannot survive the field-derived command builder. TASK-260 must attribute their removal under derived field options. No short option is added to FieldSpec.
- **task-refiner**: Grounding: current engine is src document.rs run (line 221), dispatch_typed (354), create_generic (444), edit_generic (673), archive_record (824), delete_record (846), status_record (891). Command metadata still comes from TypeDescriptor verb_specs and ordered_verb_specs (registry 256-290) and the transitional document_schema plus *_verbs (771-2074). The seed fields are in src config.rs SEED_DOCUMENT_TYPES (614) and FieldSpec (196).
- **task-refiner**: Task boundary: this task lands the shared engine, the model-neutral field map, the field-derived command metadata, and the SingleMarkdown baseline end to end. The GroupedMarkdown create-group and group-link validation and the SingleHtmlMockup fixed model options, archive, and supersede paths are TASK-256, which runs after this task. The dispatch shape and field map must support all three models here, so AC-2 rows for grouped and mockup baseline verbs are satisfied jointly with TASK-256; do not duplicate the TASK-256 logic in this task.
- **planner**: ADR-042 ruling 1 applied: the audit seed entry carries field_defaults: { audit_type: security }; SeedDocumentType and the resolved descriptor carry the map, create applies an absent field's seed default through the ordinary PatchOp path, and edit leaves an absent field unchanged. field_defaults is not a FieldSpec key, configuration key, or project-entry key; no slug literal supplies the value. The audit blocker is cleared.
- **planner**: ADR-042 ruling 2 applied: verb about lines are engine-generated generic text from the document model and field sequence; no seed verb-about data is added and TASK-260 attributes every resulting help diff to generated prose. The frozen per-type verb-about requirement is narrowed accordingly.
- **rust-dev**: rust-dev evidence test
- **rust-dev**: rust-dev evidence commit 57be510: document::run now resolves one descriptor, scope and record descriptor and dispatches every Markdown model through one engine (create_document, create_group_document, list_documents, view_document, edit_document); dispatch_typed and the nine per-type handle calls are gone, while SingleHtmlMockup keeps ui_mockup::handle until TASK-256. The engine builds the role field map and emits only PatchOp values applied through record::patch and written once through record::store::write_atomic, with no fs::write and no body render in production. verb_specs derives Markdown verb metadata from the model and role field sequence (key or option, text multiline or list, required, per-field verbs, label and help); list fields also derive a removal companion to keep --no-requirement; title shapes stay transitional and glossary keeps --term for TASK-257. SeedDocumentType.field_defaults and TypeDescriptor.field_defaults were added, only audit carries audit_type security, create applies an absent default through the ordinary PatchOp path, edit leaves it absent, and it is not a FieldSpec or configuration key. Dual heading and frontmatter fields write both; list fields write YAML sequences via FrontMatterValue::Sequence including one element and the empty list; optional fields clear and required fields reject clearing; missing required keys aggregate into one exit-4 validation with no write. Deltas: required flags follow the seed table so ui-spec, tech-spec, use-case, test-case and test-report view, description and applicability become optional per ADR-036 decision 29; the -d short is gone; verb about and option help prose are generated and help goldens were regenerated here for TASK-260 to attribute; the characterization corpus is green with no title delta because TASK-257 was not pre-empted; backlog and decision commands are unchanged. Checks: env -u EDITOR -u VISUAL cargo test, cargo clippy --all-targets -- -D warnings and cargo fmt --check are clean; new engine tests cover the audit default, required aggregation, required versus optional clearing, field option order, and dual storage plus list sequences. Seed-inexpressible facts to record before TASK-259: glossary and requirement view keep the trailing newline, ui-spec re-adopts frontmatter after the editor, and nfr create ignores --story.
