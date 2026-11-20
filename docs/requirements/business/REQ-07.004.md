---
id: REQ-07.004
title: "Provide One Schema-Driven Document Content Model with Three Document Models"
created_date: '2026-11-13T21:00:17Z'
updated_date: '2026-11-20T20:14:44Z'
labels: []
references:
  - RE-010
  - ADR-036
  - ADR-039
---
# REQ-07.004 Provide One Schema-Driven Document Content Model with Three Document Models

## User Story
As a project contributor, I want every document type to declare its content fields under one of three document models so that a record's fields, commands, and presentation stay consistent without per-type code.

## Group
REQ-07 Artifact Categories and Config-Defined Document Types

## Criticality
Must have

## Description
Every document type — a default seed type or a user-defined type — has exactly one **document model**. There are three models, and the former `ContentModel` is removed:

- **SingleMarkdown** — one record per ID, with a Markdown body.
- **GroupedMarkdown** — a group record plus member records, each with a Markdown body.
- **SingleHtmlMockup** — one record per ID, with an HTML body.

The model is not a configuration key: `model`, `title`, `content`, and `id_shape` are rejected in `document_types`. A seed slug takes its model from the seed table, and every user-defined slug is `SingleMarkdown`. `GroupedMarkdown` and `SingleHtmlMockup` are seed-only: only `req` and `nfr` are grouped, only `ui-mockup` is an HTML mockup, and a user-defined type stays `SingleMarkdown` even when it reuses a seed prefix, directory, or field name.

**Declared field set.** A document type declares the content fields its records support, and a field's position in its sequence is its order in the record body, in CLI options and help, and in the browser form. `SingleMarkdown` and `SingleHtmlMockup` declare one `fields` sequence, which may be empty (`fields: []`). `GroupedMarkdown` declares `group.fields` and `member.fields` with independent order, and a top-level `fields` on a grouped type is rejected. A field declares a stable `key`, at least one storage location (a body `heading`, a `frontmatter_key`, or both), its CLI `option`, a presentation `label` and `help`, a `kind` of `text`, `multiline`, or `list`, whether it is `required`, and the `verbs` (`create` and `edit`) on which its option is offered. The keys `id`, `title`, `status`, `labels`, `references`, `created_date`, `updated_date`, and `group` are reserved: a field may use none of them as its key or front-matter key, and field keys are unique within a role. A required field must be supplied on create and may be changed but not cleared on edit; an optional field may be cleared. Adding or changing a field is a configuration file edit, not a code change.

A type therefore has no fixed content model. A record carries the general front-matter keys `id`, `title`, `created_date`, `updated_date`, `labels`, `references`, and `status` (REQ-05.010) plus exactly the fields its type and role declare, stored under the declared headings and front-matter keys. `labels` and `references` remain general repeatable metadata rather than fields and are supplied through `--label` and `--reference`.

**Title.** Title is fixed and non-configurable for every model. It is single-line, non-empty, always present in the record, required on create, and can never be cleared by an edit. It is not a field, its key is reserved, it is the positional title on create, and it is changed through `--title` on edit. A `SingleHtmlMockup` record applies the same title contract to the title in its HTML metadata.

**GroupedMarkdown.** The group record has a plain ID `<PREFIX>-<group-sequence>`, and each member record has a dotted ID `<PREFIX>-<group-sequence>.<member-sequence>`. `create-group` creates the group record with a positional title. `create` creates a member and requires an existing active group of the same type, supplied through the group slot or `--group`; a group of another type, an archived group, or an unknown group is rejected and nothing is written. The member's `group` front-matter key is written and maintained only by the engine: it is reserved, is never a field, is never editable through a field option, and a member's group link cannot change after creation. Group and member records follow the document status model (REQ-07.001). Archiving or deleting a group archives, deletes, or otherwise changes none of its members, and a list shows the group records and their members in group order, then member order.

**SingleHtmlMockup.** A type of this model has one record per ID with a plain ID, and its state lives in the HTML metadata comment rather than in YAML front matter. The allowed states are fixed to `pending`, `approved`, and `archived`; configuration cannot add, remove, or rename them, a `statuses` or `default_status` declaration on such a type is rejected, and the type has no `status` verb. `archive` is the only path to `archived`, and `supersede` records the superseding ID (REQ-05.008). The type declares no field sequence and its record is presented through its fixed metadata rather than through fields.

**Configuration is data, never code.** Configuration declares fields and presentation facts only. It is never executable: no template, expression, script, or code path is read from configuration. Content changes are computed by the document engine as patch operations (`PatchOp` values that set a front-matter key, set or clear a body section, set the heading title, and the model-specific group-link, HTML-metadata, and supersession changes) and applied in one atomic write by the single record writer. The engine never formats or writes a record body itself, so byte preservation, optimistic concurrency, and the single write path are retained (ADR-0013), and a patch that changes nothing performs no write.

