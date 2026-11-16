---
id: TECH-066
title: "Refactor the Record Layer for Document Archive and Status"
created_date: '2026-11-16T17:46:53Z'
updated_date: '2026-11-16T17:47:29Z'
labels: []
references:
  - ADR-0006
  - ADR-0010
  - ADR-0012
  - ADR-0013
  - ADR-0017
  - ADR-0022
  - ADR-0023
  - REQ-05.007
  - REQ-05.010
  - REQ-05.011
  - REQ-07.001
  - REQ-07.002
  - TASK-105
  - TASK-107
  - TASK-108
  - TASK-110
  - TASK-113
  - TASK-114
status: verified
---
# TECH-066 Refactor the Record Layer for Document Archive and Status

## Description
## Status

Specification for TASK-107 (record layer refactor; ADR-0022 and ADR-0023).

## Scope

This specification defines the storage and front-matter contract for document archive and status that TASK-107 establishes in `src/record/`. It is consumed by TASK-110 (generic document handler), TASK-113 (search), and TASK-114 (browser). It does not change command names, output, exit codes, or the on-disk layout beyond the `archived/` location already mandated by ADR-0022/ADR-0023.

In scope:
- `src/record/descriptor.rs`: directory resolution, `frontmatter_keys`, allocation inputs.
- `src/record/store.rs`: document `archive` and `delete`, atomic destination-write-then-unlink sequence, idempotency, disagreement validation.
- `src/record/migrate.rs`: lazy adoption seeds `status`; deprecated locations stop being managed/read.
- `src/record/identity.rs`: allocation scans active + archived only.
- `src/record/frontmatter.rs` and `src/record/resolve.rs`: tolerant preservation of non-conforming status and duplicate detection.

Out of scope:
- Static per-type command modules keep their current verbs until TASK-108/TASK-110; TASK-107 only changes the shared record contract they will later consume.
- No UI mockup design.
- Deleted-ID reuse, auto-relocation on read, and inert orphaned `deprecated/` are settled defaults (ADR-0022 decision 12); TASK-107 implements the auto-relocate-on-read path and documents the other two. No high-water mark is introduced and no repair or inventory command is added.
## Source authority

- ADR-0022 (Accepted): lifecycle operations, archive sequence, status command, no restore, tolerant read, visibility.
- ADR-0023 (Accepted): consolidated layout and field map.
- ADR-0010, ADR-0012 (Superseded where contradicted): allocation and metadata rationale.
- ADR-0006 (atomic writes), ADR-0013 (patch/single writer), ADR-0017 (registry/status models).
- REQ-05.007, REQ-05.010, REQ-05.011, REQ-07.001, REQ-07.002.

## Directory resolution contract

`RecordDescriptor`:

- Retain `resolve_directory` (active) and `resolve_archive_directory` (tasks/milestones).
- Add `resolve_archived_directory(cfg, project_dir) -> Option<Result<PathBuf>>`:
  - Document descriptors: `Some(Ok(active_dir.join("archived")))`, contained via `Context::ensure_within`.
  - Task/milestone descriptors: `Some(Ok(.../archive/<subpath>))` (same as their archive location) so downstream has one method.
  - Decision and other families: `None`.
- Delete `resolve_deprecated_directory` entirely.
- `resolve_read_directories` returns active + archived + task/milestone legacy alias only. No deprecated branch.
- `resolve_legacy_archive_directory` remains read-only for tasks/milestones.

No production code in `src/record/` may resolve or read `deprecated/`.

## Front-matter key contract

`GENERAL_FRONTMATTER_KEYS` order unchanged. `frontmatter_keys`:

- milestone: general keys, then `status`, then `release` (canonical order per ADR-0023).
- release: general keys, then `status`.
- every document descriptor: general keys, then `status`, then existing type-specific keys.
- audit: general keys, then `status`, then `type`.
- decision: unchanged (`superseded_by` only; status stays in document core).
- UI mockup: no front-matter keys.
- task: unchanged (status already present).

`migrate::migrate_content` must seed:
- task: existing behavior (status from legacy section or config default).
- milestone: `release` + `status = "open"`.
- release: `status = "planned"`.
- document: general keys + `status = <default_status or "active">` + type-specific keys.
- audit: general keys + `status = <default or active>` + `type`.

## Public API consumed by downstream tasks

`src/record/store.rs`:

