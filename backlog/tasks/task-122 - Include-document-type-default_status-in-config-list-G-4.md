---
id: TASK-122
title: Include document type default_status in config list (G-4)
status: To Do
assignee:
  - rust-dev
created_date: '2026-11-14T20:31:00Z'
labels:
  - rust
  - config
milestone: m-18
dependencies: []
references:
  - docs/specifications/project-initialization-and-configuration.md
  - docs/architecture/adr/ADR-0024-list-valued-configuration-keys.md
documentation:
  - src/config.rs
  - tests/config_list_keys_cli.rs
priority: medium
ordinal: 41240
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why.** TASK-117 recorded G-4: config list flattens document_types.<slug> to prefix, directory, enabled, and statuses, but omits default_status even when the configuration sets it. default_status is a validated key that can make the whole configuration invalid (it must be a declared, non-archived status), yet the primary inspection surface hides it, so an operator cannot see the value the validator is rejecting and the documented repair path starts blind. The owner kept this as a Medium follow-up to the TASK-111 configuration surface.

**What.** Emit document_types.<slug>.default_status in config list whenever the entry declares one, in the same per-slug block and stable order as the other document-type keys, and omit the key when it is unset. config get, config set, config unset, validation, and repair behavior stay unchanged.

**In scope.**

- src/config.rs: flattened() emits the optional default_status key after statuses for each slug.
- Stable ordering across slugs and keys; config list text and JSON output both carry it.
- Unit test on flattened() and integration coverage in tests/config_list_keys_cli.rs for set, unset, invalid, and JSON cases.
- Documentation that enumerates the flattened document-type keys, if it lists them explicitly.

**Out of scope.**

- Changing key parsing, validation rules, the repair-mode contract, or the browser configuration view.
- Adding new keys; seeding defaults at init (already delivered by TASK-112).
- Reopening TASK-111 or TASK-117; this is a follow-up to their delivered behavior.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 config list prints document_types.<slug>.default_status for every document type that declares one, in a stable position after that slug statuses
- [ ] #2 The key is omitted when the entry has no default_status, and no empty or placeholder value is printed
- [ ] #3 Text and JSON output of config list agree; config get document_types.<slug>.default_status returns the same value it did before
- [ ] #4 config set and config unset still validate before write, and an invalid default_status is rejected with the same message as before
- [ ] #5 Unit tests cover flattened() with and without the key; integration tests cover the printed line, ordering, JSON output, unset, and an invalid value
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 cargo build, cargo test, and cargo clippy are clean.
- [ ] #2 No new dependency, configuration key, or flag is introduced.
- [ ] #3 G-4 is closed with evidence in the ticket notes and no change to BR-016 or REQ-01.002.
<!-- DOD:END -->
