# Feature Specification — Interactive Init Walkthrough for Artifact Activation

**Ticket:** TASK-75
**Milestone:** m-12 (Final Cross-Type Integration)
**Status:** Refined (execution-ready)
**UI mockups:** none

## Purpose

Deliver the interactive walkthrough at `papyrus init <project_name>` required by REQ-01.006 (and REQ-01.001, REQ-01.002, REQ-01.007, BR-016): before writing `papyrus.config.yml`, present every available artifact, let the user choose which to activate, and prompt for the ID prefix and storage folder of each activated artifact. Non-interactive/CI environments (no TTY) use sensible defaults without prompting. The generated configuration validates successfully, including the `id_padding` range.

## Scope

### In scope

- TTY detection and interactive vs. non-interactive execution paths in `init::run_init`.
- Presentation and activation of all available artifacts:
  - Core artifacts (always activated): task, milestone, release.
  - Decision types: `adr`, `bdr`.
  - Document types: `req`, `nfr`, `glossary`, `rule`, `ui-spec`, `ui-mockup`, `audit`, `deploy`, `tech-spec`.
- Per-activated-artifact ID prefix prompting with defaults and validation.
- Storage folder prompting for shared roots and per-document-type directories.
- Activation flags written as the existing `enabled` field for decision and document types (BR-016); inactive types are written with `enabled: false`.
- Non-interactive/CI default behavior (all artifacts activated, default prefixes/folders, `id_padding` 3).
- Preservation of `--force` and `--no-git` behavior (REQ-01.001).
- Final configuration validation before writing (REQ-01.007).
- README/manpage (`papyrus(1)` init section) and BR-002 wording update describing the walkthrough.
- Directory creation limited to activated artifacts (core always; decision/document types only when activated).

### Out of scope

- New CLI flags or environment variables for non-interactive selection (defaults are used instead).
- Changes to the configuration schema or removal of `enabled` flags (see Open Questions).
- UI mockups (none for this ticket; CLI-only).
- Migration or re-padding of existing files.
- Git workflows beyond invoking repository initialization when `--no-git` is absent.

## Behavior

### 1. Mode selection

1. Interactive mode runs only when `stdin` is a terminal (`std::io::IsTerminal`); otherwise the walkthrough is skipped and defaults are applied.
2. In interactive mode, prompts are read line-by-line from `stdin`. An unexpected EOF on a prompt uses that prompt's default value and continues; the walkthrough never hangs.
3. `--force` and `--no-git` behave exactly as documented (REQ-01.001) regardless of mode.

### 2. Core artifacts (task, milestone, release)

1. Task, milestone, and release are always activated: the configuration schema has no `enabled` flag for them and their prefixes are required (BR-016).
2. The walkthrough prompts for `task_prefix` (default `TASK`), `milestone_prefix` (default `MS`), and `release_prefix` (default `REL`), one per artifact, in that order.
3. The shared `backlog_directory` (default `backlog/`) is prompted once because it stores task, milestone, and release records.

### 3. Decision types (`adr`, `bdr`)

1. For each decision type in key order, the walkthrough prints the artifact name and its default prefix, then prompts `Activate? [Y/n]` (default `Y`).
2. An activated type is written with `enabled: true` and its prompted prefix; a deactivated type is written with `enabled: false` (prefix retained as configured/default).
3. All decision types share `decision_directory` (default `decisions/`), prompted once if at least one decision type is activated; if none are activated, the default is still written.

### 4. Document types

1. For each document type in key order, the walkthrough prints the artifact name, default prefix, and default directory, then prompts `Activate? [Y/n]` (default `Y`).
2. An activated type is written with `enabled: true`, its prompted prefix, and its prompted directory (relative to `doc_directory`); a deactivated type is written with `enabled: false`.
3. The documentation root `doc_directory` (default `docs/`) is prompted once if at least one document type is activated; if none are activated, the default is still written.
4. Each activated document type additionally prompts for its directory relative to `doc_directory` (default as in `Config::default()`).

### 5. Prompt validation and re-prompting

