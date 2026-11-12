---
id: TASK-103
title: Render the backlog as a per-milestone kanban board with card status change
status: To Do
assignee: []
created_date: '2026-11-11T21:15:00Z'
labels:
  - browser
  - release-1.0
milestone: m-16
dependencies:
  - TASK-97
  - TASK-101
  - TASK-102
references:
  - docs/requirements/business/REQ-06.013.md
  - docs/specifications/ui/browser-backlog-records-ui.md
  - docs/specifications/browser-tasks-milestones-and-releases.md
  - >-
    docs/architecture/adr/ADR-0006-optimistic-concurrency-and-atomic-record-writes.md
  - >-
    docs/architecture/adr/ADR-0013-shared-record-envelope-without-a-storage-layer.md
priority: high
type: feature
ordinal: 39145
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The browser backlog list is a flat per-category list, while REQ-06.013 requires each milestone to expand into a kanban board and requires status change on the board to persist immediately. The board is a new list surface plus a status-only write from that surface: it must reuse the single browser write contract and the shared record writer that TASK-97 leaves in place instead of adding a second write path, and client-side drag-and-drop is new for the list surface but introduces no new module, stack, or non-functional requirement. Implementing it after TASK-97 avoids building the write on the browser helpers that TASK-97 deletes.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The Backlog view keeps milestones expandable and collapsible, and an expanded milestone shows its tasks as a kanban board with one column per configured task status, in configured order, with columns for statuses that have no tasks still visible.
- [ ] #2 Milestones assigned to a release are grouped under that release, milestones with no release are grouped before the first release, each release boundary is a bold separator that names the release and does not rely on colour, and tasks with no milestone appear in their own expandable section after all milestone sections.
- [ ] #3 Each task card shows the task ID and title, links to the task record view, and cards appear in ascending task ID order within a column; archived tasks and milestones are not shown.
- [ ] #4 A task status is changed either by dragging its card into another status column or with the card's equivalent status control, both persist the status immediately through the existing browser write contract with the version token and the shared record writer, the change survives a reload, and only the status field changes.
- [ ] #5 When persistence fails, the card stays in its original status column and the failure is reported to the user.
- [ ] #6 Tests cover board rendering for every configured status including empty columns, release and no-release grouping, the no-milestone section, card order and links, both status-change paths and their persistence, the persistence-failure path, archived exclusion, and a status control operable without a pointer at 320 px.
<!-- AC:END -->
