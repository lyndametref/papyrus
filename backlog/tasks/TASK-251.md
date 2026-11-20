---
id: TASK-251
title: "Prove old configurations load identically and cover the schema with unit tests"
created_date: '2026-11-20T17:41:37Z'
updated_date: '2026-11-20T17:41:37Z'
labels: []
references:
  - REQ-07.002
  - REQ-07.004
  - TECH-077
  - ADR-036
status: ToDo
assignee:
  - rust-dev
milestone: MS-033
dependencies:
  - TASK-250
ordinal: 80
documentation: []
type: implementation
---

## Description
Context: MS-033 is only complete when existing projects load without edits and the new rules are regression-protected.

Scope: load the current papyrus.config.yml and the MS-032 fixture configs and assert the effective descriptor set is unchanged. Add unit tests for default-fill, collection replace, absent slug, prefix and directory never filled, unknown key and enum diagnostics, reserved field keys, duplicate field keys, fields: [], quoted heading round-trip, mapping-sequence emission, seed-only models, and immutable group membership. Assert no read path builds a config or registry per record.
