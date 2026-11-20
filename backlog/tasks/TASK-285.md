---
id: TASK-285
title: "Run the code audit on the refactored document engine"
created_date: '2026-11-20T17:44:13Z'
updated_date: '2026-11-20T17:44:13Z'
labels: []
references:
  - ADR-036
  - ADR-037
  - ADR-038
  - ADR-039
  - REQ-07.004
status: ToDo
assignee:
  - code-analyzer
milestone: MS-037
dependencies:
  - TASK-283
  - TASK-284
ordinal: 60
documentation: []
type: audit
---

## Description
Context: the refactor replaces nine modules and every document browser branch; a code audit is the last chance to catch behavior that was deleted rather than generalized.

Scope: audit the unified handler, the descriptor and seed layer, the snapshot and cache, the asset path, and the guards. Check for per-type special cases that survived as slug comparisons, for PatchOps bypasses, for unwrap and panic paths on config-controlled input, and for dead code left behind. Record findings as an audit record with severity; blocking findings create fix tasks through the planner, not direct edits.
