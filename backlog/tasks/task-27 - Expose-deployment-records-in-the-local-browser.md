---
id: TASK-27
title: Expose deployment records in the local browser
status: To Do
assignee: []
created_date: '2026-11-02T20:19:00Z'
labels:
  - browser
  - documentation
milestone: m-4
dependencies:
  - TASK-26
  - TASK-17
references:
  - README.md
  - docs/requirements/business/REQ-06.002.md
  - docs/requirements/non-functional/NFR-01.002.md
  - docs/requirements/non-functional/NFR-01.003.md
  - docs/requirements/non-functional/NFR-05.001.md
  - docs/business-rules/BR-004-browser-command-behavior.md
type: feature
ordinal: 19500
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
papyrus-config(5) declares a deploy document type (prefix DEP, directory architecture/deployment/), but papyrus(1) documents no deploy command. Per the planning directive, every document type in the configured inventory needs an investigated browser view, and NFR-01.002 requires the browser to display all Papyrus content.

Scope: investigate whether any command writes deployment records and how enabled deploy-type documents are discovered, then implement browser list, view, and edit for deploy-type documents using the generic document pipeline and the document views built for earlier types. Deprecated deploy documents follow the same deprecated/ subfolder visibility as other document types. This task covers the deploy document type only; cross-type navigation and the final completeness check belong to TASK-19.

Out of scope: adding a deploy CLI command (not documented in papyrus(1); requires a requirements change), unified search (TASK-18).

## Open Questions
- Deployment records have no documented CLI command. Should the browser expose enabled deploy-type documents at all, and if so with full edit support? Recommended default: expose enabled deploy-type documents with list, view, and edit like other document types, because NFR-01.002 requires all content to be displayed; if the user prefers to defer deployment records until a deploy command is specified, drop this task.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The browser lists deploy-type documents found in the configured deploy directory with their ID and title.
- [ ] #2 Selecting a deploy document displays its content and allows editing it; saved changes persist in project-local text files and are visible after reload.
- [ ] #3 Investigation notes record whether any command writes deploy records, how enabled/disabled document types affect browser visibility, and the resulting exposure decision.
- [ ] #4 The interface works without internet access, sanitizes browser input, and reads or writes no path outside the project directory except during init.
- [ ] #5 The interface renders in Firefox, Microsoft Edge, and Safari version 100 or later.
<!-- AC:END -->
