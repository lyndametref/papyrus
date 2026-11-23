---
id: TASK-285
title: "Run the code audit on the refactored document engine"
created_date: '2026-11-20T17:44:13Z'
updated_date: '2026-11-23T08:50:58Z'
labels: []
references:
  - ADR-036
  - ADR-037
  - ADR-038
  - ADR-039
  - REQ-007.004
  - MS-037
  - REL-001
  - AUD-146
  - AUD-149
status: Done
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
## Comments
- **requirements-engineer**: REQ-007.002 aligned with ADR-036 clarifications (2026-11-22 seed resolution; 2026-11-23 values and reserved keys). Description: field-item list gains tenth key `values` with its semantics; reserved set now 13 (adds version, kind, form, path, metadata); merge rule now 'eleven seed slugs always effective, present document_types is an overlay, absent slug scoped to user-defined slugs and the written file'. Acceptance criteria: field-item key list gains `values`; new `values` validation criterion; reserved-key criterion now 13 keys; merge criterion now the overlay rule. Verified: `papyrus req view REQ-007.002` parses; `papyrus check` output identical to baseline (no new findings, exit 4 from pre-existing findings). Impact flags: BR-016 clauses on field-item keys, reserved keys and seed merge are stale (delegation to business-analyst blocked by subagent depth limit); REQ-005.010 still cites 8 reserved keys; TECH-077 section 3 lists 9 field keys and 8 reserved keys; docs release-1.0-upgrade-notes.md line 339. No other requirement or task changed.
- **requirements-engineer**: BR-016 and REQ-005.010 aligned with ADR-036 decisions 11 and 23 and the 2026-11-22 seed-overlay clarification. BR-016: line 46 field-item key list now the ten keys including `values` (optional; absent means no restriction beyond `kind`; an allowed-string sequence restricts a scalar, `record-ids` requires every `list` item to be a record ID; adds no option and no verb); line 49 reserved field keys now the 13-key set (id, title, status, labels, references, created_date, updated_date, group, version, kind, form, path, metadata); line 61 merge rule now states the eleven seed slugs are always effective, a present `document_types` map is an overlay, an absent user-defined slug stays absent, and the written file never gains an untargeted entry. REQ-005.010: line 35 reserved-key statement now the 13-key set; line 63 reserved-key acceptance criterion now the 13-key set. No other clause changed; both updated_date lines (BR-016 line 5, REQ-005.010 line 5) set to 2026-11-23T07:27:35Z. Verified: papyrus rule view BR-016 and papyrus req view REQ-005.010 parse; papyrus check identical to baseline (same 5 pre-existing findings, exit 4). Impact flags: TECH-077 section 3 still lists 9 field keys and 8 reserved keys; docs release-1.0-upgrade-notes.md line 339 still stale; REQ-005.010 traceability line naming "the eight reserved keys" left intact per instruction. No request artifact was changed; TASK-285 still In-Progress.
- **rust-dev**: ADR-036 clarification (2026-11-23), documentation only, no code change. TECH-077 section 3: field-item key list now the ten keys including values, with a new values bullet (optional; a non-empty allowed-string sequence, or record-ids for a list whose items must be record IDs; enforced by the field-domain validator; no option and no verb of its own). TECH-077 section 3: reserved field-key list now the 13 keys, adding version, kind, form, path, and metadata, with the reason that they collide with the envelope, token, and form-post keys. docs release-1.0-upgrade-notes.md line 339 now states the eleven seed slugs are always effective, a present document_types map is an overlay, and collections replace per declared entry. TECH-077 status left verified (consistent clarification); body edited in place because the managed CLI edits only the Description field. Verified: papyrus tech-spec view TECH-077 exits 0; relative links in the upgrade notes resolve; papyrus check shows the same 5 pre-existing findings (exit 4) and no new finding.
- **orchestrator**: Release code audit AUD-152 PASS: 0 blocker, 0 high, 0 medium, 2 low; 1430 tests green with and without EDITOR; corpus reconciled (ADR-036 values key, 13 reserved keys, seed overlay in REQ-007.002, REQ-005.010, BR-016, TECH-077).
