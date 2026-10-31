---
id: TASK-16
title: Package and verify the decision-record milestone
status: To Do
assignee: []
created_date: '2026-10-31T22:26:00Z'
labels: []
milestone: m-3
dependencies:
  - TASK-15
references:
  - docs/requirements/business/REQ-04.md
  - docs/requirements/business/REQ-06.001.md
  - docs/requirements/business/REQ-06.002.md
  - docs/requirements/non-functional/NFR-03.002.md
  - docs/requirements/non-functional/NFR-06.001.md
  - docs/requirements/non-functional/NFR-06.002.md
  - docs/business-rules/BR-001-papyrus-command-behavior.md
ordinal: 16000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Decision management, search, and browser changes must be available together in an installable milestone release.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 An amd64 Debian package installs on Debian Trixie and Forky and includes decision creation, view/edit, search, and browser workflows.
- [ ] #2 Package smoke tests verify decision lifecycle operations and decision visibility in search and the offline browser.
- [ ] #3 The installed application retains the project-local text-file policy, supported browser targets, GPL-3.0-or-later license, and dependency compatibility.
<!-- AC:END -->
