---
id: TECH-047
title: "Interactive Init Walkthrough for Artifact Activation"
created_date: '2026-11-16T17:46:49Z'
updated_date: '2026-11-16T17:47:20Z'
labels: []
references:
  - ADR-0001
  - ADR-0004
  - ADR-0005
  - ADR-0010
  - ADR-0013
  - ADR-0015
  - ADR-0019
  - ADR-0022
  - ADR-0023
  - BR-002
  - BR-016
  - BR-018
  - BR-019
  - BR-020
  - REQ-01.001
  - REQ-01.002
  - REQ-01.003
  - REQ-01.006
  - REQ-01.007
  - REQ-05.008
  - REQ-05.012
  - REQ-05.013
  - REQ-05.014
  - REQ-07.002
  - TASK-104
  - TASK-112
  - TASK-127
  - TASK-2
  - TASK-74
  - TASK-75
  - TASK-85
status: verified
---
# TECH-047 Interactive Init Walkthrough for Artifact Activation

## Description
**Ticket:** TASK-75 / TASK-85 (spec revision) / TASK-104 (cursor-driven list revision) / TASK-112 (default-set and status-seeding revision) / TASK-127 (directory-creation revision)
**Milestone:** m-12 (Final Cross-Type Integration)
**Status:** Refined (execution-ready)
**UI mockups:** none

## Purpose

Deliver the interactive walkthrough at `papyrus init <project_name>` required by REQ-01.006 (and REQ-01.001, REQ-01.002, REQ-01.007, BR-016): before writing `papyrus.config.yml`, present every available artifact in a single cursor-driven list, let the user choose which to activate, and prompt for the ID prefix and storage folder of each activated artifact. Non-interactive/CI environments (no TTY) use sensible defaults without prompting. The generated configuration validates successfully, including the `id_padding` range.

This specification was revised (TASK-85) so that the walkthrough matches REQ-01.006: a single artifact list with task, milestone, and release always active; per-activated-artifact prefix/folder prompts; shared roots prompted once; defaults in brackets; validation hints naming the default equivalent; inactive types written with `enabled: false`; and TTY-only execution with no new flag or environment variable. It was revised again (TASK-104) to fix the presentation contract: the list is cursor-driven (one highlighted selectable row, Up/Down and `k`/`j` navigation, Space toggle, Enter confirm, redraw on every keypress, clamped navigation, core rows never highlighted, typed token input rejected, end of input confirming and continuing) and the offered document-type set includes `use-case`, `test-case`, and `test-report`.

It was revised again (TASK-112) so that the offered document-type set is exactly the eleven-entry default set of REQ-07.002 — `req`, `nfr`, `glossary`, `rule`, `ui-spec`, `ui-mockup`, `audit`, `tech-spec`, `use-case`, `test-case`, `test-report`, with `deploy` removed — and so that the generated configuration seeds the document status model: `statuses: [active, archived]` for every front-matter-backed document type, no `statuses` for `ui-mockup`, and no `default_status` on any entry.

It was revised again (TASK-127) so that section 6 states the settled directory-creation contract: only the activated type's active directory is created, `archived/` appears on the first archive, and no `deprecated/` subfolder and no `backlog_directory/archive/` folder is pre-created (ADR-0022, ADR-0023 decision 2).

## Scope

### In scope

- TTY detection and interactive vs. non-interactive execution paths in `init::run_init`.
- Presentation of all available artifacts as a single cursor-driven list:
  - Core artifacts (always active, shown as such, never highlighted or toggled): task, milestone, release.
  - Selectable decision types: `adr`, `bdr`.
  - Selectable document types: `req`, `nfr`, `glossary`, `rule`, `ui-spec`, `ui-mockup`, `audit`, `tech-spec`, `use-case`, `test-case`, `test-report`.
