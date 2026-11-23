---
id: TASK-258
title: "Implement config set, list, get, and unset with selector paths"
created_date: '2026-11-20T17:42:06Z'
updated_date: '2026-11-22T14:28:25Z'
labels: []
references:
  - ADR-036
  - REQ-001.002
  - REQ-007.002
  - TECH-077
  - ADR-0019
  - ADR-0024
  - TC-114
  - TC-115
  - TC-116
  - TC-117
  - TC-172
  - ADR-042
status: Done
assignee:
  - rust-dev
milestone: MS-034
dependencies:
  - TASK-255
ordinal: 60
documentation:
  - src/commands.rs
  - src/config.rs
  - tests/nested_config_cli.rs
  - tests/config_list_keys_cli.rs
  - tests/config_relocation_cli.rs
type: implementation
---

## Description
## Context
`src/commands.rs` implements `config_list`, `config_get`, `config_set`, and `config_unset`; `src/config.rs` holds `parse_selector` (~1600), `valid_config_selector` (~1706), `selector_targets_field` (~1723), `resolve_selector` (~1730), `yaml_set`/`yaml_unset` (~2667), `flattened` (~2045), `is_list_valued_key`/`parse_list_value` (~19-57), and the seed resolution in `load_effective_value`. Current state: `get` and `list` resolve keyed and numeric field selectors; `set` and `unset` reject every bracketed selector and every `fields` selector; `statuses` is the only comma-separated list; seed-filled values are printed without a marker. `config set` already materializes only the target seed entry (`materialize_target_entry`), matching ADR-036 decision 32; REQ-007.002's wholesale-replace wording is stale (AUD-140 L-4).

