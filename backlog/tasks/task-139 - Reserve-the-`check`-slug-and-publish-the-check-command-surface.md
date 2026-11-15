---
id: TASK-139
title: Reserve the `check` slug and publish the check command surface
status: To Do
assignee:
  - '@rust-dev'
created_date: '2026-11-15T16:12:00Z'
labels:
  - rust
  - cli
  - config
milestone: m-21
dependencies:
  - TASK-137
  - TASK-138
references:
  - docs/requirements/business/REQ-07.002.md
  - >-
    docs/architecture/adr/ADR-0029-read-only-corpus-diagnostics-with-duplicate-id-repair.md
  - src/cli/tree.rs
  - src/cli/builtin.rs
  - src/config.rs
priority: high
ordinal: 41520
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why.** `check` is a built-in command name, so a configured document type must never take it (REQ-07.002), and the command needs a documented surface that matches the accepted ADR-0029. Today the reserved-name list predates `check`, and the built-in help has no entry for it.

**What.** Reserve the slug, publish the help text, and lock both with tests.

**In scope.**
- Add `check` to the built-in command names used by configuration validation, compared case-insensitively, so `document_types.check`, `Check`, and `CHECK` are rejected with a validation outcome naming the conflict (REQ-07.002).
- `papyrus check --help` documents the synopsis, `--fix`, `--force`, the finding kinds, the exit statuses (0 clean, 4 findings, 1 I/O failure, 2 usage), the tolerant configuration behaviour, and the read-only guarantee.
- The command tree built with no registry still exposes `check` and its help.
- Assert that `check` is absent from the document-family command list and that a configured `check` entry fails validation while the project stays repairable through `config` and `init --force`.
- Golden tests for `papyrus --help`, `papyrus check --help`, and the existing CLI help golden file are updated in one change.
- No browser route, no JSON API, no library surface beyond `run`.

**Out of scope.** README, business rules, and specification prose (TASK-140, TASK-148); man page (ADR-0021 remains Proposed); new flags.

**Assumptions.** The ADR-0029 decision 1 flag names and exit codes are final. Help wording follows the existing command help style.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A configured document type with the slug check, Check, or CHECK is rejected with a validation outcome naming the conflict.
- [ ] #2 The reserved-name comparison is case-insensitive and the project stays repairable through config and init --force after the rejection.
- [ ] #3 papyrus check --help documents the synopsis, --fix, --force, the finding kinds, and the exit statuses 0, 4, 1, and 2.
- [ ] #4 Help also states the tolerant configuration behaviour and the read-only default.
- [ ] #5 The command tree built with no registry still exposes check and its help.
- [ ] #6 The CLI help goldens for papyrus --help and papyrus check --help are updated and pass.
<!-- AC:END -->
