---
id: REQ-05.013
title: "Manage Test Cases"
created_date: '2026-11-13T20:17:45Z'
updated_date: '2026-11-20T20:45:09Z'
labels: []
references:
  - RE-010
  - ADR-036
  - ADR-039
  - ADR-033
  - REQ-07.001
  - REQ-07.002
  - REQ-07.003
  - REQ-07.004
  - REQ-05.007
  - REQ-05.010
  - REQ-05.011
---
# REQ-05.013 Manage Test Cases

## User Story
As a project contributor, I want to create, find, and maintain Test Case documents so that test scenarios and verification activities can be documented and traced.

## Group
REQ-05 Project Documentation Management

## Criticality
Should have

## Description
`test-case` is a `SingleMarkdown` document type (REQ-07.004): one record per plain ID `TC-<n>`, stored under `test-cases/` below `doc_directory` by default, with a Markdown body. Test cases capture specific test scenarios, expected outcomes, and verification steps.

**Declared fields.** The type's field set is seed data and is not declared in project configuration (ADR-036 decisions 26 and 29):

- `description` — stored under the heading `## Description`.
- `applicability` — stored under the heading `## Applicability`.

Both fields are optional and body-only: either may be omitted or cleared, and each value is stored in the record body under its declared heading, never in YAML front matter. Field sequence position is the order of the fields everywhere — body sections, CLI options and help, and the browser form (ADR-036 decision 20). The record title is required on create under the uniform title contract and is not a field (RE-010 D13, REQ-05.010). No other field belongs to this type; adding or changing a field is a configuration file edit (REQ-07.002).

**Title and options.** `papyrus test-case create <TITLE> --description <TEXT> --applicability <TEXT>` creates a test case; `--title` is accepted on create and edit. The title is single-line and non-empty, is required on create, and can never be cleared by an edit; supplying both the positional `TITLE` and `--title` is rejected and writes nothing (REQ-07.003). `--description` and `--applicability` are the field options, and a `multiline` field option accepts `-` to read the value from standard input. Labels and references remain general metadata supplied through `--label` and `--reference` (REQ-05.010). No create or edit option sets `status` (REQ-05.011).

**Command family.** The verbs are derived from the `SingleMarkdown` model: `create`, `list`, `view`, `edit`, `archive`, `delete`, and `status` (REQ-07.003). The slug and defaults come from the `test-case` configuration entry (REQ-07.002).

**Lifecycle.** Test cases carry front-matter `status` with the value `active`, a declared custom status, or `archived`; only the `status` and `archive` verbs write it (REQ-07.001). Archiving hides a test case from lists, search, and the web UI, moves it to the type's `archived/` subfolder, and preserves its ID (REQ-05.007). Deletion permanently removes the record with confirmation, and `--force` skips the prompt (REQ-05.007). Reference visibility for this type is `visible`, so a list row renders its reference list (ADR-036 decision 29).

## Acceptance Criteria
- `test-case` is a `SingleMarkdown` document type whose model comes from the seed table, with one record per plain ID `TC-<n>` (REQ-07.004).
- The type declares exactly the fields `description` under the heading `## Description` and `applicability` under the heading `## Applicability`, in that sequence order.
- Both fields are optional: a create that supplies only a title succeeds, and an edit may supply or clear either field.
- Each field value is stored in the record body under its declared heading, and neither field declares a front-matter key.
- Field sequence position governs body section order, CLI option and help order, and browser form order.
- `papyrus test-case create <TITLE>` creates a test case with the first positional `TITLE`, and `--title` is accepted on create and edit.
- A create without a title, with an empty title, with a multi-line title, or with both the positional `TITLE` and `--title` is rejected and writes no file; an edit cannot clear the title.
- `--description` and `--applicability` are the field options, and a `multiline` field option accepts `-` to read the value from standard input.
- No create or edit option sets `status`, and labels and references are supplied through `--label` and `--reference`.
- Test cases can be listed, viewed, and edited for title and description.
- The command family is `papyrus test-case create|list|view|edit|archive|delete|status`, derived from the `SingleMarkdown` model, with the slug and defaults defined by the `test-case` configuration entry (REQ-07.002, REQ-07.003).
- A list row for a `test-case` record renders its reference list because the seed reference visibility of the type is `visible`.
- Test cases can be archived; archiving hides them from lists, search, and the web UI, moves them to the type's `archived/` subfolder, and preserves their ID (REQ-05.007).
- Test cases can be permanently deleted with confirmation, and `--force` skips the prompt (REQ-05.007).
- Test cases have no restore command (REQ-05.007).

## Traceability
- Source: customer-feedback handoff — Internal team discussion, Item 10 (medium priority)
- Raw quote: "add the following type of documents Use Cases, Test Cases and Test Report. Add the needed BR and edit the readme accordingly."
- Project-owner decision 2026-11-13: document state `archived` replaces `deprecated`, `archive` replaces `deprecate`, and no restore command exists (REQ-05.007); the command name is the document slug.
- RE-010 — Descriptor-driven document engine refactor (accepted, not reopenable): D6 (`test-case` is a seed `SingleMarkdown` type), D8 (reference visibility is seed data), D10 (presentation facts in the seed), D13 (title contract).
- ADR-036 — Config-declared document field schema and one descriptor model: decision 7 (single-Markdown rules and model-derived verbs), decisions 11-24 (field vocabulary, required rule, title contract, reserved keys), decision 29 (seed table: `test-case` declares `description` under `## Description` and `applicability` under `## Applicability`, reference visibility visible), decision 32 (seed fill).
- ADR-039 — One descriptor-driven document view and edit surface: decision 15 (field map for a single-role type), decision 19 (field sections in descriptor order), decision 22 (edit form order), decision 33 (reference visibility affects list rows only).
- ADR-033 — Uniform single-document browser view layout for managed Markdown records, as amended by ADR-039 decision 7.
- ADR-0017 — Artifact category registry and type descriptors (decisions 5, 6, and 8 remain in force; decisions 1-4, 7, 9, and 10 replaced by ADR-036).
- ADR-0018 — Generic document content model (superseded in full by ADR-036).
- REQ-05.007 — Manage Document Archive State and Deletion
- REQ-05.010 — Record Metadata in YAML Front Matter (front matter required for test cases)
- REQ-07.001 — Organize Artifacts into Three Categories (document status model)
- REQ-07.002 — Configure Document Types and Slugs
- REQ-07.003 — Dispatch Config-Defined Document Commands (model-derived verbs, title contract, field options)
- REQ-07.004 — Provide One Schema-Driven Document Content Model with Three Document Models (single-Markdown model)
- BR-019 — Test-Case Document Type Behavior (needs revision for the seed-declared fields and the title contract)
