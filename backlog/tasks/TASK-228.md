---
id: TASK-228
title: "Update BR-001 and BR-016 for the derived document command surface and nested configuration schema"
created_date: '2026-11-20T17:39:27Z'
updated_date: '2026-11-20T17:39:27Z'
labels: []
references:
  - BR-001
  - BR-016
  - REQ-07.002
  - REQ-07.003
  - ADR-036
status: ToDo
assignee:
  - requirements-engineer
milestone: MS-031
dependencies:
  - TASK-224
  - TASK-225
ordinal: 120
documentation:
  - docs/business-rules/BR-001-papyrus-command-behavior.md
  - docs/business-rules/BR-016-configuration-keys-and-precedence.md
type: requirements
---

## Description
Context: BR-001 describes the global command surface and BR-016 the configuration keys and precedence. Both change when the document command surface becomes model-derived and the entry shape becomes nested.

Scope:
- BR-001: the document command family is generated from the descriptor; baseline verbs plus the model-specific verbs create-group and supersede; title is always the first positional argument on create and create-group, with --title on create, create-group, and edit; no status verb for SingleHtmlMockup; Backlog and Decision commands are unchanged.
- BR-016: entry keys, nested group and member blocks, the fields sequence, selector path grammar, default-fill for scalars, collection replace for fields and statuses, prefix and directory never filled, absent slug stays absent, reserved front-matter keys, and the exact diagnostic form document_types.<slug>.<path>.
- Record precedence: file value beats seed for scalars; a configured collection replaces the seed collection wholesale; config unset of a collection restores the seed collection.
