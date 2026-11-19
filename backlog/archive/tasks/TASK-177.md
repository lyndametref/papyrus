---
id: TASK-177
title: "Fix milestone archive rollback on failed source unlink"
created_date: '2026-11-17T21:17:34Z'
updated_date: '2026-11-19T10:53:20Z'
labels: []
references:
  - AUD-041
  - MS-025
  - TECH-009
  - ADR-032
  - TC-020
status: Done
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
## Comments
- **rust-dev**: Implementation test comment
- **rust-dev**: archive now rolls back destination on unlink failure
- **rust-dev**: archive now removes destination when fs.remove_file of source fails
- **rust-dev**: archive returns AppError Runtime exit 1
- **rust-dev**: doc comment describes write then unlink with rollback
- **rust-dev**: Implementation commit bfc0515
- **rust-dev**: AppError::Runtime exit 1
- **rust-dev**: write-then-unlink with rollback
- **rust-dev**: The doc comment describes write-then-unlink with rollback instead of an atomic move.
- **rust-dev**: milestone.rs
- **rust-dev**: cargo test --lib 426 passed
- **rust-dev**: milestone.rs archive function
- **rust-dev**: tests milestone_archive_cli.rs
- **rust-dev**: TASK-177 implementation commit bfc0515: milestone archive now removes the written destination when the active-source unlink fails, then returns AppError::Runtime (exit 1), leaving only the source copy. The doc comment describes write-then-unlink with rollback instead of an atomic move.
- **rust-dev**: Regression test archive_rolls_back_destination_when_source_unlink_fails in the milestone archive CLI suite makes the active milestones directory mode 0555, asserts exit 1, destination absent, source bytes intact, and a successful retry yielding one copy; PermissionGuard restores mode on drop and a write probe skips environments that do not enforce permissions.
- **rust-dev**: Checks: cargo fmt check pass; cargo clippy --all-targets shows only the pre-existing browser collapsible_if warning; cargo test --lib 426 passed; milestone archive CLI suite 10 passed; milestone CLI suite 4 passed. Full-suite pre-existing failures remain as documented in AUD-039 and AUD-041.
- **orchestrator**: Correction verified: rollback and regression test reviewed; milestone archive suites pass.
