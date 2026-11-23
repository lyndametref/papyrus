---
id: TECH-002
title: "Project Initialization and Configuration Commands"
created_date: '2026-11-16T17:46:43Z'
updated_date: '2026-11-18T22:50:56Z'
labels: []
references:
  - ADR-0001
  - ADR-0002
  - ADR-0003
  - ADR-0010
  - ADR-0013
  - ADR-0019
  - ADR-0022
  - ADR-0023
  - ADR-0024
  - BR-001
  - BR-002
  - BR-003
  - BR-016
  - NFR-002.001
  - NFR-002.002
  - NFR-004.001
  - NFR-004.002
  - NFR-006.001
  - REQ-001
  - REQ-001.001
  - REQ-001.002
  - REQ-001.003
  - REQ-001.004
  - REQ-001.005
  - REQ-001.006
  - REQ-001.007
  - REQ-005.008
  - REQ-006.008
  - REQ-007.001
  - REQ-007.002
  - TASK-001
  - TASK-111
  - TASK-112
  - TASK-117
  - TASK-122
  - TASK-127
  - TASK-2
  - TASK-3
  - TASK-85
  - ADR-034
  - REQ-006.018
  - RE-005
status: draft
---
# TECH-002 Project Initialization and Configuration Commands

> **Revised (TASK-203, 2026-11-18):** the browser configuration view clauses — section 3 item 12, the browser-view tail of item 21, the out-of-scope list-editing item, and the relocation assumption — are retired per ADR-034 and REQ-006.018 (RE-005): the browser serves no configuration view and configuration editing is CLI-only. The `config set` configuration, list-key, and whole-folder relocation clauses (BR-003, REQ-001.002) remain current. REQ-006.008, which the retired clauses cite, is archived; the retired text is retained below as history. See TECH-080.

## Description
## Purpose

Deliver the `init` and `config` commands, plus project/configuration context selection and the common CLI controls and outcomes defined by REQ-001, so a maintainer can create a project context, select it deterministically, inspect and change effective configuration, and rely on consistent command behavior and exit statuses. This ticket also delivers the configuration keys that govern stable record IDs (prefixes and padding); record commands in later tickets consume those keys.

This specification was revised (TASK-85) so that initialization, missing-configuration failure, and folder relocation match REQ-001.001, REQ-001.002, REQ-001.003 and ADR-0010 decision 8 (whole-folder relocation), including the browser configuration view as a consumer of the same `config set` path.

It was revised again (TASK-111) so that the list-valued keys `statuses` and `document_types.<slug>.statuses` can be written through `config set` using a comma-separated value stored as a YAML flow sequence, and read back by `config get` and `config list` (REQ-001.002, REQ-007.002, ADR-0024).

It was revised again (TASK-112) so that the generated default configuration is the eleven-entry document-type set of REQ-007.002 without `deploy`, and so that every front-matter-backed default document type carries the seeded status list `[active, archived]` while `ui-mockup` carries none and no entry carries `default_status` (REQ-007.001, REQ-007.002, REQ-005.008, ADR-0019).

It was revised again (TASK-122) so that `papyrus config list` also prints `document_types.<slug>.default_status` for each entry that declares one, closing G-4 of TASK-117: an invalid `default_status` is now visible on the inspection surface an operator uses before repairing it (REQ-001.002, BR-003, ADR-0019).

It was revised again (TASK-127) so that OQ-1 is resolved: section 2 item 4 now names the exact directory set init creates, states that no `archived/`, `deprecated/`, or `backlog_directory/archive/` folder is pre-created, and keeps the no-overwrite rule (ADR-0022, ADR-0023, REQ-007.001).

## Scope

### In scope

