---
id: TASK-248
title: "Implement nested config parsing, validation, and emission for document entries"
created_date: '2026-11-20T17:41:37Z'
updated_date: '2026-11-22T10:32:13Z'
labels: []
references:
  - ADR-036
  - ADR-019
  - ADR-0019
  - ADR-0024
  - TECH-077
  - REQ-07.002
  - TC-103
  - TC-104
  - TC-107
  - TC-108
  - TC-109
  - TC-114
  - TC-115
  - TC-116
  - TC-117
  - TC-172
  - TC-190
  - TC-191
status: Done
assignee:
  - rust-dev
milestone: MS-033
dependencies:
  - TASK-247
ordinal: 50
documentation:
  - src/config.rs
  - src/commands.rs
  - src/init.rs
  - docs/specifications/technical/TECH-077.md
type: implementation
---

## Description
Context: MS-033 start commit c3103fc. After TASK-247 the entry is nested and the field sequence is mapping data, but the loader, validator, and emitter still assume flat scalar keys:
- `src/commands.rs::load_effective_value` (line 58) parses the file and calls `src/config.rs::merge_yaml` (line 659), which replaces `document_types` wholesale and deep-merges every other key.
- `src/config.rs::Config::validate` (line 296) validates only the flat entry; `validate_doc_entry_statuses` (line 518) covers the status list.
- `src/commands.rs::config_get` (line 158) and `config_set` (line 237) address leaves with `yaml_get`/`yaml_set`, which split on `.` only, so `fields[0]` and `fields[story]` never resolve. `config_unset` (line 329) and `config_list` (line 132) have the same limit.
- `src/config.rs::to_config_yaml` (line 773) renders every sequence in flow style through `render_yaml_flow` (line 797), so a `fields` sequence of mappings would render as `[{...}]`; `render_block_mapping` (line 816) only recurses into mappings.
- `src/init.rs:446` serializes the whole `Config`, so `init` writes every seeded scalar once the entry carries seed values.

Scope:
1. Parsing: read `document_types.<slug>` with the full key set of TASK-247, the nested `group` and `member` blocks, and `fields` as a YAML sequence of mappings. Reject an unknown key and an unknown enum value with a `document_types.<slug>.<path>` diagnostic; accept `fields: []`; reject a duplicate field key within a role and a reserved field key or `frontmatter_key`.
2. Fill and merge (ADR-036 decision 32, ADR-0019 decision 8 as amended): scalars fill from the seed when absent and a present scalar overrides; collections (`fields`, `group.fields`, `member.fields`, `statuses`, `slots`, `api_aliases`) replace wholesale when present, including `fields: []`; `group` and `member` resolve independently; `prefix` and `directory` are never filled; an absent slug stays absent and no seed entry is re-added.
3. Diagnostics: collect every problem in one pass and report them newline-separated with exit code 4, ordered top-level fields, then `document_types` in `BTreeMap` order, then entry keys, then role blocks, then the field sequence. A field diagnostic uses the keyed form `document_types.req.group.fields[story].heading`, names the field key when present, and uses the indexed form when the key is absent or ambiguous (ADR-0019 amendment to decisions 1 and 13, TECH-077 sections 1 and 8).
4. Emitter: `to_config_yaml` renders a sequence whose elements are mappings as a block sequence with `- key: value` items, keeps flat string lists inline as `[a, b]`, and reproduces a quoted heading such as `"## User Story"` unchanged. The list-valued classifier stays `config::is_list_valued_key` (ADR-0024 downstream constraint 1): `fields`, `group.fields`, `member.fields`, `slots`, and `api_aliases` are not comma-separated keys, and `statuses` remains the only list-valued key. The existing round-trip self-check in `to_config_yaml` stays.
5. Configuration commands: with no `document_types` in the file, `config set document_types.<seed-slug>.<leaf>` materializes exactly that one entry, carrying the seed prefix and directory plus the change (TC-114), and never the whole seed set; an entry for a slug with no seed entry cannot be introduced by `config set`, because `prefix` and `directory` are never filled, so it remains a file edit. `config set` applies one change, validates the resulting configuration, and writes only on success; a nested leaf creates only its path. `config get` and `config list` resolve one selector in both the keyed (`fields[story].heading`) and indexed (`fields[0].heading`) forms. `config unset` of `statuses` resets it to the seed collection and unset of a scalar removes the override. `slots` and `api_aliases` stay declarative file data (ADR-0024 amendment).
6. `init` writes a minimal file: each entry declares `prefix` and `directory`, plus `enabled` only when the walkthrough disables it; no seed-filled scalar or collection is written. The generated file validates. `init --force` still replaces an invalid file.
7. No document command, record byte, JSON payload, HTTP status, or browser page changes. The only user-visible differences are the D14 configuration-surface ones: minimal `init` output and a full `config list` with selector paths (RE-010 D14, REQ-07.002).

