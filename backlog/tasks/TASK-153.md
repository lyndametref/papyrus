---
id: TASK-153
title: "Fix Backlog.md import to emit Papyrus-canonical quoted assignee metadata and repair affected task files"
created_date: '2026-11-16T19:34:36Z'
updated_date: '2026-11-17T10:50:07Z'
labels:
  - bug
references:
  - RE-001
  - NFR-04.003
  - REQ-05.010
  - REQ-05.011
status: Done
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

## Comments
- **rust-dev**: Implemented: format_scalar now single-quotes @-prefixed scalars; import translates unquoted @ assignee entries to quoted form before YAML parsing; repaired TASK-144/TASK-145. Tests: cargo test import_backlog_cli (10 passed), cargo test lib frontmatter (pass), cargo clippy/fmt (pass). Commit: pending.
- **orchestrator**: Implementation verified and task marked Done.
