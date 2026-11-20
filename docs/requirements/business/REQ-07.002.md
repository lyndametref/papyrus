---
id: REQ-07.002
title: "Configure Document Types and Slugs"
created_date: '2026-11-13T21:19:03Z'
updated_date: '2026-11-20T20:22:39Z'
labels: []
references:
  - RE-010
  - ADR-036
  - ADR-0019
  - ADR-0024
  - REQ-07.004
  - ADR-0016
  - REQ-01.002
  - REQ-01.004
  - REQ-01.006
  - REQ-01.008
  - REQ-05.007
  - REQ-05.008
  - REQ-05.010
  - NFR-02.002
  - NFR-04.002
  - BR-016
---
# REQ-07.002 Configure Document Types and Slugs

## User Story
As a project maintainer, I want to declare each document type, its content fields, and its defaults in the configuration file so that I can add, rename, disable, or remove a document type and change its fields without changing Papyrus.

## Group
REQ-07 Artifact Categories and Config-Defined Document Types

## Criticality
Must have

## Description
`document_types` is a map keyed by **document slug**. The slug is the configuration key, the CLI command name (`papyrus <document-slug>`, REQ-07.003), and the identifier used for that document type elsewhere. No separate `document-slug` attribute exists, because the map key already carries that role.

### Entry shape

Each entry is a mapping. Its top-level keys are:

- `prefix` — the ID prefix; required and never filled from the seed.
- `directory` — the type's folder relative to `doc_directory`; required and never filled from the seed. Archived records are stored in an `archived/` subfolder of it (REQ-05.007).
- `enabled` — whether the type is active; fills from the seed.
- `label`, `plural`, `order`, `summary`, `type_alias` — presentation scalars; each fills from the seed.
- `reference_visibility` — `visible` or `hidden`, the reference visibility of the type's lists; fills from the seed. On a grouped type it is declared inside the `group` and `member` blocks instead.
- `statuses` — the type's document statuses (REQ-07.001); a collection.
- `default_status` — the status a new record starts at; fills from the seed.
- `slots` — presentation slots; a collection.
- `api_aliases` — API aliases; a collection.
- `fields` — the field sequence of a single-role type; a collection.
- `group`, `member` — the two role blocks of a grouped type; each contains a `fields` sequence and an optional `reference_visibility`.

`model`, `title`, `content`, and `id_shape` are not configuration keys; an entry that declares any of them is rejected (REQ-07.004).

A single-role type (`SingleMarkdown` or `SingleHtmlMockup`) uses a top-level `fields` sequence and no role block. A grouped type (`GroupedMarkdown`) uses `group` and `member` blocks; a top-level `fields` on a grouped type is rejected. `fields: []` is valid and declares no field, and a `SingleHtmlMockup` type declares `fields: []`. The model itself is seed data and is not declared in configuration.

### Field items

A `fields` sequence contains field items. A field item has exactly these keys:

- `key` — required and stable; identifies the field in field maps, selector paths, diagnostics, and browser form submissions.
- `heading` — the level-2 Markdown heading that stores the value in the body, written with the `## ` prefix (for example `## User Story`).
- `frontmatter_key` — the front-matter key that stores the value; it may differ from `key`.
- `option` — the long CLI option name without the leading `--`; when absent it is derived from `key` in kebab case.
- `label` — the human-facing label used by CLI help and the browser form.
- `kind` — exactly one of `text`, `multiline`, or `list`.
- `required` — a boolean; a required field must be supplied on create and may be changed but not cleared on edit.
- `verbs` — the verbs on which the field's option is offered, a subset of `create` and `edit`, defaulting to `[create, edit]`.
- `help` — the option help text.

`label` and `help` are presentation-only. Field sequence position is the field's order everywhere — body sections, CLI option and help order, and browser form order; a field declares no `order` and no `enabled`, and either key is rejected.

A field must declare `heading`, `frontmatter_key`, or both; a field with neither is rejected. A field with both writes both the body section and the front-matter key; a field with only `heading` is body-only, and a field with only `frontmatter_key` is metadata-only.

