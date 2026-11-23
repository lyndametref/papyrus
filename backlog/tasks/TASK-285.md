---
id: TASK-285
title: "Run the code audit on the refactored document engine"
created_date: '2026-11-20T17:44:13Z'
updated_date: '2026-11-23T06:17:42Z'
labels: []
references:
  - ADR-036
  - ADR-037
  - ADR-038
  - ADR-039
  - REQ-07.004
  - MS-037
  - REL-001
  - AUD-146
  - AUD-149
status: ToDo
assignee:
  - code-analyzer
milestone: MS-037
dependencies:
  - TASK-283
  - TASK-284
ordinal: 60
documentation:
  - src/browser/mod.rs
  - src/document.rs
  - src/record/edit.rs
  - src/record/field_map.rs
  - src/config.rs
  - src/registry.rs
  - src/record/descriptor.rs
  - src/browser/records.rs
  - src/browser/assets.rs
  - src/browser/http.rs
  - src/browser/sha256.rs
  - tests/no_drift.rs
  - tests/document_type_no_drift.rs
  - tests/no_descriptor_leaks.rs
  - src/record/mod.rs
  - tests/build_package_guard.rs
type: audit
---

## Description
## Context
The refactor replaces nine per-type document modules and every document browser branch with one descriptor-driven engine (ADR-036, ADR-039). A code audit is the last chance to catch behaviour that was deleted rather than generalized. MS-035 and MS-036 already passed code gates AUD-145/AUD-146/AUD-148/AUD-149; this audit is release-wide over the refactored tree at the MS-037 boundary.

## Scope
Audit the production tree and record one audit record of type `code` with a severity per finding:
- Unified handler: `src/browser/mod.rs` (`form_post` ~2352, the document handlers, `save_mockup_document` ~2023, `document_allowed_keys`, `document_submitted_values`), `src/document.rs`, `src/record/edit.rs`, `src/record/field_map.rs`.
- Descriptor and seed layer: `src/config.rs` (`SEED_DOCUMENT_TYPES` ~746, the `SEED_*_FIELDS` tables, `validate_field_sequence`), `src/registry.rs` (`TypeDescriptor`, `DocumentModel`, `ArtifactCategory`, `TypedHandler`), `src/record/descriptor.rs`.
- Snapshot and cache: `src/browser/mod.rs` (`ServingSnapshot` ~159, `ServerState` ~201, `reload_if_changed` ~251), `src/browser/records.rs` (`build_index_with_snapshot` ~1284, request snapshot ~2064).
- Asset path: `src/browser/assets.rs`, `src/browser/http.rs` (`CacheClass`), `src/browser/sha256.rs`.
- Guards: `tests/no_drift.rs`, `tests/document_type_no_drift.rs`, `tests/no_descriptor_leaks.rs`, `src/record/mod.rs::read_path_guard_tests`, `tests/build_package_guard.rs`.

Check specifically for:
- per-type special cases surviving as slug comparisons or `TypedHandler` matches in the document path;
- `PatchOp` bypasses (direct body writes, `fs::write`, a second writer in the engine);
- `unwrap`, `expect`, `panic`, or indexing on configuration-controlled input (field keys, headings, directories, statuses, HTML metadata);
- dead code left after TASK-282 and the refactor.

## Out of scope
- Do not fix findings. A blocking finding creates a fix task through the planner, not a direct edit.
- Do not edit source, requirements, rules, ADRs, specifications, mockups, or test cases.

## Definition of done
One `AUD` record exists with type `code`, references MS-037 and REL-001 and the audited files, a pass or fail verdict and severity counts (blocker, high, medium, low), and each finding has severity, file and line, evidence, and disposition. A blocker or high finding is reported to the orchestrator with a proposed fix task.
## Acceptance Criteria
- [ ] Exactly one audit record of type code is created and references MS-037 and REL-001.
- [ ] The audit covers the unified handler, the descriptor and seed layer, the snapshot and cache, the asset path, and the guard modules named in the description.
- [ ] Every finding has a severity (blocker, high, medium, low), a file and line, evidence, and a disposition; the record states the severity counts and a pass or fail verdict.
- [ ] The audit explicitly checks for per-type slug comparisons or document TypedHandler matches and reports none or lists each.
- [ ] The audit explicitly checks that the engine emits PatchOp values only and that the record store is the only writer; any direct filesystem write or body formatting is a finding.
- [ ] The audit explicitly checks unwrap, expect, panic, and indexing on configuration-controlled input and lists each with file and line.
- [ ] The audit explicitly checks for dead code left after TASK-282 and the refactor.
- [ ] No blocking finding is left open: each blocker or high finding is reported to the orchestrator with a proposed fix task, and the task does not repair it.
- [ ] The audit record is referenced in a task comment and papyrus check reports no new finding.
