---
id: TASK-101
title: Revise the backlog browser specifications for the per-milestone kanban board
status: To Do
assignee: []
created_date: '2026-11-11T21:15:00Z'
labels:
  - specification
  - browser
  - release-1.0
milestone: m-16
dependencies:
  - TASK-86
references:
  - docs/requirements/business/REQ-06.013.md
  - docs/specifications/ui/browser-backlog-records-ui.md
  - docs/specifications/browser-tasks-milestones-and-releases.md
  - docs/specifications/ui/mockups/README.md
priority: high
type: docs
ordinal: 39035
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
REQ-06.013 was validated on 2026-11-11 and the backlog browser specifications still describe the earlier backlog list: releases as separators between a flat milestone list in browser-tasks-milestones-and-releases.md section 3, and a backlog view note in browser-backlog-records-ui.md section 4 that fixes neither the per-milestone board, the visible empty columns, the release grouping of milestones, the tasks-without-milestone section, card order and contents, nor a status control that works without dragging. Implementation and the mockup revision cannot start on those surfaces without a specification that states the new contract, so this task states it and names the mockup work it requires.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The backlog browser specification defines the per-milestone kanban board: each expanded milestone shows its tasks in one column per configured task status, in configured order, and statuses with no tasks keep their column visible.
- [ ] #2 The specification defines release grouping: milestones assigned to a release appear under that release, milestones with no release are grouped before the first release, and every release boundary is a bold separator that names the release and does not rely on colour.
- [ ] #3 The specification defines the section for tasks that belong to no milestone after all milestone sections, cards in ascending task ID order showing ID and title and linking to the record view, and the exclusion of archived tasks and milestones.
- [ ] #4 The specification defines status change from the board: dragging a card between columns and an equivalent status control on the card that needs no dragging, immediate persistence of the status value only through the existing browser write contract (ADR-0006 version token, ADR-0013 single write path), all other task fields preserved, and the card left in its original column with the failure reported when persistence fails.
- [ ] #5 No revised section still describes releases as separators between a flat milestone list, or status change as drag-only, and the specification names the mockup revision that must be approved before implementation.
<!-- AC:END -->
