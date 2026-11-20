---
id: REQ-05.011
title: "Maintain Record Metadata Through All Commands"
created_date: '2026-11-13T20:55:54Z'
updated_date: '2026-11-20T20:45:09Z'
labels: []
references:
  - RE-010
  - ADR-036
  - ADR-039
  - ADR-0007
  - ADR-0009
  - ADR-0013
  - REQ-07.001
  - REQ-07.002
  - REQ-07.003
  - REQ-07.004
  - REQ-05.010
---
# REQ-05.011 Maintain Record Metadata Through All Commands

## User Story
As a project contributor, I want Papyrus commands to set and maintain record metadata automatically so that front matter stays accurate without manual editing.

## Group
REQ-05 Project Documentation Management

## Criticality
Must have

## Description
`created_date` is set automatically when a record is created and is never changed afterwards. `updated_date` is set automatically at creation and refreshed by every write path, including a browser save and a write that changes a declared field. `id` is generated and stable (REQ-01.005), and `id`, `created_date`, and `updated_date` are exempt from the option rule. For document records, `status` is also exempt from the option rule and is lifecycle- and configuration-owned: a new record takes the type's configured `default_status` (or `active`), and the value changes only through `papyrus <document-slug> status` and `archive`, so no create or edit option sets it (REQ-07.001). The status of a task, milestone, release, or decision keeps its documented behavior (REQ-02, REQ-03, REQ-04); the exemption applies to document records only.

**Options come from the field sequence.** For a document record, the remaining options are the declared fields of the type's field sequence for the record's role: each field contributes one option on the verbs listed in its `verbs`, named by its `option` or by its `key` in kebab case when `option` is absent (REQ-07.003). A field whose `verbs` exclude `edit` has no edit option and is never writable after creation, and a field whose `verbs` exclude `create` has no create option. The sequence is the option and help order. `title` is not a field: it is the first positional `TITLE` on create and create-group and `--title` on create, create-group, and edit, and an edit can never clear it (RE-010 D13). The engine-owned `group` link of a `GroupedMarkdown` member never has an option (ADR-036 decision 9), and `--label` and `--reference` remain the general repeatable metadata options rather than fields (REQ-05.010).

**One write path.** A create and an edit each produce one patch of operations — set a front-matter key, set or clear a body section, set the heading title, and the model-specific group-link, HTML-metadata, and supersession changes — and the single record writer applies the patch atomically (ADR-036 decisions 35-37, ADR-039 decision 27). The engine never formats or writes a record body itself. A patch that changes nothing performs no write, and a validation failure writes nothing.

**What a write preserves.** Every command that writes a record file keeps the existing front matter intact and changes only the fields that the operation changes; a browser save follows the same contract. A write preserves an undeclared front-matter key already on disk, because that key is stored data and is not repaired, merged, or removed (ADR-039 decision 18). `ordinal` is an opaque non-negative integer supplied through a create/update option and defaulted from the current maximum; it belongs to backlog records, not to document types.

**Adoption.** Existing managed Markdown records that do not yet carry a front-matter block are adopted per record: the block is added on the record's first read and on its first write, and no bulk migration command is required. The managed record types are task, milestone, release, decision, business requirement, non-functional requirement, glossary term, UI specification, technical specification, business rule, and audit record, plus every document type declared under `document_types` (REQ-07.002, REQ-07.004). Prose Markdown that no managed record type claims, such as files under `docs/specifications`, `docs/domain-model`, and `docs/business-processes` or `README` files, is out of scope and is not adopted. UI mockups and other HTML files are excluded. Frozen ADR bodies are not edited beyond the added front-matter block.

Adoption adds the block and changes nothing else in the body; when adoption happens on a write, a field that the record still holds in a legacy document section moves to front matter and that section is removed, so no field is stored twice. Adoption is idempotent: once a record carries front matter, neither a read nor a write adopts it again. Papyrus does not convert a record to, or rewrite it in, a Backlog.md on-disk layout and does not use a Backlog.md storage ID (NFR-04.004): Backlog.md backlog records enter Papyrus only through the one-way import script, and no read or write path re-shapes a task or milestone record. Timestamps are quoted ISO-8601 UTC strings, for example `'2026-10-31T22:26:00Z'`.

