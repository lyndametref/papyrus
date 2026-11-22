---
id: TASK-310
title: "Correct MS-034 field-driven CLI engine findings F1-F15 per AUD-141"
created_date: '2026-11-22T15:26:32Z'
updated_date: '2026-11-22T16:16:50Z'
labels: []
references:
  - AUD-141
  - MS-034
  - REQ-05.001
  - REQ-05.002
  - REQ-05.003
  - REQ-05.008
  - REQ-05.011
  - REQ-07.003
  - REQ-07.004
  - BR-009
  - BR-010
  - BR-011
  - BR-014
  - ADR-036
  - ADR-042
  - TECH-042
  - TECH-066
  - TECH-070
  - TC-119
  - TC-120
  - TC-121
  - TC-125
  - TC-126
status: Done
assignee:
  - rust-dev
milestone: MS-034
dependencies:
  - TASK-260
  - TASK-308
ordinal: 85
documentation:
  - src/document.rs
  - src/registry.rs
  - src/config.rs
  - src/record/frontmatter.rs
  - src/record/store.rs
  - src/record/identity.rs
  - src/record/patch.rs
  - src/sanitize.rs
  - src/ui_mockup.rs
  - src/cli/tree.rs
  - tests/document_dispatch.rs
  - tests/req_cli.rs
  - tests/ui_mockup_cli.rs
  - tests/glossary_cli.rs
  - tests/nested_config_cli.rs
type: implementation
---

## Description
## Context
AUD-141 (code audit of MS-034) returned **corrections required**: 4 high, 6 medium, 5 low findings. MS-034 cannot be verified while this task is open. This is the single correction task for MS-034: it depends on TASK-260 and TASK-308 (both Done), sits at ordinal 85, and TASK-261 (milestone verification, ordinal 90) depends on it and must verify afterwards.

AUD-141's evidence, file locations, and severities are authoritative; do not re-derive them. Scope is correction only: no new features, no requirement, ADR, specification or test-case edits, no new crates.

## Scope, in order

### High
1. **F1 create-group validation bypass** (`src/document.rs` create_group_document). It runs its own argument path: no stdin resolution, no required-field enforcement, no frontmatter_key, no field_defaults. Correction: run `resolve_field_stdin`, `ensure_required_fields` with `descriptor.field_defaults`, and the shared create-op builder exactly as `create_document` does. Test: stdin `-`, required-field diagnostics (every key, no write), frontmatter key and defaults on create-group.
2. **F2 non-atomic ID allocation** (`src/document.rs` allocation sites; `src/record/store.rs` write_atomic rename). Scan-then-rename overwrites: 12 concurrent tech-spec creates produced 2 files. Correction: reserve the ID atomically (create_new or no-replace link), retry on AlreadyExists with the next number, never replace an existing target. Test: 12 concurrent single and 12 concurrent grouped-member creates; an existing target is not overwritten.
3. **F3 YAML-unsafe title and label rendering** (`src/record/frontmatter.rs` format_scalar and format_kv; `src/document.rs` create). Titles starting with YAML indicators break the next command; a bracketed title becomes a sequence. Correction: quote every YAML-unsafe scalar, never coerce a scalar to a sequence, and re-parse the rendered front matter before writing (non-round-trip is a validation error, no write). Test: round-trip table for indicator prefixes, bracketed title, labels, read-back.
4. **F4 unescaped HTML mockup metadata** (`src/ui_mockup.rs` render and parse). A title can set status on an unrelated edit; entities are rewritten and the visible-paragraph fallback is asymmetric. Correction: escape every rendered field, parse metadata only from the fixed comment block, preserve the raw title. Test: injection titles, entity round-trip, comment-block-only parsing.

### Medium
5. **F5 group reference normalization** (create --group REQ-1 fails while view REQ-1 works). Correction: resolve through `resolve_in_dirs`, use the canonical ID. Test: accepted reference forms produce the same canonical record.
6. **F6 role-scoped edit fields** (edit mixes group and member fields). Correction: resolve the role first, use only that role's fields, reject other-role options as one exit-4 validation outcome. Test: both directions, no write, help and runtime agreement.
7. **F7 config validation of the derived command surface** (field key or option label panics; json shadows the global flag; cross-role duplicate option panics; ui-mockup fields accepted and ignored; empty derived option accepted). Correction: validate reserved names, cross-role duplicates, mockup fields, and the empty option; emit selector-path diagnostics. Test: tree-build and help for every seed descriptor plus an adversarial config.
8. **F8 title sanitizer** (rejects CI-CD with a forward slash and `A backslash B`). Correction: title-specific sanitizer without path rules; keep rejection of empty, whitespace, multi-line, control and dual-source titles. Test: case table plus create round-trip.
9. **F9 duplicated field and role resolution** in engine and registry. Correction: one shared function. Test: a grouped type with different group and member fields agrees across help, create-group, create and edit.
10. **F10 vacuous tests** (tautological handler assertion, req_errors accepts two causes, missing-title only checks is_err). Correction: assert concrete handlers, messages and no-write.

