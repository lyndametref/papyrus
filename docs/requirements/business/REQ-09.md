# REQ-09 ID Padding Constraint for File IDs

## User Story
As a project maintainer, I want to constrain the `id_padding` configuration to 1-6 characters so that file IDs (task-001, ms-001, rel-001, etc.) use consistent padding with a clear upper limit, improving readability and preventing invalid configurations.

## Group
REQ-01 Project Setup and CLI Access

## Criticality
Medium

## Description
The `id_padding` configuration currently accepts any positive integer with default 3. This requirement adds a constraint to limit `id_padding` to values between 1 and 6 inclusive. Values outside this range should error out during configuration validation. This affects task, milestone, and release ID formatting, and must interact properly with existing files whose IDs may exceed the new width.

## Acceptance Criteria
- `id_padding` must be configured to a value between 1 and 6 inclusive.
- Values outside 1-6 range error out with clear messages during configuration validation or init.
- Existing records with IDs exceeding the new width must be handled appropriately (e.g., migration path or validation).
- The constraint applies to task, milestone, and release IDs as specified in REQ-01.005.
- Default value remains 3, but is now constrained to 1-6.

## Traceability
- Source: customer-feedback handoff — Internal team discussion, Item 9 (medium), raw quote:
  - "add a configuration to set the padding of the file ids for example choose task-1 task-0001 (between 1 and 6 characters for the numbering after the prefix)"
  - Open questions: constrain `id_padding` to 1–6 or separate key; error behavior for out-of-range values; interaction with existing files whose IDs exceed new width.
  - Affects REQ-01.005 (Assign Stable Record IDs), REQ-01.002 (Read and Edit Project Configuration), and BR-016 (Configuration Keys and Precedence).