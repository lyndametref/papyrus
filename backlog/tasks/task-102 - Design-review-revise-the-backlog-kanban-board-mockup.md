---
id: TASK-102
title: 'Design review: revise the backlog kanban board mockup'
status: To Do
assignee: []
created_date: '2026-11-11T21:15:00Z'
labels:
  - design-review
  - browser
  - release-1.0
milestone: m-16
dependencies:
  - TASK-101
references:
  - docs/specifications/ui/mockups/MOCK-0002-browser-backlog-records-review.html
  - docs/specifications/ui/mockups/README.md
  - docs/specifications/ui/browser-backlog-records-ui.md
  - docs/requirements/business/REQ-06.013.md
priority: high
type: task
ordinal: 39141.25
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
MOCK-0002 is approved but shows the earlier backlog list, so it no longer represents the backlog view that REQ-06.013 requires. The mockup must be revised to the per-milestone kanban board and explicitly approved by the user before the board is implemented; UI mockups may not be implemented as final UI while unapproved.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 MOCK-0002 is revised so the backlog prototype shows expandable milestones containing a kanban board with one column per configured status, visible empty columns, milestone grouping under each release with a bold named separator, a tasks-without-milestone section after the milestones, and ID plus title cards linking to the record view.
- [ ] #2 The revised prototype shows the card status control that changes status without dragging, together with the drag-and-drop interaction, and shows the state after a failed status persistence with the card in its original column and the failure reported.
- [ ] #3 The revised package keeps the existing shell, record views, and edit flows unchanged except where the board changes them, and covers the 320 px layout and a pointer-free status control.
- [ ] #4 The UI description and the mockup registry entry for MOCK-0002 record the revision, its new views, and its status.
- [ ] #5 The revised mockup is explicitly approved by the user and the registry status is approved before implementation starts.
<!-- AC:END -->
