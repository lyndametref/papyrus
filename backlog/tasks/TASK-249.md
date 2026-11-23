---
id: TASK-249
title: "Add the seed table, derivation, and selector-path resolution"
created_date: '2026-11-20T17:41:37Z'
updated_date: '2026-11-22T10:51:17Z'
labels: []
references:
  - ADR-036
  - REQ-005.001
  - REQ-005.003
  - REQ-005.005
  - REQ-005.006
  - REQ-005.008
  - REQ-007.004
  - TECH-071
  - TC-102
  - TC-119
  - TC-120
  - TC-121
  - TC-122
  - TC-123
  - TC-128
  - TC-172
  - TC-191
status: Done
assignee:
  - rust-dev
milestone: MS-033
dependencies:
  - TASK-248
  - TASK-293
ordinal: 60
documentation:
  - src/config.rs
  - src/registry.rs
  - docs/specifications/technical/TECH-070.md
  - docs/specifications/technical/TECH-077.md
type: implementation
---

## Description
Context: MS-033 start commit c3103fc. The seed is the only definition of built-in document types after the nine typed modules are deleted (ADR-036 decision 28). The current static sources that must be replaced by one table:
- `src/config.rs:101 DEFAULT_DOCUMENT_TYPES` (slug, prefix, directory, has-status) and `Config::default` (line 146).
- `src/registry.rs:2177 DOCUMENT_TEMPLATES`, including `RecordFacts`, `frontmatter_status`, `frontmatter_extra`, `TemplateStatus`, `TemplateVerbs`, and the typed schemas `REQUIREMENT_BUSINESS_SCHEMA`, `GLOSSARY_SCHEMA`, `RULE_SCHEMA`, `UI_SPEC_SCHEMA`, `AUDIT_SCHEMA`, `TECH_SPEC_SCHEMA`, `USE_CASE_SCHEMA`, `TEST_CASE_SCHEMA`, `TEST_REPORT_SCHEMA`, `UI_MOCKUP_SCHEMA`.
- `src/browser/routes.rs:149 PRESENTATION`, which today supplies the label, heading, short noun, inventory note, order, legacy `?type=` key, API aliases, and list slots for a seed slug.

Scope:
1. Replace `DEFAULT_DOCUMENT_TYPES` with one static `SEED_DOCUMENT_TYPES` table next to `DocTypeEntry` and `Config::default` in `src/config.rs` (ADR-036 decision 28). The table has exactly the eleven default types of REQ-007.002 and ADR-036 decision 29: `req`, `nfr`, `glossary`, `rule`, `ui-spec`, `ui-mockup`, `audit`, `tech-spec`, `use-case`, `test-case`, `test-report`. `request` and `release-reports` are user-defined types, not seed entries (ADR-036 decision 30, REQ-007.002 default set); this corrects the seed list in the task text, which named `request`.
2. Seed content per ADR-036 decision 29: `req` and `nfr` are `GroupedMarkdown` with `group.fields` and `member.fields` `[story]` (`## User Story`) and `[description]` (`## Description`), hidden reference visibility for both roles; `glossary` `definition` (`## Definition`, required); `rule` `description` (required), `rationale` (required, verbs `[create]`), `applicability` (required, verbs `[create]`), `requirements` (`## Requirements`, kind `list`, option `requirement`); `ui-spec` `view` and `description`; `audit` `audit_type` (`## Audit Type`, `frontmatter_key: type`, option `audit-type`) and `content` (`## Content`, required); `tech-spec` `description`; `use-case`, `test-case`, and `test-report` `description` and `applicability`; `ui-mockup` `fields: []` with the fixed `pending`, `approved`, `archived` metadata. Seed statuses are `[active, archived]` for every front-matter-backed type and none for `ui-mockup`.
3. Seed presentation facts come from the current `DOCUMENT_TEMPLATES` labels and the `PRESENTATION` entry of the same slug: `label`, `plural` (the `short` noun), `order` (the existing `PRESENTATION` order 10, 20, ... 110), `summary`/inventory note, `type_alias` (the legacy `?type=` key), `api_aliases`, and `slots`.
4. `Config::default`, `init`, and `ArtifactRegistry::build` derive from that one table. `Config::default` resolves each seed entry for the registry; `init` writes minimal entries only (TASK-248). `src/registry.rs` derives every document `TypeDescriptor` and its `RecordDescriptor` from the seed entry plus the resolved configuration entry, and the document entries and `RecordFacts` are deleted from `DOCUMENT_TEMPLATES`.
5. Derivation: the verb set is a function of `DocumentModel` plus the baseline `create`, `list`, `view`, `edit`, `archive`, `delete`, adding `status` for `SingleMarkdown` and `GroupedMarkdown`, `create-group` and the required `--group` for `GroupedMarkdown`, and `supersede` with no `status` for `SingleHtmlMockup` (ADR-036 decisions 7 and 8). Each field contributes one long option named by `option` or by `key` in kebab case, offered only on the field's `verbs`, in sequence order, with the field `label` and `help` and the `kind` value form (ADR-036 decisions 15, 17, 19, 20). No per-type document verb array survives.
6. The record descriptor derived from the same data: `frontmatter_keys` are the general keys, then `status` where the model stores it, then each field's `frontmatter_key` in sequence order; `sections` are each field's `heading` in sequence order (TECH-071 field-source matrix, ADR-036 decision 20).
7. A user-defined slug has no seed entry. It resolves to `SingleMarkdown` with `enabled: true`, a label derived from the slug, an order after every seed type, an empty summary and aliases, visible reference visibility, `statuses: [active, archived]`, no `default_status`, and one optional `multiline` `description` field with heading `## Description` (ADR-036 decision 33).
8. Selector paths: expose the flattening that `config list` uses, addressing every leaf as `document_types.<slug>.<path>`, including `fields[k]` and `fields[key]`, `group.fields[...]`, and `member.fields[...]`; the numeric and keyed forms resolve to the same leaf, and an out-of-range index or a keyed selector that matches no unique field is rejected with the same `document_types.<slug>.<path>` diagnostic (ADR-0019 amendment to decision 10, REQ-007.002).
9. Keep the enumerated order unchanged in this milestone: no consumer sorts by seed `order` yet, so registry enumeration stays in today's order for documents (slug order), and root help, browser navigation, and inventory output are unchanged. TASK-260 and TASK-268 adopt `order` later.

