---
id: TASK-191
title: "Refine multiline stdin input implementation"
created_date: '2026-11-18T21:13:37Z'
updated_date: '2026-11-18T21:13:37Z'
labels:
  - bug
references:
  - RE-007
  - MS-029
status: ToDo
assignee:
  - task-refiner
milestone: MS-029
dependencies:
  - TASK-190
ordinal: 2
documentation:
  - src/document.rs
  - src/cli/dispatch.rs
  - src/sanitize.rs
type: refinement
---

## Description
Outcome: confirmed and complete implementation scope for RE-007, including affected create and update commands across document and backlog types, the stdin contract, which fields accept multiline values, and unchanged single-line validation for identifiers and titles. Done when refinement fields, references, and documentation are complete.
