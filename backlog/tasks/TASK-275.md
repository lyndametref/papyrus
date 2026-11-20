---
id: TASK-275
title: "Build the inventory from one registry-wide scan"
created_date: '2026-11-20T17:43:20Z'
updated_date: '2026-11-20T17:43:20Z'
labels: []
references:
  - ADR-037
  - NFR-007.001
  - TECH-043
status: ToDo
assignee:
  - rust-dev
milestone: MS-036
dependencies:
  - TASK-274
ordinal: 50
documentation: []
type: implementation
---

## Description
Context: the inventory walks every type, every record, and rebuilds configuration and registry state along the way, which produced 14.2 s at 382 records.

Scope: build the inventory from a single registry-wide scan over already-loaded records. Render from the request snapshot with no per-record resolve, no directory rescan, and no config read. Keep archive filtering, merged group and member ordering, and reference visibility identical. Record the before and after numbers from the MS-032 harness in the task comment.
