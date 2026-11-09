---
id: TASK-74
title: Constrain id_padding to 1-6 in configuration validation
status: To Do
assignee: []
created_date: '2026-11-09T18:51:00Z'
labels:
  - config
  - ids
milestone: m-12
dependencies: []
references:
  - docs/requirements/business/REQ-01.007.md
  - docs/requirements/business/REQ-01.005.md
  - docs/business-rules/BR-016-configuration-keys-and-precedence.md
priority: high
type: feature
ordinal: 30005
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
id_padding currently accepts any positive integer. REQ-01.007 requires a 1-6 range so file IDs use consistent padding with a clear upper limit. This task adds validation and updates ID formatting so padding is a minimum (wider numbers are not truncated or errored), with a default of 3.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Configuration validation rejects id_padding values outside 1-6 with a clear error message.
- [ ] #2 The default id_padding remains 3 and is within the allowed range.
- [ ] #3 When a storage number has more digits than id_padding, the display ID uses the wider number; padding is a minimum, never truncated or errored.
- [ ] #4 The constraint applies to task, milestone, and release IDs per REQ-01.005.
- [ ] #5 An existing configuration with an out-of-range id_padding is rejected at load with a clear error rather than clamped.
<!-- AC:END -->