The keys `id`, `title`, `status`, `labels`, `references`, `created_date`, `updated_date`, and `group` are reserved. A field may not use a reserved key as its `key` or as its `frontmatter_key`. Field keys are unique within a role — inside `fields`, inside `group.fields`, and inside `member.fields` — while the same key may appear in both the group and member roles.

### Defaults, fill, and merge

Configuration is resolved per entry against the seed entry of the same slug:

- Scalars fill from the seed when absent: `enabled`, `label`, `plural`, `order`, `summary`, `type_alias`, `default_status`, and `reference_visibility`. A present scalar overrides the seed.
- Collections replace wholesale when present: `fields`, `group.fields`, `member.fields`, `statuses`, `slots`, and `api_aliases`. An absent collection inherits the seed collection; a present collection replaces it, including `fields: []`.
- The `group` and `member` blocks resolve independently: a present block replaces only that role's collection, and an absent block inherits that role's seed collection.
- `prefix` and `directory` are never filled from the seed. Every entry, including an entry for a seed slug, declares both; a missing or invalid value is rejected.
- An absent slug stays absent. `document_types` is replaced wholesale when present in the project file, and no seed entry is re-added.

A user-defined slug has no seed entry. It defaults to `SingleMarkdown`, `enabled: true`, a label derived from the slug, an order after the seed types, an empty summary and empty aliases, visible reference visibility, `statuses: [active, archived]`, no `default_status`, and one optional `description` field with heading `## Description` and kind `multiline`. A declared value replaces the default under the same scalar-fill and collection-replace rules.

### Validation

Configuration loading validates every entry before the type set is used. Validation is fail-closed and reports every diagnostic in one pass, in deterministic order, with no partially valid document-type set: an invalid entry is never skipped, cleaned, or normalized, and no command runs on the resulting set (NFR-02.002).

A diagnostic names the offending location in the form `document_types.<slug>.<path>`, where `<path>` is a top-level key or a nested path such as `group.fields[story].heading`; a field diagnostic also names the field key when one is present. An entry-level violation that has no narrower path uses `document_types.<slug>`.

Unknown keys and unknown enum values are rejected with `document_types.<slug>.<path>`. This covers an unknown top-level key, an unknown field-item key, a value outside an allowed set (such as a `kind` other than `text`, `multiline`, or `list`, a `verbs` entry other than `create` or `edit`, or a `reference_visibility` other than `visible` or `hidden`), a field `order` or `enabled` key, a `model`, `title`, `content`, or `id_shape` key, a field with neither `heading` nor `frontmatter_key`, a reserved field key or `frontmatter_key`, and a duplicate field key within a role.

The remaining entry rules are:

- A slug must be usable as a command name: non-empty, composed of ASCII letters, digits, `-`, and `_`, not starting with `-`, and free of whitespace, control characters, and path separators. A slug must not equal a built-in command name — `init`, `config`, `check`, `task`, `milestone`, `release`, `decision`, `search`, `browser`, `import`, `help`, or `version` — so a configured document type can never shadow a built-in command; the comparison is case-insensitive, so `Config` is rejected as well as `config`.
- Each entry must have a non-empty `prefix` and a `directory` that is relative and stays inside the project directory (NFR-04.002). A `prefix` must be unique across all configured document types, including disabled entries. Two enabled entries must not declare the same `directory`.
- A `document_types` mapping that declares the same slug key twice is invalid configuration and is rejected; a repeated key is never resolved by keeping the first or the last occurrence.
- `statuses`, when present, must be a non-empty list with no duplicates and must include `active` and `archived` exactly once. Those two values are fixed: configuration cannot remove, rename, or replace them, and a custom status is any other value in the list. Every declared status follows the same character rules as a slug. `default_status`, when present, must be one of the declared statuses and must not be `archived`, so a new record never starts archived.
- A `statuses` or `default_status` declaration on a `SingleHtmlMockup` type is rejected, because that type keeps its state in HTML metadata (REQ-05.008); `ui-mockup` is the only seed type of that model.

### Configuration commands

