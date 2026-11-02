---
id: TASK-10
title: Expand unified search to task, milestone, and release records
status: To Do
assignee: []
created_date: '2026-10-31T22:26:00Z'
updated_date: '2026-11-01T20:32:00Z'
labels: []
milestone: m-2
dependencies:
  - TASK-9
  - TASK-5
references:
  - docs/requirements/business/REQ-06.001.md
  - docs/requirements/business/REQ-02.001.md
  - docs/requirements/business/REQ-02.003.md
  - docs/requirements/business/REQ-03.001.md
  - docs/requirements/business/REQ-03.002.md
  - docs/business-rules/BR-015-search-command-behavior.md
ordinal: 11000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The initial search increment covers requirements; users need search to include the new backlog record types as they become available.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Search now returns task, milestone, and release records in addition to business and non-functional requirements.
- [ ] #2 Type filters and limits can select these records, and results honor documented visibility and archive behavior.
- [ ] #3 Searching for quoted phrases works across all supported record types.
<!-- AC:END -->
