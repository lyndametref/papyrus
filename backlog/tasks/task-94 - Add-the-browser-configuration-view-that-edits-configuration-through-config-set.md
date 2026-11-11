---
id: TASK-94
title: Add the browser configuration view that edits configuration through config set
status: To Do
assignee: []
created_date: '2026-11-11T20:43:00Z'
labels:
  - browser
  - configuration
  - release-1.0
milestone: m-14
dependencies:
  - TASK-87
  - TASK-90
references:
  - docs/requirements/business/REQ-06.008.md
  - docs/requirements/business/REQ-01.002.md
  - docs/requirements/business/REQ-01.003.md
  - docs/requirements/non-functional/NFR-04.002.md
priority: high
type: feature
ordinal: 39110
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
REQ-06.008 asks for configuration editing in the browser, while REQ-01.002 requires a folder change to move folders atomically and NFR-04.002 restricts access to the project directory. A browser writer that touched papyrus.config.yml directly would bypass relocation and containment, so the view must go through the same command path as the CLI. This task delivers that view on top of the relocation routine.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The web interface serves a configuration view for the current project.
- [ ] #2 The view shows effective values including built-in defaults and marks the keys explicitly set in papyrus.config.yml.
- [ ] #3 Editable keys are exactly those config set accepts, with the same validation and precedence; the view cannot store a value the CLI rejects.
- [ ] #4 Saving delegates to the config set path, so an artifact folder change performs the full relocation of REQ-01.002 and a conflict is reported without persisting a value.
- [ ] #5 No path outside the project can be defined, and no environment-variable value is displayed as editable configuration.
- [ ] #6 No configuration key is treated as secret.
- [ ] #7 Tests cover effective-versus-explicit display, rejection parity with the CLI, a folder move performed from the view, and containment refusal.
<!-- AC:END -->
