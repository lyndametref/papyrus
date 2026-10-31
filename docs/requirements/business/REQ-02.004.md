# REQ-02.004 Archive, Restore, and Delete Tasks

## User Story
As a project maintainer, I want to archive, restore, or permanently delete tasks so that completed or unwanted records can be managed appropriately.

## Group
REQ-02 Task and Board Management

## Criticality
Must have

## Description
Archiving moves a task off the board while preserving its ID and every dependency. Restore returns an archived task to the board with its previous status. Deletion permanently removes a task; an active task is archived first. `--force` skips the deletion confirmation prompt.

## Acceptance Criteria
- Archiving a task removes it from the board and preserves its ID and dependencies.
- Restoring an archived task returns it to the board with its previous status and same ID.
- Deleting an archived task permanently deletes it; deleting an active task archives it first.
- `--force` skips the confirmation prompt when deleting.

## Traceability
- README.md, `papyrus(1)` → COMMANDS → task archive/restore/delete (lines 215–227)