## Acceptance Criteria
- `created_date` and `updated_date` are set automatically when a record is created.
- `updated_date` is refreshed automatically on every write, including a browser save of an edit and a write that changes a declared field; `created_date` is never changed.
- `id`, `created_date`, and `updated_date` are exempt from the option rule.
- For document records, `status` is exempt from the option rule: a new record starts at the type's `default_status` or `active`, no create or edit option sets it, and it changes only through `papyrus <document-slug> status` and `archive` (REQ-07.001).
- The status of a task, milestone, release, or decision keeps its documented behavior and is not affected by the document `status` exemption.
- Each declared field of a document type contributes one option on the verbs listed in its `verbs`, named by its `option` or by its `key` in kebab case, and a field whose `verbs` exclude `edit` has no edit option and is never writable after creation.
- Field options follow field sequence order in parsing and help, including the role-appropriate sequence of a `GroupedMarkdown` type.
- `title` is not a field: it is the first positional `TITLE` on create and create-group and `--title` on create, create-group, and edit, and an edit cannot clear it.
- The engine-owned `group` link of a `GroupedMarkdown` member has no option and is never writable through create or edit.
- `--label` and `--reference` remain the general repeatable metadata options and are not fields of a document type.
- A create and an edit each produce one patch applied atomically by the single record writer; the engine never formats or writes a record body itself.
- A patch that changes nothing performs no write, and a validation failure writes nothing.
- Every command that writes a record (create, edit, status, archive, delete, supersede, milestone close, release close) preserves the front-matter fields that the operation does not change.
- A write preserves an undeclared front-matter key already on disk and never repairs, merges, or removes it.
- `ordinal` is an opaque non-negative integer with a create option and an update option; a new record's default is derived from the current maximum.
- A browser save preserves the front-matter fields and body content the save does not change.
- An existing command that has no option for an applicable declared field provides that option.
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
- RE-010 — Descriptor-driven document engine refactor (accepted, not reopenable): D13 (fixed title, never clearable, and no `--term`), D14 (field sequence with no per-field order, so sequence position is option order), D6 (only `req` and `nfr` are grouped).
- ADR-036 — Config-declared document field schema and one descriptor model: decisions 15 and 19 (option name and per-field verbs), decision 18 (required and clear rules), decision 20 (sequence position is order everywhere), decision 24 (title contract), decisions 35-37 (the engine emits only patch operations, the single writer applies them, and validation precedes any write).
- ADR-039 — One descriptor-driven document view and edit surface: decision 18 (undeclared keys preserved read-only), decision 22 (field controls by `verbs` and `editable`), decision 26 (one submit path builds a patch and writes once), decision 27 (the concurrency and write contract is unchanged).
- ADR-0013 — Shared record envelope without a storage layer (single patch and write path).
- REQ-01.002 — Read and Edit Project Configuration (task and document statuses)
- REQ-01.005 — Assign Stable Record IDs
- REQ-02.001 — Create and Describe Tasks
- REQ-02.002 — View and Edit Tasks
- REQ-04.002 — Update, Supersede, and Archive Decisions
- REQ-05.010 — Record Metadata in YAML Front Matter (general-key ownership and document keys)
- REQ-07.001 — Organize Artifacts into Three Categories (`status` command ownership)
- REQ-07.002 — Configure Document Types and Slugs (`default_status`, field items)
- REQ-07.003 — Dispatch Config-Defined Document Commands (field options, per-field verbs, title contract)
- REQ-07.004 — Provide One Schema-Driven Document Content Model with Three Document Models (the engine emits only patch operations)
- NFR-04.004 — Renounce Backlog.md Compatibility and Provide Import
