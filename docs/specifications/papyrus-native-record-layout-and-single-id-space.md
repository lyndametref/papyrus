# Feature Specification — Papyrus-Native Record Layout and Single ID Space

**Ticket:** TASK-83
**Milestone:** m-16 (Release 1.0)
**Status:** Refined (execution-ready)
**UI mockups:** none

## Purpose

Define the Papyrus-native on-disk record layout and the single Papyrus ID space that replace the Backlog.md layout of ADR-0008. This specification implements ADR-0010, REQ-01.005, and REQ-01.002 for every managed record type: configuration-driven directories, one file per record, one Papyrus ID as the record identity, tolerant reads, allocation without offsets, moves for archive/deprecate, and the intentional invisibility of Backlog.md stems. It is the destination layout the one-way import (see `import-backlog-md-records.md`) writes into and the layout every later implementation task (TASK-88 … TASK-98) must honour.

## Scope

### In scope

- Configuration-driven directory resolution for every record family (task, milestone, release, decision, document types).
- Native file names: `<ID>.md` by default; decisions `<ID>-<kebab-slug>.md`; cosmetic slugs never used for resolution.
- Tolerant read-stem resolution: `<PREFIX>-<digits>` optionally followed by `-<slug>` or ` - <slug>`, case-insensitive prefix, optional leading zeros; legacy archive aliases.
- Front-matter `id` as the authoritative record identity; disagreement with the stem is a validation error that refuses writes.
- Number allocation without offsets: `max(observed) + 1` over every active, archived, and deprecated location of a family; minimum 1; milestones start at 1.
- Archive, restore, deprecate, restore as file moves that keep the file name and the ID.
- Atomic whole-folder relocation during `config set` (REQ-01.002).
- The rule that Backlog.md storage stems (`task-18`, `m-0`, …) match no Papyrus record, so an unimported Backlog.md store is invisible to every Papyrus surface.

### Out of scope

- The one-way import itself — see `import-backlog-md-records.md`.
- Front-matter field map and patching — ADR-0012 / `migrate-document-record-types-to-front-matter-lazily-on-first-read-and-first-write.md`.
- UI mockups (HTML records) — excluded from this layout's Markdown rules.
- Corpus-wide migration or renaming of existing records.

## Behavior

### 1. One record, one file, one identity

1. The Papyrus ID (REQ-01.005) is the record identity everywhere: in the front-matter `id`, in CLI and browser output, and as command input.
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
   | Release | `<backlog_directory>/releases/` | `<backlog_directory>/archive/releases/` | — |
   | Decision (ADR, BDR) | `<decision_directory>/` | — | in place via `## Status` |
   | Document record (`req`, `nfr`, `glossary`, `rule`, `ui-spec`, `tech-spec`, `audit`, `deploy`, `use-case`, `test-case`, `test-report`) | `<doc_directory>/<type directory>/` | — | `<doc_directory>/<type directory>/deprecated/` |

3. `backlog_directory` and `decision_directory` are relative to the project directory; a document type's `directory` is relative to `doc_directory`.
4. Every resolved path passes `Context::ensure_within` before any read or write (NFR-04.002).
5. UI mockups remain HTML records under their configured `ui-mockup` directory and keep REQ-05.008 metadata; they are not Markdown records and this layout does not change them.

### 3. File name

