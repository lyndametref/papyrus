# REQ-05.012 Manage Use Cases

## User Story
As a project contributor, I want to create, find, and maintain Use Case documents so that behavioral requirements and user journeys can be documented and traced.

## Group
REQ-05 Project Documentation Management

## Criticality
Should have

## Description
Use cases are a document type that captures functional behavior and user interactions. They are managed with the `req` command and follow the same lifecycle as other documentation types (active → deprecated). Use cases can be listed, viewed, edited, deprecated, and restored with their ID preserved.

## Acceptance Criteria
- A use case can be created with the required title, description, and optional applicability.
- Use cases can be listed, viewed, and edited for title and description.
- Use cases can be deprecated and restored with their ID preserved.
- Use cases follow the same lifecycle as other documentation types (active → deprecated).
- The `req` command supports creating, listing, viewing, editing, deprecating, and restoring use cases.

## Traceability
- Source: customer-feedback handoff — Internal team discussion, Item 10 (medium priority)
- Raw quote: "add the following type of documents Use Cases, Test Cases and Test Report. Add the needed BR and edit the readme accordingly."
- REQ-05.007 — Manage Documentation Record Lifecycle (lifecycle applies to use cases)
- REQ-05.005 — Manage Business Rules (business rules can reference use cases)
- REQ-05.010 — Record Metadata in YAML Front Matter (front matter required for use cases)

## Open Questions
- Document type fields (prefix MOCK, directory, enabled, deprecate/restore) need to be defined for use cases
- Business rules should be able to reference use cases (already supported via requirement references)
- README sections/structure for new types need to be added
- Visibility in lists/search for use cases
- Association to tickets/milestones for use cases
