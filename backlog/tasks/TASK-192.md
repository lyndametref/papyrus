---
id: TASK-192
title: "Implement multiline stdin input for create and update commands"
created_date: '2026-11-18T21:13:40Z'
updated_date: '2026-11-18T21:13:40Z'
labels:
  - bug
references:
  - RE-007
  - MS-029
status: ToDo
assignee:
  - rust-dev
milestone: MS-029
dependencies:
  - TASK-191
ordinal: 3
documentation:
  - src/document.rs
  - src/cli/dispatch.rs
  - src/sanitize.rs
type: implementation
---

## Description
Outcome: create and update commands accept multiline body and content field values from stdin for all affected document and backlog types, while identifiers, titles, and other single-line fields still reject embedded newlines. Existing argument and editor paths are unchanged, and CLI help documents the stdin input path.
