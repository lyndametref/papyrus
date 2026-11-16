---
id: TASK-117
title: Include document type default_status in config list (G-4)
status: Done
assignee:
  - rust-dev
created_date: '2026-11-14T20:31:00Z'
updated_date: '2026-11-15T10:23:00Z'
labels:
  - rust
  - config
milestone: MS-019
dependencies: []
references:
  - docs/specifications/project-initialization-and-configuration.md
  - >-
    docs/specifications/config-defined-document-types-verification-and-packaging.md
  - docs/architecture/adr/ADR-0024-list-valued-configuration-keys.md
documentation:
  - src/config.rs
  - tests/config_list_keys_cli.rs
  - tests/release_document_types_smoke.rs
priority: medium
ordinal: 41240
---


## Description
**Why.** TASK-117 recorded G-4: `config list` flattens `document_types.<slug>` to `prefix`, `directory`, `enabled`, and `statuses`, but omits `default_status` even when the configuration sets it. `default_status` is a validated key that can invalidate the whole configuration (it must be a declared, non-archived status and only on a type with a status list), yet the primary inspection surface hides it, so an operator cannot see the value the validator is rejecting and the documented repair path starts blind. `src/config.rs::flattened()` builds that listing from a `BTreeMap<String, DocTypeEntry>` and simply has no `default_status` branch.

**What.** Emit `document_types.<slug>.default_status` in `config list` whenever the entry declares one, inside that slug's block, immediately after the same slug's `statuses` line; omit the key entirely when the entry declares none. `config get`, `config set`, `config unset`, validation, and repair behavior stay byte-for-byte unchanged.

**In scope.**

- `src/config.rs::flattened()`: within the existing `document_types` loop, after the `statuses` push, push `document_types.<slug>.default_status` from the entry's `Option<String>` only when it is `Some`. No other ordering change; slugs stay in ascending key order because the map is a `BTreeMap`.
- Per-slug block order becomes: `prefix`, `directory`, `enabled`, `statuses` (if declared), `default_status` (if declared). When an entry declares `default_status` but no `statuses`, the key follows `enabled`.
- JSON: no serialization change is needed — `Config` is serialized directly for `config list --json` (`src/commands.rs::config_list`) and `DocTypeEntry::default_status` already carries `skip_serializing_if = "Option::is_none"`. Add regression assertions only.
- Tests: one unit test on `flattened()` and integration coverage in `tests/config_list_keys_cli.rs` for declared, undeclared, invalid, JSON, and repair-path cases; update the inverted TASK-117 assertion in `tests/release_document_types_smoke.rs`.
- Documentation: the behavioral claim now lives in `docs/specifications/project-initialization-and-configuration.md` §3 item 21 (already written for this ticket). Append one resolution line to the G-4 entry of `docs/specifications/config-defined-document-types-verification-and-packaging.md`, whose current text still states the omission.

**Out of scope.**

- Changing key parsing, list-key classification, validation rules, the tolerant repair load, the relocation path, or the repair-mode contract.
- Adding new keys or flags; seeding `default_status` at init (TASK-112 behavior stays: no generated entry declares it).
- Browser code changes: the `/config` view renders `flattened()`, so a declared `default_status` will appear there as an ordinary editable scalar row automatically. That is an accepted consequence, not a feature to design; no browser source edit and no new route.
- Reopening TASK-111 or TASK-117; this closes their recorded follow-up.

## Acceptance Criteria
- [x] config list prints document_types.<slug>.default_status: <value> for every entry that declares a default status, immediately after that slug's statuses line when the entry also declares a status list, and immediately after that slug's enabled line when it does not
- [x] No document_types.<slug>.default_status key is printed for an entry that declares none: no empty value, no placeholder, and no bare key; slugs remain in ascending key order
- [x] config list --json carries document_types.<slug>.default_status with the same string value as the text listing, and omits the member when the entry declares none; config get document_types.<slug>.default_status returns exactly what it returned before this change
- [x] config list still prints an invalid default_status (tolerant load, exit 0) so the rejected value is inspectable, while config set and config unset still validate before write and reject the same invalid declaration with exit 4 and an unchanged message naming document_types.<slug>.default_status
- [x] A unit test covers flattened() with a declared key, a default_status without statuses, and an undeclared entry; integration tests cover the printed line and its adjacency, JSON presence and absence, the unset case, an invalid value, and the config set repair path
- [x] The inverted TASK-117 G-4 assertion in tests/release_document_types_smoke.rs now asserts the printed line instead of the omission, and the whole suite passes

