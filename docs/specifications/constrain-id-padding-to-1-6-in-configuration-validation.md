# Feature Specification — Constrain id_padding to 1-6 in Configuration Validation

**Ticket:** TASK-74
**Milestone:** m-12 (Final Cross-Type Integration)
**Status:** Refined (execution-ready)
**UI mockups:** none

## Purpose

Constrain the `id_padding` configuration key to the inclusive range 1-6 per REQ-01.007, REQ-01.005, and BR-016, so file IDs (task, milestone, release, and all other record types) use consistent padding with a clear upper limit. Validation rejects out-of-range values instead of clamping them, and ID formatting keeps padding as a minimum width (wider numbers are never truncated or errored).

## Scope

### In scope

- `Config::validate()` (in `src/config.rs`) rejects `id_padding` values below 1 or above 6 with a key-prefixed error message.
- Default `id_padding` remains 3 and passes validation.
- Boundaries: 1 and 6 accepted; 0 and 7 rejected.
- Load-time rejection of an existing configuration file whose `id_padding` is out of range (exit code 4), never clamped.
- `config set id_padding` and `config unset` validate the merged effective configuration and reject out-of-range values.
- ID formatting for task, milestone, release, and every other record type that uses `id_padding` treats padding as a minimum width: when a storage number's digit count exceeds `id_padding`, the display ID uses the wider number; no truncation and no error.
- Manpage (`README.md`, `papyrus-config(5)`) and BR-016 wording updated to state the 1-6 range.

### Out of scope

- Introducing a new configuration key; the existing `id_padding` key is constrained.
- Migration or re-padding of existing record files (no on-disk change).
- Interactive init walkthrough validation (TASK-75), which independently validates generated configurations.
- UI mockups (none for this ticket).
- Changes to the pure ID formatting helpers in `src/record/identity` beyond their existing minimum-width behavior.

## Behavior

### 1. Validation rule

1. `id_padding` must be an integer in the inclusive range 1-6.
2. Values outside the range are rejected with the key-prefixed message `id_padding: must be between 1 and 6`.
3. The default value 3 is within range and remains the built-in default.
4. Validation runs through `Config::validate()`, which is invoked on effective-config load, on `config set`, and on `config unset` (merged effective configuration).
5. An out-of-range value in a configuration file is rejected at load (exit code 4); it is never clamped or silently corrected.

### 2. ID formatting (minimum width)

1. Padding is a minimum width: when a storage number's digit count exceeds `id_padding`, the display ID uses the wider number.
2. No truncation and no error is raised for wide numbers (ADR-0008).
3. The existing Rust formatting (`{:0width$}` in record creation and `pad_number` in `src/record/identity`) already implements minimum width; this ticket confirms no code path truncates or errors on wide numbers.
4. No re-padding or migration of existing files occurs.

### 3. Scope of affected IDs

The constraint applies to task, milestone, and release IDs (REQ-01.005) and, by extension, to every other record type whose ID uses `id_padding` (decisions, requirements, glossary terms, business rules, UI specifications, UI mockups, audit records, technical specifications).

## Inputs and outputs

- Input: a configuration value for `id_padding` (YAML scalar integer) via `papyrus.config.yml`, `--config`, `PAPYRUS_CONFIG`, or `config set`.
- Output: validated effective configuration, or a validation error (exit code 4) naming `id_padding`.
- No new command or output format is introduced.

## Validation and error behavior

- `id_padding` outside 1-6 → validation error, exit code 4, message `id_padding: must be between 1 and 6`.
- Non-integer or non-scalar value → existing YAML deserialization/validation error (exit code 4).
- Out-of-range value in an existing config file → load fails (exit code 4) rather than clamping.
- ID formatting of a wide storage number → no error; wider display ID returned.

## Assumptions

- Assumption: The constraint applies to the existing `id_padding` key, not a new key.
- Assumption: Validation lives in `Config::validate()` and runs on load, set, and unset via the merged effective configuration; `init` uses the default 3.
- Assumption: Out-of-range values are rejected (exit code 4), never clamped.
- Assumption: Error message follows the established key-prefixed pattern: `id_padding: must be between 1 and 6`.
- Assumption: `IdSpec` in `src/record/identity` remains a pure formatter; config validation is the boundary that enforces 1-6. Tests may construct specs with paddings outside 1-6.
- Assumption: No migration or re-padding of existing files.
- Assumption: No UI mockups are required.

## Open questions

- None. (REQ-01.007 open questions are resolved above: existing key constrained; reject-at-load behavior; message wording follows the established pattern.)

## Traceability

| Artifact | Reference |
|---|---|
| Ticket | TASK-74 |
| Requirements | REQ-01.007, REQ-01.005 |
| Business rule | BR-016 |
| ADR | ADR-0008 (display width resolution) |
| Related tickets | TASK-2 (config foundation), TASK-75 (init walkthrough validation) |
| UI mockups | none |
