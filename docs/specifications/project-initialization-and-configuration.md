# Feature Specification — Project Initialization and Configuration Commands

**Ticket:** TASK-2
**Milestone:** m-0 (Rust CLI Foundation and Requirements)
**Status:** Refined (execution-ready)
**UI mockups:** none

## Purpose

Deliver the `init` and `config` commands, plus project/configuration context selection and the common CLI controls and outcomes defined by REQ-01, so a maintainer can create a project context, select it deterministically, inspect and change effective configuration, and rely on consistent command behavior and exit statuses. This ticket also delivers the configuration keys that govern stable record IDs (prefixes and padding); record commands in later tickets consume those keys.

## Scope

### In scope

- `papyrus init <name> [--no-git] [--force]` (BR-002, REQ-01.001).
- `papyrus config list|get|set|unset` with dotted keys and YAML-scalar values (BR-003, REQ-01.002).
- Project directory selection: `--dir`, `PAPYRUS_DIR`, then current working directory, in that precedence order (REQ-01.003).
- Configuration file selection: `--config`, `PAPYRUS_CONFIG`, then `<project_dir>/papyrus.config.yml`, then built-in defaults (REQ-01.003, BR-016).
- Common CLI controls and outcomes: `--json` (including errors), `--no-color` / `NO_COLOR`, `--version`, `-h`/`--help`, command help, and exit statuses 0/1/2/3/4 (REQ-01.004).
- Configuration keys and validation from BR-016, including task statuses uniqueness/default-status membership, default statuses `ToDo, Initiated, Refined, Implemented, Validated, Done`, new-task default `ToDo`, and ID prefix/padding keys (REQ-01.002, REQ-01.005).
- File-access boundary: init is the sole exception to the project-directory restriction; all other reads/writes stay within the selected project directory, resolved canonically (NFR-04.002).
- Persistence format: all data saved by Papyrus is UTF-8 text (YAML/Markdown); no binary artifacts (NFR-04.001).
- Input sanitization by normalization/escaping before use (NFR-02.001, NFR-02.002).

### Out of scope

- Record-management subcommands (`task`, `milestone`, `release`, `decision`, document types, `search`, `browser`) — TASK-3 onward.
- Actual record ID assignment and archive/restore preservation logic — consumed by record commands in later tickets; this ticket delivers their configuration keys only.
- Git workflows beyond invoking repository initialization when `--no-git` is absent.
- UI mockups or web UI — none for this ticket.
- Non-amd64 or non-Debian distribution concerns — ADR-0002.

## Behavior

### 1. Project and configuration context selection

1. The project directory is selected from `--dir`, then `PAPYRUS_DIR`, then the current working directory.
2. The configuration file is selected from `--config`, then `PAPYRUS_CONFIG`, then `<project_dir>/papyrus.config.yml`.
3. If no configuration file is found, built-in defaults apply.
4. Command-line options take precedence over environment variables, project configuration, and defaults.
5. For every command except `init`, the selected project directory is canonicalized (symlinks resolved) and all reads/writes must remain within it. A path that resolves outside it is rejected.

### 2. `init`

1. `papyrus init <name>` initializes the current directory; `papyrus --dir <path> init <name>` initializes `<path>`, creating it if absent.
2. `<name>` is required and is stored as `project_name` in `papyrus.config.yml`.
3. Initialization creates the backlog store, the documentation structure, and `papyrus.config.yml`.
4. Created directories are derived from effective configuration (defaults when no config exists yet): at minimum the configured backlog directory and documentation root directory.
5. `--no-git` suppresses Git repository initialization.
6. `--force` permits initialization in a non-empty directory. Existing files are never overwritten.
7. Without `--force`, initializing a non-empty directory fails with a validation/conflict error (exit code 4).
8. `init` is the only command permitted to read from or write to a path outside an already-selected project directory (it may create the target directory).
9. All files written by init are UTF-8 text.
10. User-controlled input (`<name>`, paths) is sanitized by normalization/escaping before use; if normalization cannot produce a valid value, the command fails with exit code 4.

### 3. `config`

1. `papyrus config list` prints all effective keys, including defaults, in a stable order.
2. `papyrus config get <key>` prints one effective value.
3. `papyrus config set <key> <value>` persists one value; `<value>` is parsed as a YAML scalar.
4. `papyrus config unset <key>` removes the configured key so its default applies.
5. Nested keys use dotted notation (for example `document_types.req.enabled`).
6. Keys and defaults match BR-016 (project name, prefixes, padding, date format, editor, paths, statuses, default status, decision/document types).
7. Configured task statuses must be unique; `default_status` must be one of them.
8. With default settings, new tasks start at `ToDo` and the ordered statuses are `ToDo`, `Initiated`, `Refined`, `Implemented`, `Validated`, `Done`.
9. With default configuration, task, milestone, and release IDs use prefixes `TASK`, `MS`, `REL` and three-digit padding (`TASK-001`, `MS-001`, `REL-001`); `id_padding` is a positive integer and IDs wider than the padding are not truncated.
10. Invalid key, invalid value, or validation failure returns exit code 4 (validation/conflict).

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
- `config get <key>`: output one value.
- `config set <key> <value>`: persists value; success output minimal.
- `config unset <key>`: removes key; success output minimal.
- JSON mode wraps the above outputs as stable JSON, including errors.

## Validation and error behavior

- Missing/invalid command usage → exit code 2.
- Filesystem/IO failure or project-directory boundary violation → exit code 1.
- Unknown record/key or missing target → exit code 3 where applicable (for example `config get` of an unknown key).
- Validation/conflict (non-empty dir without `--force`, duplicate statuses, invalid `default_status`, invalid padding, unnormalizable input) → exit code 4.
- `--json` error output includes the exit-relevant error information.

## Assumptions

- Assumption: Rust 1.99.1 via rustup and Cargo are available (ADR-0001).
- Assumption: `git` is available in environments where `--no-git` is not used; otherwise init fails with a runtime error.
- Assumption: single-crate layout (`src/main.rs` + `src/lib.rs`) is retained; module boundaries beyond this are allowed within the crate but no new crate/workspace without an ADR.
- Assumption: `init` creates configured directories based on effective (default) configuration; it does not overwrite existing files.
- Assumption: configuration is read/written as YAML using a Rust YAML crate added as a runtime dependency; that crate's license must be audited for GPL-3.0-or-later compatibility before release (NFR-06.001/002).
- Assumption: input sanitization uses normalization/escaping (per user decision), not wholesale rejection, except where normalization cannot yield a valid value.
- Assumption: symlink resolution uses canonical paths for the project-directory boundary.

## Open questions

- OQ-1: The exact set of directories/files that constitute the "documentation structure" created by `init` is not specified by the man page; implementation creates configured directories as needed and must not overwrite existing files. Residual detail (for example whether deprecated subfolders are pre-created) is implementation-defined and should be documented.

## Traceability

| Artifact | Reference |
|---|---|
| Requirement | REQ-01, REQ-01.001, REQ-01.002, REQ-01.003, REQ-01.004, REQ-01.005 |
| Non-functional | NFR-02.001, NFR-02.002, NFR-04.001, NFR-04.002 |
| Business rule | BR-002 (init), BR-003 (config), BR-016 (keys/precedence), BR-001 (command behavior) |
| ADR | ADR-0001 (Rust), ADR-0003 (Debian Forky only; supersedes ADR-0002) |
| Ticket | TASK-2 |
| UI mockups | none |
path: docs/specifications/project-initialization-and-configuration.md