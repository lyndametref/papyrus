---
id: TASK-282
title: "Remove the dead descriptor, sanitizer, and per-type test artifacts"
created_date: '2026-11-20T17:43:51Z'
updated_date: '2026-11-23T06:40:17Z'
labels: []
references:
  - ADR-036
  - TECH-072
  - TECH-066
status: Done
assignee:
  - rust-dev
milestone: MS-037
dependencies:
  - TASK-281
ordinal: 30
documentation:
  - src/record/descriptor.rs
  - src/record/patch.rs
  - src/record/store.rs
  - src/record/document.rs
  - src/record/frontmatter.rs
  - src/decision.rs
  - src/record/identity.rs
  - src/sanitize.rs
  - src/browser/records.rs
  - src/browser/mod.rs
  - src/browser/render.rs
  - tests/browser_config_types_cli.rs
type: implementation
---

## Description
## Context
The descriptor-driven refactor (ADR-036, ADR-039, TECH-072) leaves dead helpers behind. Keeping them hides regressions and inflates the binary. TASK-281 refined this task against the current tree on branch `task/TASK-281`.

## Scope
Remove each symbol only after confirming no live production or test caller, and record the replacement path. Verified state on the refined tree:

Present and unused (remove):
- `src/record/descriptor.rs`: `all_descriptors()` (line ~346), `list_descriptors()` (~350), `is_managed_record()` (~364). Keep `descriptors()` and `descriptor_for()`.
- `src/record/patch.rs`: `apply_patches()` (~290). Keep `apply_patch()` and `apply()`.
- `src/record/store.rs`: `read_store()` (~1033). Keep `read_document()` and `write_atomic()`.
- `src/record/document.rs`: `to_text()` (~183).
- `src/record/frontmatter.rs`: `build_raw()` (~550), `format_scalar_pub()` (~718).
- `src/decision.rs`: `canonical_key()` (~173); no caller exists, including Decision handling.
- `src/record/identity.rs`: `matches_family_id()` (~483) and `IdentitySpec::validate_input()` (~44).
- `src/sanitize.rs`: `sanitize_reference()` (~280), `sanitize_documentation()` (~285), `sanitize_path_value()` (~290). Keep `sanitize_path_string()` and `src/document.rs::sanitize_references()`.

Present but live (replace, then remove):
- `src/browser/records.rs::Record::is_html_mockup()` (~89) has five live callers: `src/browser/mod.rs:1970`, `:2003`, `:2141`, and `src/browser/render.rs:2542`, `:2893`. Replace each with `record.descriptor.document_model == DocumentModel::SingleHtmlMockup` and then delete the helper. `src/browser/mod.rs` already uses that direct form at `:2913` and `:3744`.

Already absent (confirm, do not re-add):
- `edit_page_test_case`, `edit_page_test_report`, `conflict_page_test_case`, `conflict_page_test_report` are not defined in `src/`. The last two appear only as negative strings in `tests/browser_config_types_cli.rs:705-728`, which must keep passing unchanged.

## Out of scope
- Do not remove `descriptor_for`, `descriptors`, `apply_patch`, `apply`, `read_document`, `write_atomic`, `sanitize_path_string`, or `sanitize_references` (plural).
- Do not change Backlog (`Task`/`Milestone`/`Release`) or Decision behaviour, record bytes, JSON, status codes, routes, or the plan.
- Do not edit requirements, rules, ADRs, specifications, mockups, test cases, dependencies, ordinals, or milestone scope.

## Assumptions
- The five `is_html_mockup` call sites are the only inlining change this task allows; no other caller appears.
- `src/init.rs` slug enumeration and the seed table are not part of this task (TASK-283 owns the slug-dispatch guard).

