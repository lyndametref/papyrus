---
id: TASK-106
title: Support list-valued configuration keys in config set, get, and list
status: Done
assignee:
  - rust-dev
created_date: '2026-11-13T21:25:00Z'
updated_date: '2026-11-14T14:24:00Z'
labels:
  - rust
milestone: MS-018
dependencies:
  - TASK-104
references:
  - REQ-01.002
  - REQ-07.002
  - ADR-0019
  - ADR-0024
documentation:
  - docs/architecture/adr/ADR-0024-list-valued-configuration-keys.md
  - docs/requirements/business/REQ-01.002.md
  - docs/specifications/project-initialization-and-configuration.md
priority: high
ordinal: 41050
---


## Description
**Why.** `config set` currently parses every value as a YAML scalar and rejects any sequence, so the list-valued keys — the top-level task `statuses` and each `document_types.<slug>.statuses` — cannot be written through the documented configuration surface. They can only be introduced by hand-editing `papyrus.config.yml`, which contradicts REQ-01.002/REQ-07.002 and leaves the ADR-0019 repair path incomplete for the keys most likely to be invalid.

**What.** Add one list-key classification and a comma-separated value form. `config set` classifies the key first (never the value), splits the raw value on commas, trims each element, rejects an empty value or empty element with exit 4 naming the key, stores the result as a YAML flow sequence, validates the resulting configuration with the existing ADR-0019 validator, and writes only when valid. `config get` and `config list` render the same flow form in declared order. Scalar keys keep their current scalar semantics, where a comma is ordinary data. `config unset` is unchanged. No escaping, quote interpretation, new flags, or browser list editor are added (ADR-0024).

**In scope.**

- `src/config.rs`: the list-key classification and the comma-list parser.
- `src/commands.rs`: `config_set`, `config_get`, `config_list` list behavior; keep the existing validation-before-write path.
- YAML persistence: list values written as flow sequences.
- `config set --help` and the `papyrus(1)` / `papyrus-config(5)` entries in `README.md`.
- Unit and integration tests.

**Out of scope.**

- New list keys; `--json`, `--append`, or `--remove` flags; list editing in the browser configuration view (its editable-key whitelist is unchanged).
- Changing scalar parsing, key parsing, relocation, tolerant-load rules, or the ADR-0019 validator.
- Seeding document statuses at init (TASK-112) and the broader README/upgrade-notes pass (TASK-116).
- Editing requirement or business-rule files.

**Assumptions.**

- An element that is empty after trimming is rejected rather than dropped; otherwise `a,,b` would persist an empty task status as valid data.
- A list element containing a newline or other control character is rejected (exit 4), because a single-line flow sequence cannot represent it unambiguously (NFR-02.002).
- Flow rendering is `[a, b]` (brackets, comma, single space), matching the existing top-level `statuses` rendering in `config list`.
- `config get` on a document `statuses` key that is not declared remains `key not found` (exit 3); unset means "no declaration", and the effective document model falls back to `active`/`archived`.
- Setting `document_types.<slug>.statuses` for an entry without `prefix`/`directory` is refused by the existing validator; fields are not auto-filled.

