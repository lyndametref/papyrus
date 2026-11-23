---
id: TASK-086
title: Align papyrus init and command startup with the new configuration requirements
status: Done
assignee: []
created_date: '2026-11-11T20:42:00Z'
updated_date: '2026-11-13T05:57:00Z'
labels:
  - configuration
  - init
  - release-1.0
milestone: MS-017
dependencies:
  - TASK-080
references:
  - docs/requirements/business/REQ-001.001.md
  - docs/requirements/business/REQ-001.003.md
  - docs/requirements/business/REQ-001.006.md
  - docs/specifications/project-initialization-and-configuration.md
  - docs/specifications/interactive-init-walkthrough-for-artifact-activation.md
  - docs/requirements/business/REQ-001.002.md
  - docs/requirements/business/REQ-001.007.md
  - docs/architecture/adr/ADR-0010-papyrus-native-on-disk-record-layout.md
  - docs/architecture/adr/ADR-0011-one-way-backlog-md-import.md
  - >-
    docs/architecture/adr/ADR-0013-shared-record-envelope-without-a-storage-layer.md
priority: high
type: feature
ordinal: 39080
---


## Description
Align `papyrus init` and non-init command startup with REQ-001.001, REQ-001.002, REQ-001.003, REQ-001.006, and REQ-001.007.

Scope:
- Init succeeds in a non-empty directory and modifies no pre-existing file other than `papyrus.config.yml`.
- An existing `papyrus.config.yml` blocks init without `--force` with a conflict outcome and writes nothing; with `--force` it is replaced in full by the generated configuration, with no merge and no previous value retained. `--force` never overwrites any other pre-existing file.
- `--no-git` continues to suppress Git initialization.
- Every command except `papyrus init` fails with a runtime error and exit code 1 when no configuration file is found. The message names the searched path and the `papyrus init` remedy; there is no built-in-defaults fallback and no directory discovery or walk-up. An explicitly named missing `--config` or `PAPYRUS_CONFIG` path fails the same way.
- Interactive init presents one space-toggle artifact list where task, milestone, and release are always active; only activated artifacts are prompted for prefix and folder; each shared root (`backlog_directory`, `decision_directory`, `doc_directory`) is prompted once.
- Prompts show defaults in brackets, accept empty input as the default, and re-prompt invalid prefixes/folders with a hint naming the default equivalent.
- The walkthrough runs only when stdin is a TTY; otherwise defaults are written without prompting and no new flag or environment variable is introduced.
- Inactive decision/document types are written with `enabled: false`; the generated configuration validates, including `id_padding` 1-6.
- Directory creation follows activation: core directories always; decision/document directories only for activated artifacts.

Out of scope: record-management subcommands, schema changes, UI mockups, migration/re-padding of existing files, and new CLI flags or environment variables.

Assumptions:
- Assumption: TTY detection uses stdin terminal status (`std::io::IsTerminal`); no new flag or environment variable is introduced.
- Assumption: Non-interactive runs activate every artifact with built-in prefixes/folders and `id_padding` 3.
- Assumption: `--force` is a replacement of `papyrus.config.yml`, not a merge, and does not permit overwriting any other file.
- Assumption: Missing-configuration failure is implemented in the shared command startup path used by every non-init command, before any context-dependent read.
- Assumption: The interactive presentation is a single space-toggle list; exact prompt wording beyond the specified defaults/hints is implementation-defined.
- Assumption: README/manpage wording may need updating to describe the walkthrough and startup failure; no UI mockups are required.

## Acceptance Criteria
- [x] papyrus init succeeds in a non-empty directory and modifies no pre-existing file other than papyrus.config.yml.
- [x] An existing papyrus.config.yml blocks init without --force with a conflict outcome and nothing written; with --force the file is replaced in full by the generated configuration and no previous value is kept.
- [x] --force never overwrites any pre-existing file other than papyrus.config.yml.
- [x] --no-git still suppresses Git initialization.
- [x] Every command except papyrus init fails with a runtime error and exit code 1 when no configuration file is found, naming the searched path and the papyrus init remedy, with no built-in-defaults fallback and no directory discovery.
- [x] An explicitly named missing --config or PAPYRUS_CONFIG path fails the same way as a missing default configuration file.
- [x] Interactive init presents one space-toggle artifact list where task, milestone, and release are shown always active, then prompts prefix and folder for activated artifacts and each shared root once.
- [x] Prompts show the default in brackets, accept empty input as the default, and re-prompt with a hint naming the default equivalent for an invalid prefix or folder.
- [x] The walkthrough runs only when standard input is a terminal; otherwise defaults are written without prompting and no new flag or environment variable is introduced.
- [x] Inactive types are written with enabled: false and the generated configuration validates, including the id_padding range.
- [x] Directory creation follows artifact activation: core directories always; decision and document directories only for activated artifacts.
- [x] Tests cover non-empty-directory init, force replacement, conflict without force, missing-config failure for a non-init command, TTY and non-TTY walkthrough paths, and validation re-prompting.

## Definition of Done
- [x] #1 Implementation plan recorded in the ticket and followed.
- [x] #2 Unit and integration tests updated for the changed init and startup behavior.
- [x] #3 cargo test passes.
- [x] #4 README and manpage wording updated if user-visible behavior changed.
- [x] #5 No direct task/specification file edits were made outside the backlog CLI and documented spec locations.


## Implementation Notes
1. Review current init/startup implementation and tests; map gaps to REQ-001.001/002/003/006/007.
2. Update src/init.rs: remove non-empty-directory rejection; block only when papyrus.config.yml exists without --force; replace the config file in full under --force; leave all other files untouched; preserve --no-git.
3. Update command startup in src/lib.rs::run (or a shared helper): for every non-init command, check the resolved configuration path exists before building context; return runtime error exit 1 naming the searched path and papyrus init remedy; remove defaults fallback and discovery.
4. Refactor the interactive walkthrough in src/init.rs to a single space-toggle artifact list: task, milestone, and release always active; decision/document toggles; prompt prefix/folder only for activated artifacts; prompt each shared root once; bracket defaults; validation hints; inactive types enabled: false.
5. Align directory creation with activation rules and validate configuration before writing.
6. Update unit and integration tests for non-empty init, force replacement, conflict, missing-config startup, TTY/non-TTY paths, and validation re-prompting.
7. Run cargo test and relevant smoke tests; update README/manpage wording if needed.
8. Record implementation notes and check acceptance criteria.