- Keyboard navigation of the list (Up/Down, `k`/`j`), Space toggle of the highlighted row, Enter confirm, redraw on every keypress, clamped navigation, and end-of-input confirmation.
- Per-activated-artifact ID prefix prompting with defaults and validation.
- Storage folder prompting for shared roots and per-document-type directories, for activated artifacts only.
- Activation flags written as the existing `enabled` field for decision and document types (BR-016); inactive types are written with `enabled: false`.
- Non-interactive/CI default behavior (all artifacts activated, default prefixes/folders, `id_padding` 3).
- Preservation of `--force` and `--no-git` behavior (REQ-01.001).
- Final configuration validation before writing (REQ-01.007).
- README/manpage (`papyrus(1)` init section) and BR-002 wording update describing the walkthrough.
- Directory creation limited to activated artifacts (core always; decision/document types only when activated) and to their active directories only: no `archived/`, no `deprecated/`, and no `backlog_directory/archive/` folder is pre-created by init.

### Out of scope

- New CLI flags or environment variables for non-interactive selection (defaults are used instead).
- Configuration schema changes or removal of `enabled` flags (see Open Questions).
- UI mockups (none for this ticket; CLI-only).
- Typed-token artifact activation (for example space-separated artifact names on one text line): explicitly rejected by REQ-01.006.
- Mouse input, scrolling beyond the visible artifact set, search/filter inside the list, and coloured/highlight styling beyond a visible cursor marker.
- Migration or re-padding of existing files.
- Git workflows beyond invoking repository initialization when `--no-git` is absent.

## Behavior

### 1. Mode selection

1. Interactive mode runs only when standard input is a terminal (TTY); otherwise the walkthrough is skipped and defaults are applied. No new CLI flag or environment variable is required or introduced.
2. In interactive mode, the artifact list is driven by individual keypresses, not by text lines (section 2). The prefix and folder prompts that follow the list are read line-by-line from `stdin`. An unexpected end of input on a prefix/folder prompt uses that prompt's default value and continues; the walkthrough never hangs.
3. `--force` and `--no-git` behave exactly as documented (REQ-01.001) regardless of mode.

### 2. Artifact list presentation (single cursor-driven list)

1. The walkthrough presents all available artifacts in a single vertical list with exactly one cursor: exactly one selectable row is highlighted at a time.
2. On entry the highlight is on the first selectable artifact (the first decision type, `adr`). The highlight is never on task, milestone, or release.
3. Selectable rows are the decision types and document types. The Space key toggles only the highlighted selectable row between active and inactive; toggling is immediate and is shown by the row's `[x]`/`[ ]` marker.
4. Task, milestone, and release are shown as always active because the configuration has no `enabled` flag for them (BR-016). They render with a fixed active marker and an "always active" indication, are never highlighted, and cannot be toggled.
5. The Up and Down arrow keys and the `k` and `j` keys move the highlight to the previous or next selectable artifact, skipping the core rows. Navigating past the first or last selectable artifact keeps the highlight on that artifact (clamped, no wrap-around).
6. Enter confirms the list. The walkthrough then continues with the prefix and folder prompts (section 3), and the activation written to `papyrus.config.yml` equals exactly the state shown when Enter was pressed.
7. The whole list is redrawn after every keypress so the cursor position and every row's `[x]`/`[ ]` state are always visible; no keypress is acknowledged by silent re-prompting.
8. End of input on the list (for example closed `stdin` or Ctrl-D) confirms the current state and continues with the remaining prompts, so the walkthrough never waits on closed input.
9. The list accepts no typed artifact tokens: there is no prompt that reads artifact names from a text line, and characters other than Space, `k`, `j`, and Enter have no effect on activation (they may be ignored, or cause a redraw only).
10. The offered artifact set is task, milestone, release, the decision types (`adr`, `bdr`), and the eleven default document types of REQ-07.002 (`req`, `nfr`, `glossary`, `rule`, `ui-spec`, `ui-mockup`, `audit`, `tech-spec`, `use-case`, `test-case`, `test-report`). `deploy` is not offered; it remains available only as a user-added document type declared in configuration after init.
11. The schema defaults are the default selections (all artifacts active).
12. Rows are listed in the configuration key order (core rows first, then decision types, then document types in schema order).

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
4. Activation is never an interactive yes/no question: the only activation input is the cursor-driven list (section 2). No prompt accepts `y`/`yes`/`n`/`no` or any artifact name for activation.

