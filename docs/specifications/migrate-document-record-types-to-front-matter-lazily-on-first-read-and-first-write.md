# Feature Specification — Migrate Document Record Types to Front Matter Lazily on First Read and First Write

**Ticket:** TASK-68
**Milestone:** m-12 (Final Cross-Type Integration)
**Status:** Refined (execution-ready)
**UI mockups:** none

## Purpose

Deliver ADR-0009 Stage 3: lazy, per-record migration of the document record types (business requirement, non-functional requirement, glossary term, UI specification, technical specification, business rule, audit record, decision, release) so that each managed Markdown record gains a YAML front-matter block on its first read and on its first write, with no bulk migration command required. This ticket implements REQ-05.011 (lazy per-record migration) and REQ-05.010 (front matter on every managed record) for the document record family, building only on `record::patch` and `record::store` from TASK-63 and the option exposure from TASK-65. Task and milestone migration is out of scope (TASK-69). UI mockups and other HTML are excluded (REQ-05.008, ADR-0007 decision 11). Non-record prose Markdown is never modified (ADR-0007 decision 15).

## Scope

### In scope

- Lazy migration of every descriptor-claimed document record type: `release`, `decision`, `business requirement`, `non-functional requirement`, `glossary term`, `ui specification`, `technical specification`, `business rule`, `audit record`.
- Front-matter block added on first read and on first write of a managed record that lacks one.
- Idempotent no-op when a record already carries front matter.
- `created_date` sourced from the record's own date section where present (decisions: `## Date`), otherwise the file's modification time; `updated_date` set to the migration time.
- Per-file migration report emitted through the shared migration function: path, action, resulting keys.
- Dry-run mode over the descriptor-claimed corpus as a reporting/verification aid (no writes).
- Recursive traversal of each descriptor directory, including `deprecated/` and `archived/` subdirectories.
- Read-path write failures (read-only media, missing permission) leave the original file byte-identical and surface the error.
- Audit records: move the retired `## Audit Type` value into front-matter `type` and clear that section.
- Frozen ADR bodies: only the front-matter block is added; `## Date` and `## References` stay intact; decision `status` and `type` stay in the document core.

### Out of scope

- Task and milestone migration to the Backlog.md layout — TASK-69.
- Removal of legacy per-module parsers/writers and no-drift enforcement — TASK-70.
- Backlog.md round-trip verification — TASK-71.
- Browser UI changes and metadata projection — TASK-66, TASK-67, TASK-72.
- UI mockups and other HTML files — excluded by REQ-05.008 and ADR-0007 decision 11.
- Backlog.md `docs` and `drafts` records — out of scope (REQ-05.010).
- Prose Markdown not claimed by a descriptor (`docs/specifications/`, `docs/domain-model/`, `docs/business-processes/`, `README` files) — never modified (ADR-0007 decision 15).
- A dedicated bulk migration command or subcommand — none is added (REQ-05.011, ADR-0007 decision 9).
- Identity translation — TASK-64 (document records use Papyrus display IDs directly; no storage-ID translation).
- Corpus-wide committed migration — the repository's own records converge lazily.

## Behavior

### 1. Managed record set and descriptor claim

1. A file is a migration candidate only when a record descriptor claims it. The document record types covered by this ticket are: `release`, `decision`, `business requirement`, `non-functional requirement`, `glossary term`, `ui specification`, `technical specification`, `business rule`, `audit record`.
2. `task` and `milestone` are descriptor-claimed but are handled by TASK-69, not this ticket.
3. `ui mockup` is not a descriptor and is never migrated (REQ-05.008).
4. A file is claimed when its path is under the descriptor's directory (relative to the project root) and its filename matches the descriptor's display ID pattern. Traversal is recursive and includes `deprecated/` and `archived/` subdirectories.
5. Prose Markdown that no descriptor claims — including files under `docs/specifications/`, `docs/domain-model/`, `docs/business-processes/`, and `README` files — is never read for migration and never modified.
6. HTML files, including UI mockups, are never read for migration and never modified.

### 2. Trigger: first read and first write

