---
id: TASK-228
title: "Update BR-001 and BR-016 for the derived document command surface and nested configuration schema"
created_date: '2026-11-20T17:39:27Z'
updated_date: '2026-11-20T18:15:40Z'
labels: []
references:
  - BR-001
  - BR-016
  - REQ-07.002
  - REQ-07.003
  - ADR-036
  - REQ-01.002
  - TECH-070
  - TECH-077
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
## Refinement inputs (TASK-290)

### Business-rule file map
- BR-001 — `docs/business-rules/BR-001-papyrus-command-behavior.md` — sections `Description`, `Rationale`, `Applicability` (Command model; Categories and lifecycles).
- BR-016 — `docs/business-rules/BR-016-configuration-keys-and-precedence.md` — sections `Description`, `Rationale`, `Applicability` (Precedence; General, path, and workflow keys; document_types; Validation; Repair surfaces).
- Read-only consistency inputs: REQ-07.002, REQ-07.003, ADR-036 (`decisions/ADR-036-config-declared-document-field-schema.md`).

### Refinement assumptions / definition of done
BR-001 and BR-016 describe the generated surface and nested entry shape; no manual command builder or flat-entry rule survives. Only the two BR files change; the user verifies both. Done means all acceptance criteria pass and no source or test file changes (MS-031 AC-10).
## Acceptance Criteria
- [ ] BR-001 states that the document command family is generated from the descriptor, with baseline verbs plus create-group for GroupedMarkdown and supersede for SingleHtmlMockup, and no status verb for SingleHtmlMockup (MS-031 AC-6, RE-010 D6, D13).
- [ ] BR-001 states that title is always the first positional on create and create-group, with --title on create, create-group, and edit, and that Backlog and Decision commands are unchanged (MS-031 AC-6, RE-010 D13).
- [ ] BR-016 documents entry keys, nested group and member blocks, the fields sequence, selector path grammar, scalar default-fill, collection replace, prefix and directory never filled, absent slug stays absent, reserved front-matter keys, and the `document_types.<slug>.<path>` diagnostic form (MS-031 AC-6, RE-010 D14).
- [ ] BR-016 records precedence exactly: file value beats seed for scalars, a configured collection replaces the seed collection wholesale, and config unset restores the seed collection (MS-031 AC-6).
- [ ] Both rules remain consistent with REQ-07.002, REQ-07.003, and ADR-036, and the user has verified them (MS-031 AC-6, AC-8).
