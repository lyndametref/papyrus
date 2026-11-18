---
id: TASK-173
title: "Refine removing browser sections implementation"
created_date: '2026-11-16T21:03:23Z'
updated_date: '2026-11-18T20:36:20Z'
labels: []
references:
  - REQ-06.018
  - RE-005
  - REQ-06.005
  - REQ-06.008
  - NFR-01.002
  - ADR-034
  - TECH-080
  - UI-018
  - MOCK-020
  - BR-003
  - BR-004
  - TC-033
  - TC-034
  - TC-035
  - TC-036
  - TC-037
  - TC-038
  - TC-039
  - REQ-06.003
status: Done
assignee:
  - task-refiner
milestone: MS-027
dependencies:
  - TASK-172
ordinal: 40
documentation:
  - src/browser/render.rs
  - src/browser/mod.rs
  - src/browser/routes.rs
type: refinement
---

## Description
Outcome: confirm all information needed to implement removal of the Configuration, Managed artifacts and lifecycles, and Agent skills and workflows sections from the browser (MS-027) is available and complete the refinement fields. Scope: remove browser navigation entries and routes for Configuration, Managed artifacts and lifecycles, and Agent skills and workflows; remove the browser configuration view and its edit endpoint and query alias so configuration editing is available only through the CLI (papyrus config); remove the managed-artifacts-and-lifecycles page and the agent-skills page and their query aliases; remove their inventory hub cards and sidebar utility entries; keep underlying record and configuration files unchanged; keep managed-artifact and skill documentation available outside the browser; keep all other browser pages, navigation, and routes working. Out of scope: changing records or configuration files, adding restore commands, and any change to dependencies, ordinals, milestone scope, or the plan. Assumptions: ADR-034 is accepted and TECH-080 conformance was reviewed as conformant in TASK-170 (Done); TASK-171 is Done with MOCK-020 approved; TASK-172 is Done with TC-033 through TC-039 approved; dependency TASK-172 is Done; TASK-174 and TASK-175 track execution and verification; BR-003 governs papyrus config behavior and BR-004 governs papyrus browser behavior; Rust toolchain available; Firefox 115+ and no-JavaScript support required (NFR-01.002). Code locations: browser module route dispatch and handlers; browser render module sidebar and page render functions; browser routes helper used by the removed lifecycle page (assess retention). Affected tests and no_drift scanner expectations that assert removed surfaces may need updates during implementation. Traceability: REQ-06.018, RE-005, REQ-06.003, REQ-06.005, REQ-06.008, NFR-01.002, ADR-034, TECH-080, UI-018, MOCK-020, BR-003, BR-004, TC-033 through TC-039, MS-027. Open questions: none. Definition of done: all acceptance criteria are satisfied by TASK-174 and verified by TASK-175; cargo fmt, clippy, and tests pass; no open material questions remain.
## Acceptance Criteria
- [ ] Browser navigation contains no entry for Configuration, Managed artifacts and lifecycles, or Agent skills and workflows; remaining entries are present and usable (TC-033).
- [ ] Routes for Configuration, Managed artifacts and lifecycles, and Agent skills and workflows and their query aliases are removed and unreachable; requests do not serve the removed pages (TC-034).
- [ ] No browser configuration editing view exists; configuration reads and writes occur only through the CLI, including folder moves (TC-035, BR-003, BR-004).
- [ ] Managed artifact and lifecycle documentation remains available outside the browser and is unchanged by the removal (TC-036).
- [ ] Agent skills and workflows documentation remains available outside the browser and is unchanged by the removal (TC-037).
- [ ] Underlying record files and the configuration file remain unchanged, including identifiers, prefixes, and folders (TC-038).
- [ ] Other browser pages, navigation, and routes continue to work and records remain viewable and editable (TC-039).
- [ ] The sidebar no longer renders utility entries for Configuration, Managed artifacts and lifecycles, or Agent skills and workflows.
- [ ] The inventory hub no longer renders cards for Managed artifacts and lifecycles or Agent skills and workflows.
- [ ] No stale links to removed routes remain in the browser module, including the lifecycle page link to Configuration.
- [ ] Implementation is confined to the existing browser module files and introduces no new dependency, asset, or template engine (ADR-034, TECH-080).
- [ ] Traceability is recorded: REQ-06.018, RE-005, REQ-06.003, REQ-06.005, REQ-06.008, NFR-01.002, ADR-034, TECH-080, UI-018, MOCK-020, BR-003, BR-004, TC-033 through TC-039, MS-027.
- [ ] No open material questions remain.
## Comments
- **task-refiner**: Refinement complete. Scope: remove Configuration, Managed artifacts and lifecycles, and Agent skills and workflows navigation entries and routes from the browser; configuration editing remains CLI-only via papyrus config; managed-artifact and skill documentation remains outside the browser; underlying records and configuration files unchanged; other pages continue to work. Assumptions: ADR-034 accepted, TECH-080 conformance conformant, TASK-171 Done with MOCK-020 approved, TASK-172 Done with TC-033 through TC-039 approved, TASK-172 dependency Done, TASK-174 and TASK-175 track execution and verification, BR-003 and BR-004 apply, Rust toolchain available, Firefox 115+ and no-JavaScript support required. Code locations: browser module route dispatch and handlers, browser render module sidebar and page render functions, browser routes helper used by the removed lifecycle page (assess retention). Traceability: REQ-06.018, RE-005, REQ-06.003, REQ-06.005, REQ-06.008, NFR-01.002, ADR-034, TECH-080, UI-018, MOCK-020, BR-003, BR-004, TC-033 through TC-039, MS-027. Open questions: none. Proposed split: none.
