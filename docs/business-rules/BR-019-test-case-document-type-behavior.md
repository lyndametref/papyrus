---
id: "BR-019:"
title: "Test-Case Document Type Behavior"
created_date: '2026-11-14T18:39:51Z'
updated_date: '2026-11-20T21:22:06Z'
labels: []
references: []
status: active
---
# BR-019: Test-Case Document Type Behavior

## Description

`papyrus test-case` is the command family generated from the `SingleMarkdown` `test-case` descriptor: records use the baseline verbs plus `status` with no type-specific verb, the title is the first positional `TITLE` on create and `--title` on edit, and the seed declares two optional body fields, `description` under `## Description` then `applicability` under `## Applicability`, in that sequence.

## Rationale

Based on REQ-005.013 — Manage Test Cases, REQ-007.004 — Provide One Schema-Driven Document Content Model with Three Document Models, REQ-007.002 — Configure Document Types and Slugs, and REQ-007.003 — Dispatch Config-Defined Document Commands, this rule exists because a test case is a stable, traceable document that adds an applicability section to the plain document shape, and deleting `src/test_case.rs` must not lose either section or the command behavior around them. The command family, the `--description` and `--applicability` options, the `## Description` then `## Applicability` body order, and the status model must all be reconstructed from the descriptor, with no per-type module, verb table, or help text. The exact field list is recorded here because the seed is the only definition of a built-in type: changing a field is a requirement or ADR change, and a project must be able to add a field by editing its configuration file alone, without a code change (RE-010 D14).

## Applicability

Applies to every invocation of `papyrus test-case` and to every test-case record, in projects where the `test-case` entry is enabled.

### Type entry

- `test-case` is the seed `SingleMarkdown` entry (prefix `TC`, directory `test-cases/`); its model and field sequence are seed data and are never declared in configuration, and renaming the entry key renames the command without moving an existing file (BR-016).
- The seed declares exactly the fields in this table, in this sequence; the body headings and field order below are the existing ones and are preserved.

| key | heading | kind | required | verbs | option | frontmatter_key |
|---|---|---|---|---|---|---|
| `description` | `## Description` | `multiline` | no | `create`, `edit` | `description` | none |
| `applicability` | `## Applicability` | `text` | no | `create`, `edit` | `applicability` | none |

- Both fields are optional and body-only: a create that supplies only a title succeeds, an edit may supply or clear either field, and a field's front-matter key is its `frontmatter_key`; none of these fields declares one, so no field value appears in YAML front matter.
- Field sequence position is the order everywhere — body sections, CLI options and help, and the browser form (ADR-036 decision 20).
- IDs use the entry prefix and `id_padding` and are preserved on archive.

### Verbs

- The model generates `create`, `list`, `view`, `edit`, `archive`, `delete`, and `status <id> <value>`; no extension verb, no type-specific verb, and no type-specific module exists (BR-001).
- `--description` and `--applicability` are the field options in field-sequence order; a `multiline` field accepts `-` to read the value from standard input; `--label` and `--reference` remain the general metadata options, and no create or edit option sets `status`.
- `papyrus test-case create <TITLE>` takes the first positional `TITLE`, and `--title` is accepted on create and edit; the title is single-line and non-empty, required on create, never clearable by an edit, and supplying both the positional `TITLE` and `--title` is rejected and writes nothing.

### User extensibility and seed contract

- Adding a field is a configuration file edit: a project adds an entry to the type's `fields` sequence, and Papyrus generates the option, help, body section, and browser form input from it with no code change, rebuild, or per-type module (RE-010 D14, REQ-007.002).
- The field list above is the seed contract for `test-case`; changing a field key, heading, kind, required flag, verbs, option, or front-matter key is a requirement change (ADR-036 decision 30), never a silent behavior change.

### Lists and references

- Reference visibility is `visible`, so a list row renders its reference list; labels render in the Title cell and the single-document view keeps its References disclosure.

### Status and archive

- Test cases carry front-matter `status` with the value `active`, `archived`, or a configured custom status; creation writes the configured `default_status` or `active`, and only `status` and `archive` write it, so no create, edit, or field option sets it.
- `archive` sets `archived`, moves the file into the entry's `archived/` subfolder, keeps the ID, discards any previous custom status value, and has no inverse command.
- `status` never archives: `status <id> archived` is rejected with a validation outcome naming `archive` as the only way, and `status` on an archived record is rejected without a move.
- Deleting a test case permanently removes it with confirmation, with `--force` skipping the prompt, and decision records and the fixed milestone and release lifecycles are separate and unaffected.

## Requirements

- REQ-005.013
- REQ-007.004
- REQ-007.002
- REQ-007.003
- REQ-005.007