Invariants:
- Old configuration files must parse to the same effective document-type descriptor set after this task. A file that loaded before loads without edits, and the fill and replace rules reproduce the same prefix, directory, enabled flag, statuses, default status, model, verb set, headings, and front-matter keys (MS-033 AC-1, AC-2).
- No CLI or browser behaviour changes in this milestone beyond the configuration surface above (MS-033 AC-7).

Out of scope: the seed values (TASK-249), the document engine and typed modules (MS-034), and the read path (TASK-250). No dependency, ordinal, milestone, or assignee changes.

Assumptions:
- `config set` on a field property is refused because REQ-07.002 and REQ-01.002 make any change to a field a configuration file edit; only `config get` and `config list` resolve field selectors. Scalar and `statuses` writes keep their documented behaviour.
- Materialization for a seed slug uses the seed prefix and directory that the entry would otherwise inherit; this is the TC-114 case and is not the "never filled" rule, which governs an entry present in the file.
- `init` keeps writing one entry per enabled or disabled seed type; only the per-entry content becomes minimal.
- The existing test `tests/cli.rs::init_writes_the_eleven_document_types_with_seeded_flow_statuses` is updated to assert the minimal entry and the absence of seed-filled keys; this is the documented D14 delta, not a regression.

Definition of done: the loader, validator, selector resolver, and emitter cover the nested entry and the field sequence; every diagnostic matches the `document_types.<slug>.<path>` form in one pass; an emitted file reparses to an equal value; `init` writes minimal entries; `cargo test` and `cargo clippy --all-targets -- -D warnings` are clean.
## Acceptance Criteria
- [ ] An unknown entry key, an unknown field-item key, and an unknown enum value (`kind: invalid`, a verb other than `create` and `edit`, `reference_visibility: maybe`) are each rejected with a `document_types.<slug>.<path>` diagnostic and exit code 4, in one aggregated pass in deterministic order (TC-103, TC-104, REQ-07.002, TECH-077 sections 1 and 8).
- [ ] `fields: []` parses as a valid empty field sequence, and a grouped entry that declares a top-level `fields` is rejected (TC-107, TC-108, ADR-036 decision 26).
- [ ] A field with neither `heading` nor `frontmatter_key`, a reserved key or `frontmatter_key`, and a duplicate field key within one role are rejected with the keyed or indexed selector and the field key named; the same key in `group.fields` and `member.fields` is accepted (TC-105, TC-106, ADR-036 decisions 21 to 23).
- [ ] Fill and replace match ADR-036 decision 32: an absent scalar inherits the seed and a present scalar overrides it; an absent collection inherits the seed and a present collection replaces it including `fields: []`; a present `group` block replaces only the group role; a missing `prefix` or `directory` is rejected even for a seed slug; an absent slug stays absent and no seed entry is re-added (TC-109, REQ-07.002).
- [ ] The emitter round-trips every fixture configuration: `to_config_yaml` reparses to an equal value, a sequence of mappings renders as a block sequence, a flat string list stays inline, and `"## User Story"` is reproduced with its quotes and two hashes (MS-033 AC-4, ADR-0024).
- [ ] When the file declares no `document_types`, `config set document_types.req.enabled false` writes exactly one `req` entry carrying its seed prefix and directory plus the disabled value, writes no other seed entry, and leaves the other seed types effective from the seed; an invalid result is refused with the file byte-identical, and `config set document_types.req.statuses active,archived,review` still writes a YAML flow sequence (TC-114, TC-115, TC-190).
- [ ] `config unset document_types.req.statuses` resets the list to the seed collection, and `config unset` of a scalar removes the override (TC-116, ADR-0024 decision 6).
- [ ] `config list` prints the full effective configuration with seed-filled values and a selector path for every leaf, including `document_types.req.group.fields[story].heading`; `config get document_types.req.group.fields[0].heading` and the keyed form return the same value, and an unknown key or out-of-range index is rejected with the same diagnostic form (TC-172, TC-191, TC-117).
- [ ] `papyrus init` writes each entry with only `prefix` and `directory`, plus `enabled: false` when the walkthrough disables it, writes no seed-filled key, and the generated file validates; `init --force` still replaces an invalid file (REQ-07.002, RE-010 D14).
- [ ] No document command, record byte, JSON payload, HTTP status, or browser page changes; the only user-visible differences are the minimal `init` entry and the full `config list`, and the CLI-help goldens are unchanged (MS-033 AC-7).
- [ ] `cargo test` reports zero failures and `cargo clippy --all-targets -- -D warnings` is clean.
## Comments
- **task-refiner**: Refined for MS-033: description grounded in load_effective_value, merge_yaml, Config::validate, the config get, set, unset and list commands, to_config_yaml and init; 11 testable acceptance criteria added; TECH-077 documented. The field-edit-only rule is taken from REQ-07.002 and REQ-01.002. No dependency, ordinal, milestone, or assignee change.
- **planner**: Non-blocking refinement notes: (1) TASK-249's seed-slug wording named request as a seed entry; it is corrected to user-defined per ADR-036 decision 30 and REQ-07.002, so no further action. (2) config set field-selector semantics: TASK-248 scope already fixes keyed and indexed selector forms and the single-change validate-then-write rule; this is recorded for traceability, not a defect.
- **rust-dev**: test comment
- **rust-dev**: Diagnostics: unknown entry keys
- **rust-dev**: Implemented the nested document-type configuration surface in the config, commands and init modules.
- **rust-dev**: Diagnostics: one aggregated pass for unknown entry and field keys, unknown kind, verb and reference_visibility values, reserved entry keys (model, title, content, id_shape), field order and enabled, field key and frontmatter_key reserved keys, missing heading and frontmatter_key, duplicate field keys per role, grouped top-level fields and roles on a non-grouped type. Field paths use keyed selectors when the key is unique and indexed selectors otherwise; every line carries document_types followed by the slug and path.
- **rust-dev**: Implemented the nested document-type configuration surface in the config, commands and init modules. Diagnostics: one aggregated pass for unknown entry and field keys, unknown kind, verb and reference_visibility values, reserved entry keys (model, title, content, id_shape), field order and enabled, field key and frontmatter_key reserved keys, missing heading and frontmatter_key, duplicate field keys per role, grouped top-level fields and roles on a non-grouped type. Field paths use keyed selectors when the key is unique and indexed selectors otherwise; every line carries document_types followed by the slug and path. Fill and merge: scalars fill from the seed and present scalars override; fields, group fields, member fields, statuses, slots and api_aliases replace wholesale including an empty fields sequence; group and member resolve independently; prefix and directory are never filled; an absent slug stays absent. Selectors and commands: config get and config list resolve keyed and numeric forms including the req group story heading selector; config list prints every field property with its keyed selector path; config set refuses field sequence edits, materializes only the target entry with seed prefix and directory, and stores statuses as a flow sequence; config unset restores the seed statuses or removes a scalar override. Emission and init: mapping sequences render as block sequences, flat string lists stay inline, quoted headings such as the User Story heading round-trip, and init writes minimal entries with prefix and directory plus enabled only when disabled. Checks: env -u EDITOR -u VISUAL cargo test --no-fail-fast passed all 65 test targets; cargo clippy --all-targets -- -D warnings clean; cargo fmt --check clean. Characterization replay and CLI help goldens stay green. TASK-249: the seed values are still absent, so the seed label, seed fields and req group and member field selectors of TC-109, TC-172 and TC-191 become observable once the seed table lands; the fill, flattening and selector mechanics are in place. Open item: TC-114 says the other seed types stay effective after config set, while the amended ADR-0019 decision 8 and the task acceptance also require an absent slug to stay absent. With only req written, nfr is absent under the wholesale entry set. Implemented the documented wholesale and absent-slug rule; the clashing TC-114 clause needs a decision.
- **orchestrator**: Acceptance met: nested parsing and document_types diagnostics, seed fill and collection replace, block-sequence emission with round-trip, selector resolution for get and list, set materializing one entry and refusing field edits, unset restoring seed statuses, minimal init; full suite green, clippy and fmt clean. TC-114 clause conflict recorded.
