---
id: ADR-036
title: "Config-declared document field schema and one descriptor model"
created_date: '2026-11-20T18:28:00Z'
updated_date: '2026-11-22T12:45:34Z'
labels: []
references:
  - RE-010
  - ADR-0017
  - ADR-0018
  - ADR-0019
  - REQ-07.001
  - REQ-07.002
  - REQ-07.003
  - REQ-07.004
  - REQ-05.001
  - REQ-05.002
  - REQ-05.005
  - REQ-05.006
  - REQ-05.008
  - REQ-05.010
  - REQ-05.011
  - TECH-007
  - TECH-040
  - TECH-066
  - TECH-067
  - TECH-068
  - TECH-070
  - TECH-071
  - TECH-072
  - TECH-077
  - ADR-042
superseded_by: null
---
# ADR-036 Config-declared document field schema and one descriptor model

## Status
accepted
## Decision
### Clarification (2026-11-22) — seed slugs are always effective

- **Decision 32, "An absent slug stays absent":** the eleven seed slugs of decision 29 are always part of the effective configuration and resolve from the seed table whether or not the project file declares them; a partially declared `document_types` map does not remove the other seed types. "An absent slug stays absent" applies to non-seed (user-defined) slugs, and to the written file, which never gains entries that the command did not target. The fill and replace rules for a declared entry in decision 32 are unchanged. ADR-041's seed slot sets are likewise always effective.
- No other decision, downstream constraint, or status changes.

### Clarification (2026-11-22) — audit field default

- **Decision 29 seed table — `audit` carries one field default.** The `audit` seed entry carries `field_defaults: { audit_type: security }`; every other seed entry carries an empty `field_defaults`. `SeedDocumentType` gains the `field_defaults` slice and the resolved descriptor exposes the same map. On `create`, an absent field that has a seed default is written through the ordinary `PatchOp` path; on `edit` an absent field stays absent. `field_defaults` is not a `FieldSpec` key, not a configuration key, and not accepted in a project entry; a user-defined type has no field defaults (ADR-042 ruling 1). Decision 11's nine-key `FieldSpec` and decision 32's scalar-fill and collection-replace rules are unchanged; no other type gains a default.
- **Not changed:** decisions 4-28 and 30-37, the downstream constraints, and this record's accepted status.
## Supersession

1. This ADR supersedes ADR-0017 decisions 1, 2, 3, 4, 7, 9, and 10 and downstream constraints 1 and 4, and only those. ADR-0017 decisions 5, 6, and 8 and downstream constraints 2, 3, 5, and 6 remain in force. ADR-0017 keeps its accepted status because the supersession is by decision, not in full.
2. This ADR supersedes ADR-0018 in full. The ADR-0018 record is marked Superseded by ADR-036. Its decisions 1, 3, 5, 6, 7, 9, and 10 and downstream constraints 2, 5, and 6 are replaced explicitly; its remaining clauses are retired with the record rather than contradicted.
3. ADR-0019 (fail-closed validation) still applies to the configuration and schema introduced here. No other ADR is superseded or amended by this record.

## One descriptor model

4. There is exactly one document descriptor model. A document type is one `TypeDescriptor` in the `Document` category carrying its slug, prefix, directory, enabled state, status vocabulary, `DocumentModel`, role field sets, reference visibility, and presentation facts. Backlog and decision records stay bespoke per RE-010 D4; only documents are unified.
5. `ContentModel` is removed. `DocumentModel` is an enum with exactly three variants: `SingleMarkdown`, `GroupedMarkdown`, and `SingleHtmlMockup`. Nothing named `ContentModel` remains in code, configuration, or documentation.
6. The model is not a configuration key. `model`, `title`, `content`, and `id_shape` are not configuration keys and a declaration of any of them is rejected. A type's model comes from the seed table for a seed slug and is `SingleMarkdown` for every user-defined slug.
7. Per-variant behaviour:

