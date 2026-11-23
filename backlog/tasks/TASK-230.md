---
id: TASK-230
title: "Update BR-017 to BR-020 for config-declared SingleMarkdown document types"
created_date: '2026-11-20T17:39:27Z'
updated_date: '2026-11-20T21:22:43Z'
labels: []
references:
  - BR-017
  - BR-018
  - BR-019
  - BR-020
  - REQ-005.009
  - REQ-005.012
  - REQ-005.013
  - REQ-005.014
  - BR-001
  - BR-016
status: Done
assignee:
  - requirements-engineer
milestone: MS-031
dependencies:
  - TASK-226
ordinal: 140
documentation:
  - docs/business-rules/BR-017-tech-spec-command-behavior.md
  - docs/business-rules/BR-018-use-case-document-type-behavior.md
  - docs/business-rules/BR-019-test-case-document-type-behavior.md
  - docs/business-rules/BR-020-test-report-document-type-behavior.md
type: requirements
---

## Description
Context: tech-spec, use-case, test-case, and test-report lose their dedicated modules and become SingleMarkdown entries in the seed. Their existing fields and command behavior must survive as configuration.

Scope:
- BR-017 (tech-spec), BR-018 (use-case), BR-019 (test-case), BR-020 (test-report): each becomes a SingleMarkdown document type whose fields are declared in the seed; baseline verbs plus status; no type-specific verb; title is the positional argument and --title on edit; front-matter keys come from each field's frontmatter_key; the existing body headings and field order are preserved in the seed sequence.
- Record the exact field list for each type in the rule so a later seed change is a requirement change, not a silent behavior change.
- State that these types are user-extensible: adding a field is a file edit and needs no code.
## Refinement inputs (TASK-290)

### Business-rule file map
- BR-017 — `docs/business-rules/BR-017-tech-spec-command-behavior.md` — sections `Description`, `Rationale`, `Applicability` (Type entry; Verbs and content; Status and archive).
- BR-018 — `docs/business-rules/BR-018-use-case-document-type-behavior.md` — sections `Description`, `Rationale`, `Applicability` (Type entry; Verbs and content; Status and archive).
- BR-019 — `docs/business-rules/BR-019-test-case-document-type-behavior.md` — sections `Description`, `Rationale`, `Applicability` (Type entry; Verbs and content; Status and archive).
- BR-020 — `docs/business-rules/BR-020-test-report-document-type-behavior.md` — sections `Description`, `Rationale`, `Applicability` (Type entry; Verbs and content; Status and archive).

### Refinement assumptions / definition of done
The exact field list for each type is the behavior contract; a later seed change is a requirement change. Only the four listed BR files change; the user verifies all four. Done means all acceptance criteria pass and no source or test file changes (MS-031 AC-10).
## Acceptance Criteria
- [ ] BR-017 to BR-020 each state that the type is SingleMarkdown with fields declared in the seed, baseline verbs plus status, and no type-specific verb (MS-031 AC-6, RE-010 D6).
- [ ] Each rule states the positional title and --title on edit, and that front-matter keys come from each field's frontmatter_key (MS-031 AC-6, RE-010 D13).
- [ ] Each rule records the exact field list and preserves the existing body headings and field order in the seed sequence (MS-031 AC-6).
- [ ] Each rule states that these types are user-extensible and adding a field is a file edit requiring no code (MS-031 AC-6, RE-010 D14).
- [ ] The four rules remain consistent with REQ-005.009, REQ-005.012, REQ-005.013, REQ-005.014, and ADR-036, and the user has verified them (MS-031 AC-6, AC-8).
## Comments
- **requirements-engineer**: Updated BR-017 to BR-020 for seed-declared SingleMarkdown descriptors. Each rule now records the exact field table with key, heading, kind, required, verbs, option and frontmatter_key; baseline verbs plus status with no type-specific verb; positional title on create and --title on edit; body-only storage; and user extensibility by configuration file edit with no code change. Requirement links added for REQ-005.009, REQ-005.012, REQ-005.013, REQ-005.014, REQ-007.002, REQ-007.003, REQ-007.004 and REQ-005.007. papyrus check unchanged with the same three pre-existing task findings.
- **orchestrator**: Acceptance met: BR-017..020 rewritten as seeded SingleMarkdown types with exact field lists, model-derived verbs, title contract and user extensibility; no new check findings.
