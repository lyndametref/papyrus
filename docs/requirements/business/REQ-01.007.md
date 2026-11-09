# REQ-01.007 ID Padding Constraint for File IDs

## User Story
As a project maintainer, I want to constrain the `id_padding` configuration to 1-6 characters so that file IDs (task-001, ms-001, rel-001, etc.) use consistent padding with a clear upper limit, improving readability and preventing invalid configurations.

## Group
REQ-01 Project Setup and CLI Access

## Criticality
Should have

## Description
The `id_padding` configuration sets the numbering width after the prefix (for example `task-1` to `task-0001`). It currently accepts any positive integer with default 3. This requirement adds a constraint to limit `id_padding` to values between 1 and 6 inclusive. Values outside this range should error out during configuration validation or init. This affects task, milestone, and release ID formatting. Display padding is a minimum, not a truncation: when a storage number's digit count exceeds `id_padding`, the display ID uses the wider number and no error is raised (ADR-0008). There is no re-padding on disk and no migration of existing files.

## Acceptance Criteria
- `id_padding` must be configured to a value between 1 and 6 inclusive.
- Values outside the 1-6 range error out with clear messages during configuration validation or init.
- The default value remains 3 and is constrained to 1-6.
- When a storage number's digit count exceeds `id_padding`, the display ID uses the wider number; padding is a minimum, never truncated or errored.
- The constraint applies to task, milestone, and release IDs as specified in REQ-01.005.

## Open Questions
- Error behavior for out-of-range values: exact validation point (config set vs init vs load), exit code, and message wording.
- Whether to constrain the existing `id_padding` key or introduce a separate key.
- Interaction with configuration files that already specify an out-of-range value: reject at load vs clamp (recommend reject with clear error).

## Traceability
- Source: customer-feedback handoff — Internal team discussion, Item 9 (medium), raw quote:
  - "add a configuration to set the padding of the file ids for example choose task-1 task-0001 (between 1 and 6 characters for the numbering after the prefix)"
- User decision (ADR-0008, 2026-11-08): when a storage number's digit count exceeds `id_padding`, the display ID uses the wider number; padding is a minimum, never truncated or errored.
- ADR-0008 — Adopt the Backlog.md On-Disk Layout for Task and Milestone Records (display width resolution)
- REQ-01.002 — Read and Edit Project Configuration
- REQ-01.005 — Assign Stable Record IDs
- BR-016 — Papyrus Configuration Keys and Precedence
