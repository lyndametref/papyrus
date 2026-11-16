---
id: "BR-019:"
title: "Test-Case Document Type Behavior"
created_date: '2026-11-14T18:39:51Z'
updated_date: '2026-11-16T16:57:06Z'
labels: []
references: []
status: active
---
# BR-019: Test-Case Document Type Behavior

## Description

A test case is the ordinary `document_types.test-case` entry managed by `papyrus test-case`; its records carry a required title and description plus an optional applicability, use only the baseline document verbs and the document status model, and become inactive only through `archive`.

## Rationale

Based on REQ-05.013 — Manage Test Cases, with REQ-05.007 — Manage Document Archive State and Deletion, REQ-05.010 — Record Metadata in YAML Front Matter, REQ-07.002 — Configure Document Types and Slugs, and REQ-07.003 — Dispatch Config-Defined Document Commands, this rule exists because test scenarios are only traceable when they are first-class document records with a stable ID and the common command family and status model; the type adds an applicability field but no verb or lifecycle of its own.

## Applicability

Applies to every test-case record and to the command invocations that create, list, view, edit, archive, or delete one, in projects where the `test-case` entry is enabled.

### Type entry

- Ordinary `document_types` entry (default prefix `TC`, directory `test-cases/`); renameable, removable, disableable, and assignable another prefix or `doc_directory`-relative directory.
- IDs use the entry prefix and `id_padding` and are preserved on archive.
- Records carry the general front-matter keys `id`, `title`, `created_date`, `updated_date`, `labels`, `references`, and `status`, plus a `# <ID> <title>` body heading; `description` and `applicability` are body fields.

### Verbs and content

- Baseline only: `create`, `list`, `view`, `edit`, `archive`, `delete`, plus `status <id> <value>`.
- No extension verb is declared.
- Create requires a title and description; applicability is optional. Edit updates title and description.

### Status and archive

- Main statuses `active` and `archived` plus optional configured custom statuses; custom statuses are front-matter values only and move no file.
- A new record starts at the configured `default_status`, or `active` when none is configured.
- `archive` sets `archived`, moves the file into the entry's `archived/` subfolder, and keeps the ID.
- Archiving does not retain a previous custom status value, and there is no inverse command.
- `status` never archives: `status <id> archived` is rejected, naming `archive` as the only way.
- Decision statuses are a separate lifecycle and never apply to documents; the milestone `open -> closed` and release `planned -> released` lifecycles are fixed and unaffected by `statuses` or `default_status`.
