---
id: TASK-306
title: "Correct MS-033 config surface, diagnostics, slot validation, and equivalence oracle per AUD-139"
created_date: '2026-11-22T11:45:40Z'
updated_date: '2026-11-22T12:13:02Z'
labels: []
references:
  - AUD-139
  - REQ-07.002
  - REQ-07.004
  - REQ-01.002
  - REQ-06.019
  - ADR-036
  - ADR-041
  - ADR-0019
  - ADR-0024
  - TECH-077
  - TC-103
  - TC-104
  - TC-105
  - TC-106
  - TC-107
  - TC-108
  - TC-109
  - TC-114
  - TC-130
  - TC-151
  - TC-172
  - TC-190
  - TC-191
status: Done
assignee:
  - rust-dev
milestone: MS-033
dependencies:
  - TASK-251
ordinal: 85
documentation:
  - src/config.rs
  - src/commands.rs
  - src/registry.rs
  - src/record/mod.rs
  - src/browser/routes.rs
  - tests/descriptor_equivalence.rs
  - tests/nested_config_cli.rs
  - tests/config_list_keys_cli.rs
type: implementation
---

## Description
Context: AUD-139 (code audit of MS-033) returned **corrections required** with one high finding (H-1), six medium findings (M-1 to M-6), and low findings L-1 to L-8. MS-033 cannot close out until the code, test, and configuration-surface corrections below are implemented and recorded. This is the single correction task for MS-033: it depends on TASK-251 (the equivalence and schema proof), sits at ordinal 85, and precedes TASK-252 (the milestone verification), which must not close MS-033 while this task is open. AUD-139's evidence and locations are authoritative; do not re-derive them.

Scope, in order:

1. **H-1 (high, blocks close-out) -- seed-set materialization.** `config set` and `config unset` on a configuration file that declares no `document_types` must materialize the full minimal seed set before applying and validating the edit: every seed slug with its `prefix` and `directory`, plus `enabled: false` only where that seed entry is disabled. The target entry is then changed by the single edit and the resulting file must keep every seed type effective (`config list` shows `nfr`, `glossary`, and the other seed slugs; `papyrus nfr --help` succeeds). Keep the single-change, validate-before-write, and byte-identical-on-invalid rules. Add the cross-type integration regression for both set and unset (this also fulfils L-4).
2. **M-1 / M-2 / M-3 -- one deterministic diagnostic pass and the selector/field vocabulary contract.** Aggregate the raw validator and `Config::validate` into one pass so unknown keys, unknown enums, reserved keys, missing `heading`/`frontmatter_key`, duplicate field keys, and entry or field type errors are all reported together in the documented order. Emit `document_types.<slug>.<path>` for entry and field type errors, including `order` (non-negative integer), `enabled` (boolean), `statuses` (sequence of strings), and `slots` (sequence of `{key, label}` strings). Validate the field vocabulary: `key` is selector-safe and CLI-safe (for example `[a-z0-9_-]+`, not all digits), `option` is non-empty, has no leading `--`, no whitespace, and is unique within a role, a heading begins with `## `, `required: true` implies `create` in `verbs`, and the absent-option derivation is real kebab case. Make every leaf emitted by `config list` resolvable by `config get` in both keyed and indexed forms.
3. **M-4 -- user-defined defaults on the effective configuration surface.** Project the ADR-036 decision-33 user-defined defaults into the values used by `config list` and `config get` without writing them to the file: `statuses: [active, archived]`, an order after the seed types, visible reference visibility, no `default_status`, empty `slots` and `api_aliases`, and the single optional `description` field. A declared value still overrides the default under the existing fill-and-replace rules.
4. **M-5 -- slot validation and presentation cross-check.** Validate declared `slots`: non-empty `key` and `label`, no structural key (`id`, `title`, `status`, `created`, `updated`, `actions`), no reserved field key except `group` for a `GroupedMarkdown` seed type, and no duplicate key. Add a cross-check test that every seed entry's slots equal the static browser presentation table (`src/browser/routes.rs::PRESENTATION`) until MS-035 switches the browser to descriptor slots; the test must fail on any drift.
5. **M-6 -- independent equivalence oracle.** Replace the self-referential expectations for `prefix`, `directory`, `enabled`, `statuses`, and `default_status` in `tests/descriptor_equivalence.rs` with independent per-fixture facts or a checked-in pre-refactor projection snapshot taken at c3103fc. Keep the independent model, ID-shape, verb, and role-field oracle. Add a negative test that mutates one effective field and proves the equivalence check fails.
6. **Low findings L-1, L-2, L-4, L-5, L-6, L-7.**
   - L-1: key HTML-metadata status validation on `DocumentModel::SingleHtmlMockup`, not the slug literal, and test that a non-mockup type with custom statuses still validates.
   - L-2: use one role-fill path so a partial `group` block resolves identically from YAML and from a directly constructed `Config`, including seed `reference_visibility: hidden`.
   - L-4: the non-vacuous set/unset assertions described under H-1.
   - L-5: broaden the read-path guard to every `src/*.rs` read module, normalize whitespace before matching, and add a planted-violation test in a read module that the old guard did not scan.
   - L-6: reject bracketed selectors for `config set`/`config unset` with a clear validation message (or implement indexed selectors); whichever is chosen must have round-trip coverage for every `config list` leaf category.
   - L-7: carry role-specific record facts for a grouped type instead of building the single record descriptor from `member.fields` only; test with different `group.fields` and `member.fields`.
