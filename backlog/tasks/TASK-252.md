---
id: TASK-252
title: "Verify MS-033 model foundation: identical old-config load and validated schema"
created_date: '2026-11-20T17:41:41Z'
updated_date: '2026-11-20T17:41:41Z'
labels: []
references:
  - REQ-07.002
  - REQ-07.004
  - TECH-077
  - ADR-036
status: ToDo
assignee:
  - quality-gatekeeper
milestone: MS-033
dependencies:
  - TASK-251
ordinal: 90
documentation: []
type: milestone-verification
---

## Description
Milestone boundary verification for MS-033.

Scope: check every MS-033 acceptance criterion. Replay the MS-032 config fixtures and diff the effective descriptor set; run the schema unit tests; confirm each diagnostic uses document_types.<slug>.<path>; confirm the emitter round-trip is byte-stable; confirm no read path builds config or registry per record; confirm no CLI or browser behavior changed and the full suite is green.
