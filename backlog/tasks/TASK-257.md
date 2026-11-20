---
id: TASK-257
title: "Implement the uniform title contract and remove --term"
created_date: '2026-11-20T17:42:06Z'
updated_date: '2026-11-20T17:42:06Z'
labels: []
references:
  - ADR-036
  - REQ-07.003
  - REQ-05.003
  - REQ-05.006
  - BR-011
  - BR-014
status: ToDo
assignee:
  - rust-dev
milestone: MS-034
dependencies:
  - TASK-255
ordinal: 50
documentation: []
type: implementation
---

## Description
Context: title is currently a type-specific option for some document types and a term option for glossary; a title that can be cleared produces records that violate their own contract.

Scope:
- Title is single-line, non-empty, always present, required on create, never clearable on edit, and not a field; title is a reserved field key.
- create and create-group take the title as the first positional argument and also accept --title; edit takes --title only.
- --term is removed; glossary uses the title contract and its existing term values survive as titles.
- audit gains a positional title and create-group gains a positional title, both additively, without renaming existing positional arguments.
- HTML mockup metadata uses the same title contract.
- Attempting an empty or whitespace-only title on create or edit fails with a diagnostic and writes nothing.
