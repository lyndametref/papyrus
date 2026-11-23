---
id: TASK-223
title: "Rewrite REQ-007.004 for one schema-driven document content model with three models"
created_date: '2026-11-20T17:37:30Z'
updated_date: '2026-11-20T20:17:03Z'
labels: []
references:
  - ADR-036
  - ADR-0017
  - ADR-0018
  - REQ-005.010
  - TECH-070
  - TECH-071
  - REQ-005.003
  - REQ-005.008
  - REQ-007.001
  - REQ-007.002
  - REQ-007.003
  - BR-017
status: Done
assignee:
  - requirements-engineer
milestone: MS-031
dependencies:
  - TASK-218
ordinal: 70
documentation:
  - docs/requirements/business/REQ-007.004.md
type: requirements
---

## Description
Context: REQ-007.004 currently promises a generic model with no per-type schema and excludes config-defined types from type-specific fields. The accepted architecture replaces that with a schema declared in configuration and three document models.

Scope: full rewrite of REQ-007.004.
- Define DocumentModel with SingleMarkdown, GroupedMarkdown, SingleHtmlMockup; ContentModel is removed.
- A type declares its field set: one fields sequence for SingleMarkdown and SingleHtmlMockup, group.fields plus member.fields for GroupedMarkdown.
- Remove the seven-key fixed content model, the "no per-type field schema" exclusion, and the "no type-specific body section" exclusion for user-defined types.
- GroupedMarkdown: group record with plain ID, member records with dotted IDs, engine-owned group link, role field sets, immutable membership after creation.
- SingleHtmlMockup: one record, plain ID, no status verb, fixed pending/approved/archived metadata.
- Seed-only restriction: GroupedMarkdown and SingleHtmlMockup only for seed types; only req and nfr are grouped; only ui-mockup is HTML; user-defined types default to SingleMarkdown.
- Title is fixed and non-configurable: single-line, non-empty, always present, required on create, never clearable on edit, not a field, reserved field key.
- Configuration declares fields and presentation but is never executable: no templates, expressions, or code.
- The engine emits only PatchOps; it never writes a record body directly.
## Refinement inputs (TASK-290)

### Requirement file map (current section numbers)
- REQ-007.004 (§7.4) — docs/requirements/business/REQ-007.004.md — Provide a Generic Content Model for Config-Defined Document Types — rewritten record; sections `Description`, `Acceptance Criteria`, `Open Questions`, `Traceability`
- REQ-007.001 (§7.1) — docs/requirements/business/REQ-007.001.md — Organize Artifacts into Three Categories — status-model boundary; read-only input
- REQ-007.002 (§7.2) — docs/requirements/business/REQ-007.002.md — Configure Document Types and Slugs — config entry shape; read-only input
- REQ-007.003 (§7.3) — docs/requirements/business/REQ-007.003.md — Dispatch Config-Defined Document Commands — derived command surface; read-only input
- REQ-005.010 (§5.10) — docs/requirements/business/REQ-005.010.md — Record Metadata in YAML Front Matter — general front-matter keys; read-only input

### Refinement assumptions / definition of done
RE-010 D5/D6/D9/D13/D14 are accepted and not reopenable. Only the listed REQ file changes; the user verifies the rewrite. No archived REQ-006.003 is cited. Done means all acceptance criteria pass and no source or test file changes (MS-031 AC-10).
## Acceptance Criteria
- [ ] REQ-007.004 defines DocumentModel with SingleMarkdown, GroupedMarkdown, and SingleHtmlMockup and removes ContentModel and the seven-key fixed model (MS-031 AC-4).
- [ ] REQ-007.004 defines one fields sequence for SingleMarkdown and SingleHtmlMockup, and group.fields plus member.fields for GroupedMarkdown, and removes the no-per-type-schema and no-type-specific-body-section exclusions (MS-031 AC-4).
- [ ] REQ-007.004 states the GroupedMarkdown and SingleHtmlMockup behavior in the scope, including immutable membership, the engine-owned group link, and fixed pending, approved, and archived metadata with no status verb (MS-031 AC-4, RE-010 D6, D7).
- [ ] REQ-007.004 states the seed-only model restriction, the fixed title contract, that configuration is never executable, and that the engine emits only PatchOps (MS-031 AC-4, RE-010 D5, D6, D13).
- [ ] REQ-007.004's Acceptance Criteria and Traceability sections are updated coherently, cite no archived REQ-006.003, and the user has verified the rewrite (MS-031 AC-4, AC-8).
## Comments
- **requirements-engineer**: REQ-007.004 rewritten with 18 acceptance criteria, papyrus check unchanged at 3 pre-existing archived-task findings
- **orchestrator**: Acceptance met: REQ-007.004 rewritten with 18 acceptance criteria covering DocumentModel, field schema, title contract, group semantics, mockup semantics, seed-only rules and PatchOps-only engine; traceability to RE-010 and ADR-036, ADR-039; no new check findings.