## Scope
- `config list`: print the full effective configuration, seed-filled values included, with a selector path for every leaf and keyed selectors for field properties.
- `config get`: accept the keyed and numeric `[n]` field forms; reject an unknown key, a keyed selector with no unique match, or an out-of-range index with exit 4 and the selector path.
- `config set`/`config unset`: implement scalar-leaf writes, target-entry materialization, nested-path creation, validation before persistence, and byte-identical failure. Reject every selector inside `fields`, `group.fields`, or `member.fields` as file-edit-only: `config set` and `config unset` never add, remove, reorder, or edit a field property (ADR-042 ruling 5; ADR-0019 decision 10 as amended; ADR-0024's 2026-11-20 amendment). Preserve folder relocation.
- `config unset`: remove a scalar override and reset a collection to its seed collection.
- Lists: only `statuses` and `document_types.<slug>.statuses` keep the comma syntax and flow-sequence storage; `fields` and role field sequences are never comma-set. `api_aliases` and `slots` are declarative file data: they are not `config set` targets and never use comma syntax, and `slots` is a sequence of mappings (ADR-042 ruling 6).
- Diagnostics: make field string type errors name `document_types.<slug>.fields[<key>].<property>` or the indexed form (AUD-140 L-1).
- Update `tests/nested_config_cli.rs` (the bracket-selector expectation), `tests/config_list_keys_cli.rs`, `tests/config_relocation_cli.rs`, and the config unit tests.

## Contract (ADR-042 rulings 5 and 6)
`config get` and `config list` accept keyed and numeric field selector paths and resolve them to effective leaves. `config set` and `config unset` accept scalar leaves only and reject any selector inside a `fields`, `group.fields`, or `member.fields` sequence with a selector-path diagnostic and no write; field properties are file-edit-only. Only `statuses` keeps comma syntax on set and unset; `api_aliases` and `slots` are declarative file data and are not set targets.

## Out of scope
The document engine (TASK-255), grouped and mockup surfaces (TASK-256), title contract (TASK-257), module deletion (TASK-259), help goldens (TASK-260), and the browser.

## Grounding
ADR-036 decision 32; ADR-0019 decisions 10 and 13 and the 2026-11-20 amendment; ADR-0024 and its 2026-11-20 amendment; ADR-042 rulings 5 and 6; TECH-077 sections 3, 5, 6, and 8; REQ-001.002; REQ-007.002; TC-114, TC-115, TC-116, TC-117, TC-172; AUD-140 L-1 and L-4.

## Definition of done
The narrowed selector contract is implemented: get/list resolve field leaves; set/unset reject field-sequence selectors as file-edit-only; seed materialization writes only the target entry; only statuses keeps comma syntax; api_aliases and slots are not set targets; diagnostics match the accepted decisions; the config suites pass; `cargo test`, `cargo clippy --all-targets -- -D warnings`, and `cargo fmt --check` are clean.

## Acceptance Criteria
- [ ] #1 config list prints the full effective configuration including seed-filled values, and each leaf is addressed by its selector path, with field properties using the keyed form such as document_types.req.member.fields[story].heading. (MS-034 AC-3; REQ-001.002; TC-172)
- [ ] #2 config get accepts both the keyed form and the zero-based numeric form such as document_types.req.group.fields[0].required, and both resolve to the same element and value. (MS-034 AC-3; TC-172)
- [ ] #3 An unknown selector, a keyed selector that matches no unique field, or an out-of-range index fails with exit 4 and a diagnostic naming the selector path; no value is printed and nothing is written. (MS-034 AC-3; TC-117)
- [ ] #4 config set on a document type materializes only the target seed entry with its seed prefix and directory when the file has no document_types map; it never writes the whole seed set, and the other seed slugs remain effective from the seed table. (MS-034 AC-3; ADR-036 d32; TC-114)
- [ ] #5 config unset of a scalar removes the override so the seed value applies again; config unset of a collection resets it to the seed collection. (MS-034 AC-3; TC-116)
- [ ] #6 Only statuses and document_types.<slug>.statuses keep the comma-separated value form and are stored as a YAML flow sequence; an empty value is rejected; fields and role field sequences are never comma-set; api_aliases and slots are declarative file data and are not config set targets. (MS-034 AC-3; ADR-0024; ADR-042 ruling 6)
- [ ] #7 config get and list resolve keyed and numeric field selector leaves; config set and unset reject every selector inside fields, group.fields, or member.fields with a selector-path diagnostic and no write, because field edits are file-edit-only. A field selector accepted by set or unset is a defect. (MS-034 AC-3; ADR-042 ruling 5; ADR-0019 d10 as amended; ADR-0024 amendment)
- [ ] #8 A field property with the wrong string type (key, heading, frontmatter_key, option, label, or help) is rejected with a document_types.<slug>.fields[<key>].<property> diagnostic, using the keyed form when the key is present and the indexed form otherwise. (AUD-140 L-1; TECH-077 section 3; TC-103, TC-104)
- [ ] #9 The configuration that would result from set or unset is validated before persistence; an invalid result exits 4, writes nothing, and leaves the file byte-identical; folder relocation behavior is unchanged. (MS-034 AC-3; REQ-001.002; TC-115)
- [ ] #10 The nested config, config list keys, config relocation, and config unit suites pass after the selector changes; cargo test, cargo clippy --all-targets -- -D warnings, and cargo fmt --check are clean. (DoD)
## Comments
- **task-refiner**: AUD-140 L-1 reproduced: label, help, and frontmatter_key (and the other string-typed field properties) supplied as a non-string fail during deserialization with a bare invalid type message, for example `invalid type: integer 5, expected a string`, and do not name the selector path. A config with document_types.deploy.fields[description].label: 5 must report document_types.deploy.fields[description].label. Cover the keyed and indexed diagnostic forms.
- **task-refiner**: AUD-140 L-4: REQ-007.002 and TC-114 still contain the stale never-materialize-the-seed wording. ADR-036 decision 32 and the AUD-140 clarification govern: the eleven seed slugs are always effective, a present document_types map is an overlay, and config set writes only the target entry. Implement to ADR-036 d32 and record the requirement wording as owned by requirements-engineer in Phase 2.
- **planner**: ADR-042 rulings 5 and 6 applied: the field-set/unset blocker is cleared. config get and config list resolve keyed and numeric field leaves; config set and config unset reject every selector inside fields, group.fields, or member.fields as file-edit-only with no write. Only statuses keeps comma syntax; api_aliases and slots are declarative file data. MS-034 AC-3 is narrowed accordingly. The seed-filled-value marker is not introduced because no accepted source defines it.
- **rust-dev**: test
- **rust-dev**: rust-dev implementation evidence for MS-034 AC-3 and AUD-140 L-1. Selectors: config get and config list resolve keyed and numeric field leaves (member.fields[story].heading and group.fields[0].heading resolve the same leaf); config list prints effective leaves with keyed field selectors. Unknown keyed selectors, ambiguous matches, and out-of-range indexes exit 4 and name the selector path; no value is printed. Writes: config set and unset reject every fields, group.fields, and member.fields selector as file-edit-only; they also reject api_aliases and slots as declarative file data with the selector path, so fields is never comma-set. Only statuses and document_types.<slug>.statuses keep comma syntax and flow-sequence storage; empty values stay rejected. Target-only seed materialization, validate-before-write, byte-identical rejected writes, and folder relocation are unchanged. AUD-140 L-1 closed: non-string label, help, frontmatter_key, option, key, and heading values report document_types.<slug> field selector plus property with must be a string, keyed when the key is unique and indexed otherwise, and no longer fall through to the bare serde message. Checks: nested_config_cli 22 passed (2 new, 1 updated), config_list_keys_cli 14 passed, config_relocation_cli 7 passed; full cargo test 1254 passed and 0 failed; cargo clippy --all-targets -- -D warnings clean; cargo fmt --check clean. Branch task-TASK-258; no commit per instruction; task status left In-Progress.
- **orchestrator**: Acceptance met: config get and list resolve keyed and numeric field leaves, set and unset reject field and declarative collection edits as file-edit-only, statuses keep comma syntax with flow storage, target-only materialization and validate-before-write preserved; AUD-140 L-1 field string type errors now carry selector paths; 1254 tests pass.
