---
id: ADR-0011
title: "One-Way Backlog.md Import as the Only Bridge"
created_date: '2026-11-11T00:00:00Z'
updated_date: '2026-11-15T15:47:30Z'
labels: []
references: []
---
# One-Way Backlog.md Import as the Only Bridge

## Status

Accepted — approved by explicit user confirmation on 2026-11-11

## Date

2026-11-11 (status: Accepted, 2026-11-11)

## Context

NFR-04.004 requires Papyrus to stop maintaining file-level interoperability with Backlog.md and, instead, to offer a one-way script that imports Backlog.md backlog records. Scope is fixed by the requirement: tasks and milestones are imported; Backlog.md `docs`, `drafts` and `decisions` are out of scope. Imported records receive Papyrus IDs (REQ-01.005) and are written in the Papyrus record format with the front matter of REQ-05.010. REQ-05.011 confirms that no read or write path may adopt a Backlog.md storage ID or re-shape a record into the Backlog.md layout, so the import is the only entry point for such data.

NFR-04.004 records two open questions that this ADR must answer: **collision handling when an existing Papyrus record has the same number or title**, and implicitly the contractual behaviour of the import itself (what it reads, what it writes, how it reports). ADR-0010 fixes the destination: native `<ID>.md` records with a single Papyrus ID space, `<backlog_directory>/archive/<family>/` for archived records, and no recognition of Backlog storage stems. ADR-0013 fixes that every write goes through the shared record writer.

Forces: one-way and explicit (no sync, no watcher, no bidirectional expectation); no second ID space and no persisted ID mapping; never destructive on either side; deterministic, reviewable allocation; partial metadata (Backlog has no release, no Papyrus `release` key on milestones); status vocabularies differ (`To Do` versus `ToDo`); a single write path with atomic writes; and NFR-04.002 containment for both the source read and the destination write.

## Decision

1. **Delivery shape (confirmed by the user on 2026-11-11).** The import is the dedicated CLI command `papyrus import backlog`, implemented inside the single crate (ADR-0004, ADR-0005) and using the shared record reader, descriptor, and writer (ADR-0013). No second parser and no second writer may exist for Backlog.md files. An optional convenience shell entry point (for example `scripts/import-backlog.sh`) MAY be provided, but it must only forward arguments to the CLI command and contain no import logic. This satisfies NFR-04.004's "one-way script" without creating an independent writer.

2. **Invocation and inputs.**

   ```
   papyrus import backlog --from <source-project-dir> [--source-backlog-dir <dir>]
                          [--dry-run] [--allow-existing] [--report <path>]
   ```

   - The target project is selected per REQ-01.003 (`--dir`, `PAPYRUS_DIR`, cwd) and must have a `papyrus.config.yml`; otherwise the command fails as a runtime error (REQ-01.003, REQ-01.004).
   - `--from` names the Backlog.md project root; `--source-backlog-dir` defaults to `<from>/backlog` for projects that do not use the default folder name.
   - `--dry-run` performs phase 1 and phase 2 fully and prints the plan and report without writing anything.
   - `--allow-existing` is required when the target already holds any record of a family being imported; without it the command refuses before writing (see decision 7).
   - `--report <path>` writes the same report to a path inside the project directory only; a path outside is a validation error (NFR-04.002). The report is diagnostic output, never a machine-readable store that later commands consult.
   - Exit codes follow REQ-01.004: `0` when every in-scope source record was imported (or planned, for a dry run); `1` runtime/validation failure; import with skipped records exits non-zero and the summary lists them.