### 5. Configuration generation

1. The walkthrough builds a `Config` with `project_name` set to the sanitized name and `id_padding` 3 (default), then applies the collected prefixes, folders, and `enabled` flags.
2. Before writing, `Config::validate()` is called; a validation failure aborts `init` with exit code 4 and writes no configuration file.
3. The YAML is serialized and written to `papyrus.config.yml`: if the file already exists and `--force` is absent, `init` fails with a conflict (exit code 4) and writes nothing; under `--force` the existing file is replaced in full, with no value from the previous file kept. `--force` never overwrites any other pre-existing file (REQ-01.001).
4. Every default document type backed by front matter is written with `statuses: [active, archived]` in declared order; `ui-mockup` is written with no `statuses` key (its state lives in HTML metadata, REQ-05.008); no entry is written with `default_status`, so a new record starts at the effective default `active`.
5. Deactivating a document type in the list writes `enabled: false` and keeps the entry's prefix, directory, and seeded status list; it removes the type from management without touching existing files.

### 6. Directory creation

1. Core directories are always created: configured `backlog_directory` with its `tasks/`, `milestones/`, and `releases/` subfolders, and the configured `doc_directory`. No `archive/` folder is created under `backlog_directory`.
2. The configured `decision_directory` is created only if at least one decision type is activated; if none are activated it is not created, but its configured (default) value is still written to `papyrus.config.yml`.
3. For each activated document type, only that type's active directory (relative to `doc_directory`) is created.
4. An `archived/` subfolder is not pre-created for any type; it appears when the first record of that type is archived (ADR-0022 decision 2). No `deprecated/` subfolder is created for any type, and no `backlog_directory/archive/` folder is created by init: archive folders are created by the first archive operation of the family that uses them (ADR-0022, ADR-0023 decision 2).
5. Inactive artifacts' directories are not created by init.

### 7. Non-interactive/CI defaults

1. Interactive mode runs only when standard input is a terminal (TTY); otherwise the walkthrough is skipped and defaults are written without prompting.
2. All artifacts are activated.
3. All prefixes, folders, and `id_padding` use built-in defaults.
4. No prompts are emitted and no `stdin` reads occur.
5. No new CLI flag or environment variable is required.

## Inputs and outputs

- Input: project name (required), optional `--dir`, `--no-git`, `--force`, interactive keypresses for the artifact list (Up, Down, `k`, `j`, Space, Enter, end-of-input), and interactive line responses for the prefix and folder prompts.
- Output: created project structure and `papyrus.config.yml` containing the walkthrough results. The rendered list is written to standard output in redraws (the visible list reflects the last keypress). On success prints nothing further (or a short success message); errors to stderr.

## Validation and error behavior

- Missing/invalid command usage → exit code 2.
- Filesystem/IO failure or project-directory boundary violation → exit code 1.
- Existing `papyrus.config.yml` without `--force` → exit code 4.
- Invalid project name (unnormalizable) → exit code 4.
- Final configuration validation failure → exit code 4 (no file written).
- Interactive invalid prefix/folder → re-prompt (not an abort).
- Ctrl-C on the artifact list → exit code 130 (interrupt), no `papyrus.config.yml` written for that run, and the terminal input mode restored before exit.
- Terminal cbreak-mode entry failure (keypress navigation unavailable; ADR-0015) → one stderr warning, the list rendered once with all artifacts active, a single line-based "press Enter to continue" confirmation that cannot change activation, and the walkthrough then continues into the normal prefix/folder prompts. The walkthrough always terminates and never leaves the terminal in a modified input mode.

## Verification

