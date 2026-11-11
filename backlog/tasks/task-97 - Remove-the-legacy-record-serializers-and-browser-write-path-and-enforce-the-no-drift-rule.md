---
id: TASK-97
title: >-
  Remove the legacy record serializers and browser write path and enforce the
  no-drift rule
status: To Do
assignee: []
created_date: '2026-11-11T20:44:00Z'
labels:
  - native-layout
  - refactor
  - release-1.0
milestone: m-14
dependencies:
  - TASK-88
  - TASK-89
  - TASK-92
  - TASK-93
  - TASK-94
  - TASK-95
  - TASK-96
references:
  - >-
    docs/architecture/adr/ADR-0013-shared-record-envelope-without-a-storage-layer.md
  - docs/architecture/adr/ADR-0012-papyrus-owned-front-matter-metadata.md
  - >-
    docs/specifications/remove-legacy-record-serializers-and-enforce-the-no-drift-rule.md
priority: high
type: feature
ordinal: 39140
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Fourteen per-type modules still assemble whole record files and the browser has its own record-writing helpers, so preservation, identity, and metadata behaviour can diverge between surfaces. ADR-0013 stage 7 deletes those paths and makes the no-drift rule mechanically checkable. Delivering the new layout and browser surfaces without this removal would leave two writers in the codebase the release claims to have unified.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The per-type parser and serializer paths are replaced by descriptors and the shared envelope, and no render function assembles a whole record file.
- [ ] #2 The browser uses the shared document, patch, and identity modules, and its record-writing helpers are removed.
- [ ] #3 No module outside the shared record module writes a record file, formats a front-matter block, or formats a record ID.
- [ ] #4 Storage-ID fields are removed from CLI and browser JSON, and the metadata id carries the Papyrus ID.
- [ ] #5 Mechanical review checks exist for storage-ID translation symbols, offsets, storage prefixes, hardcoded record directories, and Backlog-shaped stems.
- [ ] #6 Every record type still passes its existing tests after the removal.
<!-- AC:END -->
