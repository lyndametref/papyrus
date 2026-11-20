---
id: REQ-05.004
title: "Manage UI Specifications"
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
# REQ-05.004 Manage UI Specifications

## User Story
As a project contributor, I want to maintain UI specifications associated with views so that UI documentation can be found and updated.

## Group
REQ-05 Project Documentation Management

## Criticality
Must have

## Description
`ui-spec` is a `SingleMarkdown` document type (REQ-07.004): one record per plain ID `UI-<n>`, stored in the type's configured directory, with a Markdown body.

**Declared fields.** The type's field set is seed data and is not declared in project configuration (ADR-036 decisions 26 and 29):

- `view` — stored under the heading `## View`.
- `description` — stored under the heading `## Description`.

Both fields are optional and body-only: either may be omitted or cleared, and each value is stored in the record body under its declared heading. The seed marks neither field required, so a create that supplies only a title is valid; the record title is required on create under the uniform title contract and is not a field (RE-010 D13, REQ-05.010). Field sequence position is the order of the fields everywhere — body sections, CLI options and help, and the browser form (ADR-036 decision 20). No other field belongs to this type; adding or changing a field is a configuration file edit (REQ-07.002).

**Title and options.** `papyrus ui-spec create <TITLE>` creates a specification, and `--title` is accepted on create and edit. The title is single-line and non-empty, is required on create, and can never be cleared by an edit; supplying both the positional `TITLE` and `--title` is rejected and writes nothing (REQ-07.003). The field options are `--view` and `--description`; a `multiline` field accepts `-` to read the value from standard input. Labels and references remain general metadata supplied through `--label` and `--reference` (REQ-05.010). No create or edit option sets `status` (REQ-05.011).

**Lifecycle.** Specifications carry front-matter `status` with the value `active`, a declared custom status, or `archived`; only the `status` and `archive` verbs write it (REQ-07.001). Archiving hides a specification from lists, search, and the web UI, moves it to the type's `archived/` subfolder, and preserves its ID (REQ-05.007). Deleting a specification permanently removes it under the document deletion rules (REQ-05.007). Reference visibility for this type is `visible`, so a list row renders its reference list (ADR-036 decision 29).

## Acceptance Criteria
- `ui-spec` is a `SingleMarkdown` document type whose model comes from the seed table, with one record per plain ID `UI-<n>` (REQ-07.004).
- The type declares exactly the fields `view` under the heading `## View` and `description` under the heading `## Description`, in that sequence order.
- Neither field is required: a create that supplies only a title succeeds, and an edit may supply or clear either field.
- Each field value is stored in the record body under its declared heading, and neither field declares a front-matter key.
- Field sequence position governs body section order, CLI option and help order, and browser form order.
- `papyrus ui-spec create <TITLE>` creates a specification; the title is the first positional `TITLE`, and `--title` is accepted on create and edit.
- A create without a title, with an empty title, with a multi-line title, or with both the positional `TITLE` and `--title` is rejected and writes no file; an edit cannot clear the title.
- The field options are `--view` and `--description`, and a `multiline` field option accepts `-` to read the value from standard input.
- No create or edit option sets `status`, and labels and references are supplied through `--label` and `--reference`.
- UI specifications can be listed, viewed, and edited for title, view, and description.
- A list row for a `ui-spec` record renders its reference list because the seed reference visibility of the type is `visible`.
- Archiving hides a specification from lists, search, and the web UI, moves it to the type's `archived/` subfolder, and preserves its ID (REQ-05.007).
- Deleting a specification permanently removes it under the document deletion rules (REQ-05.007).
- A `ui-spec` record carries front-matter `status`, written only by the `status` and `archive` verbs (REQ-07.001, REQ-05.011).

## Traceability
- Source: customer-feedback handoff — in-session user feedback, Item 10; and the accepted owner decisions of RE-010 (not reopenable): D5 and D6 (three document models; `ui-spec` is a seed `SingleMarkdown` type), D8 (reference visibility is seed data), D10 (presentation facts in the seed), D13 (title contract).
- ADR-036 — Config-declared document field schema and one descriptor model: decision 7 (single-Markdown rules), decisions 11-24 (field vocabulary, required rule, title contract, reserved keys), decision 29 (seed table: `ui-spec` declares `view` and `description`, neither required, reference visibility visible), decision 32 (seed fill).
- ADR-039 — One descriptor-driven document view and edit surface: decision 15 (field map for a single-role type), decision 19 (field sections in descriptor order), decision 22 (edit form order), decision 30 (display headings come from field labels), decision 33 (reference visibility affects list rows only).
- ADR-035 — Per-family reference visibility in the shared browser list layout, as amended by ADR-039 decision 8.
- ADR-033 — Uniform single-document browser view layout for managed Markdown records, as amended by ADR-039 decision 7.
- ADR-0017 — Artifact category registry and type descriptors (decisions 5, 6, and 8 remain in force; decisions 1-4, 7, 9, and 10 replaced by ADR-036).
- ADR-0018 — Generic document content model (superseded in full by ADR-036).
- REQ-07.001 — Organize Artifacts into Three Categories (document status model).
- REQ-07.002 — Configure Document Types and Slugs (entry keys, field items, seed fill).
- REQ-07.003 — Dispatch Config-Defined Document Commands (title contract, field options, `multiline` and `-`).
- REQ-07.004 — Provide One Schema-Driven Document Content Model with Three Document Models (single-Markdown model, optional fields).
- REQ-05.007 — Manage Document Archive State and Deletion.
- REQ-05.010 — Record Metadata in YAML Front Matter (general keys, `status`, reserved title).
- REQ-05.011 — Maintain Record Metadata Through All Commands (`status` exemption, field options).
- BR-012 — Papyrus ui-spec Command Behavior (needs revision for the seed-declared fields and the title contract).
