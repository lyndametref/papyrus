---
id: TASK-65
title: Expose front-matter metadata options on all record commands
status: To Do
assignee: []
created_date: '2026-11-08T15:06:00Z'
labels:
  - front-matter
  - cli
  - metadata
milestone: m-12
dependencies:
  - TASK-63
  - TASK-64
references:
  - docs/requirements/business/REQ-05.010.md
  - docs/requirements/business/REQ-05.011.md
  - docs/requirements/business/REQ-01.005.md
  - docs/requirements/business/REQ-02.001.md
  - docs/requirements/business/REQ-02.002.md
  - docs/requirements/business/REQ-04.002.md
  - >-
    docs/architecture/adr/ADR-0007-yaml-front-matter-as-single-record-metadata-source.md
  - >-
    docs/architecture/adr/ADR-0009-shared-record-envelope-and-unified-write-path.md
priority: high
type: feature
ordinal: 30030
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Make every managed record command create and update all applicable front-matter fields per REQ-05.011, with automatic timestamps and an ordinal option, using patch-preserving writes.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Every managed record type exposes create and update options for every front-matter key except id, created_date, and updated_date.
- [ ] #2 Task options cover status, assignee as a list, milestone, dependencies, ordinal, documentation, and type; milestone adds release; decision adds superseded_by; audit adds type.
- [ ] #3 created_date is written once at creation and never changed; updated_date is refreshed by every write path; a no-op write does not refresh it.
- [ ] #4 ordinal is an opaque non-negative integer with a create and update option, defaulted from the current maximum.
- [ ] #5 Existing commands that lack an option for an applicable field gain that option for task, milestone, release, decision, requirement, NFR, glossary, UI spec, technical spec, business rule, and audit records.
- [ ] #6 Every write patches only the keys the operation changes and preserves all other front-matter keys and body content.
- [ ] #7 Tests assert option coverage per type, timestamp behavior, ordinal defaults, and field preservation.
<!-- AC:END -->
