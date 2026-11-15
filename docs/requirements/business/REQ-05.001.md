---
id: REQ-05.001
title: "Manage Business Requirements"
created_date: '2026-11-13T20:16:44Z'
updated_date: '2026-11-15T15:47:29Z'
labels: []
references: []
---
# REQ-05.001 Manage Business Requirements

## User Story
As a project contributor, I want to create, find, and maintain business requirements in groups so that business needs can be documented and traced.

## Group
REQ-05 Project Documentation Management

## Criticality
Must have

## Description
The user can create a business-requirement group and create business requirements only within a required group. Groups contain business requirements only, and each requirement belongs to exactly one group. Requirements contain a title, user story, and description, and can be listed, viewed, and edited. They can be archived and deleted under the common document rules (REQ-05.007), and archiving preserves their ID.

## Acceptance Criteria
- A group can be created with a title, optional user story, and optional description.
- A business requirement can be created only with its required business-requirement group.
- A requirement belongs to exactly one group, and a group contains business requirements only.
- Requirements can be listed, viewed, and edited for title, user story, and description.
- Archiving hides a requirement from lists, search, and the web UI, moves it to the type's `archived/` subfolder, and preserves its ID (REQ-05.007).
- Deleting a requirement permanently removes it under the document deletion rules (REQ-05.007).

## Traceability
- README.md, `papyrus(1)` → COMMANDS → req and Documents lifecycle (lines 396–462, 778–784)
- Project-owner decision 2026-11-13: document state `archived` replaces `deprecated`, and no restore command exists (REQ-05.007).
- REQ-05.007 — Manage Document Archive State and Deletion
- REQ-07.002 — Configure Document Types and Slugs
