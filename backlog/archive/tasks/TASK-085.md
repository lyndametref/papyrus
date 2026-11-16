---
id: TASK-085
title: Implement atomic whole-folder relocation for config set
status: Done
assignee: []
created_date: '2026-11-11T20:42:00Z'
updated_date: '2026-11-13T05:45:00Z'
labels:
  - configuration
  - relocation
  - release-1.0
milestone: MS-017
dependencies:
  - TASK-080
  - TASK-083
references:
  - docs/requirements/business/REQ-01.002.md
  - docs/architecture/adr/ADR-0010-papyrus-native-on-disk-record-layout.md
  - >-
    docs/architecture/adr/ADR-0013-shared-record-envelope-without-a-storage-layer.md
  - docs/specifications/project-initialization-and-configuration.md
priority: high
type: feature
ordinal: 39070
---


## Description
Implement the single shared whole-folder relocation routine (record::relocate) required by ADR-0010 decision 8 and ADR-0013 stage 5, and wire it into `papyrus config set` for backlog_directory, decision_directory, doc_directory, and any document type directory key.

Scope:
- Add `src/record/relocate.rs` exposing `relocate(ctx, source, destination) -> Result<RelocationReport, AppError>` (plus `relocate_folder`/`move_folder` aliases). The routine is the only folder-move implementation.
- It moves every file and subfolder of the source folder (including unmanaged files), runs synchronously during `config set`, removes the source folder once empty, and never persists configuration on failure.
- Pre-check the entire plan for destination entry conflicts, containment, and readability before moving anything; on any failure move nothing or roll back already-moved entries, persist no configuration value, and return a conflict outcome (exit code 4).
- Fall back to copy-then-remove on EXDEV (cross-device) and report that a single rename was impossible.
- All source and destination paths pass `Context::ensure_within`; `Config::validate` rejects absolute or traversal folder values.
- Folder-key string values in `config set` are sanitized with the path-aware sanitizer so relative multi-segment folders are accepted while traversal/absolute values are rejected.
- The browser configuration view reaches folder moves only through this `config set` path and performs no folder move of its own.

Out of scope: browser UI implementation (TASK-94/TASK-95), Backlog.md import (TASK-92), and any new crate or storage layer.

## Acceptance Criteria
- [x] record::relocate::relocate is the single implementation used by config set for backlog_directory, decision_directory, doc_directory, and document_types.<type>.directory; the browser performs no folder move of its own.
- [x] On a successful folder change, every file and subfolder of the source folder (including files Papyrus does not manage) is present at the destination and the source folder is removed once empty.
- [x] The move runs inside config set, is a move rather than a copy on the same filesystem, and the configuration value is persisted only after a successful move.
- [x] A destination entry conflict, permission failure, or write failure moves nothing or rolls back already-moved entries, persists no configuration value, and returns exit code 4 with a conflict message.
- [x] A partial failure during relocation rolls back moved entries and leaves both folders as they were.
- [x] A cross-device move falls back to copy-then-remove and config set reports that a single rename was impossible.
- [x] Every source and destination path passes Context::ensure_within, and Config::validate rejects absolute and traversal folder values.
- [x] config set for a folder key accepts relative multi-segment folder values and rejects traversal/absolute values.
- [x] Integration tests cover a successful move, a destination conflict, partial-failure rollback, cross-device fallback handling, and unchanged configuration on failure.

## Definition of Done
- [x] #1 Implementation in src/record/relocate.rs and src/commands.rs; Config::validate updated.
- [x] #2 cargo build and cargo test pass, including no_drift checks.
- [x] #3 Existing config/browser/integration tests remain green.


## Implementation Notes
1. Add src/record/relocate.rs implementing plan/pre-check/move/rollback and EXDEV fallback.
2. Register module in src/record/mod.rs.
3. Add Config::is_folder_key and folder-value validation in Config::validate.
4. Update config_set sanitization for folder keys and integrate relocation before persistence.
5. Add folder_paths helper in commands.rs.
6. Add/adjust tests for success, conflict, rollback, cross-device report, and validation.


## Notes
Refinement only: ticket scope, acceptance criteria, assumptions, dependencies, and DoD aligned with REQ-01.002, ADR-0010 decision 8, and ADR-0013 stage 5. Implementation not performed in this pass.

Assumptions:
- Rust 1.99.1/Cargo available (ADR-0001).
- record::relocate lives in src/record/ and is the only folder-move implementation (ADR-0013).
- Config file values for folders are relative to the project directory; absolute/traversal values are rejected (NFR-04.002).
- Cross-device detection uses OS rename failure EXDEV and falls back to copy-then-remove.
- Browser configuration view (TASK-94) delegates to config set and performs no folder move.

Open questions: none.

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Implemented record::relocate atomic folder relocation and wired into config set for folder keys; added Config::is_folder_key and folder validation; path-aware sanitization; updated affected tests.
