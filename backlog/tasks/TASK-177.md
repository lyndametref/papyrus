---
id: TASK-177
title: "Fix milestone archive rollback on failed source unlink"
created_date: '2026-11-17T21:17:34Z'
updated_date: '2026-11-17T21:17:34Z'
labels: []
references:
  - AUD-041
  - MS-025
  - TECH-009
  - ADR-032
  - TC-020
status: ToDo
assignee:
  - rust-dev
milestone: MS-025
dependencies:
  - TASK-162
ordinal: 45
documentation:
  - src/milestone.rs
type: implementation
---

## Description
Correction for code-audit finding AUD-041 (medium finding 1) in MS-025. Repair scope: archive() in the Rust milestone module (see documentation path) writes the archived destination before removing the active-folder source with fs::remove_file. Failure: if the source unlink fails, both copies remain, so the same milestone ID exists twice; a retry or delete --force returns exit 4 on the duplicate-ID conflict, and list skips the record. The doc comment currently claims an atomic move. Scope note: delivers within MS-025 approved scope.

## Acceptance Criteria
- [ ] If the source unlink fails after the destination write, roll back by removing the written destination so that only the source remains.
- [ ] Return exit 1 on that failure.
- [ ] Add a regression test that simulates the failed unlink and asserts that no duplicate remains.
- [ ] Update the doc comment so it no longer claims an atomic move.
- [ ] Existing milestone archive behavior and tests stay green.
