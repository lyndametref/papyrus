# BR-011: Papyrus glossary Command Behavior

## Description

`papyrus glossary` manages glossary terms as the ordinary `document_types.glossary` entry using only the baseline document verbs and the document status model, with each term carrying a required term and definition.

## Rationale

Based on REQ-05.003 — Manage Glossary Terms, REQ-07.002 — Configure Document Types and Slugs, REQ-07.003 — Dispatch Config-Defined Document Commands, and REQ-05.007 — Manage Document Archive State and Deletion, this rule exists because a glossary term is a standard document record: it needs a stable identity, the common create/list/view/edit/archive/delete family, and the common status model, and it adds no verb or lifecycle of its own.

## Applicability

Applies to every invocation of `papyrus glossary` and to every glossary record, in projects where the `glossary` entry is enabled.

### Type entry

- Ordinary `document_types` entry (default prefix `GLO`, directory `glossary/`); renameable, removable, disableable, and assignable another prefix or `doc_directory`-relative directory.
- IDs use the entry prefix and `id_padding` and are preserved on archive.

### Verbs

- Baseline only: `create`, `list`, `view`, `edit`, `archive`, `delete`, plus `status <id> <value>`.
- No extension verb is declared.
- Content is retained: `--term` and `--definition` on create and edit, and `--editor` for the body.

### Status and archive

- Main statuses `active` and `archived` plus optional configured custom statuses; custom statuses are front-matter values only and move no file.
- A new record starts at the configured `default_status`, or `active` when none is configured.
- `archive` sets `archived`, moves the file into the entry's `archived/` subfolder, and keeps the ID.
- Archiving does not retain a previous custom status value, and there is no inverse command.
- `status` never archives: `status <id> archived` is rejected, naming `archive` as the only way.
- Decision statuses are a separate lifecycle and never apply to documents; the milestone `open -> closed` and release `planned -> released` lifecycles are fixed and unaffected by `statuses` or `default_status`.
