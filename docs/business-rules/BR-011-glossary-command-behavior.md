---
id: "BR-011:"
title: "Papyrus glossary Command Behavior"
created_date: '2026-11-14T18:39:51Z'
updated_date: '2026-11-20T21:13:34Z'
labels: []
references: []
status: active
---
# BR-011: Papyrus glossary Command Behavior

## Description

`papyrus glossary` is the command family generated from the `SingleMarkdown` `glossary` descriptor: the term is the record title and the first positional `TERM` on create, the single required `definition` field is stored under `## Definition` and offered as `--definition`, and no document command accepts `--term`.

## Rationale

Based on REQ-05.003 — Manage Glossary Terms, REQ-07.003 — Dispatch Config-Defined Document Commands, REQ-05.010 — Record Metadata in YAML Front Matter, and REQ-05.007 — Manage Document Archive State and Deletion, this rule exists because the glossary is the type where the uniform title contract replaces a former type-specific option: a term is the record's name and the definition its meaning, so the term must be the title, positional on create and renameable through `--title`, while the definition stays a required declared field under its heading. Removing `--term` must change the command surface only: every stored term value survives as the record title, and an invocation that still supplies `--term` fails as a usage error instead of writing a record.

## Applicability

Applies to every invocation of `papyrus glossary` and to every glossary record, in projects where the `glossary` entry is enabled.

### Type entry

- `glossary` is the seed `SingleMarkdown` entry (prefix `GLO`, directory `glossary/`); its model and field set are seed data and are never declared in configuration, and renaming the entry key renames the command without moving an existing file (BR-016).
- The type declares exactly one field, `definition`, stored under the heading `## Definition`, of kind `multiline`, and marked required; it declares no front-matter key, so a definition lives only in the body under `## Definition`.
- IDs use the entry prefix and `id_padding` and are preserved on archive.

### Verbs

- The model generates `create`, `list`, `view`, `edit`, `archive`, `delete`, and `status <id> <value>`; no extension verb and no type-specific module exists (BR-001).
- `--definition` is the field option, accepts `-` to read the value from standard input, and the definition may also be edited through `--editor`; `--label` and `--reference` remain the general metadata options.
- `create <TERM> --definition <TEXT>` takes the term as the first positional `TITLE`; `--title` is accepted on create and edit; the term is single-line and non-empty, required on create, and an edit can never clear it.
- No document command has `--term`: a glossary create or edit that supplies it is a usage error and writes nothing.

### Term, title, and definition

- A term is the record title and not a field: it is the canonical name of one business concept, and the definition is that concept's meaning.
- A create that omits the definition, or an edit that clears it, is rejected with one validation outcome and writes no file.
- Renaming a term changes the displayed title only: the record ID, the file location, and every reference by ID stay unchanged.
- Existing term values survive the removal of `--term`: a stored term is already the record title, so no term text is lost, re-derived, or moved, and the definition remains under `## Definition`.

### Lists and references

- Reference visibility is `visible`, so a list row renders its reference list; labels render in the Title cell and the single-document view keeps its References disclosure.

### Status and archive

- Terms carry front-matter `status` with the value `active`, `archived`, or a configured custom status; creation writes the configured `default_status` or `active`, and only `status` and `archive` write it, so no create, edit, or field option sets it.
- `archive` sets `archived`, moves the file into the entry's `archived/` subfolder, keeps the ID, discards any previous custom status value, and has no inverse command.
- `status` never archives: `status <id> archived` is rejected with a validation outcome naming `archive` as the only way, and `status` on an archived record is rejected without a move.
- Deleting a term permanently removes it with confirmation, with `--force` skipping the prompt, and decision records and the fixed milestone and release lifecycles are separate and unaffected.
