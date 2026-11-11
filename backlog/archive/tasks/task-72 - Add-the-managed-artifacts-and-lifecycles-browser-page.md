---
id: TASK-72
title: Add the managed artifacts and lifecycles browser page
status: Done
assignee: []
created_date: '2026-11-08T15:06:00Z'
updated_date: '2026-11-09T21:10:00Z'
labels:
  - browser
  - lifecycle
  - documentation
milestone: m-12
dependencies:
  - TASK-66
references:
  - docs/requirements/business/REQ-06.003.md
  - docs/requirements/business/REQ-06.002.md
  - docs/requirements/business/REQ-01.002.md
  - docs/requirements/business/REQ-03.002.md
  - docs/requirements/business/REQ-04.001.md
  - docs/requirements/business/REQ-05.007.md
  - docs/requirements/non-functional/NFR-01.002.md
  - docs/requirements/business/REQ-03.001.md
  - docs/requirements/business/REQ-05.006.md
  - docs/requirements/business/REQ-05.008.md
documentation:
  - docs/specifications/browser-managed-artifacts-and-lifecycles.md
priority: medium
type: feature
ordinal: 30097.5
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add a read-only browser page that describes every managed artifact/document type and its lifecycle, per REQ-06.003 and the approved MOCK-0010 layout. The page covers a fixed set of twelve types (tasks, milestones, releases, decisions, business requirements, non-functional requirements, glossary terms, UI specifications, technical specifications, UI mockups, business rules, audit records); the set does not change with configured document_types. The task section renders the configured task statuses (REQ-01.002) in their configured order. Every other section states the documented lifecycle for its type (REQ-03.001, REQ-03.002, REQ-04.001, REQ-05.006, REQ-05.007, REQ-05.008). Backlog.md docs, drafts, and project configuration files are excluded. The page is HTML only, reuses the shared shell/sidebar/CSP, and adds a sidebar entry 'Managed artifacts & lifecycles' at the end of the list. No records are listed, viewed, edited, or mutated by this page; all lifecycle operations remain CLI-only.

Scope and behavior are fully specified in docs/specifications/browser-managed-artifacts-and-lifecycles.md. Lifecycle wording uses the documented lifecycles (per user selection during refinement), not the shorthand sentences in MOCK-0010 where those conflict.

Assumptions:
- Assumption: The page is read-only and renders no project records.
- Assumption: Route is /lifecycle with /?type=lifecycle as an alias.
- Assumption: The sidebar entry is always present, independent of document_types.*.enabled.
- Assumption: Custom task statuses have no per-state descriptions; only their names and order are rendered.
- Assumption: No per-state one-line descriptions beyond the single lifecycle sentence per section.

Dependencies: TASK-66 (Done). Related: TASK-19, TASK-20 (depend on this ticket).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The browser serves a page with one section for each of tasks, milestones, releases, decisions, business requirements, non-functional requirements, glossary terms, UI specifications, technical specifications, UI mockups, business rules, and audit records.
- [ ] #2 The covered set is fixed and does not change with configured document_types.
- [ ] #3 Task states and transitions reflect the configured task statuses.
- [ ] #4 Every other section explains the documented lifecycle states and transitions and agrees with REQ-03.002, REQ-04.001, and REQ-05.007.
- [ ] #5 Backlog.md docs, drafts, and project configuration files are not described.
- [ ] #6 A navigation entry reaches the page and the layout follows the approved mockup.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Feature specification docs/specifications/browser-managed-artifacts-and-lifecycles.md created and marked execution-ready
- [ ] #2 Implementation follows the approved MOCK-0010 layout and the documented lifecycle wording in the specification
- [ ] #3 Unit/integration tests added for the page route, all twelve sections, configured-status rendering, alias route, method handling, and custom status list
- [ ] #4 No code, dependency, or ADR changes beyond those required by the specification
<!-- DOD:END -->
