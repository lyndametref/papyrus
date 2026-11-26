---
id: TECH-097
title: "Browser Backlog Board"
created_date: '2026-11-25T20:42:11Z'
updated_date: '2026-11-25T20:42:11Z'
labels: []
references:
  - REQ-006.013
status: draft
---
# TECH-097 Browser Backlog Board

## Description
Covers the browser Backlog view as a per-milestone kanban board: section and
column structure, card order, milestone and release ordering, lifecycle
markers, immediate status change, task cleanup, and milestone and release
cleanup. It does not cover the task record page, other artifact lists, or
presentation details that the requirement leaves to design.

## Board Structure

- Milestones are expandable and collapsible sections (REQ-006.013).
- Inside an expanded milestone, its tasks appear as a kanban board with one
  column per task status, laid out horizontally (REQ-006.013).
- Columns follow the configured task status list exactly and in configured
  order; a status with no task keeps its column visible (REQ-006.013).
- Milestones that belong to a release appear under that release; milestones that
  belong to no release are grouped before the first release (REQ-006.013).
- Each release boundary is marked by a bold separator naming the release, so the
  boundary does not depend on colour (REQ-006.013).
- Tasks that belong to no milestone appear in their own expandable section after
  all milestone sections (REQ-006.013).
- Cards are ordered by ascending task identifier within a column (REQ-006.013).
- Each card links to the task record view (REQ-006.013).
- Card contents beyond the identifier and title, column widths, separator
  styling, and control placement are outside this specification (REQ-006.013).

## Ordering

- A milestone is ordered by the lowest ordinal among the non-archived tasks
  assigned to it, ascending (REQ-006.013).
- A milestone with no such task comes after every milestone that has one,
  ordered by milestone identifier ascending (REQ-006.013).
- Milestones with equal lowest ordinals are ordered by milestone identifier
  ascending (REQ-006.013).
- The same milestone rule applies inside a release and in the no-release group
  (REQ-006.013).
- A release is ordered by the lowest ordinal among the non-archived tasks in its
  milestones, ascending (REQ-006.013).
- A release with no such task comes last, ordered by release identifier
  ascending (REQ-006.013).
- Releases with equal lowest ordinals are ordered by release identifier
  ascending (REQ-006.013).
- The order is recomputed from current record data on each render, so changing a
  task ordinal moves its milestone and release on reload (REQ-006.013).

## Lifecycle Markers

- A lifecycle marker is placed immediately before each milestone title and each
  release title, in addition to the record identifier (REQ-006.013).
- An open milestone and a planned release carry a ring marker (REQ-006.013).
- A closed milestone and a released release carry a check marker
  (REQ-006.013).
- The marker reflects the stored lifecycle state and changes when that state
  changes and the view is reloaded (REQ-006.013).
- Archived milestones and archived releases are not shown and carry no marker
  (REQ-006.013).

## Task Status Change

- A task's status is changed by dragging its card into another status column, or
  with an equivalent status control on the card that needs no dragging
  (REQ-006.013).
- Either action persists the status immediately; the board is not a staging area
  (REQ-006.013).
- A change that cannot be persisted leaves the card in its original column and
  reports the failure (REQ-006.013).
- A status change alters only the status value and preserves every other task
  field (REQ-006.013).

## Task Cleanup

- The final configured task status column of each milestone board offers a
  "Clean up all" action (REQ-006.013).
- The final configured status is the last entry of the configured task status
  list; the action appears only when that status differs from `active`
  (REQ-006.013).
- Activating the action shows a warning listing the identifiers of the affected
  tasks and requires confirmation; on cancel nothing is archived
  (REQ-006.013).
- On confirmation each task in that column for the current milestone is archived
  independently; a failure is reported for that task and does not abort the
  others, and archived tasks leave the board (REQ-006.013).
- The action never acts project-wide and has no command-line equivalent
  (REQ-006.013).

## Milestone and Release Cleanup

- A cleanup control appears immediately after the title of every closed
  milestone (REQ-006.013).
- The control appears after the title of every released release that contains at
  least one closed milestone; a released release with no closed milestone has
  none (REQ-006.013).
- Activating it shows a confirmation listing the identifiers of the records that
  will be archived; cancelling archives nothing (REQ-006.013).
- For a closed milestone, confirmation archives that milestone
  (REQ-006.013).
- For a released release, confirmation archives each closed milestone contained
  in it, one by one, and leaves the release itself visible and released
  (REQ-006.013).
- If archiving one record fails, the failure is reported for that record, the
  remaining records are still archived, and the failed record stays visible
  (REQ-006.013).
- A release cleanup never archives the release itself (REQ-006.013).
- The cleanup controls have no command-line equivalent and change no lifecycle
  rule (REQ-006.013).

## Exclusions

- Archived tasks appear on no board (REQ-006.013).
- Archived milestones and archived releases are not shown
  (REQ-006.013).
