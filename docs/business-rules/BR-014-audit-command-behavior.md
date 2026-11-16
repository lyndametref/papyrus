---
id: "BR-014:"
title: "Papyrus audit Command Behavior"
created_date: '2026-11-14T18:39:51Z'
updated_date: '2026-11-16T16:57:07Z'
labels: []
references: []
status: active
---
# BR-014: Papyrus audit Command Behavior

## Description

`papyrus audit` manages audit records as the ordinary `document_types.audit` entry using only the baseline document verbs and the document status model, with each record carrying a title, content, and audit type.

## Rationale

Based on REQ-05.006 — Manage Audit Records, REQ-05.007 — Manage Document Archive State and Deletion, REQ-07.002 — Configure Document Types and Slugs, and REQ-07.003 — Dispatch Config-Defined Document Commands, this rule exists because an audit record is a front-matter document record with a configurable audit type; it keeps that type field while using the common document command family, status model, and archive location.

## Applicability

Applies to every invocation of `papyrus audit` and to every audit record, in projects where the `audit` entry is enabled.

### Type entry

- Ordinary `document_types` entry (default prefix `AUD`, directory `audit-records/`); renameable, removable, disableable, and assignable another prefix or `doc_directory`-relative directory.
- IDs use the entry prefix and `id_padding` and are preserved on archive.

### Verbs

- Baseline only: `create`, `list`, `view`, `edit`, `archive`, `delete`, plus `status <id> <value>`.
- No extension verb is declared.
- Content is retained: `--title`, `--content`, and `--audit-type` on create and edit, and `--editor` for the body.
- The built-in audit types remain `security` (default), `licence`, and `architecture`; additional audit types are configurable.

### Status and archive

- Main statuses `active` and `archived` plus optional configured custom statuses; custom statuses are front-matter values only and move no file.
- A new record starts at the configured `default_status`, or `active` when none is configured.
- `archive` sets `archived`, moves the file into the entry's `archived/` subfolder, and keeps the ID.
- Archiving does not retain a previous custom status value, and there is no inverse command.
- `status` never archives: `status <id> archived` is rejected, naming `archive` as the only way.
- Decision statuses are a separate lifecycle and never apply to documents; the milestone `open -> closed` and release `planned -> released` lifecycles are fixed and unaffected by `statuses` or `default_status`.
