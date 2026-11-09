---
id: TASK-74
title: Constrain id_padding to 1-6 in configuration validation
status: Done
assignee: []
created_date: '2026-11-09T18:51:00Z'
updated_date: '2026-11-09T19:39:00Z'
labels:
  - config
  - ids
milestone: m-12
dependencies: []
references:
  - docs/requirements/business/REQ-01.007.md
  - docs/requirements/business/REQ-01.005.md
  - docs/business-rules/BR-016-configuration-keys-and-precedence.md
documentation:
  - >-
    docs/specifications/constrain-id-padding-to-1-6-in-configuration-validation.md
priority: high
type: feature
ordinal: 30005
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Constrain the `id_padding` configuration key to 1-6 (inclusive) per REQ-01.007, REQ-01.005, and BR-016. Currently `Config::validate()` only rejects 0. This task tightens validation and confirms ID formatting treats padding as a minimum width.

In scope:
- `Config::validate()` (src/config.rs) rejects `id_padding` < 1 or > 6 with a key-prefixed message (`id_padding: must be between 1 and 6`).
- Default `id_padding` remains 3 and passes validation.
- Boundaries: 1 and 6 accepted; 0 and 7 rejected.
- Load-time rejection of an existing config file with an out-of-range `id_padding` (exit code 4); never clamped.
- `config set`/`config unset` validate the merged effective configuration and reject out-of-range values.
- ID formatting for task, milestone, release, and all other record types keeps padding as a minimum: wider numbers are not truncated or errored.
- Manpage/README `id_padding` description updated to state the 1-6 range.

Out of scope:
- Introducing a new configuration key; the existing `id_padding` key is constrained.
- Migration or re-padding of existing record files.
- Interactive init walkthrough validation (TASK-75).
- UI mockups (none).

Assumptions:
- Assumption: The constraint applies to the existing `id_padding` key, not a new key.
- Assumption: Validation lives in `Config::validate()` and runs on load, set, and unset via the merged effective configuration; `init` uses the default 3.
- Assumption: Out-of-range values are rejected (exit code 4), never clamped.
- Assumption: Error message follows the established pattern `id_padding: must be between 1 and 6`.
- Assumption: `IdSpec` in src/record/identity remains a pure formatter; config validation is the boundary that enforces 1-6. Tests may construct specs with paddings outside 1-6.
- Assumption: No migration or re-padding of existing files.
- Assumption: No UI mockups are required.

Dependencies: none (builds on the config foundation delivered by TASK-2).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Configuration validation rejects id_padding values outside 1-6 with a clear, key-prefixed error message (exit code 4).
- [ ] #2 The default id_padding remains 3 and passes validation.
- [ ] #3 id_padding values 1 and 6 are accepted; 0 and 7 are rejected.
- [ ] #4 When a storage number has more digits than id_padding, the display ID uses the wider number; padding is a minimum, never truncated or errored.
- [ ] #5 The constraint applies to task, milestone, and release IDs, and to all other record types using id_padding.
- [ ] #6 An existing configuration file with an out-of-range id_padding is rejected at load with a clear error rather than clamped.
- [ ] #7 config set id_padding and config unset validate the merged effective configuration and reject out-of-range values.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Config::validate() rejects id_padding outside 1-6 with message 'id_padding: must be between 1 and 6'.
- [ ] #2 Default id_padding remains 3.
- [ ] #3 Tests cover acceptance (in-range/out-of-range, load rejection, set rejection, padding-as-minimum) with no regressions.
- [ ] #4 Manpage/README id_padding description states the 1-6 range.
- [ ] #5 No migration or re-padding of existing files.
<!-- DOD:END -->
