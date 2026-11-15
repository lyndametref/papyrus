---
id: TASK-142
title: >-
  Migrate every artifact family list to the shared layout with characterization
  tests
status: To Do
assignee:
  - '@rust-dev'
created_date: '2026-11-15T16:12:00Z'
labels:
  - browser
  - shell
  - tests
milestone: m-21
dependencies:
  - TASK-141
references:
  - docs/requirements/business/REQ-06.016.md
  - docs/requirements/business/REQ-06.014.md
  - >-
    docs/architecture/adr/ADR-0028-browser-shell-shared-list-layout-content-width-and-collapsible-navigation.md
  - src/browser/render.rs
  - src/browser/templates/document_list.html
priority: high
ordinal: 41550
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why.** REQ-06.016 forbids a bespoke list view for any family. After TASK-141 the task family proves the shared layout, but the other families still render through `document_list.html` and per-family builders, and ADR-0028 constraint 1 requires that none of them survive.

**What.** Migrate every remaining list view to the shared layout, delete the second list path, and replay the characterization fixtures captured in TASK-141.

**In scope.**
- Migrate milestones, releases, decisions, requirements and NFRs (hierarchical group plus member rows in one table body), business rules, audit records, technical specifications, UI specifications, UI mockups, glossary terms, use cases, test cases, test reports, and every config-defined document type.
- Family slots become declared data only: task to Milestone and Assignee; milestone to Release; release to Status; decision to Type; requirement and NFR to Group; audit to Audit type; rule to Applicability; UI mockup to Task and Specs; generic document to none.
- Delete `document_list.html` and every per-family list builder; no `match` on a slug and no per-family list template may remain.
- Replay the TASK-141 fixtures per family and record any accepted visual difference explicitly.
- Add a test that enumerates every enabled descriptor and asserts its list route returns the shared marker and the same header sequence, plus a grouped-row test for requirements and NFRs and an archived-exclusion test for every family.
- Keep archived records out of every list through the existing enabled-only, active-only index; the layout never sees them.

**Out of scope.** Detail, edit, and error pages; CLI list output; width classes and navigation (TASK-144); math and diagram rendering (TASK-145, TASK-146); new columns that do not exist today.

**Assumptions.** ADR-0028 decision 2 is the slot table; ADR-0017 and ADR-0020 supply the descriptor and presentation data. Flattening a family-specific presentation is acceptable only when its field survives as a slot; no field is dropped silently.

**UI gate.** This task changes every list view. If refinement produces a UI mockup for the shared list, no merge happens before `ui-designer` approval; otherwise the refiner records a no-mockup waiver naming the reviewed fixtures.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Every list view for every family renders through the shared layout, including config-defined document types and disabled types having no views.
- [ ] #2 The header, row and column structure, and action placement are identical across families, and family fields appear only in designated slots.
- [ ] #3 document_list.html and every per-family list builder are deleted, with no slug match and no per-family list template left.
- [ ] #4 Requirements and NFRs render group rows and member rows in the same table body through the hierarchical row variant.
- [ ] #5 The TASK-141 fixtures are replayed per family, and any accepted visual difference is recorded in the task.
- [ ] #6 A test enumerates every enabled descriptor and asserts the shared marker plus the same header sequence on its list route.
- [ ] #7 Archived records stay excluded from every list view.
- [ ] #8 An approved mockup or a recorded no-mockup waiver exists before merge.
<!-- AC:END -->
