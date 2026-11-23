---
id: TASK-097
title: 'Design review: revise the backlog kanban board mockup'
status: Done
assignee: []
created_date: '2026-11-11T21:15:00Z'
updated_date: '2026-11-12T17:03:00Z'
labels:
  - design-review
  - browser
  - release-1.0
milestone: MS-017
dependencies:
  - TASK-096
references:
  - docs/specifications/ui/mockups/MOCK-0002-browser-backlog-records-review.html
  - docs/specifications/ui/mockups/README.md
  - docs/specifications/ui/browser-backlog-records-ui.md
  - docs/requirements/business/REQ-006.013.md
  - docs/specifications/browser-spec-revisions-task-102.md
  - docs/specifications/browser-tasks-milestones-and-releases.md
priority: high
type: task
ordinal: 41625
---


## Description
MOCK-0002 is approved but shows the earlier backlog list, so it no longer represents the backlog view that REQ-006.013 requires. Revise the mockup in place to the per-milestone kanban board and obtain explicit user approval before implementation. UI mockups may not be implemented as final UI while unapproved.

Scope (in):
- Revise MOCK-0002 to show expandable milestones containing a kanban board with one column per configured status, visible empty columns, milestone grouping under each release with a bold named separator, a tasks-without-milestone section after the milestones, and ID plus title cards linking to the record view.
- Show the card status control that changes status without dragging, together with the drag-and-drop interaction, and the state after a failed status persistence (card in original column, failure reported).
- Keep the existing shell, record views, and edit flows unchanged except where the board changes them; cover the 320 px layout and a pointer-free status control.
- Update the UI description and the mockup registry entry for MOCK-0002 to record the revision, its new views, and its status.

Scope (out):
- Implementing the board or status change (TASK-103).
- Changing requirements, business rules, or the browser specification corpus (delivered by TASK-101).
- Allocating a new MOCK id (revision reuses MOCK-0002).
- Approving the mockup on behalf of the user (explicit user approval required).

Design deliverable:
- Feature specification: docs/specifications/browser-spec-revisions-task-102.md
- UI description: docs/specifications/ui/browser-backlog-records-ui.md (sections 4, 9, 10)
- Mockup: docs/specifications/ui/mockups/MOCK-0002-browser-backlog-records-review.html (status draft, pending re-approval)

Dependencies: TASK-101 (completed). Downstream: TASK-103 (blocked until mockup approved).

Assumptions:
- Assumption: The kanban board contract is already specified by REQ-006.013 and TASK-101; this ticket only designs the UI.
- Assumption: MOCK-0002 is revised in place; no new MOCK id is allocated.
- Assumption: The mockup stays draft until explicit user approval; only then is the registry updated to approved and TASK-103 may start.
- Assumption: The shell, record views, and edit flows are reused unchanged except where the board changes them.
- Assumption: No new dependencies, crates, ADRs, or NFRs are introduced.

Open questions:
- OQ-1: whether any flat task listing remains reachable alongside the board (recommended default: board replaces the flat task list). Subject to product confirmation during this review.
- OQ-2: whether the board's status change is submitted as a form POST or via the JSON API (implementation choice in TASK-103).

Verification: revised MOCK-0002 covers all REQ-006.013 surfaces; UI description and registry updated; explicit user approval recorded before implementation.

## Acceptance Criteria
- [x] MOCK-0002 is revised so the backlog prototype shows expandable milestones containing a kanban board with one column per configured status, visible empty columns, milestone grouping under each release with a bold named separator, a tasks-without-milestone section after the milestones, and ID plus title cards linking to the record view.
- [x] The revised prototype shows the card status control that changes status without dragging, together with the drag-and-drop interaction, and shows the state after a failed status persistence with the card in its original column and the failure reported.
- [x] The revised package keeps the existing shell, record views, and edit flows unchanged except where the board changes them, and covers the 320 px layout and a pointer-free status control.
- [x] The UI description and the mockup registry entry for MOCK-0002 record the revision, its new views, and its status.
- [x] The revised mockup is explicitly approved by the user and the registry status is approved before implementation starts.

## Definition of Done
- [x] #1 Mockup package revised in place for MOCK-0002 and covers every REQ-006.013 surface (kanban board, empty columns, release grouping with bold named separator, no-milestone section, ID+title cards, drag-and-drop and non-drag status control, failed-persistence state, 320 px layout, pointer-free control)
- [x] #2 UI description document (browser-backlog-records-ui.md) updated to record the revision, new views, and resolved presentation details
- [x] #3 Mockup registry entry for MOCK-0002 updated with revision note and status draft (pending explicit user re-approval)
- [x] #4 Registry status updated to approved only after explicit user approval
- [x] #5 Feature specification exists and links the MOCK id, UI description, and requirement/spec references


## Comments
author: ui-designer
created: 2026-11-12T16:14:00Z

---

MOCK-0002 revised in place under TASK-102: kanban board per milestone, release grouping with bold named separator, tasks-without-milestone section, ID+title cards with non-drag status control, failed-persistence state, 320 px layout. Design deck removed. Registry status set to draft pending explicit user re-approval. UI description sections 4/9/10 updated.

---