`papyrus init` writes the default document-type set with minimal entries: each written entry declares only the values that are never filled (`prefix` and `directory`), plus `enabled` when the interactive walkthrough disables that artifact (REQ-01.006). Every other value comes from the seed.

Selector paths address configuration leaves in the form `document_types.<slug>.<path>`. A field sequence element is addressed either by key, as in `document_types.req.member.fields[story].heading`, or by zero-based numeric index, as in `document_types.req.group.fields[0].required`; both forms are accepted and resolve to the same element. `papyrus config get` and `papyrus config set` accept one selector, and `config list` prints the full effective configuration, seed-filled values included, addressing each field property by its key selector. An unknown selector or an out-of-range index is rejected with the same `document_types.<slug>.<path>` diagnostic.

`papyrus config set document_types.<slug>.<path> <value>` materializes only the target entry when the file does not declare it, applies the single change, validates the resulting configuration, and writes it; it never materializes the whole seed set. `papyrus config unset` of a collection resets it to the seed collection for that slug, and unset of a scalar removes the override so the seed value applies again.

`fields` is a sequence of mappings and is never written with a comma-separated value; adding, removing, reordering, or changing a field is a configuration file edit. The flat lists `statuses`, `api_aliases`, and `slots` keep comma syntax: `papyrus config set document_types.<slug>.statuses active,archived,custom` writes a YAML flow sequence (REQ-01.002, ADR-0024). Commas are separators with no escape form, and an empty value is rejected rather than written as an empty list. The configuration that would result is validated before it is persisted, so an invalid value is refused and nothing is written.

A violating configuration is rejected with a validation outcome (REQ-01.004) that names the offending slug or key. Validation is fail-closed and there is no partially valid document-type set: every command refuses to run while the effective configuration is invalid, with three exceptions that stay available. `papyrus config get|set|unset|list` loads the configuration tolerantly, validates the configuration that would result from the change, and refuses an invalid write. `papyrus init --force` replaces the configuration file with the generated default set. `papyrus check` loads the configuration tolerantly, reports the configuration findings, and runs the checks that do not depend on the effective `document_types` set (REQ-01.008). A user can therefore always repair a broken configuration through Papyrus itself.

Entries are fully ordinary. The default entries seeded by `papyrus init` have no reserved status: any entry can be renamed, removed, disabled, or given a different prefix or folder, and new entries can be added by editing `papyrus.config.yml` or with `papyrus config set`. A project initialized with `papyrus init` writes the default set:

- `req` — `REQ`, `requirements/business/`
- `nfr` — `NFR`, `requirements/non-functional/`
- `glossary` — `GLO`, `glossary/`
- `rule` — `RUL`, `business-rules/`
- `ui-spec` — `UI`, `specifications/ui/`
- `ui-mockup` — `MOCK`, `specifications/ui/mockups/`
- `audit` — `AUD`, `audit-records/`
- `tech-spec` — `TECH`, `technical/`
- `use-case` — `UC`, `use-cases/`
- `test-case` — `TC`, `test-cases/`
- `test-report` — `TR`, `test-reports/`

All default entries are enabled unless the interactive walkthrough (REQ-01.006) disables an artifact. `deploy` is not part of the default set; a project may still add it, like any other user-defined type, by declaring a `deploy` entry.

Adding an entry adds a document type and its command; disabling an entry removes its CLI command and browser views while leaving existing files untouched; removing an entry stops managing those records without deleting files. The non-functional requirement family is addressed by its slug `nfr`; the former `nfreq` command name is not retained as an alias.

