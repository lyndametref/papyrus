# REQ-05.006 Manage Audit Records

## User Story
As a project contributor, I want to record and maintain audit information so that the project can retain searchable audit records.

## Group
REQ-05 Project Documentation Management

## Criticality
Must have

## Description
The user can create audit records with a title and content, optionally specifying an audit type. Built-in types are `security` (default), `licence`, and `architecture`; additional types are configurable. Records can be listed, viewed, edited, deprecated, and restored.

## Acceptance Criteria
- An audit record can be created with the required title and content.
- The default audit type is `security`; `licence` and `architecture` are also built in, and additional types may be configured.
- Audit records can be listed, viewed, and edited for title, content, and type.
- Audit records can be deprecated and restored using their documented commands.

## Traceability
- README.md, `papyrus(1)` → COMMANDS → audit (lines 670–718)
- README.md, `papyrus-config(5)` → Documents keys (lines 1007–1026)
