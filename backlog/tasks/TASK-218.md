---
id: TASK-218
title: "Write ADR-A: config-declared document field schema and one descriptor model"
created_date: '2026-11-20T17:36:36Z'
updated_date: '2026-11-20T17:36:36Z'
labels: []
references:
  - ADR-0017
  - ADR-0018
  - ADR-0019
  - REQ-07.002
  - REQ-07.004
  - TECH-070
  - TECH-077
status: ToDo
assignee:
  - architect
milestone: MS-031
dependencies:
  - TASK-217
ordinal: 20
documentation:
  - decisions/ADR-036-config-declared-document-field-schema.md
type: architecture
---

## Description
Context: ADR-0017 built the artifact category registry on type descriptors with a shared ContentModel (ADR-0018). The refactor needs one descriptor model whose per-type fields are declared in configuration, and ADR-0018's seven-key ContentModel cannot express role-specific field sets or an HTML model.

Scope: author ADR-036, "Config-declared document field schema and one descriptor model".
- Supersede ADR-0017 decisions 1-4, 7, 9, 10 and constraints 1, 4.
- Supersede ADR-0018 in full: decisions 1, 3, 5, 6, 7, 9, 10 and constraints 2, 5, 6.
- Own the DocumentModel enum with variants SingleMarkdown, GroupedMarkdown, SingleHtmlMockup, replacing ContentModel. SingleMarkdown: one record, plain ID, baseline verbs plus status, one field set. GroupedMarkdown: group record with plain ID plus member records with dotted IDs, baseline plus status plus create-group, role field sets, group link owned by the engine. SingleHtmlMockup: one record, plain ID, baseline plus supersede, no status verb, fixed pending/approved/archived metadata.
- Own role field sets, the seed table location and contents, and the field vocabulary: key, heading, frontmatter_key, option, label, kind (text|multiline|list), required, verbs (default [create, edit]), help; no order or enabled; sequence position is order for body and CLI options.
- Own default-fill for scalars and collection-replace for fields, group.fields, member.fields, statuses; prefix and directory are never filled; absent slug stays absent.
- Own the seed-only restriction: GroupedMarkdown and SingleHtmlMockup are seed-only; only req/nfr are grouped; only ui-mockup is HTML; user-defined types default to SingleMarkdown.
- State that the engine emits only PatchOps and never writes record bodies directly.
- Record reserved field keys id, title, status, labels, references, created_date, updated_date, group; heading and/or frontmatter_key required; field keys unique per role.