1. The shared reader (`record::store::read` / `read_document`) checks whether the file begins with a front-matter block. If it does, no migration runs and the file is returned as-is (byte-identical no-op).
2. If the file lacks a front-matter block and is descriptor-claimed, the reader builds a migration patch, applies it through `record::patch`, and writes the result through `record::store::write_atomic` before returning the migrated document. This is the "first read" migration.
3. The shared writer applies the same migration check before any normal write: if the target record lacks front matter, the migration patch is applied first, then the requested patch operations are applied, in a single atomic write. This is the "first write" migration.
4. After either path, the record carries a front-matter block and subsequent reads/writes are no-ops.
5. Migration is idempotent: a record that already carries front matter is never re-migrated on any later read or write, and its bytes are unchanged by the check itself.

### 3. Migration patch contents (document records)

For each migrated record, the patch consists of:

1. `SetFrontMatter` for the general keys in canonical order:
   - `id` — from the existing `# <ID> <title>` heading or, if absent, from the filename (without extension). For document records this is the Papyrus display ID; no storage-ID translation applies.
   - `title` — from the existing `# <ID> <title>` heading (the text after the ID). If no heading exists, an empty string.
   - `created_date` — sourced per section 4.
   - `updated_date` — the migration time.
   - `labels` — empty list if not present in the body.
   - `references` — empty list if not present in the body.
2. `SetFrontMatter` for type-specific keys, when applicable:
   - `audit record`: `type` — from the `## Audit Type` section value if present, otherwise empty.
   - `decision`: `superseded_by` — only if a supersession is already recorded in the body (for example in `## Status`); otherwise omitted.
   - `release`, `business requirement`, `non-functional requirement`, `glossary term`, `ui specification`, `technical specification`, `business rule`: no type-specific keys.
3. Audit records only: `ClearSection("## Audit Type")` after its value has moved to front matter, so the retired metadata section does not duplicate the front-matter field. All other document record bodies are not changed beyond the inserted front-matter block.
4. No `SetHeadingTitle`, `SetSection`, or `ClearSection` operation is applied to any other record type. The existing heading, preamble, sections, marker comments, and line endings are preserved byte-for-byte.
5. Frozen ADR bodies (`docs/architecture/adr/ADR-*.md` and `docs/architecture/adr/BDR-*.md`) keep their existing heading and all sections; only the front-matter block is inserted. `## Date` and `## References` remain part of the body and are neither removed nor duplicated into front matter as a replacement. Decision `status` stays in `## Status` and decision `type` stays derived from the ID prefix.

### 4. `created_date` provenance

1. For decisions, parse the `## Date` section. If it contains a date, convert it to a quoted ISO-8601 UTC timestamp (`'YYYY-MM-DDTHH:MM:SSZ'`), using midnight UTC when only a date is given. If the section is absent or unparseable, fall back to the file's modification time.
2. For all other document record types, use the file's modification time.
3. `updated_date` is the migration time (the single UTC clock value obtained for this write).
4. Provenance is best-effort and is documented as such: pre-existing `created_date` values approximate the real creation date and are never recomputed by later writes.

### 5. Timestamp format

1. `created_date` and `updated_date` are quoted ISO-8601 UTC strings at second precision: `'YYYY-MM-DDTHH:MM:SSZ'`.
2. A small in-project UTC formatter is used; no new time-formatting dependency is added (ADR-0001, ADR-0005).

### 6. Idempotency

1. A record that already begins with a front-matter block is a no-op on both read and write: no patch is applied, no write occurs, no timestamp is refreshed, and no report entry is emitted for a live run.
2. Re-running migration on an already-migrated record produces byte-identical file content.
3. Migration never runs twice for the same record within one process once the in-memory check has established front matter; the file itself is the source of truth.

### 7. Per-file report

1. The shared migration function emits one report entry per file it actually changes, containing: path (relative to project root), action, and the resulting front-matter key list.
2. Action values for document records:
   - `added-front-matter` — front-matter block inserted, body otherwise unchanged.
   - `added-front-matter;cleared-retired-section` — front-matter block inserted and the retired `## Audit Type` section cleared (audit records only).
