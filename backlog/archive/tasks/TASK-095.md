---
id: TASK-095
title: Import this repository own backlog store into the Papyrus-native layout once
status: ToDo
assignee: []
created_date: '2026-11-11T20:45:00Z'
updated_date: '2026-11-11T20:53:00Z'
labels:
  - import
  - release-1.0
  - migration
dependencies:
  - TASK-093
  - TASK-094
references:
  - docs/architecture/adr/ADR-0011-one-way-backlog-md-import.md
  - docs/architecture/adr/ADR-0010-papyrus-native-on-disk-record-layout.md
  - docs/requirements/non-functional/NFR-004.004.md
priority: high
type: chore
ordinal: 39170
milestone: null
---


## Description
This repository still stores its own work as backlog/tasks/task-N and backlog/milestones/m-N, which the shipped native layout no longer recognises, so the upgrade documented in the release notes must actually be executed here once. The import reallocates every task ID and milestone number and rewrites references through the plan, so it has to run after the implementation and verification are Done and before the release gate closes, with the plan and report preserved as the only mapping record.

## Acceptance Criteria
- [ ] A dry run of papyrus import backlog against this repository reports every task and milestone, and the plan and possible duplicates are reviewed before any write.
- [ ] The import runs once and the plan and report are preserved in the repository as the only record of the source-to-Papyrus mapping.
- [ ] Every imported record is readable by the Papyrus CLI and the browser, and task, milestone, and dependency references resolve to the new IDs.
- [ ] The pre-import store is preserved as a documented backup or tag until verification completes, as the upgrade notes require.
- [ ] Post-import verification is recorded: record counts, unresolved references with their warnings, and the renumbered milestone list.
- [ ] The source store stays in place until the release ticket approval is recorded, and its retirement is documented as the final upgrade step.


## Notes
Deferred by user decision (2026-11-11): this repository own store is NOT imported in Release 1.0. The import of this repository own backlog is postponed until after the release; the release still ships papyrus import backlog and documents the one-time import in its upgrade notes (TASK-92, TASK-99). This task is archived and leaves milestone m-14, so it no longer blocks TASK-73. Restore it when the migration is scheduled, before this repository starts relying on the native layout for its own records.
