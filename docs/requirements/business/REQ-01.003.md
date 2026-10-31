# REQ-01.003 Select Project and Configuration Context

## User Story
As a user managing project data, I want to select the project directory and configuration file so that commands operate using the intended project settings.

## Group
REQ-01 Project Setup and CLI Access

## Criticality
Must have

## Description
Papyrus selects a project directory from `--dir`, `PAPYRUS_DIR`, or the current working directory, in that precedence order. Configuration selection uses `--config`, `PAPYRUS_CONFIG`, the project directory's `papyrus.config.yml`, then built-in defaults. Command-line options take precedence over environment variables, project configuration, and defaults.

## Acceptance Criteria
- With no project-directory override, commands use the current working directory.
- `PAPYRUS_DIR` selects the default project directory, and `--dir` overrides it.
- The configuration file is selected from `--config`, then `PAPYRUS_CONFIG`, then the project directory's `papyrus.config.yml`.
- A command-line option takes precedence over the corresponding environment, file, or built-in setting.

## Traceability
- README.md, `papyrus(1)` → DESCRIPTION and GLOBAL OPTIONS (lines 28–44), ENVIRONMENT (lines 845–854)
- README.md, `papyrus-config(5)` → DESCRIPTION and PRECEDENCE (lines 930–943)