## Acceptance Criteria
- [x] A single classification in `config` (`LIST_VALUED_KEYS` plus the document-statuses rule) classifies exactly `statuses` and `document_types.<slug>.statuses`; every other key is scalar and classification never inspects the supplied value.
- [x] `config set statuses "ToDo, Initiated, Done"` writes `statuses: [ToDo, Initiated, Done]` as a YAML flow sequence; the file reparses to exactly those strings in that order; `config get statuses` prints `[ToDo, Initiated, Done]`; `config list` prints `statuses: [ToDo, Initiated, Done]`.
- [x] `config set document_types.audit.statuses "active, archived, open"` writes the flow form under the existing entry without changing its `prefix`, `directory`, or `enabled`; `config get` returns `[active, archived, open]`; `config list` prints `document_types.audit.statuses: [active, archived, open]` and omits that line for entries that declare no status list.
- [x] Parsing trims surrounding whitespace only (`" a , b "` -> `a`, `b`), preserves order, case, internal spaces, and duplicates, and never YAML-interprets elements (`true`, `1`, and `[a]` stay strings); scalar keys still parse as before, so `config set project_name "Acme, Inc"` persists one scalar and `config get` prints `Acme, Inc`.
- [x] An empty or whitespace-only list value, an element that is empty after trimming, and an element containing a newline or control character are each rejected with exit 4 and a message naming the key; nothing is written and an existing file is byte-identical.
- [x] `config get` renders a block-style list read from a hand-edited file in flow form without modifying the file; a later `config set` of an unrelated key reserializes declared lists in flow form.
- [x] Validation precedes persistence: a list that violates ADR-0019 rules (missing `active`/`archived`, duplicate value, invalid status character, `statuses` on `ui-mockup`, or a top-level `default_status` not in the new task list) exits 4 with the validator message naming the offending key(s) and leaves the file byte-identical.
- [x] Repair path: with an already-invalid document status list in the file, `config set document_types.<slug>.statuses "active,archived"` succeeds and writes a valid list; `config get|set|list` load tolerantly, while an invalid resulting configuration is still refused.
- [x] `config unset statuses` removes the key and `config get statuses` returns the default task list; `config unset document_types.<slug>.statuses` removes the declaration without writing an empty list when the entry has no dependent `default_status`, and `config get` then exits 3; an entry whose `default_status` would no longer be declared is refused (exit 4) and unchanged.
- [x] No escaping: `active\,archived` is split at the comma into `active\` and `archived` and rejected by status validation (exit 4) with nothing written; `config set --help` states the comma-separated form for list keys, flow-sequence storage, and that commas, quotes, and backslashes are not escaped.
- [x] `--json` output keeps list values as JSON arrays of strings for `config get` and `config list`.
- [x] README `papyrus(1)` (`config set`) and `papyrus-config(5)` (`statuses`, `document_types`) document the comma-separated form, flow-sequence storage, empty-value rejection, and no-escape limitation, with no behavior change documented for scalar keys.

## Definition of Done
- [x] #1 `cargo build`, `cargo test`, and `cargo clippy` are clean.
- [x] #2 Unit tests cover the classification and parser; integration tests cover set/get/list round-trips, empty value and empty element, invalid lists, scalar comma preservation, unset, the repair path, JSON output, and help text.
- [x] #3 `config set` writes list values as flow sequences and refuses an invalid resulting configuration with the file byte-identical.
- [x] #4 No new dependency or flag is added; scalar keys, key parsing, relocation, and the ADR-0019 validator keep their existing behavior.
- [x] #5 `config set --help`, the README config man pages, and `docs/specifications/project-initialization-and-configuration.md` describe the implemented behavior.
- [x] #6 ADR-0024 downstream constraints 1–5 are satisfied and cited in the final summary; the TASK-112 renderer follow-up is noted.


## Implementation Notes
## Implementation plan

Authority: ADR-0024 (accepted), ADR-0019, REQ-01.002, REQ-07.002, and the TASK-111 revision of `docs/specifications/project-initialization-and-configuration.md`.

### 1. List-key classification (`src/config.rs`)

- Add `LIST_VALUED_KEYS` with the exact list key `statuses`, plus the declarative rule for `document_types.<slug>.statuses` where the slug is one non-empty dotted segment.
- Add `is_list_valued_key(key)` as the only classifier: exact match in `LIST_VALUED_KEYS` or the document-statuses rule. It must not inspect the value.
- `config_set`, `config_get`, and `config_list` all call this classifier.
- Unit-test that both sequence-typed fields (`Config::statuses`, `DocTypeEntry::statuses`) are covered and that `default_status`, `document_types.<slug>.default_status`, prefixes, directories, `enabled`, `id_padding`, `browser_*`, and a four-segment key are scalar.

### 2. Comma-list parser (`src/config.rs`)

- `parse_list_value(key, raw)` returns the ordered `Vec<String>`: split on `,`, trim each element, preserve order, case, internal spaces, and duplicates.
- Return a validation error whose message begins with `<key>:` when the value is empty or whitespace-only, any element is empty after trimming, or any element contains a newline or other control character.
- Do not YAML-parse elements and do not apply scalar sanitizers; `true` and `1` stay strings.

### 3. `config set` (`src/commands.rs`)

- Classify the key before parsing the value. List path: build a `Value::Sequence` from the parser and skip the YAML-scalar parse, scalar sanitization, `browser_port`/`browser_host` checks, and folder relocation. Scalar path: unchanged.
- Keep `valid_sanitized_key`, default `document_types` materialization, `yaml_set`, merge, browser-key validation, deserialization, and the existing `cfg.validate(Some(project_dir))` call; write only after validation succeeds.
- Persist with the flow renderer (step 4).
- Setting `document_types.<slug>.statuses` on an entry without `prefix`/`directory` stays refused by the existing validator; do not auto-fill fields.

### 4. Flow-sequence persistence (`src/config.rs`)

- `yaml_serde` 0.10 has no flow-style option. Add a small renderer over `yaml_serde::Value` used by `config set`/`config unset`: mappings in block style, sequences inline as `[a, b]`, scalars and keys rendered by the YAML scalar serializer so quoting stays correct, empty map as `{}`.
- Round-trip every write through `yaml_serde::from_str`; assert the written text contains the exact `statuses: [a, b]` line.
- Leave init's writer unchanged in this task; TASK-112 adopts the renderer when it seeds default status lists.

### 5. `config get` (`src/commands.rs`)

- For a list-valued key whose effective value is a sequence, render `[a, b]` in declared order; this covers block-style lists read from a hand-edited file.
- Keep missing key -> `AppError::NotFound` (exit 3), scalar values unchanged, and the existing `--json` array conversion.

### 6. `config list` (`src/config.rs`)

- `Config::flattened` already renders the top-level `statuses` as `[a, b]`; keep it.
- After each `document_types.<slug>.enabled` line, append `document_types.<slug>.statuses` as `[a, b]` when `statuses` is `Some`, and omit the line when it is `None`. Preserve `BTreeMap` order and all other keys.

### 7. Help and documentation

- `src/cli/builtin.rs`: give `ConfigCmd::Set` long help stating that list keys take a comma-separated value, are stored as a flow sequence, reject an empty value, and have no escape or quote form; scalar keys keep comma-as-data.
- `README.md`: update the `papyrus(1)` `config set` paragraph and the `papyrus-config(5)` `statuses`/`document_types` text with the same rules. Do not rewrite the rest of the README (TASK-116 owns the broader dynamic-command pass).

### 8. Verification

- `cargo build`, `cargo test`, `cargo clippy`.
- Add the tests in the test plan.
- Confirm the existing browser config-edit whitelist still rejects `/config/edit?key=statuses` (404).

## Test plan

**Classification.** Positive: `statuses`, `document_types.audit.statuses`. Negative: `default_status`, `document_types.audit.default_status`, `document_types.audit.prefix`, `document_types.audit.directory`, `document_types.audit.enabled`, `id_padding`, `browser_port`, `document_types.a.b.statuses` (four segments).

**Parser.** `" a , b "` -> `a`, `b`; single element; order/case/duplicates preserved; empty, whitespace-only, `a,,b`, `a,` rejected; `true,1,[a]` remain strings; newline and control characters rejected.

**Round-trip.** Top-level: set writes the exact flow line, file reparses, `get` prints the flow form, `list` prints the key, JSON is an array. Document: set on an existing entry leaves `prefix`/`directory`/`enabled` untouched; `get` and `list` match; `list` omits undeclared entries.

**Source styles.** A hand-written block list `statuses:\n  - active\n  - archived` is printed as `[active, archived]` by `get`; an unrelated `config set` rewrites declared lists in flow form.

**Rejections.** Empty/whitespace value, empty element, missing `active`/`archived`, duplicate, invalid character, `ui-mockup` statuses, and a top-level `default_status` not in the new task list all produce exit 4, name the key, and leave the file byte-identical.

**Repair path.** A pre-existing invalid document status list is repaired by a valid `config set`; an invalid replacement is refused; `config get|list` still load tolerantly.

**Scalars.** `project_name "Acme, Inc"` stays one scalar; booleans and numbers are unchanged.

**Unset.** Top-level `get` falls back to the default task list; document `get` exits 3 after removal; no `[]` is written. An entry whose `default_status` would no longer be declared is refused (exit 4) and unchanged.

**No escape.** `active\,archived` splits into `active\` and `archived` and is rejected by status validation.

**Help/docs.** `config set --help` contains the comma, flow, and no-escape statements; README sections match.

**Regression.** Existing config/relocation tests pass; browser `/config/edit?key=statuses` remains 404.

## Risks and notes

- The flow renderer changes formatting for non-list nodes. Keep it deterministic and round-trip tested; comments are already discarded by the current writer.
- A classification mistake could turn a scalar into a list. Four-segment and default-status keys are explicit negative tests.
- The empty-element rule is an interpretation of ADR-0024's empty-list rule; it is recorded as an assumption for review.
- TASK-112 must reuse the renderer for seeded status lists, or init will write block lists. Note this in the final summary.
- README edits shift the line ranges quoted by BR-003/BR-016; TASK-115 updates those business rules.

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Implemented list-valued config keys for `statuses` and `document_types.<slug>.statuses`: single classifier (ADR-0024 constraint 1), comma-separated parsing with trim-only (2), flow-sequence persistence and validate-before-write via ADR-0019 (3), empty list rejected and unset as the only way back to defaults (4), no escape/quote form with help and README stating it (5). Browser list keys stay read-only (404 on edit routes) per ADR-0024. TASK-112 follow-up: init must reuse `config::to_config_yaml` when seeding default status lists, otherwise it writes block-style lists. Verified: 731 tests pass, clippy/fmt clean, file byte-identical on every refusal.
