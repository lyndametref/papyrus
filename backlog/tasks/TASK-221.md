---
id: TASK-221
title: "Write ADR-D: one descriptor-driven document view and edit surface"
created_date: '2026-11-20T17:36:52Z'
updated_date: '2026-11-20T17:36:52Z'
labels: []
references:
  - ADR-0020
  - ADR-0028
  - ADR-0033
  - ADR-0035
  - REQ-06.014
  - REQ-06.019
  - TECH-066
  - TECH-067
status: ToDo
assignee:
  - architect
milestone: MS-031
dependencies:
  - TASK-218
ordinal: 50
documentation:
  - decisions/ADR-039-one-descriptor-driven-document-surface.md
type: architecture
---

## Description
Context: the browser renders each document type through its own view, edit, form_post, conflict, and not-found code. There are 9 conflict-page renderers and 5 not-found pages. Presentation facts are duplicated in code instead of coming from configuration.

Scope: author ADR-039, "One descriptor-driven document view and edit surface".
- Supersede ADR-0020 decision 5 and its rejected alternative.
- Amend ADR-0028 decisions 1 and 2 and constraint 10, and ADR-0020 decisions 1 and 3.
- Define one generic list, view, edit, conflict, and not-found renderer driven by the record envelope plus the descriptor field map.
- Move presentation facts (labels, section headings, field order, help text) into the config seed, with no per-type code branch.
- Define one generic form_post that preserves optimistic concurrency and the existing error surface.
- Define one generic 404 and one generic conflict page replacing the per-type variants.
- Keep Backlog and Decision browser surfaces bespoke and unchanged.
- State that reference visibility comes from seed reference_visibility per type and role, superseding the ADR-0035 discriminator as amended.
