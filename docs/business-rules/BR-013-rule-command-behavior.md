# BR-013: Papyrus rule Command Behavior

## Description

`papyrus rule` manages business rules as the ordinary `document_types.rule` entry using only the baseline document verbs and the document status model, with each rule carrying a description, rationale, applicability, and optional requirement references.

## Rationale

Based on REQ-05.005 — Manage Business Rules, REQ-07.002 — Configure Document Types and Slugs, REQ-07.003 — Dispatch Config-Defined Document Commands, and REQ-05.007 — Manage Document Archive State and Deletion, this rule exists because a business rule is a front-matter document record whose content is defined by its three prose sections and its requirement references; it therefore keeps those content fields while using the common document command family and status model.

## Applicability

Applies to every invocation of `papyrus rule` and to every business-rule record, in projects where the `rule` entry is enabled.

### Type entry

- Ordinary `document_types` entry (default prefix `RUL`, directory `business-rules/`); renameable, removable, disableable, and assignable another prefix or `doc_directory`-relative directory.
- IDs use the entry prefix and `id_padding` and are preserved on archive.

### Verbs

- Baseline only: `create`, `list`, `view`, `edit`, `archive`, `delete`, plus `status <id> <value>`.
- No extension verb is declared.
- Content is retained: `--description`, `--rationale`, and `--applicability` on create; `--title`, `-d/--description`, `--requirement <id>` (repeatable), `--no-requirement`, and `--editor` on edit.

### Status and archive

- Main statuses `active` and `archived` plus optional configured custom statuses; custom statuses are front-matter values only and move no file.
- A new record starts at the configured `default_status`, or `active` when none is configured.
- `archive` sets `archived`, moves the file into the entry's `archived/` subfolder, and keeps the ID.
- Archiving does not retain a previous custom status value, and there is no inverse command.
- `status` never archives: `status <id> archived` is rejected, naming `archive` as the only way.
- Decision statuses are a separate lifecycle and never apply to documents; the milestone `open -> closed` and release `planned -> released` lifecycles are fixed and unaffected by `statuses` or `default_status`.
