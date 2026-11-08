---
id: TASK-69
title: Migrate tasks and milestones to the Backlog.md layout
status: To Do
assignee: []
created_date: '2026-11-08T15:06:00Z'
updated_date: '2026-11-08T16:10:00Z'
labels:
  - front-matter
  - migration
  - backlog-interop
milestone: m-12
dependencies:
  - TASK-64
  - TASK-68
references:
  - docs/requirements/non-functional/NFR-04.003.md
  - docs/requirements/business/REQ-01.005.md
  - docs/requirements/business/REQ-05.010.md
  - docs/requirements/business/REQ-05.011.md
  - >-
    docs/architecture/adr/ADR-0007-yaml-front-matter-as-single-record-metadata-source.md
  - >-
    docs/architecture/adr/ADR-0008-adopt-backlog-md-on-disk-layout-for-tasks-and-milestones.md
  - >-
    docs/architecture/adr/ADR-0009-shared-record-envelope-and-unified-write-path.md
priority: high
type: feature
ordinal: 30070
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Implement lazy, per-record migration for task and milestone files under REQ-05.011 and NFR-04.003: each task or milestone file gains front matter on its first read and on its first write. Files already in Backlog.md shape keep bodies and marker comments byte-identical apart from the front-matter block; legacy Papyrus-shape files are converted to the Backlog.md shape at that time, with unknown sections and free text preserved verbatim and a before/after diff reported per converted file. Migration is idempotent and runs through record::patch.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Each active and archived task and milestone file gains front matter on its first read and on its first write; display IDs are unchanged before and after.
- [ ] #2 Files already in Backlog.md shape keep bodies, marker comments, and section order byte-identical apart from added front-matter keys; tasks with a complete block are not rewritten.
- [ ] #3 Legacy Papyrus-shape files are converted on first read or first write: heading removed, retired metadata sections moved to front matter, description and acceptance criteria mapped to marker sections, and every unknown section and free-text block preserved verbatim in file order.
- [ ] #4 Each converted legacy file is reported with a before/after diff in the per-file inventory.
- [ ] #5 Milestone front matter carries the general keys plus release, and Backlog.md reads, lists, updates, and preserves the additive keys.
- [ ] #6 Conflict and duplicate cases follow ADR-0008 decision 9; migration stops and reports rather than merging, deleting, or renumbering.
- [ ] #7 Legacy archive paths are read for compatibility, consolidated into the Backlog.md archive paths, and never written again.
- [ ] #8 Migration is idempotent: once a record carries front matter, neither its first read nor its first write re-runs the conversion.
<!-- AC:END -->
