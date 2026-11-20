---
id: TASK-236
title: "Add a Document Type object to the domain model and glossary"
created_date: '2026-11-20T17:40:32Z'
updated_date: '2026-11-20T17:40:32Z'
labels: []
references:
  - ADR-036
  - REQ-07.004
  - REQ-07.002
status: ToDo
assignee:
  - tech-writer
milestone: MS-031
dependencies:
  - TASK-218
ordinal: 200
documentation:
  - docs/domain-model/business-object-model.md
  - docs/domain-model/context-map.md
type: docs
---

## Description
Context: the domain model has no object for a document type; document types exist only implicitly in configuration. The new model makes the type a first-class object with a model, roles, field sets, verbs, and presentation.

Scope:
- Add Document Type to the business object model: identity (slug), DocumentModel, prefix, directory, statuses and default status, slots, reference visibility, single or group/member field sets, derived verbs, and presentation facts.
- State the invariants: model is seed-only; group membership is immutable; reserved field keys and general front-matter keys are protected; title is fixed and not a field; the group link is engine-owned.
- Add glossary terms for DocumentModel and its three variants, FieldSpec, role, group record, member record, and seed.
- Update the context map if the new object changes an aggregate boundary.
Do not add planning or delivery language.
