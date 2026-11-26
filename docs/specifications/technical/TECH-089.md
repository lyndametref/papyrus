---
id: TECH-089
title: "Command Dispatch and Document Views"
created_date: '2026-11-25T20:42:10Z'
updated_date: '2026-11-25T20:42:10Z'
labels: []
references:
  - REQ-007.003
  - REQ-005.015
  - REQ-005.017
status: draft
---
# TECH-089 Command Dispatch and Document Views

## Description
Covers the command tree derived from configuration for document types, the verb
set derived from each type's document model, the options and help derived from a
type's field sequence, built-in name collisions, the identifier shorthand for
`view`, and the list and single-record view contract shared by document types.
Browsing and the field schema itself are outside this specification.

## Command tree from configuration

- Every document type declared in the configuration exposes a command named by
  its slug: `papyrus <document-slug>` (REQ-007.003).
- The tree is built from the configuration at startup. Declaring a valid entry
  is enough to obtain the command and its help line; no rebuild and no per-type
  code is involved.
- Each type contributes one command node carrying the verbs of its model, one
  help line, and one entry in the root help listing.
- Every configured document type is listable and viewable from the command line
  and from the browser (REQ-005.017).
- An unknown slug, or a slug that is not configured, is a usage error. Such a
  token never falls back to another command.
- A slug equal to a built-in command name is rejected when the configuration
  loads, so at invocation a slug always resolves to a document type and never
  shadows a built-in command.

## Slug contract

- The slug is the configuration key and the command name at once.
- Command outcomes use the common exit codes.
- Global options `--json`, `--dir`, `--config`, and `--no-color` apply to
  document commands as to any other command.

## Verbs derived from the document model

The verb family is a function of the type's document model. It is never declared
in configuration, in a per-type table, or in a per-type module.

| Verb | Availability |
|---|---|
| `create`, `list`, `view`, `edit`, `archive`, `delete` | every model |
| `status <ID> <VALUE>` | `SingleMarkdown`, `GroupedMarkdown` |
| `create-group`, `--group` on `create` | `GroupedMarkdown` |
| `supersede` | `SingleHtmlMockup` |

- Two types of the same model expose the same verb set; their options differ
  only through their own field sequences.
- No `deprecate` and no `restore` verb exists for documents. `archive` replaces
  `deprecate` and has no inverse.
- `delete` permanently removes a record. An active record is archived first, and
  confirmation is required unless `--force` is given.

## Field-derived options

- `create` and `edit` expose the options generated from the fields of the
  record's role, taken from the type's field sequence.
- Option order and per-option help prose come from the field sequence: field
  label, help text, and position. A field change therefore changes the generated
  options and their help.
- Required and clearing rules, `list` and `multiline` handling, `--editor`,
  `--label`, and `--reference` follow the field contract (REQ-005.015), applied
  to the type's field sequence for the record's role.
- No create, edit, or field option sets `status`.

## Help determinism

- Help output for one configuration is identical across runs, machines, working
  directories, terminal widths, and locales.
- It contains no absolute path, host name, timestamp, process identifier, or
  memory address.
- For a given configuration these are stable: the root about line, the per-type
  line `Manage <slug> documents.`, each type's help line, the type order, and
  each verb's about line.
- Configured types are listed in `order` order, ties broken by slug.
- Per-option help prose and option order are derived from the field sequence, so
  a field change legitimately changes them.
- `papyrus --help` lists every configured slug alongside the built-in commands.
- `papyrus <document-slug> --help` shows the model-derived verbs with their
  generated options.

## Identifier shorthand

- When no verb is given and the next argument is a record identifier of the
  type, `papyrus <document-slug> <ID>` behaves as `view <ID>`.
- A token that is neither a verb of the type nor a record identifier of the type
  is a usage error.

## List contract

- A list shows one row per record.
- A row carries the record title and its labels in the title cell.
- A row carries the type's declared list slots.
- A row carries the record's reference list only when the type's reference
  visibility is `visible`. When it is `hidden` the row carries no reference
  list, while labels still render.
- Slot content and the reference-visibility value come from the type's
  configuration entry.
- Archived records appear in no list.
- Shared list chrome, layout, and action placement are outside this
  specification.

## GroupedMarkdown lists

- A list of a `GroupedMarkdown` type shows its group records and their members
  merged into one sequence.
- Order is group order first, then member order within each group.
- Member rows are read-only in the list.

## Single-record view

- A view renders one section per declared field, in field-sequence order.
- The section heading is the field's label.
- A field with no value contributes no section. A required field with no stored
  value is rendered as an empty section.
- Field content is Markdown rendered with the console's advanced formatting, so
  headings, emphasis, lists, tables, code, and quotes are formatted for the
  terminal rather than printed as raw source.
- A `SingleHtmlMockup` view renders the fixed metadata block instead of field
  sections.
- The References disclosure is present on the single-record view of every model,
  whatever the reference visibility of its type.

## Output modes

- `--no-color` disables the console styling; the content stays readable.
- `--json` output carries the field values and no console formatting.

## Unavailable records

- A record that is unknown, archived, or of a type that is not configured is not
  viewable.
- No command and no browser route returns such a record.
- The browser shows the shared not-found page for it.