| Model | Records and IDs | Verbs | Field sets | Status behaviour | Model-specific rules |
|---|---|---|---|---|---|
| `SingleMarkdown` | One record per ID; plain ID `<PREFIX>-<zero-padded sequence>`. | Baseline plus `status`. | One `fields` sequence. | Front-matter `status`: `active`, a declared custom status, or `archived`; `status` and `archive` are the only writers. | None. |
| `GroupedMarkdown` | A group record with a plain ID and member records with dotted IDs `<PREFIX>-<group-sequence>.<member-sequence>`. | Baseline plus `status` plus `create-group`. | `group.fields` and `member.fields`, each a sequence; no top-level `fields`. | Front-matter `status` on both roles, under the same rules as `SingleMarkdown`. | The engine owns the group link; membership is immutable after creation; no cascade on archive or delete; merged group-then-member ordering. |
| `SingleHtmlMockup` | One record per ID; plain ID. | Baseline plus `supersede`; no `status` verb. | No field sequence; `fields` must be empty. | Fixed `pending`, `approved`, and `archived` metadata in the HTML metadata comment; no custom statuses; archive is the only path to `archived`. | HTML body; the title contract applies to the HTML metadata; `supersede` replaces the superseded metadata. |

8. The baseline verb set is `create`, `list`, `view`, `edit`, `archive`, and `delete`. Verbs are derived from the model, never declared in configuration: `status` is added for `SingleMarkdown` and `GroupedMarkdown`, `create-group` and `--group` are added only for `GroupedMarkdown`, and `supersede` is added only for `SingleHtmlMockup`. No per-type module supplies verbs, options, or help.
9. `GroupedMarkdown`: `create-group` creates the group record with a positional title; `create` creates a member and requires an existing active group of the same type, supplied through the group slot or `--group`. A cross-type group, an archived group, or an unknown group is rejected. The member's `group` front-matter key is written and maintained only by the engine: it is a reserved key, never a field, and never editable through a field option. A member's group link cannot be changed after creation. Archiving or deleting a group does not archive, delete, or otherwise change its members. Lists merge the group records and their members in group order, then member order.
10. `SingleHtmlMockup`: the record body is HTML and its state lives in the metadata comment, not in YAML front matter. The allowed states are fixed to `pending`, `approved`, and `archived`; configuration cannot add, remove, or rename them, and a `statuses` or `default_status` declaration on this model is rejected. There is no `status` verb. `archive` sets `archived`; `supersede` records the superseding ID. The title is always present, single-line, and non-empty, and follows the same title contract as the Markdown models.

## Field vocabulary

