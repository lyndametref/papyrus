---
id: ADR-0010
title: "Papyrus-Native On-Disk Record Layout and Path Convention"
created_date: '2026-11-11T00:00:00Z'
updated_date: '2026-11-16T21:08:35Z'
labels: []
references: []
---
# Papyrus-Native On-Disk Record Layout and Path Convention

## Status
superseded
## Date

2026-11-11 (status: Accepted, 2026-11-11; status: Superseded, 2026-11-13)

## Context

NFR-04.004 (adopted 2026-11-11) renounces file-level interoperability with Backlog.md and replaces it with a one-way import. NFR-04.003 is superseded. REQ-01.005 now defines a **single Papyrus ID space**: the Papyrus ID is the record identity in front matter, CLI and browser output, and command input, with no tool-specific storage ID and no translation. REQ-05.010 and REQ-05.011 require each managed Markdown record to carry YAML front matter as the single metadata store and to be adopted lazily per record. REQ-01.002 requires a configuration-driven folder change to move the whole folder atomically. REQ-01.003 forbids falling back to built-in defaults when no configuration file is found.

NFR-04.004 leaves the **native on-disk file layout and path convention** as an explicit open question, and so does REQ-01.005 ("the on-disk file-name convention is an architecture decision"). ADR-0008 currently answers it the other way: it makes the Backlog.md on-disk contract authoritative for task and milestone storage (`backlog/tasks/task-18 - slug.md`, `backlog/milestones/m-0 - slug.md`), derives a second storage identity, and translates numbers with a task offset of 0 and a milestone offset of +1. It cannot survive NFR-04.004 and is superseded by this ADR. ADR-0007's "Backlog.md key set" and "Backlog-compatible quoting" clauses are superseded by ADR-0012; ADR-0009's storage-identity stage is superseded by ADR-0013.

Observed state that the layout must account for:

- On-disk corpus: `backlog/tasks/task-18 - <slug>.md` … `task-73`, `backlog/milestones/m-0` … `m-14`, `backlog/archive/tasks/`, `backlog/archive/milestones/`, `backlog/docs/`, `backlog/drafts/`, `backlog/decisions/`, `backlog/config.yml`. Legacy Papyrus archives remain at `backlog/tasks/archived/` and `backlog/milestones/archived/`.
- Code: `src/record/identity.rs` implements bidirectional display/storage translation with storage prefixes and offsets; `src/record/descriptor.rs` declares fixed directories (`backlog/tasks`, `backlog/milestones`, `backlog/releases`, `docs/decisions`, `docs/requirements/...`); `src/task.rs` still resolves three task locations (`tasks/`, `tasks/archived/`, `archive/tasks/`) and three milestone locations.
- Configuration roots already exist: `backlog_directory`, `decision_directory`, `doc_directory`, and a `directory` per document type. `papyrus.config.yml` for this project sets `decision_directory: docs/architecture/adr`.
- Document records already follow `<ID>.md` (`REQ-01.001.md`, `NFR-04.004.md`, `GLO-001.md`, `TASK-73` front matter); decisions follow `<ID>-<kebab-slug>.md` (`ADR-0010-papyrus-native-on-disk-record-layout.md`) because the ADR corpus and external links depend on that name.

Forces: one identity, one file, one name, derived and not translated; paths fully configuration-driven; stable IDs across rename, archive, restore, deprecate; no renaming on title edits (browser sessions, in-flight edits and version tokens address a path); atomic, conflict-detecting folder moves; project-directory containment (NFR-04.002, NFR-04.001); and no read or write path that produces a Backlog.md shape.

## Decision

1. **One record, one file, one identity.** The Papyrus ID (REQ-01.005) is the record identity everywhere and is derivable from the file location. There is no storage ID, no second prefix, no number offset, and no translation layer in either direction. `record::identity` keeps only: parse and format a Papyrus ID under the configured prefix and `id_padding`, validate a reference value, and allocate the next number of a type.

