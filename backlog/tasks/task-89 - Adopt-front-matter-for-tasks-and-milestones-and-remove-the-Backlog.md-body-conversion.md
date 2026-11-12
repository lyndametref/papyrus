---
id: TASK-89
title: >-
  Adopt front matter for tasks and milestones and remove the Backlog.md body
  conversion
status: To Do
assignee: []
created_date: '2026-11-11T20:42:00Z'
updated_date: '2026-11-12T19:05:00Z'
labels:
  - front-matter
  - native-layout
  - release-1.0
milestone: m-16
dependencies:
  - TASK-88
references:
  - docs/architecture/adr/ADR-0012-papyrus-owned-front-matter-metadata.md
  - >-
    docs/architecture/adr/ADR-0013-shared-record-envelope-without-a-storage-layer.md
  - docs/requirements/business/REQ-05.010.md
  - docs/requirements/business/REQ-05.011.md
priority: high
type: feature
ordinal: 39060
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
src/record/migrate.rs still converts legacy task and milestone bodies to the Backlog.md layout on read or write. ADR-0012 requires task and milestone adoption to be front-matter-only with legacy metadata sections retired on write, and ADR-0013 stage 4 deletes the conversion because its target shape no longer exists. This task converges the existing corpus without a bulk pass and removes the last code path that rewrites a task body. The lazy legacy relocation to a Backlog-shaped file name and directory is removed by TASK-88; this task removes only the body conversion.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A task or milestone without front matter gains the block on first read and first write, and the body is otherwise byte-identical.
- [ ] #2 Legacy task Status, Assignee, Milestone, Labels, and Dependencies sections and the milestone Release section move their values into front matter and are removed on write; a value that cannot be moved keeps its section and is reported.
- [ ] #3 Adoption is idempotent: a record that already carries front matter is a byte-identical no-op on both read and write.
- [ ] #4 The legacy-to-Backlog body conversion and its tests and fixtures are deleted; no read or write path rewrites a body into the Backlog.md layout.
- [ ] #5 Tasks and milestones gain no leading title heading, and a task carries the documented front-matter key set with assignee as a YAML list.
- [ ] #6 updated_date is refreshed on every write from one clock value, and created_date is written once and never changed.
- [ ] #7 Tests cover adoption on read and on write, section retirement, idempotency, and byte identity for a record that already has front matter.
<!-- AC:END -->
