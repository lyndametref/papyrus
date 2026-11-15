---
id: REQ-05.014
title: "Manage Test Reports"
created_date: '2026-11-13T20:17:45Z'
updated_date: '2026-11-15T15:47:29Z'
labels: []
references: []
---
# REQ-05.014 Manage Test Reports

## User Story
As a project contributor, I want to create, find, and maintain Test Report documents so that test execution results, pass/fail outcomes, and verification evidence can be documented and traced.

## Group
REQ-05 Project Documentation Management

## Criticality
Should have

## Description
Test reports are a document type declared by the default `document_types` entry `test-report`, with prefix `TR`, stored under `test-reports/` below `doc_directory`, and managed by the command `papyrus test-report`. Test reports capture the results of test executions, including pass/fail status, test case coverage, and verification evidence. They follow the document lifecycle `active -> archived` (REQ-05.007) and can be listed, viewed, edited, archived, and deleted with their ID preserved until deletion.

## Acceptance Criteria
- A test report can be created with the required title and description and optional applicability.
- Test reports can be listed, viewed, and edited for title and description.
- Test reports can be archived; archiving hides them from lists, search, and the web UI, moves them to the type's `archived/` subfolder, and preserves their ID (REQ-05.007).
- Test reports can be permanently deleted with confirmation, and `--force` skips the prompt (REQ-05.007).
- Test reports follow the document lifecycle `active -> archived` and have no restore command.
- The command family is `papyrus test-report create|list|view|edit|archive|delete`, with the slug and defaults defined by the `test-report` configuration entry (REQ-07.002, REQ-07.003).

## Traceability
- Source: customer-feedback handoff — Internal team discussion, Item 10 (medium priority)
- Raw quote: "add the following type of documents Use Cases, Test Cases and Test Report. Add the needed BR and edit the readme accordingly."
- Project-owner decision 2026-11-13: document state `archived` replaces `deprecated`, `archive` replaces `deprecate`, and no restore command exists (REQ-05.007); the command name is the document slug.
- REQ-05.007 — Manage Document Archive State and Deletion
- REQ-05.010 — Record Metadata in YAML Front Matter (front matter required for test reports)
- REQ-07.002 — Configure Document Types and Slugs
- REQ-07.003 — Dispatch Config-Defined Document Commands
