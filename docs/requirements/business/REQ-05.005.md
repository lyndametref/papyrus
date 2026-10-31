# REQ-05.005 Manage Business Rules

## User Story
As a project contributor, I want to record and maintain business rules with their rationale and applicability so that rules can be understood in the context of requirements.

## Group
REQ-05 Project Documentation Management

## Criticality
Must have

## Description
The user can create rules with a title, description, rationale, and applicability, optionally referencing business requirements. Rules can be listed, viewed, edited, deprecated, and restored. Requirement references are repeatable and can be removed individually or all at once.

## Acceptance Criteria
- Creating a rule requires description, rationale, and applicability; title is supplied as the command title.
- A rule may reference zero or more business requirements.
- Rules can be listed, viewed, and edited for title, description, and requirement references.
- A requirement reference can be removed by ID or all references can be removed.
- Deprecation hides a rule from lists, search, and the web UI; restoration returns it to its original folder with the same ID.

## Traceability
- README.md, `papyrus(1)` → COMMANDS → rule and Documents lifecycle (lines 612–668, 778–784)
