---
id: TASK-181
title: "Refine removing reference list from document-type browser list views implementation"
created_date: '2026-11-18T19:30:32Z'
updated_date: '2026-11-19T07:46:16Z'
labels: []
references:
  - REQ-06.019
  - RE-006
  - ADR-035
  - TECH-074
  - TECH-076
  - TECH-068
  - TECH-045
  - UI-019
  - MOCK-021
  - TC-055
  - TC-056
  - TC-057
  - TC-058
  - TC-059
  - TC-060
  - TC-061
  - TC-062
  - TC-063
  - TC-064
  - TC-065
status: Done
assignee:
  - task-refiner
milestone: MS-028
dependencies:
  - TASK-180
ordinal: 40
documentation:
  - src/browser/list.rs
  - src/browser/render.rs
  - tests/list_characterization_replay.rs
  - tests/browser_config_types_cli.rs
  - tests/list_shared_layout.rs
type: refinement
---

## Description
Outcome: confirm all information for implementing MS-028 (remove the reference list from document-type browser list views) is available, and complete the refinement fields.

Scope: in the shared browser list only, a `ContentModel::Generic` row (config-defined document type) omits the front-matter reference list entirely; `ContentModel::Typed` and `ContentModel::HtmlMockup` families keep the current reference rendering; labels render in the Title cell for every family. Visibility is resolved from the descriptor content model (ADR-035), never from a slug match, a presentation-table override, a per-family template, or a new presentation field. The reference block is omitted, not collapsed, emptied, or made expandable. `Row.references` may still carry the `RecordView` projection; only rendering omits it (TECH-074 5.1, 5.7).

Unchanged: fixed column order ID, Title, Status, family slot(s), Created, Updated, Actions; declared family slots; grouped rows; status filter; View/Edit row actions; shell, page width, CSP, and no-JavaScript behaviour; single-document view with its collapsible References section (REQ-06.012, ADR-033); CLI list and view; JSON API; `RecordView`; front matter and record files; glossary landing page; the task board (REQ-06.013).

Implementation surface: the shared list renderer in `src/browser/list.rs` (`ListPlan::for_descriptor`, `ListPlan::render_table`, `row_for`, `render_row`) called by `render::list_page` in `src/browser/render.rs`. Out of scope: record files, CLI output, JSON API, detail view, mockup HTML views, assets, dependencies, ordinals, milestone scope, and the plan.

Assumptions:
- Prerequisites met: architecture settled and user-verified (TASK-178 Done; ADR-035 accepted; architect conformance verdict recorded for TECH-074, TECH-076, TECH-068, and TECH-045); UI design TASK-179 Done with UI-019 verified and MOCK-021 approved by the user; test design TASK-180 Done with TC-055 through TC-065 approved; dependency TASK-180 Done.
- Config-defined document types are exactly the descriptors whose content model is `ContentModel::Generic` (ADR-0018, ADR-035).
- `Row.references` keeps carrying the `RecordView` references for every family; only the renderer omits the block for Generic (TECH-074 5.1).
- Labels are general front-matter metadata and keep rendering for every family, independent of content model.
- `/?type=task` is the backlog board, not a shared list, and is unchanged.
- Rust 1.99.1 toolchain available (ADR-0001); Firefox 115+ and no-JavaScript support required (NFR-05.001, NFR-01.002); browser command behaviour governed by BR-004; views need no network access (NFR-01.003).
- Implementation task TASK-182 and verification task TASK-183 track execution and verification.

Open questions: none.

