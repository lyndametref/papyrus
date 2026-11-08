# Feature Specification — Migrate Tasks and Milestones to the Backlog.md Layout

**Ticket:** TASK-69
**Milestone:** m-12 (Final Cross-Type Integration)
**Status:** Refined (execution-ready)
**UI mockups:** none

## Purpose

Deliver ADR-0009 Stage 4: lazy, per-record migration of task and milestone files to the Backlog.md on-disk layout, so that each task and milestone gains front matter on its first read and on its first write and is stored at the Backlog.md path, filename, storage ID, and marker-based body shape. This ticket implements REQ-05.011 (lazy per-record migration, including the legacy Papyrus-shape conversion), REQ-05.010 (front matter on every managed record), REQ-01.005 (display/storage ID translation at the storage boundary), and NFR-04.003 (Backlog.md interoperability) for the task and milestone family. It builds on `record::patch` and `record::store` (TASK-63), the ID translation layer (TASK-64), and the option exposure (TASK-65). Document record migration is out of scope (TASK-68); removal of legacy parsers/writers and no-drift enforcement is TASK-70; Backlog.md round-trip verification is TASK-71.

## Scope

### In scope

- Lazy migration of every descriptor-claimed task and milestone file: `backlog/tasks/**`, `backlog/milestones/**`, and their Backlog.md archive directories `backlog/archive/tasks/**`, `backlog/archive/milestones/**`.
- Legacy read support for `backlog/tasks/archived/**` and `backlog/milestones/archived/**`, converted to the Backlog.md archive paths on first read or first write and never written again.
- Front-matter block added on first read and on first write of a task or milestone that lacks one.
- Translation of the Papyrus display ID to the Backlog.md storage ID in the filename, front-matter `id`, and reference values (`milestone:`, `dependencies:`) for every migrated record.
- Body conversion of legacy Papyrus-shape task and milestone files to the Backlog.md marker-based shape, with unknown sections and free text preserved verbatim and a before/after diff reported per converted file.
- Byte-identical preservation of bodies, marker comments, and section order for files already in the Backlog.md storage shape, apart from the front-matter block (and, for milestones created by Backlog.md, the additive general keys and `release`).
- Milestone front matter carrying the general keys plus `release`, with Backlog.md reads, lists, updates, and preservation of the additive keys.
- Conflict and duplicate detection per ADR-0008 decision 9, reported per record, never merged, deleted, or renumbered.
- Idempotent no-op when a record already carries a complete front-matter block.
- Per-file report emitted through the shared migration function: path, action, resulting key list, and a before/after diff for converted files.
- Dry-run mode over the descriptor-claimed task and milestone corpus as a reporting/verification aid (no writes).
- Recursive traversal of each descriptor directory, including `archived/` and legacy `archived/` subdirectories.
- Read-path write failures (read-only media, missing permission) leaving the original file byte-identical and surfacing the error.

### Out of scope

- Lazy migration of document record types — TASK-68.
- Removal of legacy per-module parsers/writers and no-drift enforcement — TASK-70.
- Backlog.md round-trip verification — TASK-71.
- Browser UI changes and metadata projection — TASK-66, TASK-67, TASK-72.
- UI mockups and other HTML files — excluded by REQ-05.008 and ADR-0007 decision 11.
- Backlog.md `docs` and `drafts` records — out of scope (REQ-05.010).
- Prose Markdown not claimed by a descriptor — never modified (ADR-0007 decision 15).
- A dedicated bulk migration command or subcommand — none is added (REQ-05.011, ADR-0007 decision 9).
- Release records (Papyrus-only, no storage counterpart).
- Corpus-wide committed migration — the repository's own records converge lazily.

## Behavior

### 1. Managed record set and descriptor claim

1. A file is a migration candidate only when a record descriptor claims it. The record types covered by this ticket are `task` and `milestone`.
2. `task` and `milestone` are claimed when their path is under the descriptor's directory (relative to the project root) and their filename matches one of the accepted forms:
   - Backlog.md storage form: `<storage-prefix>-<n>` or `<storage-prefix>-<n> - <slug>` (case-insensitive prefix, unpadded or zero-padded number, optional slug suffix).
   - Legacy Papyrus form: `<DISPLAY-PREFIX>-<padded n>` (the configured display prefix and `id_padding`).
   - Legacy archive form: either of the above under `tasks/archived/` or `milestones/archived/`.
