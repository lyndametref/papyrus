---
id: TASK-167
title: "Refine uniform single-document browser layout implementation"
created_date: '2026-11-16T21:03:15Z'
updated_date: '2026-11-19T10:53:58Z'
labels: []
references:
  - REQ-006.012
  - RE-004
  - BR-004
  - NFR-005.001
  - NFR-001.002
  - ADR-033
  - TECH-079
  - UI-017
  - MOCK-019
  - TC-024
  - TC-025
  - TC-026
  - TC-027
  - TC-028
  - TC-029
  - TC-030
  - TC-031
  - TC-032
status: Done
assignee:
  - task-refiner
milestone: MS-026
dependencies:
  - TASK-166
ordinal: 40
documentation:
  - src/browser/render.rs
  - src/browser/mod.rs
  - src/browser/routes.rs
type: refinement
---

## Description
Outcome: confirm all information for implementing the uniform single-document browser layout (MS-026) is available and complete the refinement fields. Scope: implement one shared rendering path for every managed Markdown record single view with the fixed metadata order and rules from REQ-006.012, RE-004, ADR-033, TECH-079, UI-017, and MOCK-019: - small record ID above the title; - title big and bold; - status beside the ID in a label element (color deferred); - created and updated dates as small text directly under the title before references; - references, dependencies, and dependents as separate collapsible native details sections under the title, each shown only when non-empty and collapsed by default; - no document type repetition; - all metadata under artifact content. UI mockup HTML views are excluded. Rendering consumes the shared RecordView metadata projection (REQ-006.004, ADR-0013, ADR-0020); no browser-only projection and no new dependency, asset, or template engine. Implementation surfaces: render.rs, mod.rs, and routes.rs in the browser module. Out of scope: status label colors (deferred to a future change), UI mockup view changes, and any change to dependencies, ordinals, milestone scope, or plan. Assumptions: - Architecture impact assessed and user-verified (ADR-033 accepted). - TECH-079 conformance reviewed. - UI design task TASK-165 is Done with MOCK-019 approved. - Test design task TASK-166 is Done with TC-024 through TC-032 approved. - Dependency TASK-166 is Done. - Implementation task TASK-168 and verification task TASK-169 track execution and verification. - Rust 1.99.1 toolchain available (ADR-0001). - Firefox 115+ and no-JavaScript support required (NFR-005.001, NFR-001.002). - Browser command behavior governed by BR-004. Open questions: none. Definition of done: all 15 acceptance criteria are satisfied by TASK-168 and verified by TASK-169; cargo fmt, clippy, and tests pass; no open material questions remain.
## Acceptance Criteria
- [ ] Small record ID is rendered above the title in every managed Markdown single record view (TC-024, TC-025).
- [ ] Title is rendered big and bold in every managed Markdown single record view (TC-024, TC-025).
- [ ] Status is rendered beside the ID in a label element, with color support deferred (TC-024, TC-025).
- [ ] Created and updated dates are rendered as small text directly under the title before references (TC-024, TC-025).
- [ ] References, dependencies, and dependents are rendered as separate collapsible sections under the title (TC-024, TC-027).
- [ ] Each collapsible section appears only when its collection is non-empty (TC-026).
- [ ] Collapsible sections use native details and summary elements, collapsed by default, with a visible summary (TC-027).
- [ ] The single record view renders and remains usable with JavaScript disabled (TC-028).
- [ ] The document type is not repeated anywhere on the view (TC-024, TC-025).
- [ ] All metadata elements are rendered under the artifact content (TC-024, TC-025).
- [ ] The uniform layout applies to every managed Markdown record type with a single view; UI mockup HTML views are excluded (TC-025, TC-029).
- [ ] The layout renders in Firefox 115 and later without layout breakage (TC-030).
- [ ] All Papyrus content remains viewable and editable with the uniform layout (TC-031).
- [ ] Reference values resolve to hyperlinks when possible and unresolved values render as plain text without error (TC-032).
- [ ] Implementation uses the shared rendering path in render.rs, mod.rs, and routes.rs consuming the shared RecordView projection, with no new dependency, asset, or template engine (ADR-033, TECH-079, UI-017).
## Comments
- **task-refiner**: Refinement complete. Scope: implement the uniform single-document browser layout per REQ-006.012, RE-004, ADR-033, TECH-079, UI-017, and MOCK-019 with the fixed metadata order, native collapsible sections, no document type repetition, and metadata under content; UI mockup HTML views excluded. Assumptions: ADR-033 accepted, TECH-079 conformance reviewed, TASK-165 Done with MOCK-019 approved, TASK-166 Done with TC-024 through TC-032 approved, TASK-166 dependency Done, TASK-168 and TASK-169 track execution and verification, Rust 1.99.1 available, Firefox 115+ and no-JavaScript support required, BR-004 applies. Code locations: render.rs, mod.rs, routes.rs in the browser module. Traceability: REQ-006.012, RE-004, BR-004, NFR-005.001, NFR-001.002, ADR-033, TECH-079, UI-017, MOCK-019, TC-024 through TC-032, MS-026. Open questions: none. Proposed split: none.
