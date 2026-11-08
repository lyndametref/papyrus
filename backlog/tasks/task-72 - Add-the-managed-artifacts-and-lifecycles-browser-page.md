---
id: TASK-72
title: Add the managed artifacts and lifecycles browser page
status: To Do
assignee: []
created_date: '2026-11-08T15:06:00Z'
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
priority: medium
type: feature
ordinal: 30090
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add the browser page required by REQ-06.003 describing every managed artifact type and its lifecycle, using configured task statuses.
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
