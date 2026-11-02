---
id: TASK-2
title: Implement project initialization and configuration commands
status: Refined
assignee: []
created_date: '2026-10-31T22:25:00Z'
updated_date: '2026-11-02T18:05:00Z'
labels: []
milestone: m-0
dependencies:
  - TASK-1
references:
  - docs/requirements/business/REQ-01.md
  - docs/requirements/business/REQ-01.001.md
  - docs/requirements/business/REQ-01.002.md
  - docs/requirements/business/REQ-01.003.md
  - docs/requirements/business/REQ-01.004.md
  - docs/requirements/business/REQ-01.005.md
  - docs/requirements/non-functional/NFR-02.001.md
  - docs/requirements/non-functional/NFR-02.002.md
  - docs/requirements/non-functional/NFR-04.001.md
  - docs/requirements/non-functional/NFR-04.002.md
  - docs/business-rules/BR-002-init-command-behavior.md
  - docs/business-rules/BR-003-config-command-behavior.md
  - docs/business-rules/BR-016-configuration-keys-and-precedence.md
  - docs/specifications/project-initialization-and-configuration.md
  - docs/business-rules/BR-001-papyrus-command-behavior.md
type: feature
ordinal: 2000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Implement Papyrus project initialization and configuration commands, including project/configuration context selection, common CLI controls and exit outcomes, and the configuration keys that govern stable record IDs.

Scope (in):
- `papyrus init <name> [--no-git] [--force]`: initialize the current or `--dir`-selected directory (creating it if absent), store `<name>` as `project_name`, create the backlog store, documentation structure, and `papyrus.config.yml`; `--no-git` skips Git initialization; `--force` permits a non-empty directory without overwriting existing files.
- `papyrus config list|get|set|unset`: inspect and persist effective configuration, with dotted keys and YAML-scalar values, defaults included.
- Project directory selection from `--dir`, then `PAPYRUS_DIR`, then the current working directory.
- Configuration file selection from `--config`, then `PAPYRUS_CONFIG`, then `<project_dir>/papyrus.config.yml`, then built-in defaults.
- Common CLI controls: `--json` (including errors), `--no-color`/`NO_COLOR`, `--version`, `-h`/`--help`, command help, exit statuses 0/1/2/3/4.
- Configuration keys and validation per BR-016: unique statuses, `default_status` membership, default statuses `ToDo, Initiated, Refined, Implemented, Validated, Done`, new-task default `ToDo`, ID prefixes `TASK`/`MS`/`REL` and padding `3`.
- File-access boundary: `init` is the only exception to the project-directory restriction; all other reads/writes stay within the canonicalized selected project directory.
- Persistence as UTF-8 text only; no binary data.
- Input sanitization by normalization/escaping before use.

Scope (out):
- Record-management subcommands (`task`, `milestone`, `release`, `decision`, document types, `search`, `browser`) — TASK-3 onward.
- Actual record ID assignment and archive/restore preservation logic — later tickets; this ticket delivers their configuration keys only.
- Git workflows beyond invoking repository initialization when `--no-git` is absent.
- UI mockups or web UI.

Assumptions:
- Assumption: Rust 1.99.1 via rustup and Cargo are available (ADR-0001).
- Assumption: `git` is available where `--no-git` is not used; otherwise init fails with a runtime error.
- Assumption: single-crate layout (`src/main.rs` + `src/lib.rs`) is retained.
- Assumption: `init` creates configured directories from effective defaults and never overwrites existing files.
- Assumption: configuration is stored as YAML; any YAML runtime dependency must be license-audited for GPL-3.0-or-later compatibility (NFR-06.001/002).
- Assumption: input sanitization uses normalization/escaping, rejecting only values that cannot be normalized into a valid form.
- Assumption: the project-directory boundary resolves symbolic links to canonical paths.

Open questions:
- OQ-1: The exact set of directories/files constituting the "documentation structure" created by `init` is not specified by the man page; implementation creates configured directories as needed and must not overwrite existing files.

Dependencies: TASK-1 (Done).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Running `papyrus init <name>` initializes the current directory and stores `<name>` as `project_name` in `papyrus.config.yml`, creating the backlog store, documentation structure, and config file.
- [ ] #2 Running `papyrus --dir <path> init <name>` initializes `<path>`, creating it if absent, with the same resulting structure.
- [ ] #3 `--no-git` suppresses Git repository initialization; without it Papyrus invokes Git initialization when available.
- [ ] #4 `--force` permits initialization in a non-empty directory and leaves pre-existing files unmodified; without `--force` a non-empty target directory fails with exit code 4.
- [ ] #5 Project directory selection follows `--dir`, then `PAPYRUS_DIR`, then the current working directory.
- [ ] #6 Configuration file selection follows `--config`, then `PAPYRUS_CONFIG`, then `<project_dir>/papyrus.config.yml`, then built-in defaults.
- [ ] #7 `papyrus config list` prints all effective keys including defaults; `get <key>` prints one effective value; `set <key> <value>` persists a YAML scalar; `unset <key>` removes the key and uses its default; dotted keys such as `document_types.req.enabled` work.
- [ ] #8 Configured task statuses are unique, `default_status` is one of them, and defaults are `ToDo, Initiated, Refined, Implemented, Validated, Done` with new tasks starting at `ToDo`.
- [ ] #9 Default ID configuration uses prefixes `TASK`, `MS`, `REL` and padding `3` (`TASK-001`, `MS-001`, `REL-001`); `id_padding` is a positive integer and wider IDs are not truncated.
- [ ] #10 `--version` prints `papyrus <version>` and exits 0; `-h`/`--help` and `papyrus <command> --help` print help and exit 0.
- [ ] #11 `--json` produces machine-readable JSON for output-producing commands, including errors; `--no-color` or a non-empty `NO_COLOR` disables ANSI color; exit statuses are 0 success, 1 runtime error, 2 usage error, 3 not found, 4 validation/conflict.
- [ ] #12 Usage errors print usage to stderr and exit with status 2.
- [ ] #13 Outside `init`, Papyrus reads and writes only within the canonicalized selected project directory; boundary violations fail without modifying files.
- [ ] #14 All configuration and other persisted data is written as UTF-8 text; no binary data is written.
- [ ] #15 User-controlled input is sanitized by normalization/escaping before use; values that cannot be normalized into a valid form fail with exit code 4.
- [ ] #16 Security is explicitly considered in implementation and review records (NFR-02.001).
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 All acceptance criteria are verified by automated tests or documented manual checks.
- [ ] #2 cargo build --release and cargo test pass with the pinned Rust 1.99.1 toolchain.
- [ ] #3 cargo clippy --all-targets -- -D warnings and cargo fmt --check pass.
- [ ] #4 init creates the documented structure without overwriting existing files, including --no-git and --force cases.
- [ ] #5 Config list/get/set/unset behavior and precedence are verified for CLI, environment, file, and defaults.
- [ ] #6 UTF-8 text-only persistence and project-directory boundary (including symlinks) are verified.
- [ ] #7 Input sanitization (normalization/escaping) is covered by tests for representative inputs.
- [ ] #8 Any new runtime dependency (e.g. YAML parser) is added to the license audit and is GPL-3.0-or-later compatible.
- [ ] #9 Build, packaging, install, and smoke-test steps remain documented and pass.
- [ ] #10 Acceptance criteria reviewed and signed off.
<!-- DOD:END -->
