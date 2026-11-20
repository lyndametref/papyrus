---
id: TASK-224
title: "Rewrite REQ-07.002 for the nested document-type entry shape, defaults, and diagnostics"
created_date: '2026-11-20T17:37:43Z'
updated_date: '2026-11-20T17:37:43Z'
labels: []
references:
  - ADR-036
  - ADR-0019
  - REQ-01.002
  - TECH-077
status: ToDo
assignee:
  - requirements-engineer
milestone: MS-031
dependencies:
  - TASK-218
ordinal: 80
documentation:
  - docs/requirements/business/REQ-07.002.md
type: requirements
---

## Description
Context: REQ-07.002 describes validation for a flat document-type entry. The accepted config format adds nested role blocks, a field sequence, and selector paths, so the requirement must state the entry shape and the merge rules exactly.

Scope: major rewrite of REQ-07.002.
- Top-level entry keys: prefix, directory, enabled, label, plural, order, summary, type_alias, api_aliases, reference_visibility, statuses, default_status, slots, plus fields (single) or group and member blocks (grouped).
- group and member contain fields and optional reference_visibility.
- Field item keys: key, heading, frontmatter_key, option, label, kind (text|multiline|list), required, verbs (default [create, edit]), help. No order and no enabled.
- heading and/or frontmatter_key required; general front-matter keys reserved; field keys unique per role; reserved field keys id, title, status, labels, references, created_date, updated_date, group.
- Defaults and merge: scalars fill from the seed; collections (fields, group.fields, member.fields, statuses) replace; prefix and directory are never filled; an absent slug stays absent.
- Unknown keys and unknown enum values are rejected with document_types.<slug>.<path>.
- fields: [] is valid; init writes minimal entries; config set materializes only the target entry.
- Selector paths such as document_types.<slug>.member.fields[story].heading, with numeric [0] accepted; config list prints effective values with key selectors; adding a field is a file edit; config unset of a collection resets to seed.
- Fields is never comma-set; statuses, api_aliases, and slots keep comma syntax.
- Validation is fail-closed and reports every diagnostic in one pass.
