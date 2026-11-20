---
id: TASK-266
title: "Implement the generic edit form and form_post handler"
created_date: '2026-11-20T17:42:43Z'
updated_date: '2026-11-20T17:42:43Z'
labels: []
references:
  - ADR-039
  - ADR-0028
  - ADR-0013
  - REQ-07.003
  - TECH-068
status: ToDo
assignee:
  - rust-dev
milestone: MS-035
dependencies:
  - TASK-265
ordinal: 50
documentation: []
type: implementation
---

## Description
Context: edit forms and form_post handlers are duplicated per document type, including field validation and conflict detection.

Scope: one edit renderer and one form_post driven by the field sequence, honoring per-field verbs, required flags, and kind (text, multiline, list). Title is a required text input that cannot be blank and cannot clear an existing value. Preserve optimistic concurrency: a stale version returns the conflict page and writes nothing. Keep the existing error surface and status codes. GroupedMarkdown member create and edit use the same generic path.
