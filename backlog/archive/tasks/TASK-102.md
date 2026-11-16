---
id: TASK-102
title: Refactor the record layer for document archive and status
status: Done
assignee:
  - rust-dev
created_date: '2026-11-13T21:24:00Z'
updated_date: '2026-11-14T09:00:00Z'
labels:
  - rust
  - architecture
milestone: MS-018
dependencies:
  - TASK-100
references:
  - >-
    docs/architecture/adr/ADR-0022-document-lifecycle-state-and-archive-location.md
  - docs/architecture/adr/ADR-0023-consolidated-record-layout-and-field-map.md
  - docs/specifications/refactor-record-layer-for-document-archive-and-status.md
documentation:
  - >-
    docs/architecture/adr/ADR-0022-document-lifecycle-state-and-archive-location.md
  - docs/architecture/adr/ADR-0023-consolidated-record-layout-and-field-map.md
  - docs/specifications/refactor-record-layer-for-document-archive-and-status.md
priority: high
type: feature
ordinal: 41010
---


## Description
Refactor `src/record/` so document lifecycle uses main status `active`/`archived`, an `archived/` subfolder, archive as patch-status-then-write-destination-then-unlink, no document restore, tolerant reads of non-conforming status, and auto-relocate on read for archived-in-active-folder records (ADR-0022 decision 12). This establishes the storage/status contract consumed by TASK-110, TASK-113, and TASK-114.

Scope:
- `src/record/descriptor.rs`: add `resolve_archived_directory`; delete `resolve_deprecated_directory`; remove deprecated from `resolve_read_directories`; update `frontmatter_keys` for milestone, release, audit, and every document type to include `status` in canonical order.
- `src/record/store.rs`: add document `archive`, `delete`, and `archive_content`; no document restore; archive idempotency, auto-relocate-on-read counterpart, and location/status disagreement validation. Read paths auto-relocate archived-in-active-folder records using the archive sequence; on failure source stays intact and read returns a read-only record with a validation outcome.
- `src/record/migrate.rs`: stop managing/reading `deprecated/`; seed `status` during lazy adoption for milestone, release, document, and audit records.
- `src/record/identity.rs`: allocation scans active + archived only; deprecated inputs removed from helpers.
- `src/record/resolve.rs` and `frontmatter.rs`: preserve non-conforming status verbatim and expose duplicate detection across active + archived.

Authority: ADR-0022 (incl. decision 12), ADR-0023, REQ-05.007, REQ-05.010, REQ-05.011, REQ-07.001, REQ-07.002. Specification: `docs/specifications/refactor-record-layer-for-document-archive-and-status.md`.

Assumptions:
- Assumption: static `RecordDescriptor` remains until TASK-108/TASK-110; TASK-107 updates it to the new contract.
- Assumption: `store::archive` creates the destination directory when absent.
- Assumption: archived records are not resolvable by ID for view/list/search; read directories are used for migration/conflict/duplicate detection only.
- Assumption: no high-water mark is added; allocation remains `max(observed) + 1` (deleted-ID reuse accepted, documented).
- Assumption: `deprecated/` folders remain on disk untouched and are not migrated; no inventory command.
- Assumption: read-path auto-relocate uses `store::archive`'s destination-write-then-unlink sequence; on failure the source stays intact and the read returns a read-only record with a validation outcome; writes refused until resolved.
- Assumption: an archived-folder record whose status is `active` or a custom status is a validation error (no relocation).

Dependencies: TASK-105 (completed); blocks TASK-110, TASK-113, TASK-114. Scope risk: identity helper signature changes ripple to per-type modules.

Open questions:
- Configuration-status shrink inventory and git handling of `archived/` remain open in ADR-0022; not implemented here.
- Settled defaults (ADR-0022 decision 12): deleted-ID reuse under `max+1`, auto-relocate on read with failure handling, orphaned `deprecated/` inert. No further product decision needed for this release.

## Acceptance Criteria
- [x] `RecordDescriptor` provides `resolve_archived_directory` for active/archive locations and no longer provides `resolve_deprecated_directory`; `resolve_read_directories` excludes every `deprecated/` path.
- [x] `record::store` exposes document `archive` and `delete` (and `archive_content`) and no document restore operation; task and milestone archive/restore remain unchanged.
- [x] `record::store::archive` patches status to `archived`, writes the destination through the atomic sequence, and removes the source only after the destination write succeeds.
- [x] `record::store::archive` is idempotent for an already-archived record (no write, no `updated_date` change); an active-folder record whose `status` is `archived` is relocated to `archived/` using the same sequence (status patch is a no-op), returning an error with the source intact on failure; an archived-folder record whose status is `active` or a custom status returns a validation error with no write or relocation.
- [x] `record::store::delete` archives an active document before unlinking and removes an archived document directly.
- [x] A document status write sets `active` or a declared custom status, refreshes `updated_date`, performs no move, creates no folder, rejects `archived` with a message naming `archive`, and rejects an archived record.
- [x] The front-matter key registry declares `status` for milestones (`open`/`closed`), releases (`planned`/`released`), and every front-matter document type (`active`, `archived`, plus configured custom/default); decisions and UI mockups do not use the document status model.
- [x] ID allocation scans active and archived locations only and excludes `deprecated/`; allocation remains `max(observed)+1` (minimum 1), so deleting the highest record can reuse its number; no persisted high-water mark is introduced (deleted-ID reuse is a settled default).
- [x] Reading an undeclared status preserves the value verbatim, flags the record, and refuses status writes until the value or configuration is fixed; archive and delete still work.
- [x] `migrate` no longer reads or manages `deprecated/` and seeds `status` during lazy adoption of milestone, release, document, and audit records.
- [x] Read paths auto-relocate an active-folder record whose `status` is `archived` to `archived/` before returning it; on relocation failure the record is returned read-only with a validation outcome and writes are refused. Tests cover success, destination-write failure (source intact), and source-unlink failure (both copies, runtime error naming both paths).
- [x] Settled defaults are documented and not silently encoded as open decisions: deleted-ID reuse under `max+1`, auto-relocate on read with failure handling, orphaned `deprecated/` inert (no inventory command). Configuration-status shrink inventory and git handling of `archived/` remain explicitly open in ADR-0022 and are not implemented here.