11. A field is one `FieldSpec` with exactly these keys: `key`, `heading`, `frontmatter_key`, `option`, `label`, `kind`, `required`, `verbs`, and `help`. There is no `order` key and no `enabled` key on a field.
12. `key` is required and stable. It identifies the field in field maps, configuration selector paths, diagnostics, and browser form posts.
13. `heading` is the level-2 Markdown heading that stores the value in the body, written with the `## ` prefix (for example `## User Story`). It is optional only when `frontmatter_key` is present.
14. `frontmatter_key` is the front-matter key that stores the value. It is optional only when `heading` is present. It may differ from `key` (for example `audit_type` stored under `type`). It may not be a reserved field key.
15. `option` is the long CLI option name, written without the leading `--`. When absent it is derived from `key` in kebab case. The option is offered only on the verbs listed in `verbs`, and options follow field sequence order in help and parsing.
16. `label` is the human-facing label used by CLI help and the browser form. `help` is the option help text. Both are presentation-only.
17. `kind` is exactly one of `text`, `multiline`, or `list`. `text` is a single-line scalar. `multiline` is multi-line text and accepts `-` to read from standard input on the CLI and a text area in the browser. `list` is a repeatable option and a sequence of scalar values, stored as a YAML list when it has a `frontmatter_key`; a nested mapping sequence is never a list-valued field (ADR-0024). No other kind exists.
18. `required` is a boolean. A required field must be supplied on create; one diagnostic lists every missing required key and no write occurs. On edit a required field may be changed but not cleared; an optional field may be cleared.
19. `verbs` is a subset of `create` and `edit` and defaults to `[create, edit]`. It controls whether the field's option is offered on that verb and whether the value is editable there. No other verb name is valid.
20. Sequence position is the order of the field everywhere: body sections, CLI option order, help text, browser form order, and field-map traversal. Configuration order is the only order.
21. A field must declare `heading`, `frontmatter_key`, or both. A field with both writes both the body section and the front-matter key. A field with only `heading` is body-only; a field with only `frontmatter_key` is metadata-only. A field with neither is invalid.
22. Field keys are unique within a role. `fields` keys are unique; `group.fields` keys are unique; `member.fields` keys are unique. The same key may appear in both the group and member roles.
23. The reserved field keys are `id`, `title`, `status`, `labels`, `references`, `created_date`, `updated_date`, and `group`. A field may not use a reserved key, and `frontmatter_key` may not name a reserved key. `group` is reserved because the engine owns the group link.
24. Title is fixed and non-configurable. It is not a field, is always present, is single-line and non-empty, is required on create, and can never be cleared on edit. It is the first positional `TITLE` on `create` and `create-group` and `--title` on create, create-group, and edit. The HTML mockup metadata uses the same title contract. There is no `--term` option.
25. Configuration declares data only. It is never executable: no templates, expressions, scripts, or code paths are read from configuration.

## Role field sets

26. `SingleMarkdown` and `SingleHtmlMockup` declare a top-level `fields` sequence. `fields: []` is valid and declares no field.
27. `GroupedMarkdown` declares `group.fields` and `member.fields`, each a sequence with independent order, plus an optional `reference_visibility` for each role. A top-level `fields` on a grouped type is invalid. The group link is not a field and appears in neither role.

## Seed table

28. The seed table is the only definition of built-in document types. It lives in `src/config.rs` as a static `SEED_DOCUMENT_TYPES` table of seed entries, replacing `DEFAULT_DOCUMENT_TYPES`, next to `DocTypeEntry` and `Config::default`. `Config::default`, `init`, and the registry derive from that one table; `src/registry.rs` derives document descriptors from it. No per-type module and no second static table may hold document fields, headings, options, or role sets.
29. The seed contains exactly the default document types of REQ-07.002, with these models, roles, and fields:

| Slug | Model | Roles and fields | Seed statuses | Reference visibility |
|---|---|---|---|---|
| `req` | `GroupedMarkdown` | `group.fields`: `story` (`## User Story`), `description` (`## Description`); `member.fields`: `story`, `description` | `[active, archived]` | hidden for both roles |
| `nfr` | `GroupedMarkdown` | Same as `req` | `[active, archived]` | hidden for both roles |
| `glossary` | `SingleMarkdown` | `definition` (`## Definition`, required) | `[active, archived]` | visible |
| `rule` | `SingleMarkdown` | `description` (`## Description`, required), `rationale` (`## Rationale`, required, verbs `[create]`), `applicability` (`## Applicability`, required, verbs `[create]`), `requirements` (`## Requirements`, kind `list`, option `requirement`) | `[active, archived]` | visible |
| `ui-spec` | `SingleMarkdown` | `view` (`## View`), `description` (`## Description`) | `[active, archived]` | visible |
| `ui-mockup` | `SingleHtmlMockup` | none; `fields: []` | none; fixed `pending`, `approved`, `archived` metadata | visible |
| `audit` | `SingleMarkdown` | `audit_type` (`## Audit Type`, `frontmatter_key: type`, option `audit-type`), `content` (`## Content`, required) | `[active, archived]` | visible |
| `tech-spec` | `SingleMarkdown` | `description` (`## Description`) | `[active, archived]` | visible |
| `use-case` | `SingleMarkdown` | `description` (`## Description`), `applicability` (`## Applicability`) | `[active, archived]` | visible |
| `test-case` | `SingleMarkdown` | `description` (`## Description`), `applicability` (`## Applicability`) | `[active, archived]` | visible |
| `test-report` | `SingleMarkdown` | `description` (`## Description`), `applicability` (`## Applicability`) | `[active, archived]` | visible |

