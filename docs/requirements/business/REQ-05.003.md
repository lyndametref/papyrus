---
id: REQ-05.003
title: "Manage Glossary Terms"
created_date: '2026-11-13T20:16:54Z'
updated_date: '2026-11-20T20:44:54Z'
labels: []
references:
  - RE-010
  - ADR-036
  - ADR-039
  - REQ-07.001
  - REQ-07.002
  - REQ-07.003
  - REQ-07.004
  - REQ-05.007
  - REQ-05.010
  - REQ-05.011
---
# REQ-05.003 Manage Glossary Terms

## User Story
As a project contributor, I want to maintain glossary terms and definitions so that project terminology can be looked up consistently.

## Group
REQ-05 Project Documentation Management

## Criticality
Must have

## Description
`glossary` is a `SingleMarkdown` document type (REQ-07.004): one record per plain ID `GLO-<n>`, with a Markdown body.

**The term is the record title.** Under the uniform title contract, a glossary term is the record title (RE-010 D13, REQ-07.003, ADR-036 decision 24). `papyrus glossary create <TERM> --definition <TEXT>` creates a term. The term is single-line and non-empty, is required on create, and can never be cleared by an edit; `papyrus glossary edit <ID> --title <TERM>` renames it. The former `--term` option is removed: no document command accepts `--term`, and a glossary command invocation that supplies it is a usage error. The term remains the canonical name of one business concept and the definition its meaning; the record ID is the term's identity, so renaming a term changes the displayed wording and leaves the ID, the file location, and every reference by ID unchanged.

**Definition field.** The type declares exactly one field, `definition`, stored under the heading `## Definition` and marked required (ADR-036 decision 29). `--definition` is the field's option, and because the field is multiline the option also accepts `-` to read the value from standard input (REQ-07.003). A create that omits the definition, or an edit that clears it, is rejected with one validation outcome and writes no file (ADR-036 decision 18). The field is body-only: it declares no front-matter key, so the definition lives in the body and never in YAML front matter. The definition can be edited with `--definition` or with `--editor`.

**Title and field order.** The title is fixed and is not a field; `definition` is the only field, so it is the only body section the engine writes for this type besides the writer-generated `# <ID> <title>` line (REQ-05.010). Labels and references remain general metadata supplied through `--label` and `--reference` (REQ-05.010).

**Lifecycle.** Terms carry front-matter `status` with the value `active`, a declared custom status, or `archived`; only the `status` and `archive` verbs write it (REQ-07.001, REQ-05.011). Archiving hides a term from lists, search, and the web UI, moves it to the type's `archived/` subfolder, and preserves its ID (REQ-05.007). Deleting a term permanently removes it under the document deletion rules (REQ-05.007).

## Acceptance Criteria
- `glossary` is a `SingleMarkdown` document type whose model comes from the seed table, with one record per plain ID `GLO-<n>` (REQ-07.004).
- A glossary term is the record title: `papyrus glossary create <TERM> --definition <TEXT>` creates a term whose title is the term.
- The term is single-line and non-empty, is required on create, and an edit can never clear it; `--title` on edit renames the term.
- No `--term` option exists on any document command, and a glossary create or edit that supplies `--term` is a usage error.
- The type declares exactly one field, `definition`, under the heading `## Definition`, marked required.
- `--definition` is the field option, accepts `-` to read the value from standard input, and the definition may also be edited through `--editor`.
- A create that omits the definition, or an edit that clears it, is rejected with one validation outcome and writes no file.
- `definition` declares no front-matter key, so the value is stored only in the body under `## Definition`.
- Renaming the title leaves the record ID, the file location, and every reference by ID unchanged.
- Terms can be listed, viewed, and edited for term (title) and definition.
- Archiving hides a term from lists, search, and the web UI, moves it to the type's `archived/` subfolder, and preserves its ID (REQ-05.007).
- Deleting a term permanently removes it under the document deletion rules (REQ-05.007).
- A glossary record carries front-matter `status`, written only by the `status` and `archive` verbs (REQ-07.001, REQ-05.011).

## Open Questions
- Whether the term text must be unique across the glossary. This requirement fixes the record ID as the term's identity and adds no uniqueness comparison on the title; the title contract requires only a single-line, non-empty value.

## Traceability
- Source: customer-feedback handoff — in-session user feedback, Item 10; and the accepted owner decisions of RE-010 (not reopenable): D13 (title is fixed and non-configurable, uniform positional `TITLE` and `--title`, `--term` removed) and D6 (only `req` and `nfr` are grouped, so `glossary` is `SingleMarkdown`).
- ADR-036 — Config-declared document field schema and one descriptor model: decision 7 (single-Markdown rules), decisions 11-24 (field vocabulary, required rule, title contract and reserved keys), decision 29 (seed table: `glossary` declares `definition` under `## Definition`, required), decision 32 (seed fill).
- ADR-039 — One descriptor-driven document view and edit surface: decision 15 (field map for a single-role type), decision 19 (one section per non-empty field, an empty required field rendered as an empty section), decision 22 (edit form order and required marker), decision 30 (the display heading is the field label).
- ADR-0017 — Artifact category registry and type descriptors (decisions 5, 6, and 8 remain in force; decisions 1-4, 7, 9, and 10 replaced by ADR-036).
- ADR-0018 — Generic document content model (superseded in full by ADR-036).
- REQ-07.001 — Organize Artifacts into Three Categories (document status model).
- REQ-07.002 — Configure Document Types and Slugs (field items and the required flag).
- REQ-07.003 — Dispatch Config-Defined Document Commands (field options, `multiline` and `-`, title contract, no `--term`).
- REQ-07.004 — Provide One Schema-Driven Document Content Model with Three Document Models (single-Markdown model).
- REQ-01.004 — Use Common CLI Controls and Outcomes (validation and usage outcomes).
- REQ-05.007 — Manage Document Archive State and Deletion.
- REQ-05.010 — Record Metadata in YAML Front Matter (title as a reserved key, general metadata).
- REQ-05.011 — Maintain Record Metadata Through All Commands (`status` exemption, field options).
- BR-011 — Papyrus glossary Command Behavior (needs revision: the title contract replaces `--term`).
