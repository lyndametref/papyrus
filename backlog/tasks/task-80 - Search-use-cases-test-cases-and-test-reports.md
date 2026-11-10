---
id: TASK-80
title: Search use cases, test cases, and test reports
status: To Do
assignee: []
created_date: '2026-11-10T20:16:00Z'
labels:
  - search
  - documentation
milestone: m-15
dependencies:
  - TASK-78
references:
  - docs/requirements/business/REQ-05.012.md
  - docs/requirements/business/REQ-05.013.md
  - docs/requirements/business/REQ-05.014.md
  - docs/requirements/business/REQ-05.007.md
  - docs/requirements/business/REQ-06.001.md
  - docs/business-rules/BR-018-use-case-document-type-behavior.md
  - docs/business-rules/BR-019-test-case-document-type-behavior.md
  - docs/business-rules/BR-020-test-report-document-type-behavior.md
priority: medium
type: task
ordinal: 35100
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Extend unified search to include use cases, test cases, and test reports. Records are stored under their configured document_types directories (proposed use-cases/, test-cases/, test-reports/). Active records matching <prefix>-<digits>.md are searched; deprecated/ subfolders and non-record files are never scanned. A disabled or absent type contributes no results and no error. Deprecated records hidden; restored reappear. Matching, ordering, output shapes, exit codes, boundary enforcement, and read-only guarantee reused from search-requirements.md; type filter value remains doc.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Unified search returns active use-case, test-case, and test-report records.
- [ ] #2 Deprecated records of all three types are hidden; restored records reappear.
- [ ] #3 Disabled or absent types contribute no results and no error.
- [ ] #4 Type filter value remains doc; no dedicated per-type filter added.
<!-- AC:END -->
