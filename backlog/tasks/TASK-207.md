---
id: TASK-207
title: "Design tests for lowest-ordinal milestone ordering, markers, and cleanup in browser"
created_date: '2026-11-19T11:02:50Z'
updated_date: '2026-11-19T18:38:06Z'
labels: []
references:
  - REQ-006.020
  - REQ-006.021
  - REQ-006.022
  - REQ-006.013
  - RE-008
status: Done
assignee:
  - test-manager
milestone: MS-030
dependencies:
  - TASK-206
ordinal: 30
documentation: []
type: test-design
---

## Description
Gherkin test cases for ordering milestones and releases by the lowest ordinal of their non-archived tasks ascending, lifecycle markers, and cleanup controls covering REQ-006.020, REQ-006.021, and REQ-006.022. The ascending lowest-ordinal rule supersedes the earlier highest-ordinal descending rule. Done when user verified.

## Comments
- **test-manager**: Created integration test cases TC-066..TC-097 (32) covering REQ-006.020, REQ-006.021, REQ-006.022, REQ-006.013 and MS-030 AC-1..AC-15 (ordering, markers, cleanup). All test cases are draft pending explicit user verification. AC-16 (UI-002 revision) and AC-17 (mockup impact) are documentation/mockup scope tracked via UI-002/MOCK-022.
- **test-manager**: User verified and approved TC-066..TC-097 (status approved).
- **orchestrator**: User verified TC-066..TC-097 (32 cases). Test design verified.
