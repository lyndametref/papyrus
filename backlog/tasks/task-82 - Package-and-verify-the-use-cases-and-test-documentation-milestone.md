---
id: TASK-82
title: Package and verify the use cases and test documentation milestone
status: To Do
assignee: []
created_date: '2026-11-10T20:16:00Z'
labels:
  - packaging
milestone: m-15
dependencies:
  - TASK-78
  - TASK-80
  - TASK-81
references:
  - docs/requirements/business/REQ-05.012.md
  - docs/requirements/business/REQ-05.013.md
  - docs/requirements/business/REQ-05.014.md
  - docs/requirements/business/REQ-05.007.md
  - docs/requirements/business/REQ-06.001.md
  - docs/requirements/business/REQ-06.002.md
  - docs/business-rules/BR-018-use-case-document-type-behavior.md
  - docs/business-rules/BR-019-test-case-document-type-behavior.md
  - docs/business-rules/BR-020-test-report-document-type-behavior.md
priority: medium
type: chore
ordinal: 35400
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Package and verify the m-15 milestone in an installable increment: rebuild amd64 Debian package, verify metadata and licensing, install/uninstall in isolated Debian Forky environment, run smoke tests proving the packaged binary can manage use cases, test cases, and test reports (create/list/view/edit/deprecate/restore with IDs preserved), search them, and serve the offline local browser categories end to end. Add milestone-level integration test and update docs/packaging.md.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Packaged binary manages use cases, test cases, and test reports end to end (create/list/view/edit/deprecate/restore, IDs preserved).
- [ ] #2 Packaged binary searches all three types; deprecated records hidden.
- [ ] #3 Offline browser serves all three categories with embedded assets.
- [ ] #4 Milestone-level integration test added; docs/packaging.md updated.
<!-- AC:END -->
