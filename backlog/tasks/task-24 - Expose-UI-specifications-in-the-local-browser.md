---
id: TASK-24
title: Expose UI specifications in the local browser
status: To Do
assignee: []
created_date: '2026-11-02T20:19:00Z'
labels:
  - browser
  - documentation
milestone: m-4
dependencies:
  - TASK-23
  - TASK-17
references:
  - README.md
  - docs/requirements/business/REQ-05.004.md
  - docs/requirements/business/REQ-05.007.md
  - docs/requirements/business/REQ-06.002.md
  - docs/requirements/non-functional/NFR-01.002.md
  - docs/requirements/non-functional/NFR-01.003.md
  - docs/requirements/non-functional/NFR-05.001.md
  - docs/business-rules/BR-004-browser-command-behavior.md
  - docs/business-rules/BR-012-ui-spec-command-behavior.md
type: feature
ordinal: 18000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
UI specification management becomes available through the CLI in the same increment (TASK-17). Per the planning directive, the browser view for a new document type ships with its command increment, so UI specifications must be usable in the local browser as soon as the ui-spec command exists.

Scope: investigate how UI specification records are stored and currently rendered, then implement list, view, and edit for UI specifications (title, associated view, description) in the existing local browser UI, reusing the document views built for earlier document types. Deprecated specifications stay hidden from the web UI as documented in REQ-05.007. This task covers the ui-spec document type only; cross-type navigation and the final completeness check belong to TASK-19.

Out of scope: CLI command behavior (TASK-17), unified search (TASK-18), mockup authoring or rendering.

## Open Questions
- Should mockup files (SVG/HTML) linked from a UI specification be rendered in the browser? Recommended default: no. This task displays and edits the documented UI specification fields only; mockup rendering needs a separate product decision and task.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The browser lists UI specifications with their ID and title, and excludes deprecated specifications from browser lists per the documented visibility rule.
- [ ] #2 Selecting a UI specification displays its title, associated view, and description and allows editing those fields; saved changes persist in project-local text files and are visible after reload.
- [ ] #3 Investigation notes record where UI specifications appear in browser navigation, the list/detail/edit route shape, and which existing browser components are reused.
- [ ] #4 The interface works without internet access, sanitizes browser input, and reads or writes no path outside the project directory except during init.
- [ ] #5 The interface renders in Firefox, Microsoft Edge, and Safari version 100 or later.
<!-- AC:END -->