### Low
11. **F11 archived-edit guard** unreachable for config-defined types (edit returns exit 3). Correction: resolve archived for config-defined types or delete the dead branch and document. Test: chosen exit code, no write.
12. **F12 dead code and stale docs**: duplicate list readers, unreachable Supersede arm, unused CreateOptions, stale module comments, stale allow attributes. Correction: remove; document any reachable Supersede behaviour.
13. **F13 second slug table** (`registry.rs` document_handler) outside the seed. Correction: carry the legacy handler in the seed **or** add a drift test over every seed slug.
14. **F14 legacy `## Group` sections** cannot take new members. Required: an explicit recorded decision in the evidence (accept legacy `## Group` resolution **or** emit a named validation error) plus a fixture.
15. **F15 labels and references sentinel**: bracket sentinel and newline-joined string instead of FrontMatterValue::Sequence. Correction: use the sequence variant everywhere; remove sentinel join and split. Test: empty, single, multiple and bracketed round-trip.

## Invariants
- Behaviour changes only where a finding requires it; no new capability.
- Characterization corpus, CLI help goldens and fixtures change only through the documented capture path, with the delta recorded in EXPECTED-DELTAS.md.
- Tests added by this task must fail against the pre-correction code where the finding describes observable behaviour.
- No change to requirements, ADRs, technical specifications, or approved test cases; no new crates.

## Definition of done
Every finding F1-F15 has a recorded correction, a regression test, and a result; F14's decision and F13's handler treatment are recorded; the full suite is green; clippy with -D warnings and fmt are clean; the task comment maps findings to evidence.

