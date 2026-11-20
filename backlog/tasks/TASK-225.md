---
id: TASK-225
title: "Update REQ-07.001 and REQ-07.003 for status-model separation, derived verbs, and the uniform title contract"
created_date: '2026-11-20T17:37:43Z'
updated_date: '2026-11-20T18:15:39Z'
labels: []
references:
  - ADR-036
  - ADR-0016
  - ADR-0019
  - REQ-05.011
  - TECH-070
  - REQ-01.002
  - REQ-05.003
  - REQ-05.006
  - BR-001
  - BR-011
status: ToDo
assignee:
  - requirements-engineer
milestone: MS-031
dependencies:
  - TASK-218
ordinal: 90
documentation:
  - docs/requirements/business/REQ-07.001.md
  - docs/requirements/business/REQ-07.003.md
type: requirements
---

## Description
Context: the content model changes must not alter the status model, and the command surface must be derived from the model instead of hand-written per type.

Scope:
REQ-07.001 (light amendment): content model must not alter the status model; a record's status stays active, a declared custom status, or archived; status changes only through the status and archive verbs; SingleHtmlMockup has the fixed states pending, approved, archived and no status verb; archiving remains the only path to archived.
REQ-07.003 (rewrite of command derivation): the baseline verb set is derived from the model; create-group and --group appear only for GroupedMarkdown; supersede appears only for SingleHtmlMockup; field options are derived from the field sequence and its verbs; uniform title contract: first positional TITLE plus --title on create and create-group, --title on edit, required on create, non-empty, never clearable on edit, reserved field key; --term is removed and glossary uses the title contract; audit and create-group gain a positional title additively (no breaking rename of existing positional arguments); help output is deterministic across runs and machines; about lines and type order are frozen while per-option prose may regenerate.
## Refinement inputs (TASK-290)

### Requirement file map (current section numbers)
- REQ-07.001 (§7.1) — docs/requirements/business/REQ-07.001.md — Organize Artifacts into Three Categories — light amendment; sections `Description`, `Acceptance Criteria`, `Traceability`
- REQ-07.003 (§7.3) — docs/requirements/business/REQ-07.003.md — Dispatch Config-Defined Document Commands — full rewrite; sections `Description`, `Acceptance Criteria`, `Open Questions`, `Traceability`
- REQ-05.011 (§5.11) — docs/requirements/business/REQ-05.011.md — Maintain Record Metadata Through All Commands — status/archive behavior; read-only input
- REQ-01.002 (§1.2) — docs/requirements/business/REQ-01.002.md — Read and Edit Project Configuration — configuration surface; read-only input

### Refinement assumptions / definition of done
The content model must not alter the status model. RE-010 D13 (fixed title, uniform positional title, --term removal) and D6/D7 are accepted and not reopenable. Only the two listed REQ files change; the user verifies both. Done means all acceptance criteria pass and no source or test file changes (MS-031 AC-10).
## Acceptance Criteria
- [ ] REQ-07.001 states that the content model does not alter the status model, that status changes only through the status and archive verbs, and that SingleHtmlMockup has fixed pending, approved, and archived states and no status verb (MS-031 AC-4, RE-010 D6, D7).
- [ ] REQ-07.003 derives the baseline verb set from the model; create-group and --group appear only for GroupedMarkdown, supersede only for SingleHtmlMockup, and field options come from the field sequence and its verbs (MS-031 AC-4).
- [ ] REQ-07.003 states the uniform title contract: first positional TITLE plus --title on create and create-group, --title on edit, required on create, non-empty, never clearable on edit, reserved field key; --term is removed and glossary uses the title contract; audit and create-group gain a positional title additively (MS-031 AC-4, RE-010 D13).
- [ ] REQ-07.003 states deterministic help output across runs and machines, with frozen about lines and type order while per-option prose may regenerate (MS-031 AC-4).
- [ ] Both files have updated Acceptance Criteria and Traceability, cite no archived REQ-06.003, and the user has verified the changes (MS-031 AC-4, AC-8).
