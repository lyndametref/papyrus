# REQ-01.001 Initialize a Papyrus Project

## User Story
As a project maintainer, I want to initialize Papyrus in a chosen directory so that the project has its backlog store, documentation structure, and configuration.

## Group
REQ-01 Project Setup and CLI Access

## Criticality
Must have

## Description
The user can initialize a project in the current directory or select a project directory with `--dir`; if it does not exists it is created. Initialization records the supplied project name in configuration, creates the backlog store and documentation structure, and creates `papyrus.config.yml`. `--no-git` skips Git initialization. `--force` permits initialization in a non-empty directory without overwriting existing files.

## Acceptance Criteria
- Running `papyrus init <name>` initializes the current directory and stores `<name>` as the project name.
- Running `papyrus --dir <path> init <name>` initializes `<path>`, creating it if absent.
- `--no-git` suppresses Git repository initialization.
- `--force` permits a non-empty target directory, and pre-existing files remain unmodified.

## Traceability
- README.md, `papyrus(1)` → COMMANDS → init (lines 57–69)