## Definition of done
Every listed symbol is either removed with no remaining reference or confirmed already absent; `cargo build --release`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`, and `cargo test` are green; a task comment records each symbol as removed or already absent, with the replacement path for `is_html_mockup`.
## Acceptance Criteria
- [ ] `all_descriptors`, `list_descriptors`, and `is_managed_record` are removed from the record descriptor module; no reference remains in source or tests, and `descriptors` and `descriptor_for` remain.
- [ ] `apply_patches` is removed from the record patch module; `apply_patch` and `apply` remain and the write path is unchanged.
- [ ] `read_store` is removed from the record store module; `read_document` and `write_atomic` remain.
- [ ] `to_text` is removed from the record document module; no reference remains.
- [ ] `build_raw` and `format_scalar_pub` are removed from the record frontmatter module; no reference remains.
- [ ] `canonical_key` is removed from `crate::decision`; a grep returns only the unrelated test name in the task module.
- [ ] `matches_family_id` and `IdentitySpec::validate_input` are removed from the record identity module; no reference remains.
- [ ] `sanitize_reference`, `sanitize_documentation`, and `sanitize_path_value` are removed from `crate::sanitize`; `sanitize_path_string` and `crate::document::sanitize_references` remain.
- [ ] `Record::is_html_mockup` is removed; every former call site uses `record.descriptor.document_model == DocumentModel::SingleHtmlMockup`, and a repository-wide grep for `is_html_mockup` returns nothing.
- [ ] `edit_page_test_case`, `edit_page_test_report`, `conflict_page_test_case`, and `conflict_page_test_report` are confirmed absent from source; the `browser_config_types_cli` test passes without weakening its negative assertions at lines 705-728.
- [ ] No unused import or dead-code warning remains: clippy with `-D warnings` and `cargo fmt --check` are clean.
- [ ] `cargo test` passes the full suite, and no Backlog or Decision behaviour test changes.
- [ ] A task comment lists every symbol from the MS-037 AC-1 dead-code list as removed or already absent, with module and replacement path.
## Comments
- **rust-dev**: TASK-282 rust-dev evidence. REMOVED: all_descriptors, list_descriptors, is_managed_record (src.record.descriptor.rs; replacement descriptors() and descriptor_for); apply_patches (src.record.patch.rs; apply_patch and apply); read_store (src.record.store.rs; Store.read and read_document); to_text (src.record.document.rs; render); build_raw, format_scalar_pub (src.record.frontmatter.rs; patch_raw_value and format_scalar); canonical_key (src.decision.rs; canonical_id); matches_family_id, IdentitySpec.validate_input (src.record.identity.rs; parse_id_parts and IdSpec.parse_id); sanitize_reference, sanitize_documentation, sanitize_path_value (src.sanitize.rs; sanitize_path_string; plural sanitize_references kept). Record.is_html_mockup removed after inlining its 5 call sites (browser mod.rs lines 1970, 2003, 2141 and browser render.rs lines 2542, 2893) to record.descriptor.document_model == DocumentModel.SingleHtmlMockup. ALREADY ABSENT (no action): edit_page_test_case, edit_page_test_report, conflict_page_test_case, conflict_page_test_report; the last two remain only as negative strings in tests.browser_config_types_cli.rs lines 705-728, kept unchanged and passing. CHECKS: cargo build OK; cargo build --release OK, binary 8245400 bytes = 7.86 MiB (< 12 MiB gate); cargo test (env -u EDITOR -u VISUAL) 1398 passed, 0 failed, 27 ignored across 70 binaries; browser_config_types_cli 30 of 30 pass; cargo clippy --all-targets -- -D warnings clean; cargo fmt --check clean. No public API, help text, JSON, or behaviour change (removed items were internal aliases only; cli_help_golden, characterization_replay, no_drift green). Status left In-Progress.
- **orchestrator**: Acceptance met: 14 dead symbols removed, is_html_mockup inlined at 5 call sites then deleted, 4 symbols already absent and recorded; negative renderer assertions preserved; 1398 tests pass, clippy and fmt clean, release binary 7.86 MiB under the 12 MiB gate.
