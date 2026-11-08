# REQ-06.003 Describe Managed Artifacts and Their Lifecycles in the Browser

## User Story
As a project contributor, I want a browser page that describes every artifact and document managed in the project and its lifecycle so that I can understand what each record type is and how it moves through its states.

## Group
REQ-06 Search and Local Web Interface

## Criticality
Should have

## Description
The local web interface provides a page that describes the artifact and document types managed in the current project, with one section per type, and explains the lifecycle of each type. The page covers a fixed set of record types: tasks, milestones, releases, decisions, business requirements, non-functional requirements, glossary terms, UI specifications, technical specifications, UI mockups, business rules, and audit records. The set is fixed by this requirement and is not derived from the configured `document_types`. The task section's states and transitions are derived from the configured statuses (REQ-01.002); every other type uses the lifecycle documented for that type. Backlog.md `docs`, `drafts`, and project configuration files are not covered. The page route, navigation entry, and format are design decisions and are not constrained by this requirement.

## Acceptance Criteria
- The local web interface serves a page that describes every managed artifact/document type in a separate section.
- The page covers tasks, milestones, releases, decisions, business requirements, non-functional requirements, glossary terms, UI specifications, technical specifications, UI mockups, business rules, and audit records.
- The covered set is fixed and does not change with the configured `document_types`.
- The task section's states and transitions reflect the configured task statuses.
- Each other section explains the lifecycle documented for its type, including the states and the transitions between states, and agrees with that type's lifecycle requirements (REQ-03.002, REQ-04.001, REQ-05.007).
- Backlog.md `docs` and `drafts` records and project configuration files are not described by the page.

## Traceability
- Source: customer-feedback handoff — Internal team discussion, Item 5 (medium), raw quote:
  - "on the browser add a page describing all the artifacts/document managed in the current project (one section each) and explain the life cycle of each type"
- User decision 2026-11-08: fixed record-type set independent of configured `document_types`, task states derived from configured statuses, other types use their documented lifecycles, route/format left to design.
- REQ-01.002 — Read and Edit Project Configuration (configured task statuses)
- REQ-06.002 — Serve the Local Web Interface
- REQ-05 — Project Documentation Management (managed record types)
- REQ-02 — Task and Board Management (task lifecycle)
- REQ-03.002 — Manage Releases (release lifecycle)
- REQ-04.001 — Create and Inspect Decisions (decision statuses)
- REQ-05.007 — Manage Documentation Record Lifecycle
- NFR-01.002 — Display, View, and Edit All Papyrus Content in a Local Browser
