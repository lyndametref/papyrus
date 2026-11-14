# BR-017: Papyrus tech-spec Command Behavior

## Description

`papyrus tech-spec` manages technical specifications as the ordinary `document_types.tech-spec` entry using only the baseline document verbs and the document status model, with each record carrying a required title and optional description.

## Rationale

Based on REQ-05.009 — Manage Technical Specifications, REQ-07.002 — Configure Document Types and Slugs, REQ-07.003 — Dispatch Config-Defined Document Commands, and REQ-05.007 — Manage Document Archive State and Deletion, this rule exists because technical specifications are the model the generic document content follows: a title, an optional description, and the general front-matter keys, with the common command family, status model, and archive location.

## Applicability

Applies to every invocation of `papyrus tech-spec` and to every technical-specification record, in projects where the `tech-spec` entry is enabled.

### Type entry

- Ordinary `document_types` entry (default prefix `TECH`, directory `technical/`); renameable, removable, disableable, and assignable another prefix or `doc_directory`-relative directory.
- IDs use the entry prefix and `id_padding` and are preserved on archive.

### Verbs

- Baseline only: `create`, `list`, `view`, `edit`, `archive`, `delete`, plus `status <id> <value>`.
- No extension verb is declared.
- Content is retained: a required title and description on create; `--title`, `--description`, and `--editor` on edit.
- Records follow the same input sanitization and project-directory boundary as every other document type.

### Status and archive

- Main statuses `active` and `archived` plus optional configured custom statuses; custom statuses are front-matter values only and move no file.
- A new record starts at the configured `default_status`, or `active` when none is configured.
- `archive` sets `archived`, moves the file into the entry's `archived/` subfolder, and keeps the ID.
- Archiving does not retain a previous custom status value, and there is no inverse command.
- `status` never archives: `status <id> archived` is rejected, naming `archive` as the only way.
- Decision statuses are a separate lifecycle and never apply to documents; the milestone `open -> closed` and release `planned -> released` lifecycles are fixed and unaffected by `statuses` or `default_status`.
