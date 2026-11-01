---
id: TASK-12
title: Package and verify the backlog-discovery milestone
status: To Do
assignee: []
created_date: '2026-10-31T22:26:00Z'
updated_date: '2026-11-01T20:32:00Z'
labels: []
milestone: m-2
dependencies:
  - TASK-11
references:
  - docs/requirements/non-functional/NFR-03.002.md
  - docs/requirements/non-functional/NFR-04.001.md
  - docs/requirements/non-functional/NFR-05.001.md
  - docs/requirements/non-functional/NFR-06.001.md
  - docs/requirements/non-functional/NFR-06.002.md
  - docs/business-rules/BR-005-task-command-behavior.md
  - docs/business-rules/BR-006-milestone-command-behavior.md
  - docs/business-rules/BR-007-release-command-behavior.md
  - docs/business-rules/BR-004-browser-command-behavior.md
ordinal: 12000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Task, milestone, release, search, and browser features must remain usable from the installed Debian package at the end of this milestone.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 An amd64 Debian package installs on Debian Trixie and Forky and provides the implemented task, milestone, release, search, and browser features.
- [ ] #2 Installation smoke tests verify creating and viewing tasks/milestones/releases, searching them, and viewing/editing them in the offline browser.
- [ ] #3 The package preserves project-local text-file persistence, the declared license/dependency policy, and the supported browser behavior.
<!-- AC:END -->
