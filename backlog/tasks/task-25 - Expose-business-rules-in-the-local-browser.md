---
id: TASK-25
title: Expose business rules in the local browser
status: To Do
assignee: []
created_date: '2026-11-02T20:19:00Z'
labels:
  - browser
  - documentation
milestone: m-4
dependencies:
  - TASK-24
  - TASK-17
references:
  - README.md
  - docs/requirements/business/REQ-05.005.md
  - docs/requirements/business/REQ-05.007.md
  - docs/requirements/business/REQ-06.002.md
  - docs/requirements/non-functional/NFR-01.002.md
  - docs/requirements/non-functional/NFR-01.003.md
  - docs/requirements/non-functional/NFR-05.001.md
  - docs/business-rules/BR-004-browser-command-behavior.md
  - docs/business-rules/BR-013-rule-command-behavior.md
type: feature
ordinal: 18500
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Business rule management becomes available through the CLI in the same increment (TASK-17). Per the planning directive, the browser view for a new document type ships with its command increment, so business rules must be usable in the local browser as soon as the rule command exists.

Scope: investigate how business rule records are stored and currently rendered, then implement list, view, and edit for business rules (title, description, rationale, applicability, and referenced business requirements) in the existing local browser UI, reusing the document views built for earlier document types. Deprecated rules stay hidden from the web UI as documented in REQ-05.007. This task covers the rule document type only; cross-type navigation and the final completeness check belong to TASK-19.

Out of scope: CLI command behavior (TASK-17), unified search (TASK-18).

## Open Questions
- Should requirement references in a rule view navigate to the corresponding requirement browser view? Recommended default: yes, render references as links to existing requirement views while preserving the documented add/remove reference semantics on edit.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The browser lists business rules with their ID and title, and excludes deprecated rules from browser lists per the documented visibility rule.
- [ ] #2 Selecting a business rule displays its title, description, rationale, applicability, and referenced requirement IDs and allows editing those fields; saved changes persist in project-local text files and are visible after reload.
- [ ] #3 Requirement references are displayed and editable with the same add and remove semantics as the rule command.
- [ ] #4 Investigation notes record where business rules appear in browser navigation, the list/detail/edit route shape, and which existing browser components are reused.
- [ ] #5 The interface works without internet access, sanitizes browser input, and reads or writes no path outside the project directory except during init.
- [ ] #6 The interface renders in Firefox, Microsoft Edge, and Safari version 100 or later.
<!-- AC:END -->
