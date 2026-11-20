---
id: TASK-248
title: "Implement nested config parsing, validation, and emission for document entries"
created_date: '2026-11-20T17:41:37Z'
updated_date: '2026-11-20T17:41:37Z'
labels: []
references:
  - ADR-036
  - ADR-019
  - ADR-0019
  - ADR-0024
  - TECH-077
  - REQ-07.002
status: ToDo
assignee:
  - rust-dev
milestone: MS-033
dependencies:
  - TASK-247
ordinal: 50
documentation: []
type: implementation
---

## Description
Context: the entry is now nested and the emitter must round-trip it without corrupting quoted headings.

Scope:
- Parse document_types.<slug> with the full key set, nested group and member blocks, and fields as a YAML sequence.
- Reject unknown keys and unknown enum values with document_types.<slug>.<path>; accept fields: [] as valid; reject duplicate field keys per role and reserved field keys.
- Merge: scalars fill from the seed; collections (fields, group.fields, member.fields, statuses) replace; prefix and directory are never filled; an absent slug stays absent.
- Emitter: mapping sequences render as block sequences; flat string lists stay inline; quoted "## ..." headings round-trip unchanged; list-valued keys are not confused with mapping sequences (ADR-0024).
- Diagnostics: report every problem in one pass, ordered deterministically by path.
- config set materializes only the target entry; init writes a minimal file; config unset of a collection resets to the seed collection.
Do not start until MS-031 governance is accepted.