3. **Source scope.**

   | Source | Treatment |
   |---|---|
   | `backlog/tasks/*.md` | Imported as tasks |
   | `backlog/archive/tasks/*.md` | Imported as archived tasks |
   | `backlog/milestones/*.md` | Imported as milestones |
   | `backlog/archive/milestones/*.md` | Imported as archived milestones |
   | `backlog/docs/**`, `backlog/drafts/**`, `backlog/archive/drafts/**`, `backlog/decisions/**` | Not read as records; counted and reported as excluded by NFR-04.004 |
   | `backlog/config.yml`, `backlog/.locks/**` | Source metadata only; never written; the source status vocabulary and prefixes may be read to interpret the source |

   Only `.md` files whose stem matches the source family's storage form (`<prefix>-<number>`, optionally followed by ` - <slug>`) are candidates. Anything else in a source directory is reported as skipped. Files outside the source backlog directory and outside `backlog/archive/` are never read.

4. **ID assignment: fresh Papyrus numbers, allocated before any write.**

   - Phase 1 (plan) enumerates every in-scope source record, sorts per family by source number ascending (from the stem, cross-checked against the source front-matter `id`), and allocates consecutive target numbers per family starting from `max(existing target number in active, archived and deprecated locations) + 1`, minimum 1 (ADR-0010 decision 5). Milestones are allocated in the Papyrus milestone space starting at 1; the source `m-0` base carries no offset.
   - Ties, duplicate source numbers and unparseable source numbers are ordered by source path and reported as source anomalies.
   - Phase 2 resolves every reference against the completed plan, and validates statuses, timestamps and destinations.
   - Phase 3 writes. The plan is computed once, so a failed write can never shift a later number.
   - The plan is printed by both a dry run and a real import. It is the only record of the source→Papyrus mapping: **no mapping file, sidecar, marker key, or extra front-matter field is written**, because that would be the parallel ID space NFR-04.004 and REQ-01.005 forbid. The source ID is not persisted in the imported record.

5. **Field mapping and content preservation.**

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

   Timestamps are normalized to the quoted ISO-8601 UTC form of REQ-05.010; a source value in another format is replaced by the import time and reported.

6. **Body mapping.** Backlog.md marker sections are converted to the record's native section layout, preserving content:

   | Source marker | Target section |
   |---|---|
   | `SECTION:DESCRIPTION` | `## Description` |
   | `AC` (numbered `- [ ] #N …` items) | `## Acceptance Criteria` — the `#N` ordinal is dropped, checkbox state and item text are preserved verbatim and order is unchanged |
   | `DOD` | `## Definition of Done` |
   | `SECTION:PLAN` | `## Implementation Notes` |
   | `SECTION:NOTES` | `## Notes` |
   | `COMMENTS` | `## Comments` |

   Marker comment lines are removed. Unknown sections, free text, and any content outside a recognized marker block are appended verbatim in file order, so nothing is discarded. Milestone bodies keep a single `## Description` with the source text verbatim. Tasks acquire no `# <ID> <title>` heading (REQ-05.010). Archived source records are written to the archived destination directory.

7. **Collision and duplicate handling.**

   | Situation | Outcome |
   |---|---|
   | Target already holds a record with the same number | Cannot occur through allocation; if the computed destination path exists at write time, that record fails with a conflict, nothing is overwritten, and the import aborts |
   | Target already holds a record with the same title | Not a collision: the record is imported as a new record and the report flags "possible duplicate: `<new ID>` and `<existing ID>` share the title". No merge, no skip, no renumber |
   | Target store already holds records of an imported family and `--allow-existing` is absent | Refused before writing, with a message telling the user to use `--allow-existing` if the duplication is intended |
   | Two source files share a source number | Both are imported as distinct records; the report flags the source anomaly |
   | Source record is unparseable (no title, malformed block) | Skipped, reported, exit non-zero; other records still import |
   | Destination write fails mid-import | The import aborts and reports exactly which records were written; already-written records are kept (no rollback), and the user resolves the cause and continues in a fresh target or with `--allow-existing` |

   The import never overwrites, deletes, renumbers, or merges an existing Papyrus record.

