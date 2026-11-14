# Upgrading to Papyrus Release 1.0

**Applies to:** Papyrus 1.0, the one-time import of Backlog.md task and milestone records, and the move to configuration-defined document types.

Release 1.0 completes the move to the Papyrus-native record layout. Papyrus no longer reads or writes the Backlog.md on-disk layout and ships `papyrus import backlog` as the only bridge from a Backlog.md store into Papyrus.

## Who must act

You must act if your project's backlog records are still stored in the Backlog.md layout:

```text
backlog/tasks/task-<N> - <slug>.md
backlog/milestones/m-<N> - <slug>.md
backlog/archive/tasks/task-<N> - <slug>.md
backlog/archive/milestones/m-<N> - <slug>.md
```

- Upgrading the binary is enough to be affected. No automatic import runs at install time, at startup, or on the first command.
- The old files stay on disk but the new version does not see them: they are not listed, viewed, searched, or rendered, and no command accepts their IDs (`task view task-1` and `milestone view m-0` fail with exit status `3`).
- No compatibility path exists: no transitional reader, no environment flag, no dual-recognition mode. `papyrus import backlog` is the only way in.
- A project that never used Backlog.md needs no action.

## What the import covers

| Source under the Backlog.md project | Treatment |
|---|---|
| `backlog/tasks/*.md` | Imported as active tasks |
| `backlog/archive/tasks/*.md` | Imported as archived tasks |
| `backlog/milestones/*.md` | Imported as active milestones |
| `backlog/archive/milestones/*.md` | Imported as archived milestones |
| `backlog/docs/**` | Excluded; counted and reported |
| `backlog/drafts/**` and `backlog/archive/drafts/**` | Excluded; counted and reported |
| `backlog/decisions/**` | Excluded; counted and reported |
| `backlog/config.yml` | Never read as a record and never written; reported as excluded |
| `backlog/.locks/**` | Never read as a record and never written; reported as excluded |

Only `.md` files whose stem matches `<prefix>-<number>`, optionally followed by ` - <slug>`, are candidates. Other files are reported as skipped, and a run that skips records exits non-zero.

## Before you start

1. Have a target project with `papyrus.config.yml`. Select it with `--dir`, `PAPYRUS_DIR`, or the current directory; create one with `papyrus init <name>`.
2. Keep the source store. The import reads it read-only and never rewrites it, but keep it unchanged until you verify the import.
3. Decide whether the target already holds tasks or milestones. If so, the command refuses before writing unless you pass `--allow-existing`.
4. Plan your review before the real run: the printed plan and optional report are the only record of the mapping. Save the report.

The target and source may be the same project: legacy files sit in the target's own `backlog/` folder and the import writes native `<ID>.md` records beside them.

## Procedure

Run from, or point `--dir` at, the target project. `<source-project-dir>` is the root of the project holding the Backlog.md store.

### 1. Initialize or select the target project

```sh
papyrus --dir <target-project> init <name>     # only when the target is new
```

The target must contain `papyrus.config.yml`.

### 2. Run the dry run

```sh
papyrus --dir <target-project> import backlog --from <source-project-dir> --dry-run
```

The dry run performs the full plan and prints it. It imports no records and does not touch the source. Without `--report` it writes nothing at all; with `--report` it writes only that report file.

### 3. Review the printed plan

```text
Papyrus import backlog
Source project: /path/to/legacy-project
Source backlog: /path/to/legacy-project/backlog
Target project: /path/to/papyrus-project
Mode: dry-run (no writes)
Excluded: docs=3 drafts=1 decisions=2 config=1 locks=1
Planned: tasks=5 milestones=3
Skipped: 0
Plan:
  TASK-001 <- task-1 Foundation task
  ...
  MS-001 <- m-0 Foundation
```

`Planned:` gives the counts. Each `Plan:` line maps one source stem to one new Papyrus ID. `Source anomalies:`, `Warnings:`, and `Skipped records:` appear only when relevant.

### 4. Save the plan as a report

