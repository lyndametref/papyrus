# REQ-04.001 Create and Inspect Decisions

## User Story
As a project decision-maker, I want to create and inspect decisions with context and consequences so that decisions are recorded for future reference.

## Group
REQ-04 Decision Records

## Criticality
Must have

## Description
The user can create and list decisions, optionally filtering by type, and view a decision. A decision records a title, decision type, decision, context, and consequences. Built-in types are `adr` and `bdr`; additional types may be configured. Decisions may reference zero or more business and non-functional requirements. Records follow the Nygard ADR template and decision status vocabulary described in the man page.

## Acceptance Criteria
- A decision can be created with the documented title, type, decision, context, and consequences fields.
- The default decision type is `adr`; `adr` and `bdr` are the built-in types.
- Decisions can be listed and viewed; listing can be filtered by type.
- Decision records expose the documented Nygard fields and status values: proposed, accepted, rejected, deprecated, and superseded.
- Status transitions shown in the man page are proposed to accepted or rejected, and accepted to deprecated or superseded.
- A decision may reference zero or more business requirements and zero or more non-functional requirements.

## Traceability
- README.md, `papyrus(1)` → COMMANDS → decision (lines 334–394), STATUSES → Decisions (lines 771–777)
- README.md, `papyrus-config(5)` → Decisions keys (lines 994–1005)
