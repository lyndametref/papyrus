---
id: TASK-73
title: 'Release 1.0: verify, audit, and close the release'
status: To Do
assignee: []
created_date: '2026-11-08T15:06:00Z'
labels:
  - release
milestone: m-14
dependencies:
  - TASK-20
references:
  - docs/requirements/business/REQ-05.010.md
  - docs/requirements/business/REQ-05.011.md
  - docs/requirements/business/REQ-06.003.md
  - docs/requirements/business/REQ-06.004.md
  - docs/requirements/non-functional/NFR-04.003.md
  - >-
    docs/architecture/adr/ADR-0007-yaml-front-matter-as-single-record-metadata-source.md
  - >-
    docs/architecture/adr/ADR-0008-adopt-backlog-md-on-disk-layout-for-tasks-and-milestones.md
  - >-
    docs/architecture/adr/ADR-0009-shared-record-envelope-and-unified-write-path.md
  - docs/specifications/release-lifecycle.md
priority: high
type: task
ordinal: 40000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Close the first complete Papyrus release after the final cross-type integration milestone is Done. Run release-level build and test checks, delegate the mandatory end-of-release audit to security-officer, compile the release report, and record the audit summary and explicit user approval here.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Every m-12 task, including TASK-20, is Done.
- [ ] #2 The Debian package builds and installs on the supported targets and the full test suite passes.
- [ ] #3 The security-officer release audit is complete; no Critical or High finding is unresolved and remaining findings have documented owners.
- [ ] #4 This ticket records the audit summary, the release version, and explicit user approval.
- [ ] #5 The release report is compiled.
<!-- AC:END -->
