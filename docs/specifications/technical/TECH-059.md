---
id: TECH-059
title: "Import Backlog.md Records (`papyrus import backlog`)"
created_date: '2026-11-16T17:46:52Z'
updated_date: '2026-11-16T17:47:26Z'
labels: []
references:
  - ADR-0005
  - ADR-0010
  - ADR-0011
  - ADR-0012
  - ADR-0013
  - NFR-04.002
  - NFR-04.004
  - REQ-01.002
  - REQ-01.003
  - REQ-01.004
  - REQ-01.005
  - REQ-05.010
  - TASK-100
  - TASK-83
  - TASK-88
  - TASK-89
  - TASK-90
  - TASK-92
  - TASK-97
  - TASK-98
  - TASK-99
status: verified
---
# TECH-059 Import Backlog.md Records (`papyrus import backlog`)

## Description
**Ticket:** TASK-83 (import sub-specification; implemented by TASK-92)
**Milestone:** m-16 (Release 1.0)
**Status:** Refined (execution-ready)
**UI mockups:** none

## Purpose

Define the one-way import of Backlog.md task and milestone records into Papyrus records: invocation and options, source scope and exclusions, planned ID allocation, field and body mapping, status normalization, collision and duplicate handling, the one-way non-idempotent caveat, exit codes, and containment. This specification implements ADR-0011 and NFR-04.004 and is the only supported bridge from a Backlog.md store into the Papyrus-native layout (see `papyrus-native-record-layout-and-single-id-space.md`).

## Scope

### In scope

- The dedicated CLI command `papyrus import backlog` with its options.
- Source scope: `backlog/tasks`, `backlog/milestones`, and their `archive/` counterparts; exclusion of `docs`, `drafts`, `decisions`, configuration, and locks.
- Phase 1 (plan) / phase 2 (resolve) / phase 3 (write) allocation of fresh Papyrus IDs before any write.
- Field mapping and content preservation, including unknown keys and unknown sections.
- Status normalization (case-insensitive, separators removed) with verbatim preservation and warning on no match.
- Body mapping from Backlog.md marker sections to the Papyrus section layout.
- Collision and duplicate handling, including `--allow-existing`.
- One-way, non-idempotent semantics; no persisted mapping.
- Exit codes per REQ-01.004.
- Containment per NFR-04.002.
- Upgrade-notes contract for the one-time import.

### Out of scope

- Importing `docs`, `drafts`, or `decisions`.
- Importing Backlog.md configuration as Papyrus configuration.
- Release records (none exist upstream).
- Re-import reconciliation, continuous or scheduled sync, or reverse export.
- UI mockups and browser UI changes.

## Behavior

### 1. Invocation and inputs

```
papyrus import backlog --from <source-project-dir> [--source-backlog-dir <dir>]
                       [--dry-run] [--allow-existing] [--report <path>]
```

1. The target project is selected per REQ-01.003 (`--dir`, `PAPYRUS_DIR`, cwd) and must have a `papyrus.config.yml`; otherwise the command fails as a runtime error (REQ-01.003, REQ-01.004).
2. `--from` names the Backlog.md project root; `--source-backlog-dir` defaults to `<from>/backlog` for projects that do not use the default folder name.
3. `--dry-run` performs phase 1 and phase 2 fully and prints the plan and report without writing anything.
4. `--allow-existing` is required when the target already holds any record of a family being imported; without it the command refuses before writing (section 7).
5. `--report <path>` writes the same report to a path inside the project directory only; a path outside is a validation error (NFR-04.002). The report is diagnostic output, never a machine-readable store that later commands consult.
6. Exit codes follow REQ-01.004: `0` when every in-scope source record was imported (or planned, for a dry run); `1` runtime/validation failure; import with skipped records exits non-zero and the summary lists them.

### 2. Source scope

| Source | Treatment |
|---|---|
| `backlog/tasks/*.md` | Imported as tasks |
| `backlog/archive/tasks/*.md` | Imported as archived tasks |
| `backlog/milestones/*.md` | Imported as milestones |
| `backlog/archive/milestones/*.md` | Imported as archived milestones |
| `backlog/docs/**`, `backlog/drafts/**`, `backlog/archive/drafts/**`, `backlog/decisions/**` | Not read as records; counted and reported as excluded by NFR-04.004 |
| `backlog/config.yml`, `backlog/.locks/**` | Source metadata only; never written; the source status vocabulary and prefixes may be read to interpret the source |

1. Only `.md` files whose stem matches the source family's storage form (`<prefix>-<number>`, optionally followed by ` - <slug>`) are candidates. Anything else in a source directory is reported as skipped.
2. Files outside the source backlog directory and outside `backlog/archive/` are never read.
3. The import never overwrites, deletes, renumbers, or merges an existing Papyrus record.

