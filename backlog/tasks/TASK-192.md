---
id: TASK-192
title: "Implement multiline stdin input for create and update commands"
created_date: '2026-11-18T21:13:40Z'
updated_date: '2026-11-19T04:56:34Z'
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
## Comments
- **task-refiner**: Refinement contract fixed by TASK-191 (Done). Sentinel: exact `-` value on --description for document, task, and milestone create and edit; on --decision, --context, and --consequences for decision create and edit. One stdin read per invocation to EOF; UTF-8 required; CRLF to LF; trim outer line terminators only; internal blank lines preserved; LF the only allowed control character; empty or whitespace-only input and multiple sentinels exit 4 with no write; no implicit stdin read. Help sentence: Use "-" to read multiline text from standard input. See the TASK-191 description for the full contract, assumptions, and definition of done.
