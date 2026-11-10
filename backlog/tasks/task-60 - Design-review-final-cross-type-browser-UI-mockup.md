---
id: TASK-60
title: 'Design review: final cross-type browser UI mockup'
status: Done
assignee:
  - '@ui-designer'
created_date: '2026-11-04T19:21:00Z'
updated_date: '2026-11-10T17:40:00Z'
labels:
  - design-review
  - ui-design
milestone: m-12
dependencies:
  - TASK-59
  - TASK-66
  - TASK-76
references:
  - docs/requirements/business/REQ-06.002.md
  - docs/requirements/business/REQ-06.003.md
  - docs/requirements/business/REQ-06.004.md
  - docs/requirements/business/REQ-05.010.md
  - docs/requirements/business/REQ-05.011.md
  - docs/requirements/non-functional/NFR-01.002.md
  - docs/requirements/non-functional/NFR-01.003.md
  - docs/requirements/non-functional/NFR-02.002.md
  - docs/requirements/non-functional/NFR-04.001.md
  - docs/requirements/non-functional/NFR-04.002.md
  - docs/requirements/non-functional/NFR-05.001.md
  - >-
    docs/architecture/adr/ADR-0007-yaml-front-matter-as-single-record-metadata-source.md
  - >-
    docs/architecture/adr/ADR-0009-shared-record-envelope-and-unified-write-path.md
  - docs/specifications/browser-requirements.md
  - docs/specifications/browser-managed-artifacts-and-lifecycles.md
  - docs/specifications/ui/browser-front-matter-and-lifecycle-ui.md
  - docs/specifications/ui/browser-agent-skills-ui.md
  - >-
    docs/specifications/ui/mockups/MOCK-0010-browser-front-matter-and-lifecycle-review.html
  - docs/specifications/ui/mockups/MOCK-0011-browser-agent-skills-review.html
  - >-
    docs/specifications/ui/mockups/MOCK-0012-browser-final-cross-type-review.html
  - docs/specifications/ui/mockups/README.md
  - docs/specifications/ui/browser-final-cross-type-ui.md
documentation:
  - docs/specifications/ui/browser-final-cross-type-ui.md
  - >-
    docs/specifications/ui/mockups/MOCK-0012-browser-final-cross-type-review.html
  - docs/specifications/ui/mockups/README.md
priority: medium
type: task
ordinal: 30095
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Design review for the final cross-type browser navigation/inventory mockup required before TASK-19 (Complete browser access for all Papyrus content) implementation. This ticket delivers the UI review package (MOCK-0012, draft, pending explicit user approval) and its UI description (docs/specifications/ui/browser-final-cross-type-ui.md), consolidating prior mockups MOCK-0001..MOCK-0011 into one coherent navigation without modifying them. No backend implementation is performed in this ticket.

Scope:
- Part A - Complete navigation/inventory: final sidebar in established order (Business requirements, Business rules, Audit records, Technical specifications, Non-functional requirements, Backlog, Decisions, Glossary, UI specifications, UI mockups, Managed artifacts & lifecycles, Agent skills & workflows); all 12 entries present; represented types clickable, others non-clickable context; deprecated records not reachable (OQ-4).
- Part B - Front-matter metadata display: reuse MOCK-0010 pattern on a representative subset (task, milestone, release, technical specification) showing at least id, title, created_date, updated_date, labels, references, and type-specific keys; timestamps read-only in edit; metadata edits target front matter, not body sections.
- Part C - Lifecycle page: one section per fixed record type (12); task states from configured statuses; documented lifecycles for other types; excludes Backlog.md docs, drafts, and project configuration files (reuses MOCK-0010).
- Part D - Agent skills page: read-only, config-parameterized (reuses MOCK-0011).
- Part E - Cross-type consistency: inventory hub listing all record types with counts; reused approved shell, error pages, responsive 320px views.

