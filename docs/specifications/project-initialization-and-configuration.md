# Feature Specification — Project Initialization and Configuration Commands

**Ticket:** TASK-2 / TASK-85 (spec revision); TASK-111 (list-valued configuration keys revision)
**Milestone:** m-0 (Rust CLI Foundation and Requirements)
**Status:** Refined (execution-ready)
**UI mockups:** none

## Purpose

Deliver the `init` and `config` commands, plus project/configuration context selection and the common CLI controls and outcomes defined by REQ-01, so a maintainer can create a project context, select it deterministically, inspect and change effective configuration, and rely on consistent command behavior and exit statuses. This ticket also delivers the configuration keys that govern stable record IDs (prefixes and padding); record commands in later tickets consume those keys.

This specification was revised (TASK-85) so that initialization, missing-configuration failure, and folder relocation match REQ-01.001, REQ-01.002, REQ-01.003 and ADR-0010 decision 8 (whole-folder relocation), including the browser configuration view as a consumer of the same `config set` path.

It was revised again (TASK-111) so that the list-valued keys `statuses` and `document_types.<slug>.statuses` can be written through `config set` using a comma-separated value stored as a YAML flow sequence, and read back by `config get` and `config list` (REQ-01.002, REQ-07.002, ADR-0024).

## Scope

### In scope

- `papyrus init <name> [--no-git] [--force]` (BR-002, REQ-01.001, REQ-01.006).
- `papyrus config list|get|set|unset` with dotted keys and YAML-scalar values (BR-003, REQ-01.002).
- List-valued keys `statuses` and `document_types.<slug>.statuses`: comma-separated values on `config set`, YAML flow-sequence storage, flow-form output from `config get`/`config list`, empty-value rejection, and validation of the resulting configuration before persistence (REQ-01.002, REQ-07.002, ADR-0024).
- Project directory selection: `--dir`, `PAPYRUS_DIR`, then current working directory, in that precedence order (REQ-01.003).
- Configuration file selection: `--config`, then `PAPYRUS_CONFIG`, then `<project_dir>/papyrus.config.yml` (REQ-01.003). No built-in-defaults fallback and no directory walk-up/discovery when none is found.
- Common CLI controls and outcomes: `--json` (including errors), `--no-color` / `NO_COLOR`, `--version`, `-h`/`--help`, command help, and exit statuses 0/1/2/3/4 (REQ-01.004).
- Configuration keys and validation from BR-016, including task statuses uniqueness/default-status membership, default statuses `ToDo, Initiated, Refined, Implemented, Validated, Done`, and ID prefix/padding keys (REQ-01.002, REQ-01.005, REQ-01.007).
- File-access boundary: init is the sole exception to the project-directory restriction; all other reads/writes stay within the selected project directory, resolved canonically (NFR-04.002).
- Persistence format: all data saved by Papyrus is UTF-8 text (YAML/Markdown); no binary artifacts (NFR-04.001).
- Input sanitization by normalization/escaping before use (NFR-02.001, NFR-02.002).
- Atomic whole-folder relocation during `config set` for `backlog_directory`, `decision_directory`, `doc_directory`, and any document type's `directory` (REQ-01.002, ADR-0010 decision 8, ADR-0013 stage 5).

### Out of scope

- Record-management subcommands (`task`, `milestone`, `release`, `decision`, document types, `search`, `browser`) — TASK-3 onward.
- Actual record ID assignment and archive/restore preservation logic — consumed by record commands in later tickets; this ticket delivers their configuration keys only.
- Git workflows beyond invoking repository initialization when `--no-git` is absent.
- UI mockups or web UI — none for this ticket.
- List editing in the browser configuration view: the view's editable-key whitelist is unchanged, and a list key remains non-editable there (CLI `config set` only).
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
3. Initialization runs the interactive walkthrough (REQ-01.006) to select activated artifacts and collect their prefixes/folders (TTY mode; defaults in non-TTY mode) before creating the backlog store, the documentation structure, and `papyrus.config.yml`.
4. Created directories are derived from effective configuration (defaults when no config exists yet): at minimum the configured backlog directory and documentation root directory.
5. `--no-git` suppresses Git repository initialization.
6. Initialization succeeds in a non-empty target directory. Files other than `papyrus.config.yml` are never modified by `init`.
7. An existing `papyrus.config.yml` blocks initialization unless `--force` is given. Without `--force`, initialization fails with a conflict outcome (exit code 4) and writes nothing.
8. With `--force`, initialization overwrites `papyrus.config.yml` in full with the generated configuration; no value from the previous file is kept. `--force` is a replacement, not a merge. `--force` does not permit overwriting any other pre-existing file.
9. `init` is the only command permitted to read from or write to a path outside an already-selected project directory (it may create the target directory).
10. All files written by init are UTF-8 text.
11. User-controlled input (`<name>`, paths) is sanitized by normalization/escaping before use; if normalization cannot produce a valid value, the command fails with exit code 4.