2. **Directories are configuration-driven; descriptors name a configuration key, never a literal path.**

   | Record family | Active location | Inactive location | Idle/deprecated location |
   |---|---|---|---|
   | Task | `<backlog_directory>/tasks/` | `<backlog_directory>/archive/tasks/` | — |
   | Milestone | `<backlog_directory>/milestones/` | `<backlog_directory>/archive/milestones/` | — |
   | Release | `<backlog_directory>/releases/` | `<backlog_directory>/archive/releases/` | — |
   | Decision (ADR, BDR) | `<decision_directory>/` | — | — (deprecation is expressed by the `## Status` line in place; decision files are not moved) |
   | Document record (`req`, `nfr`, `glossary`, `rule`, `ui-spec`, `tech-spec`, `audit`, `deploy`, `use-case`, `test-case`, `test-report`) | `<doc_directory>/<type directory>/` | — | `<doc_directory>/<type directory>/deprecated/` |

   UI mockups remain HTML records under their configured `ui-mockup` directory and keep REQ-05.008 metadata; they are not Markdown records and this layout does not change them. `backlog_directory` and `decision_directory` are relative to the project directory; a document type's `directory` is relative to `doc_directory`. Every resolved path passes `Context::ensure_within` before any read or write (NFR-04.002).

3. **File name.**

   - Default: `<ID>.md`, where `<ID>` is the full Papyrus ID including the configured prefix and padding (`TASK-001.md`, `MS-001.md`, `REL-001.md`, `REQ-01.001.md`, `NFR-04.004.md`, `GLO-001.md`, `RUL-016.md`, `AUD-001.md`).
   - Decisions keep a descriptive suffix because the ADR corpus convention and its external links require it: `<ID>-<kebab-case-slug>.md` (`ADR-0010-papyrus-native-on-disk-record-layout.md`). The slug is fixed at creation and is never regenerated; the ADR skill's `ADR-NNNN-<kebab-case-title>.md` convention is preserved.
   - The slug is cosmetic. Resolution never uses it. No writer renames a file when a title changes. Document records that already carry a slug (for example this repository's `BR-016-<slug>.md`) keep their names when read and written; new records of those types use `<ID>.md` unless a later decision gives that type its own suffix rule.

4. **Resolution (read tolerance), never used to write.**

   - A file inside a claimed directory whose stem is `<PREFIX>-<digits>` optionally followed by `-<slug>` or ` - <slug>` is a candidate record. The prefix is compared case-insensitively against the configured prefix; leading zeros are optional, so `TASK-1.md`, `TASK-001.md` and `TASK-0001.md` all denote `TASK-001`.
   - Backlog.md storage stems are **not** recognized. `task-18`, `m-0` and any other Backlog storage prefix match no Papyrus record, so an unimported Backlog store is invisible to every Papyrus surface. This is the enforcement of "the import is the only bridge" (ADR-0011).
   - Front-matter `id` is authoritative. If it disagrees with the stem after normalization, the record is read from front matter, is flagged invalid, and writes to it are refused until the file is fixed.
   - Legacy archive aliases `<backlog_directory>/tasks/archived/` and `<backlog_directory>/milestones/archived/` are read as archived tasks and milestones. A read never relocates or renames them; archive/restore operations follow decision 7.

5. **Numbering and immutability.**

   - Each record family has one number space: tasks, milestones, releases, and each decision prefix and document prefix separately. **Milestones start at 1 (`MS-001`); the Backlog `m-0` offset is gone (confirmed by the user on 2026-11-11).**
   - **Milestone renumbering is accepted as a user-visible consequence (confirmed 2026-11-11).** A milestone stored as `m-N` is remapped into the Papyrus milestone space by allocation (ADR-0011 decision 4), for example `m-0` → `MS-001`-style IDs; the source `m-` number is never preserved, and the new number is what appears in front matter, the file name, references, the CLI, and the browser.
   - Allocation is `max(number present in every active, archived and deprecated location of that family) + 1`, minimum 1. A number is allocated once, at creation, and never recomputed.
   - Deleting the highest-numbered record makes that number available again; existing records are never renumbered. Archive, restore, deprecate, restore, supersede, and title edits never change an ID or a file name.
   - A configuration change to a prefix or `id_padding` affects only records created afterwards. Existing IDs, file names and reference values are untouched, and padding remains a minimum (REQ-01.005, REQ-01.007).

6. **No shape conversion, no renaming on read or write.**

   - No path converts a record to a Backlog.md on-disk shape, writes a Backlog storage ID, or renames a file into another naming convention.
   - Lazy adoption (REQ-05.011, ADR-0012) adds a front-matter block and may retire legacy metadata sections; it does not move, rename, or re-shape the file.
   - A record that is not in the native layout stays where it is; the one-way import (ADR-0011) is the supported way to bring a Backlog.md store into the native layout.

7. **Archive, restore, deprecate, restore are moves, not rewrites.**

   - An archive moves the file from the active directory to the archive directory of its family and an unarchive moves it back, keeping the file name and the ID.
   - Deprecate and restore do the same between the active and `deprecated/` directories of a **document** type (REQ-05.007). Decision records are never moved on deprecation: their state is the `## Status` line, and keeping them in the decision directory keeps them in the ADR index.
   - A move inside the project uses a same-directory-or-same-filesystem rename where available, otherwise the ADR-0006 atomic write sequence (write the destination with `create_new`, `sync_all`, then remove the source). A failed move leaves the record readable at exactly one location and reports the error; it never duplicates a record silently. If both locations hold the file, the record is flagged invalid and writes are refused.

8. **A configured folder change moves the whole folder during `config set` (REQ-01.002).** `backlog_directory`, `decision_directory`, `doc_directory`, and a document type's `directory` are handled by one shared relocation routine:
   - it runs inside the `config set` command, never lazily and never on a later read;
   - it moves every file and subfolder of the source folder, not only managed records;
   - it pre-checks the whole plan (destination entry conflicts, containment, permissions) before moving anything;
   - on any failure it moves nothing (or rolls back what it already moved), persists no configuration value, and reports the conflict outcome of REQ-01.004;
   - after success the source folder is removed when empty.
   The browser configuration view (REQ-06.008) must call this same `config set` path; it may not move folders itself.

9. **Configuration is the only source of paths and statuses.** `papyrus.config.yml` is authoritative for prefixes, padding, roots, directories and the status vocabulary. The Backlog.md `backlog/config.yml` is no longer read (dropping ADR-0008 decision 7): statuses come only from Papyrus configuration, and a record whose status is not in the configured list is preserved verbatim and reported, never silently normalized.

10. **The import writes native records only.** Imported tasks and milestones are created with `<ID>.md` under the configured directories, with a Papyrus ID and Papyrus front matter; the import contract is ADR-0011. After import, the source Backlog.md store is not read again and no compatibility is guaranteed (NFR-04.004).

11. **Switchover rule for a project whose store is still in the Backlog.md layout (confirmed by the user on 2026-11-11).** Because decision 4 stops recognizing Backlog stems, such records become invisible the moment this layout ships. A project in that state (including this repository, whose backlog is `backlog/tasks/task-N - …`) must run the one-way import before using the new version. The release that introduces this layout **must** ship the import command in the same release (ADR-0013 stage constraint) and **must** document a one-time import in its upgrade notes, naming the affected directories (`backlog/tasks`, `backlog/milestones`, `backlog/archive/tasks`, `backlog/archive/milestones`) and the resulting renumbering. No transitional read compatibility, environment flag, or dual-recognition mode is introduced: a temporary reader for a layout the product has renounced would be exactly the compatibility layer NFR-04.004 removes.

## Confirmed decisions (user validation 2026-11-11)

- Milestone renumbering (`m-N` → `MS-001`-style Papyrus IDs, imported milestones remapped by allocation) is accepted (decision 5).
- Switchover: the release that introduces this layout also ships the import and documents the one-time import in its upgrade notes (decision 11).
- The native layout, the single ID space, and the configuration-driven path convention are accepted as recorded.

## Consequences

**Positive:**

- The identity story collapses to one value and one rule; no affine mapping, offsets, storage prefixes, or normalization layer remains to be tested, documented, and explained.
- A record is addressable from its file name, and its file name is stable for its whole life, so browser URLs, version tokens, and links do not break on edits, configuration changes, or archiving.
- Paths become fully configuration-driven and testable through one relocation routine, which is also what REQ-01.002 needs.
- Removing Backlog.md layout recognition means an unimported store cannot be half-read; the import is the single, explicit bridge.
- The existing document corpus (`<ID>.md`, decisions `<ID>-<slug>.md`) already matches the decision, so document records need no renaming.

**Negative:**

- This is a breaking on-disk change for a project whose backlog is in the Backlog.md layout: tasks and milestones must be imported once, and their Papyrus IDs are newly allocated, so displayed numbers differ from the Backlog numbers.
- Milestone numbering changes from `m-0`-based to `MS-001`-based, and imported milestones are remapped rather than preserved (`m-0` becomes an `MS-` ID allocated in the Papyrus space). This is a confirmed, accepted consequence (2026-11-11): a project used to `MS-001` = `m-0` will see different milestone numbers after import.
- Existing specs, tests and documentation that describe storage IDs, offsets, slugs-as-identity, or `backlog/config.yml` precedence become wrong and must be revised.
- A record that is deleted loses its number only until it is reused; users who expect monotonic, never-reused IDs may be surprised. (Stated, accepted: existing records are never renumbered.)

**Risks and mitigations:**

- *Legacy files that are neither native nor importable*: they are read through the tolerant stem rules when their stem matches the Papyrus prefix; a Backlog-shaped file is intentionally invisible and must be imported. A diagnostic for "a file exists in a claimed directory but does not resolve" is available to the import dry run, not to normal listing.
- *Both archive locations holding the same record*: detected as a duplicate, flagged invalid, writes refused, neither copy deleted (same rule as ADR-0008 decision 9, retained).
- *Rename hazards*: no rename path exists outside archive/restore, deprecate/restore, and the two location moves; any future rename needs a new decision because it invalidates version tokens and browser URLs.
- *Folder moves crossing filesystems*: pre-flight check plus rollback; a cross-device move falls back to the atomic copy-then-remove sequence and is reported as a non-atomic operation in the command output if a single rename is impossible.
- *Case-insensitive filesystems*: prefix matching is case-insensitive but the writer always emits the configured casing, so two files differing only in prefix case cannot be produced by Papyrus; a pre-existing pair is a duplicate conflict.
- *Configuration pointing outside the project*: rejected by validation at load time, not only at access time (NFR-04.002).
- *Orphaned old directories*: the model does not delete empty directories except after a successful relocation; leftover legacy archive folders remain readable and are not migrated.
- *This repository's own store disappearing on upgrade*: the switchover rule (decision 11) requires importing `backlog/tasks` and `backlog/milestones` once, in the same release that ships this layout, with upgrade notes naming it; a test project fixture exercises the import before the old names are removed from the tree.

## Downstream constraints

Implementation and specification tasks that touch record storage MUST follow:

1. `record::identity` exposes Papyrus ID parse/format/allocation only. Remove storage prefixes, offsets, `to_storage_id`, `to_display_id`, and every translation call site (`src/record/identity.rs`, `src/task.rs`, `src/milestone.rs`, `src/browser/*`, migration code).
2. Record descriptors reference configuration keys for directories; no module hardcodes `backlog/tasks`, `backlog/milestones`, `docs/decisions`, or `archive/tasks`.
3. Writers emit `<ID>.md` (decisions `<ID>-<slug>.md`) and never rename an existing file. No code path may construct a Backlog.md stem.
4. Number allocation scans every location of a family, applies no offset, and starts milestones at 1; removal of the `m-0` base changes `next_task_number`/milestone allocation tests.
5. Reading `backlog/config.yml` is deleted together with its precedence rules and test fixtures.
6. Archive/restore/deprecate/restore are moves of an unchanged file name; tests assert the ID and file name are identical after the round trip.
7. `config set` folder relocation is implemented once, used by CLI and browser, with the pre-flight/rollback semantics of decision 8 and tests for conflict, partial failure, and cross-device fallback.
8. Every path is checked with `Context::ensure_within`; configuration validation rejects absolute or traversal paths.
9. Specifications that describe the Backlog.md layout, storage IDs, the `m-0` offset, or slug-based identity must be rewritten; `docs/specifications/migrate-tasks-and-milestones-to-the-backlog-md-layout.md`, `docs/specifications/translate-papyrus-and-backlog-md-ids-at-the-storage-boundary.md`, and `docs/specifications/verify-backlog-md-round-trip-interoperability.md` are directly superseded in intent.

## Alternatives Considered

| Option | Outcome | Reason |
|---|---|---|
| Native `<ID>.md` layout, configuration-driven roots, no translation (this ADR) | Selected | Only layout consistent with one ID space (REQ-01.005) and no Backlog.md compatibility (NFR-04.004); simplest identity and strongest stability. |
| Keep ADR-0008's Backlog.md layout without the ID translation (store Papyrus IDs in Backlog-shaped stems) | Rejected | A Backlog-shaped stem cannot carry `TASK-001` without an offset/lowercase mapping, so it either reintroduces a second form or breaks Backlog.md for no benefit now that compatibility is renounced. |
| Keep the Backlog.md layout and IDs as-is, importing nothing | Rejected | Contradicts REQ-01.005 and NFR-04.004; `task-18` would remain the identity and no Papyrus ID would exist. |
| `<ID> - <slug>.md` for every record | Rejected | Slugs go stale on title edits, invite renames, and add a second naming surface with no benefit; the slug is not identity. |
| Nested per-type directories under a single `backlog/` root, e.g. `backlog/tasks/archive/` | Rejected | Either convention works; keeping a top-level `archive/<family>/` preserves the existing archive layout and avoids nesting moves inside the active scan directory. |
| Keep `decision_directory` out of the layout and hardcode ADR paths | Rejected | Paths must come from configuration; the ADR skill's `docs/architecture/adr/` requirement is met by the configured value. |
| Move the whole project to a flat store (`records/`) | Rejected | Loses the per-family, per-type directories that the browser, search, and configuration already use. |

## References

- `docs/requirements/non-functional/NFR-04.004.md` — renounce Backlog.md compatibility; native layout is an explicit open question.
- `docs/requirements/non-functional/NFR-04.002.md` — project-directory boundary.
- `docs/requirements/non-functional/NFR-04.001.md` — text-file persistence.
- `docs/requirements/business/REQ-01.005.md` — single Papyrus ID space.
- `docs/requirements/business/REQ-01.002.md` — configuration folder change moves the whole folder atomically.
- `docs/requirements/business/REQ-01.007.md` — `id_padding` 1–6, padding is a minimum.
- `docs/requirements/business/REQ-05.010.md`, `REQ-05.011.md` — front matter and lazy adoption.
- `docs/requirements/business/REQ-06.008.md` — browser configuration view delegates to `config set`.
- `docs/architecture/adr/ADR-0008-adopt-backlog-md-on-disk-layout-for-tasks-and-milestones.md` — superseded by this ADR.
- `docs/architecture/adr/ADR-0011-one-way-backlog-md-import.md` — the only bridge from a Backlog.md store.
- `docs/architecture/adr/ADR-0012-papyrus-owned-front-matter-metadata.md` — metadata representation.
- `docs/architecture/adr/ADR-0013-shared-record-envelope-without-a-storage-layer.md` — code structure and write path.
- `docs/architecture/adr/ADR-0006-optimistic-concurrency-and-atomic-record-writes.md` — atomic write sequence reused by moves.
- Implementation: `src/record/identity.rs`, `src/record/descriptor.rs`, `src/record/store.rs`, `src/task.rs`, `src/milestone.rs`, `src/config.rs`, `src/context.rs`.
