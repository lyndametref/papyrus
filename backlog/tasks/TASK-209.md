---
id: TASK-209
title: "Implement lowest-ordinal milestone ordering, markers, and cleanup in browser"
created_date: '2026-11-19T11:02:54Z'
updated_date: '2026-11-19T11:19:51Z'
labels: []
references:
  - REQ-06.020
  - REQ-06.021
  - REQ-06.022
  - REQ-06.013
  - RE-008
status: ToDo
assignee:
  - rust-dev
milestone: MS-030
dependencies:
  - TASK-208
ordinal: 50
documentation: []
type: implementation
---

## Description
Code and automated tests for ordering milestones and releases by the lowest ordinal of their non-archived tasks ascending, lifecycle markers, and cleanup controls in the browser Backlog board. The ascending lowest-ordinal rule supersedes the earlier highest-ordinal descending rule.
