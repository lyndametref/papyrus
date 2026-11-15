---
id: REQ-05.003
title: "Manage Glossary Terms"
created_date: '2026-11-13T20:16:54Z'
updated_date: '2026-11-15T15:47:29Z'
labels: []
references: []
---
# REQ-05.003 Manage Glossary Terms

## User Story
As a project contributor, I want to maintain glossary terms and definitions so that project terminology can be looked up consistently.

## Group
REQ-05 Project Documentation Management

## Criticality
Must have

## Description
The user can create glossary terms with a term and definition, list and view terms, and edit either field. Terms can be archived and deleted under the common document rules (REQ-05.007). Archived terms are hidden from lists, search, and the web UI, and archiving preserves their IDs.

## Acceptance Criteria
- A glossary term can be created with the required term and definition.
- Terms can be listed, viewed, and edited for term and definition.
- Archiving hides a term from lists, search, and the web UI, moves it to the type's `archived/` subfolder, and preserves its ID (REQ-05.007).
- Deleting a term permanently removes it under the document deletion rules (REQ-05.007).

## Traceability
- README.md, `papyrus(1)` → COMMANDS → glossary and Documents lifecycle (lines 520–564, 778–784)
- Project-owner decision 2026-11-13: document state `archived` replaces `deprecated`, and no restore command exists (REQ-05.007).
- REQ-05.007 — Manage Document Archive State and Deletion
- REQ-07.002 — Configure Document Types and Slugs
