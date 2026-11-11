---
id: TASK-75
title: Add interactive init walkthrough for artifact activation
status: Done
assignee: []
created_date: '2026-11-09T18:51:00Z'
updated_date: '2026-11-09T19:53:00Z'
labels:
  - init
  - config
milestone: m-12
dependencies:
  - TASK-74
references:
  - docs/requirements/business/REQ-01.006.md
  - docs/requirements/business/REQ-01.001.md
  - docs/requirements/business/REQ-01.002.md
  - docs/business-rules/BR-016-configuration-keys-and-precedence.md
  - docs/requirements/business/REQ-01.007.md
  - docs/business-rules/BR-002-init-command-behavior.md
documentation:
  - docs/specifications/interactive-init-walkthrough-for-artifact-activation.md
priority: high
type: feature
ordinal: 30006
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Deliver the interactive walkthrough at `papyrus init <project_name>` per REQ-01.006, REQ-01.001, REQ-01.002, REQ-01.007, and BR-016. Before writing `papyrus.config.yml`, present all available artifacts (task, milestone, release, decision types, document types), let the user choose which to activate, and prompt for the ID prefix and storage folder of each activated artifact. Non-interactive/CI environments (no TTY) use default selections and defaults without prompting. The generated configuration must validate, including the `id_padding` range.

In scope:
- TTY detection; interactive prompts for activation, prefix, and storage folder.
- Core artifacts (task, milestone, release) always activated; prompt for their prefixes and shared backlog directory.
- Decision types (adr, bdr) and document types (req, nfr, glossary, rule, ui-spec, ui-mockup, audit, deploy, tech-spec): activation toggle; inactive types written with `enabled: false`.
- Shared storage roots (backlog, decision, docs) and per-document-type directories prompted with validation and re-prompt hints.
- Non-interactive defaults: all artifacts activated, built-in prefixes/folders, id_padding 3.
- `--force` and `--no-git` unchanged.
- Directory creation limited to activated artifacts.
- README/manpage init section and BR-002 wording updated.

Out of scope:
- New CLI flags or environment variables for selection.
- Configuration schema changes or removal of `enabled` flags.
- UI mockups (CLI-only).
- Migration or re-padding of existing files.

See docs/specifications/interactive-init-walkthrough-for-artifact-activation.md for full behavior.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Running `papyrus init` in an interactive terminal presents all available artifacts and prompts for activation selection.
- [ ] #2 For each activated artifact, the walkthrough prompts for its ID prefix and storage folder before writing `papyrus.config.yml`.
- [ ] #3 Only activated artifacts are enabled in the generated configuration; inactive decision/document types are written with `enabled: false`.
- [ ] #4 Invalid prefix or folder choices are rejected and re-prompted with hints naming equivalent default values.
- [ ] #5 Non-interactive or CI environments (no TTY) use default selections and defaults without prompting or reading stdin.
- [ ] #6 The `--force` and `--no-git` options continue to behave as documented (REQ-01.001).
- [ ] #7 The generated configuration validates successfully, including the `id_padding` range (REQ-01.007).
- [ ] #8 Directories are created only for activated artifacts (core artifacts always).
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Interactive vs. non-interactive mode is implemented and covered by tests.
- [ ] #2 Generated `papyrus.config.yml` validates via `Config::validate()`.
- [ ] #3 Inactive decision/document types are written with `enabled: false`.
- [ ] #4 Invalid prefix/folder inputs are re-prompted with hints; no abort on a single invalid entry.
- [ ] #5 `--force` and `--no-git` behavior unchanged and covered by tests.
- [ ] #6 Directory creation is limited to activated artifacts.
- [ ] #7 README/manpage init section and BR-002 updated to describe the walkthrough.
- [ ] #8 No regressions in existing init/config tests.
<!-- DOD:END -->