## Definition of Done
- [ ] #1 `cargo build`, `cargo test`, and `cargo clippy` pass, including `tests/no_drift.rs`.
- [ ] #2 No `resolve_deprecated_directory` or deprecated read path remains in `src/record/`.
- [ ] #3 No hardcoded `deprecated/` record-directory literal in production code outside historical ADRs/upgrade notes.
- [ ] #4 Front-matter key order assertions pass for milestone, release, documents, and audit.
- [ ] #5 Tests listed in the specification Test plan exist and pass.
- [ ] #6 Settled defaults and any remaining open questions are explicitly recorded; no silent resolution (deleted-ID reuse, auto-relocate on read with failure handling, orphaned deprecated/ inert; configuration-status shrink inventory and git handling of archived/ remain open in ADR-0022).


## Implementation Notes
# Implementation Plan

## 1. Descriptor (`src/record/descriptor.rs`)
1.1 Add `resolve_archived_directory(cfg, project_dir) -> Option<Result<PathBuf>>`:
   - documents: `Some(Ok(active.join("archived")))`.
   - task/milestone: `Some(Ok(backlog/archive/<subpath>))`.
   - decision/other: `None`.
1.2 Delete `resolve_deprecated_directory`.
1.3 `resolve_read_directories`: active + archived + task/milestone legacy alias only; no deprecated.
1.4 `frontmatter_keys`:
   - milestone: `["release", "status"]` -> canonical `status` before `release` per ADR-0023 (match registry).
   - release: `["status"]`.
   - document descriptors incl. audit: prepend/insert `status` immediately after general keys.
   - task/decision/ui-mockup unchanged.
1.5 Update unit assertions for descriptor count/key order.

## 2. Store (`src/record/store.rs`)
2.1 `pub fn archive(ctx, descriptor, id) -> Result<PathBuf, AppError>`:
   - resolve active record via descriptor prefixes/dotted shape.
   - archived location -> return existing path, no write.
   - active + `status == "archived"` -> relocate to archived dir using the same destination-write-then-unlink sequence (status patch is a no-op on the value); on failure source stays intact and error returned.
   - otherwise patch `status` to `archived` via `record::patch`.
   - ensure destination dir exists and contained.
   - `write_atomic` destination.
   - unlink source only after successful destination write.
   - unlink failure -> runtime error naming both paths.
2.2 Read-path auto-relocate: expose a helper (or reuse `archive`) that read/list/view/search/browser/ID-lookup paths call when an active-folder file has `status: archived`; on failure return a read-only record plus `AppError::Validation` and refuse writes until resolved. This is the only permitted write on a read path.
2.3 `pub fn delete(ctx, descriptor, id) -> Result<(), AppError>`:
   - active -> `archive`, then remove archived file.
   - archived -> remove file directly.
2.4 `pub fn archive_content(ctx, source, destination, content) -> Result<(), AppError>` for supersede/custom patched content.
2.5 No document restore function. Keep `move_record`, `write_atomic`.

## 3. Migrate (`src/record/migrate.rs`)
3.1 `managed_descriptor`: remove deprecated read locations.
3.2 Adoption seeds status:
   - milestone: `release` + `status=open`.
   - release: `status=planned`.
   - document: general keys + `status=<default or active>` + type-specific keys.
   - audit: general keys + `status=<default or active>` + `type`.
3.3 No status normalization on read/write.
3.4 Conflict/duplicate detection remains across active+archived+legacy alias.

## 4. Identity (`src/record/identity.rs`)
4.1 Keep `next_number_in_dirs` primitive.
4.2 Change helpers to active+archived semantics:
   - `next_number(active, archived, prefix)`
   - `next_html_number(active, archived, prefix)`
   - `next_seq_number(active, archived, group_id)`
4.3 Update all callers (audit, glossary, req, rule, tech_spec, test_case, test_report, ui_spec, use_case, ui_mockup) to pass archived dirs, not deprecated.
4.4 `allocate_next` unchanged (`max + 1`, min 1); deleted-ID reuse accepted and documented.

## 5. Resolve/Frontmatter (`src/record/resolve.rs`, `frontmatter.rs`)
5.1 Duplicate detection across active+archived (callers pass those dirs).
5.2 Preserve non-conforming status verbatim; no normalization.
5.3 Expose current status value for diagnostics; no new vocabulary mutation.

## 6. Verification
6.1 Unit tests for descriptor resolution, archive ordering/idempotency/auto-relocate/relocation failure, delete, allocation scope, adoption, tolerant read, duplicates, read-only outcome.
6.2 Integration tests for archive CLI, auto-relocate on read (success + failure), interruption residual, no restore, status front-matter-only, visibility, supersede.
6.3 `cargo test`, `cargo clippy`, `tests/no_drift.rs` clean.

## Downstream consumption
- TASK-110 generic handler calls `store::archive/delete` and `descriptor::resolve_archived_directory`; read paths trigger auto-relocate.
- TASK-113 search indexes only `resolve_directory` active path.
- TASK-114 browser resolves only active path; archived never indexed/resolvable by ID.
