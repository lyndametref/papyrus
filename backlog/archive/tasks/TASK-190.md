---
id: TASK-190
title: "Design tests for multiline stdin input in create and update commands"
created_date: '2026-11-18T21:13:33Z'
updated_date: '2026-11-19T10:54:53Z'
labels:
  - bug
references:
  - RE-007
  - MS-029
status: Done
assignee:
  - test-manager
milestone: MS-029
dependencies: []
ordinal: 1
documentation: []
type: test-design
---

## Description
Outcome: Gherkin test cases for RE-007 covering multiline body and content values supplied on stdin to document and backlog create and update commands, and rejection of embedded newlines in identifiers and titles. Include unchanged single-line argument and editor paths. Done when the user has verified the test cases.
## Comments
- **test-manager**: Test cases TC-040 to TC-054 designed and approved by user. Coverage: RE-007 AC-1 to AC-6; REQ-007.003, REQ-007.004, REQ-004.001, REQ-004.002, REQ-002.001, REQ-002.002, REQ-003.001, REQ-001.004, NFR-002.002; BR-001, BR-005, BR-006, BR-008. Open questions: stdin contract to be fixed in refinement TASK-191.
