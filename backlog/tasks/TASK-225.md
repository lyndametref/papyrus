---
id: TASK-225
title: "Update REQ-07.001 and REQ-07.003 for status-model separation, derived verbs, and the uniform title contract"
created_date: '2026-11-20T17:37:43Z'
updated_date: '2026-11-20T17:37:43Z'
labels: []
references:
  - ADR-036
  - ADR-0016
  - ADR-0019
  - REQ-05.011
  - TECH-070
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