```sh
papyrus --dir <target-project> import backlog --from <source-project-dir> \
  --dry-run --report import-report.md
```

`--report <path>` writes the same text to a file. A relative path is resolved against the project directory; a path outside the project is rejected (exit `4`) before anything is imported. The report is diagnostic output only; no later command reads it.

### 5. Run the import

```sh
papyrus --dir <target-project> import backlog --from <source-project-dir> \
  --report import-report.md
```

This runs the same plan and writes the records. Active source records land in configured active directories; archived source records in configured archive directories. No existing record is overwritten, merged, or renumbered.

## Command reference

```text
Usage: papyrus import backlog [OPTIONS] --from <source-project-dir>
```

| Option | Required | Meaning |
|---|---|---|
| `--from <source-project-dir>` | yes | Source project root holding the Backlog.md store. |
| `--source-backlog-dir <dir>` | no | Source backlog directory. Default: `<source-project-dir>/backlog`. Relative values resolve against `--from`. |
| `--dry-run` | no | Plan and report without writing records or touching the source. |
| `--allow-existing` | no | Allow importing into a target that already holds records of an imported family. |
| `--report <path>` | no | Write the report to a path inside the project directory. |
| `-h`, `--help` | no | Print help and exit. |

### Global options

These apply to this command as to every other command and do not change the plan:

| Option | Meaning |
|---|---|
| `--json` | Emit stable machine-readable JSON. |
| `--dir <path>` | Project directory. Default: current directory. |
| `--config <file>` | Configuration file. Default: `papyrus.config.yml`. |
| `--no-color` | Disable ANSI color; `NO_COLOR` has the same effect. |

`--debug` belongs to the `browser` command only.

### Exit status

| Code | Meaning |
|---|---|
| `0` | Every in-scope record was imported, or planned for a dry run. |
| `1` | Runtime failure: missing/unreadable source, refusal without `--allow-existing`, a run that skipped records, or an import aborted by a write failure. |
| `2` | Usage error: missing required or unknown option. |
| `4` | Validation rejection before any write, e.g. a `--report` path outside the project. |

Exit status `3` (record not found) is not produced by this command.

## What changes

### Task IDs are reallocated

Imported tasks receive fresh consecutive `TASK-nnn` numbers, allocated per family in ascending source-number order, starting from the highest task number present in the target's active, archived, and deprecated locations, plus one (minimum 1). No source number is preserved or accepted later: `task-18` may become `TASK-042`, and `task view task-18` exits `3`.

### Milestone numbers are remapped

Imported milestones are allocated in the Papyrus milestone space from the highest existing milestone number plus one; when the target holds no milestones the first imported milestone becomes `MS-001`. The source `m-` number is never preserved, in filenames, front matter, references, the CLI, or the browser. Backlog.md has no release concept, so no `release` value is written on an imported milestone.

### The mapping exists only in the plan and the report

The printed plan is the only record of the source-to-Papyrus mapping; `--report` is the only way to keep a copy. No mapping file, sidecar, marker key, extra front-matter field, import registry, or source-state database is created.

### Fields and body content carried over

- Front matter copied: title (required), status, assignees, created and updated dates, labels, milestone and dependency references, `references`, `documentation`, `type`, `ordinal`, and remaining source keys such as `priority`. Unknown keys are preserved verbatim.
- Body markers converted: `SECTION:DESCRIPTION` to `## Description`, `AC` to `## Acceptance Criteria` (the `#N` ordinal is dropped; checkbox state and text are kept), `DOD` to `## Definition of Done`, `SECTION:PLAN` to `## Implementation Notes`, `SECTION:NOTES` to `## Notes`, `COMMENTS` to `## Comments`. Unknown sections and free text are appended verbatim in file order.
- A record without a title is skipped and reported.

### Source handling and ownership

The source is opened read-only and left byte-identical. Keep it until the imported records are verified. After the import, Papyrus owns the records: subsequent commands, views, and searches treat them like any other record.

## One-way behaviour and limits

