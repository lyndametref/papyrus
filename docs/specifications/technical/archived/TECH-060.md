---
id: TECH-060
title: "Papyrus-Native Record Layout and Single ID Space"
created_date: '2026-11-16T17:46:52Z'
updated_date: '2026-11-18T22:50:56Z'
labels: []
references:
  - ADR-0006
  - ADR-0008
  - ADR-0010
  - ADR-0011
  - ADR-0012
  - ADR-0013
  - BR-007
  - BR-008
  - BR-016
  - GLO-001
  - NFR-004.002
  - NFR-004.004
  - REQ-001.001
  - REQ-001.002
  - REQ-001.004
  - REQ-001.005
  - REQ-001.007
  - REQ-003.001
  - REQ-004.002
  - REQ-005.007
  - REQ-005.008
  - REQ-005.010
  - REQ-005.011
  - REQ-006.008
  - TASK-0001
  - TASK-001
  - TASK-1
  - TASK-100
  - TASK-83
  - TASK-88
  - TASK-89
  - TASK-90
  - TASK-92
  - TASK-98
  - ADR-034
  - REQ-006.018
  - RE-005
status: archived
---
# TECH-060 Papyrus-Native Record Layout and Single ID Space

> **Revised (TASK-203, 2026-11-18):** section 8 item 7, which required the browser configuration view to reach the `config set` relocation path, is retired per ADR-034 and REQ-006.018 (RE-005): the browser serves no configuration view and configuration editing is CLI-only. The CLI `config set` whole-folder relocation clauses (section 8 items 1-6, REQ-001.002) remain current. REQ-006.008, which the retired item cites, is archived; the retired text is retained below as history. See TECH-080.

## Description
## Purpose

Define the Papyrus-native on-disk record layout and the single Papyrus ID space that replace the Backlog.md layout of ADR-0008. This specification implements ADR-0010, REQ-001.005, and REQ-001.002 for every managed record type: configuration-driven directories, one file per record, one Papyrus ID as the record identity, tolerant reads, allocation without offsets, moves for archive/deprecate, and the intentional invisibility of Backlog.md stems. It is the destination layout the one-way import (see `import-backlog-md-records.md`) writes into and the layout every later implementation task (TASK-88 … TASK-98) must honour.

## Scope

### In scope

- Configuration-driven directory resolution for every record family (task, milestone, release, decision, document types).
- Native file names: `<ID>.md` by default; decisions `<ID>-<kebab-slug>.md`; cosmetic slugs never used for resolution.
- Tolerant read-stem resolution: `<PREFIX>-<digits>` optionally followed by `-<slug>` or ` - <slug>`, case-insensitive prefix, optional leading zeros; legacy archive aliases.
- Front-matter `id` as the authoritative record identity; disagreement with the stem is a validation error that refuses writes.
- Number allocation without offsets: `max(observed) + 1` over every active, archived, and deprecated location of a family; minimum 1; milestones start at 1.
- Archive, restore, deprecate, restore as file moves that keep the file name and the ID.
- Family-specific lifecycle behavior: tasks and milestones archive into the canonical archive directories, decisions archive into `<decision_directory>/archived/`, document records deprecate into `deprecated/`, and releases are never archived.
- Atomic whole-folder relocation during `config set` (REQ-001.002).
- The rule that Backlog.md storage stems (`task-18`, `m-0`, …) match no Papyrus record, so an unimported Backlog.md store is invisible to every Papyrus surface.

### Out of scope

- The one-way import itself — see `import-backlog-md-records.md`.
- Front-matter field map and patching — ADR-0012 / `migrate-document-record-types-to-front-matter-lazily-on-first-read-and-first-write.md`.
- UI mockups (HTML records) — excluded from this layout's Markdown rules.
- Corpus-wide migration or renaming of existing records, including existing unslugged decision files and slugged document files.
- Adding lifecycle commands. The documented command set is unchanged: no release archive/restore, no milestone unarchive/restore, and no decision restore (BR-008, BR-007, REQ-003.001, `release-lifecycle.md`).

## Behavior

### 1. One record, one file, one identity

1. The Papyrus ID (REQ-001.005) is the record identity everywhere: in the front-matter `id`, in CLI and browser output, and as command input.
2. There is no storage ID, no second prefix, no number offset, and no translation layer in either direction.
3. `record::identity` keeps only: parse and format a Papyrus ID under the configured prefix and `id_padding`, validate a reference value, and allocate the next number of a type.
4. A record is addressable from its file name and its file name is stable for its whole life; title edits, archive/restore, deprecate/restore, and configuration changes never rename a file.

