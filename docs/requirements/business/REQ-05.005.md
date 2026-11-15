---
id: REQ-05.005
title: "Manage Business Rules"
created_date: '2026-11-13T20:17:02Z'
updated_date: '2026-11-15T15:47:29Z'
labels: []
references: []
---
# REQ-05.005 Manage Business Rules

## User Story
As a project contributor, I want to record and maintain business rules with their rationale and applicability so that rules can be understood in the context of requirements.

## Group
REQ-05 Project Documentation Management

## Criticality
Must have

## Description
The user can create rules with a title, description, rationale, and applicability, optionally referencing business requirements. Rules can be listed, viewed, and edited. They can be archived and deleted under the common document rules (REQ-05.007). Requirement references are repeatable and can be removed individually or all at once.

## Acceptance Criteria
- Creating a rule requires description, rationale, and applicability; title is supplied as the command title.
- A rule may reference zero or more business requirements.
- Rules can be listed, viewed, and edited for title, description, and requirement references.
- A requirement reference can be removed by ID or all references can be removed.
- Archiving hides a rule from lists, search, and the web UI, moves it to the type's `archived/` subfolder, and preserves its ID (REQ-05.007).
- Deleting a rule permanently removes it under the document deletion rules (REQ-05.007).

## Traceability
- README.md, `papyrus(1)` → COMMANDS → rule and Documents lifecycle (lines 612–668, 778–784)
- Project-owner decision 2026-11-13: document state `archived` replaces `deprecated`, and no restore command exists (REQ-05.007).
- REQ-05.007 — Manage Document Archive State and Deletion
- REQ-07.002 — Configure Document Types and Slugs
