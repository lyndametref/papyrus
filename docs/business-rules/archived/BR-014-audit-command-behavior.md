---
id: "BR-014:"
title: "Papyrus audit Command Behavior"
created_date: '2026-11-14T18:39:51Z'
updated_date: '2026-11-20T21:13:34Z'
labels: []
references: []
status: archived
---
# BR-014: Papyrus audit Command Behavior

## Description

`papyrus audit` is the command family generated from the `SingleMarkdown` `audit` descriptor: the title is the positional `TITLE`, the `audit_type` field is offered as `--audit-type` and stored both under `## Audit Type` and the front-matter key `type`, and the required `content` field is stored under `## Content`.

## Rationale

Based on REQ-005.006 — Manage Audit Records, REQ-007.004 — Provide One Schema-Driven Document Content Model with Three Document Models, REQ-005.010 — Record Metadata in YAML Front Matter, and REQ-007.003 — Dispatch Config-Defined Document Commands, this rule exists because `audit` is the seed type where the field key, the CLI option, and the storage key all differ: `audit_type` is offered as `--audit-type` and kept in front matter as `type` beside its `## Audit Type` section. The generated command must take the option name, the heading, and the `frontmatter_key` from the descriptor rather than a per-type module, or the `type` key, the body section, and the required content would drift apart once the type becomes data; the positional title is added to audit additively, without renaming or removing its existing `--title`.

## Applicability

Applies to every invocation of `papyrus audit` and to every audit record, in projects where the `audit` entry is enabled.

### Type entry

- `audit` is the seed `SingleMarkdown` entry (prefix `AUD`, directory `audit-records/`); its model and field sequence are seed data and are never declared in configuration, and renaming the entry key renames the command without moving an existing file (BR-016).
- The field sequence is `audit_type` (heading `## Audit Type`, `frontmatter_key: type`, option `audit-type`, kind `text`) then `content` (heading `## Content`, required, no front-matter key), in that order.
- IDs use the entry prefix and `id_padding` and are preserved on archive.

### Verbs

- The model generates `create`, `list`, `view`, `edit`, `archive`, `delete`, and `status <id> <value>`; no extension verb and no type-specific module exists (BR-001).
- `--audit-type` and `--content` are the field options in field-sequence order; `--label` and `--reference` remain the general metadata options, and no create or edit option sets `status`.
- `content` is required: a create that omits it, or an edit that clears it, is rejected with one validation outcome and writes no file.
- A write of `audit_type` sets both the `## Audit Type` section and the front-matter `type` key from the same value; a read displays the body section without merging the two locations, and a divergence already on disk is left as stored.
- `papyrus audit create <TITLE> --content <TEXT>` takes the positional `TITLE`; `--title` keeps its name and meaning and is accepted on create and edit; the title is single-line and non-empty, required on create, never clearable by an edit, and no `--term` option exists.

### Audit types

- The built-in audit types are `security`, used when `--audit-type` is omitted, `licence`, and `architecture`; additional audit types are configurable.
- The audit type is record data in the `audit_type` field, not a document type, a status, or a configuration key of its own.

### Lists and references

- Reference visibility is `visible`, so a list row renders its reference list; labels render in the Title cell and the single-document view keeps its References disclosure.

### Status and archive

- Audit records carry front-matter `status` with the value `active`, `archived`, or a configured custom status; creation writes the configured `default_status` or `active`, and only `status` and `archive` write it.
- `archive` sets `archived`, moves the file into the entry's `archived/` subfolder, keeps the ID, discards any previous custom status value, and has no inverse command.
- `status` never archives: `status <id> archived` is rejected with a validation outcome naming `archive` as the only way, and `status` on an archived record is rejected without a move.
- Deleting a record permanently removes it with confirmation, with `--force` skipping the prompt, and decision records and the fixed milestone and release lifecycles are separate and unaffected.
