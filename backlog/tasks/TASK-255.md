---
id: TASK-255
title: "Implement the field-driven document handler with baseline verbs and status"
created_date: '2026-11-20T17:41:57Z'
updated_date: '2026-11-20T17:41:57Z'
labels: []
references:
  - ADR-036
  - ADR-0016
  - REQ-07.003
  - REQ-05.011
  - TECH-070
status: ToDo
assignee:
  - rust-dev
milestone: MS-034
dependencies:
  - TASK-254
ordinal: 30
documentation: []
type: implementation
---

## Description
Context: nine typed modules each implement create, list, view, edit, status, archive, and delete with near-identical code.

Scope: one handler driven by FieldSpec. Create and edit build PatchOps from the field sequence and each field's verbs; list and view render through the field map with frontmatter_key; status and archive keep their existing semantics, including archive into the type's archived subfolder and the no-cascade rule. Missing required fields fail with one diagnostic listing every missing key. Front-matter options exist only for fields whose verbs include the command; options follow sequence order.