3. Traversal is recursive and includes `archive/`, `archived/`, and any nested subdirectory.
4. Prose Markdown that no descriptor claims — including files under `docs/specifications/`, `docs/domain-model/`, `docs/business-processes/`, and `README` files — is never read for migration and never modified.
5. HTML files, including UI mockups, are never read for migration and never modified.
6. A file is claimed by at most one descriptor; `task` and `milestone` are distinct and never overlap.

### 2. Trigger: first read and first write

1. The shared reader (`record::store::read` / `read_document`) checks whether the file begins with a front-matter block. If it does, no migration runs and the file is returned as-is (byte-identical no-op), unless the block is incomplete for the type, in which case the missing keys are added without rewriting present keys or the body.
2. If the file lacks a front-matter block and is descriptor-claimed, the reader builds a migration patch, applies it through `record::patch`, and writes the result through `record::store::write_atomic` before returning the migrated document. This is the "first read" migration.
3. The shared writer applies the same migration check before any normal write: if the target record lacks front matter, the migration patch is applied first, then the requested patch operations are applied, in a single atomic write. This is the "first write" migration.
4. After either path, the record carries a front-matter block and subsequent reads/writes are no-ops.
5. Migration is idempotent: a record that already carries a complete front-matter block is never re-migrated on any later read or write, and its bytes are unchanged by the check itself.

### 3. Storage identity, paths, and filenames

1. On migration, the record is relocated to the Backlog.md storage path and storage filename, and its front-matter `id` becomes the storage form:
   - Task: `backlog/tasks/<n> - <slug>.md` (active) or `backlog/archive/tasks/<n> - <slug>.md` (archived), front-matter `id: TASK-<n>` (uppercase, unpadded).
   - Milestone: `backlog/milestones/m-<n> - <slug>.md` (active) or `backlog/archive/milestones/m-<n> - <slug>.md` (archived), front-matter `id: m-<n>` (lowercase, unpadded).
   where `<n>` is the storage number derived from the existing record identity per ADR-0008 decision 4 (task offset `0`, milestone offset `1`).
2. The display ID is never written to disk; it is derived on every read by `record::identity`.
3. The `- <slug>` suffix is cosmetic. A record is resolved by its storage prefix and number, never by slug; a stale slug is not a conflict and the file is not renamed on title edit.
4. Reference values are translated to storage form on disk:
   - `milestone:` becomes `m-<n>` (storage form).
   - `dependencies:` becomes a list of `TASK-<n>` (storage form, uppercase, unpadded).
5. On read, reference values are translated back to display form for command output, browser views, and JSON.
6. The storage number is taken from the existing record identity and is never recomputed or renumbered. For a legacy Papyrus file named `TASK-018.md`, the storage number is `18`; for `MS-001.md`, the storage number is `0`.
7. If the storage number's digit count exceeds `id_padding`, the display ID uses the wider number; padding is a minimum, never truncated or errored.

### 4. Migration of files already in the Backlog.md storage shape

1. A file is "already in the Backlog.md storage shape" when its filename and front-matter `id` (if present) normalize to the same storage ID in Backlog.md form and its body uses the Backlog.md marker comments or, for milestones, the plain `## Description` form.
2. For such a file:
   - The front-matter block is added if absent, or extended with any missing keys if present but incomplete. Present keys retain their values; no present key is overwritten.
   - The body, including marker comments and section order, is preserved byte-identically. No heading is added or removed. No section is cleared.
   - A task whose front-matter block is already complete is not rewritten at all (no timestamp refresh, no write).
   - A milestone whose front matter carries only `id` and `title` (Backlog.md-created) is extended with `created_date`, `updated_date`, `labels`, `references`, and `release` (defaulting `release` to unset and the lists to empty), while its body stays byte-identical.
3. `created_date` for a storage-shaped file that lacks it is sourced from the file's modification time; `updated_date` is the migration time.
4. The migration of a storage-shaped file never alters a marker comment, a numbered acceptance-criterion item, or any unknown section or free-text block.

