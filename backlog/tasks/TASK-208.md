---
id: TASK-208
title: "Refine lowest-ordinal milestone ordering and cleanup implementation"
created_date: '2026-11-19T11:02:52Z'
updated_date: '2026-11-19T18:44:36Z'
labels: []
references:
  - REQ-006.020
  - REQ-006.021
  - REQ-006.022
  - REQ-006.013
  - RE-008
  - TECH-011
  - TECH-062
  - UI-002
  - MOCK-022
  - BR-004
  - ADR-0005
  - ADR-0006
  - ADR-0010
  - ADR-0012
  - ADR-0013
  - ADR-0022
  - ADR-0032
  - TC-066
  - TC-067
  - TC-068
  - TC-069
  - TC-070
  - TC-071
  - TC-072
  - TC-073
  - TC-074
  - TC-075
  - TC-076
  - TC-077
  - TC-078
  - TC-079
  - TC-080
  - TC-081
  - TC-082
  - TC-083
  - TC-084
  - TC-085
  - TC-086
  - TC-087
  - TC-088
  - TC-089
  - TC-090
  - TC-091
  - TC-092
  - TC-093
  - TC-094
  - TC-095
  - TC-096
  - TC-097
status: Done
assignee:
  - task-refiner
milestone: MS-030
dependencies:
  - TASK-207
ordinal: 40
documentation:
  - docs/specifications/technical/TECH-011.md
  - docs/specifications/technical/TECH-062.md
  - docs/specifications/ui/browser-backlog-records-ui.md
  - src/browser/render.rs
  - src/browser/mod.rs
  - src/browser/records.rs
  - src/task.rs
  - tests/browser_cli.rs
type: refinement
---

## Description
Outcome: confirm all information needed to implement lowest-ordinal milestone ordering, lifecycle markers, and cleanup controls in the browser Backlog board is available, and complete the refinement fields for TASK-209.

Scope: refine TASK-209 so it is self-contained and implementation-ready. The implementation covers REQ-006.020 (ordering milestones and releases by the lowest ordinal of their non-archived tasks, ascending), REQ-006.021 (open and closed lifecycle markers before titles), and REQ-006.022 (browser-only cleanup controls with confirmation and partial-failure handling) in the combined Backlog board only. The ascending lowest-ordinal rule supersedes the earlier highest-ordinal descending rule.

Prerequisites verified:
- Architecture impact assessed and user-verified: TASK-205 Done; TECH-011 and TECH-062 revised to draft and conformance recorded as conformant by the architect; no new ADR.
- Technical specification conformance verified by the architect (TECH-011, TECH-062).
- UI design TASK-206 Done; UI-002 revised and verified; MOCK-022 approved by the user.
- Test design TASK-207 Done; TC-066 through TC-097 approved by the user.
- Dependency TASK-207 Done.

Implementation surface for TASK-209: task ordinal parsing in the task record loader, milestone and release lifecycle status parsing in the browser record loader, sorting and marker and cleanup rendering in the browser render module, and cleanup preview and confirmation handling in the browser request module. Archiving reuses the existing milestone archive entry point; no new CLI command, JSON API endpoint, write path, dependency, or ADR is introduced.

Assumptions:
- Managed milestones and releases always carry a status front-matter key; a missing value defaults to open for milestones and planned for releases.
- A missing or unparseable task ordinal is treated as having no ordinal and does not contribute to a sort key.
- Active (non-archived) tasks and milestones come from the active backlog directories; archived records are already excluded by the index.
- Milestone and release cleanup controls are confined to the combined Backlog board; the milestone and release category list pages are unchanged.
- Rust 1.99.1 toolchain available; Firefox 115+ and no-JavaScript support required; browser command behaviour governed by BR-004; views render only local data.
- TASK-210 tracks milestone verification.

Open questions: none.

Definition of done: TASK-209 has a self-contained description, testable acceptance criteria, references, and documentation; TASK-209 is execution-ready; refinement evidence is recorded as a comment on TASK-208; TASK-208 is marked Done.
## Acceptance Criteria
- [ ] #1 Prerequisite architecture impact assessment is settled and user-verified (TASK-205 Done; TECH-011 and TECH-062 draft verified; conformance conformant; no new ADR).
- [ ] #2 Prerequisite UI design is Done and its mockup is approved (TASK-206 Done; UI-002 verified; MOCK-022 approved).
- [ ] #3 Prerequisite test design is Done and its test cases are approved (TASK-207 Done; TC-066 through TC-097 approved).
- [ ] #4 Dependency TASK-207 is Done.
- [ ] #5 TASK-209 has a self-contained description stating scope, implementation surface, out-of-scope items, assumptions, and definition of done.
- [ ] #6 TASK-209 has testable acceptance criteria covering ordering, markers, cleanup, browser-only constraints, and verification.
- [ ] #7 TASK-209 references REQ-006.020, REQ-006.021, REQ-006.022, REQ-006.013, RE-008, TECH-011, TECH-062, UI-002, MOCK-022, BR-004, and TC-066 through TC-097.
- [ ] #8 TASK-209 documentation lists the implementation and test files it will touch.
- [ ] #9 No open material questions remain.
- [ ] #10 Refinement evidence is recorded as a comment on TASK-208.
## Comments
- **task-refiner**: Refinement complete. Prerequisites verified: TASK-205 Done (TECH-011 and TECH-062 draft verified, conformance conformant, no new ADR); TASK-206 Done (UI-002 verified, MOCK-022 approved); TASK-207 Done (TC-066 through TC-097 approved); dependency TASK-207 Done. TASK-209 updated with self-contained scope, implementation surface (task ordinal parsing, milestone and release status parsing, ordering, markers, cleanup controls and handlers), out-of-scope items, assumptions, definition of done, testable acceptance criteria, references, and documentation. TASK-209 remains ToDo and is execution-ready. No open material questions. Proposed split: none.