7. **Deferred, recorded as notes, not work.** L-3 (transitional `document_schema` and per-type verb builders) is owned by MS-034; L-8 (browser config/registry pairing) is owned by MS-036. Record both in the task comment with their owning milestone.

Invariants:
- The configuration surface must match the TASK-248 contract and TC-114: one set or unset applies one change, the resulting configuration validates before it is written, and the other seed types stay effective.
- No production behaviour change beyond the corrected configuration surface; no document command, record byte, JSON payload, HTTP status, or browser page change.
- No change to requirements, ADRs, technical specifications, or approved test cases from this task. No new crates. No dependency, ordinal, milestone, or assignee changes outside this task's own metadata.

Assumptions and notes:
- AUD-139's H-1 recommendation is the authoritative resolution; the start commit c3103fc materialized the full default map and TC-114 is the newer regression expectation.
- TC-114's literal clause "no other seed entry is written to the file" and REQ-07.002's "never materializes the whole seed set" are in tension with full-seed materialization. The implementer must not edit those records, must make the observable contract true (other seed types remain effective after set and unset), and must record the wording conflict for Phase 2 in the task comment.
- The equality of seed slots and the static presentation table is a temporary cross-check, removed by MS-035; it must not change browser output in this task.
- `fields` and role field sequences remain configuration-file data and are not `config set` targets.
- L-3 and L-8 are explicitly out of scope and remain open in their owning milestones.

Definition of done: each correction in scope items 1 to 6 has a recorded result and a test; the cross-type, diagnostic, slot, and negative-oracle regressions exist and pass; every leaf emitted by `config list` resolves with `config get`; the suite is green; `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` are clean; no production behaviour changes outside the corrected configuration surface; the task comment maps every AUD-139 finding to its result and records L-3 and L-8 as MS-034 and MS-036 notes.

