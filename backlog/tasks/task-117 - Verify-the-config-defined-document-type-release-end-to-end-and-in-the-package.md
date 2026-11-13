---
id: TASK-117
title: Verify the config-defined document type release end to end and in the package
status: To Do
assignee:
  - rust-dev
created_date: '2026-11-13T21:27:00Z'
labels:
  - rust
  - browser
milestone: m-17
dependencies:
  - TASK-110
  - TASK-112
  - TASK-113
  - TASK-114
  - TASK-116
documentation:
  - docs/packaging.md
  - docs/requirements/business/REQ-07.003.md
priority: high
ordinal: 41110
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Each implementation task carries its own tests, but this change replaces the command tree, the configuration contract, the document storage lifecycle, search, and the browser at once. A single verification task is needed to run the cross-surface scenarios, exercise an upgrade from an old configuration, and prove the packaged binary exposes a configuration-added command. It also enforces the no-drift rules that keep deprecated/, static document commands, nfreq, and slug literals from returning.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Unit and integration suites pass for the registry, validator, record layer, generic handler, search, and browser, including a user-added type created, listed, viewed, edited, archived, deleted, and status-changed through the CLI.
- [ ] #2 CLI tests assert the generated --help inventory, per-slug help, unknown and disabled slug exit 2, --json on document commands, and the documented exit codes 0/1/2/3/4.
- [ ] #3 Browser tests cover list/detail/edit for a config-defined type, read-only status, the active-plus-custom filter and rejection, archived/unknown equivalence, disabled invisibility, and the lifecycle page.
- [ ] #4 An upgrade scenario starts from a pre-change configuration with deploy and a custom document command, runs the documented repair and migration steps, and ends with no nfreq, no static command assumptions, and a working dynamic command.
- [ ] #5 The Debian package is built, installed in a clean environment, and runs papyrus <new-slug> and papyrus --help offline with the enabled slugs listed; no man page is asserted for config-defined slugs and the NFR-03.003 risk is recorded.
- [ ] #6 A no-drift check fails on any deprecated/ reference in code, descriptors, routes, search, or docs outside historical ADRs and upgrade notes; on any static document-type command variant or nfreq alias; and on any slug-literal match outside the registry.
- [ ] #7 The full test suite and package build pass, and the results are recorded on the task for the release gate.
<!-- AC:END -->
