# REQ-01.006 Interactive Init Walkthrough at `papyrus init <project_name>`

## User Story
As a user managing project data, I want the initialization process to ask which artifact to activate before generating the corresponding `papyrus.config.yml`, so that commands operate using the intended project settings.

## Group
REQ-01 Project Setup and CLI Access

## Criticality
Should have

## Description
At `papyrus init <project_name>` (and with `--dir`/`PAPYRUS_DIR`), an interactive walkthrough runs before writing `papyrus.config.yml`. It presents the available artifacts as a single list in which decision types and document types can be toggled on/off with space; the core artifacts task, milestone, and release are always active and are shown as such, because the configuration has no `enabled` flag for them (BR-016). For each activated artifact the walkthrough prompts for its ID prefix and storage folder before generating `papyrus.config.yml`; the shared roots (`backlog_directory`, `decision_directory`, `doc_directory`) are prompted once. Prompts show their default in brackets, and invalid input re-prompts with a hint naming the default equivalent value.

Sensible defaults are kept for non-interactive/CI environments: interactive mode runs only when standard input is a terminal (TTY); otherwise the walkthrough is skipped and defaults are written without prompting. Artifact activation toggles the corresponding `enabled` flags, prefixes, and directories in configuration (BR-016). This requirement replaces the earlier behavior of writing defaults without prompts.

## Acceptance Criteria
- After running `papyrus init <name>`, the walkthrough presents all available artifacts as a single list, where decision types and document types can be toggled on/off with space and task, milestone, and release are shown as always active.
- For each activated artifact, the walkthrough prompts for its ID prefix and storage folder before writing `papyrus.config.yml`; each shared root is prompted once.
- Details (prefix/folder) are prompted only for activated artifacts.
- Only activated artifacts are enabled in the generated `papyrus.config.yml`; inactive types are written with `enabled: false` (BR-016).
- Prompts show the default in brackets and accept it with empty input.
- Choices are validated: an ID prefix must be non-empty, contain no control characters, and contain no path separators; a folder must be non-empty, relative, and free of traversal segments and backslashes.
- An invalid prefix or folder is rejected and re-prompted with a hint naming the default equivalent value (for example `invalid prefix; use TASK`).
- The offered artifact set is task, milestone, release, the decision types (`adr`, `bdr`), and the document types defined in the configuration schema (`req`, `nfr`, `glossary`, `rule`, `ui-spec`, `ui-mockup`, `audit`, `deploy`, `tech-spec`), with the schema defaults as default selections.
- Interactive mode runs only when standard input is a terminal (TTY); otherwise the walkthrough is skipped and defaults are written without prompting (all artifacts activated, default prefixes/folders, `id_padding` 3), and no new CLI flag or environment variable is required.
- `--force` and `--no-git` continue to behave as documented (REQ-01.001).
- The generated configuration validates successfully (including `id_padding` range, REQ-01.007).

## Traceability
- Source: customer-feedback handoff — Internal team discussion, Item 8 (medium), raw quote:
  - "at papyrus init project_name command a walk through ask the user which artifact they want to activate for the project before generating the corresponding papapyrus.config.yml"
- Maintainer decision 2026-11-11: the single space-toggle list is the required presentation, replacing per-artifact `Activate? [Y/n]` prompts; task, milestone, and release are always active because the schema has no `enabled` flag for them; non-interactive runs apply defaults and introduce no new flags; validation rules and hint wording are fixed as stated.
- REQ-01.001 — Initialize a Papyrus Project
- REQ-01.002 — Read and Edit Project Configuration
- REQ-01.003 — Select Project and Configuration Context
- REQ-01.007 — ID Padding Constraint for File IDs
- BR-016 — Papyrus Configuration Keys and Precedence
