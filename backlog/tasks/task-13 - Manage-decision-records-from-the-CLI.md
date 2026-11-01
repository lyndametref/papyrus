---
id: TASK-13
title: Manage decision records from the CLI
status: To Do
assignee: []
created_date: '2026-10-31T22:26:00Z'
updated_date: '2026-11-01T20:32:00Z'
labels: []
milestone: m-3
dependencies:
  - TASK-12
references:
  - docs/requirements/business/REQ-04.md
  - docs/requirements/business/REQ-04.001.md
  - docs/requirements/business/REQ-04.002.md
  - docs/requirements/non-functional/NFR-02.002.md
  - docs/requirements/non-functional/NFR-04.001.md
  - docs/requirements/non-functional/NFR-04.002.md
  - docs/business-rules/BR-008-decision-command-behavior.md
ordinal: 13000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Decision-makers need to record, inspect, update, supersede, and archive decisions so project rationale and status are available alongside other Papyrus content.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Decisions can be created, listed, viewed, and edited with documented Nygard fields and decision types.
- [ ] #2 Decision status values and allowed transitions, filtering, and requirement references match the documented behavior.
- [ ] #3 Supersession and archival rules match the documented commands and constraints.
- [ ] #4 Decision data is stored in project-local text files, user input is sanitized, and no paths outside the project directory are accessed.
<!-- AC:END -->