- **Compatibility renounced.** `papyrus import backlog` is the only bridge. There is no reverse export, re-import reconciliation, sync, watcher, or bidirectional behaviour, and Papyrus never regenerates Backlog.md files.
- **Not idempotent.** Run it once per source store. A second run against the same source allocates new numbers and duplicates every record under new IDs. Existing records are never overwritten, merged, or renumbered. The dry-run plan and same-title warnings are the review mechanisms; no marker or lock exists.
- **`--allow-existing` for a non-empty target.** When the target already holds any record of an imported family, the command refuses before writing and tells you to use `--allow-existing` if duplication is intended.

### Known import caveats

| Situation | Result |
|---|---|
| Source status does not match configured statuses after normalization (case-insensitive; spaces, hyphens, underscores ignored) | Preserved verbatim with a warning; never silently coerced. |
| A dependency or milestone reference cannot be resolved | Dropped with a warning; no source ID survives. The imported graph can be incomplete. |
| A timestamp is absent or not a valid quoted ISO-8601 UTC value | Replaced by the import time and reported as approximate. |
| A record is unparseable (no title, malformed section block) | Skipped and reported; other records still import; the run exits `1`. |
| A destination write fails mid-import | The import aborts, reports which records were written, and keeps them. No rollback. |
| `--report` points outside the project directory | Validation error (exit `4`) before any import write. |
| Two source files share a source number | Both import as distinct records; the report flags the anomaly. |

### No rollback or undo

There is no `import undo`, no transaction across the whole run, and no automatic cleanup. A partial import is repaired by resolving the cause and importing into a fresh target, or by accepting duplicates with `--allow-existing` and deleting what you do not want.

## Verify the import

1. List and view by the new IDs: `papyrus --dir <target-project> task list`, `milestone list`, and `task view TASK-001`. Backlog IDs still fail with exit `3`.
2. Search by the new ID or title: `papyrus --dir <target-project> search TASK-001`. Search matches whole persisted text including front matter.
3. Check the count against the saved report's `Planned:` line.
4. Confirm the source is unchanged (for example `diff -r <source-backup>/backlog <source-project-dir>/backlog`, or `git status`).
5. Confirm a second run without `--allow-existing` is refused with exit `1` and `use --allow-existing`, and no new file appears.
6. Check the browser: `papyrus --dir <target-project> browser --no-open` serves `/doc/TASK-001` with HTTP 200 and the backlog board shows the imported milestone.
7. Keep `import-report.md`; it is the only mapping.

These behaviours were exercised end to end for Release 1.0 against an isolated sample store; see [`packaging.md`](packaging.md).

## Document types are now configuration-defined

**Applies to:** every project that used a per-type document command, the `nfreq` alias, or a seeded `deploy` type, and every project with documents under a `deprecated/` folder.

Release 1.0 makes the document command surface configuration. The command name is the `document_types` key, and only enabled keys exist as commands. `req` and `nfr` still work because they are default entries; the parser gives them no special treatment.

### Who must act

You must act if any of the following applies:

- You used `papyrus nfreq ...`. The alias is gone; use `papyrus nfr ...`.
- You used a per-type document command (`req`, `glossary`, `ui-spec`, `ui-mockup`, `tech-spec`, `audit`, `use-case`, `test-case`, `test-report`, `rule`) with `deprecate` or `restore`.
- You relied on `deploy` being seeded. It is not; declare it before `papyrus deploy ...` works.
- Your documents live under a `deprecated/` folder. Those files are no longer read, listed, or searched.

The static per-type command blocks are removed from the command tree. A slug that is not an enabled `document_types` entry is an unknown command and exits `2`.

### Command migration

