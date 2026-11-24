---
id: "BR-012:"
title: "Papyrus ui-spec Command Behavior"
created_date: '2026-11-14T18:39:51Z'
updated_date: '2026-11-20T21:13:34Z'
labels: []
references: []
status: archived
---
# BR-012: Papyrus ui-spec Command Behavior

## Description

`papyrus ui-spec` is the command family generated from the `SingleMarkdown` `ui-spec` descriptor: records use the baseline verbs plus `status`, the title is the positional `TITLE` required on create, and the two optional body fields, `view` under `## View` and `description` under `## Description`, come from the seed in that sequence order.

## Rationale

Based on REQ-005.004 — Manage UI Specifications, REQ-007.004 — Provide One Schema-Driven Document Content Model with Three Document Models, REQ-007.002 — Configure Document Types and Slugs, and REQ-005.007 — Manage Document Archive State and Deletion, this rule exists because a UI specification is the plainest generated document type: it adds a view association and nothing else, so its whole command surface — verbs, field options, help order, storage headings, and list rendering — must be reconstructed from the seed descriptor, and it must keep accepting a create that supplies only a title, without gaining a verb, a lifecycle, or a type-specific module of its own.

## Applicability

Applies to every invocation of `papyrus ui-spec` and to every UI-specification record, in projects where the `ui-spec` entry is enabled. UI mockups are a separate `SingleHtmlMockup` entry (`ui-mockup`) with `supersede` and no `status` verb and are not covered by this rule.

### Type entry

- `ui-spec` is the seed `SingleMarkdown` entry (prefix `UI`, directory `specifications/ui/`); its model and field sequence are seed data and are never declared in configuration, and renaming the entry key renames the command without moving an existing file (BR-016).
- The type declares exactly two fields, `view` under the heading `## View` and `description` under the heading `## Description`, in that order; both are optional and body-only and declare no front-matter key, so neither value appears in YAML front matter.
- IDs use the entry prefix and `id_padding` and are preserved on archive.

### Verbs

- The model generates `create`, `list`, `view`, `edit`, `archive`, `delete`, and `status <id> <value>`; no extension verb and no type-specific module exists (BR-001).
- `--view` and `--description` are the field options in field-sequence order, and a `multiline` field accepts `-` to read the value from standard input; `--label` and `--reference` remain the general metadata options.
- `papyrus ui-spec create <TITLE>` takes the first positional `TITLE`, and `--title` is accepted on create and edit; the title is single-line and non-empty, required on create, never clearable by an edit, and no `--term` option exists.
- A create that supplies only a title is valid, and an edit may supply or clear either field.

### Lists and references

- Reference visibility is `visible`, so a list row renders its reference list; labels render in the Title cell and the single-document view keeps its References disclosure.

### Status and archive

- Specifications carry front-matter `status` with the value `active`, `archived`, or a configured custom status; creation writes the configured `default_status` or `active`, and only `status` and `archive` write it, so no create, edit, or field option sets it.
- `archive` sets `archived`, moves the file into the entry's `archived/` subfolder, keeps the ID, discards any previous custom status value, and has no inverse command.
- `status` never archives: `status <id> archived` is rejected with a validation outcome naming `archive` as the only way, and `status` on an archived record is rejected without a move.
- Deleting a specification permanently removes it with confirmation, with `--force` skipping the prompt, and decision records and the fixed milestone and release lifecycles are separate and unaffected.
