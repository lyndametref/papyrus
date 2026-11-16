---
id: TASK-153
title: "Fix Backlog.md import to emit Papyrus-canonical quoted assignee metadata and repair affected task files"
created_date: '2026-11-16T19:34:36Z'
updated_date: '2026-11-16T19:34:36Z'
labels:
  - bug
references:
  - RE-001
  - NFR-04.003
  - REQ-05.010
  - REQ-05.011
status: ToDo
assignee:
  - rust-dev
milestone: MS-023
dependencies:
  - TASK-152
ordinal: 30
documentation:
  - src/import.rs
type: implementation
---

## Description
Outcome: import emits valid Papyrus YAML for assignee entries and affected task files are corrected or re-imported so their metadata parses.
