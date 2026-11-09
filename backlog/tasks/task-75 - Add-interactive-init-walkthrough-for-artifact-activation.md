---
id: TASK-75
title: Add interactive init walkthrough for artifact activation
status: To Do
assignee: []
created_date: '2026-11-09T18:51:00Z'
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
priority: high
type: feature
ordinal: 30006
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
papyrus init currently writes defaults without prompts. REQ-01.006 requires an interactive walkthrough that presents all available artifacts, lets the user choose which to activate, and prompts for the ID prefix and storage folder of each before generating papyrus.config.yml, while retaining sensible defaults for non-interactive or CI environments.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Running papyrus init presents all available artifacts and prompts for activation selection.
- [ ] #2 For each activated artifact, the walkthrough prompts for its ID prefix and storage folder before writing papyrus.config.yml.
- [ ] #3 Only activated artifacts are enabled in the generated configuration; inactive artifacts are disabled or omitted per BR-016.
- [ ] #4 Invalid prefix or folder choices are rejected with hints naming equivalent values.
- [ ] #5 Non-interactive or CI environments with no TTY use default selections and defaults without prompting.
- [ ] #6 The --force and --no-git options continue to behave as documented (REQ-01.001).
- [ ] #7 The generated configuration validates successfully, including the id_padding range (REQ-01.007).
<!-- AC:END -->