Invariants:
- Old configuration files must parse to the same effective document-type descriptor set (MS-033 AC-1): the seed adds data for the eleven default types, and the effective prefix, directory, enabled flag, statuses, default status, model, verb set, headings, and front-matter keys for a given file are the same as before.
- No CLI or browser behaviour changes in this milestone (MS-033 AC-7). The seed is new data that no CLI or browser consumer reads until MS-034 and MS-035; the derived verbs/options/help and selector paths are exercised by descriptor tests only.

Open question that blocks execution: the seed `slots` values conflict across the accepted records. ADR-039 decision 25 says a document descriptor's slots come from its seed entry and are "currently none", and ADR-039 decision 6 forbids the reserved key `group` as a slot key. The current `PRESENTATION` table declares document slots that the browser renders today: `req` and `nfr` `[group]`, `rule` `[applicability]`, `audit` `[audit_type]`, `ui-mockup` `[task, specs]`; the corpus page `tests/fixtures/characterization/corpus/expected/http/list-req/page.html` has a `Group` column. Preserving today's output therefore needs those five slot sets in the seed, including the engine-owned `group`, while ADR-039 says the seed declares none. The architect must either amend ADR-039 decisions 6 and 25 to allow the declared slot sets (including `group` for a grouped type) or declare the five column changes as expected deltas. The rest of this task is independent of that answer; the seed `slots` values and the matching acceptance criterion are added once it is settled.

Out of scope: the configuration file shape and emitter (TASK-248), the read path (TASK-250), the CLI command builder (MS-034), and the browser surface (MS-035). No dependency, ordinal, milestone, or assignee changes.

Assumptions:
- `plural` is the `PRESENTATION` `short` noun, and `summary` is the inventory note; neither is invented where the table has no value.
- Seed order values are the existing `PRESENTATION` numbers, so a later milestone that adopts them keeps the current navigation order.
- `request` and `release-reports` keep behaving as user-defined types; the project file declares both, with their own statuses and default status.

