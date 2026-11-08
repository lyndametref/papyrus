---
id: TASK-65
title: Expose front-matter metadata options on all record commands
status: Done
assignee: []
created_date: '2026-11-08T15:06:00Z'
updated_date: '2026-11-08T18:06:00Z'
labels:
  - front-matter
  - cli
  - metadata
milestone: m-12
dependencies:
  - TASK-63
  - TASK-64
references:
  - docs/requirements/business/REQ-05.010.md
  - docs/requirements/business/REQ-05.011.md
  - docs/requirements/business/REQ-06.004.md
  - docs/requirements/non-functional/NFR-04.003.md
  - >-
    docs/architecture/adr/ADR-0007-yaml-front-matter-as-single-record-metadata-source.md
  - >-
    docs/architecture/adr/ADR-0008-adopt-backlog-md-on-disk-layout-for-tasks-and-milestones.md
  - >-
    docs/architecture/adr/ADR-0009-shared-record-envelope-and-unified-write-path.md
documentation:
  - >-
    docs/specifications/expose-front-matter-metadata-options-on-all-record-commands.md
priority: high
type: feature
ordinal: 30030
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Expose create and update options for every front-matter field except id, created_date, and updated_date on all managed record commands, with automatic timestamps, ordinal defaulting, and patch-preserving writes (REQ-05.010, REQ-05.011, REQ-06.004, NFR-04.003; ADR-0007, ADR-0008, ADR-0009).

Managed record types: task, milestone, release, decision, business requirement, non-functional requirement, glossary term, UI specification, technical specification, business rule, and audit record. General fields on every type: title, labels, references. Type-specific fields: task status, assignee (list), milestone, dependencies, ordinal, documentation, type; milestone release; decision superseded_by; audit type.

created_date is set once at creation and never changed. updated_date is set at creation and refreshed only by writes that change at least one byte; a no-op write does not refresh it. ordinal is an opaque non-negative integer supplied through create/update options and defaulted to max(observed ordinals for the type across active and archived or deprecated locations) + 1, or 0 when none exist.

Every write uses record::patch and record::store::write_atomic: only changed keys are patched, unknown keys, key order, comments, marker comments, body lines, and line endings are preserved byte-for-byte, and an empty or unchanged patch produces byte-identical text with no write and no timestamp refresh. Commands continue to accept and display Papyrus display IDs only; storage IDs are translated at the boundary by record::identity. Browser saves use the same patch and store path and keep created_date and updated_date read-only.

Assumptions: flag names follow existing Papyrus conventions and add --reference, --documentation, --ordinal, and --type only where missing; repeatable list fields append unique values on edit; --assignee accepts one or more values and replaces the assignee list; ordinal defaults use max(observed)+1 or 0; general references and documentation paths are not validated for existence; lazy migration, legacy removal, and browser UI changes remain out of scope (TASK-68, TASK-69, TASK-70, TASK-66, TASK-67, TASK-72).

Open questions: whether list fields should gain replace/remove variants; whether documentation paths and general references should be validated for existence; whether archive, restore, deprecate, and restore operations should refresh updated_date.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Every managed record type exposes create and update options for every front-matter key except id, created_date, and updated_date.
- [ ] #2 Task options cover status, assignee as a list, milestone, dependencies, ordinal, documentation, and type; milestone adds release; decision adds superseded_by; audit adds type; every type also exposes labels and references.
- [ ] #3 created_date is written once at creation and never changed; updated_date is refreshed by every content-changing write; a no-op write does not refresh updated_date.
- [ ] #4 ordinal is an opaque non-negative integer with a create and update option, defaulted to max(observed ordinals for the type across active and archived or deprecated locations) + 1, or 0 when none exist.
- [ ] #5 Existing commands that lack an option for an applicable field gain that option for task, milestone, release, decision, requirement, NFR, glossary, UI spec, technical spec, business rule, and audit records.
- [ ] #6 Every write patches only the keys the operation changes and preserves all other front-matter keys, key order, comments, body content, and line endings byte-for-byte.
- [ ] #7 A browser save of a metadata-backed field writes the new value to front matter, preserves the remaining fields and body, keeps created_date and updated_date read-only, and does not refresh updated_date on a no-op save.
- [ ] #8 Timestamps are quoted ISO-8601 UTC strings of the form 'YYYY-MM-DDTHH:MM:SSZ'.
- [ ] #9 Tests assert option coverage per type, timestamp behavior, ordinal defaults, and field preservation, including no-op byte identity.
- [ ] #10 Task and milestone files created or edited with the new options remain readable, listable, and updatable by Backlog.md, including additive keys (NFR-04.003).
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 cargo test passes for the record module, affected command modules, and integration tests.
- [ ] #2 No fs::write call on a record path exists outside src/record/store.
- [ ] #3 No front-matter delimiter formatting exists outside src/record/.
- [ ] #4 Feature specification is updated and linked from the ticket.
- [ ] #5 No existing corpus record file is modified by this stage.
<!-- DOD:END -->
