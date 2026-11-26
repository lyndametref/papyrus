---
id: TECH-109
title: "Project Initialization"
created_date: '2026-11-26T09:26:53Z'
updated_date: '2026-11-26T09:26:53Z'
labels: []
references:
  - REQ-001
  - REQ-001.001
  - REQ-001.006
  - REQ-004.001
  - REQ-005.016
  - REQ-007.002
  - NFR-002.002
  - NFR-004.001
  - NFR-004.002
status: draft
---
# TECH-109 Project Initialization

## Description
Covers `papyrus init`: project creation, the generated configuration, the
directory layout, the interactive walkthrough, non-interactive behavior, and
`--force` replacement. The CLI foundation and configuration commands are
separate specifications.

## Command

- `papyrus init <NAME>` initializes the current directory;
  `papyrus --dir <PATH> init <NAME>` initializes `<PATH>`, creating it if absent
  (REQ-001.001).
- `<NAME>` is required and is stored as `project_name` in `papyrus.config.yml`
  (REQ-001.001).
- `init` runs without an existing configuration file (REQ-001.001).
- `init` may create the selected project directory outside an existing project;
  no other command may (NFR-004.002).
- All files written by `init` are UTF-8 text (NFR-004.001).
- An existing `papyrus.config.yml` blocks initialization unless `--force` is
  given; without `--force` the outcome is a conflict and nothing is written
  (REQ-001.001).
- `--force` replaces `papyrus.config.yml` in full with the generated
  configuration and keeps no value from the previous file. It overwrites no
  other existing file (REQ-001.001).
- `init` is allowed in a non-empty directory; no file other than
  `papyrus.config.yml` is modified (REQ-001.001).
- User-controlled input, including the project name and paths, is sanitized. A
  value that cannot be normalized fails with a validation outcome and writes
  nothing (NFR-002.002).

## Generated Configuration

- The generated configuration validates before it is written, including
  `id_padding` from 1 to 6 (REQ-007.002).
- Every selected Seed entry is written in full; an unselected entry is not
  written at all (REQ-005.016).
- There is no `enabled` key; presence in the configuration is the only switch
  (REQ-005.016).
- A selected document type is written with its model, field sequence, prefix,
  directory, status set, reference visibility, slots, and field defaults
  (REQ-005.016).
- The decision-type Seed offers `adr` and `bdr`; `init` writes the selected one
  or both (REQ-004.001).

## Generated Layout

- The configured `backlog_directory` is created with `tasks/`, `milestones/`,
  and `releases/` subfolders.
- The configured `doc_directory` and the active directory of every selected
  document type are created.
- The configured `decision_directory` is created only when at least one
  decision type is selected; the key is written even when none is.
- No `archive/` or `deprecated/` folder is pre-created; an archive folder
  appears when a record is first archived.
- No backlog archive folder is pre-created.
- Existing directories are reused; existing files other than
  `papyrus.config.yml` are never overwritten.

## Interactive Walkthrough

- The walkthrough runs before `papyrus.config.yml` is written, at
  `papyrus init <NAME>`, also with `--dir` or `PAPYRUS_DIR` (REQ-001.006).
- It presents the offered artifacts as a checklist rendered with the terminal
  formatting. The row under the cursor and the selection state of every row are
  distinguishable, and the display reflects the current state.
- Selection is cursor-driven. Typed token input, such as space-separated names
  on one line, is not accepted (REQ-001.006).
- End of input confirms the current state instead of waiting.
- The offered artifacts are task, milestone, and release, the decision-type
  Seed entries `adr` and `bdr`, and the document-type Seed entries.
- One or both decision types may be selected; each selected type becomes its
  own command family.
- Task, milestone, and release are always written, are shown as always
  included, and are never toggled; their prefixes are still prompted.
- For every selected decision or document type, the walkthrough prompts the
  identifier prefix and the storage folder.
- The shared roots `backlog_directory`, `decision_directory`, and
  `doc_directory` are prompted once each.
- Prompts show their default in brackets, and empty input accepts the default.
- Invalid input is rejected and re-prompted with a hint naming the default.
- The written configuration matches the state shown when the confirmation was
  accepted.
- Each selected document type is written in full; an unselected type is not
  written at all.

## Non-Interactive Initialization

- A run without an interactive terminal skips the walkthrough (REQ-001.006).
- It writes every Seed entry of both families in full, with the default
  prefixes and folders (REQ-005.016).
- It never waits on a prompt.
