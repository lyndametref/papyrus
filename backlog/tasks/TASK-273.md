---
id: TASK-273
title: "Implement the request-scoped record snapshot for resolution"
created_date: '2026-11-20T17:43:07Z'
updated_date: '2026-11-20T17:43:07Z'
labels: []
references:
  - ADR-037
  - TECH-043
  - NFR-007.002
status: ToDo
assignee:
  - rust-dev
milestone: MS-036
dependencies:
  - TASK-272
ordinal: 30
documentation: []
type: implementation
---

## Description
Context: load_record resolves titles through resolve_id, which rescans descriptor directories, so index construction is O(n^2) and each view repeats it.

Scope: build one request-scoped snapshot mapping id to Record from the registry and pass it through resolution, list, view, and edit rendering. No write derives its content or version token from the snapshot; optimistic concurrency still reads the live record. Fail with a clear diagnostic if a resolution target is missing from the snapshot rather than falling back to a rescan.
