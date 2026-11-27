---
id: TECH-082
title: "CLI Foundation and Context"
created_date: '2026-11-25T20:41:41Z'
updated_date: '2026-11-27T17:25:34Z'
labels: []
references:
  - REQ-001
  - REQ-001.003
  - REQ-001.004
  - NFR-002
  - NFR-002.002
  - NFR-004.002
status: verified
---
# TECH-082 CLI Foundation and Context

## Description
Covers the CLI command surface, common options, and context selection. Input sanitization, configuration, and validation are separate specifications.

## Command Surface

- The available commands are the built-in commands `init`, `config`, `task`, `milestone`, `release`, `search`, `import`, `help`, and `version`, plus one command family for every configured document type and decision type. The web UI is served by the separate `papyrus-browser` binary.
- Help output — `-h`/`--help`, per-command help, and `papyrus help` — is specified in the user-documentation specification (REQ-008).
- The global options `--json`, `--no-color`, `--version`, `--dir`, and `--config` are accepted with every command that can honour them.
- A command-line option takes precedence over the corresponding environment variable, configuration value, and default.

## Startup Sequence

- Phase 1 is a lexical scan of `argv`, not a parser. It reads only `--dir <path>`, `--dir=<path>`, `--config <path>`, `--config=<path>`, `PAPYRUS_DIR`, and `PAPYRUS_CONFIG`, and stops at `--`.
- The scan resolves the project directory and the configuration path before any command tree is built.
- Phase 2 loads and fully validates the selected configuration for every command that reads it.
- Phase 3 builds the command tree from the validated configuration and parses the invocation exactly once.
- Phase 4 dispatches to a built-in handler or to a configured document or decision command.

## Built-in Names

- The reserved built-in names are `init`, `config`, `task`, `milestone`, `release`, `search`, `import`, `help`, and `version`.
- A configured document-type or decision-type slug that collides with a built-in name is rejected when the configuration loads.

## Common Options and Outcomes

- `--version` prints the version and exits.
- `--json` produces machine-readable JSON for every command that produces output, including error output.
- `--no-color` disables ANSI colour output; the environment variable `NO_COLOR` has the same effect.
- Exit statuses are distinct and fixed:

| Outcome | Exit code |
| --- | --- |
| Success | 0 |
| Runtime error | 1 |
| Usage error | 2 |
| Record not found | 3 |
| Validation or conflict | 4 |

## Project and Configuration Context

- The project directory is selected from `--dir`, then `PAPYRUS_DIR`, then the current working directory.
- The configuration file is selected from `--config`, then `PAPYRUS_CONFIG`, then `papyrus.config.yml` in the selected project directory.
- The selected configuration path must resolve inside the project directory; a `--config` or `PAPYRUS_CONFIG` path outside it is rejected.
- An explicitly named `--config` or `PAPYRUS_CONFIG` path that does not exist counts as not found.
- When no source yields an existing configuration file, every command except `papyrus init` fails; the global `--help` and `--version` options are also exempt and must work without a configuration. There is no fallback to built-in defaults and no walk-up or other discovery of a configuration file.
- That failure is a runtime error whose message names the searched path and states that `papyrus init` creates the configuration.
- `papyrus init` runs without an existing configuration file.
- `--help` and `--version` print their output and exit 0 without an existing configuration file.
- Built-in defaults apply only to individual keys absent from a configuration file that was found.