3. Live runs (triggered by a real read or write) print one line per changed file to stdout in the form `<path>\t<action>\t<comma-separated keys>`. Unchanged files produce no output.
4. Dry-run mode traverses the descriptor-claimed corpus without writing and prints one line per candidate file with action `would-migrate` (if lacking front matter) or `already-migrated` (if carrying front matter), followed by the resulting key list. Dry-run is exposed to tests and internal callers; no migration subcommand is added.
5. The per-file report is the review artifact for convergence: after a full dry-run shows no `would-migrate` entries, every managed document record carries front matter.

### 8. Read-path failure handling

1. If a first-read migration cannot complete the atomic write (read-only media, missing permission, I/O error), the reader leaves the original file byte-identical, removes any temporary file, and returns the error to the caller. The read command fails with a non-zero exit code.
2. The record is not reported as migrated and is not returned as migrated; the caller sees the failure.
3. A failed migration does not partially write the file: atomicity follows ADR-0006 (create_new + sync_all + same-directory rename).

### 9. Concurrency

1. Two processes reading the same unmigrated record concurrently may both attempt migration. The atomic write contract serializes them; the second writer observes the already-migrated file and becomes a no-op.
2. A reader that loaded pre-migration bytes is not silently upgraded.

### 10. Validation and error behavior

1. A file whose path escapes the project directory produces a runtime error (exit code 1) consistent with existing commands.
2. A malformed front-matter block produces a validation error (exit code 4) rather than a panic.
3. A missing record produces a not-found error (exit code 3).
4. A file that is not descriptor-claimed is never migrated and never modified, even if it is Markdown.
5. Migration failures on the read path are reported as errors, not warnings.

## Inputs and outputs

- Input: a project `Context` and a record path or descriptor-claimed location.
- Output: a migrated `Document` envelope (front matter plus body units) and the ADR-0006 version token on read; new file text on patch; per-file report lines on stdout for changed files; success or error on write.
- No new user-facing command or subcommand is added.

## Assumptions

- Assumption: document record migration is front-matter insertion plus, for audit records only, clearing the retired `## Audit Type` section. No other body section is added, removed, or rewritten.
- Assumption: `labels` and `references` default to empty lists when the legacy body does not contain them; no inference from prose is attempted.
- Assumption: decision `superseded_by` is migrated only when already recorded in the body (for example in `## Status`); otherwise it is omitted and remains unset until a command writes it.
- Assumption: `title` and `id` are derived from the existing `# <ID> <title>` heading when present; if the heading is absent, `id` is derived from the filename and `title` defaults to an empty string.
- Assumption: dry-run traversal covers every Markdown file under each descriptor's directory recursively, including `deprecated/` and `archived/` subdirectories, and is invoked by tests and internal callers rather than a user-facing command.
- Assumption: live migration prints report lines only for files that actually change; unchanged files produce no output.
- Assumption: the repository's own managed document records are not bulk-migrated by this ticket; they converge lazily on first read or write.
- Assumption: no UI design or mockup is required because migration is a backend read/write-path change with no user-facing UI beyond existing command output.

## Open questions

- OQ-1: Whether decision `superseded_by` should be parsed from `## Status` prose during migration, or left unset. This ticket specifies parsing only when the value is unambiguously present; if stakeholders want automatic inference, a follow-up is required.
- OQ-2: Whether `labels` and `references` could be backfilled from body sections (e.g., `## Labels`) during migration. This ticket specifies empty defaults; if backfilling is desired, the migration would need section-parsing rules per type.

## Traceability

| Artifact | Reference |
|---|---|
| Ticket | TASK-68 |
| Requirements | REQ-05.010, REQ-05.011, REQ-06.004, NFR-04.001, NFR-04.002, NFR-04.003 |
| ADRs | ADR-0006, ADR-0007, ADR-0008, ADR-0009 |
| Related tickets | TASK-63 (Done), TASK-64 (Done), TASK-65 (Done), TASK-69, TASK-70, TASK-71, TASK-72 |
| Source | src/record/*, src/requirement.rs, src/glossary.rs, src/ui_spec.rs, src/tech_spec.rs, src/rule.rs, src/audit.rs, src/decision.rs, src/release.rs |
| Tests | tests/*_cli.rs, tests/browser_cli.rs |