30. A seed entry also carries its prefix, directory, label, plural, order, summary, type alias, API aliases, reference visibility, and slots. Those presentation values are seed data per RE-010 D10; changing prose or aliases is a seed edit, while changing a model, role, field key, heading, option, kind, required flag, per-field verbs, or status set is an architectural change to this ADR. A project may declare additional document types (for example `request` or `release-reports`); they are user-defined and are not part of the seed.
31. `GroupedMarkdown` and `SingleHtmlMockup` are seed-only. Only `req` and `nfr` are grouped. Only `ui-mockup` is an HTML mockup. A user-defined slug always resolves to `SingleMarkdown`, even when it reuses a seed prefix, directory, or field names.

## Defaults, fill, and merge

32. Configuration is resolved per entry against the seed entry of the same slug:
    - Scalars fill from the seed when absent: `enabled`, `label`, `plural`, `order`, `summary`, `type_alias`, `default_status`, `reference_visibility`, and the model and roles. A present scalar overrides the seed.
    - Collections replace wholesale when present: `fields`, `group.fields`, `member.fields`, `statuses`, `slots`, and `api_aliases`. An absent collection inherits the seed collection; a present collection replaces it, including `fields: []`. Role blocks resolve independently: a present `group` block replaces only the group role's collection, and an absent one inherits it.
    - `prefix` and `directory` are never filled from the seed. Every entry, including an entry for a seed slug, must declare both; a missing or invalid value is rejected.
    - An absent slug stays absent. `document_types` is replaced wholesale when present in the project file, and no seed entry is re-added.
33. A user-defined slug has no seed entry. Its defaults are `SingleMarkdown`, `enabled: true`, a label derived from the slug, an order after the seed types, empty summary and aliases, visible reference visibility, `statuses: [active, archived]`, no `default_status`, and one optional `description` field with heading `## Description` and kind `multiline`. Declared values replace those defaults under the same scalar and collection rules.
34. Unknown keys and unknown enum values are rejected with a `document_types.<slug>.<path>` diagnostic. Field validation also rejects a missing `heading` and `frontmatter_key`, a reserved field key, a duplicate field key within a role, a field `order` or `enabled` key, and a `model`, `title`, `content`, or `id_shape` key.

## The engine writes only PatchOps

35. One document engine, parameterized by the descriptor, handles every document model. It builds `Vec<PatchOp>` values from the field sequence and the model: `SetFrontMatter` or `SetFrontMatterValue` for a `frontmatter_key`, `SetSection` or `ClearSection` for a `heading`, `SetHeadingTitle` for the title, and the model-specific operations for group links, HTML metadata, and supersession. List fields become front-matter sequences or section lists under the same ops.
36. The engine never writes a record body. It does not format a `---` block, re-render a body, call `fs::write`, or hold a per-type writer. `record::patch` applies the ops and `record::store` remains the only writer; a create and an edit each produce one patch, and a patch that changes nothing performs no write.
37. Required-field and reserved-key checks are enforced when the patch is built, before any write. A missing required field, a reserved field key, an invalid group link, or an invalid status produces one validation outcome and writes nothing.

## Downstream constraints

