---
id: TASK-223
title: "Rewrite REQ-07.004 for one schema-driven document content model with three models"
created_date: '2026-11-20T17:37:30Z'
updated_date: '2026-11-20T17:37:30Z'
labels: []
references:
  - ADR-036
  - ADR-0017
  - ADR-0018
  - REQ-05.010
  - TECH-070
  - TECH-071
status: ToDo
assignee:
  - requirements-engineer
milestone: MS-031
dependencies:
  - TASK-218
ordinal: 70
documentation:
  - docs/requirements/business/REQ-07.004.md
type: requirements
---

## Description
Context: REQ-07.004 currently promises a generic model with no per-type schema and excludes config-defined types from type-specific fields. The accepted architecture replaces that with a schema declared in configuration and three document models.

Scope: full rewrite of REQ-07.004.
- Define DocumentModel with SingleMarkdown, GroupedMarkdown, SingleHtmlMockup; ContentModel is removed.
- A type declares its field set: one fields sequence for SingleMarkdown and SingleHtmlMockup, group.fields plus member.fields for GroupedMarkdown.
- Remove the seven-key fixed content model, the "no per-type field schema" exclusion, and the "no type-specific body section" exclusion for user-defined types.
- GroupedMarkdown: group record with plain ID, member records with dotted IDs, engine-owned group link, role field sets, immutable membership after creation.
- SingleHtmlMockup: one record, plain ID, no status verb, fixed pending/approved/archived metadata.
- Seed-only restriction: GroupedMarkdown and SingleHtmlMockup only for seed types; only req and nfr are grouped; only ui-mockup is HTML; user-defined types default to SingleMarkdown.
- Title is fixed and non-configurable: single-line, non-empty, always present, required on create, never clearable on edit, not a field, reserved field key.
- Configuration declares fields and presentation but is never executable: no templates, expressions, or code.
- The engine emits only PatchOps; it never writes a record body directly.
