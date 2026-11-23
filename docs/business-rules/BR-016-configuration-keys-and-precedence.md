---
id: "BR-016:"
title: "Papyrus Configuration Keys and Precedence"
created_date: '2026-11-15T18:37:54Z'
updated_date: '2026-11-23T07:27:35Z'
labels: []
references: []
status: active
---
# BR-016: Papyrus Configuration Keys and Precedence

## Description

`papyrus.config.yml` is the single source of project paths, ID formats, workflow statuses, document types, and browser settings, and each `document_types` entry is a nested mapping — presentation scalars, status and alias collections, and either a top-level `fields` sequence or `group` and `member` blocks — whose effective value is resolved per slug against the seed entry by scalar default-fill and wholesale collection replace, under the fixed precedence order command-line options, environment variables, the project configuration file, then built-in defaults.

## Rationale

Based on REQ-001.002 — Read and Edit Project Configuration, REQ-001.003 — Select Project and Configuration Context, REQ-007.001 — Organize Artifacts into Three Categories, REQ-007.002 — Configure Document Types and Slugs, REQ-007.004 — Provide One Schema-Driven Document Content Model with Three Document Models, and REQ-001.008 — Run Project Diagnostics, this rule exists because configuration is the only place document types, their slugs, prefixes, directories, status vocabularies, fields, and role field sets are declared, and the entry is now nested rather than flat: the loader must resolve a sparse entry against the seed deterministically, validate the whole map fail-closed with a diagnostic that names the exact nested path, and still leave a repair path, or one mistyped entry would make every command unusable and the project could not be fixed through Papyrus itself.

## Applicability

Applies to every `papyrus.config.yml` file and to the project data it governs.

### Precedence

1. Command-line options.
2. Environment variables (`PAPYRUS_DIR`, `PAPYRUS_CONFIG`).
3. `papyrus.config.yml` in the project directory.
4. Built-in defaults.

This order is unchanged by the nested document-type model, and the seed is the last source inside it: a seed value applies only where the project file declares none.

### General, path, and workflow keys

- `project_name`, `task_prefix`, `milestone_prefix`, `release_prefix`, `id_padding` (1–6, default 3), `date_format`, and `editor`.
- `backlog_directory`, `decision_directory`, and `doc_directory`; each document type's `directory` is relative to `doc_directory`.
- `statuses` and `default_status` configure task statuses only. The milestone (`open -> closed -> archived`), release (`planned -> released`), and decision lifecycles stay fixed, and the document main statuses `active` and `archived` are not configurable.

### document_types entry shape

- The map is keyed by document slug; the key is the command name (`papyrus <slug>`) and the identifier used elsewhere. There is no separate document-slug attribute.
- An entry is a mapping whose top-level keys are `prefix`, `directory`, `enabled`, `label`, `plural`, `order`, `summary`, `type_alias`, `reference_visibility`, `statuses`, `default_status`, `slots`, `api_aliases`, `fields`, and the role blocks `group` and `member`.
- `prefix` and `directory` are required on every entry, including an entry for a seed slug, and are never filled from the seed; `directory` is relative to `doc_directory` and archived records are stored in an `archived/` subfolder of it.
- A single-role type (`SingleMarkdown` or `SingleHtmlMockup`) declares one top-level `fields` sequence and no role block. A grouped type (`GroupedMarkdown`) declares `group` and `member` blocks, each with a `fields` sequence and an optional `reference_visibility`; a top-level `fields` on a grouped type is rejected.
- `fields` is a YAML sequence of field mappings, never a scalar and never a comma-separated list; `fields: []` is valid and declares no field, and a `SingleHtmlMockup` type declares no field.
- A field item has exactly the keys `key`, `heading`, `frontmatter_key`, `option`, `label`, `kind`, `required`, `verbs`, `help`, and `values`. It declares no `order` and no `enabled`, and either key is rejected. `values` is optional: absent means no restriction beyond `kind`; a present `values` is either a non-empty sequence of allowed strings, which restricts a scalar field to those exact values, or the string `record-ids`, which requires every item of a `list` field to be a valid record ID, and it adds no option and no verb.
- A field declares `heading`, `frontmatter_key`, or both; a field with neither is rejected. `kind` is `text`, `multiline`, or `list`, and `verbs` is a subset of `create` and `edit` that defaults to both.
- Sequence position is the field's order everywhere: body sections, CLI option and help order, and browser form order.
- The reserved field keys `id`, `title`, `status`, `labels`, `references`, `created_date`, `updated_date`, `group`, `version`, `kind`, `form`, `path`, and `metadata` are not usable as a field `key` or `frontmatter_key`, and field keys are unique within a role, while the same key may appear in `group.fields` and `member.fields`.
- `model`, `title`, `content`, and `id_shape` are not configuration keys: the model is seed data for a seed slug and `SingleMarkdown` for a user-defined slug, and an entry that declares any of those keys is rejected.
- `statuses`, when present, is a non-empty duplicate-free list that contains `active` and `archived` exactly once; those two values are fixed and every other value is a custom status. Every declared value follows the slug character rule. `default_status`, when present, must be a declared status and must never be `archived`. An entry without `statuses` has only the two main statuses and starts `active` unless `default_status` is set. `statuses` and `default_status` are rejected on a `SingleHtmlMockup` type (`ui-mockup`), which keeps its state in HTML metadata.
- `statuses` is list-valued: `papyrus config set document_types.<slug>.statuses active,archived,review` writes the value as a YAML flow sequence; commas are separators with no escape form, and an empty value is rejected.