1. There is one document descriptor model and one engine. No per-variant handler module, no per-type field table, and no slug literal in the engine.
2. `DocumentModel`, the role field sets, and the model-derived verbs are not configuration keys; configuration declares fields and presentation, never behaviour.
3. Reserved field keys, the title contract, and the uniqueness rule per role are enforced in validation and again when a patch is built.
4. Every field declares `heading`, `frontmatter_key`, or both; the seed table is the only built-in field definition.
5. `GroupedMarkdown` and `SingleHtmlMockup` are seed-only; user-defined types are `SingleMarkdown`.
6. The engine emits only PatchOps; `record::store` is the only writer.
7. Scalar fill, collection replace, `prefix` and `directory` never filled, and absent slug stays absent are enforced by configuration resolution.
8. `SingleHtmlMockup` has fixed `pending`, `approved`, and `archived` metadata and no `status` verb; `GroupedMarkdown` membership is immutable and archive or delete never cascades.
9. ADR-0017 decisions 5, 6, and 8 and constraints 2, 3, 5, and 6, and ADR-0019, remain in force.
## Context
REQ-07.004 currently promises one generic content model for config-defined document types: seven front-matter keys plus an optional `## Description`. ADR-0018 fixed that model and explicitly excluded a per-type field schema and a template file; ADR-0017 decisions 1-4, 7, 9, and 10 fixed the descriptor model around `ContentModel` with `Generic`, `Typed`, and `HtmlMockup`, and around static `'static` typed schemas. Built-in document types keep their fields in nine per-type modules and static descriptors: `src/requirement.rs` owns `## User Story`, `## Group`, `## Description` and dotted IDs; `src/glossary.rs` owns `## Definition`; `src/rule.rs` owns four sections; `src/ui_spec.rs`, `src/audit.rs`, `src/tech_spec.rs`, `src/use_case.rs`, `src/test_case.rs`, and `src/test_report.rs` each own their fields and verb options; `src/ui_mockup.rs` owns HTML metadata. `src/config.rs` holds `DocTypeEntry { prefix, directory, enabled, statuses, default_status }` and a `DEFAULT_DOCUMENT_TYPES` table with no field, model, role, or presentation facts. `src/registry.rs` holds a `DescriptorTemplate` with `ContentModel::Typed`, `Generic`, or `HtmlMockup` per type. The record layer already has the single mutation API `record::patch` and the single writer `record::store` (ADR-0013), so the missing piece is a data model for fields, not another write path.

The accepted owner decisions in RE-010 change the premise: D1 supersedes ADR-0018 in full and replaces ADR-0017's descriptor decisions; D5 replaces `ContentModel` with `DocumentModel { SingleMarkdown, GroupedMarkdown, SingleHtmlMockup }`; D6 makes grouped and HTML models seed-only; D13 fixes the title contract; D14 fixes the configuration format as nested group and member blocks, a `fields` sequence, no per-field `order` or `enabled`, scalar fill, collection replace, no `model`, `title`, `content`, or `id_shape` keys, and file-edit-only field addition. The refactor also deletes the nine typed document modules (MS-034), so every field, heading, option, and verb they currently hold must become data before the modules can go.

Forces: one descriptor model so CLI, record layer, search, and browser cannot drift; all built-in document behaviour expressible as seed data with no per-type code; config-defined types still usable with minimal setup; reserved front-matter and engine-owned keys protected; byte preservation and the single write path retained; no new Rust crate (D3); old configuration files must load with the same effective set (MS-033 AC-1); fail-closed validation stays with ADR-0019; and the title stays a fixed command and record invariant rather than a field.

Alternatives considered:

| Option | Outcome | Reason |
|---|---|---|
| One `DocumentModel` enum, one seed table, config-declared `FieldSpec` sequences, and one PatchOp-emitting engine (this ADR) | Selected | Satisfies RE-010 D5, D6, D13, and D14, removes per-type modules, and keeps one write path. |
| Keep `ContentModel` and add a fourth variant for HTML | Rejected | `ContentModel` is superseded by `DocumentModel` (D5), and a fourth variant preserves the typed/generic split that the refactor removes. |
| Keep per-type static schemas for built-ins and add config fields only for user-defined types | Rejected | Built-in behaviour would still need code, so the typed modules and their verb tables could not be deleted. |
| Declare `model` in configuration so a user-defined type can be grouped or HTML | Rejected by RE-010 D6 | Grouped and HTML are seed-only; a user-defined type always defaults to `SingleMarkdown`. |
| Let a field declare its own `order` and `enabled` | Rejected by RE-010 D14 | Sequence position is the only order, and a field cannot be disabled without deleting it. |
| Store fields in a separate schema file or template | Rejected | A second source of body shape; fields are part of the document-type entry and `document_types` is replaced wholesale. |
| Render a record body in the engine and write it directly | Rejected | Breaks ADR-0013's single writer and the patch contract; the engine must emit PatchOps only. |
| Keep `title` as a configurable field | Rejected by RE-010 D13 | Title is fixed, always present, required on create, never clearable, and reserved. |
## Consequences
**Positive:**