### 5. Migration of legacy Papyrus-shape files (body conversion)

1. A file is "in the legacy Papyrus shape" when it has a `# <ID> <title>` heading and retired metadata sections (`## Status`, `## Milestone`, `## Assignee`, `## Labels`, `## Dependencies`) and no Backlog.md marker comments in its `## Description` / `## Acceptance Criteria` sections.
2. Converting such a file necessarily changes the body. The conversion:
   - Drops the `# <ID> <title>` heading; the ID and title move to front matter (the ID in storage form).
   - Moves each retired metadata section's value into the corresponding front-matter key and removes that section from the body:
     - `## Status` → `status`.
     - `## Milestone` → `milestone` (translated to storage form).
     - `## Assignee` → `assignee` (YAML list with the single value).
     - `## Labels` → `labels`.
     - `## Dependencies` → `dependencies` (each translated to storage form).
     - `## Release` (milestones only) → `release`.
   - Maps `## Description` into the Backlog.md marker form:
     ```
     ## Description

     <!-- SECTION:DESCRIPTION:BEGIN -->
     <original description text, verbatim>
     <!-- SECTION:DESCRIPTION:END -->
     ```
   - Maps `## Acceptance Criteria` into the Backlog.md marker form, renumbering items sequentially starting at 1:
     ```
     ## Acceptance Criteria
     <!-- AC:BEGIN -->
     - [ ] #1 <criterion 1>
     - [ ] #2 <criterion 2>
     <!-- AC:END -->
     ```
   - Maps `## Comments` (if present) into the Backlog.md marker form:
     ```
     ## Comments
     <!-- COMMENTS:BEGIN -->
     - **<author>**: <text>
     <!-- COMMENTS:END -->
     ```
   - Preserves every unknown section (any `## ` heading not in the retired set above and not `Description`/`Acceptance Criteria`/`Comments`) and every free-text block verbatim, in file order, appended after the mapped sections.
   - Preserves the `## Description`/`## Acceptance Criteria` content verbatim apart from the added marker lines; an empty description or empty acceptance-criteria list omits the corresponding marker block's inner lines but keeps the heading and markers.
3. `created_date` is sourced from the file's modification time for both task and milestone legacy files. `updated_date` is the migration time.
4. `labels` and `references` default to empty lists when the legacy body does not contain them; no inference from prose is attempted.
5. `documentation` and `type` (task) and `ordinal` (task) default to empty/unset when absent from the legacy body; `ordinal` defaults to the current maximum plus one for the project, or `0` when no tasks exist, matching the create allocation rule.
6. The legacy-to-storage conversion is the only migration path permitted to alter a task or milestone body, it happens on first read or first write, and it is reported per file with a before/after diff.
7. The conversion is a one-time path with no second use in the writer: once a record carries front matter and the Backlog.md body shape, neither a normal edit nor a re-migration re-runs it.

### 6. Milestone front matter and additive keys

1. Milestone front matter carries the general keys (`id`, `title`, `created_date`, `updated_date`, `labels`, `references`) plus `release`.
2. `release` is unset (omitted or null) when the milestone has no release assignment.
3. Backlog.md reads, lists, and updates such a file and preserves the added general keys and `release` (NFR-04.003). Additive keys are tolerated rather than forbidden.
4. A milestone body keeps `## Description` in Backlog.md's plain form (no marker comments for milestones).

### 7. Conflict and duplicate handling

1. Every comparison normalizes both sides first (case-insensitive prefix, padding-insensitive number, offset applied), per ADR-0008 decision 9.
2. The following situations are detected and reported, never resolved silently:
   - **Duplicate storage ID**: two files resolve to the same storage ID (for example `task-18 - a.md` and `task-18 - b.md`, or `TASK-018.md` and `task-18 - ....md`). Listing shows one entry flagged invalid; `view` by that ID fails naming the candidate paths; writes refuse; the lazy conversion stops on that record and reports it. Never merged, deleted, or renumbered.
   - **Front-matter/filename mismatch**: the front-matter `id` and filename disagree after normalization. Read succeeds using the front-matter `id` (canonical); the record is flagged invalid and writes refuse until fixed.
   - **Same title, different numbers**: distinct records that happen to share a title or slug. Not a conflict; slug is never identity.
   - **Legacy and storage-shaped files with different numbers but the same title**: distinct records; migration reports the pair for a human decision; no automatic equivalence, merge, or renumber.
   - **Unknown visible ID**: a visible ID that maps to no storage file. Normal not-found error (exit code 3), listing the expected storage path once for diagnosis.
   - **Number already used in the other ID space**: create allocates past every number in both spaces, active and archived.
