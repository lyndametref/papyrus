---
id: TASK-268
title: "Implement seed-declared presentation and reference visibility per type and role"
created_date: '2026-11-20T17:42:43Z'
updated_date: '2026-11-20T17:42:43Z'
labels: []
references:
  - ADR-039
  - ADR-0035
  - REQ-06.019
  - REQ-06.014
  - TECH-068
status: ToDo
assignee:
  - rust-dev
milestone: MS-035
dependencies:
  - TASK-267
ordinal: 70
documentation: []
type: implementation
---

## Description
Context: reference visibility and list-row presentation were decided in code by type; they become seed data.

Scope: read presentation facts from the seed for section headings, labels, help text, and field order; read reference_visibility per type and per role for the browser list view. Hidden means the reference list is absent, not empty. Group rows and member rows may differ per role. No fallback branches keyed on document slug.
