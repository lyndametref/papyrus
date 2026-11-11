---
id: TASK-99
title: Write the Release 1.0 upgrade notes for the one-time Backlog.md import
status: To Do
assignee: []
created_date: '2026-11-11T20:44:00Z'
labels:
  - documentation
  - import
  - release-1.0
milestone: m-14
dependencies:
  - TASK-83
  - TASK-92
references:
  - docs/architecture/adr/ADR-0010-papyrus-native-on-disk-record-layout.md
  - docs/architecture/adr/ADR-0011-one-way-backlog-md-import.md
  - docs/requirements/non-functional/NFR-04.004.md
priority: high
type: docs
ordinal: 39160
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
ADR-0010 decision 11 and ADR-0011 decision 9 make the upgrade notes a release gate: the release that switches to native record stems must document the one-time import, because a project still holding backlog/tasks/task-N and backlog/milestones/m-N records loses sight of them the moment the new version runs. Users need the exact command, the affected directories, the renumbering they will see, and what to do with the source store. This task writes that document and the release-level change summary.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Upgrade notes exist in the repository documentation and are referenced from the README.
- [ ] #2 They name the affected directories (backlog/tasks, backlog/milestones, backlog/archive/tasks, backlog/archive/milestones), the exact command to run, and the dry-run and report review step.
- [ ] #3 They state that task IDs and milestone numbers are reallocated with milestones moving to the MS- space, that the source store is kept until the import is verified, and that Papyrus owns the records afterwards.
- [ ] #4 They state that Backlog.md compatibility is renounced, that papyrus import backlog is the only bridge, and that re-running it duplicates records.
- [ ] #5 They list the other user-visible changes of the release: the record view layout, the browser configuration view, non-functional requirement grouping, the browser-only --debug flag, and the removed storage-ID JSON fields.
- [ ] #6 The documented command, options, and caveats match the shipped command help.
<!-- AC:END -->
