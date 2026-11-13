# REQ-05.012 Manage Use Cases

## User Story
As a project contributor, I want to create, find, and maintain Use Case documents so that behavioral requirements and user journeys can be documented and traced.

## Group
REQ-05 Project Documentation Management

## Criticality
Should have

## Description
Use cases are a document type declared by the default `document_types` entry `use-case`, with prefix `UC`, stored under `use-cases/` below `doc_directory`, and managed by the command `papyrus use-case`. Use cases capture functional behavior and user interactions. They follow the document lifecycle `active -> archived` (REQ-05.007) and can be listed, viewed, edited, archived, and deleted with their ID preserved until deletion.

## Acceptance Criteria
- A use case can be created with the required title and description and optional applicability.
- Use cases can be listed, viewed, and edited for title and description.
- Use cases can be archived; archiving hides them from lists, search, and the web UI, moves them to the type's `archived/` subfolder, and preserves their ID (REQ-05.007).
- Use cases can be permanently deleted with confirmation, and `--force` skips the prompt (REQ-05.007).
- Use cases follow the document lifecycle `active -> archived` and have no restore command.
- The command family is `papyrus use-case create|list|view|edit|archive|delete`, with the slug and defaults defined by the `use-case` configuration entry (REQ-07.002, REQ-07.003).

## Traceability
- Source: customer-feedback handoff — Internal team discussion, Item 10 (medium priority)
- Raw quote: "add the following type of documents Use Cases, Test Cases and Test Report. Add the needed BR and edit the readme accordingly."
- Project-owner decision 2026-11-13: document state `archived` replaces `deprecated`, `archive` replaces `deprecate`, and no restore command exists (REQ-05.007); the command name is the document slug.
- REQ-05.007 — Manage Document Archive State and Deletion
- REQ-05.010 — Record Metadata in YAML Front Matter (front matter required for use cases)
- REQ-07.002 — Configure Document Types and Slugs
- REQ-07.003 — Dispatch Config-Defined Document Commands