## Acceptance Criteria
- [ ] F1: create-group runs the shared create pipeline -- resolve_field_stdin, ensure_required_fields with descriptor.field_defaults, and the shared create-op builder -- so stdin '-', required-field enforcement, frontmatter_key and field_defaults behave exactly as on create. Regression: create-group with '-' stdin resolves before ID allocation; a missing required group-role field reports every missing key with exit 4 and writes nothing; a group-role field with frontmatter_key is written; field_defaults are applied.
- [ ] F2: ID allocation reserves the ID atomically before writing (create_new or no-replace link) and retries the next number on AlreadyExists, so no existing record can be overwritten. Regression: at least 12 concurrent creates for one type (and 12 concurrent grouped-member creates) yield 12 distinct stored files and IDs with exit 0 each; an explicitly existing target is never replaced.
- [ ] F3: rendered front matter quotes every YAML-unsafe scalar (leading YAML indicators, document start and end markers, colon, hash, space and quote cases) and never coerces a scalar into a sequence; the rendered block is re-parsed before writing and a non-round-tripping render is a validation error with no write. Regression: a title and label round-trip table covering each indicator prefix; a bracketed title stays one string; view and list read back the exact input value.
- [ ] F4: HTML mockup rendering escapes every rendered field (title, task, specs, refs, superseded-by) and metadata is parsed only from the fixed comment block, so a title cannot set status on an unrelated edit. Regression: injection titles containing '-->', '<script>', and '-- status: approved' do not change status or other metadata on create or unrelated edit; an entity and angle-bracket title survives an unrelated edit without re-encoding; the parser ignores visible paragraph lines when the comment block exists.
- [ ] F5: --group references are normalized through resolve_in_dirs and the canonical ID is used, so create --group accepts the same forms as view and list (for example REQ-1, REQ-001, and a valid dotted member ID) and writes the canonical group ID in the reserved group key plus the canonical dotted member ID. Regression: every accepted reference form creates the same canonical member record.
- [ ] F6: edit resolves the role first (group vs member) and offers and accepts only that role's fields; an option declared only for the other role is rejected as one exit-4 validation outcome with no write. Regression: group edit rejects a member-only option and accepts a group-only option; member edit does the converse; help and runtime field sets agree for both roles.
- [ ] F7: config validation covers the derived command surface: reserved option names (including json, title, label, reference, editor, force, status, id), cross-role duplicate options, ui-mockup fields, and an empty derived option are rejected with exit 4 selector-path diagnostics instead of a panic or silent acceptance. Regression: a tree-build test constructs the full command tree and help for every seed descriptor and for an adversarial config, asserts no exit-101 panic, and asserts each invalid case is named.
- [ ] F8: the title sanitizer is title-specific and no longer applies path rules, so legal titles containing a path separator such as CI-CD with a forward slash or 'A backslash B' are accepted; empty, whitespace-only, multi-line, control-character and dual-source titles are still rejected with exit 4 and no write. Regression: a sanitizer case table covers the accepted and rejected titles, and create round-trips the accepted ones.
- [ ] F9: field and role resolution exists once and is shared by the document engine and registry descriptor derivation. Regression: a grouped type with different group.fields and member.fields yields identical role fields through help, create-group, create and edit, and the descriptor equivalence test stays green.
- [ ] F10: vacuous or ambiguous tests are replaced with concrete assertions: the handler assertion names the expected TypedHandler variant for every seed slug, each req error asserts one concrete cause, and the missing-title test asserts the exact message and that no record is written.
- [ ] F11: the archived-edit guard for config-defined types is reachable or removed with documentation: either archived records resolve for config-defined types so the guard returns its recorded outcome, or the dead branch is deleted and archived edits are documented as not found; a test asserts the chosen exit code and that no write occurs.
- [ ] F12: dead code and stale docs are removed -- duplicate list readers, the unreachable Supersede arm (or it is made reachable and documented), unused CreateOptions, stale module comments, and stale allow attributes; no unexplained #[allow] remains and clippy with -D warnings stays clean.
- [ ] F13: the second slug table is removed from the engine either by carrying the legacy TypedHandler in the seed table or by a drift test that asserts document_handler(slug) equals the seed-declared handler for every seed slug and fails on any divergence.
- [ ] F14: legacy '## Group' sections have a recorded decision in the evidence -- either accepted as a member-group resolution or rejected with a named validation error -- and a fixture covering a legacy-section member asserts the chosen outcome; neither behaviour may be silent.
- [ ] F15: labels, references, documentation, dependencies and assignee are written as FrontMatterValue::Sequence (create and edit), the bracket-sentinel joined string and line splitting are gone, and round-trip tests cover empty, single, multiple and bracketed values.
- [ ] Correction-only scope: no new features, no requirement, ADR, technical-specification or approved test-case edits, no new crates, and no dependency, ordinal, milestone or assignee changes outside this task. The full suite is green and cargo clippy --all-targets -- -D warnings and cargo fmt --check are clean.
- [ ] Evidence: the task comment maps every finding F1-F15 to its correction, files, regression test and result, and records the F14 decision; any characterization, golden or fixture change is updated through the documented capture path and recorded as a delta.
## Comments
- **rust-dev**: TASK-310 implemented all 15 AUD-141 findings. Branch task.TASK-310, commit ad82aa4. Tests 1287 passed, clippy -D warnings and fmt clean; no characterization or golden deltas.
- **rust-dev**: F1 create-group now runs resolve_field_stdin, ensure_required_fields with descriptor.field_defaults, and the shared create-op builder, so stdin '-', required group fields, frontmatter_key and defaults match create. Tests: document.rs create_group_enforces_required_fields_and_applies_shared_field_ops; tests.document_engine_corrections.rs f1_create_group_resolves_stdin_before_allocation.
- **rust-dev**: F2 identity.reserve_number and reserve_seq_number reserve the final path with create_new and retry on AlreadyExists; document, grouped member and HTML mockup creates use them. Tests: identity.rs reservation_never_replaces_an_existing_target; tests.document_engine_corrections.rs f2_concurrent_creates_produce_one_file_per_create.
- **rust-dev**: F3 format_scalar quotes indicator-leading, flow-punctuation, control and non-string-parsing scalars; scalar keys are never coerced to sequences; patch re-parses and verifies every typed value before writing. Tests: frontmatter.rs yaml_indicator_scalars_round_trip_as_strings, scalar_never_coerces_to_a_sequence, sequences_keep_bracketed_and_empty_items, verify_value_rejects_a_non_round_tripping_render; tests.document_engine_corrections.rs f3_yaml_unsafe_titles_round_trip.
- **rust-dev**: F4 every mockup field is HTML or comment escaped, metadata is parsed only from the fixed comment block, and the raw encoded title is preserved on unrelated edits. Tests: ui_mockup.rs injection_titles_do_not_change_metadata, visible_paragraphs_are_not_metadata_sources, unrelated_render_preserves_the_raw_encoded_title; tests.document_engine_corrections.rs f4_mockup_title_cannot_inject_metadata, f4_encoded_title_survives_an_unrelated_edit.
- **rust-dev**: F5 resolve_group uses resolve_in_dirs, accepts every view and list reference form including dotted members, and writes the canonical group ID. Test: tests.document_engine_corrections.rs f5_group_references_are_canonicalized.
- **rust-dev**: F6 edit resolves the canonical role first, uses only that role's fields, and rejects other-role options as one exit-4 validation outcome with no write. Test: tests.document_engine_corrections.rs f6_edit_is_role_scoped_and_help_matches_the_role_fields.
- **rust-dev**: F7 config validation and registry build reject reserved option names, cross-role duplicate options, HTML mockup fields, and empty derived options with selector paths; every seed descriptor builds its command and help without panic. Tests: config.rs derived_option_names_are_validated_before_a_command_is_built; registry.rs build_rejects_invalid_derived_options_instead_of_panicking; tests.document_engine_corrections.rs f7_every_seed_descriptor_builds_its_command_and_help.
- **rust-dev**: F8 sanitize_title_text allows slash and backslash but rejects empty, whitespace, multiline, NUL and control titles; fields keep the stricter name rules. Tests: sanitize.rs title_tests and the updated document CLI sanitization tests.
- **rust-dev**: F9 registry.role_fields and registry.fields_for_verb are the only resolver, used by help, create-group, create and edit. Test: tests.document_engine_corrections.rs f6_edit_is_role_scoped_and_help_matches_the_role_fields; registry descriptor tests stay green.
- **rust-dev**: F10 handler assertions name the concrete TypedHandler per seed slug, req errors assert one concrete cause and no write, and missing-title asserts the exact message and no record. Tests: registry.rs default_document_set_has_the_eleven_typed_slugs; tests.req_cli.rs req_errors; tests.ui_spec_cli.rs ui_spec_create_requires_all_arguments.
- **rust-dev**: F11 all document types resolve the archived folder, so the config-defined archived-edit guard returns exit 4 and writes nothing. Test: tests.document_engine_corrections.rs f11_archived_config_defined_edit_is_rejected_without_a_write.
- **rust-dev**: F12 removed the duplicate list reader, list_ops_value, the second slug handler table, unused CreateOptions and GroupRequirement, stale transitional comments, and stale too_many_arguments allows; the Supersede arm is documented as unreachable for Markdown because the HTML mockup is routed earlier. Clippy and fmt are clean.
- **rust-dev**: F13 SeedDocumentType now carries handler, TypeDescriptor.handler reads the seed, and the second slug table is gone; the concrete handler map test covers every seed slug.
- **rust-dev**: F14 decision: legacy ## Group sections are accepted, not rejected. Legacy groups and members stay resolvable and new members can be created under a legacy group; new members still write the reserved front-matter group key. Fixture tests.fixtures.legacy-group; test tests.document_engine_corrections.rs f14_legacy_group_section_accepts_new_members.
- **rust-dev**: F15 labels, references, documentation, dependencies and assignee are FrontMatterValue.Sequence in document, task, milestone, release, decision, edit, migrate and import paths; sentinel joins and splits are gone. Tests: tests.document_engine_corrections.rs f15_labels_and_references_are_sequences and f15_backlog_list_fields_are_sequences; frontmatter.rs sequences_keep_bracketed_and_empty_items.
- **rust-dev**: Status left In-Progress.
- **orchestrator**: Acceptance met: F1 to F15 corrected with regression tests (atomic ID reservation, YAML-safe scalars with pre-write parse, escaped mockup metadata, shared create-group pipeline, canonical group references, role-scoped edits, config surface validation, title sanitizer, shared resolver, de-vacuous tests, archived resolution, cleanup, seed-carried handler, legacy group accepted, sequence API for labels and references); 1287 tests pass.
- **orchestrator**: Reopened for the AUD-142 residuals: B1 reserved clap help option still panics; B2 sequence API missing on task, milestone, release and decision edit paths.
- **rust-dev**: B1 F7 fixed: the reserved option set now includes the clap auto-generated help flag, so config validation and registry validate_derived_surface reject it with the exit-4 document_types.<slug>.<path> selector diagnostic instead of panicking at command build. Tests: config.rs derived_option_names_are_validated_before_a_command_is_built; registry.rs build_rejects_invalid_derived_options_instead_of_panicking; tests.document_engine_corrections.rs f7_reserved_help_option_rejects_every_document_command, which builds every seed descriptor from an adversarial config declaring help as a derived option and asserts exit 4 for both the verb and its help flag with no panic.
- **rust-dev**: B2 F15 fixed: task, milestone, release and decision edit now write labels, references, documentation, dependencies and assignee as FrontMatterValue.Sequence, and the bracketed sentinel interpretation is removed from frontmatter.rs format_kv. Tests: tests.document_engine_corrections.rs f15_backlog_edit_list_fields_are_sequences covers all four edit paths with bracketed, single, multiple and empty values. Full suite 1289 passed, 0 failed, 26 ignored; clippy -D warnings and fmt clean; no characterization or golden deltas.
- **orchestrator**: B1 and B2 closed (AUD-143): help reserved in both validators with an adversarial regression, and all four backlog edit paths use FrontMatterValue Sequence with the sentinel removed; 1289 tests pass.
