---
id: REQ-05.002
title: "Manage Non-Functional Requirements"
created_date: '2026-11-13T20:16:54Z'
updated_date: '2026-11-15T15:47:29Z'
labels: []
references: []
---
# REQ-05.002 Manage Non-Functional Requirements

## User Story
As a project contributor, I want to manage non-functional requirements in their own groups so that non-functional needs can be documented separately from business requirements.

## Group
REQ-05 Project Documentation Management

## Criticality
Must have

## Description
The user can create a non-functional-requirement group and create non-functional requirements only within a required group. Groups contain non-functional requirements only, and each requirement belongs to exactly one group. Requirements have a title and description and can be listed, viewed, and edited. They can be archived and deleted under the common document rules (REQ-05.007), and archiving preserves their ID. The family is addressed by the document slug `nfr` (`papyrus nfr`); the former `nfreq` command name is not retained (REQ-07.002).

## Acceptance Criteria
- A group can be created with a title and optional description.
- A non-functional requirement can be created only with its required non-functional-requirement group.
- A requirement belongs to exactly one group, and a group contains non-functional requirements only.
- Non-functional requirements can be listed, viewed, and edited for title and description.
- Archiving hides a requirement from lists, search, and the web UI, moves it to the type's `archived/` subfolder, and preserves its ID (REQ-05.007).
- Deleting a requirement permanently removes it under the document deletion rules (REQ-05.007).
- The family command is `papyrus nfr`; no `papyrus nfreq` alias exists.

## Traceability
- README.md, `papyrus(1)` → COMMANDS → nfreq and Documents lifecycle (lines 464–519, 778–784)
- Project-owner decision 2026-11-13: documents use `archived` and have no restore command; the slug is the command name, so the family is addressed as `nfr`.
- REQ-05.007 — Manage Document Archive State and Deletion
- REQ-07.002 — Configure Document Types and Slugs
- BR-010 — nfreq Command Behavior (needs revision for the slug-based command name)
