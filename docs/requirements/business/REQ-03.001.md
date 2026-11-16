---
id: REQ-03.001
title: "Manage Milestones"
created_date: '2026-11-15T13:39:01Z'
updated_date: '2026-11-16T20:55:57Z'
labels: []
references:
  - RE-002
---
# REQ-03.001 Manage Milestones

## User Story
As a project maintainer, I want to create, maintain, and archive milestones so that related tasks can be organized around a delivery milestone and completed milestones can be archived.

## Group
REQ-03 Milestone and Release Planning

## Criticality
Must have

## Description
The user can create, list, view, edit, close, archive, and delete milestones. A milestone has a title, optional description, and may be assigned to zero or one release. Milestone IDs use the configured prefix and padding (default example `MS-001`).

The milestone lifecycle is fixed: `open -> closed -> archived`. A milestone starts `open` and its state is stored in the front-matter `status` field (REQ-05.010). `papyrus milestone close <id>` changes `open` to `closed`. `papyrus milestone archive <id>` closes the milestone if it is still open and then moves its file to `backlog/archive/milestones/`, marking it `archived`; archived milestones are hidden from active listings. There is no configurable milestone status list and no milestone restore command. Deleting a milestone permanently removes it, with `--force` skipping confirmation.

## Acceptance Criteria
- The documented milestone commands create, list, view, edit, close, archive, and delete milestones, including title, description, and optional release assignment.
- A milestone can be assigned to no more than one release.
- A milestone starts `open` and its lifecycle state is stored in the front-matter `status` field.
- `papyrus milestone close <id>` changes the milestone from `open` to `closed`.
- `papyrus milestone archive <id>` closes an open milestone if needed and moves it to `backlog/archive/milestones/`, marking it `archived`.
- Archived milestones are hidden from active listings.
- The milestone lifecycle is `open -> closed -> archived`; it is not configurable and has no restore command.
- Deleting a milestone permanently deletes it, with `--force` skipping confirmation.
- With default settings, a generated milestone ID follows the `MS-001` format.

## Traceability
- README.md, `papyrus(1)` -> COMMANDS -> milestone (lines 229-277), STATUSES -> Milestones (lines 759-764)
- README.md, `papyrus-config(5)` -> General keys (lines 949-962)
- Project-owner/user decision 2026-11-16: milestone archive closes the milestone first, then moves it to `backlog/archive/milestones/` (RE-002).
- REQ-05.010 - Record Metadata in YAML Front Matter (milestone `status`)
- REQ-07.001 - Organize Artifacts into Three Categories
- BR-006 - milestone Command Behavior (needs revision)
- RE-002 - Milestone command must support archiving milestones
