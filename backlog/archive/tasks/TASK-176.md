---
id: TASK-176
title: "Add integration tests for quoted assignee and comparable list fields in Backlog.md import"
created_date: '2026-11-17T11:00:05Z'
updated_date: '2026-11-17T18:47:29Z'
labels:
  - bug
references:
  - RE-001
  - NFR-004.003
  - REQ-005.010
  - REQ-005.011
  - NFR-004.004
status: Done
assignee:
  - rust-dev
milestone: MS-023
dependencies:
  - TASK-153
ordinal: 35
documentation:
  - tests/import_backlog_cli.rs
  - src/import.rs
type: implementation
---

## Description
Outcome: integration tests in tests import backlog cli rs verify at-prefixed assignee and comparable list fields survive import as valid YAML and round-trip; correction for AUD-010 finding 1 MS-023 AC-4.
