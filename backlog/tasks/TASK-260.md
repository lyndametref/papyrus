---
id: TASK-260
title: "Regenerate CLI help goldens and replay the equivalence fixtures"
created_date: '2026-11-20T17:42:17Z'
updated_date: '2026-11-20T17:42:17Z'
labels: []
references:
  - REQ-07.003
  - ADR-0016
  - TECH-024
status: ToDo
assignee:
  - rust-dev
milestone: MS-034
dependencies:
  - TASK-259
ordinal: 80
documentation: []
type: implementation
---

## Description
Context: help is derived now, so output will change in known ways; anything else is a regression.

Scope: regenerate the help goldens; produce a diff against the MS-032 snapshot and attribute every change to the uniform title contract, --term removal, derived field options, create-group, --group, supersede, or the audit positional title. Replay the MS-032 CLI fixtures: record files and exit codes match except recorded title and config differences; stdout differs only where the attributed changes explain it. Assert help output is deterministic across two runs.
