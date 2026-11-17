---
id: TASK-151
title: "Design tests for quoted assignee metadata in Backlog.md import"
created_date: '2026-11-16T19:34:33Z'
updated_date: '2026-11-17T18:47:28Z'
labels:
  - bug
references:
  - RE-001
  - NFR-04.003
  - REQ-05.010
  - REQ-05.011
status: Done
assignee:
  - test-manager
milestone: MS-023
dependencies: []
ordinal: 10
documentation:
  - src/import.rs
type: test-design
---

## Description
Outcome: test cases covering import of Backlog.md task metadata where assignee entries require Papyrus-canonical YAML quoting, plus parsing of the affected task files. Done when the user has verified the test cases.
## Comments
- **orchestrator**: Test cases TC-001..TC-006 created and approved by user (verified). Coverage: REQ-05.010, REQ-05.011.
