---
id: REQ-05.011
title: "Maintain Record Metadata Through All Commands"
created_date: '2026-11-13T20:55:54Z'
updated_date: '2026-11-15T15:47:29Z'
labels: []
references: []
---
# REQ-05.011 Maintain Record Metadata Through All Commands

## User Story
As a project contributor, I want Papyrus commands to set and maintain record metadata automatically so that front matter stays accurate without manual editing.

## Group
REQ-05 Project Documentation Management

## Criticality
Must have

## Description
`created_date` is set automatically when a record is created and is never changed afterwards. `updated_date` is set automatically at creation and refreshed by every write path, including a browser save. `id` is generated and stable (REQ-01.005), and `id`, `created_date`, and `updated_date` are exempt from the option rule; every other front-matter field can be supplied as a command option at creation and at update, and a command that lacks an option for an applicable field gains one. For document records, `status` is also exempt from the option rule and is lifecycle- and configuration-owned: a new record takes the type's configured `default_status` (or `active`), and the value changes only through `papyrus <document-slug> status` and `archive`, so no create or edit option sets it (REQ-07.001). The status of a task, milestone, release, or decision keeps its documented behavior (REQ-02, REQ-03, REQ-04); the exemption applies to document records only. `ordinal` is an opaque non-negative integer supplied through a create/update option and defaulted from the current maximum. Every command that writes a record file keeps the existing front matter intact and changes only the fields that the operation changes; a browser save follows the same contract.

Existing managed Markdown records that do not yet carry a front-matter block are adopted per record: the block is added on the record's first read and on its first write, and no bulk migration command is required. The managed record types are task, milestone, release, decision, business requirement, non-functional requirement, glossary term, UI specification, technical specification, business rule, and audit record, plus every document type declared under `document_types` (REQ-07.002). Prose Markdown that no managed record type claims, such as files under `docs/specifications`, `docs/domain-model`, and `docs/business-processes` or `README` files, is out of scope and is not adopted. UI mockups and other HTML files are excluded. Frozen ADR bodies are not edited beyond the added front-matter block.

Adoption adds the block and changes nothing else in the body; when adoption happens on a write, a field that the record still holds in a legacy document section moves to front matter and that section is removed, so no field is stored twice. Adoption is idempotent: once a record carries front matter, neither a read nor a write adopts it again. Papyrus does not convert a record to, or rewrite it in, a Backlog.md on-disk layout and does not use a Backlog.md storage ID (NFR-04.004): Backlog.md backlog records enter Papyrus only through the one-way import script, and no read or write path re-shapes a task or milestone record. Timestamps are quoted ISO-8601 UTC strings, for example `'2026-10-31T22:26:00Z'`.

## Acceptance Criteria
- `created_date` and `updated_date` are set automatically when a record is created.
- `updated_date` is refreshed automatically on every write, including a browser save of an edit; `created_date` is never changed.
- `id`, `created_date`, and `updated_date` are exempt from the option rule; for every record type, each other front-matter field can be supplied as an option on creation and as an option on update.
- For document records, `status` is exempt from the option rule: a new record starts at the type's `default_status` or `active`, no create or edit option sets it, and it changes only through `papyrus <document-slug> status` and `archive` (REQ-07.001).
- The status of a task, milestone, release, or decision keeps its documented behavior and is not affected by the document `status` exemption.
- `ordinal` is an opaque non-negative integer with a create option and an update option; a new record's default is derived from the current maximum.
- Every command that writes a record (create, edit, status, archive, delete, supersede, milestone close, release close) preserves the front-matter fields that the operation does not change.
- A browser save preserves the front-matter fields and body content the save does not change.
- An existing command that has no option for an applicable front-matter field provides that option.
- Each managed Markdown record file without a front-matter block is adopted lazily: the block is added on its first read and on its first write; after either, the record carries front matter.
- Adoption is idempotent: a record that already carries front matter is not adopted again on subsequent reads or writes.
- No bulk migration command is required; the corpus converges record by record as records are read or written.
- Adoption covers only existing files claimed by a managed record type; prose Markdown that is not a managed record (for example under `docs/specifications`, `docs/domain-model`, `docs/business-processes`, or `README` files) is not modified.
- Adoption excludes HTML files, including UI mockups.
- Adoption changes nothing in a frozen ADR body except adding the front-matter block.
- Adoption adds nothing else to the body; on a write, a field-holding legacy section is removed after its values move to front matter, so no field is stored twice.
- No read or write path converts a task or milestone record to a Backlog.md on-disk layout or assigns a Backlog.md storage ID; Backlog.md backlog records enter Papyrus only through the one-way import script (NFR-04.004).
- `created_date` and `updated_date` are quoted ISO-8601 UTC strings of the form `'YYYY-MM-DDTHH:MM:SSZ'`.

## Traceability
- Source: customer-feedback handoff — Internal team discussion, Item 6 (high), raw quote and clarifications (verbatim in REQ-05.010).
- User decision 2026-11-08: browser saves refresh `updated_date`; `id`/`created_date`/`updated_date` are exempt from the option rule; `ordinal` is an opaque non-negative integer defaulted from the current maximum; timestamps are quoted ISO-8601 UTC.
- User decision 2026-11-08 (adoption, revised): adoption is lazy and per-record — each managed Markdown record gains front matter on its first read and its first write; no bulk migration command is required. Excluding UI mockups/HTML and frozen ADR bodies beyond the block.
- User decision 2026-11-08 (adoption scope): adoption covers only files claimed by a managed record type and is idempotent; non-record prose Markdown is out of scope.
- Maintainer decision 2026-11-11: the Backlog.md interoperability and lazy Backlog-layout conversion clauses are removed (NFR-04.004). Front-matter adoption for existing records remains, since it is Papyrus-native and not part of Backlog.md compatibility.
- ADR-0007 — YAML Front Matter as the Single Record Metadata Source (timestamp format, patch contract, option exemption, lazy adoption decision).
- ADR-0009 — Shared Record Envelope and Unified Write Path for CLI and Browser (one write path for CLI and browser).
- Project-owner decision 2026-11-13 (D5): document `status` is exempt from the option rule because it is lifecycle- and configuration-owned; no create or edit option sets it.
- REQ-01.002 — Read and Edit Project Configuration (task and document statuses)
- REQ-01.005 — Assign Stable Record IDs
- REQ-02.001 — Create and Describe Tasks
- REQ-02.002 — View and Edit Tasks
- REQ-04.002 — Update, Supersede, and Archive Decisions
- REQ-05.010 — Record Metadata in YAML Front Matter
- REQ-07.001 — Organize Artifacts into Three Categories (`status` command ownership)
- REQ-07.002 — Configure Document Types and Slugs (`default_status`)
- NFR-04.004 — Renounce Backlog.md Compatibility and Provide Import
