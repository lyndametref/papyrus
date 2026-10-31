# REQ-05.002 Manage Non-Functional Requirements

## User Story
As a project contributor, I want to manage non-functional requirements in their own groups so that non-functional needs can be documented separately from business requirements.

## Group
REQ-05 Project Documentation Management

## Criticality
Must have

## Description
The user can create a non-functional-requirement group and create non-functional requirements only within a required group. Groups contain non-functional requirements only, and each requirement belongs to exactly one group. Requirements have a title and description and can be listed, viewed, edited, deprecated, and restored with their ID preserved.

## Acceptance Criteria
- A group can be created with a title and optional description.
- A non-functional requirement can be created only with its required non-functional-requirement group.
- A requirement belongs to exactly one group, and a group contains non-functional requirements only.
- Non-functional requirements can be listed, viewed, and edited for title and description.
- Deprecation hides a requirement from lists, search, and the web UI; restoration returns it to its original folder with the same ID.

## Traceability
- README.md, `papyrus(1)` → COMMANDS → nfreq and Documents lifecycle (lines 464–519, 778–784)
