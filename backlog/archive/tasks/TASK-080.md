---
id: TASK-080
title: >-
  Revise the initialization and configuration specifications for
  force-overwrite, folder relocation, missing-config failure, and the TTY
  walkthrough
status: Done
assignee: []
created_date: '2026-11-11T20:41:00Z'
updated_date: '2026-11-12T07:02:00Z'
labels:
  - specification
  - configuration
  - release-1.0
milestone: MS-017
dependencies: []
references:
  - docs/requirements/business/REQ-01.001.md
  - docs/requirements/business/REQ-01.002.md
  - docs/requirements/business/REQ-01.003.md
  - docs/requirements/business/REQ-01.006.md
  - docs/architecture/adr/ADR-0010-papyrus-native-on-disk-record-layout.md
  - docs/specifications/project-initialization-and-configuration.md
  - docs/specifications/interactive-init-walkthrough-for-artifact-activation.md
  - >-
    docs/architecture/adr/ADR-0013-shared-record-envelope-without-a-storage-layer.md
  - docs/requirements/business/REQ-01.007.md
priority: high
type: docs
ordinal: 39020
---


## Description
REQ-01.001, REQ-01.002, REQ-01.003, and REQ-01.006 changed on 2026-11-11, but the initialization and configuration specifications still document the earlier behaviour: a non-empty directory blocked without --force, a missing configuration falling back to defaults, no folder move on config set, and per-artifact activation prompts. Implementers would otherwise build the old contract. This task makes the configuration and walkthrough specifications match the requirements and ADR-0010 decision 8.

## Acceptance Criteria
- [ ] The initialization and configuration specification states that init succeeds in a non-empty directory and modifies no pre-existing file other than papyrus.config.yml, that an existing papyrus.config.yml blocks init without --force, and that --force replaces that file in full with no previous value kept.
- [ ] It states that every command except papyrus init fails as a runtime error with exit code 1 when no configuration file is found, that the message names the searched path and the papyrus init remedy, and that no built-in-defaults fallback or directory discovery happens.
- [ ] It states that a configured folder change via config set on backlog_directory, decision_directory, doc_directory, or a document type directory moves every file and subfolder of the source folder synchronously as an atomic move, with pre-flight conflict detection, rollback, no partial move, and the emptied source folder removed.
- [ ] The interactive init walkthrough specification describes a single space-toggle artifact list where task, milestone, and release are shown always active, per-artifact prefix and folder prompts for activated artifacts only, each shared root prompted once, defaults shown in brackets, validation hints naming the default equivalent, and inactive types written with enabled: false.
- [ ] It states that the walkthrough runs only when standard input is a terminal and that non-TTY runs write defaults without prompting and without a new flag or environment variable.
- [ ] No specification states that a non-empty directory requires --force or that a missing configuration file falls back to built-in defaults.
- [ ] The specifications name the browser configuration view as a consumer of the same config set path, not as a folder mover.
- [ ] Traceability links REQ-01.001, REQ-01.002, REQ-01.003, REQ-01.006, REQ-01.007, BR-016, ADR-0010 decision 8, and ADR-0013 stage 5.

## Definition of Done
- [ ] #1 Both target specifications are updated and internally consistent with REQ-01.001, REQ-01.002, REQ-01.003, REQ-01.006, REQ-01.007, BR-016, ADR-0010 decision 8, and ADR-0013 stage 5
- [ ] #2 No specification states that a non-empty directory requires --force or that a missing configuration file falls back to built-in defaults
- [ ] #3 Traceability tables in both specifications list REQ-01.001, REQ-01.002, REQ-01.003, REQ-01.006, REQ-01.007, BR-016, ADR-0010 decision 8, and ADR-0013 stage 5
- [ ] #4 Browser configuration view is named as a consumer of the same config set path, not as a folder mover
