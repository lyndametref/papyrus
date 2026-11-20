---
id: TASK-283
title: "Extend the no-drift and descriptor-leak guards for the three-handler architecture"
created_date: '2026-11-20T17:44:01Z'
updated_date: '2026-11-20T17:44:01Z'
labels: []
references:
  - ADR-036
  - ADR-039
  - TECH-072
status: ToDo
assignee:
  - rust-dev
milestone: MS-037
dependencies:
  - TASK-282
ordinal: 40
documentation:
  - tests/no_drift.rs
  - tests/document_type_no_drift.rs
  - tests/no_descriptor_leaks.rs
type: implementation
---

## Description
Context: the existing guards catch some descriptor leaks but not the new invariants.

Scope: extend the guards so the build fails when a document TypedHandler variant exists, when a document slug is matched outside the seed (tests and docs exempt), when a per-type document module reappears, when a per-type conflict or not-found renderer reappears, or when config reads occur on a per-record path. Keep Backlog and Decision exemptions explicit. Guards must fail with a message naming the offending file and symbol.