### 2. Configuration-driven directories

1. Descriptors name a configuration key, never a literal path. The configured roots are the only source of locations. A descriptor stores a key such as `backlog`, `decision`, or `doc` (plus, for a document type, the type entry), and the resolver appends only the fixed family subpath (`tasks/`, `milestones/`, `releases/`, `archive/...`, or the document type's configured `directory`); no descriptor or module may embed a full path literal such as `backlog/tasks`, `backlog/milestones`, `archive/tasks`, or `docs/decisions`, and no `concat!` (or equivalent) path construction may remain in `src/record/descriptor.rs`.
2. Directory layout:

   | Record family | Active | Inactive (archive) | Deprecated |
   |---|---|---|---|
   | Task | `<backlog_directory>/tasks/` | `<backlog_directory>/archive/tasks/` | — |
   | Milestone | `<backlog_directory>/milestones/` | `<backlog_directory>/archive/milestones/` | — |
   | Release | `<backlog_directory>/releases/` | — (no archive command; status `released`) | — |
   | Decision (ADR, BDR) | `<decision_directory>/` | `<decision_directory>/archived/` (explicit `papyrus decision archive` only) | in place via `## Status` (deprecation never moves the file) |
   | Document record (`req`, `nfr`, `glossary`, `rule`, `ui-spec`, `tech-spec`, `audit`, `deploy`, `use-case`, `test-case`, `test-report`) | `<doc_directory>/<type directory>/` | — (no archive command) | `<doc_directory>/<type directory>/deprecated/` |

   The `Inactive (archive)` column lists only locations an existing archive command writes. Releases are never archived (BR-007, `release-lifecycle.md`), and no `archive/releases/` path is read or written. Decision deprecation is the `## Status` line and never moves a file; the decision archive move is the separate documented `papyrus decision archive` command (BR-008, REQ-004.002). The `Inactive (archive)` destination is the only destination for a new archive in every family; the legacy alias directories of section 4.5 are read-only tolerances and are never written by an archive operation.

3. `backlog_directory` and `decision_directory` are relative to the project directory; a document type's `directory` is relative to `doc_directory`.
4. Every resolved path passes `Context::ensure_within` before any read or write (NFR-004.002).
5. UI mockups remain HTML records under their configured `ui-mockup` directory and keep REQ-005.008 metadata; they are not Markdown records and this layout does not change them.

### 3. File name

1. Default: `<ID>.md`, where `<ID>` is the full Papyrus ID including the configured prefix and padding (`TASK-001.md`, `MS-001.md`, `REL-001.md`, `REQ-001.001.md`, `NFR-004.004.md`, `GLO-001.md`, `RUL-016.md`, `AUD-001.md`).
2. Decisions keep a descriptive suffix because the ADR corpus convention and its external links require it: `<ID>-<kebab-case-slug>.md` (`ADR-0010-papyrus-native-on-disk-record-layout.md`). At creation the writer derives the slug from the created title (see the assumption on slug derivation) and writes that name. The slug is fixed at creation and is never regenerated.
3. The slug is cosmetic. Resolution never uses it: a decision is resolved by its `<ID>` part, and every CLI and browser surface reports the canonical ID (`ADR-0010`), never the slugged stem. No writer renames a file when a title changes. A decision file that already exists without a slug (for example one created before this convention) remains readable, editable, and archivable under its existing `<ID>.md` name and is never renamed.
4. Document records are named `<ID>.md`. A document file that already carries a slug after its full canonical ID (`BR-016-<slug>.md`) is read as `<ID>` and keeps its name when read and written; new records of those types use `<ID>.md` unless a later decision gives that type its own suffix rule. A trailing suffix is stripped only when the prefix plus ID matches the family's full ID shape exactly, so a dotted ID component (`REQ-001.001` versus `REQ-001.002`) is never mistaken for a slug.

### 4. Resolution (read tolerance)

1. A file inside a claimed directory whose stem is `<PREFIX>-<digits>` optionally followed by `-<slug>` or ` - <slug>` is a candidate record. Decision stems follow this shape: `<ID>-<kebab-slug>` or `<ID>`. Document families use their full ID shape (`REQ-001.001`, `NFR-004.004`, `GLO-001`), optionally followed by `-<slug>` or ` - <slug>` after that complete ID.
2. The prefix is compared case-insensitively against the configured prefix and leading zeros are optional, so `TASK-1.md`, `TASK-001.md`, and `TASK-0001.md` all denote `TASK-001`. Case-insensitive matching is retained for Papyrus prefixes (for example `Task-001` or `task-001` with a padded number).
3. Backlog.md storage stems are **not** recognized. A stem is Backlog-shaped, and resolves to no record, when its prefix is the all-lowercase form of a configured prefix that contains at least one uppercase letter **and** its number has no leading zeros (the Backlog storage shape, e.g. `task-18`, `m-0`). Such a stem is invisible even though it case-folds to the configured prefix. Any other Backlog storage prefix has the same effect. This enforces "the import is the only bridge" (ADR-0011). All other case variants, including padded lowercase forms such as `task-001`, resolve normally. When the configured prefix is itself all lowercase, its unpadded stems cannot be distinguished from Backlog storage stems and are treated as native (see the assumption on prefix casing).
4. Front-matter `id` is authoritative. If it disagrees with the stem's ID after normalization, the record is read from front matter, is flagged invalid, and writes to it are refused until the file is fixed. The normalized stem ID is the canonical ID part only, never the slug.
5. Legacy archive aliases `<backlog_directory>/tasks/archived/` and `<backlog_directory>/milestones/archived/` are read as archived tasks and milestones. A read never relocates or renames them, and an archive never writes into these directories. Archiving a record found there fails as already archived without writing, moving, or removing it; restoring a record found there is the only operation that moves it, to the active directory, keeping the file name and the ID (section 7).
6. Decisions are resolved by canonical ID across `<decision_directory>/` and `<decision_directory>/archived/`, regardless of whether the file is named `<ID>.md` or `<ID>-<slug>.md`. If both directories, or two files with the same ID and different slugs in one directory, hold the ID, the record is a duplicate: it is flagged invalid and writes are refused (section 7.4). Listing reads only the active directory.
7. Document records are resolved by canonical ID across their active directory and its `deprecated/` subdirectory; a slugged stem resolves to the ID before the first slug separator, and every surface reports that ID. A duplicate ID across the two directories is flagged invalid and writes are refused (section 7.4).

### 5. Numbering and immutability

1. Each record family has one number space: tasks, milestones, releases, and each decision prefix and document prefix separately.
2. **Milestones start at 1 (`MS-001`); the Backlog `m-0` offset is gone.** Milestone renumbering is accepted as a user-visible consequence: a milestone stored as `m-N` is remapped into the Papyrus milestone space by allocation (see `import-backlog-md-records.md`), and the source `m-` number is never preserved.
3. Allocation is `max(number present in every active, archived, archived/ and deprecated location of that family) + 1`, minimum 1. The scanned locations are: tasks and milestones — active, canonical archive, and the legacy alias; releases — active only; decisions — active and `archived/`; document records — active and `deprecated/`. A number is allocated once, at creation, and never recomputed.
4. Deleting the highest-numbered record makes that number available again; existing records are never renumbered.
5. Archive, restore, deprecate, restore, supersede, and title edits never change an ID or a file name.
6. A configuration change to a prefix or `id_padding` affects only records created afterwards. Existing IDs, file names, and reference values are untouched, and padding remains a minimum (REQ-001.005, REQ-001.007).

### 6. No shape conversion, no renaming on read or write

1. No path converts a record to a Backlog.md on-disk shape, writes a Backlog storage ID, or renames a file into another naming convention. No read or write path relocates or renames a record; the legacy conversion and relocation path in `src/record/migrate.rs` is removed (relocation by TASK-88, body conversion by TASK-89).
2. Lazy adoption (REQ-005.011, ADR-0012) adds a front-matter block and may retire legacy metadata sections; it does not move or rename the file and does not convert its body to the Backlog.md layout.
3. A record that is not in the native layout stays where it is; the one-way import (`import-backlog-md-records.md`) is the supported way to bring a Backlog.md store into the native layout.

### 7. Archive, restore, deprecate, restore are moves

1. An archive moves the file from the active directory to the archive directory of its family and keeps the file name and the ID. The family destinations are:
   - tasks → `<backlog_directory>/archive/tasks/`;
   - milestones → `<backlog_directory>/archive/milestones/`;
   - decisions → `<decision_directory>/archived/`, through the existing `papyrus decision archive` command (BR-008, REQ-004.002).
   Releases and document records have no archive command. No milestone unarchive/restore command exists (REQ-003.001), no decision restore command exists (BR-008), and this specification adds none; task restore moves a record from the canonical archive or either legacy alias to the active directory.
2. Deprecate and restore do the same between the active and `deprecated/` directories of a **document** type (REQ-005.007). Decision deprecation is not a move: it is the `## Status` line (`deprecated`, `rejected`, or `superseded`) in place, and keeping decisions in the decision directory keeps them in the ADR index. The decision archive move of section 7.1 is a separate, explicit command and does not by itself change `## Status`.
3. A move inside the project uses a same-directory-or-same-filesystem rename where available, otherwise the ADR-0006 atomic write sequence (write the destination with `create_new`, `sync_all`, then remove the source).
4. A failed move leaves the record readable at exactly one location and reports the error; it never duplicates a record silently. If more than one of the active, archive, `archived/`, legacy alias, and deprecated locations of a family hold the same canonical ID, the record is flagged invalid and writes are refused. Duplicate detection is keyed by canonical ID, not by file name.
5. Archiving a record that is already in its archive location or in a legacy alias directory is refused as already archived and performs no write, move, or removal. Only restore may move a record out of an archive or alias, and it targets the active directory.

### 8. Whole-folder relocation on `config set`

1. `backlog_directory`, `decision_directory`, `doc_directory`, and a document type's `directory` are handled by one shared relocation routine (REQ-001.002).
2. The routine runs inside the `config set` command, never lazily and never on a later read.
3. It moves every file and subfolder of the source folder, not only managed records.
4. It pre-checks the whole plan (destination entry conflicts, containment, permissions) before moving anything.
5. On any failure it moves nothing (or rolls back what it already moved), persists no configuration value, and reports the conflict outcome of REQ-001.004.
6. After success the source folder is removed when empty.
7. **Retired (TASK-203, 2026-11-18, ADR-034, REQ-006.018):** the browser configuration view (REQ-006.008, archived) must call this same `config set` path; it may not move folders itself. No browser configuration view exists, so the CLI `config set` command is the only caller of this routine; items 1-6 above remain current.

### 9. Configuration is the only source of paths and statuses

1. `papyrus.config.yml` is authoritative for prefixes, padding, roots, directories, and the status vocabulary.
2. The Backlog.md `backlog/config.yml` is no longer read (dropping ADR-0008 decision 7): statuses come only from Papyrus configuration.
3. A record whose status is not in the configured list is preserved verbatim and reported, never silently normalized.

### 10. Switchover rule

1. Because section 4.3 stops recognizing Backlog stems, records in the Backlog.md layout become invisible the moment this layout ships.
2. A project in that state (including this repository, whose backlog is `backlog/tasks/task-N - …`) must run the one-way import before using the new version.
3. The release that introduces this layout must ship the import command in the same release (ADR-0013 stage constraint) and must document a one-time import in its upgrade notes, naming the affected directories (`backlog/tasks`, `backlog/milestones`, `backlog/archive/tasks`, `backlog/archive/milestones`) and the resulting renumbering.
4. No transitional read compatibility, environment flag, or dual-recognition mode is introduced.

## Inputs and outputs

- Input: a project `Context` and a record path or descriptor-claimed location.
- Output: resolved record paths and IDs, allocation results, move results, or validation/conflict errors.
- No new user-facing command is added by this specification; it governs read/write path behavior.

## Assumptions

- Assumption: configuration roots already exist (`backlog_directory`, `decision_directory`, `doc_directory`, and a `directory` per document type) and are relative to the project directory.
- Assumption: `id_padding` is constrained to 1–6 and is a minimum width (REQ-001.007); existing IDs are never reformatted when padding changes.
- Assumption: a decision's kebab slug is derived at creation from the sanitized title: lowercase ASCII alphanumerics, every run of other characters collapsed to a single `-`, leading and trailing `-` removed. If the result is empty, the file is written as `<ID>.md`. The slug is derived once and never recomputed.
- Assumption: document records created after this layout use `<ID>.md`; document records that already carry `<ID>-<slug>.md` keep their names and are resolved by the ID part. No corpus rename is performed.
- Assumption: the existing `papyrus decision archive` command and its `<decision_directory>/archived/` directory remain part of the documented command set (BR-008, REQ-004.002). This layout changes the decision file name and resolution rules, not the existence of the command.
- Assumption: the Backlog-shaped exclusion of section 4.3 applies only when the configured prefix contains at least one uppercase letter. A project configured with an all-lowercase prefix is treated as Papyrus-native because Papyrus itself can emit an unpadded stem when `id_padding` is 1; such a project cannot distinguish a Backlog storage stem from its own record, and that limitation is accepted.
- Assumption: no UI design or mockup is required; this is a storage/path and ID-derivation specification whose only visible effect is the ID and path shown by existing CLI and browser record views.

## Open questions

- None. The layout, single ID space, and configuration-driven path convention were confirmed by the user on 2026-11-11 (ADR-0010).

## Revision notes

- 2026-11-12 (TASK-88 alignment): resolved four ambiguities left by the earlier refinement. (1) Decision file names follow ADR-0010 decision 3: creation writes `<ID>-<kebab-slug>.md`, resolution and reporting use the canonical `<ID>`, existing unslugged decision files stay readable and are never renamed. (2) TASK-88 covers all managed families: tasks and milestones are implemented, decisions and document records get the layout behavior specific to them, and releases are verified from the existing release implementation (native `<ID>.md`, configuration-driven directory, no translation) with no new lifecycle command; the ticket's ACs are now family-specific. (3) Archive and deprecation are distinct: decision deprecation is `## Status` in place, while the documented `papyrus decision archive` moves to `<decision_directory>/archived/`; releases and documents have no archive command. (4) Archiving an already archived record, including one in a legacy alias directory, fails without any write or move; restore is the only operation that moves a record to the active directory.
- 2026-11-12 (TASK-88 refinement): confirmed the canonical archive target `<backlog_directory>/archive/<family>/` of section 2.2 and section 7.1; the legacy `tasks/archived/` and `milestones/archived/` locations are read-only aliases and a restore from them moves the file to the active directory. Confirmed that no milestone unarchive command exists, so section 7.1 adds none. Approved the section 2.1 and section 4.2–4.3 wording, including the all-lowercase/unpadded exclusion and the prefix-casing assumption. Recorded that the lazy relocation path is removed by TASK-88 and the lazy body conversion by TASK-89, and that lazy adoption never relocates or renames a file.
- Note for architecture: ADR-0010's decision 2 table and decision 7 text should be aligned with this specification and the existing documented commands: remove `<backlog_directory>/archive/releases/` (releases are never archived, BR-007/`release-lifecycle.md`) and record the decision archive location `<decision_directory>/archived/` for the explicit `papyrus decision archive` command (BR-008, REQ-004.002), while keeping deprecation in place via `## Status`. ADR-0010 decision 4 also states the Backlog-shaped exclusion absolutely; this specification records its qualification for an all-lowercase configured prefix. Both points need architect sign-off; they do not change the ticket's behavior beyond the documented commands.
- Specification conflicts resolved: `decision-records-management.md` section 8.1 is updated from `<ID>.md` to the ADR-0010 slug convention with canonical-ID resolution, and `browser-decision-records.md` section 2 now derives the canonical ID from a slugged stem instead of treating the stem as the ID.

## Sources

- `docs/architecture/adr/ADR-0010-papyrus-native-on-disk-record-layout.md`
- `docs/architecture/adr/ADR-0011-one-way-backlog-md-import.md`
- `docs/architecture/adr/ADR-0012-papyrus-owned-front-matter-metadata.md`
- `docs/architecture/adr/ADR-0013-shared-record-envelope-without-a-storage-layer.md`
- `docs/requirements/business/REQ-001.002.md`
- `docs/requirements/business/REQ-001.005.md`
- `docs/requirements/business/REQ-001.007.md`

## Traceability

| Artifact | Reference |
|---|---|
| Ticket | TASK-83 (specification), TASK-88 (implementation) |
| Requirements | REQ-001.005, REQ-001.002, REQ-001.007, REQ-004.002, REQ-005.010, REQ-005.011, REQ-006.008 |
| Business rules | BR-007 (release lifecycle), BR-008 (decision commands) |
| ADRs | ADR-0010, ADR-0011, ADR-0012, ADR-0013 |
| Related specifications | `decision-records-management.md`, `release-lifecycle.md`, `browser-decision-records.md` |
| Related tickets | TASK-88, TASK-89, TASK-90, TASK-92, TASK-98, TASK-100 |
| Source | src/record/identity.rs, src/record/descriptor.rs, src/record/store.rs, src/task.rs, src/milestone.rs, src/config.rs, src/context.rs |
| UI mockups | none |
