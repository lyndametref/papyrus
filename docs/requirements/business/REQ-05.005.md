---
id: REQ-05.005
title: "Manage Business Rules"
created_date: '2026-11-13T20:17:02Z'
updated_date: '2026-11-20T20:44:54Z'
labels: []
references:
  - RE-010
  - ADR-036
  - ADR-039
  - ADR-035
  - ADR-033
  - REQ-07.001
  - REQ-07.002
  - REQ-07.003
  - REQ-07.004
  - REQ-05.007
  - REQ-05.010
  - REQ-05.011
---
# REQ-05.005 Manage Business Rules

## User Story
As a project contributor, I want to record and maintain business rules with their rationale and applicability so that rules can be understood in the context of requirements.

## Group
REQ-05 Project Documentation Management

## Criticality
Must have

## Description
`rule` is a `SingleMarkdown` document type (REQ-07.004): one record per plain ID `RUL-<n>`, with a Markdown body.

**Declared fields.** The type's field sequence is seed data and is not declared in project configuration (ADR-036 decisions 19, 26, and 29):

1. `description` — stored under the heading `## Description`, required, offered on `create` and `edit`.
2. `rationale` — stored under the heading `## Rationale`, required, offered on `create` only.
3. `applicability` — stored under the heading `## Applicability`, required, offered on `create` only.
4. `requirements` — stored under the heading `## Requirements`, kind `list`, option `requirement`, offered on `create` and `edit`.

All four fields are body-only: none declares a front-matter key, so their values live in the record body under the declared headings. The record title is required on create under the uniform title contract, is never cleared by an edit, and is not a field (RE-010 D13, REQ-05.010). Field sequence position is the order of the fields everywhere — body sections, CLI options and help, and the browser form (ADR-036 decision 20). No other field belongs to this type; adding or changing a field is a configuration file edit (REQ-07.002).

**Create and edit.** `papyrus rule create <TITLE> --description <TEXT> --rationale <TEXT> --applicability <TEXT>` creates a rule; a create that omits any required field produces one validation outcome and writes no file (ADR-036 decision 18). `papyrus rule edit <ID> --description <TEXT>` changes the description, and an edit that clears a required field is rejected with no write. Because `rationale` and `applicability` are offered on `create` only, they are read-only after creation: they are displayed in the same field sequence and are never writable through an edit option or an edit control (ADR-039 decision 22 and its clarification).

**Requirement references.** `requirements` is a repeatable list field: each `--requirement <REQ-ID>` value is one list item under `## Requirements`, and a rule may reference zero or more business requirements. On edit a reference is added through `--requirement` and removed by ID, or all references are removed at once; the same list is written on create. Labels and references remain general metadata supplied through `--label` and `--reference` (REQ-05.010). No create or edit option sets `status` (REQ-05.011).

**Lifecycle.** Rules carry front-matter `status` with the value `active`, a declared custom status, or `archived`; only the `status` and `archive` verbs write it (REQ-07.001). Archiving hides a rule from lists, search, and the web UI, moves it to the type's `archived/` subfolder, and preserves its ID (REQ-05.007). Deleting a rule permanently removes it under the document deletion rules (REQ-05.007). Reference visibility for this type is `visible`, so a list row renders its reference list (ADR-036 decision 29).

## Acceptance Criteria
- `rule` is a `SingleMarkdown` document type whose model comes from the seed table, with one record per plain ID `RUL-<n>` (REQ-07.004).
- The type declares exactly the fields `description`, `rationale`, `applicability`, and `requirements`, in that sequence order, each under the headings `## Description`, `## Rationale`, `## Applicability`, and `## Requirements`.
- `description`, `rationale`, and `applicability` are required; a create that omits any of them produces one validation outcome and writes no file.
- `rationale` and `applicability` declare the verb `create` only: they are absent from create-inapplicable help, are read-only after creation, and are never writable through an edit option or edit control.
- `description` declares the verbs `create` and `edit`; an edit changes it, and an edit that clears it is rejected with no write.
- `requirements` declares kind `list` and option `requirement`; each `--requirement <REQ-ID>` value becomes one list item under `## Requirements`.
- A rule may reference zero or more business requirements through `--requirement`, including none.
- On edit a requirement reference is added through `--requirement` and removed by ID, or all references are removed at once.
- All four fields are body-only, and no field declares a front-matter key.
- Field sequence position governs body section order, CLI option and help order, and browser form order.
- `papyrus rule create <TITLE>` takes the first positional `TITLE`, and `--title` is accepted on create and edit.
- A create without a title, with an empty title, with a multi-line title, or with both the positional `TITLE` and `--title` is rejected and writes no file; an edit cannot clear the title.
- No create or edit option sets `status`, and labels and references are supplied through `--label` and `--reference`.
- Rules can be listed, viewed, and edited for title, description, and requirement references, and are created with rationale and applicability.
- A list row for a `rule` record renders its reference list because the seed reference visibility of the type is `visible`.
- Archiving hides a rule from lists, search, and the web UI, moves it to the type's `archived/` subfolder, and preserves its ID (REQ-05.007).
- Deleting a rule permanently removes it under the document deletion rules (REQ-05.007).

## Traceability
- Source: customer-feedback handoff — in-session user feedback, Item 10; and the accepted owner decisions of RE-010 (not reopenable): D8 (reference visibility is seed data), D10 (presentation facts in the seed), D13 (title contract), D14 (fields are a sequence with no per-field order or enabled).
- ADR-036 — Config-declared document field schema and one descriptor model: decision 7 (single-Markdown rules), decisions 11-24 (field vocabulary, required rule, per-field verbs, sequence position, title contract, reserved keys), decision 29 (seed table: `rule` declares `description` required, `rationale` and `applicability` required with verbs `[create]`, `requirements` kind `list` option `requirement`), decision 32 (seed fill).
- ADR-039 — One descriptor-driven document view and edit surface: decision 15 (field map for a single-role type), decision 19 (field sections in descriptor order), decision 22 and its clarification (a field whose verbs exclude `edit` is rendered read-only and is never writable), decision 30 (display headings come from field labels), decision 33 (reference visibility affects list rows only).
- ADR-035 — Per-family reference visibility in the shared browser list layout, as amended by ADR-039 decision 8.
- ADR-0017 — Artifact category registry and type descriptors (decisions 5, 6, and 8 remain in force; decisions 1-4, 7, 9, and 10 replaced by ADR-036).
- ADR-0018 — Generic document content model (superseded in full by ADR-036).
- REQ-07.001 — Organize Artifacts into Three Categories (document status model).
- REQ-07.002 — Configure Document Types and Slugs (field items, `verbs`, `kind`, `required`).
- REQ-07.003 — Dispatch Config-Defined Document Commands (field options, repeatable `list`, per-field verbs, title contract).
- REQ-07.004 — Provide One Schema-Driven Document Content Model with Three Document Models (single-Markdown model, declared fields).
- REQ-05.007 — Manage Document Archive State and Deletion.
- REQ-05.010 — Record Metadata in YAML Front Matter (general keys, reserved title).
- REQ-05.011 — Maintain Record Metadata Through All Commands (`status` exemption, field options).
- BR-013 — Papyrus rule Command Behavior (needs revision for the seed-declared fields, per-field verbs, and the title contract).
