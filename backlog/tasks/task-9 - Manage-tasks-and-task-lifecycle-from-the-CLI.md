---
id: TASK-9
title: Manage tasks and task lifecycle from the CLI
status: To Do
assignee: []
created_date: '2026-10-31T22:25:00Z'
labels: []
milestone: m-2
dependencies:
  - TASK-8
references:
  - docs/requirements/business/REQ-02.md
  - docs/requirements/business/REQ-02.001.md
  - docs/requirements/business/REQ-02.002.md
  - docs/requirements/business/REQ-02.003.md
  - docs/requirements/business/REQ-02.004.md
  - docs/requirements/non-functional/NFR-02.002.md
  - docs/requirements/non-functional/NFR-04.001.md
  - docs/requirements/non-functional/NFR-04.002.md
  - docs/business-rules/BR-001-papyrus-command-behavior.md
ordinal: 9000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Contributors need usable task records, filters, dependencies, milestone assignment, and lifecycle controls to organize and track project work.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Tasks can be created with documented fields and links to requirements, specifications, audit records, and business rules; task ID and initial status follow configuration.
- [ ] #2 Users can view and edit task fields, list and filter tasks, sort and limit results, and use watch mode as documented.
- [ ] #3 Task dependency, milestone assignment, archive, restore, delete, and confirmation behaviors match the documented contract.
- [ ] #4 Task data is stored in project-local text files, user input is sanitized, and Papyrus does not read or write outside the project directory except during init.
<!-- AC:END -->
