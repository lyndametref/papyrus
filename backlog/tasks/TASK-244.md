---
id: TASK-244
title: "Assess architecture impact of the owned descriptor and field-schema model"
created_date: '2026-11-20T17:41:19Z'
updated_date: '2026-11-20T17:41:19Z'
labels: []
references:
  - ADR-036
  - ADR-0019
  - TECH-070
  - TECH-071
  - TECH-077
status: ToDo
assignee:
  - architect
milestone: MS-033
dependencies:
  - TASK-243
ordinal: 10
documentation: []
type: architecture
---

## Description
Confirm that removing the 'static descriptor assumption and adding FieldSpec plus DocumentModel needs no decision beyond ADR-036 and the ADR-0019 amendments. Decide where the seed table lives, what owns derived verbs, and how the validation pass reports multiple diagnostics. Record deviations as amendment requests to the planner rather than inventing structure. Do not start until MS-031 is accepted.