### 3. `config`

1. `papyrus config list` prints all effective keys, including defaults, in a stable order.
2. `papyrus config get <key>` prints one effective value.
3. `papyrus config set <key> <value>` persists one value. For a scalar key, `<value>` is parsed as a YAML scalar, unchanged from the current behavior. For a list-valued key, `<value>` is a comma-separated list (see items 13–16).
4. `papyrus config unset <key>` removes the configured key so its default applies.
5. Nested keys use dotted notation (for example `document_types.req.enabled`).
6. Keys and defaults match BR-016 (project name, prefixes, padding, date format, editor, paths, statuses, default status, decision/document types).
7. Configured task statuses must be unique; `default_status` must be one of them.
8. With default settings, new tasks start at `ToDo` and the ordered statuses are `ToDo`, `Initiated`, `Refined`, `Implemented`, `Validated`, `Done`.
9. With default configuration, task, milestone, and release IDs use prefixes `TASK`, `MS`, `REL` and three-digit padding (`TASK-001`, `MS-001`, `REL-001`); `id_padding` is constrained to 1–6 (REQ-01.007) and IDs wider than the padding are not truncated.
10. Invalid key, invalid value, or validation failure returns exit code 4 (validation/conflict).
11. **Whole-folder relocation on a folder change.** When `papyrus config set` changes `backlog_directory`, `decision_directory`, `doc_directory`, or a document type's `directory`, the entire content of the original (source) folder is moved to the new folder:
    - The move covers every file and subfolder of the source folder, not only Papyrus-managed records.
    - The move happens during the `set` command, synchronously, not lazily and not on a later read.
    - The move is atomic: the shared relocation routine (ADR-0010 decision 8, ADR-0013 stage 5) pre-checks the whole plan (destination entry conflicts, containment, permissions) before moving anything.
    - If any entry cannot be moved (for example an entry with the same relative path already exists at the destination, or a write failure), nothing is moved, both folders are left as they were, the configuration value is not persisted, and the command reports a conflict (exit code 4).
    - On success the source folder no longer holds the moved content and is removed once empty.
    - Cross-device moves fall back to the atomic copy-then-remove sequence and are reported as non-atomic in the command output if a single rename is impossible.
