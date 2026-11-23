---
id: TASK-251
title: "Prove old configurations load identically and cover the schema with unit tests"
created_date: '2026-11-20T17:41:37Z'
updated_date: '2026-11-22T11:26:57Z'
labels: []
references:
  - REQ-007.002
  - REQ-007.004
  - TECH-077
  - ADR-036
  - TC-102
  - TC-103
  - TC-104
  - TC-105
  - TC-106
  - TC-107
  - TC-108
  - TC-109
  - TC-130
  - TC-151
  - TC-183
  - TC-184
  - TC-185
  - TC-190
  - TC-191
status: Done
assignee:
  - rust-dev
milestone: MS-033
dependencies:
  - TASK-250
ordinal: 80
documentation:
  - tests/descriptor_equivalence.rs
  - tests/fixtures/characterization/corpus
  - tests/golden/cli-help
  - tests/fixtures/cli-help
  - papyrus.config.yml
type: implementation
---

## Description
Context: MS-033 is complete only when existing projects load without edits and the new rules are regression-protected. The pre-refactor evidence is already committed at the MS-033 start commit c3103fc and must be used as the oracle rather than re-derived by hand:
- `tests/golden/cli-help/` freezes the root type order, each enabled type's verb list, and each `create` and `edit` long-option list and order for `tests/fixtures/cli-help/papyrus.config.yml` (the config with the seed types, a disabled `legacy` entry, and the user-defined `note`).
- `tests/fixtures/characterization/corpus/` freezes record bytes and rendered output for `req`, `nfr`, `rule`, `ui-mockup`, `audit`, `tech-spec`, `ui-spec`, and the user-defined `note`; `tests/fixtures/characterization/lists/` freezes the list pages.
- The live `papyrus.config.yml` declares the user-defined types `request` and `release-reports` with their own statuses and default status.
- ADR-036 decision 29 is the authoritative seed field table, and `src/registry.rs` `DOCUMENT_TEMPLATES` plus the nine typed modules are the pre-refactor field inventory.

Invariant (old-configuration load identity): every configuration that loaded before MS-033 must still load with no edit and produce the same effective document-type descriptor projection, defined as, per slug: prefix, directory, enabled flag, statuses, default status, document model, ID shape, verb set, and for each role the ordered list of `(heading, frontmatter_key, option, kind, required, verbs)`. The projection must equal the pre-refactor facts: for a seed slug the ADR-036 decision 29 field table and the current `create`/`edit` long options in `tests/golden/cli-help/`; for a user-defined slug the current `## Description` body section and no type-specific front-matter key. The descriptor gains new data (presentation scalars, `FieldSpec` values, `DocumentModel`), but no projected fact may change. The single permitted projected difference is the reference-visibility default of a user-defined type, which moves from the former `ContentModel::Generic` hidden rule to `Visible` per ADR-036 decision 33 and REQ-006.019; it is the documented expected delta in TC-185 and no MS-033 consumer reads it until MS-035.

Scope:
1. Equivalence tests: load `papyrus.config.yml`, `tests/fixtures/cli-help/papyrus.config.yml`, `tests/fixtures/characterization/corpus/project/papyrus.config.yml`, `tests/fixtures/characterization/corpus/project-empty/papyrus.config.yml`, and both `tests/fixtures/characterization/lists/*/papyrus.config.yml` files without editing them, build the registry, and assert the projection above. Assert the config files and the corpus record files are byte-identical after the load. For the cli-help config, assert the projected verbs and long options against `tests/golden/cli-help/{top,slug-*,verb-*}.json`.
2. Unit tests in `src/config.rs`, `src/registry.rs`, and `src/record/descriptor.rs` `#[cfg(test)]` modules for: default fill and collection replace including `fields: []`; `group` and `member` blocks resolving independently; an absent slug staying absent; `prefix` and `directory` never filled (missing value rejected even for a seed slug); unknown key and unknown enum diagnostics in the `document_types.<slug>.<path>` form with a deterministic multi-error order; `fields: []`; duplicate field keys per role and the same key allowed across roles; every reserved key (`id`, `title`, `status`, `labels`, `references`, `created_date`, `updated_date`, `group`) rejected as a field `key` and as a `frontmatter_key`; a field with neither `heading` nor `frontmatter_key`; a field-level `order` or `enabled`; an entry-level `model`, `title`, `content`, or `id_shape`.
3. Unit tests for the seed-only models: `req` and `nfr` are `GroupedMarkdown`, `ui-mockup` is `SingleHtmlMockup`, every other seed slug is `SingleMarkdown`, and a user-defined slug is `SingleMarkdown` even when it reuses a seed prefix, directory, or field name; `group.fields` and `member.fields` exist only on `GroupedMarkdown` and a grouped entry with a top-level `fields` is rejected.
4. Unit tests for user-defined defaults: label from the slug, order after every seed type, empty summary and aliases, visible reference visibility, `[active, archived]` statuses, no default status, and one optional `multiline` `description` field with heading `## Description`; declared values replace each default under the same fill and replace rules.
5. Unit tests for immutable group membership at the descriptor level: `group` is not a field and never has an option in any role, no `edit` option list contains a group option, `create-group` and the required `--group` exist only for `GroupedMarkdown`, and `archive` and `delete` expose no cascade option; the existing behaviour suite (TC-151) stays green.
6. Unit tests for the emitter: a quoted `"## User Story"` heading round-trips, a mapping sequence renders as a block sequence, a flat string list stays inline, `is_list_valued_key` returns false for `fields` and the role sequences, and `to_config_yaml` output reparses to an equal value for every fixture.
7. A read-path guard: assert no source file under `src/record/` calls `load_effective_config`, `load_effective_value`, `ArtifactRegistry::build`, or `load_cfg_for`, plus the runtime proof from TASK-250 that a configuration-defined record still resolves after the configuration file is removed between the registry build and the read (MS-033 AC-5).
8. Where the tests live: unit tests in the owning module's `#[cfg(test)]`; cross-config equivalence in a new `tests/descriptor_equivalence.rs`; emitter and selector cases may extend `tests/config_list_keys_cli.rs` and `tests/cli.rs`. Any expected projection table is test data derived from ADR-036 decision 29 and the frozen goldens, not a production table.

