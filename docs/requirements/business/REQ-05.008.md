# REQ-05.008 Manage UI Mockups

## User Story
As a project contributor, I want to maintain UI mockups associated with UI specifications and tickets so that UI design artifacts can be found, reviewed, approved, and updated.

## Group
REQ-05 Project Documentation Management

## Criticality
Must have

## Description
The user can create a UI mockup as a document type with prefix MOCK stored under `docs/specifications/ui/mockups/`. A mockup realizes a UI specification: it carries a title, an associated task, references to other documents, and an implementation reference to one or more UI specifications by spec id. Mockups are editable, are not versioned by Papyrus (version control is provided by git), and can be associated to tickets (the ticket is recorded in the mockup and the mockup is referenced by the ticket). Mockups can be listed, viewed, and edited; they can be deprecated and restored with their ID preserved, with deprecation visibility matching UI specifications (hidden from lists, search, and the web UI). A mockup's status is one of: pending (waiting user approval), approved, or deprecated. A mockup may supersede an older mockup; the superseded mockup becomes deprecated and records a back-reference to its successor. Status transitions are blocked while a mockup is superseded. Restore copies a deprecated mockup back to its original folder and updates its status field in the HTML.

## Acceptance Criteria
- A mockup can be created with the required title, associated task, and reference(s) to UI specification(s) by spec id.
- Mockups are stored under `docs/specifications/ui/mockups/` with IDs using the MOCK prefix.
- A mockup records its associated ticket and references to other documents within its HTML content.
- Mockups can be listed, viewed, and edited.
- A mockup's status is one of pending (waiting user approval), approved, or deprecated.
- Deprecation hides a mockup from lists, search, and the web UI; restoration returns it to its original folder with its ID preserved and updates its status field in the HTML.
- A mockup may supersede an older mockup; the superseded mockup becomes deprecated and records a back-reference to its successor.
- Status transitions are blocked for a mockup that has been superseded.
- Mockups are not versioned by Papyrus; version control is provided by git.

## Traceability
- `docs/specifications/ui/mockups/README.md` — mockup registry; status values pending/approved
- `docs/specifications/ui/browser-requirements-ui.md` — MOCK-0001 package
- README.md, `papyrus(1)` → COMMANDS → ui-spec and Documents lifecycle (lines 538–583, 750–757) — pattern reference

## Open Questions
- Supersede mechanism: command, field, or both? (product decision pending)
- Full status-transition table beyond the supersede constraint (product decision pending)