12. The browser configuration view (REQ-06.008) is a consumer of this same `config set` path: it invokes the relocation routine through `config set` and may not move folders itself. It performs no folder move of its own.
13. **List-key classification.** A single declared classification names the list-valued keys: the top-level task `statuses` and each `document_types.<slug>.statuses`. Every other key is scalar, including `default_status`, `document_types.<slug>.default_status`, `prefix`, `directory`, `enabled`, `id_padding`, and `browser_*`. Classification depends only on the key; it never inspects the shape of the supplied value.
14. **Comma-separated form.** For a list-valued key, `config set` splits the raw value on `,`, trims surrounding whitespace from each element, preserves element order, case, internal whitespace, and duplicates, and does not interpret YAML in the value (`true`, `1`, and `[a]` remain strings). An empty or whitespace-only value is rejected with exit 4 and a message naming the key. An element that is empty after trimming is rejected the same way; it is not dropped. A value containing a newline or other control character is rejected with exit 4.
15. **No escaping or quoting.** Commas are separators only. Backslashes, single quotes, and double quotes have no escape meaning; `active\,archived` splits at the comma and is then rejected by status validation where the resulting elements are invalid. A value that must contain a comma can only be written by editing `papyrus.config.yml` directly. `config set --help` states this limitation.
16. **Flow-sequence persistence.** A list value is written to `papyrus.config.yml` as a YAML flow sequence, for example `statuses: [ToDo, Initiated, Done]`. `config get` prints the same `[a, b]` form in the same order, for both file-sourced block lists and written flow lists. `config list` prints the top-level `statuses` key and every declared `document_types.<slug>.statuses` key in that form; it does not print the key for an entry that declares no `statuses`. JSON output keeps list values as JSON arrays.
17. **Validate before write.** `config set` builds the configuration that would result, runs the ADR-0019 validator, and writes only if it is valid. A list that violates the status rules (empty, duplicate, missing `active`/`archived`, invalid status character, declared on `ui-mockup`, or a top-level `default_status` not in the new task list) is refused with exit 4 and the key named, and the existing file is byte-identical. When the existing file is otherwise invalid, the tolerant repair load applies: the change is applied and validated, and an invalid result is still refused.
18. **Scalar keys unchanged.** For a non-list key a comma remains an ordinary character of the scalar value; no existing `config set` invocation changes meaning.
19. **Unset unchanged.** `config unset` removes a list key; there is no clear-list operation and no empty list is written. The top-level key then falls back to the default task list. An absent `document_types.<slug>.statuses` declaration is reported by `config get` as a missing key (exit 3) and the effective document status model falls back to `active`/`archived`. Removing a declaration whose entry keeps a `default_status` that the fallback list would not contain is refused by the validator (exit 4) and the file is unchanged.
20. **Documentation.** `papyrus(1)` (`config set`) and `papyrus-config(5)` (`statuses`, `document_types`) state the comma-separated form, flow-sequence storage, empty-value rejection, and no-escape limitation. No new flag is introduced.

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
4. Security is explicitly considered in implementation and review (NFR-02.001).

## Inputs and outputs

- `init`: input project name and optional flags; output is the created/updated project structure and `papyrus.config.yml`. On success prints nothing or a short success message; errors to stderr.
- `config list`: output effective keys and values.
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
- Assumption: configuration is read/written as YAML using a Rust YAML crate added as a runtime dependency; that crate's license must be audited for GPL-3.0-or-later compatibility before release (NFR-06.001/002).
- Assumption: input sanitization uses normalization/escaping (per user decision), not wholesale rejection, except where normalization cannot yield a valid value; list-valued status elements follow ADR-0024's reject-only behavior.
- Assumption: symlink resolution uses canonical paths for the project-directory boundary.
- Assumption: folder relocation uses the shared `record::relocate` routine (ADR-0013 stage 5) invoked by `config set`; the browser configuration view calls the same path.
- Assumption: an empty element after trimming (for example `a,,b`) is rejected rather than dropped, because silently persisting an empty task status would be invalid data; ADR-0024 rejects an empty list and leaves no drop rule.
- Assumption: flow rendering uses `[a, b]` (brackets, comma, single space), matching the existing top-level `statuses` rendering in `config list`.
- Assumption: a list element containing a newline or other control character is rejected (exit 4), because a single-line flow sequence cannot represent it unambiguously (NFR-02.002).

## Open questions

- OQ-1: The exact set of directories/files that constitute the "documentation structure" created by `init` is not specified by the man page; implementation creates configured directories as needed and must not overwrite existing files. Residual detail (for example whether deprecated subfolders are pre-created) is implementation-defined and should be documented.

## Traceability

| Artifact | Reference |
|---|---|
| Requirement | REQ-01, REQ-01.001, REQ-01.002, REQ-01.003, REQ-01.004, REQ-01.005, REQ-01.006, REQ-01.007, REQ-07.002 |
| Non-functional | NFR-02.001, NFR-02.002, NFR-04.001, NFR-04.002 |
| Business rule | BR-002 (init), BR-003 (config), BR-016 (keys/precedence), BR-001 (command behavior) |
| ADR | ADR-0001 (Rust), ADR-0003 (Debian Forky only; supersedes ADR-0002), ADR-0010 decision 8 (whole-folder relocation), ADR-0013 stage 5 (relocation implementation), ADR-0019 (fail-closed validation and repair), ADR-0024 (list-valued keys) |
| Ticket | TASK-2, TASK-85, TASK-111 |
| UI mockups | none |
path: docs/specifications/project-initialization-and-configuration.md