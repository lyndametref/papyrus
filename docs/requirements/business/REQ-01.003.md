# REQ-01.003 Select Project and Configuration Context

## User Story
As a user managing project data, I want to select the project directory and configuration file so that commands operate using the intended project settings.

## Group
REQ-01 Project Setup and CLI Access

## Criticality
Must have

## Description
Papyrus selects a project directory from `--dir`, `PAPYRUS_DIR`, or the current working directory, in that precedence order. Configuration selection uses `--config`, then `PAPYRUS_CONFIG`, then the project directory's `papyrus.config.yml`. If none of these yields an existing configuration file, every command except `papyrus init` fails: Papyrus does not fall back to built-in defaults and does not walk up directories or discover a configuration file elsewhere. Built-in defaults apply only to individual keys absent from a configuration file that was found. Command-line options take precedence over environment variables, project configuration, and defaults.

## Acceptance Criteria
- With no project-directory override, commands use the current working directory.
- `PAPYRUS_DIR` selects the default project directory, and `--dir` overrides it.
- The configuration file is selected from `--config`, then `PAPYRUS_CONFIG`, then the project directory's `papyrus.config.yml`.
- If none of those paths exists, every command except `papyrus init` fails; no built-in-defaults fallback and no directory walk-up or discovery is performed.
- An explicitly named `--config` or `PAPYRUS_CONFIG` path that does not exist counts as "not found" and fails the same way.
- The failure is a runtime error with exit code 1 (REQ-01.004); the message names the configuration path that was looked for (the resolved `--config`/`PAPYRUS_CONFIG` path when one is given, otherwise `<project directory>/papyrus.config.yml`) and states that `papyrus init` creates it.
- `papyrus init` runs without an existing configuration file.
- Built-in defaults apply only to individual keys absent from a configuration file that was found.
- A command-line option takes precedence over the corresponding environment, file, or built-in setting.

## Traceability
- README.md, `papyrus(1)` → DESCRIPTION and GLOBAL OPTIONS (lines 28–44), ENVIRONMENT (lines 845–854)
- README.md, `papyrus-config(5)` → DESCRIPTION and PRECEDENCE (lines 930–943)
- REQ-01.001 — Initialize a Papyrus Project (init creates the configuration file)
- REQ-01.004 — Use Common CLI Controls and Outcomes (runtime error exit code 1)
- Maintainer decision 2026-11-11: the missing-configuration failure applies to all non-`init` commands, is a runtime error (exit code 1), and its message names the searched path and the `papyrus init` remedy.