## Acceptance Criteria
- `document_types` is keyed by document slug, and the slug is the CLI command name and the type identifier; no separate `document-slug` attribute exists.
- Every entry declares `prefix` and `directory`, and accepts `enabled`, `label`, `plural`, `order`, `summary`, `type_alias`, `reference_visibility`, `statuses`, `default_status`, `slots`, and `api_aliases`.
- A single-role entry uses a top-level `fields` sequence of field items and no role block; `fields: []` is valid and declares no field.
- A grouped entry uses `group` and `member` blocks, each with a `fields` sequence and an optional `reference_visibility`; a top-level `fields` on a grouped type is rejected.
- A field item accepts exactly `key`, `heading`, `frontmatter_key`, `option`, `label`, `kind`, `required`, `verbs`, and `help`; a field `order` or `enabled` key is rejected.
- A field declares `key`, at least one of `heading` and `frontmatter_key`, a `kind` of `text`, `multiline`, or `list`, a `required` flag, and `verbs`; a field with neither `heading` nor `frontmatter_key` is rejected.
- `verbs` defaults to `[create, edit]` and accepts only `create` and `edit`; any other verb is rejected.
- `option` is the long option name without the leading `--` and is derived from `key` in kebab case when absent.
- A required field must be supplied on create and may be changed but not cleared on edit; an optional field may be cleared.
- A field `key` or `frontmatter_key` equal to `id`, `title`, `status`, `labels`, `references`, `created_date`, `updated_date`, or `group` is rejected.
- A duplicate field key within a role is rejected, while the same key may appear in `group.fields` and `member.fields`.
- Field sequence position is the field's order in body sections, CLI options and help, and the browser form.
- Scalars fill from the seed when absent: `enabled`, `label`, `plural`, `order`, `summary`, `type_alias`, `default_status`, and `reference_visibility`.
- Collections replace wholesale when present: `fields`, `group.fields`, `member.fields`, `statuses`, `slots`, and `api_aliases`; an absent collection inherits the seed collection, and `fields: []` replaces it with an empty collection.
- The `group` and `member` blocks resolve independently: a present block replaces only its role's collection, and an absent block inherits its role's seed collection.
- `prefix` and `directory` are never filled; an entry that omits either is rejected, including an entry for a seed slug.
- A present `document_types` replaces the seed set wholesale, and an absent slug stays absent; no seed entry is re-added.
- A user-defined slug defaults to `SingleMarkdown`, enabled, a slug-derived label, an order after the seed types, empty summary and aliases, visible reference visibility, `[active, archived]` statuses, no default status, and one optional multiline `description` field with heading `## Description`.
- An unknown top-level or field-item key is rejected with a `document_types.<slug>.<path>` diagnostic.
- An unknown enum value — a `kind` other than `text`, `multiline`, or `list`, a `verbs` entry other than `create` or `edit`, or a `reference_visibility` other than `visible` or `hidden` — is rejected with a `document_types.<slug>.<path>` diagnostic.
- A `model`, `title`, `content`, or `id_shape` key is rejected.
- A field diagnostic names the offending location by selector path, using the keyed `fields[story].heading` or indexed `fields[0].heading` form, and names the field key when one is present.
- Validation is fail-closed and reports every diagnostic in one pass in deterministic order; no entry is skipped and no command runs on an invalid set.
- `papyrus init` writes the default set as minimal entries declaring only `prefix` and `directory`, plus `enabled` when the walkthrough disables an artifact.
- `papyrus config set document_types.<slug>.<path> <value>` materializes only the target entry, applies one change, validates the result, and never materializes the whole seed set.
- `papyrus config list` prints the full effective configuration, seed-filled values included, with key selectors for field properties.
- Selector paths accept the keyed `member.fields[story].heading` and numeric `group.fields[0].required` forms, which resolve to the same element; an unknown selector or an out-of-range index is rejected with the same diagnostic.
- `papyrus config get` and `papyrus config set` accept one selector; `config unset` of a collection resets it to the seed collection, and unset of a scalar removes the override.
- `fields` is never written with a comma-separated value; adding, removing, reordering, or changing a field is a configuration file edit.
- `statuses`, `api_aliases`, and `slots` keep comma syntax, are stored as YAML flow sequences, treat commas as separators with no escape form, and reject an empty value.
- A `config set` value is validated as the resulting configuration before it is persisted; an invalid value is refused and nothing is written.
- A slug is non-empty, uses only ASCII letters, digits, `-`, and `_`, does not start with `-`, and contains no whitespace, control character, or path separator; a violating value is rejected and not silently cleaned (NFR-02.002).
- A slug equal to `init`, `config`, `check`, `task`, `milestone`, `release`, `decision`, `search`, `browser`, `import`, `help`, or `version` is rejected with a validation outcome naming the conflict, and the comparison is case-insensitive.
- `prefix` is non-empty and unique across all entries including disabled ones; `directory` is relative and inside the project directory (NFR-04.002), and duplicate directories among enabled entries are rejected; a duplicate slug key is rejected without preferring either occurrence.
- A `statuses` list is rejected when it is empty, contains duplicates, omits `active` or `archived`, or contains a value that violates the slug character rules; `default_status` must be a declared status and must not be `archived`.
- A `statuses` or `default_status` declaration on a `SingleHtmlMockup` type is rejected; `ui-mockup` is the only seed type of that model.
- An invalid configuration blocks every command with a validation outcome (REQ-01.004); the tolerant surfaces are `config get|set|unset|list`, `init --force`, and `check`.
- A disabled entry has no CLI command or browser view and its existing files are untouched; removing an entry stops managing its records and deletes no file.
- The default set contains the eleven listed slugs with their prefixes and directories, all enabled unless the walkthrough disables an artifact; `deploy` is not in the set but can be added as a user-defined type.
- The non-functional requirement family uses the slug `nfr`, and the former `nfreq` command name is not retained as an alias.

