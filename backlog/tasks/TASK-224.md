---
id: TASK-224
title: "Rewrite REQ-07.002 for the nested document-type entry shape, defaults, and diagnostics"
created_date: '2026-11-20T17:37:43Z'
updated_date: '2026-11-20T20:24:50Z'
labels: []
references:
  - ADR-036
  - ADR-0019
  - REQ-01.002
  - TECH-077
  - REQ-07.001
  - REQ-05.010
  - REQ-01.008
  - BR-016
status: Done
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
## Refinement inputs (TASK-290)

### Requirement file map (current section numbers)
- REQ-07.002 (§7.2) — docs/requirements/business/REQ-07.002.md — Configure Document Types and Slugs — rewritten record; sections `Description`, `Acceptance Criteria`, `Open Questions`, `Traceability`
- REQ-07.001 (§7.1) — docs/requirements/business/REQ-07.001.md — Organize Artifacts into Three Categories — status model; read-only boundary input
- REQ-05.010 (§5.10) — docs/requirements/business/REQ-05.010.md — Record Metadata in YAML Front Matter — general front-matter keys; read-only input
- REQ-01.002 (§1.2) — docs/requirements/business/REQ-01.002.md — Read and Edit Project Configuration — configuration command surface; read-only input

### Refinement assumptions / definition of done
RE-010 D14 (nested config, fields sequence, selector paths, no model/title/content/id_shape keys) is accepted and not reopenable. Only the listed REQ file changes; the user verifies the rewrite. Done means all acceptance criteria pass and no source or test file changes (MS-031 AC-10).
## Acceptance Criteria
- [ ] REQ-07.002 documents every top-level entry key in the scope, including the nested group and member blocks and the fields sequence (MS-031 AC-4, RE-010 D14).
- [ ] REQ-07.002 documents the field item keys and constraints: key, heading, frontmatter_key, option, label, kind, required, verbs, help; no order or enabled; heading and/or frontmatter_key required; reserved field keys and unique keys per role (MS-031 AC-4).
- [ ] REQ-07.002 states the default and merge rules: scalars fill from the seed, fields and group.fields and member.fields and statuses replace wholesale, prefix and directory are never filled, and an absent slug stays absent (MS-031 AC-4, RE-010 D14).
- [ ] REQ-07.002 states unknown-key and unknown-enum rejection with the `document_types.<slug>.<path>` form, selector paths with numeric [0], config list effective values with key selectors, config unset collection reset, fields: [] validity, and minimal init entries (MS-031 AC-4, AC-9).
- [ ] REQ-07.002 states fail-closed one-pass validation, has updated Acceptance Criteria and Traceability, cites no archived REQ-06.003, and the user has verified the rewrite (MS-031 AC-4, AC-8).
## Comments
- **requirements-engineer**: REQ-07.002 rewritten with 40 acceptance criteria covering the nested entry shape, field item vocabulary, reserved and duplicate field keys, scalar fill and collection replace, prefix and directory never filled, absent slug absent, minimal init, config set target-entry materialization, selector paths with keyed and indexed forms, config list effective values, config unset collection reset, comma syntax only for flat lists, and one-pass document_types.<slug>.<path> diagnostics. papyrus check shows 3 findings, unchanged from baseline, none on REQ-07.002. Status kept In-Progress.
- **orchestrator**: Acceptance met: REQ-07.002 rewritten with 40 testable acceptance criteria covering entry shape, field vocabulary, fill and merge, diagnostics, selector paths and comma rules; open ambiguity on config set materialization vs ADR-0019 d9 recorded for TASK-258.
