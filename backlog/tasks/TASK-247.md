---
id: TASK-247
title: "Add FieldSpec, DocumentModel, and the extended document-type entry"
created_date: '2026-11-20T17:41:19Z'
updated_date: '2026-11-20T17:41:19Z'
labels: []
references:
  - ADR-036
  - ADR-0018
  - TECH-070
  - TECH-077
  - REQ-07.002
  - REQ-07.004
status: ToDo
assignee:
  - rust-dev
milestone: MS-033
dependencies:
  - TASK-246
ordinal: 40
documentation: []
type: implementation
---

## Description
Context: ContentModel is replaced by DocumentModel and per-type fields become data.

Scope:
- DocumentModel: SingleMarkdown, GroupedMarkdown, SingleHtmlMockup.
- FieldSpec: key, heading, frontmatter_key, option, label, kind (text|multiline|list), required, verbs (default [create, edit]), help.
- Field sets: fields for SingleMarkdown and SingleHtmlMockup; group.fields and member.fields for GroupedMarkdown; optional reference_visibility on group and member.
- Entry keys: prefix, directory, enabled, label, plural, order, summary, type_alias, api_aliases, reference_visibility, statuses, default_status, slots, plus fields or group and member.
- Rules: no order or enabled on a field; heading and/or frontmatter_key required; general front-matter keys reserved; field keys unique per role; reserved keys id, title, status, labels, references, created_date, updated_date, group; model, title, content, id_shape are not config keys.
- User-defined types default to SingleMarkdown; GroupedMarkdown and SingleHtmlMockup are seed-only.
Do not start until MS-031 governance is accepted.