3. No conflict path deletes, overwrites, or renumbers a record. Each conflict report names both paths and both IDs (display and storage) so a human resolves it.
4. When a conflict is encountered during lazy migration, that record is not migrated and not relocated; the migration reports the conflict and continues with the next file.

### 8. Legacy archive paths

1. `backlog/tasks/archived/` and `backlog/milestones/archived/` are still read so existing Papyrus archives stay visible.
2. On first read or first write of a record found only in a legacy archive path, it is converted to the Backlog.md archive path (`backlog/archive/tasks/`, `backlog/archive/milestones/`) with the storage filename, and is never written back to the legacy path.
3. After conversion, the legacy path no longer exists for that record (the file is moved, not copied). On read-only media or without write permission, the move fails, the original file is left byte-identical, and the error is reported.
4. Listing scans both the Backlog.md archive path and the legacy archive path for compatibility, but flags any record present in both as a duplicate per section 7.
5. New archives created by Papyrus use only the Backlog.md archive paths.

### 9. Idempotency

1. A record that already begins with a complete front-matter block is a no-op on both read and write: no patch is applied, no write occurs, no timestamp is refreshed, and no report entry is emitted for a live run.
2. A record with an incomplete front-matter block (for example a Backlog.md-created milestone carrying only `id` and `title`) is extended once; after that it is complete and becomes a no-op.
3. Re-running migration on an already-migrated record produces byte-identical file content.
4. Migration never runs twice for the same record within one process once the in-memory check has established front matter; the file itself is the source of truth.

### 10. Per-file report and before/after diff

1. The shared migration function emits one report entry per file it actually changes, containing: path (relative to project root), action, and the resulting front-matter key list.
2. Action values for task and milestone records:
   - `added-front-matter` — front-matter block inserted or extended, body otherwise unchanged (storage-shaped files).
   - `converted-legacy-to-backlog-md` — legacy Papyrus-shape file converted to the Backlog.md shape, including relocation and body conversion.
   - `conflict` — duplicate or mismatch detected; no write performed.
3. For every `converted-legacy-to-backlog-md` entry, a before/after diff between the original file content and the converted file content is emitted as part of the per-file inventory, in unified-diff form, so the body change is reviewable. Live runs print the one-line summary followed by the diff; dry-run prints `would-convert` followed by the diff.
4. Live runs (triggered by a real read or write) print one line per changed file to stdout in the form `<path>\t<action>\t<comma-separated keys>`. Unchanged files produce no output. Conflicts print `<path>\tconflict\t<comma-separated keys>` with the candidate paths on following lines.
5. Dry-run mode traverses the descriptor-claimed corpus without writing and prints one line per candidate file with action `would-migrate` (storage-shaped, lacking front matter), `would-convert` (legacy Papyrus shape), or `already-migrated` (carrying front matter), followed by the resulting key list. Dry-run is exposed to tests and internal callers; no migration subcommand is added.
6. The per-file report is the review artifact for convergence: after a full dry-run shows no `would-migrate` or `would-convert` entries, every managed task and milestone carries front matter in the Backlog.md shape.

### 11. Read-path failure handling

1. If a first-read migration cannot complete the atomic write (read-only media, missing permission, I/O error, or failed relocation of a legacy archive), the reader leaves the original file byte-identical, removes any temporary file, and returns the error to the caller. The read command fails with a non-zero exit code.
2. The record is not reported as migrated and is not returned as migrated; the caller sees the failure.
3. A failed migration does not partially write the file: atomicity follows ADR-0006 (create_new + sync_all + same-directory rename).

### 12. Concurrency

