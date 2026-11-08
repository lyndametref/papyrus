# REQ-08 Interactive Init Walkthrough at `papyrus init <project_name>`

## User Story
As a user managing project data, I want the initialization process to ask which artifact to activate before generating the corresponding `papyrus.config.yml`, so that commands operate using the intended project settings.

## Group
REQ-01 Project Setup and CLI Access

## Criticality
Medium

## Description
At `papyrus init <project_name>` (and with `--dir`/`PAPYRUS_DIR`), a walkthrough asks the user which artifact they want to activate for the project before generating the corresponding `papyrus.config.yml`. The command should present options (e.g., task, milestone, release, doc type) and prompt selection, defaulting to sensible defaults when none is provided. This affects how configuration keys and precedence are applied during initialization.

## Acceptance Criteria
- After running `papyrus init <name>`, the user is prompted/interactively asked which artifact to activate.
- Based on the selected artifact, `papyrus.config.yml` is generated accordingly.
- Default selections are defined (e.g., if no artifact specified, use built-in defaults).
- Non-interactive / CI environments (no TTY) must still work — either by skipping prompts or providing clear fallback behavior.
- The chosen prefix/folder validation is performed; invalid choices are rejected with hints naming equivalent values.

## Traceability
- Source: customer-feedback handoff — Internal team discussion, Item 8 (medium), raw quote:
  - "at papyrus init project_name command a walk through ask the user which artifact they want to activate for the project before generating the corresponding papapyrus.config.yml"
  - Open questions: which artifacts to offer, default selections, non-interactive/CI behavior (no TTY), defaults, validation of chosen prefix/folder.
