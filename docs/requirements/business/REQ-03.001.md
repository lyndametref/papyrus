# REQ-03.001 Manage Milestones

## User Story
As a project maintainer, I want to create and maintain milestones so that related tasks can be organized around a delivery milestone.

## Group
REQ-03 Milestone and Release Planning

## Criticality
Must have

## Description
The user can create, list, view, edit, archive, and delete milestones. A milestone has a title, optional description, and may be assigned to zero or one release. Milestone IDs use the configured prefix and padding (default example `MS-001`). The man page specifies archive but does not describe a milestone restore command.

## Acceptance Criteria
- The documented milestone commands create, list, view, and edit milestones, including title, description, and optional release assignment.
- A milestone can be assigned to no more than one release.
- Archive marks a milestone archived; delete permanently deletes it, with `--force` skipping confirmation.
- With default settings, a generated milestone ID follows the `MS-001` format.

## Traceability
- README.md, `papyrus(1)` → COMMANDS → milestone (lines 229–277), STATUSES → Milestones (lines 759–764)
- README.md, `papyrus-config(5)` → General keys (lines 949–962)