1. Two processes reading the same unmigrated record concurrently may both attempt migration. The atomic write contract serializes them; the second writer observes the already-migrated file and becomes a no-op.
2. A reader that loaded pre-migration bytes is not silently upgraded.
3. Concurrent migration of a legacy archive file serializes on the rename; the loser observes the new path and becomes a no-op.

### 13. Validation and error behavior

1. A file whose path escapes the project directory produces a runtime error (exit code 1) consistent with existing commands.
2. A malformed front-matter block produces a validation error (exit code 4) rather than a panic.
3. A missing record produces a not-found error (exit code 3).
4. A file that is not descriptor-claimed is never migrated and never modified, even if it is Markdown.
5. A storage ID supplied as command input is rejected with a hint naming the equivalent display ID (exit code 4), per TASK-64.
6. Migration failures on the read path are reported as errors, not warnings.
7. A conflict encountered during migration is reported but does not abort the whole traversal; subsequent files continue to be processed.

## Inputs and outputs

- Input: a project `Context` and a record path or descriptor-claimed location.
- Output: a migrated `Document` envelope (front matter plus body units) and the ADR-0006 version token on read; new file text on patch; per-file report lines on stdout for changed files (with a unified diff for converted files); success or error on write.
- No new user-facing command or subcommand is added.

## Assumptions

- Assumption: task and milestone migration is built only on `record::patch` and `record::store`; no second writer is introduced (ADR-0009 decision 8).
- Assumption: `record::identity` (TASK-64) is the sole location of ID parsing, normalization, formatting, and translation; this ticket consumes it but does not duplicate it.
- Assumption: the storage number is derived from the existing record identity; no renumbering occurs.
- Assumption: `labels`, `references`, `documentation`, `type`, and `ordinal` default to empty/unset when a legacy body does not contain them; no inference from prose is attempted.
- Assumption: `ordinal` for a converted legacy task defaults to the current maximum plus one, or `0` when no tasks exist, matching the create allocation rule.
- Assumption: status values are carried verbatim from the legacy `## Status` section; no silent normalization is performed. A value outside the accepted status set flags the record invalid and refuses writes until fixed (ADR-0008 decision 7).
- Assumption: `created_date` and `updated_date` are quoted ISO-8601 UTC strings at second precision (`'YYYY-MM-DDTHH:MM:SSZ'`).
- Assumption: a before/after diff is emitted in unified-diff form for each converted file; the exact diff library or formatting is an implementation choice provided the before and after content are both present.
- Assumption: no UI design or mockup is required because migration is a backend read/write-path change with no user-facing UI beyond existing command output.
- Assumption: the repository's own managed task and milestone records are not bulk-migrated by this ticket; they converge lazily on first read or write.

## Open questions

- OQ-1: Whether a legacy `## Status` value such as `ToDo` should be normalized to the Backlog.md form `To Do` during conversion, or carried verbatim and flagged invalid. This ticket specifies verbatim plus flagging; automatic normalization would require a status-alias table that is not defined anywhere. Not blocking.
- OQ-2: Whether `comments` in a legacy task should be mapped to the `<!-- COMMENTS:BEGIN -->` marker block or appended as an unknown section. This ticket specifies mapping to the comments marker block; if stakeholders prefer verbatim append, the conversion changes. Not blocking.
- OQ-3: Whether a converted legacy file should be removed from the legacy path immediately or only after a successful write to the new path. This ticket specifies move (remove legacy after successful write); a copy-then-delete variant is equivalent for the no-failure case. Not blocking.

## Traceability

| Artifact | Reference |
|---|---|
| Ticket | TASK-69 |
| Requirements | REQ-01.005, REQ-05.010, REQ-05.011, NFR-04.003 |
| ADRs | ADR-0006, ADR-0007, ADR-0008, ADR-0009 |
| Related tickets | TASK-63 (Done), TASK-64 (Done), TASK-65 (Done), TASK-68 (Done), TASK-70, TASK-71, TASK-72 |
| Source | src/record/*, src/task.rs, src/milestone.rs, src/config.rs, src/context.rs |
| Tests | tests/*_cli.rs, tests/browser_cli.rs |
| UI mockups | none |
