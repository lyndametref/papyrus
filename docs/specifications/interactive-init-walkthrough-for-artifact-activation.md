# Feature Specification — Interactive Init Walkthrough for Artifact Activation

**Ticket:** TASK-75 / TASK-85 (spec revision)
**Milestone:** m-12 (Final Cross-Type Integration)
**Status:** Refined (execution-ready)
**UI mockups:** none

## Purpose

Deliver the interactive walkthrough at `papyrus init <project_name>` required by REQ-01.006 (and REQ-01.001, REQ-01.002, REQ-01.007, BR-016): before writing `papyrus.config.yml`, present every available artifact in a single space-toggle list, let the user choose which to activate, and prompt for the ID prefix and storage folder of each activated artifact. Non-interactive/CI environments (no TTY) use sensible defaults without prompting. The generated configuration validates successfully, including the `id_padding` range.

This specification was revised (TASK-85) so that the walkthrough matches REQ-01.006: a single space-toggle artifact list with task, milestone, and release always active; per-activated-artifact prefix/folder prompts; shared roots prompted once; defaults in brackets; validation hints naming the default equivalent; inactive types written with `enabled: false`; and TTY-only execution with no new flag or environment variable.

## Scope

### In scope

- TTY detection and interactive vs. non-interactive execution paths in `init::run_init`.
- Presentation of all available artifacts as a single space-toggle list:
  - Core artifacts (always active, shown as such): task, milestone, release.
  - Decision types: `adr`, `bdr`.
  - Document types: `req`, `nfr`, `glossary`, `rule`, `ui-spec`, `ui-mockup`, `audit`, `deploy`, `tech-spec`.
- Per-activated-artifact ID prefix prompting with defaults and validation.
- Storage folder prompting for shared roots and per-document-type directories, for activated artifacts only.
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

1. Interactive mode runs only when standard input is a terminal (TTY); otherwise the walkthrough is skipped and defaults are applied. No new CLI flag or environment variable is required or introduced.
2. In interactive mode, prompts are read line-by-line from `stdin`. An unexpected EOF on a prompt uses that prompt's default value and continues; the walkthrough never hangs.
3. `--force` and `--no-git` behave exactly as documented (REQ-01.001) regardless of mode.

### 2. Artifact list presentation (single space-toggle list)

1. The walkthrough presents all available artifacts in a single list. Decision types and document types can be toggled on/off with the space key.
2. Task, milestone, and release are shown as always active because the configuration has no `enabled` flag for them (BR-016). They cannot be toggled off.
3. The offered artifact set is task, milestone, release, the decision types (`adr`, `bdr`), and the document types defined in the configuration schema (`req`, `nfr`, `glossary`, `rule`, `ui-spec`, `ui-mockup`, `audit`, `deploy`, `tech-spec`), with the schema defaults as default selections.
4. The schema defaults are the default selections (all activated).

### 3. Shared roots and per-activated-artifact prompts

1. For each activated artifact, the walkthrough prompts for its ID prefix and storage folder before generating `papyrus.config.yml`. Details (prefix/folder) are prompted only for activated artifacts.
2. The shared roots are prompted once:
   - `backlog_directory` (default `backlog/`) is prompted once because it stores task, milestone, and release records.
   - `decision_directory` (default `decisions/`) is prompted once if at least one decision type is activated; if none are activated, the default is still written.
   - `doc_directory` (default `docs/`) is prompted once if at least one document type is activated; if none are activated, the default is still written.
3. Core artifacts (task, milestone, release) are always activated; their prefixes (`task_prefix` default `TASK`, `milestone_prefix` default `MS`, `release_prefix` default `REL`) are prompted, and the shared `backlog_directory` is prompted once.
4. For each activated decision type, its prefix is prompted; the shared `decision_directory` is prompted once (see above).
5. For each activated document type, its prefix and its directory relative to `doc_directory` are prompted; the shared `doc_directory` is prompted once (see above).
6. Inactive decision/document types are written with `enabled: false` (prefix retained as configured/default).

### 4. Prompt validation and re-prompting