1. Default: `<ID>.md`, where `<ID>` is the full Papyrus ID including the configured prefix and padding (`TASK-001.md`, `MS-001.md`, `REL-001.md`, `REQ-01.001.md`, `NFR-04.004.md`, `GLO-001.md`, `RUL-016.md`, `AUD-001.md`).
2. Decisions keep a descriptive suffix because the ADR corpus convention and its external links require it: `<ID>-<kebab-case-slug>.md` (`ADR-0010-papyrus-native-on-disk-record-layout.md`). The slug is fixed at creation and is never regenerated.
3. The slug is cosmetic. Resolution never uses it. No writer renames a file when a title changes.
4. Document records that already carry a slug (for example this repository's `BR-016-<slug>.md`) keep their names when read and written; new records of those types use `<ID>.md` unless a later decision gives that type its own suffix rule.

### 4. Resolution (read tolerance)

1. A file inside a claimed directory whose stem is `<PREFIX>-<digits>` optionally followed by `-<slug>` or ` - <slug>` is a candidate record.
2. The prefix is compared case-insensitively against the configured prefix and leading zeros are optional, so `TASK-1.md`, `TASK-001.md`, and `TASK-0001.md` all denote `TASK-001`. Case-insensitive matching is retained for Papyrus prefixes (for example `Task-001` or `task-001` with a padded number).
3. Backlog.md storage stems are **not** recognized. A stem is Backlog-shaped, and resolves to no record, when its prefix is the all-lowercase form of a configured prefix that contains at least one uppercase letter **and** its number has no leading zeros (the Backlog storage shape, e.g. `task-18`, `m-0`). Such a stem is invisible even though it case-folds to the configured prefix. Any other Backlog storage prefix has the same effect. This enforces "the import is the only bridge" (ADR-0011). All other case variants, including padded lowercase forms such as `task-001`, resolve normally.
4. Front-matter `id` is authoritative. If it disagrees with the stem after normalization, the record is read from front matter, is flagged invalid, and writes to it are refused until the file is fixed.
5. Legacy archive aliases `<backlog_directory>/tasks/archived/` and `<backlog_directory>/milestones/archived/` are read as archived tasks and milestones. A read never relocates or renames them; archive/restore operations follow section 7.

### 5. Numbering and immutability

1. Each record family has one number space: tasks, milestones, releases, and each decision prefix and document prefix separately.
2. **Milestones start at 1 (`MS-001`); the Backlog `m-0` offset is gone.** Milestone renumbering is accepted as a user-visible consequence: a milestone stored as `m-N` is remapped into the Papyrus milestone space by allocation (see `import-backlog-md-records.md`), and the source `m-` number is never preserved.
3. Allocation is `max(number present in every active, archived and deprecated location of that family) + 1`, minimum 1. A number is allocated once, at creation, and never recomputed.
4. Deleting the highest-numbered record makes that number available again; existing records are never renumbered.
5. Archive, restore, deprecate, restore, supersede, and title edits never change an ID or a file name.
6. A configuration change to a prefix or `id_padding` affects only records created afterwards. Existing IDs, file names, and reference values are untouched, and padding remains a minimum (REQ-01.005, REQ-01.007).

### 6. No shape conversion, no renaming on read or write

1. No path converts a record to a Backlog.md on-disk shape, writes a Backlog storage ID, or renames a file into another naming convention.
2. Lazy adoption (REQ-05.011, ADR-0012) adds a front-matter block and may retire legacy metadata sections; it does not move, rename, or re-shape the file.
3. A record that is not in the native layout stays where it is; the one-way import (`import-backlog-md-records.md`) is the supported way to bring a Backlog.md store into the native layout.

### 7. Archive, restore, deprecate, restore are moves

1. An archive moves the file from the active directory to the archive directory of its family and an unarchive moves it back, keeping the file name and the ID.
2. Deprecate and restore do the same between the active and `deprecated/` directories of a **document** type (REQ-05.007). Decision records are never moved on deprecation: their state is the `## Status` line, and keeping them in the decision directory keeps them in the ADR index.
3. A move inside the project uses a same-directory-or-same-filesystem rename where available, otherwise the ADR-0006 atomic write sequence (write the destination with `create_new`, `sync_all`, then remove the source).
4. A failed move leaves the record readable at exactly one location and reports the error; it never duplicates a record silently. If both locations hold the file, the record is flagged invalid and writes are refused.

### 8. Whole-folder relocation on `config set`

1. `backlog_directory`, `decision_directory`, `doc_directory`, and a document type's `directory` are handled by one shared relocation routine (REQ-01.002).
2. The routine runs inside the `config set` command, never lazily and never on a later read.
3. It moves every file and subfolder of the source folder, not only managed records.
4. It pre-checks the whole plan (destination entry conflicts, containment, permissions) before moving anything.
5. On any failure it moves nothing (or rolls back what it already moved), persists no configuration value, and reports the conflict outcome of REQ-01.004.
6. After success the source folder is removed when empty.
7. The browser configuration view (REQ-06.008) must call this same `config set` path; it may not move folders itself.

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
- Assumption: `id_padding` is constrained to 1–6 and is a minimum width (REQ-01.007); existing IDs are never reformatted when padding changes.
- Assumption: document records already follow `<ID>.md` and decisions `<ID>-<slug>.md`, so no renaming is needed for the document corpus.
- Assumption: no UI design or mockup is required because this is a storage/path specification with no user-facing UI beyond existing command output.

## Open questions

- None. The layout, single ID space, and configuration-driven path convention were confirmed by the user on 2026-11-11 (ADR-0010).

## Sources

- `docs/architecture/adr/ADR-0010-papyrus-native-on-disk-record-layout.md`
- `docs/architecture/adr/ADR-0011-one-way-backlog-md-import.md`
- `docs/architecture/adr/ADR-0012-papyrus-owned-front-matter-metadata.md`
- `docs/architecture/adr/ADR-0013-shared-record-envelope-without-a-storage-layer.md`
- `docs/requirements/business/REQ-01.002.md`
- `docs/requirements/business/REQ-01.005.md`
- `docs/requirements/business/REQ-01.007.md`

## Traceability

| Artifact | Reference |
|---|---|
| Ticket | TASK-83 |
| Requirements | REQ-01.005, REQ-01.002, REQ-01.007, REQ-05.010, REQ-05.011, REQ-06.008 |
| ADRs | ADR-0010, ADR-0011, ADR-0012, ADR-0013 |
| Related tickets | TASK-88, TASK-89, TASK-90, TASK-92, TASK-98, TASK-100 |
| Source | src/record/identity.rs, src/record/descriptor.rs, src/record/store.rs, src/task.rs, src/milestone.rs, src/config.rs, src/context.rs |
| UI mockups | none |
