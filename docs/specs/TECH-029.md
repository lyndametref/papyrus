---
id: TECH-029
title: "Project Initialization"
created_date: '2026-11-27T20:35:43Z'
updated_date: '2026-11-27T20:35:44Z'
labels: []
references:
  - REQ-001
  - REQ-001.001
  - REQ-001.006
  - REQ-004.001
  - REQ-005.016
  - REQ-007.002
  - NFR-004.001
  - NFR-004.002
status: verified
---
# TECH-029 Project Initialization

## Description
Covers the implementation of `papyrus init`: input assembly, Seed expansion, prompt overrides, whole-configuration validation, directory creation, and the configuration write. Terminal key handling is specified separately; Seed content and configuration schema are separate.

## Initialization sequence

1. Resolve the project directory and configuration path with the CLI context rules.
2. Require `<NAME>`; it becomes `project_name` in the generated configuration.
3. If `papyrus.config.yml` exists and `--force` is absent, stop with a conflict before creating anything.
4. Build the selection: an interactive run uses the terminal walkthrough, a non-interactive run includes every Seed entry.
5. Expand the selected entries from the compiled Seed and apply the prompted or default prefix and directory.
6. Assemble the configuration document and validate it as a whole; a validation failure writes nothing.
7. After validation, create the configured directories and write `papyrus.config.yml`; `--force` replaces that file in full and no other file is touched.

## Seed expansion

- The compiled Seed configuration is the only source of entries; no built-in table is consulted after expansion.
- A selected document-type entry is copied with its model, field sequence, custom status set, slots, and field defaults.
- A selected decision-type entry is copied with the shared fixed status vocabulary.
- The interactive prompts replace the Seed prefix and directory in the written entry; the non-interactive path keeps the Seed values.
- Backlog families are always included; an unselected document or decision entry is omitted entirely.

## Prompt overrides

- Each selected type prompts for prefix and directory; each shared root (`backlog_directory`, `decision_directory`, `doc_directory`) prompts once.
- Defaults come from the Seed; empty input accepts the default, and invalid input is rejected and re-prompted with the default in the hint.
- Prompts run line-based under canonical terminal mode after the list phase; typeahead after confirming the list feeds the first prompt.

## Generated layout

- The backlog root is created with `tasks/`, `milestones/`, and `releases/`.
- The document root and the active directory of every selected document type are created.
- The decision root is created only when at least one decision type is selected; the key is written even when none is.
- Directories are created with create-or-reuse semantics, so an existing directory is kept.
- No `archive/` or `deprecated/` folder is created; an archive folder appears on the first archive.

## Write behavior

- The only file `init` creates or replaces is `papyrus.config.yml`; every other existing file is untouched.
- The configuration is UTF-8 YAML and is validated before the write.
- `--force` performs a full replacement and merges no value from the previous file.
- A failed validation or a prompt conflict leaves the existing configuration unchanged.
