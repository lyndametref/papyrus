---
id: TASK-5
title: Search business and non-functional requirements
status: To Do
assignee: []
created_date: '2026-10-31T22:25:00Z'
updated_date: '2026-11-01T20:32:00Z'
labels: []
milestone: m-1
dependencies:
  - TASK-4
references:
  - docs/requirements/business/REQ-06.md
  - docs/requirements/business/REQ-06.001.md
  - docs/requirements/business/REQ-05.001.md
  - docs/requirements/business/REQ-05.002.md
  - docs/requirements/business/REQ-05.007.md
  - docs/business-rules/BR-015-search-command-behavior.md
ordinal: 5000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Users need to find requirements across the records supported so far before broader search can expand to future record types.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The search command returns matching business and non-functional requirement records from project text-file stores.
- [ ] #2 Quoted phrases, repeatable type filters, and result limits behave as documented for the currently supported records.
- [ ] #3 Search results are consistent with deprecation visibility rules for requirements.
<!-- AC:END -->
