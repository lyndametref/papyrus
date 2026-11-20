---
id: "BR-017:"
title: "Papyrus tech-spec Command Behavior"
created_date: '2026-11-14T18:39:51Z'
updated_date: '2026-11-20T21:22:05Z'
labels: []
references: []
status: active
---
# BR-017: Papyrus tech-spec Command Behavior

## Description

`papyrus tech-spec` is the command family generated from the `SingleMarkdown` `tech-spec` descriptor: records use the baseline verbs plus `status` with no type-specific verb, the title is the first positional `TITLE` on create and `--title` on edit, and the seed declares exactly one optional body field, `description` under `## Description`, in that sequence.

## Rationale

Based on REQ-05.009 — Manage Technical Specifications, REQ-07.004 — Provide One Schema-Driven Document Content Model with Three Document Models, REQ-07.002 — Configure Document Types and Slugs, and REQ-07.003 — Dispatch Config-Defined Document Commands, this rule exists because `tech-spec` is the minimal typed document — a title plus one optional description — and that shape must survive the deletion of `src/tech_spec.rs` as seed data. The command family, the `--description` option, the `## Description` heading, and the status model must all be reconstructed from the descriptor, with no per-type module, verb table, or help text. The exact field list is recorded here because the seed is the only definition of a built-in type: changing a field is a requirement or ADR change, and a project must be able to add a field by editing its configuration file alone, without a code change (RE-010 D14).

## Applicability

Applies to every invocation of `papyrus tech-spec` and to every technical-specification record, in projects where the `tech-spec` entry is enabled.

### Type entry

- `tech-spec` is the seed `SingleMarkdown` entry (prefix `TECH`, directory `technical/`); its model and field sequence are seed data and are never declared in configuration, and renaming the entry key renames the command without moving an existing file (BR-016).
- The seed declares exactly the fields in this table, in this sequence; the body headings and field order below are the existing ones and are preserved.

| key | heading | kind | required | verbs | option | frontmatter_key |
|---|---|---|---|---|---|---|
| `description` | `## Description` | `multiline` | no | `create`, `edit` | `description` | none |

- `description` is optional and body-only: a create that supplies only a title succeeds, an edit may supply or clear it, and a field's front-matter key is its `frontmatter_key`; none of these fields declares one, so no field value appears in YAML front matter.
- IDs use the entry prefix and `id_padding` and are preserved on archive.

### Verbs

- The model generates `create`, `list`, `view`, `edit`, `archive`, `delete`, and `status <id> <value>`; no extension verb, no type-specific verb, and no type-specific module exists (BR-001).
- `--description` is the field option in field-sequence order, its `multiline` kind accepts `-` to read the value from standard input, and `--editor` edits the body; `--label` and `--reference` remain the general metadata options, and no create or edit option sets `status`.
- `papyrus tech-spec create <TITLE>` takes the first positional `TITLE`, and `--title` is accepted on create and edit; the title is single-line and non-empty, required on create, never clearable by an edit, and supplying both the positional `TITLE` and `--title` is rejected and writes nothing.

### User extensibility and seed contract

- Adding a field is a configuration file edit: a project adds an entry to the type's `fields` sequence, and Papyrus generates the option, help, body section, and browser form input from it with no code change, rebuild, or per-type module (RE-010 D14, REQ-07.002).
- The field list above is the seed contract for `tech-spec`; changing a field key, heading, kind, required flag, verbs, option, or front-matter key is a requirement change (ADR-036 decision 30), never a silent behavior change.

### Lists and references

- Reference visibility is `visible`, so a list row renders its reference list; labels render in the Title cell and the single-document view keeps its References disclosure.
- Technical specifications may reference non-functional requirements (REQ-02.001).

### Status and archive

- Specifications carry front-matter `status` with the value `active`, `archived`, or a configured custom status; creation writes the configured `default_status` or `active`, and only `status` and `archive` write it, so no create, edit, or field option sets it.
- `archive` sets `archived`, moves the file into the entry's `archived/` subfolder, keeps the ID, discards any previous custom status value, and has no inverse command.
- `status` never archives: `status <id> archived` is rejected with a validation outcome naming `archive` as the only way, and `status` on an archived record is rejected without a move.
- Deleting a specification permanently removes it with confirmation, with `--force` skipping the prompt, and decision records and the fixed milestone and release lifecycles are separate and unaffected.
- Technical specifications follow the same input sanitization and project-directory path boundary as every other document type.

## Requirements

- REQ-05.009
- REQ-07.004
- REQ-07.002
- REQ-07.003
- REQ-05.007