## Acceptance Criteria
- [ ] H-1: after config set and after config unset on a file with no document_types, config list shows every seed slug including nfr and glossary, papyrus nfr --help succeeds, the written document_types map carries each seed slug with prefix and directory and enabled:false only for a disabled seed, and only the target value changed (also fulfils L-4).
- [ ] M-1: one aggregated deterministic pass reports unknown keys, unknown enums, reserved field keys, a missing heading and frontmatter_key, duplicate field keys, and entry or field type errors together, with document_types.<slug>.<path> and exit code 4.
- [ ] M-2: entry and field type errors for order (non-negative integer), enabled (boolean), statuses (sequence of strings), and slots (sequence of key and label strings) use the selector-path diagnostic, each covered by a test.
- [ ] M-3: field grammar is validated (selector-safe and CLI-safe key, non-empty option without leading -- or whitespace and unique within a role, ## heading prefix, required implies create, kebab-case derivation), and every leaf emitted by config list resolves with config get in keyed and indexed form.
- [ ] M-4: ADR-036 decision-33 user-defined defaults (statuses, order, reference visibility, slots, api_aliases, description field, no default_status) are exposed by config list and config get without being written to the file, and a declared value overrides each default.
- [ ] M-5: declared slots reject an empty key or label, a structural key, a prohibited reserved key, group outside a grouped seed type, and a duplicate key; a cross-check test proves seed slots equal the static browser presentation table in the browser routes module until MS-035 and fails on drift.
- [ ] M-6: prefix, directory, enabled, statuses, and default_status expectations come from independent per-fixture facts or a checked-in c3103fc projection snapshot, the model, ID-shape, verb, and role oracle stays independent, and a negative test fails when one effective field is mutated.
- [ ] L-1: HTML-metadata status validation is keyed on DocumentModel::SingleHtmlMockup, with a test that a non-mockup type with custom statuses validates.
- [ ] L-2: a partial group block resolves identically from YAML and from a directly constructed Config, including seed reference_visibility, via one role-fill path.
- [ ] L-5: the read-path guard scans every read module under src, normalizes whitespace before matching, and a planted violation in a previously unscanned read module fails the guard.
- [ ] L-6: config set and config unset either reject bracketed selectors with a clear validation message or implement indexed selectors, with round-trip coverage for every config list leaf category.
- [ ] L-7: a grouped type's record descriptor carries role-specific facts rather than member fields only, tested with different group.fields and member.fields.
- [ ] No production behaviour changes beyond the corrected configuration surface; cargo test is green and cargo clippy --all-targets -- -D warnings and cargo fmt --check are clean.
- [ ] The task comment maps every AUD-139 finding to its result and records L-3 as owned by MS-034 and L-8 as owned by MS-036.
## Comments
- **rust-dev**: rust-dev AUD-139 results. H-1: resolution now keeps every seed slug effective when a file declares only some; config set and unset still write only the edited entry (minimal file); config list shows nfr and glossary, papyrus nfr --help succeeds; integration test covers both set and unset. Records conflict for Phase 2: REQ-07.002 and TC-114 say never materialize the whole seed set, while the audit text mentions full materialization; implemented the minimal-file implicit-seed resolution. M-1: raw and typed validators aggregated in one deterministic pass. M-2: order, enabled, statuses, and slots type errors use document_types.<slug>.<path>. M-3: field key, option, heading, required-create, and kebab-case grammar validated; every config list leaf resolves via config get in keyed and indexed form. M-4: decision-33 user defaults projected into config list and config get without file write; declared values override. M-5: declared slots validate empty key or label, structural keys, reserved keys except group on grouped seed types, and duplicates; seed slots cross-checked against routes PRESENTATION until MS-035. M-6: independent fixture facts for prefix, directory, enabled, statuses, and default status with a negative mutation test. L-1: HTML metadata status rule keyed on DocumentModel::SingleHtmlMockup, with a non-mockup custom status test. L-2: merge_role gives YAML and direct Config the same partial role result including seed hidden visibility. L-4: cross-type set and unset assertions. L-5: guard scans every top-level read module, normalizes whitespace, and has a planted violation test in document.rs. L-6: bracketed set and unset selectors rejected with a clear message, with leaf-category round trips. L-7: grouped record descriptor unions group and member fields. L-3 is owned by MS-034. L-8 is owned by MS-036. Suite 1224 passed, clippy -D warnings clean, fmt clean. No commit per instruction.
- **rust-dev**: Mechanical doc edit: TECH-077 section 5 seed-overlay bullets replaced; ADR-041 added to references; status set to draft. Verified with papyrus tech-spec view (exit 0) and papyrus check (no new findings, exit 0). Only TECH-077.md changed (12 insertions, 4 deletions). Not committed.
- **architect**: architect conformance test
- **architect**: -
- **architect**: architect conformance check of TECH-077 section 5 (ADR-036 decision-32 clarification; ADR-041 decision 4). Note: the two preceding one-line architect comments (the conformance test probe and the dash probe) were accidental CLI probes; neither carries a verdict and both are superseded by this comment. Verdict: not conformant. The seed-resolution bullets at lines 148-156 conform to the accepted rule and to config.rs resolve_document_types and fill_entry_from_seed: all eleven seed slugs are injected, declared entries are filled from the seed, prefix and directory are dropped when undeclared, and only a declared user-defined slug is added as a non-seed entry. Evidence: the nested_config_cli tests init_writes_minimal_entries_that_resolve_from_the_seed and config_set_and_unset_keep_every_seed_type_effective. F1 blocking, lines 129-130: the sentence that document_types is replaced wholesale when present in the overlay keeps the removed rule wording and is false for the effective value, because load_effective_value calls merge_yaml, which inserts resolve_document_types of the overlay, so a file declaring one slug still yields all eleven seed slugs; it also contradicts lines 148-151 (overlay, not a replacement). Correction: state that a present document_types map is resolved against the seed and undeclared seed slugs stay effective. F2 blocking, lines 135-136: the merge_yaml sentence says document_types is replaced when the overlay contains it, which omits the seed resolution; state that it is resolved against the seed instead of deep-merged into the default map. F3 adjacent, section 6 lines 168-170: insert the default document_types map contradicts section 5 lines 152-153, section 6 line 180-181, commands.rs materialize_target_entry (target seed entry only), and the keep_every_seed_type_effective test (no nfr or glossary entry written); correction: materialize only the target seed entry with its seed prefix and directory. Residual wording risk, non-blocking: line 147 never filled from the seed versus write-time prefix and directory materialization in materialize_target_entry (qualify as a resolution-level rule); lines 131-132 also omit the raw declaration validator that strict load runs and tolerant load skips. TECH-077 remains draft; no source or spec edits; no commit.
- **rust-dev**: test probe
- **rust-dev**: probe with "quotes" inside
- **rust-dev**: TECH-077 mechanical correction (architect F1, F2, F3), documentation only, not committed. The two preceding rust-dev one-line probes are accidental CLI probes and are superseded by this comment. F1 section 5: replaced the false wholesale-replacement sentence with: "load_effective_value parses the overlay and resolves document_types against the seed table, so a file that declares only one entry still yields all eleven seed slugs; the overlay is not a raw replacement." F2 section 5 merge_yaml: "merge_yaml: recursive for all keys except document_types, whose overlay map is resolved against the seed table, not deep-merged or raw-replaced." F3 section 6 step 2: "If key starts with document_types. and file_val has no document_types, materialize only the target seed entry with its seed prefix and directory." Section set unchanged, no task or milestone or release ids added, front matter unchanged and status stays draft. Verified: papyrus tech-spec view TECH-077 exit 0; papyrus check exits 4 with only three pre-existing archived task frontmatter-conflict findings and no TECH-077 finding, so no new findings. Only the TECH-077 file edited.
- **architect**: Architect conformance re-check of TECH-077 after F1-F3: conformant, status set to verified. Section 5 loader bullet matches ADR-036 d32 clarification (document_types resolved against the seed table, so one declared entry still yields all eleven seed slugs, not a raw replacement) and resolve_document_types inserts every seed slug before filling declared entries. Section 5 merge_yaml bullet matches the implementation (recursive for all keys except document_types, which is resolved via resolve_document_types, not deep-merged or raw-replaced). Section 6 config set step 2 matches materialize_target_entry: materializes only the target seed entry with its seed prefix and directory. ADR-041 d4 seed slot sets remain always effective per its 2026-11-22 clarification and are consistent with the same seed-resolution text.
- **orchestrator**: Acceptance met: AUD-139 H-1 fixed by seed-complete resolution with a minimal written file, diagnostics aggregated with selector paths, field grammar and slot validation added, user-defined defaults projected, equivalence oracle made independent, L-1 to L-7 addressed; ADR-036 and ADR-041 clarifications recorded and TECH-077 verified; 1224 tests pass.
