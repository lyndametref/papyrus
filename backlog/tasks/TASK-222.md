---
id: TASK-222
title: "Amend ADR-0019, ADR-0016, ADR-0012, ADR-0023, ADR-0035, ADR-0028, ADR-0024, ADR-0025, and ADR-0034 for the new model"
created_date: '2026-11-20T17:37:01Z'
updated_date: '2026-11-20T17:37:01Z'
labels: []
references:
  - ADR-0012
  - ADR-0016
  - ADR-0019
  - ADR-0023
  - ADR-0024
  - ADR-0025
  - ADR-0028
  - ADR-0034
  - ADR-0035
  - ADR-036
  - ADR-037
  - ADR-038
  - ADR-039
status: ToDo
assignee:
  - architect
milestone: MS-031
dependencies:
  - TASK-218
  - TASK-219
  - TASK-220
  - TASK-221
ordinal: 60
documentation:
  - decisions/ADR-0019-fail-closed-document-type-configuration-validation.md
  - decisions/ADR-0016-runtime-config-derived-command-dispatch.md
  - decisions/ADR-0035-per-family-reference-visibility-in-the-shared-browser-list-layout.md
type: architecture
---

## Description
Context: ADR-036 to ADR-039 replace the descriptor and content-model decisions, but nine other ADRs state invariants that the new model changes. Leaving them unamended produces contradictory architecture.

Scope: amend in place, preserving decision numbering and recording each amendment as a dated note.
- ADR-0019: d7 key on the content model instead of the ui-mockup slug; d8 entry-set wholesale versus collection replace; d10 selector paths and adding fields file-edit-only; d11 new invariants (reserved field keys, seed-only models, unique field keys per role); d13 indexed field diagnostics.
- ADR-0016: d4 and d7 help title arguments are engine-generated; about lines and type order are frozen; per-option help prose and order may regenerate.
- ADR-0012 d2 and d6 plus ADR-0023 d5 and constraint 4: config-declared front-matter keys are a second extension point; general front-matter keys stay protected; heading ownership is stated.
- ADR-0035: replacement discriminator is model plus seed reference_visibility.
- ADR-0028: generic edit renderer; reserved field keys versus slots.
- ADR-0024: nested mapping sequences are not list-valued keys.
- ADR-0025: prose-field selection for the math pre-pass.
- ADR-0034: note that ADR-0020 d9 is already retired and must not be reintroduced as a lifecycle page.

Do not renumber decisions. Do not modify ADR-0017 or ADR-0018 here; their supersession belongs to ADR-036.