| Before | After |
|---|---|
| `papyrus req deprecate <id>` | `papyrus req archive <id>` |
| `papyrus req restore <id>` | No command. Move the file into the type's active directory and set `status: active` (or another declared non-`archived` status) in the front matter. |
| `papyrus nfreq <verb> ...` | `papyrus nfr <verb> ...` |
| `papyrus deploy <verb> ...` | Declare a `deploy` entry in `document_types` first (edit `papyrus.config.yml`); then `papyrus deploy <verb> ...`. |
| `papyrus <unknown-or-disabled-slug> ...` | Exit `2`. Enable or declare the slug in `document_types`. |
| `papyrus <slug> restore <id>` | No document restore exists. |

`req` and `nfr` are the default `document_types` entries `req` and `nfr`. Remove either entry and its command disappears.

### `document_types` is a whole map

- A `document_types` mapping in `papyrus.config.yml` replaces the default map wholesale. When the key is present, exactly its entries exist; no default entry is re-added.
- Removing an entry stops managing its files. The files are not deleted and can be recovered by declaring the entry again.
- `enabled: false` keeps the entry, its prefix reservation, and its files, but removes the command and the browser view.
- `papyrus config set document_types.<slug>.<field>` materializes the eleven defaults when the key is absent and edits an existing entry. It cannot create a new entry; define a new entry by editing the file (see [Repairing an invalid configuration](#repairing-an-invalid-configuration)).
- `papyrus init` writes these eleven defaults. `deploy` is not one of them:

| Slug | Prefix | Directory (relative to `doc_directory`) |
|---|---|---|
| `req` | `REQ` | `requirements/business/` |
| `nfr` | `NFR` | `requirements/non-functional/` |
| `glossary` | `GLO` | `glossary/` |
| `rule` | `RUL` | `business-rules/` |
| `ui-spec` | `UI` | `specifications/ui/` |
| `ui-mockup` | `MOCK` | `specifications/ui/mockups/` |
| `audit` | `AUD` | `audit-records/` |
| `tech-spec` | `TECH` | `technical/` |
| `use-case` | `UC` | `use-cases/` |
| `test-case` | `TC` | `test-cases/` |
| `test-report` | `TR` | `test-reports/` |

### Statuses

- Every front-matter-backed document type has the fixed main statuses `active` and `archived`. Any explicit `statuses` list must declare each exactly once.
- Custom statuses are optional, ordered front-matter values. They do not control visibility or location.
- `default_status` is the status of a new record. It defaults to `active`, must be a declared status, and must not be `archived`.
- `papyrus <slug> status <id> <value>` sets `active` or a declared custom status in place. It rejects `archived` with exit `4` and names `archive` as the way to archive.
- `archive` is the only command that sets `archived`. It replaces any custom status with `archived`.
- UI mockups are HTML records whose state lives in file metadata (`pending`, `approved`, `archived`). They have no `status` command, and declaring `statuses` or `default_status` on `ui-mockup` is invalid.

### Archive location and no restore

- An archived document moves to `<doc_directory>/<document_types.<slug>.directory>/archived/`.
- Archived documents are hidden from lists, search, and the browser.
- Documents have no `restore` command. To revive one by hand, move its file back into the type's active directory and set a declared non-`archived` status in the front matter.
- `delete` on an active document archives it first and then removes the archived file. `--force` skips the confirmation prompt.

### Legacy `deprecated/` folders

A `deprecated/` folder is inert: Papyrus does not read, migrate, or inventory it. Recover records by hand:

1. Move each file from `deprecated/` into the directory of its `document_types` entry.
2. Set `status: active` (or another declared custom status) in its front matter. Use `status: archived` and move it into `archived/` if it should stay archived.
3. Verify with `papyrus <slug> list` and `papyrus <slug> view <id>`.

### Repairing an invalid configuration

Configuration validation is fail-closed: while the configuration is invalid, every command that validates it exits `4` with a message naming `document_types.<slug>.<field>`.

- Inspect values with `papyrus config get <key>` and `papyrus config list`. These read the configuration without running registry validation.
- Repair an existing entry with `papyrus config set <key> <value>` or `papyrus config unset <key>`. Both validate the merged configuration and write nothing when it is invalid.
- Define a malformed or new entry by editing `papyrus.config.yml` directly. A single `config set` cannot create a new type because the entry would still lack its required `prefix` or `directory`.
- `papyrus init --force` replaces the configuration file in full (the eleven defaults when non-interactive). It is a last resort and discards custom entries and statuses.

List-valued status keys (`statuses` and `document_types.<slug>.statuses`) take a comma-separated value:

```sh
papyrus config set document_types.audit.statuses "active, archived, review"
```

Each element is trimmed and stored as a YAML flow sequence (`[active, archived, review]`). An empty value, an element that is empty after trimming, and a value containing a control character are rejected and nothing is written. Commas, quotes, and backslashes are not escaped: every comma separates elements, so a status that must contain a comma can only be written by editing `papyrus.config.yml` directly. An invalid list is refused and the file is unchanged.

### Man pages and NFR-03.003

The document command family is generated from project configuration. Config-defined slugs have no man page: ADR-0021 remains Proposed and the scope decision is deferred. NFR-03.003 is not claimed for these slugs, and the gap is recorded as a release risk. This release adds no man-page source, generator, or package content for them.

### Verify the upgrade

1. `papyrus --help` lists the enabled document slugs, including `req` and `nfr`.
2. `papyrus nfreq --help` exits `2`; `papyrus nfr --help` lists the `nfr` verbs.
3. `papyrus deploy --help` exits `2` until a `deploy` entry is declared.
4. `papyrus req archive REQ-003` moves the file to `<doc_directory>/requirements/business/archived/REQ-003.md`, and `papyrus req list` no longer shows it.
5. `papyrus req status REQ-003 archived` exits `4` and mentions `archive`.
6. `papyrus req status REQ-004 review` sets a declared custom status; `papyrus req archive REQ-004` replaces it with `archived`.
7. Break the configuration (for example, remove `archived` from one `statuses` list); any command exits `4` naming the key. Repair it and confirm the command runs again.
8. A file left under `deprecated/` is not listed. After the manual move and `status: active`, it is listed and viewable.

## Other user-visible changes in Release 1.0

- **Shared single record view layout.** Every managed record type with a single record view uses one layout: a summary card showing only record ID, created date, and updated date (plus owning group for a sub-requirement); labels as a taglist under the title; dependencies and references at the end as collapsible native `<details>` sections, shown only when non-empty and usable without JavaScript. UI mockup views are excluded.
- **Editable browser configuration view and store relocation.** The browser serves `/config` (alias `/?type=config`), shows effective values, marks explicitly set keys, and edits through the same configuration path as the CLI. Editing a folder key, including `papyrus config set backlog_directory <path>`, relocates the store as one folder move: every file and subfolder is moved and the empty old folder removed. A conflict, unwritable destination, or path outside the project exits `4`, moves nothing, and leaves the configuration unchanged. The browser never moves a folder itself.
- **Non-functional requirements grouped under their owning group.** The browser derives the owning group from the ID prefix (`NFR-04.002` belongs to `NFR-04`), shows the group title with a number fallback, and renders empty groups explicitly.
- **Quiet browser console and browser-only `--debug`.** The browser prints only errors by default. `papyrus browser --debug` logs one line per request to standard error with method, path, status, and duration; headers and bodies are never logged. `--debug` is not a global option, and `--json` output stays a single object on standard output.
- **One ID space.** `storage_id`, `display_id`, and `storage_path` are gone from JSON, routes, links, and pages. Every surface shows only Papyrus IDs, and a Backlog ID is treated as an unknown record.

## Related documentation

- [`../README.md`](../README.md) — command reference, including the `papyrus import backlog` man page section.
- [`packaging.md`](packaging.md) — build, install, and recorded Release 1.0 verification evidence.
- [`specifications/import-backlog-md-records.md`](specifications/import-backlog-md-records.md) — the import contract.
- [`specifications/release-1.0-native-layout-verification-and-packaging.md`](specifications/release-1.0-native-layout-verification-and-packaging.md) — what was verified for this release.

---

*This document was generated with AI assistance.*
