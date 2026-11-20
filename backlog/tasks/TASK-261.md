---
id: TASK-261
title: "Verify MS-034 CLI engine: command equivalence and help attribution"
created_date: '2026-11-20T17:42:17Z'
updated_date: '2026-11-20T17:42:17Z'
labels: []
references:
  - REQ-07.003
  - REQ-05.001
  - ADR-0016
status: ToDo
assignee:
  - quality-gatekeeper
milestone: MS-034
dependencies:
  - TASK-260
ordinal: 90
documentation: []
type: milestone-verification
---

## Description
Milestone boundary verification for MS-034.

Scope: check every MS-034 acceptance criterion. Confirm the baseline verb matrix per DocumentModel; confirm create-group, --group, and supersede appear only where allowed; confirm the title contract on create, create-group, edit, and the HTML metadata; confirm every help diff is attributed; confirm the nine modules and ten ops builders are gone and ui_mockup remains; confirm the fixture replay passes with documented deltas; confirm the full suite is green.
