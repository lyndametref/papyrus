---
id: TASK-66
title: 'Design review: front-matter metadata and managed-artifact lifecycle browser UI'
status: Done
assignee:
  - '@ui-designer'
created_date: '2026-11-08T15:06:00Z'
updated_date: '2026-11-09T18:24:00Z'
labels:
  - design-review
  - ui-design
milestone: m-12
dependencies:
  - TASK-59
references:
  - docs/requirements/business/REQ-06.003.md
  - docs/requirements/business/REQ-06.004.md
  - docs/requirements/business/REQ-05.010.md
  - docs/requirements/business/REQ-05.011.md
  - docs/specifications/browser-tasks-milestones-and-releases.md
  - docs/specifications/ui/mockups/README.md
  - >-
    docs/specifications/ui/mockups/MOCK-0010-browser-front-matter-and-lifecycle-review.html
  - docs/specifications/ui/browser-front-matter-and-lifecycle-ui.md
  - >-
    docs/architecture/adr/ADR-0007-yaml-front-matter-as-single-record-metadata-source.md
  - >-
    docs/architecture/adr/ADR-0009-shared-record-envelope-and-unified-write-path.md
  - docs/specifications/browser-requirements.md
documentation:
  - docs/specifications/ui/browser-front-matter-and-lifecycle-ui.md
  - >-
    docs/specifications/ui/mockups/MOCK-0010-browser-front-matter-and-lifecycle-review.html
  - docs/specifications/ui/mockups/README.md
priority: medium
type: task
ordinal: 30092.5
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Design review for the browser changes required by REQ-06.004 (front-matter metadata in CLI and browser views) and REQ-06.003 (managed-artifact lifecycle browser page), under REQ-05.010/05.011 (YAML front matter and its maintenance). This ticket delivers the UI review package and its UI description, not implementation. Implementation is tracked by TASK-67 (metadata projection in CLI/browser views) and TASK-72 (lifecycle page).

Scope:
- Part A - Front-matter metadata display: review list, detail, and edit views for managed record types showing at least id, title, created_date, updated_date, labels, references, and type-specific keys. Timestamps are read-only in edit forms; the editable field set is unchanged; metadata edits target front matter, not body sections.
- Part B - Lifecycle page: one section per fixed record type (tasks, milestones, releases, decisions, business requirements, non-functional requirements, glossary terms, UI specifications, technical specifications, UI mockups, business rules, audit records). Task states/transitions are derived from configured statuses; other types use their documented lifecycles. Backlog.md docs, drafts, and project configuration files are excluded.
- Deliverables: MOCK-0010 review package (draft, pending explicit user approval), UI description docs/specifications/ui/browser-front-matter-and-lifecycle-ui.md, and a registry update.

Out of scope: browser/CLI implementation (TASK-67, TASK-72), packaging (TASK-20), and modifying approved MOCK-0001..MOCK-0009.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The review package covers list, detail, and edit views for managed record types, showing at least id, title, created_date, updated_date, labels, references, and type-specific keys.
- [ ] #2 Edit forms keep the current editable field set; created_date and updated_date are read-only; metadata edits target front matter, not body sections.
- [ ] #3 A lifecycle page prototype has one section for each fixed record type and shows task states from configured statuses and documented lifecycles for other types.
- [ ] #4 UI descriptions for the changed list/detail/edit views and the lifecycle page reference REQ-06.003 and REQ-06.004.
- [ ] #5 The mockup is registered in the mockup registry and marked approved only after explicit user approval.
- [ ] #6 The review package is self-contained, clickable, and consistent with the approved MOCK-0001 shell and navigation.
- [ ] #7 The lifecycle page excludes Backlog.md docs, drafts, and project configuration files.
- [ ] #8 MOCK-0010 and docs/specifications/ui/browser-front-matter-and-lifecycle-ui.md are produced and registered; status remains draft until explicit user approval.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Mockup package MOCK-0010 created and registered per ui-clickable-mockup.
- [ ] #2 Registry docs/specifications/ui/mockups/README.md updated with the MOCK-0010 row and notes.
- [ ] #3 UI description document created/updated under docs/specifications/ui/ (browser-front-matter-and-lifecycle-ui.md).
- [ ] #4 No code, dependency, or ADR changes introduced.
- [ ] #5 User approval explicitly recorded (date and ticket) in registry and UI description before TASK-67/TASK-72 implementation.
- [ ] #6 validate_mockups.sh reports no new errors attributable to MOCK-0010.
<!-- DOD:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Assumptions:
- Assumption: Metadata display is demonstrated on a representative subset of record types (task, milestone, release, technical specification); the same metadata block applies to every managed record type (OQ-3).
- Assumption: Lifecycle page route and navigation entry follow the recommended defaults in the UI description (OQ-1, OQ-2) pending user confirmation.
- Assumption: Timestamps are displayed as quoted ISO-8601 UTC strings as stored, read-only (OQ-4).
- Assumption: Labels and references use tag display in view and tag add/remove in edit, reusing the MOCK-0002 interaction (OQ-5).
- Assumption: Lifecycle sections show state names, transitions, and a one-line description per state (OQ-6).
- Assumption: MOCK-0010 remains draft until explicit user approval; TASK-67 and TASK-72 must not implement against it until approved.

Open questions (carried from UI description docs/specifications/ui/browser-front-matter-and-lifecycle-ui.md):
- OQ-1: Sidebar label and placement for the lifecycle page entry ("Managed artifacts & lifecycles", end of list recommended).
- OQ-2: Route value for the lifecycle page (recommended "lifecycle"; ?type= vs distinct route).
- OQ-3: Whether to demonstrate metadata on all types or a representative subset (main scope question).
- OQ-4: Timestamp display format (ISO-8601 recommended).
- OQ-5: Tag vs plain-text display for labels/references.
- OQ-6: Lifecycle section depth.

Deliverables produced by @ui-designer on 2026-11-09 (UTC):
- MOCK-0010: docs/specifications/ui/mockups/MOCK-0010-browser-front-matter-and-lifecycle-review.html (draft, 21 web views).
- UI description: docs/specifications/ui/browser-front-matter-and-lifecycle-ui.md.
- Registry updated: docs/specifications/ui/mockups/README.md (MOCK-0010 row + notes).
Mockup previewed via browser.preview; awaiting explicit user approval before TASK-67/TASK-72 implementation.
<!-- SECTION:NOTES:END -->

## Comments

<!-- COMMENTS:BEGIN -->
author: @task-refiner
created: 2026-11-09T18:19:00Z

---

Refinement completed on 2026-11-09. @ui-designer produced MOCK-0010 (draft) and docs/specifications/ui/browser-front-matter-and-lifecycle-ui.md; registry updated. Mockup awaits explicit user approval before TASK-67/TASK-72 implementation. Status set to Implemented pending approval/validation.

---
<!-- COMMENTS:END -->
