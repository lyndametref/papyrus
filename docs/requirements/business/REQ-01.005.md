# REQ-01.005 Assign Stable Record IDs

## User Story
As a project user, I want Papyrus to assign and preserve record identifiers so that records can be referred to consistently.

## Group
REQ-01 Project Setup and CLI Access

## Criticality
Must have

## Description
Papyrus generates record IDs automatically. IDs remain stable across archive/restore and deprecate/restore cycles. Configurable prefixes and padding determine the format for task, milestone, and release IDs; the man page gives `TASK-001`, `MS-001`, and `REL-001` as default examples.

## Acceptance Criteria
- Creating a supported record automatically assigns it an ID.
- Archiving and restoring a record, or deprecating and restoring a document, does not change its ID.
- With default configuration, task, milestone, and release IDs use the documented prefixes and three-digit examples (`TASK-001`, `MS-001`, `REL-001`); the configured prefix and padding control their format.

## Traceability
- README.md, `papyrus(1)` → DESCRIPTION (line 33), command record sections and FILES (lines 109–844)
- README.md, `papyrus-config(5)` → General and Documents keys (lines 949–962, 1007–1026)