Definition of done: TASK-182 implements the change in the shared list renderer and supplies the automated tests named below; every acceptance criterion is satisfied and verified by TASK-183; `cargo fmt`, `cargo clippy`, and `cargo test` pass; no record, front-matter, CLI, API, or detail-view change is introduced; no open material questions remain.
## Acceptance Criteria
- [ ] ContentModel::Generic (config-defined document type) list rows render no reference list: no row-refs element and no empty, collapsed, or expandable reference element in any row (TC-055, TC-056; REQ-06.019 AC-1, MS-028 AC-1).
- [ ] Visibility is resolved from the descriptor content model: Generic omits the block, Typed and HtmlMockup families keep it; no slug match, presentation override, per-family template, or new presentation field is added (ADR-035, TECH-074 5.7).
- [ ] A document type configured after the change gets the behaviour with no per-type presentation configuration (TC-056, ADR-035).
- [ ] Labels keep rendering in the Title cell for every family, including Generic (TC-055, TC-056).
- [ ] Row.references still carries the RecordView reference projection for every family; only the renderer omits the block for Generic (ADR-035, TECH-074 5.1).
- [ ] Typed family lists keep the reference list in the Title cell for tasks, decisions, milestones, releases, requirements, NFRs, audit records, and business rules (TC-061, MS-028 AC-4).
- [ ] The HtmlMockup family list (UI Mockups) keeps the reference list in the Title cell (TC-062, MS-028 AC-4).
- [ ] The Generic list is otherwise unchanged: fixed column order, no family slot, status filter, grouped rows where applicable, View then Edit actions, page width, and shell (TC-057).
- [ ] The Generic single-document view keeps its collapsible References section with all references, plus labels, body, and metadata (TC-058, MS-028 AC-2).
- [ ] CLI list and view keep references for config-defined document types (TC-059, MS-028 AC-3).
- [ ] The JSON API keeps references in stored order for config-defined document types, with other metadata unchanged (TC-060, MS-028 AC-3, AC-6).
- [ ] The glossary landing page display is unchanged and gains no reference list (TC-063, MS-028 AC-5).
- [ ] Rendering modifies no record or front-matter value and no record or configuration file (TC-064, MS-028 AC-6).
- [ ] The views render entirely from local data with no network access (TC-065, MS-028 AC-7, NFR-01.003).
- [ ] The change is confined to the shared list renderer in list.rs (ListPlan::for_descriptor and the row renderer) called by list_page in render.rs; automated tests cover Generic omission, Typed and HtmlMockup retention, label retention, and the characterization replay split (TECH-074 5.7, 6).
- [ ] Traceability is recorded: REQ-06.019, RE-006, MS-028 AC-1 through AC-7, ADR-035, TECH-074, TECH-076, TECH-068, TECH-045, UI-019, MOCK-021, TC-055 through TC-065.
- [ ] No open material questions remain.
## Comments
- **task-refiner**: Refinement complete. Scope: in the shared browser list only, ContentModel::Generic (config-defined document type) rows omit the reference list entirely; ContentModel::Typed and ContentModel::HtmlMockup families keep references; labels stay in the Title cell for every family; visibility resolves from the descriptor content model with no slug match, presentation override, per-family template, or new presentation field (ADR-035, REQ-06.019). Single-document view, CLI list and view, JSON API, RecordView, front matter, glossary landing page, and the task board are unchanged. Code locations: the shared list renderer in list.rs (ListPlan::for_descriptor, render_table, row_for, render_row) called by list_page in render.rs; TASK-182 documentation currently names only render.rs and the orchestrator should extend it with list.rs. Affected tests: list.rs unit tests (title-cell helper), the list characterization replay (Generic family asserts no reference block while Typed and HtmlMockup keep it), plus a contract test using a generic record that carries references and labels. Assumptions: prerequisites met (TASK-178 Done with ADR-035 accepted and architect conformance recorded for TECH-074, TECH-076, TECH-068, TECH-045; TASK-179 Done with UI-019 verified and MOCK-021 approved; TASK-180 Done with TC-055 through TC-065 approved); config-defined types are exactly ContentModel::Generic; Row.references keeps the projection and only rendering omits the block; Rust 1.99.1, Firefox 115+, no-JavaScript, BR-004, NFR-01.003 apply. Traceability: REQ-06.019, RE-006, MS-028 AC-1 through AC-7, ADR-035, TECH-074, TECH-076, TECH-068, TECH-045, UI-019, MOCK-021, TC-055 through TC-065. Proposed split: none. Open questions: none.
