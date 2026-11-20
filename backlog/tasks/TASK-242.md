---
id: TASK-242
title: "Record the corrected renderer and route inventory with file paths"
created_date: '2026-11-20T17:40:57Z'
updated_date: '2026-11-20T18:00:19Z'
labels: []
references:
  - ADR-039
  - TECH-068
status: ToDo
assignee:
  - code-analyzer
milestone: MS-032
dependencies:
  - TASK-239
ordinal: 50
documentation: []
type: audit
---

## Description
Context: earlier planning notes said 6 conflict-page renderers and 4 not-found pages; the audit found 9 and 5. Later milestones must delete exactly the right set.

Scope: enumerate every browser route and renderer, with file path and function name, for each document type and for shared surfaces. Confirm 9 conflict-page renderers and 5 not-found pages, list them, and list any shared helper they duplicate. Record the inventory as an audit record so MS-035 can prove every one is gone. Note that the 1,156-test count is a count, not a gate.
