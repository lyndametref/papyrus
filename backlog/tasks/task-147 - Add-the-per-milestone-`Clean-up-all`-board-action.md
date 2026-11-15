---
id: TASK-147
title: Add the per-milestone `Clean up all` board action
status: To Do
assignee:
  - '@rust-dev'
created_date: '2026-11-15T16:13:00Z'
labels:
  - browser
  - backlog
milestone: m-21
dependencies: []
references:
  - docs/requirements/business/REQ-06.013.md
  - docs/requirements/business/REQ-02.004.md
  - docs/requirements/business/REQ-05.007.md
  - src/browser/mod.rs
  - src/browser/render.rs
priority: high
ordinal: 41595
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why.** REQ-06.013 was revised on 2026-11-15: the final configured task status column of each milestone board must offer a "Clean up all" action that archives that column's tasks for the current milestone only, with a warning that lists the affected task IDs, a confirmation step, and per-task failure reporting. The board, its drag-and-status control, and its release separators already exist; the bulk action does not.

**What.** The board action, its confirmation flow, its per-task archive behaviour, and the visibility rule.

**In scope.**
- The action appears only in the final configured task status column, defined as the last entry of the configured task `statuses` list, and only when that status differs from `active`.
- Activating it shows a warning that lists the IDs of the tasks to be archived and requires confirmation; cancel archives nothing.
- On confirmation every task in that column for the current milestone is archived through the existing archive path (REQ-02.004, REQ-05.007); a failure is reported for that task and does not abort the remaining archives, and successfully archived tasks leave the board.
- Scope is the current milestone board only, never project-wide; archived tasks stay excluded from the board; task status changes and every other board behaviour are unchanged.
- No CLI equivalent and no JSON API in this release.
- Tests: visibility rule for a non-`active` and an `active` final status, cancel path, full success, partial failure with per-task reporting and no abort, milestone scoping with an identically named status column in another milestone, and archived exclusion after the action.

**Out of scope.** A CLI bulk command, project-wide archival, changes to task archive semantics, milestone archive behaviour (milestones have no archive lifecycle), and board layout changes beyond the button and warning.

**Assumptions.** The existing status submission path already resolves the current milestone and the configured status list; this task reuses it rather than adding a new write path. Button placement and dialog layout are design decisions.

**UI gate.** This task changes the board view. A UI mockup for the button and the confirmation warning must be approved through `ui-designer` before merge, or the refiner records a no-mockup waiver.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The final configured task status column offers a Clean up all action only when that status differs from active.
- [ ] #2 Activating the action lists the affected task IDs in a warning and requires confirmation, and cancel archives nothing.
- [ ] #3 On confirmation every task in that column for the current milestone is archived through the existing archive path.
- [ ] #4 An archive failure is reported for that task and does not abort the remaining archives, and archived tasks leave the board.
- [ ] #5 The action affects the current milestone only and never archives project-wide.
- [ ] #6 No CLI equivalent, JSON API, or change to task status editing is introduced.
- [ ] #7 Tests cover the visibility rule, cancel, full success, partial failure, milestone scoping, and archived exclusion.
- [ ] #8 An approved mockup for the button and warning, or a recorded no-mockup waiver, exists before merge.
<!-- AC:END -->
