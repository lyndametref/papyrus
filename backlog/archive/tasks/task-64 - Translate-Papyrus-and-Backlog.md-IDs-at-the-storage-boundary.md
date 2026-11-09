---
id: TASK-64
title: Translate Papyrus and Backlog.md IDs at the storage boundary
status: Done
assignee: []
created_date: '2026-11-08T15:06:00Z'
updated_date: '2026-11-08T17:19:00Z'
labels:
  - front-matter
  - record-identity
  - backlog-interop
milestone: m-12
dependencies:
  - TASK-63
references:
  - docs/requirements/business/REQ-01.005.md
  - docs/requirements/business/REQ-05.010.md
  - docs/requirements/non-functional/NFR-04.003.md
  - >-
    docs/architecture/adr/ADR-0008-adopt-backlog-md-on-disk-layout-for-tasks-and-milestones.md
  - >-
    docs/architecture/adr/ADR-0009-shared-record-envelope-and-unified-write-path.md
documentation:
  - >-
    docs/specifications/translate-papyrus-and-backlog-md-ids-at-the-storage-boundary.md
priority: high
type: feature
ordinal: 30020
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Implement ADR-0009 Stage 2 and ADR-0008 identity: one pure translation between Papyrus display IDs and Backlog.md storage IDs, reference translation, number allocation, and conflict detection. Commands keep accepting and showing Papyrus IDs only.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 record::identity translates Papyrus display IDs to Backlog.md storage IDs and back with fixed task offset 0 and milestone offset 1, using configured display prefix and id_padding.
- [ ] #2 Property tests prove round-trip identity, monotonicity, injectivity, and rejection of malformed IDs across prefix, padding, and offset combinations.
- [ ] #3 Storage-number allocation considers active and archived records; create never reuses a number from either ID space.
- [ ] #4 milestone: and dependencies: reference values are stored in storage form, translated for display and input validation, and resolve in both tools.
- [ ] #5 Commands reject a Backlog.md storage ID as input with a hint naming the equivalent Papyrus ID.
- [ ] #6 No CLI text/JSON, browser view, or URL presents a storage ID as the record identity; diagnostics label the storage ID separately.
- [ ] #7 Duplicate and mismatch detection follows ADR-0008 decision 9 and never merges, deletes, or renumbers a record.
<!-- AC:END -->
