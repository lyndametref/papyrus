---
id: TASK-229
title: "Update BR-009 to BR-014 for the unified document engine"
created_date: '2026-11-20T17:39:27Z'
updated_date: '2026-11-20T18:15:40Z'
labels: []
references:
  - BR-009
  - BR-010
  - BR-011
  - BR-012
  - BR-013
  - BR-014
  - REQ-05.001
  - REQ-05.003
  - REQ-05.005
  - REQ-05.006
  - REQ-07.003
  - BR-001
  - BR-016
  - REQ-05.002
  - REQ-05.004
  - REQ-05.008
status: ToDo
assignee:
  - requirements-engineer
milestone: MS-031
dependencies:
  - TASK-226
  - TASK-225
ordinal: 130
documentation:
  - docs/business-rules/BR-009-req-command-behavior.md
  - docs/business-rules/BR-010-nfr-command-behavior.md
  - docs/business-rules/BR-011-glossary-command-behavior.md
  - docs/business-rules/BR-012-ui-spec-command-behavior.md
  - docs/business-rules/BR-013-rule-command-behavior.md
  - docs/business-rules/BR-014-audit-command-behavior.md
type: requirements
---

## Description
Context: six per-type command behavior rules describe hand-written command surfaces. After the refactor these commands are generated, so each rule must describe the generated behavior without losing any user-visible invariant.

Scope:
- BR-009 (req) and BR-010 (nfr): create-group creates a group record with a plain ID; create requires an existing active group of the same type; a cross-type group and an archived group are rejected; member IDs are dotted; membership is immutable after creation; archive or delete cascades to nothing; merged list ordering is preserved; references are hidden.
- BR-011 (glossary): title is the positional argument; --term is removed; definition is required with heading "## Definition"; existing term values survive as titles.
- BR-012 (ui-spec): SingleMarkdown baseline; fields come from the seed.
- BR-013 (rule): description required; rationale and applicability are create-only; requirements is a list field with option requirement.
- BR-014 (audit): title is positional; audit_type is a text field with option audit-type and frontmatter_key type; content is required.
## Refinement inputs (TASK-290)

### Business-rule file map
- BR-009 — `docs/business-rules/BR-009-req-command-behavior.md` — sections `Description`, `Rationale`, `Applicability` (Type entry; Verbs; Status and archive).
- BR-010 — `docs/business-rules/BR-010-nfr-command-behavior.md` — sections `Description`, `Rationale`, `Applicability` (Type entry; Verbs; Status and archive).
- BR-011 — `docs/business-rules/BR-011-glossary-command-behavior.md` — sections `Description`, `Rationale`, `Applicability` (Type entry; Verbs; Status and archive).
- BR-012 — `docs/business-rules/BR-012-ui-spec-command-behavior.md` — sections `Description`, `Rationale`, `Applicability` (Type entry; Verbs; Status and archive).
- BR-013 — `docs/business-rules/BR-013-rule-command-behavior.md` — sections `Description`, `Rationale`, `Applicability` (Type entry; Verbs; Status and archive).
- BR-014 — `docs/business-rules/BR-014-audit-command-behavior.md` — sections `Description`, `Rationale`, `Applicability` (Type entry; Verbs; Status and archive).

### Refinement assumptions / definition of done
Each rule must keep every user-visible invariant of its old hand-written command family while describing generated behavior. Only the six listed BR files change; the user verifies all six. Done means all acceptance criteria pass and no source or test file changes (MS-031 AC-10).
## Acceptance Criteria
- [ ] BR-009 to BR-014 each describe the generated command surface with no lost user-visible invariant (MS-031 AC-6).
- [ ] BR-009 (req) and BR-010 (nfr) state create-group plain IDs, same-type active group requirement, cross-type and archived-group rejection, dotted member IDs, immutable membership, no archive or delete cascade, merged list ordering, and hidden references (MS-031 AC-6, RE-010 D7).
- [ ] BR-011 states the positional title, --term removal, required definition with heading "## Definition", and survival of existing term values as titles (MS-031 AC-6, RE-010 D13).
- [ ] BR-012 states the SingleMarkdown baseline and seed-sourced fields; BR-013 states required description, create-only rationale and applicability, and the list requirements field; BR-014 states positional title, audit_type option and frontmatter_key, and required content (MS-031 AC-6).
- [ ] The six rules remain consistent with BR-001, BR-016, REQ-05.001 to REQ-05.009, and ADR-036, and the user has verified them (MS-031 AC-6, AC-8).
