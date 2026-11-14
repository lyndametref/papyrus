# BR-010: Papyrus nfr Command Behavior

## Description

`papyrus nfr` manages non-functional requirements as the ordinary `document_types.nfr` entry, using the baseline document verbs plus `create-group` and a required `--group` on `create`; records use the document status model and become inactive only through `archive`.

## Rationale

Based on REQ-05.002 — Manage Non-Functional Requirements, REQ-07.002 — Configure Document Types and Slugs, REQ-07.003 — Dispatch Config-Defined Document Commands, and REQ-05.007 — Manage Document Archive State and Deletion, this rule exists because non-functional requirements are addressed by the document slug `nfr`, which is itself the command name, and add a group concept on top of the shared document family: requirement-to-group cardinality is 1..1 and groups contain non-functional requirements only, while storage, status, and archive behavior stay identical to every other document type. The former command name is not retained as an alias.

## Applicability

Applies to every invocation of `papyrus nfr` and to every non-functional-requirement record and group, in projects where the `nfr` entry is enabled.

### Type entry

- `nfr` is an ordinary `document_types` entry (default prefix `NFR`, directory `requirements/non-functional/`); it can be renamed, removed, disabled, or given another prefix or `doc_directory`-relative directory.
- Renaming the key renames the command; the change does not move existing files.
- IDs use the entry prefix and `id_padding` and are preserved on archive.

### Verbs

- Baseline: `create`, `list`, `view`, `edit`, `archive`, `delete`, plus `status <id> <value>`.
- Extensions: `create-group` and a required `--group` on `create`.
- Content is retained: create and edit carry title and description; a group carries a title and optional description; `--editor` opens the body.
- No other extension verb is declared for `nfr`.

### Status and archive

- Main statuses `active` and `archived` plus optional configured custom statuses; custom statuses are front-matter values only and move no file.
- A new record starts at the configured `default_status`, or `active` when none is configured.
- `archive` sets `archived`, moves the file into the entry's `archived/` subfolder, and keeps the ID.
- Archiving does not retain a previous custom status value, and there is no inverse command.
- `status` never archives: `status <id> archived` is rejected, naming `archive` as the only way.
- Decision statuses are a separate lifecycle and never apply to documents; the milestone `open -> closed` and release `planned -> released` lifecycles are fixed and unaffected by `statuses` or `default_status`.
