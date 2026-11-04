---
id: TASK-26
title: Expose audit records in the local browser
status: To Do
assignee: []
created_date: '2026-11-02T20:19:00Z'
updated_date: '2026-11-04T19:40:00Z'
labels:
  - browser
  - documentation
milestone: m-9
dependencies:
  - TASK-47
  - TASK-49
references:
  - README.md
  - docs/requirements/business/REQ-05.006.md
  - docs/requirements/business/REQ-05.007.md
  - docs/requirements/business/REQ-06.002.md
  - docs/requirements/non-functional/NFR-01.002.md
  - docs/requirements/non-functional/NFR-01.003.md
  - docs/requirements/non-functional/NFR-05.001.md
  - docs/business-rules/BR-004-browser-command-behavior.md
  - docs/business-rules/BR-014-audit-command-behavior.md
type: feature
ordinal: 27300
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Audit record management becomes available through the CLI in the same increment (TASK-17). Per the planning directive, the browser view for a new document type ships with its command increment, so audit records must be usable in the local browser as soon as the audit command exists.

Scope: investigate how audit records are stored and currently rendered, then implement list, view, and edit for audit records (title, content, audit type, including built-in security/licence/architecture and configured types) in the existing local browser UI, reusing the document views built for earlier document types. This task covers the audit document type only; cross-type navigation and the final completeness check belong to TASK-19.

Out of scope: CLI command behavior (TASK-17), unified search (TASK-18).

## Open Questions
- Should deprecated audit records be hidden from the web UI? REQ-05.007 records that the man page does not state the visibility effect for audit records, unlike the other document types. Recommended default: hide deprecated audit records from browser lists, consistent with the documented Documents lifecycle and the other document types.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The browser lists audit records with their ID, title, and audit type, including configured non-built-in types.
- [ ] #2 Selecting an audit record displays its title, content, and audit type and allows editing those fields; saved changes persist in project-local text files and are visible after reload.
- [ ] #3 Deprecated audit records follow the resolved visibility decision, which is recorded in the task notes.
- [ ] #4 Investigation notes record where audit records appear in browser navigation, the list/detail/edit route shape, and which existing browser components are reused.
- [ ] #5 The interface works without internet access, sanitizes browser input, and reads or writes no path outside the project directory except during init.
- [ ] #6 The interface renders in Firefox, Microsoft Edge, and Safari version 100 or later.
<!-- AC:END -->
