---
id: TASK-265
title: "Implement the record envelope field map with generic list and view rendering"
created_date: '2026-11-20T17:42:31Z'
updated_date: '2026-11-20T17:42:31Z'
labels: []
references:
  - ADR-039
  - ADR-036
  - REQ-06.014
  - REQ-06.004
  - TECH-040
  - TECH-068
status: ToDo
assignee:
  - rust-dev
milestone: MS-035
dependencies:
  - TASK-264
ordinal: 40
documentation: []
type: implementation
---

## Description
Context: each document type currently has its own list and view renderer, all walking the same record shape.

Scope: extend the record envelope with a model-neutral field map keyed by FieldSpec, then render list and view from the descriptor. Group and member records render their role fields; the group link is not shown as a field. Labels, headings, field order, and help text come from the seed. No document slug is matched by name in this path. Backlog and Decision renderers are not touched.