8. **One-way and non-idempotent by design.** The import is a one-time bridge per source store. A second run against the same source allocates new numbers and therefore duplicates records; this is documented, and `--dry-run` plus the same-title report are the review mechanisms. No marker, lock file, import registry, or source-state database is created. Nothing in the import is bidirectional: the source is opened read-only, and Papyrus never regenerates Backlog.md files afterwards (NFR-04.004).

9. **Shipment coupling with the native layout (confirmed by the user on 2026-11-11).** The import command ships in the same release as ADR-0010's native layout, and that release's upgrade notes document the one-time import: the affected source directories, the command to run, that IDs and milestone numbers are reallocated (ADR-0010 decision 5), that the source store must be kept until the import is verified, and that after import Papyrus owns the records. The import may be delivered earlier, but it may not be delivered later than the layout switch, because the layout switch hides records that have not been imported (ADR-0010 decision 11, ADR-0013 stage constraint).

10. **Security and containment.** Source paths are resolved and checked for existence and readability; destination paths go through `Context::ensure_within` before any write (NFR-04.002). Symlinked records whose real path escapes either root are refused. No source content is echoed into logs beyond the reported titles and paths.

11. **Explicitly out of scope.** Importing `docs`, `drafts`, or `decisions`; importing Backlog.md configuration as Papyrus configuration; release records (none exist upstream); issue/custom fields beyond front-matter copying; re-import reconciliation; continuous or scheduled sync; any reverse export.

## Confirmed decisions (user validation 2026-11-11)

- Delivery shape: the dedicated `papyrus import backlog` subcommand is accepted; a wrapper script is optional and logic-free (decision 1).
- Milestone renumbering: imported milestones are remapped into the Papyrus milestone space (`m-N` → `MS-001`-style IDs) and source numbers are not preserved (ADR-0010 decision 5).
- Switchover: the import ships in the same release as the native layout and that release documents a one-time import in its upgrade notes (decision 9).
- Remaining scope decisions of NFR-04.004 stand unchanged: tasks and milestones only, `docs`/`drafts`/`decisions` excluded, single Papyrus ID space, no persisted mapping.

## Consequences

**Positive:**

- Projects holding a Backlog.md store have one documented, reviewable path into Papyrus, satisfying NFR-04.004 without keeping a compatibility layer.
- Fresh, planned allocation makes numeric collisions impossible and keeps existing Papyrus records untouched, which removes the hardest failure mode of an import.
- Content fidelity (front matter, sections, unknown keys, unknown sections) is defined and testable against a real corpus.
- Only the shared writer is used, so imported records are normal records for every later command, view, and search.

**Negative:**

- Imported IDs do not match the numbers users saw in Backlog.md (for example `task-18` may become `TASK-042`), and the mapping exists only in the printed report.
- Re-running the import duplicates records, and detecting that after the fact relies on the title report.
- Status values that do not match the target vocabulary survive verbatim, which can leave a project with two spellings until the user normalizes them.
- Dependencies and milestone references that cannot be resolved are dropped (with warnings), so the imported graph can be incomplete.
- A mid-import failure leaves a partial import with no rollback.
- Release coupling is mandatory: the native layout cannot ship before the import, so the two changes are delivered and verified together in one release.

**Risks and mitigations:**

- *Silently duplicated work*: same-title report plus the `--allow-existing` gate, and the one-way, one-time framing in the command documentation.
- *Lost identity for users who relied on Backlog numbers*: the plan report is printed for every run and can be saved with `--report`; no command ever accepts the old number, so there is no half-supported identifier.
- *Reference translation bugs*: translation happens after the whole plan exists, is pure, and is covered by tests where dependencies and milestones resolve, are missing, or point outside the import set.
- *Body conversion losing content*: unknown sections and free text are appended verbatim; the report records per-record warnings; a byte-level comparison of the source text against the imported body is a test fixture for each marker family.
- *Import into a wrong directory*: target must be an initialized project and every destination path is contained; the source is read-only and is never modified.
- *Large stores*: the plan holds one entry per in-scope record; phases 1–2 are read-only, so a dry run costs no writes.