## Open Questions
- Whether removing an entry should leave its former records visible read-only somewhere, or simply unmanaged. Current requirement: unmanaged and untouched.
- Whether slug renames should rewrite existing record IDs or folders. Current requirement: renaming is a configuration-only change, so existing records stay in their folder and keep their IDs.

## Traceability
- Source: RE-010 — Descriptor-driven document engine refactor (accepted, not reopenable): D10 (presentation facts live in the config seed) and D14 (nested group and member blocks, `fields` as a YAML sequence, no per-field `order` or `enabled`, scalars fill from the seed, collections replace, `model`, `title`, `content`, and `id_shape` are not config keys, init writes a minimal file, config list is full and uses selector paths, adding a field is file-edit-only).
- ADR-036 — Config-declared document field schema and one descriptor model (decisions 4-27 field vocabulary and role sets, 28-31 seed table, 32-34 defaults, fill, merge, and diagnostics).
- ADR-0019 — Fail-Closed Document-Type Configuration Validation (decisions 1-7 validation and slug, prefix, directory, status rules; 9 materialize before edit; 10 repair surfaces; amendment: model-keyed status rejection, collection replace, selector paths, re-asserted invariants, indexed field diagnostics).
- ADR-0024 — List-Valued Configuration Keys in `papyrus config set` (comma syntax, flow-sequence storage, no escape form; amendment: `fields` and role field sequences are never list-valued, and adding a field is a file edit).
- ADR-0016 — Runtime Config-Derived Command Dispatch for Document Slugs (built-in command name list).
- REQ-07.004 — Provide One Schema-Driven Document Content Model with Three Document Models (field vocabulary, role field sets, model not a configuration key).
- REQ-07.001 — Organize Artifacts into Three Categories (document status model).
- REQ-07.003 — Dispatch Config-Defined Document Commands (slug as command name).
- REQ-01.002 — Read and Edit Project Configuration (`config get|set|unset|list`, list-valued keys, selector paths).
- REQ-01.004 — Use Common CLI Controls and Outcomes (validation outcome).
- REQ-01.006 — Interactive Init Walkthrough (default artifact set and `enabled`).
- REQ-01.008 — Run Project Diagnostics (tolerant `check` surface).
- REQ-05.007 — Manage Document Archive State and Deletion (archived subfolder).
- REQ-05.008 — Manage UI Mockups (HTML metadata state).
- REQ-05.010 — Record Metadata in YAML Front Matter (general front-matter keys and reserved field keys).
- NFR-02.002 — Sanitize Input (reject rather than silently clean).
- NFR-04.002 — Restrict File Access to the Project Directory (entry directory boundary).
- BR-016 — Papyrus Configuration Keys and Precedence (entry keys, nesting, selector paths, fill and replace rules).
