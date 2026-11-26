---
id: TECH-083
title: "Record Metadata, Identity, and Field Map"
created_date: '2026-11-25T20:42:00Z'
updated_date: '2026-11-25T20:42:07Z'
labels: []
references:
  - REQ-001.005
  - REQ-005.010
  - REQ-005.011
  - REQ-006.004
status: draft
---
# TECH-083 Record Metadata, Identity, and Field Map

## Description
Covers record identifier assignment and shape, the YAML front-matter contract of managed Markdown records, metadata maintenance performed by commands, and the parity of metadata values between the command line and the browser. It does not cover persistence and archive layout, or the field schema declared in the configuration.

## Identifier Assignment

- Papyrus generates record identifiers automatically; creating any managed record assigns one.
- An identifier never changes when a record moves between lifecycle states.
- A task keeps its identifier across archive and restore.
- A document, a milestone, and a release keep their identifier when archived.
- The display identifier and the stored identifier are the same value.

## Identifier Shape

- A document record's identifier shape follows its type's configured document model (REQ-007.004).
- A `SingleMarkdown` or `SingleHtmlMockup` record has one plain identifier `<PREFIX>-<NUMBER>`.
- A `GroupedMarkdown` group has a plain identifier `<PREFIX>-<GROUP>`.
- A member of a `GroupedMarkdown` group has a dotted identifier `<PREFIX>-<GROUP>.<MEMBER>` under the same type prefix.
- `<PREFIX>` is the type's configured `prefix`.
- A newly configured `GroupedMarkdown` type produces grouped identifiers, and a newly configured `SingleMarkdown` type produces plain identifiers.

## Identifier Padding

- The shared `id_padding` determines the generated numeric width for tasks, milestones, releases, and every document type.
- `id_padding` is a whole number from 1 to 6 inclusive and defaults to 3.
- A value outside that range is rejected during configuration validation, with a message that names the permitted range.
- Padding is a minimum, never a maximum: when a number's digit count exceeds `id_padding`, the wider number is used, never truncated, and never an error.
- Existing files are not re-padded, and no migration is performed when `id_padding` changes.
- Existing identifiers are never truncated or migrated.

## Single Identifier Space

- The Papyrus identifier is the record identity everywhere: in the front-matter `id`, in command input, and in command-line and browser output.
- Papyrus keeps no second, tool-specific storage identifier and does not translate between identifier spaces.
- Records brought in by the one-way import receive Papyrus identifiers, and no external identifier is accepted as command input.

## Front-Matter Block

- Every Markdown record that Papyrus manages begins with a YAML front-matter block at byte 0, delimited by `---` lines.
- Front matter is authoritative for the keys it carries and replaces the equivalent information in the record body for records Papyrus writes.
- Records of a `SingleHtmlMockup` type carry no front matter; their metadata is kept in their HTML metadata comment.
- Prose Markdown that no document type claims is not managed and is out of scope.

## General Keys

- Every managed Markdown record carries `id`, `title`, `status`, `labels`, `references`, `created_date`, and `updated_date`.
- A member record of a `GroupedMarkdown` type also carries `group`.
- Papyrus writes and maintains these keys.

## Reserved Keys

- The reserved keys are `id`, `title`, `status`, `labels`, `references`, `created_date`, `updated_date`, `group`, `version`, `kind`, `form`, `path`, and `metadata`.
- No document type declares a reserved key, and no field uses one as its key or as its front-matter key; such a declaration is rejected.
- `title` is fixed and is not a field.
- `status` is written only by the verbs that own it.
- `group` is the engine-owned link of a member record.

## Document Keys

- Beyond the general keys, a record's front-matter keys are exactly the front-matter keys of its type's declared fields for the record's role.
- A field with only a heading contributes no front-matter key, and its value lives in the body.
- A field with only a front-matter key is metadata-only and has no body section.
- A field with both stores the same submitted value in both locations.
- A read displays the body section and does not merge the two locations; a divergence already on disk is left as stored.
- A front-matter key may differ from the field key.
- Document keys follow field-sequence order after the general keys.
- An undeclared key already present on disk is preserved as stored and displayed read-only.

