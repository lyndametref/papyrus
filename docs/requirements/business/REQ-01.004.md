# REQ-01.004 Use Common CLI Controls and Outcomes

## User Story
As a Papyrus user or command-line integration, I want consistent common options and exit outcomes so that I can operate the tool and recognize command results.

## Group
REQ-01 Project Setup and CLI Access

## Criticality
Must have

## Description
Papyrus provides global JSON, color, version, and help options. `--json` applies to commands that produce output, including errors. Color can also be disabled with `NO_COLOR`. The man page defines distinct exit statuses for success, runtime error, usage error, not-found, and validation/conflict outcomes. An invalid effective configuration is a validation outcome reported before any command work begins, except for the repair surfaces (REQ-01.002, REQ-07.002).

## Acceptance Criteria
- `--version` prints the version and exits; `-h`/`--help` prints help, and command help is available as `papyrus <command> --help`.
- `--json` produces machine-readable JSON for output-producing commands, including error output.
- `--no-color` or setting `NO_COLOR` disables ANSI color output.
- Success, runtime error, usage error, record-not-found, and validation/conflict outcomes use exit codes 0, 1, 2, 3, and 4 respectively.
- An invalid effective configuration produces a validation outcome (exit code 4) before command dispatch, except for `config get`, `config set`, `config unset`, `config list`, and `init --force` (REQ-01.002, REQ-07.002).

## Traceability
- README.md, `papyrus(1)` → GLOBAL OPTIONS (lines 35–53), ENVIRONMENT (lines 853–854), EXIT STATUS (lines 859–874)
- REQ-01.002 — Read and Edit Project Configuration (repair surfaces)
- REQ-07.002 — Configure Document Types and Slugs (fail-closed configuration validation)
- Project-owner decision 2026-11-13 (D4): the validation outcome for an invalid configuration applies to every command except the repair surfaces.
