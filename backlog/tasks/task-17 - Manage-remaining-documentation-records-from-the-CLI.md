---
id: TASK-17
title: Manage remaining documentation records from the CLI
status: To Do
assignee: []
created_date: '2026-10-31T22:26:00Z'
labels: []
milestone: m-4
dependencies:
  - TASK-16
references:
  - docs/requirements/business/REQ-05.md
  - docs/requirements/business/REQ-05.003.md
  - docs/requirements/business/REQ-05.004.md
  - docs/requirements/business/REQ-05.005.md
  - docs/requirements/business/REQ-05.006.md
  - docs/requirements/business/REQ-05.007.md
  - docs/requirements/non-functional/NFR-02.002.md
  - docs/requirements/non-functional/NFR-04.001.md
  - docs/requirements/non-functional/NFR-04.002.md
  - docs/business-rules/BR-001-papyrus-command-behavior.md
ordinal: 17000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Contributors need the remaining documented record types—glossary terms, UI specifications, business rules, and audit records—in the same project-local documentation store.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Users can create, list, view, and edit glossary terms, UI specifications, business rules, and audit records with documented fields and type rules.
- [ ] #2 Document deprecation and restoration commands preserve documented identity, folder, and visibility behavior for each supported type.
- [ ] #3 All record data remains in project-local text files, input is sanitized, and no paths outside the project directory are accessed.
<!-- AC:END -->