- `pub fn archive(ctx, descriptor: &RecordDescriptor, id: &str) -> Result<PathBuf, AppError>`
  - Resolves the active record using descriptor prefixes and dotted shape.
  - If already archived (resolved location is the archived directory), returns `Ok(existing_path)` without write or `updated_date` change.
  - If active and `status == "archived"`, relocates the file to the archived directory using the same destination-write-then-unlink sequence (the status patch is a no-op on the value) and returns the archived path; on failure the source stays intact and an error is returned. This is the store-level counterpart of read-path auto-relocate.
  - Otherwise patches `status` to `"archived"` (plus descriptor-required back-reference where applicable), writes the destination through `write_atomic`, and removes the source only after the destination write succeeds.
  - On destination write failure: source untouched; error returned.
  - On source unlink failure after successful destination write: returns runtime error naming both paths; duplicate detection then makes the record unreadable-for-write until a human removes one copy.
  - Creates the destination directory if needed, contained in the project.
  - Returns the archived path.
- Read-path auto-relocate: when a read, list, view, search, or browser/ID-lookup path encounters a file in the active directory whose `status` is `archived`, it calls `store::archive` (or an equivalent relocate) using decision 2's sequence before returning the record; the relocation is reported in diagnostics. On destination-write or source-unlink failure the source stays intact, no partial state is left, and the read returns a read-only record together with `AppError::Validation` naming both paths; writes to that record are refused until the conflict is resolved. This is the only permitted write on a read path.
- `pub fn delete(ctx, descriptor: &RecordDescriptor, id: &str) -> Result<(), AppError>`
  - Active record: calls `archive`, then removes the archived file.
  - Archived record: removes the file directly.
  - No confirmation or `--force` (CLI concerns).
- `pub fn archive_content(ctx, source: &Path, destination: &Path, content: &str) -> Result<(), AppError>`
  - For callers that patch their own content (UI mockup supersede): atomic destination write, then source unlink; same failure semantics as `archive`.
- No `restore` for documents. `move_record` and `write_atomic` remain.

`src/record/descriptor.rs`:

- `resolve_archived_directory`, `resolve_read_directories` as above.
- `frontmatter_keys` changes above.

`src/record/identity.rs`:

- Allocation scans active and archived locations only. Replace deprecated-parameter helpers:
  - `next_number_in_dirs(dirs, prefix)` remains the primitive.
  - `next_number(active, archived, prefix)` (or remove and update callers to `next_number_in_dirs`).
  - `next_html_number(active, archived, prefix)` / `next_seq_number(active, archived, group_id)` similarly.
- `allocate_next` unchanged (`max + 1`, minimum 1). Deleted numbers remain gaps; deleting the highest number makes it reusable under `max + 1`. High-water mark is an open question, not implemented.

`src/record/migrate.rs`:

- `managed_descriptor` no longer matches `deprecated/`.
- `ensure_front_matter*` remains lazy and non-relocating.
- Adoption seeds `status` as above.
- No normalization of non-conforming status on read or write.

`src/record/resolve.rs`:

- Duplicate detection remains across all passed directories (active + archived + legacy aliases).
- No deprecated directories are passed by record-layer callers.
- Tolerant read preserves `status` verbatim; a non-conforming value is flagged by downstream diagnostics.

`src/record/frontmatter.rs`:

- No change to parsing/serialization.
- `current_value_string` remains the read path for status.
- No automatic status normalization.

## ADR-0022 decision mapping

1. `status` registry key for milestones/releases/documents: static `frontmatter_keys` and registry already model it; migrate seeds it; decisions/mockups excluded.
2. Archive = patch + relocate: `store::archive` and `store::archive_content`.
3. Idempotent archive and auto-relocate: `store::archive` is a no-op for already-archived records and relocates active-folder archived-status records (ADR-0022 decision 12); validation path for an archived-folder record whose status is `active` or a custom status.4. `status` command front-matter only: handler/commands use `write_atomic`, no move/folder; rejects `archived` and archived records. Record layer provides tolerant read and write primitive.
5. No document restore: no `restore` in `store`; document baseline verbs exclude restore (registry already enforces).
6. `archived/` only: `resolve_archived_directory` added; `resolve_deprecated_directory` deleted; no deprecated reads.
7. Delete = archive-then-unlink: `store::delete`.
8. Supersession uses same sequence: `store::archive_content` consumed by UI mockup supersede.
9. Tolerant, non-destructive read: migrate/resolve/frontmatter preserve value; diagnostics flag it; writes refused until fixed.
10. Visibility follows location: search/browser index only active directory; `archived/` never indexed/resolvable by ID.
11. Clause-level supersession: ADR-0010/0012 deprecated clauses not implemented; this spec is authoritative delta with ADR-0023.

## ADR-0023 mapping

- Active/archive layout: `resolve_directory`, `resolve_archived_directory`, `resolve_archive_directory`.
- Allocation scope: identity helpers scan active + archived only.
- Field map: `frontmatter_keys` updates.
- `status` exemption from create/edit options: record layer does not expose status as a create/edit option; only `status` and `archive` write it (enforced by commands/registry).

