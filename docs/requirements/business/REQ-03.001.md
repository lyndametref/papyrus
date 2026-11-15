# REQ-03.001 Manage Milestones

## User Story
As a project maintainer, I want to create and maintain milestones so that related tasks can be organized around a delivery milestone.

## Group
REQ-03 Milestone and Release Planning

## Criticality
Must have

## Description
The user can create, list, view, edit, close, and delete milestones. A milestone has a title, optional description, and may be assigned to zero or one release. Milestone IDs use the configured prefix and padding (default example `MS-001`).

The milestone lifecycle is fixed by the backlog category (REQ-07.001): `open -> closed`. A milestone starts `open`, is stored with its lifecycle state in the front-matter `status` field (REQ-05.010), and becomes `closed` through `papyrus milestone close <id>`. There is no configurable milestone status list and no milestone restore command. Deleting a milestone permanently removes it, with `--force` skipping confirmation.

## Acceptance Criteria
- The documented milestone commands create, list, view, and edit milestones, including title, description, and optional release assignment.
- A milestone can be assigned to no more than one release.
- A milestone starts `open` and its lifecycle state is stored in the front-matter `status` field.
- `papyrus milestone close <id>` changes the milestone from `open` to `closed`.
- The milestone lifecycle is fixed to `open -> closed`; it is not configurable and has no restore command.
- Deleting a milestone permanently deletes it, with `--force` skipping confirmation.
- With default settings, a generated milestone ID follows the `MS-001` format.

## Traceability
- README.md, `papyrus(1)` → COMMANDS → milestone (lines 229–277), STATUSES → Milestones (lines 759–764)
- README.md, `papyrus-config(5)` → General keys (lines 949–962)
- Project-owner decision 2026-11-13: milestone lifecycle is `open -> closed`, transitioned by `papyrus milestone close <id>`, replacing the `archive` command; the state is stored in front matter.
- REQ-05.010 — Record Metadata in YAML Front Matter (milestone `status`)
- REQ-07.001 — Organize Artifacts into Three Categories
- BR-006 — milestone Command Behavior
- TASK-130 delivered the close-only milestone lifecycle and TASK-127 corrected the specification pages; BR-006 now documents that lifecycle, so this trace needs no revision note.