1. Unit tests drive the list through an injected key source (a sequence of navigation, toggle, and confirm keys) and assert: the initial cursor row, movement with Up/Down and `k`/`j`, clamped movement at both ends, Space toggling only the highlighted selectable row, core rows unreachable, Enter confirming, and end-of-input confirming.
2. The end-to-end assertion is on the written `papyrus.config.yml`: `enabled` flags and prefixes/folders equal what the final rendered list showed at confirm time.
3. An integration test drives the real binary; the raw-keyboard path (real keypress handling, not the injected source) is verified on a real TTY and recorded in the ticket.
4. The non-interactive path (no TTY) is covered by existing tests and must stay unchanged: no prompts, all artifacts activated, defaults written.

## Assumptions

- Assumption: Task, milestone, and release remain always activated because the configuration schema has no `enabled` flag for them and BR-016 requires their prefixes to be non-empty.
- Assumption: Activation for decision and document types continues to use the existing `enabled` flag (BR-016); inactive types are written with `enabled: false`. Effective config still merges defaults, so retaining the entry (rather than omitting it) is what makes deactivation effective.
- Assumption: Interactive detection uses `stdin` TTY status (`std::io::IsTerminal`); no new flags or environment variables are introduced.
- Assumption: Defaults activate every artifact with built-in prefixes/folders and `id_padding` 3.
- Assumption: Prompt text and row ordering are implementation-defined beyond the artifact set and key order, but the artifact set and defaults follow `Config::default()`.
- Assumption: `Config::default()` is the single source of the default set, so it seeds the same eleven document types and the same status lists that `init` writes; when `document_types` is absent from a configuration file, the loader's whole-map default (ADR-0019 decision 8) supplies these seeded statuses too.
- Assumption: Only `ui-mockup` omits `statuses`; every other default document type is front-matter-backed and seeds `[active, archived]`.
- Assumption: No UI mockups are required (CLI-only ticket).
- Assumption: README/manpage init section and BR-002 are updated to describe the walkthrough; BR-016 and the enabled-flag model are unchanged.
- Assumption: The cursor-driven list replaces the earlier per-artifact `Activate? [Y/n]` prompts and the interim typed-token toggle line (REQ-01.006); only the prefix/folder prompts remain line-based.
- Assumption: The cursor marker is rendered with plain terminal characters (for example a `>` prefix) so the selected row is identifiable in captured output without relying on colour.
- Assumption: Terminal input handling stays dependency-free per ADR-0001/ADR-0004/ADR-0005 unless a new ADR approves otherwise (see TASK-104 architecture decision).

## Open questions

- OQ-1: The user expressed a preference for presence-based activation (an artifact is active when its configuration is present, with no `enabled` flag). This conflicts with BR-016 and with effective-config merge semantics (omitted types inherit default `enabled: true`). TASK-75 retains the existing `enabled` flags. If presence-based activation is required, it needs a separate task/ADR covering schema, loading, and command changes.
- OQ-2 (resolved by TASK-104): the presentation is a single cursor-driven list, not per-artifact sequential prompts. Only the exact prompt wording and the exact cursor glyph remain implementation-defined.
- OQ-3: Whether `id_padding` should also be prompted during the walkthrough is not specified; this spec keeps it at the default 3 (REQ-01.007) unless changed via `config set`.

## Traceability

| Artifact | Reference |
|---|---|
| Ticket | TASK-75, TASK-85, TASK-104 |
| Requirements | REQ-01.006, REQ-01.001, REQ-01.002, REQ-01.003, REQ-01.007, REQ-05.012, REQ-05.013, REQ-05.014 |
| Business rule | BR-016, BR-002, BR-018, BR-019, BR-020 |
| ADR | ADR-0001, ADR-0004, ADR-0005 (dependency baseline), ADR-0010 decision 8 (shared relocation routine), ADR-0013 stage 5, ADR-0022 (archive location and no restore), ADR-0023 decision 2 (active and archive locations) |
| Related tickets | TASK-2 (config foundation), TASK-74 (id_padding constraint), TASK-85 (spec revision), TASK-104 (cursor-driven list revision) |
| UI mockups | none |
path: docs/specifications/interactive-init-walkthrough-for-artifact-activation.md