## Validation and error behavior

- A file in the active folder whose `status` reads `archived` is auto-relocated to `archived/` on read using the archive sequence (see Read-path auto-relocate). It is not a validation error on its own; on relocation failure the source stays intact and the read returns a read-only record plus a validation outcome, and writes are refused until the conflict is resolved.
- A file in the `archived/` folder whose `status` is `active` or a custom status is `AppError::Validation` (exit 4), naming value and file location, with no write and no relocation.- Non-conforming status (outside declared vocabulary) is preserved verbatim, flagged in diagnostics, and `status` writes are refused until value or configuration is fixed. Archive and delete still work.
- Archive is idempotent: already archived returns success without write or `updated_date` change.
- `status` rejects `archived` with a message naming `archive` as the only archiving path.
- `status` rejects an archived record.
- `status` performs no move and creates no folder.

## Test plan

Unit (`src/record/`):
- Descriptor resolution: archived path for documents; no deprecated method; read directories exclude deprecated; front-matter key order/content.
- Store archive ordering: simulate destination write failure (destination path is existing directory) and assert source survives and error returned.
- Store idempotency: archived record returns Ok and content/bytes unchanged.
- Store auto-relocate: active record with `status: archived` is relocated to `archived/` and the archived path is returned; simulated destination write failure leaves the source intact and returns an error; simulated source-unlink failure leaves both copies and returns a runtime error naming both paths.
- Read-path auto-relocate failure: returns a read-only record with a validation outcome and refuses writes.- Store delete: active archives then unlinks; archived unlinks directly.
- Identity allocation: scans active + archived only; deprecated files ignored; deleted highest number reusable under max+1 (gap behavior).
- Migrate adoption: document/milestone/release records receive status on first write.
- Tolerant read: undeclared status preserved verbatim and flagged; no normalization.
- Resolve duplicates: active + archived copies of same ID returns validation error naming both paths.

Integration:
- `papyrus <type> archive <id>` (existing document types) moves file to `archived/`, patches status, leaves no source.
- Process interruption between destination write and source unlink leaves both copies; record becomes duplicate/unreadable-for-write until one copy is removed.
- No restore command exists for documents (command tree/registry invariant).
- `papyrus <type> status` changes front matter only; no move/folder.
- Archived records are absent from list/search/browser index.
- UI mockup supersede moves to `archived/` and sets HTML status to archived using the same sequence.

## Assumptions

- Assumption: static `RecordDescriptor` remains the read/migration identity source until TASK-108/TASK-110; TASK-107 updates it to the new contract rather than deleting it.
- Assumption: `store::archive` creates the destination directory when absent.
- Assumption: archived records are not resolvable by ID for view/list/search; read-directory inclusion is limited to migration/conflict/duplicate detection.
- Assumption: no high-water mark is added; allocation remains `max(observed) + 1`.
- Assumption: `deprecated/` folders remain on disk untouched and are not migrated.
- Assumption: read-path auto-relocate uses `store::archive`'s destination-write-then-unlink sequence; on failure the source stays intact and the read returns a read-only record with a validation outcome.
## Dependencies

- Depends on: TASK-105 (completed).
- Blocks: TASK-110, TASK-113, TASK-114 (and their dependents).
- Scope risk: changing identity helper signatures ripples to audit, glossary, req, rule, tech_spec, test_case, test_report, ui_spec, use_case, and ui_mockup modules. Either update callers in this task or introduce archived-parameter wrappers; recommended to update callers to avoid deprecated reads.

## Open questions

1. Configuration-status shrink: removing a declared custom status leaves records that read but cannot be written; whether to provide a read-only inventory of affected records is open (ADR-0022 Open Questions). Not implemented here.
2. Whether `archived/` is gitignored or version-controlled: kept under version control, current behaviour (ADR-0022 Open Questions). Not changed here.

Settled defaults (ADR-0022 decision 12), not open: deleted-ID reuse under `max + 1` with no high-water mark; auto-relocate on read with failure handling (source intact, read-only record plus validation outcome on failure); orphaned `deprecated/` folders stay inert with no inventory command.

## Definition of done

- `cargo build`, `cargo test`, `cargo clippy` pass, including `tests/no_drift.rs`.
- No `resolve_deprecated_directory` or deprecated read path remains in `src/record/`.
- No hardcoded `deprecated/` record-directory literal in production code outside historical ADRs/upgrade notes.
- Front-matter key order assertions pass for milestone, release, documents, audit.
- Tests listed in Test plan exist and pass.
- Open questions remain explicitly recorded; no silent resolution.
