---
id: TASK-96
title: Make the browser console quiet by default and add debug request logging
status: To Do
assignee: []
created_date: '2026-11-11T20:44:00Z'
labels:
  - browser
  - release-1.0
milestone: m-14
dependencies:
  - TASK-86
references:
  - docs/requirements/business/REQ-06.007.md
  - docs/requirements/non-functional/NFR-02.001.md
priority: high
type: feature
ordinal: 39130
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The browser command is very verbose and prints request information whether or not the user asked for it, which also risks echoing record content into logs. REQ-06.007 requires error-only output by default and a browser-only --debug flag that logs one line per request without headers or bodies. This task implements that contract.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Without --debug the browser command prints only errors.
- [ ] #2 --debug is accepted by the browser command only; no global option and no other command accepts it.
- [ ] #3 Each logged request is one line with method, path, response status, and duration, and request headers and bodies are never logged.
- [ ] #4 Errors are printed whether or not --debug is set.
- [ ] #5 Tests cover the default and debug output shapes and rejection of --debug on another command.
<!-- AC:END -->