### 3. ID assignment (fresh Papyrus numbers, allocated before any write)

1. Phase 1 (plan) enumerates every in-scope source record, sorts per family by source number ascending (from the stem, cross-checked against the source front-matter `id`), and allocates consecutive target numbers per family starting from `max(existing target number in active, archived and deprecated locations) + 1`, minimum 1 (ADR-0010 section 5). Milestones are allocated in the Papyrus milestone space starting at 1; the source `m-0` base carries no offset.
2. Ties, duplicate source numbers, and unparseable source numbers are ordered by source path and reported as source anomalies.
3. Phase 2 resolves every reference against the completed plan, and validates statuses, timestamps, and destinations.
4. Phase 3 writes. The plan is computed once, so a failed write can never shift a later number.
5. The plan is printed by both a dry run and a real import. It is the only record of the source→Papyrus mapping: no mapping file, sidecar, marker key, or extra front-matter field is written, because that would be the parallel ID space NFR-04.004 and REQ-01.005 forbid. The source ID is not persisted in the imported record.

### 4. Field mapping and content preservation

| Target front-matter key | Source | Rule |
|---|---|---|
| `id` | source `id` / stem | Replaced by the allocated Papyrus ID; the source ID is not kept |
| `title` | `title` | Copied verbatim; required — a record without it is skipped and reported |
| `status` | `status` | Compared to the target's configured statuses after normalization (case-insensitive, spaces/hyphens/underscores removed). A match adopts the configured spelling; no match preserves the source value verbatim and raises a warning (never silently coerced) |
| `assignee` | `assignee` | Copied as a YAML list |
| `created_date`, `updated_date` | same keys | Copied verbatim when a valid quoted ISO-8601 UTC value; otherwise set from the import clock and reported as approximate |
| `labels` | `labels` | Copied |
| `milestone` | `milestone` (source milestone ID) | Translated through the plan to the imported milestone ID; dropped with a warning when the source milestone is not in the import set |
| `dependencies` | `dependencies` (source task IDs) | Each entry translated through the plan; entries that resolve to no imported or existing target record are dropped with a warning. No source ID may survive |
| `references` | `references` | Copied verbatim (they name documents, not backlog records) |
| `documentation` | `documentation` | Copied verbatim |
| `type` | `type` | Copied for tasks (`task`) |
| `ordinal` | `ordinal` | Copied when a non-negative integer; otherwise allocated from the target maximum |
| other source keys | any remaining key (`priority`, …) | Preserved verbatim, so no metadata is lost and no source key-set validation exists |
| `release` (milestones) | none | Not written. Backlog.md has no release and Papyrus does not infer one from milestone names |

1. Timestamps are normalized to the quoted ISO-8601 UTC form of REQ-05.010; a source value in another format is replaced by the import time and reported.
2. Reading a Backlog.md file must reuse the shared front-matter/document parser and writing must reuse `record::store::write_atomic`; a Backlog-specific parser or serializer is prohibited (ADR-0013 downstream constraint 2).

### 5. Body mapping

Backlog.md marker sections are converted to the record's native section layout, preserving content:

| Source marker | Target section |
|---|---|
| `SECTION:DESCRIPTION` | `## Description` |
| `AC` (numbered `- [ ] #N …` items) | `## Acceptance Criteria` — the `#N` ordinal is dropped, checkbox state and item text are preserved verbatim and order is unchanged |
| `DOD` | `## Definition of Done` |
| `SECTION:PLAN` | `## Implementation Notes` |
| `SECTION:NOTES` | `## Notes` |
| `COMMENTS` | `## Comments` |

1. Marker comment lines are removed.
2. Unknown sections, free text, and any content outside a recognized marker block are appended verbatim in file order, so nothing is discarded.
3. Milestone bodies keep a single `## Description` with the source text verbatim.
4. Tasks acquire no `# <ID> <title>` heading (REQ-05.010).
5. Archived source records are written to the archived destination directory.

### 6. Status normalization

1. The source `status` is compared to the target's configured statuses after normalization (case-insensitive, spaces/hyphens/underscores removed).
2. A match adopts the configured spelling.
3. No match preserves the source value verbatim and raises a warning; the value is never silently coerced.

### 7. Collision and duplicate handling

| Situation | Outcome |
|---|---|
| Target already holds a record with the same number | Cannot occur through allocation; if the computed destination path exists at write time, that record fails with a conflict, nothing is overwritten, and the import aborts |
| Target already holds a record with the same title | Not a collision: the record is imported as a new record and the report flags "possible duplicate: `<new ID>` and `<existing ID>` share the title". No merge, no skip, no renumber |
| Target store already holds records of an imported family and `--allow-existing` is absent | Refused before writing, with a message telling the user to use `--allow-existing` if the duplication is intended |
| Two source files share a source number | Both are imported as distinct records; the report flags the source anomaly |
| Source record is unparseable (no title, malformed block) | Skipped, reported, exit non-zero; other records still import |
| Destination write fails mid-import | The import aborts and reports exactly which records were written; already-written records are kept (no rollback), and the user resolves the cause and continues in a fresh target or with `--allow-existing` |

