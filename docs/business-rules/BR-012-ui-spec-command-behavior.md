# BR-012: Papyrus ui-spec Command Behavior

## Description

`papyrus ui-spec` manages UI specifications as the ordinary `document_types.ui-spec` entry using only the baseline document verbs and the document status model, with each specification carrying a view association and a description.

## Rationale

Based on REQ-05.004 — Manage UI Specifications, REQ-07.002 — Configure Document Types and Slugs, REQ-07.003 — Dispatch Config-Defined Document Commands, and REQ-05.007 — Manage Document Archive State and Deletion, this rule exists because a UI specification is a standard front-matter document record that adds a view association to the shared document family; its storage, status, and archive behavior must remain the common document behavior.

## Applicability

Applies to every invocation of `papyrus ui-spec` and to every UI-specification record, in projects where the `ui-spec` entry is enabled. UI mockups are a separate entry (`ui-mockup`) with HTML metadata and an additional `supersede` verb.

### Type entry

- Ordinary `document_types` entry (default prefix `UI`, directory `specifications/ui/`); renameable, removable, disableable, and assignable another prefix or `doc_directory`-relative directory.
- IDs use the entry prefix and `id_padding` and are preserved on archive.

### Verbs

- Baseline only: `create`, `list`, `view`, `edit`, `archive`, `delete`, plus `status <id> <value>`.
- No extension verb is declared.
- Content is retained: `--view` and `--description` on create and edit, and `--editor` for the body.

### Status and archive

- Main statuses `active` and `archived` plus optional configured custom statuses; custom statuses are front-matter values only and move no file.
- A new record starts at the configured `default_status`, or `active` when none is configured.
- `archive` sets `archived`, moves the file into the entry's `archived/` subfolder, and keeps the ID.
- Archiving does not retain a previous custom status value, and there is no inverse command.
- `status` never archives: `status <id> archived` is rejected, naming `archive` as the only way.
- Decision statuses are a separate lifecycle and never apply to documents; the milestone `open -> closed` and release `planned -> released` lifecycles are fixed and unaffected by `statuses` or `default_status`.
