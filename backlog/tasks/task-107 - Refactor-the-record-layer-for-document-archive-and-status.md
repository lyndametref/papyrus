---
id: TASK-107
title: Refactor the record layer for document archive and status
status: To Do
assignee:
  - rust-dev
created_date: '2026-11-13T21:24:00Z'
labels:
  - rust
milestone: m-17
dependencies:
  - TASK-105
documentation:
  - >-
    docs/architecture/adr/ADR-0022-document-lifecycle-state-and-archive-location.md
  - docs/architecture/adr/ADR-0023-consolidated-record-layout-and-field-map.md
priority: high
ordinal: 41010
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The record layer still implements the retired document lifecycle: a `deprecated/` directory, per-type Deprecate and Restore moves, and no `status` key for milestones, releases, or documents. REQ-05.007 and REQ-07.001 replace it with a main status `active` or `archived`, an `archived/` subfolder, archive as a patch-then-move, no document restore, and per-type custom statuses that never move a file. The generic handler, the browser, and search all depend on this storage contract, so it is changed once here rather than per surface.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Directory resolution provides the active and archived locations from configuration; resolve_deprecated_directory and every read alias for deprecated/ are deleted, and a deprecated/ folder is never read, listed, searched, migrated, or written.
- [ ] #2 record::store offers archive and delete for documents and no document restore; tasks and milestones keep their archive/restore pair because their IDs must survive the cycle.
- [ ] #3 archive patches status to archived, writes the destination through the ADR-0006 atomic sequence, and removes the source only after the destination write succeeds; an already-archived record is a no-op success and a location/status disagreement is reported without a write.
- [ ] #4 A document status write sets active or a declared custom status, refreshes updated_date, performs no move, creates no folder, rejects archived with a message naming archive, and rejects an archived record.
- [ ] #5 The front-matter key registry declares status for milestones (open/closed), releases (planned/released), and every front-matter-backed document type (active, archived, plus the configured custom list and default); decisions and UI mockups do not use the document status model.
- [ ] #6 ID allocation scans the active and archived locations only and excludes deprecated/; delete archives an active record before unlinking and leaves the number unused only as ADR-0022 describes.
- [ ] #7 Reading an undeclared status preserves the value verbatim, flags the record, and refuses status writes until the value or configuration is fixed.
- [ ] #8 Tests cover the archive ordering (source survives a failed destination write), no-restore, status rejection paths, allocation scope, and the deprecated/ inert behavior.
<!-- AC:END -->
