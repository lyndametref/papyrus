---
id: TASK-90
title: Implement atomic whole-folder relocation for config set
status: To Do
assignee: []
created_date: '2026-11-11T20:42:00Z'
labels:
  - configuration
  - relocation
  - release-1.0
milestone: m-14
dependencies:
  - TASK-85
  - TASK-88
references:
  - docs/requirements/business/REQ-01.002.md
  - docs/architecture/adr/ADR-0010-papyrus-native-on-disk-record-layout.md
  - >-
    docs/architecture/adr/ADR-0013-shared-record-envelope-without-a-storage-layer.md
  - docs/specifications/project-initialization-and-configuration.md
priority: high
type: feature
ordinal: 39070
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
REQ-01.002 now requires config set to move every file and subfolder of an artifact folder synchronously and atomically, with no partial move and no persisted value on failure. Today config set only persists a key and nothing moves files, and the browser configuration view must reuse the same routine rather than move folders itself. This task implements the single relocation routine of ADR-0010 decision 8 and ADR-0013 stage 5.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 papyrus config set on backlog_directory, decision_directory, doc_directory, or a document type directory moves every file and subfolder of the source folder, including files Papyrus does not manage.
- [ ] #2 The move runs inside the command, is a move rather than a copy, and removes the source folder once it is empty.
- [ ] #3 A destination entry conflict, permission failure, or write failure moves nothing or rolls back, persists no configuration value, and reports the conflict outcome of REQ-01.004.
- [ ] #4 One shared implementation (record::relocate) is used by config set, and the browser reaches folder moves only through this path.
- [ ] #5 Every source and destination path passes Context::ensure_within, and configuration validation rejects absolute and traversal folder values.
- [ ] #6 A cross-device move falls back to copy-then-remove and the command reports that a single rename was impossible.
- [ ] #7 Tests cover a successful move, a destination conflict, partial-failure rollback, cross-device fallback, and unchanged configuration on failure.
<!-- AC:END -->
