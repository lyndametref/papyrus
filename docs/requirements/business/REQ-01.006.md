# REQ-01.006 Interactive Init Walkthrough at `papyrus init <project_name>`

## User Story
As a user managing project data, I want the initialization process to ask which artifact to activate before generating the corresponding `papyrus.config.yml`, so that commands operate using the intended project settings.

## Group
REQ-01 Project Setup and CLI Access

## Criticality
Should have

## Description
At `papyrus init <project_name>` (and with `--dir`/`PAPYRUS_DIR`), an interactive walkthrough runs before writing `papyrus.config.yml`. It presents the available artifacts as a single cursor-driven list: exactly one selectable row is highlighted at a time, starting on the first selectable artifact, the up and down arrow keys (or `k` and `j`) move the highlight to the previous or next selectable artifact, the space key toggles the highlighted artifact on or off, and Enter confirms the list and continues the walkthrough. The list redraws after every keypress so that the highlight and each artifact's `[x]`/`[ ]` state are always visible; navigating past the first or last artifact keeps the highlight on that artifact. Typed token input, such as space-separated artifact names entered on one text line, is not an acceptable presentation. The core artifacts task, milestone, and release are always active, are shown as such, and are never highlighted or toggled, because the configuration has no `enabled` flag for them (BR-016); their prefixes are still prompted. For each activated artifact the walkthrough prompts for its ID prefix, and for each activated decision or document type its storage folder; the shared roots (`backlog_directory`, `decision_directory`, `doc_directory`) are prompted once. Prompts show their default in brackets, and invalid input re-prompts with a hint naming the default equivalent value. End of input on the list confirms the current state and continues, so the walkthrough never waits on closed input.

Sensible defaults are kept for non-interactive/CI environments: interactive mode runs only when standard input is a terminal (TTY); otherwise the walkthrough is skipped and defaults are written without prompting. Artifact activation toggles the corresponding `enabled` flags, prefixes, and directories in configuration (BR-016). This requirement replaces the earlier behavior of writing defaults without prompts.

## Acceptance Criteria
- After running `papyrus init <name>`, the walkthrough presents all available artifacts as a single list with exactly one highlighted row as the cursor, starting on the first selectable artifact; the up and down arrow keys (or `k` and `j`) move the highlight, the space key toggles the highlighted artifact, and Enter confirms the list.
- Toggling is immediate and visible: the list redraws after each keypress so that the highlight and the `[x]`/`[ ]` state of every artifact reflect the current selection.
- The highlight never lands on task, milestone, or release, which are shown as always active, so space cannot toggle them and their prefixes are prompted without an activation decision; navigating past the first or last artifact keeps the highlight on that artifact.
- Enter confirms the current list state and the walkthrough continues with the prefix and folder prompts; the activation written to `papyrus.config.yml` equals the state shown when Enter was pressed.
- Artifact activation is never entered as typed tokens: the walkthrough does not prompt for artifact names on a text line, and end of input on the list confirms the current state and continues instead of waiting.
- For each activated artifact, the walkthrough prompts for its ID prefix before writing `papyrus.config.yml`; each activated decision or document type also has its storage folder prompted, and each shared root is prompted once.
- Details (prefix/folder) are prompted only for activated artifacts.
- Only activated artifacts are enabled in the generated `papyrus.config.yml`; inactive types are written with `enabled: false` (BR-016).
- Prompts show the default in brackets and accept it with empty input.
- Choices are validated: an ID prefix must be non-empty, contain no control characters, and contain no path separators; a folder must be non-empty, relative, and free of traversal segments and backslashes.
- An invalid prefix or folder is rejected and re-prompted with a hint naming the default equivalent value (for example `invalid prefix; use TASK`).
- The offered artifact set is task, milestone, release, the decision types (`adr`, `bdr`), and the document types defined in the configuration schema (`req`, `nfr`, `glossary`, `rule`, `ui-spec`, `ui-mockup`, `audit`, `deploy`, `tech-spec`, `use-case`, `test-case`, `test-report`), with the schema defaults as default selections.
- Interactive mode runs only when standard input is a terminal (TTY); otherwise the walkthrough is skipped and defaults are written without prompting (all artifacts activated, default prefixes/folders, `id_padding` 3), and no new CLI flag or environment variable is required.
- `--force` and `--no-git` continue to behave as documented (REQ-01.001).
- The generated configuration validates successfully (including `id_padding` range, REQ-01.007).

## Traceability
- Source: customer-feedback handoff — Internal team discussion, Item 8 (medium), raw quote:
  - "at papyrus init project_name command a walk through ask the user which artifact they want to activate for the project before generating the corresponding papapyrus.config.yml"
- Maintainer decision 2026-11-11: the single space-toggle list is the required presentation, replacing per-artifact `Activate? [Y/n]` prompts; task, milestone, and release are always active because the schema has no `enabled` flag for them; non-interactive runs apply defaults and introduce no new flags; validation rules and hint wording are fixed as stated.
- Source: customer-feedback handoff — in-session user feedback, Item 4 (severity unknown), raw quote:
  - "the init command does not work properly. it is not possible to select and artefact to activate or not the cursor stays after Toggle artifacts"
- Maintainer decision 2026-11-13: the space-toggle list is cursor-driven (highlighted row, arrow/`k`/`j` navigation, space toggles, Enter confirms); typed space-separated tokens are not an acceptable substitute. The delivered `papyrus init` behavior (typed-token prompt at src/init.rs) is a conformance gap against this requirement, not a missing requirement. The defect is not blocking because Enter keeps the defaults; no TTY fallback other than the existing non-interactive defaults is added.
- REQ-01.001 — Initialize a Papyrus Project
- REQ-01.002 — Read and Edit Project Configuration
- REQ-01.003 — Select Project and Configuration Context
- REQ-01.007 — ID Padding Constraint for File IDs
- REQ-05.012 — Manage Use Cases, REQ-05.013 — Manage Test Cases, REQ-05.014 — Manage Test Reports (document types offered in the list)
- BR-016 — Papyrus Configuration Keys and Precedence
- BR-018 — Use Case Document Type Behavior, BR-019 — Test Case Document Type Behavior, BR-020 — Test Report Document Type Behavior (document types offered in the list)
- docs/specifications/interactive-init-walkthrough-for-artifact-activation.md — Interactive Init Walkthrough for Artifact Activation
- Maintainer decision 2026-11-13 (validation): the list highlight starts on the first selectable artifact; the offered document-type set includes `use-case`, `test-case`, and `test-report` because the configuration schema defaults enable them; core-artifact prefixes are prompted even though core rows are never highlighted.
