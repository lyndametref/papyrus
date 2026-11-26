---
id: TECH-086
title: "Artifact Categories, Document Types, and Configuration Validation"
created_date: '2026-11-25T20:42:10Z'
updated_date: '2026-11-25T20:42:10Z'
labels: []
references:
  - REQ-007
  - REQ-007.001
  - REQ-007.002
  - REQ-005.008
status: draft
---
# TECH-086 Artifact Categories, Document Types, and Configuration Validation

## Description
Covers the three artifact categories, their lifecycles, the `document_types`
map, its entry keys and defaults, and its validation and diagnostics. Excludes
content models, field semantics, the Seed, and command dispatch.

## Three Artifact Categories

- `backlog` — tasks, milestones, and releases.
- `decision` — decision records.
- `document` — every document type declared in the configuration.
- Each record type belongs to exactly one category, and the category owns its
  records' lifecycle.

## Lifecycles

- **Task.** `statuses` and `default_status` are configurable.
- **Milestone, release.** Fixed: `open` -> `closed` -> `archived`;
  `planned` -> `released` -> `archived`.
- **Decision.** Fixed: `proposed`, `accepted`, `rejected`, `deprecated`,
  `superseded`; archiving moves the file.
- Configuration cannot add, remove, or rename a category, change a fixed
  lifecycle, change the document storage state, or change the meaning of
  `active`.

## Document Storage State

- A document record is archived exactly when it carries `status: archived` and
  its file is in the `archive/` subfolder of its type's directory.
- Otherwise it is not archived: its file is in the type directory and its
  status is `active` or a declared custom workflow status.
- `archive` writes `archived` for a record that is not superseded; `supersede`
  also archives only the mockup it supersedes (REQ-005.008).
- Both decisions and documents archive by moving the file into `archive/`.
  A decision keeps its status; a document's becomes `archived` and is never
  `deprecated`.

## Custom Workflow Statuses

- A document type may declare custom statuses in `statuses`; such a status is
  a front-matter value only. It moves no file, defines no per-status folder,
  introduces no lifecycle, and changes nothing about listing, archiving, or
  deleting.
- A record carrying a custom status stays a non-archived record in the type
  directory, remains listed, and remains archivable and deletable.
- A new record starts at the type's `default_status` when one is configured,
  otherwise at `active`.

## Status List Invariants

- `active` is always available, describes any non-archived state, and is never
  declared in `statuses`.
- `archived` is reserved for the storage state and is never a declared status.
- An empty `statuses` list is valid and leaves `active` as the only workflow
  status.
- Custom statuses exist only for front-matter-backed document types.
- A `SingleHtmlMockup` type uses fixed HTML-metadata states `pending`,
  `approved`, and `archived`, declares no custom status, and has no `status`
  verb.

## The `status` Verb

- `papyrus <document-slug> status <ID> <VALUE>` sets the front-matter status to
  `active` or to a declared custom status, changes nothing else, and refreshes
  `updated_date`.
- It never archives. The value `archived` is rejected with a validation outcome
  naming `archive` as the only way to archive, and no file moves.
- An archived record is rejected with a validation outcome naming it as
  archived; nothing changes and nothing is restored.
- No create, edit, or field option sets a status.

## The `document_types` Map

- `document_types` is a map keyed by slug: the slug is the configuration key,
  the CLI command name, and the type identifier.
- A document type exists if and only if the configuration declares it. An
  absent slug is absent from commands, views, and search, and no built-in table
  is consulted after `init`.
- A violating configuration blocks every command with a validation outcome,
  except `config get|set|unset|list`, `init --force`, and `check`.

## Entry Keys

An entry is a mapping that accepts these keys:

