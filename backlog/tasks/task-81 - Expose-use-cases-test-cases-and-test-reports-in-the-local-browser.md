---
id: TASK-81
title: Expose use cases, test cases, and test reports in the local browser
status: Done
assignee: []
created_date: '2026-11-10T20:16:00Z'
updated_date: '2026-11-11T08:37:00Z'
labels:
  - browser
  - documentation
milestone: m-15
dependencies:
  - TASK-78
  - TASK-79
references:
  - docs/requirements/business/REQ-05.012.md
  - docs/requirements/business/REQ-05.013.md
  - docs/requirements/business/REQ-05.014.md
  - docs/requirements/business/REQ-05.007.md
  - docs/requirements/business/REQ-06.002.md
  - docs/business-rules/BR-018-use-case-document-type-behavior.md
  - docs/business-rules/BR-019-test-case-document-type-behavior.md
  - docs/business-rules/BR-020-test-report-document-type-behavior.md
priority: medium
type: feature
ordinal: 35300
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Extend the local offline browser to expose use cases, test cases, and test reports: list/view/edit/deprecate/restore flows, front-matter metadata projection, search integration, and 320px markup, following the approved mockup package (TASK-79) and the shared record envelope (REQ-05.010/011, ADR-0007/0009).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Browser lists, views, edits, deprecates, and restores use cases, test cases, and test reports with IDs preserved.
- [x] #2 Front-matter metadata projection matches CLI views for all three types.
- [x] #3 Deprecated records hidden; restored records reappear.
- [x] #4 Assets embedded; no external references; 320px markup supported.
<!-- AC:END -->
