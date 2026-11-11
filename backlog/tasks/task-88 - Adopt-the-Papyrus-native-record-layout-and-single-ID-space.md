---
id: TASK-88
title: Adopt the Papyrus-native record layout and single ID space
status: To Do
assignee: []
created_date: '2026-11-11T20:42:00Z'
updated_date: '2026-11-11T20:57:00Z'
labels:
  - native-layout
  - identity
  - release-1.0
milestone: m-16
dependencies:
  - TASK-83
  - TASK-84
references:
  - docs/architecture/adr/ADR-0010-papyrus-native-on-disk-record-layout.md
  - >-
    docs/architecture/adr/ADR-0013-shared-record-envelope-without-a-storage-layer.md
  - docs/requirements/business/REQ-01.005.md
  - docs/requirements/non-functional/NFR-04.004.md
priority: high
type: feature
ordinal: 39050
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Task and milestone records are stored as backlog/tasks/task-18 - slug.md and m-0, and their numbers are translated to display IDs through record::identity, while descriptors hardcode directories and src/task.rs and src/milestone.rs resolve three locations each and read backlog/config.yml. ADR-0010 replaces this with one Papyrus ID, configuration-driven directories, <ID>.md filenames, and no translation, and ADR-0013 stage 2 is the prerequisite for adoption, import, and every browser surface. Without this change REQ-01.005, REQ-05.010, and NFR-04.004 stay violated.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 record::identity exposes only Papyrus ID parse, format, validate, and allocate; storage prefixes, offsets, to_storage_id, to_display_id, and their call sites are gone.
- [ ] #2 Descriptor directory values are configuration keys, and no module hardcodes backlog/tasks, backlog/milestones, archive/tasks, or docs/decisions.
- [ ] #3 New task, milestone, and release records are written as <ID>.md under the configured directories, and no code path constructs or writes a Backlog.md stem.
- [ ] #4 Reading recognises <PREFIX>-<digits>[-slug] and <PREFIX>-<digits> - <slug> stems case-insensitively with optional leading zeros; the front-matter id is authoritative and a stem mismatch is flagged with writes refused.
- [ ] #5 Allocation scans every active, archived, and deprecated location of a family with no offset, milestones start at 1, and a prefix or id_padding change affects only records created afterwards.
- [ ] #6 Archive, restore, deprecate, and restore are moves that keep the file name and ID, and the legacy tasks/archived/ and milestones/archived/ locations are read as archived records.
- [ ] #7 backlog/config.yml is no longer read; statuses come only from Papyrus configuration, and an unrecognised status is preserved and reported.
- [ ] #8 Tests cover allocation, stem resolution, archive and deprecate round trips, and that a Backlog-shaped stem resolves to no record.
<!-- AC:END -->