1. Prompts show their default in brackets (for example `ID prefix [TASK]: `) and accept it with empty input.
2. ID prefixes are validated: non-empty, no control characters, no path separators. An invalid prefix is rejected and re-prompted with a hint naming the default equivalent value (for example `invalid prefix; use TASK`).
3. Storage folders are validated: non-empty, relative, free of traversal segments and backslashes. An invalid folder is rejected and re-prompted with a hint naming the default equivalent value (for example `invalid folder; use backlog/`).
4. Activation prompts (where applicable) accept `y`, `yes`, `n`, `no`, or empty (default yes); any other value re-prompts.

### 5. Configuration generation

1. The walkthrough builds a `Config` with `project_name` set to the sanitized name and `id_padding` 3 (default), then applies the collected prefixes, folders, and `enabled` flags.
2. Before writing, `Config::validate()` is called; a validation failure aborts `init` with exit code 4 and writes no configuration file.
3. The YAML is serialized and written to `papyrus.config.yml`: if the file already exists and `--force` is absent, `init` fails with a conflict (exit code 4) and writes nothing; under `--force` the existing file is replaced in full, with no value from the previous file kept. `--force` never overwrites any other pre-existing file (REQ-01.001).

### 6. Directory creation

1. Core directories are always created: configured `backlog_directory` with `tasks/`, `milestones/`, `releases/` subfolders; configured `doc_directory`.
2. The configured `decision_directory` is created only if at least one decision type is activated; if none are activated it is not created, but its configured (default) value is still written to `papyrus.config.yml`.
3. For document types, a type's directory (relative to `doc_directory`) and its `deprecated/` subfolder are created only if that type is activated.
4. Inactive artifacts' directories are not created by init.

### 7. Non-interactive/CI defaults

1. Interactive mode runs only when standard input is a terminal (TTY); otherwise the walkthrough is skipped and defaults are written without prompting.
2. All artifacts are activated.
3. All prefixes, folders, and `id_padding` use built-in defaults.
4. No prompts are emitted and no `stdin` reads occur.
5. No new CLI flag or environment variable is required.

## Inputs and outputs

- Input: project name (required), optional `--dir`, `--no-git`, `--force`, and interactive `stdin` responses.
- Output: created project structure and `papyrus.config.yml` containing the walkthrough results. On success prints nothing (or a short success message); errors to stderr.

## Validation and error behavior

- Missing/invalid command usage → exit code 2.
- Filesystem/IO failure or project-directory boundary violation → exit code 1.
- Existing `papyrus.config.yml` without `--force` → exit code 4.
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
- Assumption: The single space-toggle list presentation replaces the earlier per-artifact `Activate? [Y/n]` prompts (REQ-01.006).

## Open questions

- OQ-1: The user expressed a preference for presence-based activation (an artifact is active when its configuration is present, with no `enabled` flag). This conflicts with BR-016 and with effective-config merge semantics (omitted types inherit default `enabled: true`). TASK-75 retains the existing `enabled` flags. If presence-based activation is required, it needs a separate task/ADR covering schema, loading, and command changes.
- OQ-2: Exact prompt wording and whether prompts use a single multi-select list or per-artifact sequential prompts is implementation-defined; hidden/automated tests run in non-interactive mode (defaults).
- OQ-3: Whether `id_padding` should also be prompted during the walkthrough is not specified; this spec keeps it at the default 3 (REQ-01.007) unless changed via `config set`.

## Traceability

| Artifact | Reference |
|---|---|
| Ticket | TASK-75, TASK-85 |
| Requirements | REQ-01.006, REQ-01.001, REQ-01.002, REQ-01.003, REQ-01.007 |
| Business rule | BR-016, BR-002 |
| ADR | ADR-0010 decision 8 (shared relocation routine), ADR-0013 stage 5 |
| Related tickets | TASK-2 (config foundation), TASK-74 (id_padding constraint), TASK-85 (spec revision) |
| UI mockups | none |
path: docs/specifications/interactive-init-walkthrough-for-artifact-activation.md