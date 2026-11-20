---
id: TASK-246
title: "Replace 'static descriptors with owned values derived from the seed and configuration"
created_date: '2026-11-20T17:41:19Z'
updated_date: '2026-11-20T17:41:19Z'
labels: []
references:
  - ADR-036
  - ADR-037
  - TECH-071
  - TECH-072
status: ToDo
assignee:
  - rust-dev
milestone: MS-033
dependencies:
  - TASK-245
ordinal: 30
documentation: []
type: implementation
---

## Description
Context: ArgSpec, VerbSpec, RecordDescriptor, and TypeDescriptor assume 'static data, which blocks configuration-derived fields.

Scope: change these types to own their data and be reconstructible from a snapshot. Derive verbs from the DocumentModel instead of storing a hand-written list. Keep the public behavior of Backlog and Decision descriptors unchanged. Confirm no process-global cache is reintroduced; the snapshot from ADR-037 is the only cache. Do not start until MS-031 governance is accepted.
