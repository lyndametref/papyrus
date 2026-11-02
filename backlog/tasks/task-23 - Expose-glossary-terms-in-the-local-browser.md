---
id: TASK-23
title: Expose glossary terms in the local browser
status: To Do
assignee: []
created_date: '2026-11-02T20:19:00Z'
labels:
  - browser
  - documentation
milestone: m-4
dependencies:
  - TASK-15
  - TASK-17
references:
  - README.md
  - docs/requirements/business/REQ-05.003.md
  - docs/requirements/business/REQ-05.007.md
  - docs/requirements/business/REQ-06.002.md
  - docs/requirements/non-functional/NFR-01.002.md
  - docs/requirements/non-functional/NFR-01.003.md
  - docs/requirements/non-functional/NFR-05.001.md
  - docs/business-rules/BR-004-browser-command-behavior.md
  - docs/business-rules/BR-011-glossary-command-behavior.md
type: feature
ordinal: 17500
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Glossary term management becomes available through the CLI in the same increment (TASK-17). Per the planning directive, the browser view for a new document type ships with its command increment, so glossary terms must be usable in the local browser as soon as the glossary command exists.

Scope: investigate how glossary records are stored and currently rendered, then implement list, view, and edit for glossary terms (term and definition) in the existing local browser UI, reusing the document views built for requirements and decisions where possible. Deprecated terms stay hidden from the web UI as documented in REQ-05.007. This task covers the glossary document type only; cross-type navigation and the final completeness check belong to TASK-19.

Out of scope: CLI command behavior (TASK-17), unified search (TASK-18).

## Open Questions
- None. The documented glossary fields and deprecation visibility are sufficient; no product decision is missing.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The browser lists glossary terms with their ID and term, and excludes deprecated terms from browser lists per the documented visibility rule.
- [ ] #2 Selecting a glossary term displays its ID, term, and definition and allows editing term and definition; saved changes persist in project-local text files and are visible after reload.
- [ ] #3 Investigation notes record where glossary terms appear in browser navigation, the list/detail/edit route shape, and which existing browser components are reused.
- [ ] #4 The interface works without internet access, sanitizes browser input, and reads or writes no path outside the project directory except during init.
- [ ] #5 The interface renders in Firefox, Microsoft Edge, and Safari version 100 or later.
<!-- AC:END -->