- `papyrus init <name> [--no-git] [--force]` (BR-002, REQ-001.001, REQ-001.006).
- `papyrus config list|get|set|unset` with dotted keys and YAML-scalar values (BR-003, REQ-001.002).
- List-valued keys `statuses` and `document_types.<slug>.statuses`: comma-separated values on `config set`, YAML flow-sequence storage, flow-form output from `config get`/`config list`, empty-value rejection, and validation of the resulting configuration before persistence (REQ-001.002, REQ-007.002, ADR-0024).
- Generated default document-type set: exactly `req`, `nfr`, `glossary`, `rule`, `ui-spec`, `ui-mockup`, `audit`, `tech-spec`, `use-case`, `test-case`, `test-report` with their declared prefixes and directories; `deploy` is absent from the default set but remains addable as a user-defined type (REQ-007.002).
- Seeded document status model in the generated configuration: `statuses: [active, archived]` for every front-matter-backed default document type, no `statuses` for `ui-mockup`, and no `default_status` on any default entry (REQ-007.001, REQ-007.002, REQ-005.008).
- Project directory selection: `--dir`, `PAPYRUS_DIR`, then current working directory, in that precedence order (REQ-001.003).
- Configuration file selection: `--config`, then `PAPYRUS_CONFIG`, then `<project_dir>/papyrus.config.yml` (REQ-001.003). No built-in-defaults fallback and no directory walk-up/discovery when none is found.
- Common CLI controls and outcomes: `--json` (including errors), `--no-color` / `NO_COLOR`, `--version`, `-h`/`--help`, command help, and exit statuses 0/1/2/3/4 (REQ-001.004).
- Configuration keys and validation from BR-016, including task statuses uniqueness/default-status membership, default statuses `ToDo, Initiated, Refined, Implemented, Validated, Done`, and ID prefix/padding keys (REQ-001.002, REQ-001.005, REQ-001.007).
- File-access boundary: init is the sole exception to the project-directory restriction; all other reads/writes stay within the selected project directory, resolved canonically (NFR-004.002).
- Persistence format: all data saved by Papyrus is UTF-8 text (YAML/Markdown); no binary artifacts (NFR-004.001).
- Input sanitization by normalization/escaping before use (NFR-002.001, NFR-002.002).
- Atomic whole-folder relocation during `config set` for `backlog_directory`, `decision_directory`, `doc_directory`, and any document type's `directory` (REQ-001.002, ADR-0010 decision 8, ADR-0013 stage 5).

### Out of scope

- Record-management subcommands (`task`, `milestone`, `release`, `decision`, document types, `search`, `browser`) — TASK-3 onward.
- Actual record ID assignment and archive/restore preservation logic — consumed by record commands in later tickets; this ticket delivers their configuration keys only.
- Git workflows beyond invoking repository initialization when `--no-git` is absent.
- UI mockups or web UI — none for this ticket.
- List editing in the browser configuration view: the view's editable-key whitelist is unchanged, and a list key remains non-editable there (CLI `config set` only). Retired (TASK-203, 2026-11-18, ADR-034, REQ-006.018): the browser serves no configuration view, so list keys remain writable only through CLI `config set`, which stays current.
- A quoting/escaping form for commas, `--append`/`--remove` flags, or a `config get --json` list mode beyond the existing JSON array (ADR-0024).
- Non-amd64 or non-Debian distribution concerns — ADR-0002.

## Behavior

### 1. Project and configuration context selection

1. The project directory is selected from `--dir`, then `PAPYRUS_DIR`, then the current working directory.
2. The configuration file is selected from `--config`, then `PAPYRUS_CONFIG`, then `<project_dir>/papyrus.config.yml`.
3. If none of those paths yields an existing configuration file, every command except `papyrus init` fails as a runtime error with exit code 1. Papyrus does not fall back to built-in defaults and does not walk up directories or discover a configuration file elsewhere. Built-in defaults apply only to individual keys absent from a configuration file that was found.
4. An explicitly named `--config` or `PAPYRUS_CONFIG` path that does not exist counts as "not found" and fails the same way.
5. The failure message names the configuration path that was looked for (the resolved `--config`/`PAPYRUS_CONFIG` path when one is given, otherwise `<project directory>/papyrus.config.yml`) and states that `papyrus init` creates it.
6. Command-line options take precedence over environment variables, project configuration, and defaults.
7. For every command except `init`, the selected project directory is canonicalized (symlinks resolved) and all reads/writes must remain within it. A path that resolves outside it is rejected.