## Acceptance Criteria
- Every document type has exactly one document model, `SingleMarkdown`, `GroupedMarkdown`, or `SingleHtmlMockup`; `ContentModel` is removed and no command, configuration key, or browser page exposes it.
- A `document_types` entry that declares `model`, `title`, `content`, or `id_shape` is rejected, and a type takes its model from the seed table or from the `SingleMarkdown` default for a user-defined slug.
- Only `req` and `nfr` are `GroupedMarkdown` and only `ui-mockup` is `SingleHtmlMockup`; a user-defined type is `SingleMarkdown`, including when it reuses a seed prefix, directory, or field name.
- A `SingleMarkdown` or `SingleHtmlMockup` type declares one `fields` sequence, which may be empty; a `GroupedMarkdown` type declares `group.fields` and `member.fields` in independent order and is rejected if it declares a top-level `fields`.
- A field declares a key, at least one of `heading` and `frontmatter_key`, a `kind` of `text`, `multiline`, or `list`, a required flag, and verbs from `create` and `edit`; a field with neither `heading` nor `frontmatter_key` is rejected.
- A field key or front-matter key equal to `id`, `title`, `status`, `labels`, `references`, `created_date`, `updated_date`, or `group` is rejected, and a duplicate field key within a role is rejected.
- A created record carries the general front matter `id`, `title`, `created_date`, `updated_date`, `labels`, `references`, and `status` plus exactly the fields its type and role declare.
- A create that omits a required field produces one validation outcome and writes no file.
- An edit that clears a required field is rejected and writes no file, while an optional field can be cleared.
- Field sequence position governs the record body order, the CLI option and help order, and the browser form order.
- Title is single-line and non-empty in every model, required on create, never clearable on edit, is not a field, and its key is reserved; a `SingleHtmlMockup` record applies the same contract in its HTML metadata.
- A `GroupedMarkdown` group has a plain ID `<PREFIX>-<group-sequence>` and its members have dotted IDs `<PREFIX>-<group-sequence>.<member-sequence>`.
- `create-group` creates a group record, and creating a member requires an existing active group of the same type supplied through the group slot or `--group`; an unknown, archived, or other-type group is rejected and nothing is written.
- The `group` front-matter link is engine-owned: it is not a field, has no option, and cannot be changed after creation.
- Archiving or deleting a group leaves its members unchanged, and a list shows group records and their members in group order, then member order.
- A `SingleHtmlMockup` type has one record per ID with a plain ID, a fixed `pending`, `approved`, or `archived` metadata state, no `status` verb, a rejection of `statuses` and `default_status`, and `archive` as its only path to `archived`.
- Configuration declares fields and presentation facts only and is never executable: no template, expression, script, or code path is read from configuration.
- The document engine emits only patch operations (`PatchOp`) and never writes a record body; the single record writer applies them atomically, and a patch that changes nothing writes no file.

## Open Questions
- Whether a maximum title length is required; this requirement fixes title as single-line and non-empty, while ADR-039 requires a length limit in form validation.
- Whether renaming a field key or heading migrates already stored sections and front-matter keys; this requirement fixes field keys as stable and defines no migration.

## Traceability
- Source: RE-010 — Descriptor-driven document engine refactor: fixed owner decisions (accepted, not reopenable): D5 (`DocumentModel` supersedes `ContentModel`), D6 (grouped and HTML models seed-only; user-defined default `SingleMarkdown`), D7 (immutable group membership, engine-owned link, no cascade, merged order), D9 (record, JSON, and status-code equivalence), D13 (fixed title contract), D14 (nested group/member field blocks, `fields` sequence, no per-field order or enabled, no `model`/`title`/`content`/`id_shape` keys).
- ADR-036 — Config-declared document field schema and one descriptor model (one descriptor, three `DocumentModel` variants, field vocabulary, role field sets, seed table, model-derived verbs, PatchOp-only engine).
- ADR-039 — One descriptor-driven document view and edit surface (one field map per model and role, fixed title control, role-aware rendering, fixed mockup metadata, one submit path).
- ADR-0017 — Artifact category registry and type descriptors (decisions 1-4, 7, 9, and 10 replaced by ADR-036; decisions 5, 6, and 8 remain in force).
- ADR-0018 — Generic document content model (superseded in full by ADR-036).
- ADR-0013 — Shared record envelope without a storage layer (single patch and write path).
- ADR-0019 — Fail-Closed Document-Type Configuration Validation (rejected keys and field declarations).
- REQ-07.001 — Organize Artifacts into Three Categories (document status model, status writers, mockup states).
- REQ-07.002 — Configure Document Types and Slugs (entry validation and rejected keys).
- REQ-07.003 — Dispatch Config-Defined Document Commands (model-derived verbs and options).
- REQ-05.001 — Manage Business Requirements, REQ-05.002 — Manage Non-Functional Requirements (the two grouped seed types).
- REQ-05.003 — Manage Glossary Terms (single-Markdown seed type with a required field).
- REQ-05.008 — Manage UI Mockups (HTML metadata lifecycle and `supersede`).
- REQ-05.010 — Record Metadata in YAML Front Matter (general front-matter keys and metadata-only fields).
- REQ-05.011 — Maintain Record Metadata Through All Commands (`status` exemption from field options).
- REQ-01.005 — Assign Stable Record IDs (prefix and padding for plain IDs; hierarchical IDs for grouped members).
- BR-016 — Papyrus Configuration Keys and Precedence (needs revision for the document model and field sequence keys).
- BR-017 — Papyrus tech-spec Command Behavior (seed single-Markdown type; needs revision for the declared field set).
