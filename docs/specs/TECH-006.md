---
id: TECH-006
title: "Record Categories, Record Types, and Configuration Validation"
created_date: '2026-11-27T20:35:36Z'
updated_date: '2026-11-27T20:35:36Z'
labels: []
references:
  - REQ-007
  - REQ-007.001
  - REQ-007.002
  - REQ-005.008
  - REQ-004.001
status: verified
---
# TECH-006 Record Categories, Record Types, and Configuration Validation

## Description
Covers the three record categories, their statuses and lifecycles, the `document_types` and `decision_types` maps, entry keys and defaults, and validation and diagnostics. Excludes record storage and folder layout, archive and delete mechanics, content models, field semantics, the Seed, and command dispatch.

## Three Record Categories

- `backlog` — tasks, milestones, and releases.
- `decision` — every decision type declared in the configuration.
- `document` — every document type declared in the configuration.
- Each record type belongs to exactly one category, and the category owns its records' lifecycle.

## Lifecycles

- **Task.** `statuses` and `default_status` are configurable. A new task starts at the configured default task status. The final task status is the last entry of the configured list, and a task is unfinished when its status is not that final status.
- **Milestone, release.** Fixed: `open` -> `closed`, with a new milestone starting `open`; `planned` -> `released`, with a new release starting `planned`. Archived is a file position, not a status value.
- **Decision.** Fixed statuses: `proposed`, `accepted`, `rejected`, `deprecated`, `superseded`. Accepted transitions: `proposed` -> `accepted`, `proposed` -> `rejected`, `accepted` -> `deprecated`, `accepted` -> `superseded`. Archiving moves the file and keeps the status.
- Configuration cannot add, remove, or rename a category, change a fixed lifecycle, change the document storage state, or change the meaning of `active` or `archived`.


## Decision Status Transitions

- The decision status vocabulary is fixed, shared by every decision type, and never declared in configuration: `proposed`, `accepted`, `rejected`, `deprecated`, `superseded`.
- A decision status moves only through the status verbs of its own command family, and no edit option sets it.

| Command | Transition |
|---|---|
| `papyrus <slug> accept <ID>` | `proposed` to `accepted` |
| `papyrus <slug> reject <ID>` | `proposed` to `rejected` |
| `papyrus <slug> deprecate <ID>` | `accepted` to `deprecated` |
| `papyrus <slug> supersede <ID> --by <SUCCESSOR-ID>` | `accepted` to `superseded` |

- A verb applied from a status other than its required starting status is rejected with a validation outcome and writes nothing.
- A decision cannot be accepted or rejected twice, and a rejected decision cannot be accepted or deprecated.
- A decision is superseded by exactly one successor of the same decision type.
- `supersede` writes the successor identifier into the front-matter `superseded_by` key and sets the status to `superseded`.
- A successor of a different type, an unknown successor, or a second supersession is rejected with a validation outcome and writes nothing.

## Custom Workflow Statuses

- A document type may declare custom statuses in `statuses`; such a status is a front-matter value only. It moves no file, defines no per-status folder, introduces no lifecycle, and changes nothing about listing, archiving, or deleting.
- A record carrying a custom status stays a non-archived record in the type directory, remains listed, and remains archivable and deletable.
- A new record starts at the type's `default_status` when one is configured, otherwise at `active`.

## Status List Invariants

- `active` is always available, describes any non-archived state, and is never declared in `statuses`.
- `archived` is not a status value and is never a declared status.
- An empty `statuses` list is valid and leaves `active` as the only workflow status.
- Custom statuses exist only for front-matter-backed document types.
- A `formatted-record` type uses fixed HTML-metadata states `draft` and `approved`, declares no custom status, and has no `status` verb; archiving moves the file and changes no state.

## The `status` Verb

- `papyrus <document-slug> status <ID> <VALUE>` sets the front-matter status to `<VALUE>`, a declared custom status, changes nothing else, and refreshes `updated_date`.
- It never archives. The value `archived` is rejected as a status with a validation outcome, and no file moves.
- An archived record is rejected with a validation outcome naming it as archived; nothing changes and nothing is restored.
- No field option sets a status; a `formatted-record` state is set only by `edit --status approved`.

## The `document_types` Map

- `document_types` is a map keyed by slug: the slug is the configuration key, the CLI command name, and the type identifier.
- A document type exists if and only if the configuration declares it. An absent slug is absent from commands, views, and search, and no built-in table is consulted after `init`.
- A violating configuration blocks every command with a validation outcome, except `config get|set|unset|list`, `init --force`, and `check`.


## The `decision_types` Map

- `decision_types` is a map keyed by slug: the slug is the configuration key, the CLI command name, and the type identifier.
- Each entry declares a `prefix` and a `directory`; presence in the configuration is what makes the type available, and there is no `enabled` key.
- Every decision type uses the shared fixed status vocabulary; a status list is not part of the entry.
- A decision type exists if and only if the configuration declares it.
- `init` writes the selected decision-type Seed entries fully expanded (REQ-004.001).