### 2. `init`

1. `papyrus init <name>` initializes the current directory; `papyrus --dir <path> init <name>` initializes `<path>`, creating it if absent.
2. `<name>` is required and is stored as `project_name` in `papyrus.config.yml`.
3. Initialization runs the interactive walkthrough (REQ-001.006) to select activated artifacts and collect their prefixes/folders (TTY mode; defaults in non-TTY mode) before creating the backlog store, the documentation structure, and `papyrus.config.yml`.
4. **Created directory set.** Created directories are derived from effective configuration (defaults when no configuration exists yet). Initialization creates exactly:
   - the configured `backlog_directory` with its `tasks/`, `milestones/`, and `releases/` subfolders (task, milestone, and release are always active);
   - the configured `doc_directory`, always;
   - the configured `decision_directory`, only when at least one decision type is enabled; if none is enabled the directory is not created but its configured (default) value is still written to `papyrus.config.yml`;
   - the configured active directory of each enabled document type (that type's `directory`, relative to `doc_directory`); a disabled document type's directory is not created.
   No `archived/` subfolder is pre-created: it appears when the first record of that type is archived (ADR-0022 decision 2). No `deprecated/` subfolder is created or read (ADR-0022 decision 6), and no `backlog_directory/archive/` folder is created by init; archive folders appear when the first archive operation of the family that uses them runs (ADR-0023 decision 2). Existing directories are reused and existing files are never overwritten (items 6–8).
5. `--no-git` suppresses Git repository initialization.
6. Initialization succeeds in a non-empty target directory. Files other than `papyrus.config.yml` are never modified by `init`.
7. An existing `papyrus.config.yml` blocks initialization unless `--force` is given. Without `--force`, initialization fails with a conflict outcome (exit code 4) and writes nothing.
8. With `--force`, initialization overwrites `papyrus.config.yml` in full with the generated configuration; no value from the previous file is kept. `--force` is a replacement, not a merge. `--force` does not permit overwriting any other pre-existing file.
9. `init` is the only command permitted to read from or write to a path outside an already-selected project directory (it may create the target directory).
10. All files written by init are UTF-8 text.
11. User-controlled input (`<name>`, paths) is sanitized by normalization/escaping before use; if normalization cannot produce a valid value, the command fails with exit code 4.
12. **Default document-type set.** `init` writes exactly the eleven document types of REQ-007.002, in this form, all enabled unless the walkthrough disables them: `req`/`REQ`/`requirements/business/`, `nfr`/`NFR`/`requirements/non-functional/`, `glossary`/`GLO`/`glossary/`, `rule`/`RUL`/`business-rules/`, `ui-spec`/`UI`/`specifications/ui/`, `ui-mockup`/`MOCK`/`specifications/ui/mockups/`, `audit`/`AUD`/`audit-records/`, `tech-spec`/`TECH`/`technical/`, `use-case`/`UC`/`use-cases/`, `test-case`/`TC`/`test-cases/`, `test-report`/`TR`/`test-reports/`. No `deploy` entry is generated.
13. **Seeded document statuses.** Each of those entries except `ui-mockup` is written with `statuses: [active, archived]` in that order, because the type is front-matter-backed. `ui-mockup` is written without a `statuses` key, because a `statuses` declaration on it is invalid configuration (REQ-005.008, ADR-0019). No generated entry declares `default_status`, so new records start at the effective default `active`.
14. **One default set.** The built-in defaults (the set applied when `document_types` is absent from a configuration file) and the configuration generated by `init`, interactive or not, are the same set with the same status seeds. Deactivating a type writes `enabled: false` and keeps its prefix, directory, and seeded status list.
15. **`--force` repair.** `papyrus init --force` replaces an existing configuration file — including one that fails ADR-0019 validation — with this generated default set, and the written file passes the validator. `deploy` may afterwards be added as an ordinary user-defined document type by editing `papyrus.config.yml` or with `papyrus config set`.

### 3. `config`

1. `papyrus config list` prints all effective keys, including defaults, in a stable order.
2. `papyrus config get <key>` prints one effective value.
3. `papyrus config set <key> <value>` persists one value. For a scalar key, `<value>` is parsed as a YAML scalar, unchanged from the current behavior. For a list-valued key, `<value>` is a comma-separated list (see items 13–16).
4. `papyrus config unset <key>` removes the configured key so its default applies.
5. Nested keys use dotted notation (for example `document_types.req.enabled`).
6. Keys and defaults match BR-016 (project name, prefixes, padding, date format, editor, paths, statuses, default status, decision/document types).
7. Configured task statuses must be unique; `default_status` must be one of them.
8. With default settings, new tasks start at `ToDo` and the ordered statuses are `ToDo`, `Initiated`, `Refined`, `Implemented`, `Validated`, `Done`.
9. With default configuration, task, milestone, and release IDs use prefixes `TASK`, `MS`, `REL` and three-digit padding (`TASK-001`, `MS-001`, `REL-001`); `id_padding` is constrained to 1–6 (REQ-001.007) and IDs wider than the padding are not truncated.
10. Invalid key, invalid value, or validation failure returns exit code 4 (validation/conflict).
11. **Whole-folder relocation on a folder change.** When `papyrus config set` changes `backlog_directory`, `decision_directory`, `doc_directory`, or a document type's `directory`, the entire content of the original (source) folder is moved to the new folder:
    - The move covers every file and subfolder of the source folder, not only Papyrus-managed records.
    - The move happens during the `set` command, synchronously, not lazily and not on a later read.
    - The move is atomic: the shared relocation routine (ADR-0010 decision 8, ADR-0013 stage 5) pre-checks the whole plan (destination entry conflicts, containment, permissions) before moving anything.
    - If any entry cannot be moved (for example an entry with the same relative path already exists at the destination, or a write failure), nothing is moved, both folders are left as they were, the configuration value is not persisted, and the command reports a conflict (exit code 4).
    - On success the source folder no longer holds the moved content and is removed once empty.
    - Cross-device moves fall back to the atomic copy-then-remove sequence and are reported as non-atomic in the command output if a single rename is impossible.
12. **Retired (TASK-203, 2026-11-18, ADR-034, REQ-006.018):** the browser configuration view (REQ-006.008, archived) is a consumer of this same `config set` path: it invokes the relocation routine through `config set` and may not move folders itself. It performs no folder move of its own. No browser configuration view exists; the CLI `config set` relocation path of item 11 remains current.
13. **List-key classification.** A single declared classification names the list-valued keys: the top-level task `statuses` and each `document_types.<slug>.statuses`. Every other key is scalar, including `default_status`, `document_types.<slug>.default_status`, `prefix`, `directory`, `enabled`, `id_padding`, and `browser_*`. Classification depends only on the key; it never inspects the shape of the supplied value.
14. **Comma-separated form.** For a list-valued key, `config set` splits the raw value on `,`, trims surrounding whitespace from each element, preserves element order, case, internal whitespace, and duplicates, and does not interpret YAML in the value (`true`, `1`, and `[a]` remain strings). An empty or whitespace-only value is rejected with exit 4 and a message naming the key. An element that is empty after trimming is rejected the same way; it is not dropped. A value containing a newline or other control character is rejected with exit 4.
15. **No escaping or quoting.** Commas are separators only. Backslashes, single quotes, and double quotes have no escape meaning; `active\,archived` splits at the comma and is then rejected by status validation where the resulting elements are invalid. A value that must contain a comma can only be written by editing `papyrus.config.yml` directly. `config set --help` states this limitation.
16. **Flow-sequence persistence.** A list value is written to `papyrus.config.yml` as a YAML flow sequence, for example `statuses: [ToDo, Initiated, Done]`. `config get` prints the same `[a, b]` form in the same order, for both file-sourced block lists and written flow lists. `config list` prints the top-level `statuses` key and every declared `document_types.<slug>.statuses` key in that form; it does not print the key for an entry that declares no `statuses`. JSON output keeps list values as JSON arrays.
17. **Validate before write.** `config set` builds the configuration that would result, runs the ADR-0019 validator, and writes only if it is valid. A list that violates the status rules (empty, duplicate, missing `active`/`archived`, invalid status character, declared on `ui-mockup`, or a top-level `default_status` not in the new task list) is refused with exit 4 and the key named, and the existing file is byte-identical. When the existing file is otherwise invalid, the tolerant repair load applies: the change is applied and validated, and an invalid result is still refused.
18. **Scalar keys unchanged.** For a non-list key a comma remains an ordinary character of the scalar value; no existing `config set` invocation changes meaning.
19. **Unset unchanged.** `config unset` removes a list key; there is no clear-list operation and no empty list is written. The top-level key then falls back to the default task list. An absent `document_types.<slug>.statuses` declaration is reported by `config get` as a missing key (exit 3) and the effective document status model falls back to `active`/`archived`. Removing a declaration whose entry keeps a `default_status` that the fallback list would not contain is refused by the validator (exit 4) and the file is unchanged.
20. **Documentation.** `papyrus(1)` (`config set`) and `papyrus-config(5)` (`statuses`, `document_types`) state the comma-separated form, flow-sequence storage, empty-value rejection, and no-escape limitation. No new flag is introduced.
21. **Document-type default status in the flattened listing.** For each `document_types.<slug>` entry, `config list` emits the keys `prefix`, `directory`, `enabled`, then `statuses` if the entry declares a status list, then `default_status` if the entry declares a default status. The `default_status` line appears immediately after the same slug's `statuses` line when both exist, and immediately after `enabled` when the entry declares a default status but no status list. An entry that declares no `default_status` gets no such key and no empty or placeholder value. Slugs appear in ascending key order, as they already do. `--json` carries the same value at `document_types.<slug>.default_status` and omits the member when the entry declares none. Printing the key is a read-only listing concern: key parsing, validation, `config set`/`config unset`, `config get`, and the tolerant repair load are unchanged. **Retired (TASK-203, 2026-11-18, ADR-034, REQ-006.018):** the former clause also stated that the browser configuration view's own logic was unchanged and that, because that view rendered this same flattened listing, a declared `default_status` appeared there as a visible editable scalar row with no browser code change; the browser serves no configuration view, so no browser row exists. The `config list` behaviour above remains current.

### 4. Common CLI controls and outcomes

1. `--version` prints `papyrus <version>` and exits 0.
2. `-h`/`--help` prints help and exits 0; `papyrus <command> --help` prints command help.
3. `--json` emits stable machine-readable JSON for every output-producing command, including errors.
4. `--no-color` or a non-empty `NO_COLOR` disables ANSI color output.
5. Exit statuses: 0 success, 1 runtime error, 2 usage error, 3 record not found, 4 validation/conflict.
6. Usage errors (unknown flags/arguments, missing required arguments) print usage to stderr and exit 2.

### 5. Persistence and security

1. All data saved by Papyrus is UTF-8 text (YAML/Markdown); no binary data is written.
2. Outside `init`, Papyrus reads and writes only within the canonicalized selected project directory.
3. User-controlled input is sanitized by normalization/escaping before use; values that cannot be normalized into a valid form are rejected with exit code 4.
4. Security is explicitly considered in implementation and review (NFR-002.001).

## Inputs and outputs

- `init`: input project name and optional flags; output is the created/updated project structure and `papyrus.config.yml`. On success prints nothing or a short success message; errors to stderr.
- `config list`: output effective keys and values, including `document_types.<slug>.default_status` for each entry that declares one and omitting it otherwise.
- `config get <key>`: output one value. For a list key, output is the flow form `[a, b]`; with `--json`, a JSON array of strings.
- `config set <key> <value>`: persists value; success output minimal. A list key takes a comma-separated value and is stored as a flow sequence. When the key is a folder key, performs the relocation described above before persisting.
- `config unset <key>`: removes key; success output minimal.
- JSON mode wraps the above outputs as stable JSON, including errors.

## Validation and error behavior

- Missing/invalid command usage → exit code 2.
- Filesystem/IO failure or project-directory boundary violation → exit code 1.
- Missing configuration file (any non-`init` command) → exit code 1 (runtime error), message names the searched path and the `papyrus init` remedy.
- Unknown record/key or missing target → exit code 3 where applicable (for example `config get` of an unknown key).
- Validation/conflict (existing `papyrus.config.yml` without `--force`, duplicate statuses, invalid `default_status`, invalid padding, unnormalizable input, failed folder relocation, empty list value, empty list element, invalid document status list) → exit code 4.
- `--json` error output includes the exit-relevant error information.

## Assumptions

- Assumption: Rust 1.99.1 via rustup and Cargo are available (ADR-0001).
- Assumption: `git` is available in environments where `--no-git` is not used; otherwise init fails with a runtime error.
- Assumption: single-crate layout (`src/main.rs` + `src/lib.rs`) is retained; module boundaries beyond this are allowed within the crate but no new crate/workspace without an ADR.
- Assumption: `init` creates configured directories based on effective (default) configuration; it does not overwrite existing files other than `papyrus.config.yml` under `--force`.
- Assumption: configuration is read/written as YAML using a Rust YAML crate added as a runtime dependency; that crate's license must be audited for GPL-3.0-or-later compatibility before release (NFR-006.001/002).
- Assumption: input sanitization uses normalization/escaping (per user decision), not wholesale rejection, except where normalization cannot yield a valid value; list-valued status elements follow ADR-0024's reject-only behavior.
- Assumption: symlink resolution uses canonical paths for the project-directory boundary.
- Assumption: folder relocation uses the shared `record::relocate` routine (ADR-0013 stage 5) invoked by `config set`; the browser configuration view calls the same path. Retired (TASK-203, 2026-11-18, ADR-034, REQ-006.018): the browser serves no configuration view, so only the CLI `config set` caller remains; the CLI relocation assumption stays current.
- Assumption: an empty element after trimming (for example `a,,b`) is rejected rather than dropped, because silently persisting an empty task status would be invalid data; ADR-0024 rejects an empty list and leaves no drop rule.
- Assumption: flow rendering uses `[a, b]` (brackets, comma, single space), matching the existing top-level `statuses` rendering in `config list`.
- Assumption: a list element containing a newline or other control character is rejected (exit 4), because a single-line flow sequence cannot represent it unambiguously (NFR-002.002).

## Open questions

- None. OQ-1 (the directory set created by `init` and whether inactive subfolders are pre-created) is resolved by section 2 item 4: the exact directory set is specified, no `archived/`, `deprecated/`, or `backlog_directory/archive/` folder is pre-created, and the no-overwrite rule is unchanged.

## Traceability

| Artifact | Reference |
|---|---|
| Requirement | REQ-001, REQ-001.001, REQ-001.002, REQ-001.003, REQ-001.004, REQ-001.005, REQ-001.006, REQ-001.007, REQ-007.001, REQ-007.002 |
| Non-functional | NFR-002.001, NFR-002.002, NFR-004.001, NFR-004.002 |
| Business rule | BR-002 (init), BR-003 (config), BR-016 (keys/precedence), BR-001 (command behavior) |
| ADR | ADR-0001 (Rust), ADR-0003 (Debian Forky only; supersedes ADR-0002), ADR-0010 decision 8 (whole-folder relocation), ADR-0013 stage 5 (relocation implementation), ADR-0019 (fail-closed validation and repair), ADR-0022 (archive location, no restore), ADR-0023 decision 2 (active and archive locations), ADR-0024 (list-valued keys) |
| Ticket | TASK-2, TASK-85, TASK-111, TASK-112, TASK-122, TASK-127 |
| UI mockups | none |
path: docs/specifications/project-initialization-and-configuration.md