1. ID prefixes are validated with `sanitize::sanitize_name`: non-empty, no control characters, no path separators. An empty or invalid value re-prompts with a hint naming the default equivalent value (for example `invalid prefix; use TASK`).
2. Storage folders are validated with `sanitize::sanitize_path_string`: non-empty, relative, no traversal segments, no backslashes. An invalid value re-prompts with a hint naming the default equivalent value (for example `invalid folder; use backlog/`).
3. Activation prompts accept `y`, `yes`, `n`, `no`, or empty (default yes); any other value re-prompts.
4. Prompts display the default in brackets, e.g. `ID prefix [TASK]: `.

### 6. Configuration generation

1. The walkthrough builds a `Config` with `project_name` set to the sanitized name and `id_padding` 3 (default), then applies the collected prefixes, folders, and `enabled` flags.
2. Before writing, `Config::validate()` is called; a validation failure aborts `init` with exit code 4 and writes no configuration file.
3. The YAML is serialized and written only if `papyrus.config.yml` does not already exist (existing files are never overwritten, including under `--force`).

### 7. Directory creation

1. Core directories are always created: configured `backlog_directory` with `tasks/`, `milestones/`, `releases/` subfolders; configured `decision_directory`; configured `doc_directory`.
2. For decision types, `decision_directory` is created only if at least one decision type is activated.
3. For document types, a type's directory (and its `deprecated/` subfolder) is created only if that type is activated.
4. Inactive artifacts' directories are not created by init.

### 8. Non-interactive/CI defaults

1. All artifacts are activated.
2. All prefixes, folders, and `id_padding` use built-in defaults.
3. No prompts are emitted and no `stdin` reads occur.

## Inputs and outputs

- Input: project name (required), optional `--dir`, `--no-git`, `--force`, and interactive `stdin` responses.
- Output: created project structure and `papyrus.config.yml` containing the walkthrough results. On success prints nothing (or a short success message); errors to stderr.

## Validation and error behavior

- Missing/invalid command usage → exit code 2.
- Filesystem/IO failure or project-directory boundary violation → exit code 1.
- Non-empty target directory without `--force` → exit code 4.
- Invalid project name (unnormalizable) → exit code 4.
- Final configuration validation failure → exit code 4 (no file written).
- Interactive invalid prefix/folder → re-prompt (not an abort).

## Assumptions

- Assumption: Task, milestone, and release remain always activated because the configuration schema has no `enabled` flag for them and BR-016 requires their prefixes to be non-empty.
- Assumption: Activation for decision and document types continues to use the existing `enabled` flag (BR-016); inactive types are written with `enabled: false`. Effective config still merges defaults, so retaining the entry (rather than omitting it) is what makes deactivation effective.
- Assumption: Interactive detection uses `stdin` TTY status (`std::io::IsTerminal`); no new flags or environment variables are introduced.
- Assumption: Defaults activate every artifact with built-in prefixes/folders and `id_padding` 3.
- Assumption: Prompt text and ordering beyond the artifact key order are implementation-defined, but the artifact set and defaults follow `Config::default()`.
- Assumption: No UI mockups are required (CLI-only ticket).
- Assumption: README/manpage init section and BR-002 are updated to describe the walkthrough; BR-016 and the enabled-flag model are unchanged.

## Open questions

- OQ-1: The user expressed a preference for presence-based activation (an artifact is active when its configuration is present, with no `enabled` flag). This conflicts with BR-016 and with effective-config merge semantics (omitted types inherit default `enabled: true`). TASK-75 retains the existing `enabled` flags. If presence-based activation is required, it needs a separate task/ADR covering schema, loading, and command changes.
- OQ-2: Exact prompt wording and whether prompts use a single multi-select list or per-artifact sequential prompts is implementation-defined; hidden/automated tests run in non-interactive mode (defaults).
- OQ-3: Whether `id_padding` should also be prompted during the walkthrough is not specified; this spec keeps it at the default 3 (REQ-01.007) unless changed via `config set`.

## Traceability

| Artifact | Reference |
|---|---|
| Ticket | TASK-75 |
| Requirements | REQ-01.006, REQ-01.001, REQ-01.002, REQ-01.007 |
| Business rule | BR-016, BR-002 |
| Related tickets | TASK-2 (config foundation), TASK-74 (id_padding constraint) |
| UI mockups | none |
