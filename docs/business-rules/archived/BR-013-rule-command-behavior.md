---
id: "BR-013:"
title: "Papyrus rule Command Behavior"
created_date: '2026-11-14T18:39:51Z'
updated_date: '2026-11-20T21:13:34Z'
labels: []
references: []
status: archived
---
# BR-013: Papyrus rule Command Behavior

## Description

`papyrus rule` is the command family generated from the `SingleMarkdown` `rule` descriptor: a create must supply the required `description`, `rationale`, and `applicability` prose sections, `rationale` and `applicability` are create-only and read-only afterwards, and the `requirements` list field is written through the repeatable `--requirement` option.

## Rationale

Based on REQ-005.005 — Manage Business Rules, REQ-007.003 — Dispatch Config-Defined Document Commands, REQ-007.004 — Provide One Schema-Driven Document Content Model with Three Document Models, and REQ-005.007 — Manage Document Archive State and Deletion, this rule exists because a business rule is the document type with the most per-field behaviour: three required prose fields with different verb sets and a list-valued reference field. The generated command must take `required`, `verbs`, `kind`, and `option` from the seed, so a rule can never be created without the rationale and applicability that ground it, can never have them rewritten afterwards, and can carry zero or more requirement references without a per-type option table.

## Applicability

Applies to every invocation of `papyrus rule` and to every business-rule record, in projects where the `rule` entry is enabled.

### Type entry

- `rule` is the seed `SingleMarkdown` entry (prefix `RUL`, directory `business-rules/`); its model and field sequence are seed data and are never declared in configuration, and renaming the entry key renames the command without moving an existing file (BR-016).
- The field sequence is `description` (`## Description`), `rationale` (`## Rationale`), `applicability` (`## Applicability`), and `requirements` (`## Requirements`, kind `list`, option `requirement`), in that order; all four are body-only and declare no front-matter key.
- IDs use the entry prefix and `id_padding` and are preserved on archive.

### Verbs

- The model generates `create`, `list`, `view`, `edit`, `archive`, `delete`, and `status <id> <value>`; no extension verb and no type-specific module exists (BR-001).
- A create must supply `--description`, `--rationale`, and `--applicability`: a missing required field produces one validation outcome that lists the missing keys and writes no file.
- `--description` is offered on create and edit, and an edit that clears it is rejected with no write; `rationale` and `applicability` declare `create` only, so they have no edit option, are absent from edit help, and are rendered read-only, never writable after creation.
- The list field is written only through the repeatable `--requirement <REQ-ID>` option: each value becomes one list item under `## Requirements`, and a rule may reference zero or more business requirements.
- On edit a requirement reference is added through `--requirement` and removed by ID, or all references are cleared at once; the same list is written on create.
- `papyrus rule create <TITLE>` takes the first positional `TITLE`, and `--title` is accepted on create and edit; the title is single-line and non-empty, required on create, never clearable by an edit, and no `--term` option exists.
- `--label` and `--reference` remain the general metadata options, and no create or edit option sets `status`.

### Lists and references

- Reference visibility is `visible`, so a list row renders its reference list; labels render in the Title cell and the single-document view keeps its References disclosure.

### Status and archive

- Rules carry front-matter `status` with the value `active`, `archived`, or a configured custom status; creation writes the configured `default_status` or `active`, and only `status` and `archive` write it.
- `archive` sets `archived`, moves the file into the entry's `archived/` subfolder, keeps the ID, discards any previous custom status value, and has no inverse command.
- `status` never archives: `status <id> archived` is rejected with a validation outcome naming `archive` as the only way, and `status` on an archived record is rejected without a move.
- Deleting a rule permanently removes it with confirmation, with `--force` skipping the prompt, and decision records and the fixed milestone and release lifecycles are separate and unaffected.
