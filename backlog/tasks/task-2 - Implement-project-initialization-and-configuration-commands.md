---
id: TASK-2
title: Implement project initialization and configuration commands
status: To Do
assignee: []
created_date: '2026-10-31T22:25:00Z'
updated_date: '2026-11-01T20:32:00Z'
labels: []
milestone: m-0
dependencies:
  - TASK-1
references:
  - docs/requirements/business/REQ-01.md
  - docs/requirements/business/REQ-01.001.md
  - docs/requirements/business/REQ-01.002.md
  - docs/requirements/business/REQ-01.003.md
  - docs/requirements/business/REQ-01.004.md
  - docs/requirements/business/REQ-01.005.md
  - docs/requirements/non-functional/NFR-02.001.md
  - docs/requirements/non-functional/NFR-02.002.md
  - docs/requirements/non-functional/NFR-04.001.md
  - docs/requirements/non-functional/NFR-04.002.md
  - docs/business-rules/BR-002-init-command-behavior.md
  - docs/business-rules/BR-003-config-command-behavior.md
  - docs/business-rules/BR-016-configuration-keys-and-precedence.md
ordinal: 2000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Users need a predictable project context and common CLI behavior before managing project records. Initialization is the sole documented exception to Papyrus's project-directory access restriction.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The documented init options create and configure the selected project as specified, including --no-git and safe --force behavior.
- [ ] #2 Project/config selection honors CLI options, environment variables, project configuration, and defaults in the documented precedence order.
- [ ] #3 Config list/get/set/unset, dotted keys, default task statuses, global options, JSON/color behavior, help, and exit outcomes match the documented contract.
- [ ] #4 Outside init, Papyrus reads and writes only within the selected project directory; saved configuration and other persisted data use text files only.
- [ ] #5 User-controlled command input is sanitized, and security is explicitly considered in implementation/review.
<!-- AC:END -->
