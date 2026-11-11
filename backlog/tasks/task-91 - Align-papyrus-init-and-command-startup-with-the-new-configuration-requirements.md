---
id: TASK-91
title: Align papyrus init and command startup with the new configuration requirements
status: To Do
assignee: []
created_date: '2026-11-11T20:42:00Z'
updated_date: '2026-11-11T20:57:00Z'
labels:
  - configuration
  - init
  - release-1.0
milestone: m-16
dependencies:
  - TASK-85
references:
  - docs/requirements/business/REQ-01.001.md
  - docs/requirements/business/REQ-01.003.md
  - docs/requirements/business/REQ-01.006.md
  - docs/specifications/project-initialization-and-configuration.md
  - docs/specifications/interactive-init-walkthrough-for-artifact-activation.md
priority: high
type: feature
ordinal: 39080
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
REQ-01.001 now allows initialization in a non-empty directory, makes --force a full replacement of an existing papyrus.config.yml, and REQ-01.003 makes a missing configuration a runtime error for every non-init command. REQ-01.006 replaces per-artifact prompts with a single space-toggle list that runs only on a TTY. Current init refuses any non-empty directory without --force and the walkthrough prompts artifact by artifact, so the CLI does not yet implement the settled requirement text.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 papyrus init succeeds in a non-empty directory and modifies no pre-existing file other than papyrus.config.yml.
- [ ] #2 An existing papyrus.config.yml blocks init without --force with a conflict outcome and nothing written; with --force the file is replaced in full by the generated configuration and no previous value is kept.
- [ ] #3 --no-git still suppresses Git initialization.
- [ ] #4 Every command except papyrus init fails with a runtime error and exit code 1 when no configuration file is found, naming the searched path and the papyrus init remedy, with no built-in-defaults fallback and no directory discovery.
- [ ] #5 Interactive init presents one space-toggle artifact list where task, milestone, and release are shown always active, then prompts prefix and folder for activated artifacts and each shared root once.
- [ ] #6 Prompts show the default in brackets, accept empty input as the default, and re-prompt with a hint naming the default equivalent for an invalid prefix or folder.
- [ ] #7 The walkthrough runs only when standard input is a terminal; otherwise defaults are written without prompting and no new flag or environment variable is introduced.
- [ ] #8 Inactive types are written with enabled: false and the generated configuration validates, including the id_padding range.
- [ ] #9 Tests cover non-empty-directory init, force replacement, conflict without force, missing-config failure for a non-init command, TTY and non-TTY walkthrough paths, and validation re-prompting.
<!-- AC:END -->