## Serialization

- `created_date` and `updated_date` are quoted ISO-8601 UTC strings of the form `'YYYY-MM-DDTHH:MM:SSZ'`.
- The `id` value and every reference value use the record identifier; no second identifier space exists.

## Body Title Line

- A document record keeps a body title line generated from the front-matter `id` and `title`, so the two cannot diverge.
- Backlog records follow the record layout of their own category.

## Automatic Values

- `created_date` is set when a record is created and never changes afterwards.
- `updated_date` is set at creation and refreshed by every write, including a browser save.
- For a decision record, `updated_date` is refreshed only when the status changes.
- `id`, `created_date`, and `updated_date` have no command option.
- For a document record, `status` has no create or edit option; a new record starts at its type's `default_status`, or at `active` when none is configured, and changes only through the `status` and `archive` verbs.
- The status of a task, milestone, release, or decision keeps the behaviour of its own category.

## Field-Derived Options

- For a document record, the remaining options are the declared fields of the type's field sequence for the record's role.
- Each field contributes one option on the verbs the field declares, named by its `option` or by its `key` in kebab case when no option is declared.
- A field whose verbs exclude `edit` has no edit option and is never writable after creation; a field whose verbs exclude `create` has no create option.
- The field sequence is the option and help order, including the role-appropriate sequence of a `GroupedMarkdown` type.
- `title` is not a field: it is the first positional `TITLE` on `create` and `create-group`, `--title` is accepted on `create`, `create-group`, and `edit`, and an edit can never clear it.
- The `group` link of a member record has no option.
- `--label` and `--reference` are the general repeatable metadata options and are not fields.

## Write Contract

- A create and an edit each produce one patch of operations.
- A single writer applies the patch atomically.
- A patch that changes nothing performs no write, and a validation failure writes nothing.
- The browser submits its edits through the same write path.
- Every write preserves the front-matter keys and body content the operation does not change, including an undeclared front-matter key already on disk, which is never repaired, merged, or removed.
- This preservation covers create, edit, status, archive, delete, supersede, milestone close, and release close.

## Adoption and Import

- An existing managed Markdown record that does not yet carry a front-matter block gains one on its first read and on its first write.
- Adoption is idempotent and requires no bulk migration command.
- Adoption covers only files claimed by a configured document type or by another managed record category; prose Markdown and HTML records are excluded.
- Adoption changes nothing in the body, except that a value still held in a legacy body section moves to front matter when its record is written, so no value is stored twice.
- Records brought in by the one-way import are written in this format.

## Backlog Ordinal

- `ordinal` belongs to backlog records only.
- It is an opaque non-negative integer supplied through a create or update option and defaulted from the current maximum.
- No document type declares an `ordinal` field.

## CLI and Browser Parity

- The command line in plain text and JSON and the browser in list and single-record views display the same metadata values for the same record.
- Both surfaces read the record's stored metadata and project the declared fields of the record's type and role; neither derives a value from the body when the metadata defines it.
- Neither surface renames a stored value for display.
- Both surfaces show at least the identifier, the title, the created and updated dates, the labels, the references, and every declared field of the record's type and role.
- A record's `status` is shown as stored, including a custom status declared for its type, and is identical in the command line and the browser.

## Browser Edit Form

- The form offers the fixed title control, one control per declared field whose verbs allow edit, the `labels` and `references` controls, and read-only `status`, created date, and updated date.
- `title` is not edited through a field control, and `status` is not editable.
- A save writes each submitted value to the storage locations its field declares and preserves the metadata and content the save does not change.
- For a `SingleHtmlMockup` record, metadata is read from and written to its HTML metadata comment, and the same parity applies.
