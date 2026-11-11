---
id: TASK-100
title: Import this repository own backlog store into the Papyrus-native layout once
status: To Do
assignee: []
created_date: '2026-11-11T20:45:00Z'
labels:
  - import
  - release-1.0
  - migration
milestone: m-14
dependencies:
  - TASK-98
  - TASK-99
references:
  - docs/architecture/adr/ADR-0011-one-way-backlog-md-import.md
  - docs/architecture/adr/ADR-0010-papyrus-native-on-disk-record-layout.md
  - docs/requirements/non-functional/NFR-04.004.md
priority: high
type: chore
ordinal: 39170
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
This repository still stores its own work as backlog/tasks/task-N and backlog/milestones/m-N, which the shipped native layout no longer recognises, so the upgrade documented in the release notes must actually be executed here once. The import reallocates every task ID and milestone number and rewrites references through the plan, so it has to run after the implementation and verification are Done and before the release gate closes, with the plan and report preserved as the only mapping record.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A dry run of papyrus import backlog against this repository reports every task and milestone, and the plan and possible duplicates are reviewed before any write.
- [ ] #2 The import runs once and the plan and report are preserved in the repository as the only record of the source-to-Papyrus mapping.
- [ ] #3 Every imported record is readable by the Papyrus CLI and the browser, and task, milestone, and dependency references resolve to the new IDs.
- [ ] #4 The pre-import store is preserved as a documented backup or tag until verification completes, as the upgrade notes require.
- [ ] #5 Post-import verification is recorded: record counts, unresolved references with their warnings, and the renumbered milestone list.
- [ ] #6 The source store stays in place until the release ticket approval is recorded, and its retirement is documented as the final upgrade step.
<!-- AC:END -->