| Change | Outcome |
|---|---|
| Add a type while at least two types are configured | Accepted; the new type receives its own subfolder and no existing record moves |
| Add the second type to a project with exactly one type | Rejected with a validation outcome; no file moves |
| Remove a configured type | Rejected with a validation outcome; no file moves |
| Change the prefix of a type that still has decisions | Rejected with a validation outcome; no file moves |

## Entry Keys

An entry is a mapping that accepts these keys:

| Key | Meaning |
|---|---|
| `model` | record model: `simple-record`, `grouped-record`, or `formatted-record` |
| `prefix`, `directory` | required prefix and record type directory; the directory is relative to the category root (`doc_directory` for document types, `decision_directory` for decision types) |
| `record_name` | singular display name used by navigation and headings |
| `plural` | plural display name used by list and section headings |
| `summary` | short note shown with the type in the inventory |
| `statuses`, `default_status` | custom workflow statuses and start status |
| `slots` | ordered list of slot names rendered in the shared list row for this type |
| `fields` | field sequence or empty list |
| `field_defaults` | field key to value written on create when absent |
| `group`, `member` | role blocks for `grouped-record` |

Each slot is one extra list column between Status and Created; its key names the record property it renders. Structural keys and reserved record keys cannot be slots, and a slot cannot repeat.

The order in which entries appear in the configuration is the navigation order. The slug is the only type path segment and API path name; there is no `type_alias` or `api_aliases` key.

## Model Shape

- `simple-record` declares a top-level `fields` sequence and no role block.
- `grouped-record` declares `group` and `member` blocks, each with a field sequence, and no top-level `fields`.
- `formatted-record` declares `fields: []`, no role block, and no `statuses` or `default_status`.

## Defaults

- An omitted `model` is `simple-record`, any configured type may declare any model, and an omitted attribute takes a model-derived default, never a value from a built-in table.
- Defaults are: no statuses, no `default_status`, a slug-derived `record_name` and plural, and empty summary, slots, and field defaults.
- A `simple-record` entry that omits `fields` declares one optional `content` field in the body of the record with kind `multiline`; `fields: []` declares no field, and a role block that omits `fields` has an empty sequence.

## Unknown Attributes

- An unknown configuration, entry, or field-item attribute is accepted, ignored with one warning at load, reported by `papyrus check`, and never changes behaviour. 

## Fail-Closed Configuration Validation

Configuration validation reports every diagnostic in one deterministic pass and never runs a command on a partially valid set.

**Slug.** Rejected: empty; any character other than an ASCII letter, digit, `-`, or `_`; a leading `-`; whitespace, a control character, or a path separator; and any slug equal, case-insensitively, to `init`, `config`, `check`, `task`, `milestone`, `release`, `search`, `browser`, `import`, `help`, or `version`.

**Required and duplicated values.** Rejected: a missing `prefix`; a `prefix` repeated across entries; a missing or absolute `directory`; a `directory` outside the project directory; a repeated directory; a repeated slug key.

**Model and field items.** Rejected: a model outside the three; a model shape violation; a field with an unknown `kind` or `verb`, with a reserved key, or with a duplicate key in its role; an empty `values` restriction, or one that is not `record-ids` on a `list` field; a `field_defaults` key naming no declared field.

Reserved keys for field keys and front-matter keys: `id`, `title`, `status`, `labels`, `references`, `created_date`, `updated_date`, `group`, `version`, `kind`, `form`, `path`, `metadata`.

**Statuses.** Rejected: a `statuses` list with duplicates, with `active` or `archived`, or with an invalid name; a `default_status` that is `archived` or neither `active` nor declared; any `statuses` or `default_status` on a `formatted-record`.

**Location.** A diagnostic names `document_types.<slug>.<path>`, with field paths in keyed `<role>.fields[<key>].heading` or indexed `<role>.fields[<index>].required` form. `<key>` is any field key the type declares; no field key is reserved by the selector grammar.

**Writes.** A configuration change is validated as the resulting configuration before it is written, and an invalid value is refused with nothing written. `config get|set|unset|list` address leaves by the same selector, and an unknown selector or out-of-range index is a diagnostic.

## Adding, Removing, and Renaming

- Adding an entry adds the type, its command family, and its browser views.
- Removing an entry leaves its former files untouched and visible nowhere; `papyrus check` reports every managed-front-matter file of no configured type.
- Renaming a slug moves no file and rewrites no identifier; former records become unmanaged unless moved, and no former name remains an alias.

## Configuration Is Data

Configuration declares models, fields, and presentation facts as values and carries no template, expression, script, or executable path.