## Definition of Done
- [x] #1 cargo build, cargo test, and cargo clippy --all-targets are clean.
- [x] #2 No new dependency, configuration key, CLI flag, or browser route is introduced; the only behavior change is the one additional listing line.
- [x] #3 G-4 is closed with command evidence in the ticket notes, and the stale omission statement in the G-4 entry of docs/specifications/config-defined-document-types-verification-and-packaging.md is updated to record its resolution.
- [x] #4 No change to BR-003, BR-016, or REQ-01.002 is required or made; docs/specifications/project-initialization-and-configuration.md §3 item 21 and the traceability table describe the delivered behavior.


## Implementation Notes
1. `src/config.rs`, `flattened()` (document_types loop, currently lines ~433-449): after the existing `if let Some(statuses) = &v.statuses { ... }` push, add the same shape of guarded push for `format!("document_types.{}.default_status", k)` from `v.default_status`. Ordering follows loop construction, so per-slug order is prefix, directory, enabled, statuses?, default_status?; `BTreeMap` keeps slugs ascending. No change to the function signature, to any other key's position, or to `is_list_valued_key` (the key stays scalar).
2. No source change for JSON: `config_list` serializes `Config` directly and `DocTypeEntry::default_status` is `#[serde(default, skip_serializing_if = "Option::is_none")]`, so the member already appears only when declared. Verify with assertions, do not add serialization code.
3. Unit test in `src/config.rs` tests module (next to `flattened_renders_document_status_lists_in_flow_form`): build a config whose entry declares statuses plus default_status, one that declares default_status only, and `ui-mockup` with neither; assert the key/value, assert the default_status key index is exactly one past the statuses key index for that slug, assert absence (not empty string) for the undeclared entry.
4. Integration in `tests/config_list_keys_cli.rs` (reuse the `Project` harness and `write_config`): (a) declared entry — text contains `document_types.audit.default_status: open` and its line immediately follows `document_types.audit.statuses: [...]`; (b) unset — absent from text and absent as a JSON member (`json["document_types"]["audit"].get("default_status").is_none()`); (c) invalid — an out-of-list value is still printed by `config list` (exit 0, tolerant load) while `config set task_prefix X` still exits 4 naming `document_types.audit.default_status`; (d) repair — `config set document_types.audit.default_status active` exits 0 and the next `config list` shows `active`; (e) JSON value parity with `config get`.
5. `tests/release_document_types_smoke.rs` (~line 674, TASK-117 G-4 block): flip `assert!(!run_ok(...).contains("document_types.note.default_status"))` to assert the line `document_types.note.default_status: archived` is present, keeping the surrounding invalid-config and repair steps intact. Check `tests/browser_cli.rs` `/config` tests for any row-count assumption; none is expected, so no edit unless a failure appears.
6. Docs: verify §3 item 21 of `docs/specifications/project-initialization-and-configuration.md` matches the shipped line; append "Resolved by TASK-122" plus the observed command output to the G-4 entry in `docs/specifications/config-defined-document-types-verification-and-packaging.md`.
7. Verify: `cargo build`, `cargo clippy --all-targets -- -D warnings`, `cargo test --lib config::`, `cargo test --test config_list_keys_cli`, `cargo test --test release_document_types_smoke`, then full `cargo test`.


## Notes
Refinement (TASK-122). Behavior contract recorded in docs/specifications/project-initialization-and-configuration.md §3 item 21 (ordering, omission, JSON parity, browser side effect). Code facts confirmed at refinement time: flattened() builds the document_types block from BTreeMap<String, DocTypeEntry> and has a statuses branch but none for default_status; DocTypeEntry::default_status is Option<String> with #[serde(default, skip_serializing_if = "Option::is_none")], so config list --json already emits it and needs only regression assertions; config_list uses the tolerant load, so an invalid value stays printable. Assumptions: (1) adjacency after statuses is the intended stable position; (2) an entry with default_status and no statuses is invalid but still listed, with the key directly after enabled; (3) the browser /config row that appears as a consequence of the shared flattened() listing is accepted without browser code changes. Dependencies: none outstanding; TASK-111, TASK-112, and TASK-117 are delivered. Open questions: none.

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
G-4 closed: Config::flattened now prints document_types.<slug>.default_status immediately after that slug's statuses (or after enabled when no statuses), omitted when unset, in BTreeMap slug order. Text and JSON agree with config get; no serialization code change. Evidence: 'config list' shows document_types.audit.default_status: open after statuses; absent for note; invalid value still listed (exit 0) while 'config set task_prefix PAP' and 'config unset document_types.audit.statuses' exit 4 with 'document_types.audit.default_status: open is not a declared status' and byte-identical file; repair via 'config set document_types.audit.default_status active' succeeds and listing shows active. Tests: config_list_keys_cli 14/14, release_document_types_smoke 8/8 (inverted assertion flipped), full suite 828 passed / 0 failed / 22 ignored; clippy and fmt clean. Spec §3 item 21 and the G-4 resolution line updated.