### Defaults, fill, and merge

- Configuration is resolved per entry against the seed entry of the same slug.
- Scalars fill from the seed when absent: `enabled`, `label`, `plural`, `order`, `summary`, `type_alias`, `default_status`, and `reference_visibility`. A present scalar overrides the seed, so a file value beats the seed value for every scalar.
- Collections replace the seed collection wholesale when present: `fields`, `group.fields`, `member.fields`, `statuses`, `slots`, and `api_aliases`. An absent collection inherits the seed collection; a present collection replaces it entirely, including `fields: []`, and no seed element is merged into it.
- The `group` and `member` blocks resolve independently: a present block replaces only that role's collections, and an absent block inherits that role's seed collections.
- `prefix` and `directory` are never filled from the seed; every entry declares both, and a missing or invalid value is rejected.
- The eleven seed slugs are always effective: each resolves from its seed entry whether or not the project file declares it, so a present `document_types` map is an overlay and declaring one entry never removes the other seed types. An absent user-defined slug stays absent, and the written file never gains an entry that the command did not target. `enabled: false` keeps the entry, its prefix reservation, and its files while removing its command and browser view.
- A user-defined slug has no seed entry: it defaults to `SingleMarkdown`, `enabled: true`, a slug-derived label, an order after the seed types, an empty summary and empty aliases, visible reference visibility, `statuses: [active, archived]`, no `default_status`, and one optional `description` field with heading `## Description` and kind `multiline`. Declared values replace those defaults under the same scalar-fill and collection-replace rules.
- `papyrus init` seeds eleven entries: `req` (REQ, `requirements/business/`, `GroupedMarkdown`), `nfr` (NFR, `requirements/non-functional/`, `GroupedMarkdown`), `glossary` (GLO, `glossary/`), `rule` (RUL, `business-rules/`), `ui-spec` (UI, `specifications/ui/`), `ui-mockup` (MOCK, `specifications/ui/mockups/`, `SingleHtmlMockup`), `audit` (AUD, `audit-records/`), `tech-spec` (TECH, `technical/`), `use-case` (UC, `use-cases/`), `test-case` (TC, `test-cases/`), and `test-report` (TR, `test-reports/`); every other seed entry is `SingleMarkdown`, and `deploy` is not seeded.
- Default entries are fully ordinary: any entry can be renamed, removed, disabled, or given another prefix or directory, while a model, role field set, or field key is seed data that a project entry does not change.

### Selector paths and configuration commands

- A configuration leaf is addressed by the selector path `document_types.<slug>.<path>`, where `<path>` is a top-level key or a nested path such as `group.fields[story].heading`.
- A field sequence element is addressed by its key, as in `document_types.req.member.fields[story].heading`, or by zero-based index, as in `document_types.req.group.fields[0].required`; both forms resolve to the same element.
- `papyrus config get` and `papyrus config set` accept one selector, and `config list` prints the full effective configuration, seed-filled values included, addressing each field property by its key selector.
- `papyrus config set document_types.<slug>.<path> <value>` materializes only the target entry when the file does not declare it, applies the single change, validates the resulting configuration, and never materializes the whole seed set.
- `papyrus config unset` of a scalar removes the override so the seed value applies again, and unset of a collection restores the seed collection for that slug.
- A field sequence is never written from a comma-separated value: adding, removing, reordering, or changing a field is a configuration file edit.
- The flat collections `statuses`, `api_aliases`, and `slots` keep comma syntax and are stored as YAML flow sequences, with no escape form and no empty value.

### Validation

- Fail-closed: every command except the repair surfaces (`config get|set|unset|list`), `init --force`, and `check` refuses to run while the effective configuration is invalid, and no partially valid document-type set is used.
- A violation is a validation outcome (exit code 4) whose diagnostic names the offending location in the exact form `document_types.<slug>.<path>`; an entry-level violation that has no narrower path uses `document_types.<slug>`, and the same form is used for an unknown selector or an out-of-range index.
- Unknown keys and unknown enum values are rejected with `document_types.<slug>.<path>`: an unknown top-level or field-item key, a `kind` outside `text`, `multiline`, and `list`, a `verbs` entry outside `create` and `edit`, a `reference_visibility` outside `visible` and `hidden`, a field `order` or `enabled` key, and a `model`, `title`, `content`, or `id_shape` key.
- Slug: non-empty, ASCII letters, digits, `-`, and `_`, not starting with `-`, with no whitespace, control character, or path separator; values are rejected, never trimmed or normalized.
- Reserved names: a slug equal to `init`, `config`, `check`, `task`, `milestone`, `release`, `decision`, `search`, `browser`, `import`, `help`, or `version` is rejected. The comparison is case-insensitive, so `Config` and `SEARCH` are rejected too.
- `prefix` is non-empty and unique across all entries, including disabled ones.
- `directory` is relative, stays inside the project directory, and is unique among enabled entries.
- A repeated `document_types` slug key is invalid configuration; neither occurrence is preferred.
- Validation reports every diagnostic in one pass, in deterministic order, with no entry skipped and no entry cleaned or normalized.

### Repair surfaces

- `papyrus config get|set|unset|list` load tolerantly, validate the configuration that would result, and refuse an invalid write.
- `papyrus init --force` replaces the file with the generated default set.
- `papyrus check` loads tolerantly, reports each configuration diagnostic as a `config-invalid` finding, runs only the checks that do not depend on the effective `document_types` set, and repairs no configuration value (REQ-001.008).
