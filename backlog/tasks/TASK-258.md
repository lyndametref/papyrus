---
id: TASK-258
title: "Implement config set, list, get, and unset with selector paths"
created_date: '2026-11-20T17:42:06Z'
updated_date: '2026-11-20T17:42:06Z'
labels: []
references:
  - ADR-036
  - REQ-01.002
  - REQ-07.002
  - TECH-077
status: ToDo
assignee:
  - rust-dev
milestone: MS-034
dependencies:
  - TASK-255
ordinal: 60
documentation: []
type: implementation
---

## Description
Context: the current config surface addresses only top-level keys and flat lists, so a nested field or role value cannot be read or changed.

Scope:
- Selector paths such as document_types.<slug>.member.fields[story].heading and document_types.req.group.fields[0].required; numeric [0] and keyed [story] indexing both work.
- config list prints effective values with key selectors and marks seed-filled values.
- config get and set accept one selector; config set materializes only the target entry, never the whole seed.
- config unset of a collection resets it to the seed collection; unset of a scalar removes the override.
- fields is never comma-set; statuses, api_aliases, and slots keep comma syntax; unknown selectors and out-of-range indices fail with the same document_types.<slug>.<path> diagnostic.