Out of scope: backend/CLI implementation (TASK-19, TASK-67, TASK-72, TASK-77), packaging (TASK-20), modifying approved MOCK-0001..MOCK-0011, new dependencies/crates/ADRs.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Mockup package MOCK-0012 created under docs/specifications/ui/mockups/ and registered in mockups/README.md for the final cross-type browser navigation/inventory.
- [ ] #2 UI description document docs/specifications/ui/browser-final-cross-type-ui.md records layout, complete sidebar inventory (all 12 document types in final order), screen map, inventory hub, front-matter metadata display, lifecycle page, agent skills page, active elements, open questions OQ-1..OQ-5, assumptions, approval status.
- [ ] #3 Mockup and UI description receive explicit user approval (status approved) before implementation of TASK-19.
- [ ] #4 Navigation entries and ordering are consistent with REQ-06.003 and prior approved mockups (MOCK-0010, MOCK-0011).
- [ ] #5 Front-matter metadata display covers id, title, created_date, updated_date, labels, references, and type-specific keys on a representative subset (task, milestone, release, technical specification), with read-only timestamps in edit forms.
- [ ] #6 Lifecycle page has one section per fixed record type (12) with task states from configured statuses and documented lifecycles for other types; excludes Backlog.md docs, drafts, and project configuration files.
- [ ] #7 Agent skills page is included (read-only, config-parameterized) per REQ-06.005.
- [ ] #8 Mockup is self-contained, clickable, consistent with approved MOCK-0001 shell; validate_mockups.sh reports no new errors attributable to MOCK-0012.
- [ ] #9 Approved MOCK-0001..MOCK-0011 are not modified in place.
- [ ] #10 No implementation code, new dependencies, crates, or ADRs introduced.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Mockup package MOCK-0012 created and registered per ui-clickable-mockup.
- [ ] #2 Registry docs/specifications/ui/mockups/README.md updated with MOCK-0012 row and notes.
- [ ] #3 UI description document created/updated under docs/specifications/ui/ (browser-final-cross-type-ui.md).
- [ ] #4 No code, dependency, or ADR changes introduced.
- [ ] #5 User approval explicitly recorded (date and ticket) in registry and UI description before TASK-19 implementation.
- [ ] #6 validate_mockups.sh reports no new errors attributable to MOCK-0012.
<!-- DOD:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. User reviews MOCK-0012 and UI description.
2. On explicit approval, set registry and UI description status to approved (date + ticket).
3. Hand off to TASK-19 implementation.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Assumptions:
- Assumption: The final package demonstrates complete navigation/inventory and cross-type consistency; full per-type CRUD remains covered by prior mockups MOCK-0001..MOCK-0011 (OQ-1).
- Assumption: A single inventory/landing view lists all record types with illustrative counts (OQ-2).
- Assumption: Sidebar ordering follows MOCK-0010/MOCK-0011 established order (OQ-3).
- Assumption: Deprecated records are not reachable from final navigation (OQ-4).
- Assumption: Responsive 320px coverage includes list, edit, lifecycle, and skills views (OQ-5).
- Assumption: Metadata display reuses MOCK-0010 pattern on representative subset (task, milestone, release, technical specification); the same metadata block applies to every managed record type.
- Assumption: MOCK-0012 remains draft until explicit user approval; TASK-19 must not implement against it until approved.
- Assumption: validate_mockups.sh reports 2 pre-existing errors (MOCK-0001/MOCK-0002 design deck) unrelated to MOCK-0012; no new errors attributable to MOCK-0012.

Open questions (OQ-1..OQ-5) recorded in docs/specifications/ui/browser-final-cross-type-ui.md with recommended defaults; pending user confirmation/approval.
<!-- SECTION:NOTES:END -->

## Comments

<!-- COMMENTS:BEGIN -->
author: @task-refiner
created: 2026-11-10T14:05:00Z

---

Refinement completed on 2026-11-10. @ui-designer produced MOCK-0012 (draft) and docs/specifications/ui/browser-final-cross-type-ui.md; registry updated. Mockup awaits explicit user approval before TASK-19 implementation. Status set to Implemented pending approval/validation.

---
<!-- COMMENTS:END -->