Definition of done: the seed table is the only production source of document models, roles, fields, headings, options, statuses, and presentation facts; the eleven default entries match ADR-036 decision 29; user-defined defaults match decision 33; verbs, options, help, record front-matter keys, and sections are derived; selector paths resolve in both forms; `cargo test` and `cargo clippy --all-targets -- -D warnings` are clean.
## Acceptance Criteria
- [ ] `SEED_DOCUMENT_TYPES` is the only production table of document models, roles, fields, headings, options, statuses, and presentation facts; `DEFAULT_DOCUMENT_TYPES` is gone, and a source guard fails if a second production table holds a document field, heading, option, or role set (ADR-036 decision 28 and downstream constraint 4).
- [ ] `Config::default()` yields exactly the eleven seed slugs of REQ-007.002 with the prefix, directory, statuses, model, and reference visibility of ADR-036 decision 29, and `ui-mockup` declares no statuses (REQ-005.008, ADR-036 decision 10).
- [ ] Every seed slug's field set equals ADR-036 decision 29: key, heading, `frontmatter_key` (`audit_type` stored as `type`), option (`requirement`, `audit-type`), kind, required, verbs (`rationale` and `applicability` are `[create]`), and sequence order (REQ-005.001, REQ-005.003, REQ-005.005, REQ-005.006, REQ-005.008, TC-122).
- [ ] `req` and `nfr` are `GroupedMarkdown` with independent `group.fields` and `member.fields` sequences and hidden reference visibility in both roles; `ui-mockup` is `SingleHtmlMockup` with `fields: []` and the fixed `pending`, `approved`, `archived` metadata; every other seed slug is `SingleMarkdown` with visible reference visibility (ADR-036 decisions 7 and 29, REQ-007.004).
- [ ] A user-defined slug with no seed entry resolves to `SingleMarkdown` with a slug-derived label, an order after every seed type, visible reference visibility, `[active, archived]` statuses, no default status, and one optional multiline `description` field with heading `## Description` (ADR-036 decision 33, TC-102).
- [ ] The derived verb set is baseline plus `status` for `SingleMarkdown` and `GroupedMarkdown`, plus `create-group` for `GroupedMarkdown`, and `supersede` with no `status` for `SingleHtmlMockup`; no per-type document verb array remains (ADR-036 decisions 7 and 8, TC-119, TC-120, TC-121).
- [ ] The derived options follow the field sequence: name from `option` or the kebab-case `key`, offered only on the field's `verbs`, ordered by sequence position, with the field `label` and `help` and the `kind` value form (ADR-036 decisions 15 to 20, TC-122, TC-123).
- [ ] `ArtifactRegistry::build` derives each record descriptor's `frontmatter_keys` and `sections` from the same field sequence: the general keys, then `status` where the model stores it, then each field's `frontmatter_key`; sections are each field's `heading` in sequence order (TECH-071 field-source matrix, ADR-036 decision 20).
- [ ] Selector paths resolve in both forms: `document_types.req.group.fields[0].heading` and `document_types.req.group.fields[story].heading` return the same leaf, `config list` addresses every field property by key selector, and an unknown key or out-of-range index is rejected with the same `document_types.<slug>.<path>` diagnostic (ADR-0019 amendment to decision 10, REQ-007.002, TC-172, TC-191, TC-117).
- [ ] Registry enumeration order is unchanged from the MS-033 start commit for every fixture configuration, so root help, browser navigation, and inventory output are unchanged (MS-033 AC-7, TC-128).
- [ ] `cargo test` reports zero failures and `cargo clippy --all-targets -- -D warnings` is clean.
## Comments
- **task-refiner**: Refined for MS-033: description grounded in DEFAULT_DOCUMENT_TYPES, DOCUMENT_TEMPLATES, the typed schemas and PRESENTATION; 11 testable acceptance criteria added; TECH-070 and TECH-077 documented. Blocked on one architect question: ADR-039 decisions 6 and 25 say document seed slots are none and forbid a group slot, while PRESENTATION renders five document slot columns including group for req and nfr. The task text also named request as a seed entry; corrected to user-defined per ADR-036 decision 30. No dependency, ordinal, milestone, or assignee change.
- **rust-dev**: TASK-249 implemented (no commit per instruction; status In-Progress). Seed: SEED_DOCUMENT_TYPES in src config.rs, 11 entries, declaration order req nfr glossary rule ui-spec ui-mockup audit tech-spec use-case test-case test-report, carrying prefix directory label plural order summary type_alias api_aliases reference_visibility per role, ADR-041 slots, statuses default_status DocumentModel fields group member and record facts. req and nfr are GroupedMarkdown with story and description in group and member and hidden visibility; glossary has required definition; rule has required description rationale applicability with rationale and applicability create-only and list requirements option requirement; ui-spec view description; audit audit_type stored as type with option audit-type plus required content; tech-spec description; use-case test-case test-report description applicability; ui-mockup SingleHtmlMockup empty fields no statuses slots task specs; other seed types empty slots statuses active archived no default status. DEFAULT_DOCUMENT_TYPES removed, RecordFacts and document DOCUMENT_TEMPLATES entries removed, is_builtin_document_slug replaced by config is_seed_document_slug. Derivation: Config::default loops the seed, init_value writes minimal entries, resolve_document_types fills seed slugs and leaves user-defined declared, registry resolved_document_entry applies seed fill or decision 33 defaults; document_descriptor derives status model verbs (baseline plus status, create-group for grouped, supersede only mockup), id shape, group requirement, archive policy, fields roles slots visibility, frontmatter keys general then status then field keys, and record sections from field headings. User-defined defaults: slug label, order 111, visible, active archived, no default status, optional multiline description. Delta: only documented user-defined reference visibility Visible. Transitional: document TypedSchema retained so MS-033 help and CLI output stay byte-identical; MS-034 deletes it. Tests added for seed contents derivation user defaults immutable group membership seed-only models selector group and member both forms source guard. env -u EDITOR -u VISUAL cargo test 1186 passed 65 suites 0 failed; cargo clippy all-targets -D warnings clean; cargo fmt check clean.
- **orchestrator**: Acceptance met: SEED_DOCUMENT_TYPES is the single seed table for the 11 types with ADR-041 slots and decision-29 fields; Config default, init and registry derive from it; verbs, options and record descriptors derive from model and sequence; selectors resolve keyed and indexed; 1186 tests pass and characterization, list and help goldens unchanged.