| Key | Meaning |
|---|---|
| `model` | `SingleMarkdown`, `GroupedMarkdown`, or `SingleHtmlMockup` |
| `prefix`, `directory` | required prefix and type directory, the latter relative to `doc_directory` |
| `label`, `plural`, `order`, `summary`, `type_alias`, `reference_visibility` | presentation facts; visibility is `visible` or `hidden` |
| `statuses`, `default_status` | custom workflow statuses and start status |
| `slots`, `api_aliases` | declared lists |
| `fields` | field sequence or empty list |
| `group`, `member` | role blocks for `GroupedMarkdown` |
| `field_defaults` | field key to value written on create when absent |

`title`, `content`, `id_shape`, and `enabled` are not keys; presence is the
only switch.

## Model Shape

- `SingleMarkdown` declares a top-level `fields` sequence and no role block.
- `GroupedMarkdown` declares `group` and `member` blocks, each with a field
  sequence and an optional `reference_visibility`, and no top-level `fields`.
- `SingleHtmlMockup` declares `fields: []`, no role block, and no `statuses` or
  `default_status`.

## Defaults

- An omitted `model` is `SingleMarkdown`, any configured type may declare any
  model, and an omitted attribute takes a model-derived default, never a value
  from a built-in table.
- Defaults are: no statuses, no `default_status`, visible reference
  visibility, a slug-derived label and plural, an `order` after the entries
  that declare one, and empty summary, aliases, slots, and field defaults.
- A `SingleMarkdown` entry that omits `fields` declares one optional
  `description` field under the heading `## Description` with kind
  `multiline`; `fields: []` declares no field, and a role block that omits
  `fields` has an empty sequence.

## Unknown Attributes

- An unknown configuration, entry, or field-item attribute is accepted,
  ignored with one warning at load, reported by `papyrus check`, and never
  changes behaviour. Declaring `title`, `content`, `id_shape`, or `enabled`
  warns and has no effect.

## Fail-Closed Validation

Validation reports every diagnostic in one deterministic pass and never runs a
command on a partially valid set.

**Slug.** Rejected: empty; any character other than an ASCII letter, digit,
`-`, or `_`; a leading `-`; whitespace, a control character, or a path
separator; and any slug equal, case-insensitively, to `init`, `config`,
`check`, `task`, `milestone`, `release`, `search`, `browser`, `import`, `help`,
or `version`.

**Required and duplicated values.** Rejected: a missing `prefix`; a `prefix`
repeated across entries; a missing or absolute `directory`; a `directory`
outside the project directory; a repeated directory; a repeated slug key.

**Model and field items.** Rejected: a model outside the three; a model shape
violation; a field with an unknown `kind` or `verb`, with neither `heading` nor
`frontmatter_key`, with a reserved key, or with a duplicate key in its role; an
empty `values` restriction, or one that is not `record-ids` on a `list` field; a
`field_defaults` key naming no declared field.

Reserved keys for field keys and front-matter keys: `id`, `title`, `status`,
`labels`, `references`, `created_date`, `updated_date`, `group`, `version`,
`kind`, `form`, `path`, `metadata`.

**Statuses.** Rejected: a `statuses` list with duplicates, with `active` or
`archived`, or with an invalid name; a `default_status` that is `archived` or
neither `active` nor declared; any `statuses` or `default_status` on a
`SingleHtmlMockup`.

**Location.** A diagnostic names `document_types.<slug>.<path>`, with field
paths in keyed `member.fields[story].heading` or indexed
`group.fields[0].required` form.

**Writes.** A configuration change is validated as the resulting configuration
before it is written, and an invalid value is refused with nothing written.
`config get|set|unset|list` address leaves by the same selector, and an unknown
selector or out-of-range index is a diagnostic.

## Adding, Removing, and Renaming

- Adding an entry adds the type, its command family, and its browser views.
- Removing an entry leaves its former files untouched and visible nowhere;
  `papyrus check` reports every managed-front-matter file of no configured
  type.
- Renaming a slug moves no file and rewrites no identifier; former records
  become unmanaged unless moved, and no former name remains an alias.

## Configuration Is Data

Configuration declares models, fields, and presentation facts as values and
carries no template, expression, script, or executable path.
