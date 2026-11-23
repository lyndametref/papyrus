---
id: TASK-316
title: "Harden config containment, request logging, YAML budgets, and path checks from AUD-153"
created_date: '2026-11-23T11:29:22Z'
updated_date: '2026-11-23T11:45:17Z'
labels: []
references:
  - AUD-153
  - REL-001
  - NFR-04.002
  - REQ-07.002
  - TASK-286
status: Done
assignee:
  - rust-dev
milestone: MS-037
dependencies:
  - TASK-286
ordinal: 75
documentation:
  - src/browser/mod.rs
  - src/browser/http.rs
  - src/config.rs
  - src/record/frontmatter.rs
  - src/record/relocate.rs
  - src/task.rs
  - src/milestone.rs
type: implementation
---

## Description
## Context
AUD-153 (security audit of REL-001) passed with findings and recorded six lows. This task closes the four lows scoped here as one security-hardening change inside MS-037; M-1 and L-4 stay with TASK-288 and M-2 remains an architect decision. AUD-153 records the NFR-04.002 gap at L-6.

## Scope
- L-1: add table-driven unit tests for `validate_host` and `validate_origin` in `src/browser/mod.rs` and a foreign-Host integration test over the HTTP surface. Evidence: AUD-153 L-1.
- L-2: stop logging the raw method token in the `src/browser/http.rs` debug output; log the mapped method or strip non-printable characters with a length cap. Evidence: AUD-153 L-2.
- L-3: cap YAML input size and reject or budget anchors and aliases for the config and record front matter (`src/config.rs`, `src/record/frontmatter.rs`), with a regression test for a hostile document. Evidence: AUD-153 L-3.
- L-5: use `symlink_metadata` instead of `Path::exists` for the occupied-destination check in `src/record/relocate.rs` and the task move paths (`src/task.rs`, `src/milestone.rs`), with a dangling-symlink regression test. Evidence: AUD-153 L-5.
- L-6: canonicalize the config path and enforce project containment before the first load, with a test asserting no read of an unreadable outside target (satisfies NFR-04.002). Evidence: AUD-153 L-6.

## Out of scope
- No change for M-1, M-2, or L-4 (M-1 and L-4 belong to TASK-288; M-2 is an open architect decision).
- No unrelated refactor, no requirement, rule, ADR, specification, or test-case edit.

## Definition of done
Each scoped low is addressed together with its test, the task records the item-to-test mapping, and `cargo test`, `cargo clippy --all-targets -- -D warnings`, and `cargo fmt --check` are clean with no unrelated behaviour change.

## Acceptance Criteria
- [ ] L-1: validate_host and validate_origin have table-driven unit tests and a foreign-Host integration test asserting 400 for a foreign Host and 403 for a foreign Origin.
- [ ] L-2: debug output no longer prints the raw method token; it logs the mapped method or a printable-only, length-capped token, covered by a unit test.
- [ ] L-3: config and front-matter YAML input is size-capped and anchors and aliases are rejected or budgeted, with a regression test for a hostile document.
- [ ] L-5: the relocate module and the task move paths use symlink_metadata for the occupied-destination check, with a dangling-symlink regression test.
- [ ] L-6: the config path is canonicalized and checked for project containment before the first load, with a test proving no read of an unreadable outside target (NFR-04.002).
- [ ] Each item is mapped to its test in the task; cargo test, cargo clippy all-targets with denied warnings, and cargo fmt check are clean; no unrelated behaviour change.
## Comments
- **planner**: planner: planned from AUD-153 lows L-1, L-2, L-3, L-5, L-6; M-1 and L-4 stay in TASK-288; ordinal 75 keeps it after TASK-286 (audit) and before TASK-287 (quality gate), which now depends on this task.
- **rust-dev**: TASK-316 implemented and left In-Progress; no commit per orchestrator. L-1: table-driven validate_host and validate_origin unit tests in src browser mod.rs and a foreign-Host and foreign-Origin HTTP integration test asserting 400 and 403 in tests browser_cli.rs. L-2: the request log now calls log_method, which returns the mapped GET HEAD or POST token or a printable-ASCII 16-character sanitized token; unit test log_method_uses_the_mapped_or_sanitized_token. L-3: config and front-matter YAML are capped at 1048576 and 65536 bytes and anchors and aliases are rejected by a quote, comment, and block-scalar aware scanner in src config.rs, wired into commands read_raw_config and frontmatter parse; tests yaml_input_limits_reject_hostile_documents, hostile_front_matter_alias_bomb_is_rejected, and oversized_front_matter_is_rejected. L-5: relocate preflight and the task archive, task restore, and milestone archive occupied-destination checks use symlink_metadata; regression tests dangling_symlink_destination_counts_as_occupied and archive_destination_occupancy_sees_a_dangling_symlink. L-6: src lib.rs canonicalizes the config path and enforces project containment before the first load; tests cli_dispatch unreadable_outside_config_is_rejected_before_any_read uses an unreadable mode-000 outside target and proves the rejection happens before any read. ADR-044: the non-loopback warning now names unauthenticated exposure and record read and modify risk, t06 asserts the text for a non-loopback host and its absence for loopback, and the README browser section documents the exposure. Checks: env -u EDITOR -u VISUAL cargo test exit 0; EDITOR=nano cargo test exit 0; cargo clippy --all-targets -- -D warnings clean; cargo fmt --check clean; cargo build clean. No new crates; cargo-audit is not installed. Branch task TASK-316; nothing committed.
- **orchestrator**: Acceptance met: Host and Origin validators tested with a foreign-Host integration case, debug log sanitized, YAML size and alias budgets, symlink_metadata occupancy checks, config containment before first load with an unreadable-outside regression, and the ADR-044 non-loopback warning with README text; suite green with and without EDITOR.
