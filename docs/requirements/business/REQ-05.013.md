# REQ-05.013 Manage Test Cases

## User Story
As a project contributor, I want to create, find, and maintain Test Case documents so that test scenarios and verification activities can be documented and traced.

## Group
REQ-05 Project Documentation Management

## Criticality
Should have

## Description
Test cases are a document type that captures specific test scenarios, expected outcomes, and verification steps. They are managed with the `req` command and follow the same lifecycle as other documentation types (active → deprecated). Test cases can be listed, viewed, edited, deprecated, and restored with their ID preserved.

## Acceptance Criteria
- A test case can be created with the required title, description, and optional applicability.
- Test cases can be listed, viewed, and edited for title and description.
- Test cases can be deprecated and restored with their ID preserved.
- Test cases follow the same lifecycle as other documentation types (active → deprecated).
- The `req` command supports creating, listing, viewing, editing, deprecating, and restoring test cases.

## Traceability
- Source: customer-feedback handoff — Internal team discussion, Item 10 (medium priority)
- Raw quote: "add the following type of documents Use Cases, Test Cases and Test Report. Add the needed BR and edit the readme accordingly."
- REQ-05.007 — Manage Documentation Record Lifecycle (lifecycle applies to test cases)
- REQ-05.005 — Manage Business Rules (business rules can reference test cases)
- REQ-05.010 — Record Metadata in YAML Front Matter (front matter required for test cases)

## Open Questions
- Document type fields (prefix MOCK, directory, enabled, deprecate/restore) need to be defined for test cases
- Business rules should be able to reference test cases (already supported via requirement references)
- README sections/structure for new types need to be added
- Visibility in lists/search for test cases
- Association to tickets/milestones for test cases
