# REQ-01.006 Interactive Init Walkthrough at `papyrus init <project_name>`

## User Story
As a user managing project data, I want the initialization process to ask which artifact to activate before generating the corresponding `papyrus.config.yml`, so that commands operate using the intended project settings.

## Group
REQ-01 Project Setup and CLI Access

## Criticality
Should have

## Description
At `papyrus init <project_name>` (and with `--dir`/`PAPYRUS_DIR`), an interactive walkthrough runs before writing `papyrus.config.yml`. It presents all available artifacts (task, milestone, release, decision types, and document types), lets the user choose which to activate for the project, and then prompts for the ID prefix and storage folder for each activated artifact before generating `papyrus.config.yml`. Current initialization writes defaults without prompts; this requirement replaces that behavior with the walkthrough while retaining sensible defaults for non-interactive/CI environments. Artifact activation toggles the corresponding `enabled` flags, prefixes, and directories in configuration (BR-016).

## Acceptance Criteria
- After running `papyrus init <name>`, the walkthrough presents all available artifacts (task, milestone, release, decision types, document types) and prompts the user to select which to activate.
- For each activated artifact, the walkthrough prompts for its ID prefix and storage folder before writing `papyrus.config.yml`.
- Only activated artifacts are enabled in the generated `papyrus.config.yml`; inactive artifacts are disabled/omitted per configuration rules (BR-016).
- Invalid prefix/folder choices are rejected with hints naming equivalent values.
- Non-interactive / CI environments (no TTY) work with default selections and default prefixes/folders (or explicit flags) without prompting.
- `--force` and `--no-git` continue to behave as documented (REQ-01.001).
- The generated configuration validates successfully (including `id_padding` range, REQ-01.007).

## Open Questions
- Which artifacts to offer (task, milestone, release, document types, and others) and their default selections.
- Non-interactive/CI behavior (no TTY): skip prompts vs apply defaults vs require explicit flags.
- Validation of chosen prefix/folder: exact rules and hint wording for invalid choices.

## Traceability
- Source: customer-feedback handoff — Internal team discussion, Item 8 (medium), raw quote:
  - "at papyrus init project_name command a walk through ask the user which artifact they want to activate for the project before generating the corresponding papapyrus.config.yml"
- REQ-01.001 — Initialize a Papyrus Project
- REQ-01.002 — Read and Edit Project Configuration
- REQ-01.003 — Select Project and Configuration Context
- BR-016 — Papyrus Configuration Keys and Precedence
