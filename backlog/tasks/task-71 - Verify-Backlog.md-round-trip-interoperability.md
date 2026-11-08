---
id: TASK-71
title: Verify Backlog.md round-trip interoperability
status: To Do
assignee: []
created_date: '2026-11-08T15:06:00Z'
labels:
  - verification
  - backlog-interop
milestone: m-12
dependencies:
  - TASK-69
  - TASK-70
references:
  - docs/requirements/non-functional/NFR-04.003.md
  - docs/requirements/business/REQ-01.005.md
  - docs/requirements/business/REQ-05.010.md
  - >-
    docs/architecture/adr/ADR-0007-yaml-front-matter-as-single-record-metadata-source.md
  - >-
    docs/architecture/adr/ADR-0008-adopt-backlog-md-on-disk-layout-for-tasks-and-milestones.md
  - >-
    docs/architecture/adr/ADR-0009-shared-record-envelope-and-unified-write-path.md
priority: high
type: chore
ordinal: 30085
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Prove NFR-04.003 interoperability empirically in both directions, including translated identity and the additive milestone keys, on new and migrated records. This is the Backlog.md round-trip gate.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A Backlog.md-created task is listed by Papyrus under the translated display ID and resolves in view.
- [ ] #2 A Papyrus-created task appears in Backlog.md list and view with the same title and fields.
- [ ] #3 Milestone m-12 maps to MS-013 and view MS-013 resolves; dependencies [TASK-31] displays as TASK-031.
- [ ] #4 Backlog.md list, view, and update accept and preserve the additive milestone general keys and release.
- [ ] #5 Both directions run on newly created records and on records produced by the document and task/milestone migrations, including both migration input shapes.
- [ ] #6 Any field rejected or rewritten by Backlog.md is recorded as a blocker with the exact command and output; the task is not Done while such a blocker exists.
<!-- AC:END -->