## Downstream constraints

Implementation and specification tasks for the import MUST follow:

1. The import is a CLI subcommand in the single crate; any wrapper script contains no logic; no new dependency (ADR-0005).
2. Reading a Backlog.md file must reuse the shared front-matter/document parser and writing must reuse `record::store::write_atomic`; a Backlog-specific parser or serializer is prohibited.
3. The plan phase must allocate all IDs before any write, must not consult or create a mapping store, and must run unchanged under `--dry-run`.
4. IDs come from `record::identity` allocation with no offset; milestones start at 1.
5. No source storage ID may appear in a written record, in front matter, in a filename, or as an accepted later input.
6. Status mapping is normalization-plus-report; unknown statuses are preserved, never coerced.
7. `docs`, `drafts` and `decisions` are never read as records; the report states they were excluded.
8. Tests must cover: active and archived tasks/milestones; dependency and milestone translation; missing/dangling references; status mismatch; duplicate source numbers; same-title target record; existing-target refusal; `--dry-run` producing no writes; and a source directory left byte-identical after import.
9. Command help and the man page gain the command, its options, the one-way/non-idempotent caveat, and the excluded-scope statement; the specification is owned by `business-analyst`/`tech-writer`.

## Alternatives Considered

| Option | Outcome | Reason |
|---|---|---|
| Planned fresh Papyrus IDs, no persisted mapping, shared writer (this ADR) | Selected | Single ID space, no collisions, no second store, and imported records become ordinary records. |
| Preserve source numbers as Papyrus numbers (offset only by prefix/padding) | Rejected | A number reused from a Backlog store can collide with an existing Papyrus record, and it implies adopting the source ID as identity, which REQ-01.005 forbids. |
| Persist a source→Papyrus mapping file or front-matter key | Rejected | It is a parallel ID space and a second store that can drift; NFR-04.004 and REQ-01.005 exclude it. |
| Skip records whose title already exists in the target | Rejected | Silently discarding a source record loses data; the requirement asks for collision handling, not deletion. Merge or overwrite would be worse. |
| Standalone external script with its own parser/writer | Rejected | A second writer with untested preservation behaviour, outside the no-drift rule (ADR-0013). |
| Import `docs`/`drafts`/`decisions` as generic documents | Rejected | NFR-04.004 excludes them; mapping them would invent record types and metadata. |
| Bidirectional sync or re-import reconciliation | Rejected | NFR-04.004 makes the import explicit and one-way; compatibility is renounced. |

## References

- `docs/requirements/non-functional/NFR-04.004.md` — renounce compatibility, provide a one-way import; open questions answered here.
- `docs/requirements/business/REQ-01.005.md` — single Papyrus ID space; imported records receive Papyrus IDs.
- `docs/requirements/business/REQ-05.010.md` — front-matter fields and record format for imported records.
- `docs/requirements/business/REQ-05.011.md` — no path adopts a Backlog storage ID or re-lays out a record.
- `docs/requirements/business/REQ-01.002.md`, `REQ-01.003.md`, `REQ-01.004.md` — configuration selection, runtime errors, conflict outcomes.
- `docs/requirements/non-functional/NFR-04.002.md` — project-directory containment.
- `docs/architecture/adr/ADR-0010-papyrus-native-on-disk-record-layout.md` — destination layout, ID allocation, no Backlog stems.
- `docs/architecture/adr/ADR-0012-papyrus-owned-front-matter-metadata.md` — target record format.
- `docs/architecture/adr/ADR-0013-shared-record-envelope-without-a-storage-layer.md` — one writer, no second serializer.
- Source evidence: `backlog/config.yml`, `backlog/tasks/task-18 - … .md`, `backlog/milestones/m-0 - … .md`, `backlog/archive/`, `backlog/docs/doc-1 - … .md`.
