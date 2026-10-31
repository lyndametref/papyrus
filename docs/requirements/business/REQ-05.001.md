# REQ-05.001 Manage Business Requirements

## User Story
As a project contributor, I want to create, find, and maintain business requirements in groups so that business needs can be documented and traced.

## Group
REQ-05 Project Documentation Management

## Criticality
Must have

## Description
The user can create a business-requirement group and create business requirements only within a required group. Groups contain business requirements only, and each requirement belongs to exactly one group. Requirements contain a title, user story, and description, and can be listed, viewed, edited, deprecated, and restored with their ID preserved.

## Acceptance Criteria
- A group can be created with a title, optional user story, and optional description.
- A business requirement can be created only with its required business-requirement group.
- A requirement belongs to exactly one group, and a group contains business requirements only.
- Requirements can be listed, viewed, and edited for title, user story, and description.
- Deprecation hides a requirement from lists, search, and the web UI; restoration returns it to its original folder with the same ID.

## Traceability
- README.md, `papyrus(1)` → COMMANDS → req and Documents lifecycle (lines 396–462, 778–784)
