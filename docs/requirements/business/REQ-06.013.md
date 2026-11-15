---
id: REQ-06.013
title: "Present the Backlog as a Per-Milestone Kanban Board"
created_date: '2026-11-15T15:46:30Z'
updated_date: '2026-11-15T15:57:53Z'
labels: []
references: []
---
# REQ-06.013 Present the Backlog as a Per-Milestone Kanban Board

## User Story
As a project contributor, I want the browser Backlog view to show tasks as a kanban board inside each milestone, with the release boundary marked by a bold separator, so that I can see how work is distributed across statuses, milestones, and releases at a glance.

## Group
REQ-06 Search and Local Web Interface

## Criticality
Should have

## Description
The browser Backlog view keeps milestones as expandable and collapsible sections. Inside an expanded milestone, that milestone's tasks are presented as a kanban board: one column per task status, laid out horizontally, with the columns therefore grouped horizontally within the milestone. Columns follow the configured task status list exactly, in configured order, and statuses with no tasks keep their column visible. Milestones that belong to a release appear under that release; milestones that belong to no release are grouped before the first release. Each release boundary is marked by a bold separator that names the release, so the boundary does not depend on colour. Tasks that belong to no milestone appear in their own expandable section after all milestone sections. Cards are ordered by ascending task ID within a column, consistent with the existing browser list ordering, and each card links to the task record view.

A task's status is changed by dragging its card into another status column, or with an equivalent status control on the card that does not require dragging. Either action persists the status immediately to the task record; the board is not a staging area with a separate save step. A change that cannot be persisted leaves the card in its original column and reports the failure to the user. Changing status from the board changes only the status value and preserves every other field of the task. Archived tasks remain excluded from the board, as in the rest of the browser backlog views. Milestones have no archived lifecycle: their lifecycle is fixed to `open -> closed`, and `papyrus milestone close <id>` writes the status in place without moving the record (REQ-03.001), so a closed milestone stays in `backlog/milestones/` and remains shown in its release or unassigned region. There is therefore no milestone archive state, no milestone archive or restore command, and no guide or board archive treatment for milestones.

The final configured task status column of each milestone board offers a "Clean up all" action that archives every task currently in that column for that milestone. The final configured task status is the last entry of the configured task `statuses` list (REQ-01.002, REQ-02.003), and it is the board's last column. The action appears only when that final status differs from `active`; when the final configured task status is `active`, the board offers no "Clean up all" action. Activating the action shows a warning that lists the IDs of the affected tasks and requires confirmation; on cancel, no task is archived. On confirmation, each task is archived independently per REQ-02.004 and REQ-05.007: an archive failure for one task is reported for that task and does not abort the remaining archives, and successfully archived tasks are removed from the board. The action archives only tasks in the current milestone's board; it does not act project-wide. The action is available in the browser only and has no CLI equivalent in this release. Button placement and the warning dialog layout are design decisions.

Presentation details not fixed here — card contents beyond the task link, column widths, and separator styling — remain design decisions.

## Acceptance Criteria
- The Backlog view keeps milestones as expandable and collapsible sections, and expanding a milestone shows its tasks as a kanban board with one column per task status.
- Board columns reflect all configured task statuses, in configured order, and remain visible when they contain no tasks.
- Each release boundary is marked by a bold separator that identifies the release and does not rely on colour alone.
- Milestones assigned to a release appear under that release, and milestones assigned to no release are grouped before the first release.
- Tasks that belong to no milestone appear in their own expandable section, placed after all milestone sections.
- Each task card shows the task ID and title and links to the task record view.
- Cards appear in ascending task ID order within a column.
- A task's status is changed by dragging its card into another status column, and each card also offers an equivalent status control that performs the same change without dragging.
- A status change made on the board is persisted to the task record and is still shown after the view is reloaded.
- A status change that cannot be persisted leaves the card in its original status column and reports the failure to the user.
- A status change made on the board changes only the task's status and preserves all other task fields.
- Each milestone board's final configured task status column offers a "Clean up all" action; that column is the last entry of the configured task `statuses` list.
- The "Clean up all" action appears only when the final configured task status differs from `active`; when the final status is `active`, no such action is shown.
- Activating "Clean up all" shows a warning listing the IDs of the tasks that will be archived and requires confirmation.
- On confirmation, every task in that column for the current milestone is archived and removed from the board.
- If archiving a task fails, the action continues with the remaining tasks and reports the failure for that task.
- On cancel, no task is archived.
- "Clean up all" archives only tasks in the current milestone's board, not project-wide.
- "Clean up all" is available only in the browser; no CLI equivalent is provided in this release.
- Archived tasks are not shown on the board.
- Every milestone is shown in its release or unassigned region, including a milestone whose status is `closed`.
- No archived milestone state exists: milestones have no archive or restore command, so no milestone is hidden from the board by an archive state.

## Traceability
- Source: customer-feedback handoff — project owner, Item 1 (high), raw quote:
  - "modify the browser backlog view to display a kanban board will one column per status, grouped horizontally by milestone and providing a bold separator for the release"
- Source: customer-feedback handoff — Key Account Manager, Item 9, raw quote:
  - "in the backlog view in the browser, add to the last column a button to archtive all task with this status the button is named 'Clean up all'. a warning message appears to say with a list of task which will be archived"
- Project-owner decisions 2026-11-11: the board is nested inside the existing expandable milestones rather than replacing them; columns are the configured statuses in configured order with empty columns visible; tasks without a milestone get their own section after all milestones; milestones without a release are grouped before the first release; drag-and-drop status change is in scope and persists immediately; each card carries an equivalent non-drag status control.
- Product decisions 2026-11-15 (in-session): "Clean up all" scopes to the current milestone only; on partial failure it continues and reports per task; no CLI equivalent is provided.
- Project-owner decision 2026-11-15: "Clean up all" acts on the final configured task status — the last entry of the configured task `statuses` list (REQ-02.003) — and the button appears only when that status differs from `active`.
- `docs/specifications/ui/browser-backlog-records-ui.md` §4 — approved backlog view layout (MOCK-0002)
- REQ-01.002 — Read and Edit Project Configuration (configured task statuses)
- REQ-02.001 — Create and Describe Tasks (task records and their fields)
- REQ-02.002 — View and Edit Tasks (task lifecycle and field editing)
- REQ-02.003 — List and Filter Tasks (final configured task status; last entry of the configured status list)
- REQ-02.004 — Archive, Restore, and Delete Tasks (archive semantics used by "Clean up all")
- REQ-06.002 — Serve the Local Web Interface
- REQ-03.001 — Manage Milestones (fixed `open -> closed` lifecycle written in place by `papyrus milestone close <id>`)
- REQ-05.007 — Manage Document Archive State and Deletion (archive storage rules; archived tasks excluded)
- ADR-0022 decision 5 and ADR-0023 decision 2 — milestones have no archive location and no archive or restore verb, so no archived milestone state exists; TASK-130 delivered that close-only lifecycle
- NFR-01.002 — Display, View, and Edit All Papyrus Content in a Local Browser
- NFR-05.001 — Display the Browser Interface in Named Browsers