- One `DocumentModel` and one seed table replace `ContentModel`, the static typed descriptors, the nine per-type document modules, and the hand-written verb tables. A document type cannot appear in the CLI and be missing from the browser, search, or record layer.
- Every built-in document field, heading, option, kind, required flag, per-field verbs, and status set becomes inspectable data. The browser form, CLI help, list filters, and the record envelope all read the same field sequence.
- Config-defined types gain real fields without code: adding a type, changing a field heading, or adding a field is a file edit. The old minimal generic type keeps working because a user-defined entry defaults to `SingleMarkdown` with one optional `description` field.
- The reserved-key list and the engine-owned `group` key keep `id`, `title`, `status`, `labels`, `references`, dates, and the group link out of user configuration, so front matter and IDs stay Papyrus-owned.
- Field values are written only through PatchOps, so existing byte-preservation, optimistic concurrency, and atomic-write guarantees continue to hold.
- Grouped and HTML behaviour is confined to two seed types, which keeps the common `SingleMarkdown` path small and testable.

**Negative:**

- A larger configuration surface: nested `group`/`member` blocks, a field sequence with nine keys, per-field verbs, and selector paths. A typo is rejected, but the file is no longer a flat three-key entry.
- Seed changes become behaviour changes. A heading, field key, option, required flag, per-field verb, or status set in the seed is now an architectural contract, so changing one requires a requirement or ADR change rather than a module edit.
- `prefix` and `directory` are never filled, so editing an existing default entry that omits them is rejected. Users must keep both keys on every entry; there is no seed fallback.
- The engine must support all three models and their patch shapes, including HTML metadata and group links, without becoming a second writer. The model-specific op builders are the main complexity.
- Old configurations that relied on the seeded flat schema cannot add fields through `config set`; fields are a YAML sequence and a file edit (D14).

**Risks and mitigations:**

- *Engine drifts into a second writer*: the engine returns PatchOps only and a contract test forbids `fs::write`, `---` formatting, and direct body rendering in the engine; the single writer remains `record::store`.
- *Field/renderer divergence*: one field sequence drives CLI options, help, browser form, list display, and patch building; a conformance test runs one type per model through create, edit, view, list, and JSON.
- *Model assignment leaks into configuration*: `model` is not a key, the seed table owns the model of a seed slug, and every user-defined slug is forced to `SingleMarkdown`; validation rejects a `model` key.
- *Reserved-key or duplicate-key bypass*: the check runs in configuration validation and again when the patch is built, and tests cover every reserved key, duplicate keys per role, and a field with neither `heading` nor `frontmatter_key`.
- *Group membership drift*: the `group` key is engine-only, `edit` exposes no group option, and a test asserts that a member cannot be moved and that archiving or deleting a group leaves members untouched.
- *HTML metadata treated as front matter*: `SingleHtmlMockup` declares no `statuses`, has no `status` verb, and its metadata states are fixed; a test asserts the four rejected configurations (custom status, default status, status verb, field sequence).
- *Seed and requirement drift*: the seed table is asserted against the per-type field inventories in REQ-05.001 to REQ-05.014 and BR-009 to BR-020; a seed change that is not reflected in those records fails the cross-check.
- *Old configuration regression*: a fixture set of pre-refactor files must load to the same effective descriptors, with the only difference being the new default field sets for seed types.