1. The import never overwrites, deletes, renumbers, or merges an existing Papyrus record.
2. Archived source records are written to the archived destination directory.

### 8. One-way and non-idempotent by design

1. The import is a one-time bridge per source store. A second run against the same source allocates new numbers and therefore duplicates records; this is documented, and `--dry-run` plus the same-title report are the review mechanisms.
2. No marker, lock file, import registry, or source-state database is created.
3. Nothing in the import is bidirectional: the source is opened read-only, and Papyrus never regenerates Backlog.md files afterwards (NFR-04.004).
4. The source is left byte-identical after import.

### 9. Security and containment

1. Source paths are resolved and checked for existence and readability.
2. Destination paths go through `Context::ensure_within` before any write (NFR-04.002).
3. Symlinked records whose real path escapes either root are refused.
4. No source content is echoed into logs beyond the reported titles and paths.

### 10. Explicitly out of scope

1. Importing `docs`, `drafts`, or `decisions`.
2. Importing Backlog.md configuration as Papyrus configuration.
3. Release records (none exist upstream).
4. Issue/custom fields beyond front-matter copying.
5. Re-import reconciliation.
6. Continuous or scheduled sync.
7. Any reverse export.

### 11. Upgrade-notes contract

The release that introduces the native layout must ship the import command in the same release and must document a one-time import in its upgrade notes, including:

1. The affected source directories: `backlog/tasks`, `backlog/milestones`, `backlog/archive/tasks`, `backlog/archive/milestones`.
2. The command to run: `papyrus import backlog --from <source-project-dir>`.
3. That task IDs and milestone numbers are reallocated (ADR-0010 section 5): imported milestones are remapped into the Papyrus milestone space and source numbers are not preserved.
4. That the source store must be kept until the import is verified.
5. That after import Papyrus owns the records.
6. The published notes live at `docs/release-1.0-upgrade-notes.md` and are linked from the README section "Upgrading to Release 1.0".

### 12. README command documentation

The README command documentation (man page) must carry:

1. The import command with its options.
2. The one-way caveat (one-time bridge; re-running duplicates records).
3. The excluded-scope statement (`docs`, `drafts`, `decisions` not imported).
4. No claim of Backlog.md file compatibility.

## Inputs and outputs

- Input: a source project directory (and optional source backlog directory), target project context, and the import options.
- Output: imported Papyrus records under the configured directories, a printed plan/report, optional report file, exit code per REQ-01.004.
- No persistent mapping or registry is produced.

## Assumptions

- Assumption: the import is a CLI subcommand in the single crate; any wrapper script contains no logic and only forwards arguments (ADR-0011 decision 1).
- Assumption: the import may be delivered earlier, but not later than the layout switch, because the layout switch hides records that have not been imported (ADR-0010 decision 11, ADR-0013 stage constraint).
- Assumption: no new dependency is added (ADR-0005).
- Assumption: no UI design or mockup is required because the import is a CLI command with text output only.

## Open questions

- None. The import scope, ID assignment, collisions, and shipment coupling were confirmed by the user on 2026-11-11 (ADR-0011).

## Sources

- `docs/architecture/adr/ADR-0010-papyrus-native-on-disk-record-layout.md`
- `docs/architecture/adr/ADR-0011-one-way-backlog-md-import.md`
- `docs/architecture/adr/ADR-0012-papyrus-owned-front-matter-metadata.md`
- `docs/architecture/adr/ADR-0013-shared-record-envelope-without-a-storage-layer.md`
- `docs/requirements/business/REQ-01.002.md`
- `docs/requirements/business/REQ-01.003.md`
- `docs/requirements/business/REQ-01.004.md`
- `docs/requirements/business/REQ-01.005.md`
- `docs/requirements/non-functional/NFR-04.002.md`
- `docs/requirements/non-functional/NFR-04.004.md`

## Traceability

| Artifact | Reference |
|---|---|
| Ticket | TASK-83 (specification); TASK-92 (implementation); TASK-99 (upgrade notes) |
| Requirements | NFR-04.004, REQ-01.005, REQ-01.002, REQ-01.003, REQ-01.004, REQ-05.010, NFR-04.002 |
| ADRs | ADR-0010, ADR-0011, ADR-0012, ADR-0013 |
| Related tickets | TASK-88, TASK-89, TASK-90, TASK-92, TASK-97, TASK-98, TASK-99, TASK-100 |
| Source | src/import/*, src/record/*, src/task.rs, src/milestone.rs, src/config.rs, src/context.rs |
| UI mockups | none |
