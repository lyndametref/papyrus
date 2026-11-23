---
id: TASK-076
title: Expose use cases, test cases, and test reports in the local browser
status: Done
assignee: []
created_date: '2026-11-10T20:16:00Z'
updated_date: '2026-11-11T08:37:00Z'
labels:
  - browser
  - documentation
milestone: MS-016
dependencies:
  - TASK-073
  - TASK-074
references:
  - docs/requirements/business/REQ-005.012.md
  - docs/requirements/business/REQ-005.013.md
  - docs/requirements/business/REQ-005.014.md
  - docs/requirements/business/REQ-005.007.md
  - docs/requirements/business/REQ-006.002.md
  - docs/business-rules/BR-018-use-case-document-type-behavior.md
  - docs/business-rules/BR-019-test-case-document-type-behavior.md
  - docs/business-rules/BR-020-test-report-document-type-behavior.md
priority: medium
type: feature
ordinal: 35300
---


## Description
Extend the local offline browser to expose use cases, test cases, and test reports: list/view/edit/deprecate/restore flows, front-matter metadata projection, search integration, and 320px markup, following the approved mockup package (TASK-79) and the shared record envelope (REQ-005.010/011, ADR-0007/0009).

## Acceptance Criteria
- [x] Browser lists, views, edits, deprecates, and restores use cases, test cases, and test reports with IDs preserved.
- [x] Front-matter metadata projection matches CLI views for all three types.
- [x] Deprecated records hidden; restored records reappear.
- [x] Assets embedded; no external references; 320px markup supported.
