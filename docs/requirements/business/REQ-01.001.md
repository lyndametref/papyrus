# REQ-01.001 Initialize a Papyrus Project

## User Story
As a project maintainer, I want to initialize Papyrus in a chosen directory so that the project has its backlog store, documentation structure, and configuration.

## Group
REQ-01 Project Setup and CLI Access

## Criticality
Must have

## Description
The user can initialize a project in the current directory or select a project directory with `--dir`; if it does not exist it is created. Initialization records the supplied project name in configuration, creates the backlog store and documentation structure, and creates `papyrus.config.yml`. `--no-git` skips Git initialization.

Initialization is allowed in a non-empty directory, and files other than `papyrus.config.yml` are never modified. An existing `papyrus.config.yml` blocks initialization unless `--force` is given, and `--force` overwrites that file completely with the generated configuration; it is a replacement, not a merge.

An interactive walkthrough (REQ-01.006) asks the user which artifact to activate for the project before generating the corresponding `papyrus.config.yml`. Defaults are provided for non-interactive/CI environments.

## Acceptance Criteria
- Running `papyrus init <name>` initializes the current directory and stores `<name>` as the project name.
- Running `papyrus --dir <path> init <name>` initializes `<path>`, creating it if absent.
- Initialization succeeds in a non-empty target directory without `--force`; every pre-existing file other than `papyrus.config.yml` remains unmodified.
- If `papyrus.config.yml` already exists and `--force` is absent, initialization fails with a conflict outcome (REQ-01.004) and writes nothing.
- If `papyrus.config.yml` already exists and `--force` is present, initialization overwrites that file in full with the generated configuration; no value from the previous file is kept.
- `--no-git` suppresses Git repository initialization.
- An interactive walkthrough presents all available artifacts, lets the user choose which to activate, and prompts for the prefix and folder of each before generating `papyrus.config.yml` (REQ-01.006).
- Non-interactive/CI environments (no TTY) work with default selections or explicit flags.

## Traceability
- README.md, `papyrus(1)` → COMMANDS → init (lines 57–69)
- REQ-01.004 — Use Common CLI Controls and Outcomes (conflict outcome for a blocked init)
- REQ-01.006 — Interactive Init Walkthrough at `papyrus init <project_name>`
- Maintainer decision 2026-11-11: `--force` overwrites an existing `papyrus.config.yml` completely (replacement, no merge); a non-empty directory alone does not require `--force`.
