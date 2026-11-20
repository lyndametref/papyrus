---
id: TASK-269
title: "Delete the remaining per-type browser branches and prove semantic equivalence"
created_date: '2026-11-20T17:42:52Z'
updated_date: '2026-11-20T17:42:52Z'
labels: []
references:
  - ADR-039
  - TECH-068
  - REQ-06.014
status: ToDo
assignee:
  - rust-dev
milestone: MS-035
dependencies:
  - TASK-268
ordinal: 80
documentation: []
type: implementation
---

## Description
Context: after the generic list, view, edit, conflict, and not-found surfaces exist, the per-type branches are dead but still compiled.

Scope: delete every per-type document browser branch, including the code-behind of the audited conflict and not-found renderers. Zero document TypedHandler variants must remain. No document slug dispatch may exist outside the seed; tests and docs are exempt. Replay the MS-032 HTML fixtures and compare semantically: selectors, text, navigation, form fields, and status codes identical; byte differences allowed only where attributed to the uniform title block or seed presentation. Backlog and Decision browser output must be byte-identical.