Invariants: the projection equality above; no CLI or browser behaviour changes in this milestone (MS-033 AC-7); the tests must fail if a seed heading, option, kind, required flag, field verb, status set, or user-defined default drifts.

Out of scope: fixing a failure by changing a requirement, an ADR, a golden, a fixture, or source outside the model and configuration layer; adding a CLI or browser assertion for MS-034 or MS-035 behaviour. No dependency, ordinal, milestone, or assignee changes.

Assumptions:
- The pre-refactor long options in `tests/golden/cli-help/` are the option oracle; short options (`-d`) are per-type argument metadata until MS-034 and are not part of the field projection.
- A seed slug's projection is compared to ADR-036 decision 29 and the goldens, not to a runtime dump of the old registry, because no descriptor-dump command exists.
- `tests/fixtures/characterization/lists/` is pinned and must not be modified; only its config is read.

Definition of done: every listed unit test exists and passes; the five fixture configurations plus the live configuration load unedited and project identically; the only projected difference is the documented user-defined reference-visibility default; the read-path guard passes; `cargo test` and `cargo clippy --all-targets -- -D warnings` are clean.
## Acceptance Criteria
- [ ] Old-configuration load identity holds: the live `papyrus.config.yml`, the cli-help fixture configuration, the two corpus fixture configurations, and the two list fixture configurations load with no edit and project to the same prefix, directory, enabled flag, statuses, default status, model, ID shape, verb set, and per-role `(heading, frontmatter_key, option, kind, required, verbs)` as the pre-refactor facts, verified by a descriptor equivalence integration test (MS-033 AC-1, REQ-007.002).
- [ ] The only permitted projected difference is the user-defined reference-visibility default (former `ContentModel::Generic` hidden to `Visible`), which is the documented expected delta in REQ-006.019 and TC-185; the test names it explicitly and fails on any other difference.
- [ ] The fixture configuration and corpus record files are byte-identical after the load, and the cli-help projection matches the cli-help top, slug, and verb goldens for type order, verbs, and long options (MS-033 AC-1, AC-7).
- [ ] Unit tests cover default fill and collection replace including `fields: []`, independent `group` and `member` resolution, an absent slug staying absent, and `prefix` and `directory` never filled even for a seed slug (MS-033 AC-2, TC-109).
- [ ] Unit tests cover every reserved key (`id`, `title`, `status`, `labels`, `references`, `created_date`, `updated_date`, `group`) rejected as a field `key` and as a `frontmatter_key`, plus a missing `heading` and `frontmatter_key` (MS-033 AC-3, ADR-036 decision 23, TC-105).
- [ ] Unit tests cover a duplicate field key within a role being rejected and the same key being accepted across `group.fields` and `member.fields` (ADR-036 decision 22, TC-106).
- [ ] Unit tests cover every unknown key and unknown enum diagnostic in the `document_types.<slug>.<path>` form, in one deterministic multi-error pass with exit code 4 (MS-033 AC-3, TC-103, TC-104, TC-130).
- [ ] Unit tests cover the seed-only models: `req` and `nfr` are `GroupedMarkdown`, `ui-mockup` is `SingleHtmlMockup`, every other seed slug is `SingleMarkdown`, a user-defined slug is `SingleMarkdown` even when it reuses a seed prefix, directory, or field name, and a grouped entry with a top-level `fields` is rejected (MS-033 AC-6, ADR-036 decisions 26 and 31, TC-108).
- [ ] Unit tests cover user-defined defaults: slug-derived label, order after every seed type, empty summary and aliases, visible reference visibility, `[active, archived]` statuses, no default status, and one optional multiline `description` field with heading `## Description`, each overridable by a declared value (MS-033 AC-6, ADR-036 decision 33, TC-102).
- [ ] Unit tests cover immutable group membership at the descriptor level: `group` is never a field and never has an option in any role, no `edit` option list contains a group option, `create-group` and the required `--group` exist only for `GroupedMarkdown`, and `archive` and `delete` expose no cascade (MS-033 AC-6, ADR-036 decision 9, TC-151).
- [ ] Unit tests cover the emitter: a quoted `"## User Story"` heading round-trips, mapping sequences render as block sequences, flat string lists stay inline, `is_list_valued_key` is false for the field sequences, and every fixture re-emits to a value that reparses equal (MS-033 AC-4, ADR-0024).
- [ ] A source guard and a runtime test together prove no read path builds a configuration or registry per record: no configuration load or registry build in the record module, and a configuration-defined record still resolves after the configuration file is removed between the registry build and the read (MS-033 AC-5).
- [ ] `cargo test` reports zero failures and `cargo clippy --all-targets -- -D warnings` is clean.
## Comments
- **task-refiner**: Refined for MS-033: description grounds the old-configuration load-identity invariant in the MS-033 start commit c3103fc oracles (cli-help goldens, corpus, lists fixtures, live config) and ADR-036 decision 29; 13 testable acceptance criteria added, including the reserved-key, seed-only-model, user-defined-default and immutable-membership unit tests; documentation paths set. No dependency, ordinal, milestone, or assignee change.
- **rust-dev**: probe
- **rust-dev**: rust-dev evidence: descriptor equivalence covers all six repo configurations unedited; cli-help goldens match; corpus bytes unchanged; only delta is user-defined reference visibility Hidden to Visible (TC-185).
- **rust-dev**: rust-dev evidence: config and registry unit tests cover every reserved key as key and frontmatter_key, missing heading and frontmatter_key, duplicate keys within a role versus across group and member, independent role fill, absent slug, prefix and directory never filled, and collection replace including fields: [].
- **rust-dev**: rust-dev evidence: unit tests also cover grouped top-level fields rejection, role-block and deterministic multi-error diagnostics, user-defined defaults and overrides, seed-only models including user-defined reuse of seed facts, immutable group membership with required --group, emitter fixture round-trips and field-sequence list classification.
- **rust-dev**: rust-dev evidence: unit tests cover every reserved key as key and as frontmatter_key, a missing heading and frontmatter_key, and duplicate keys within a role versus the same key across group and member.
- **rust-dev**: rust-dev evidence: unit tests cover independent group and member fill, an absent slug staying absent, prefix and directory never filled, collection replacement including fields: [], and grouped top-level fields rejection.
- **rust-dev**: rust-dev evidence: unit tests cover role-block and deterministic multi-error diagnostics, user-defined defaults and declared overrides, and seed-only models including user-defined reuse of seed facts.
- **rust-dev**: rust-dev evidence: unit tests cover immutable group membership with required --group, emitter fixture round-trips and field-sequence list classification. Read-path guard and runtime proof stay green.
- **rust-dev**: rust-dev evidence: unit tests cover every reserved field key in both key and frontmatter_key positions.
- **rust-dev**: rust-dev evidence: unit tests cover duplicate keys within a role and the same key across group and member roles.
- **rust-dev**: rust-dev evidence: unit tests cover a field with no heading and no frontmatter_key.
- **rust-dev**: rust-dev evidence: unit tests cover independent group and member role fill and an absent document type slug staying absent.
- **rust-dev**: rust-dev evidence: unit tests cover prefix and directory never filled from the seed, even for a seed slug.
- **rust-dev**: rust-dev evidence: unit tests cover collection replacement including fields: [] and grouped top-level fields rejection.
- **rust-dev**: rust-dev evidence: unit tests cover role-block diagnostics and deterministic multi-error ordering.
- **rust-dev**: rust-dev evidence: unit tests cover user-defined defaults and declared overrides.
- **rust-dev**: rust-dev evidence: unit tests cover seed-only models including a user-defined slug reusing seed facts.
- **rust-dev**: rust-dev evidence: unit tests cover immutable group membership and required --group only for GroupedMarkdown.
- **rust-dev**: rust-dev evidence: unit tests cover emitter fixture round-trips and field-sequence list classification.
- **rust-dev**: rust-dev evidence: read-path guard and runtime proof stay green.
- **rust-dev**: rust-dev evidence: 66 test suites ok with 0 failures; clippy and fmt checks clean; no production behaviour change; no new crates.
- **rust-dev**: rust-dev evidence: branch is the TASK-251 task branch; changes were not committed per the instruction.
- **orchestrator**: Acceptance met: six repository configurations load unedited and project identically across task, milestone, release, decision and all 11 seed document types; only the documented user-defined visibility delta; additively 5 equivalence tests and 12 schema unit tests; suite green, clippy and fmt clean.
