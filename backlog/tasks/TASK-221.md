---
id: TASK-221
title: "Write ADR-D: one descriptor-driven document view and edit surface"
created_date: '2026-11-20T17:36:52Z'
updated_date: '2026-11-20T18:15:37Z'
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
  - TECH-068
status: ToDo
assignee:
  - architect
milestone: MS-031
dependencies:
  - TASK-218
ordinal: 50
documentation:
  - decisions/ADR-039-one-descriptor-driven-document-surface.md
  - decisions/ADR-0020-registry-derived-browser-views-for-document-types.md
  - decisions/ADR-0028-browser-shell-shared-list-layout-content-width-and-collapsible-navigation.md
  - decisions/ADR-035-per-family-reference-visibility-in-the-shared-browser-list-layout.md
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
## Refinement inputs (TASK-290)

### ADR inventory and amendment targets (`decisions/`)
Inputs: ADR-0020 d5/d1/d3 and its rejected alternative (`decisions/ADR-0020-registry-derived-browser-views-for-document-types.md`), ADR-0028 d1/d2/constraint 10 (`decisions/ADR-0028-browser-shell-shared-list-layout-content-width-and-collapsible-navigation.md`), ADR-0035 (`decisions/ADR-035-per-family-reference-visibility-in-the-shared-browser-list-layout.md`).
Output path: `decisions/ADR-039-one-descriptor-driven-document-surface.md`.

### Refinement assumptions / definition of done
The source of truth is RE-010 D1-D14 (accepted, not reopenable) and MS-031 AC-2/AC-3. No decision is renumbered, softened, split, or reopened. Done means every acceptance criterion passes, the record is accepted and user-verified, and no source, test, or packaging file changes (MS-031 AC-10).
## Acceptance Criteria
- [ ] ADR-039 exists at `decisions/ADR-039-one-descriptor-driven-document-surface.md` with status accepted (MS-031 AC-2D).
- [ ] ADR-039 supersedes ADR-0020 decision 5 and its rejected alternative, and records dated amendments to ADR-0028 decisions 1, 2, and constraint 10 and ADR-0020 decisions 1 and 3 (MS-031 AC-2D).
- [ ] ADR-039 defines one generic list, view, edit, conflict, and not-found renderer driven by the record envelope plus the descriptor field map, with one generic form_post that preserves optimistic concurrency and the existing error surface (MS-031 AC-2D).
- [ ] ADR-039 moves presentation facts (labels, section headings, field order, help text) into the config seed with no per-type code branch, keeps Backlog and Decision browser surfaces unchanged, and sources reference visibility from seed reference_visibility per type and role (MS-031 AC-2D, RE-010 D4, D8, D10).
- [ ] ADR-039 leaves every source, test, and packaging file unchanged (MS-031 AC-10).
