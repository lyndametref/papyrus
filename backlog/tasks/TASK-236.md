---
id: TASK-236
title: "Add a Document Type object to the domain model and glossary"
created_date: '2026-11-20T17:40:32Z'
updated_date: '2026-11-21T16:31:09Z'
labels: []
references:
  - ADR-036
  - REQ-07.004
  - REQ-07.002
  - REQ-07.001
status: Done
assignee:
  - tech-writer
milestone: MS-031
dependencies:
  - TASK-218
ordinal: 200
documentation:
  - docs/domain-model/business-object-model.md
  - docs/domain-model/context-map.md
  - docs/domain-model/glossary
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
## Refinement inputs (TASK-290)

### Domain-model map (real paths)
- `docs/domain-model/business-object-model.md` — `Business object model`, `Object descriptions`, `Lifecycles`, `Documented relationships`, `Open questions`.
- `docs/domain-model/context-map.md` — `Context Descriptions` and its four bounded contexts; update only if the new object changes an aggregate boundary.
- `docs/domain-model/glossary/` — managed GLO records; add the terms listed in the scope as new GLO records, do not edit files directly.
- Read-only inputs: ADR-036 (`decisions/ADR-036-config-declared-document-field-schema.md`), REQ-07.002, REQ-07.004.

### Refinement assumptions / definition of done
Document Type is a new first-class object; existing object descriptions and context boundaries stay unless the new object demonstrably changes them. Done means all acceptance criteria pass, the user has verified the model, and no source or test file changes (MS-031 AC-7, AC-10).
## Acceptance Criteria
- [ ] The business object model adds Document Type with identity (slug), DocumentModel, prefix, directory, statuses and default status, slots, reference visibility, single or group and member field sets, derived verbs, and presentation facts (MS-031 AC-7).
- [ ] The model states the invariants: model is seed-only, group membership is immutable, reserved field keys and general front-matter keys are protected, title is fixed and not a field, and the group link is engine-owned (MS-031 AC-7, RE-010 D6, D7, D13).
- [ ] Glossary entries exist for DocumentModel and its three variants, FieldSpec, role, group record, member record, and seed, each added through the glossary management command (MS-031 AC-7).
- [ ] The context map is updated only if the new object changes an aggregate boundary, and the change is stated explicitly (MS-031 AC-7).
- [ ] No planning or delivery language is added, and the user has verified the domain model and glossary additions (MS-031 AC-7, AC-8).
## Comments
- **requirements-engineer**: Evidence: business object model adds Document Type (slug identity, DocumentModel, prefix, directory, statuses and default status, slots, per-role reference visibility, single fields or group.fields and member.fields role sets, derived verbs, presentation facts) with the five invariants and the two new relationship rows. Glossary adds GLO-013 Document Type, GLO-014 Document Model, GLO-015 SingleMarkdown, GLO-016 GroupedMarkdown, GLO-017 SingleHtmlMockup, GLO-018 Field Specification, GLO-019 Role, GLO-020 Group Record, GLO-021 Member Record, GLO-022 Seed. Context map unchanged: the object moves no aggregate boundary, stated in the object model. papyrus check: 3 pre-existing frontmatter-conflict findings, none new.
- **orchestrator**: Acceptance met: Document Type added to the business object model with identity, model, roles, field vocabulary, derived verbs, presentation facts and five invariants; glossary GLO-013..GLO-022 added; context map unchanged.
