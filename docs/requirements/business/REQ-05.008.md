# REQ-05.008 Manage UI Mockups

## User Story
As a project contributor, I want to maintain UI mockups associated with UI specifications and tickets so that UI design artifacts can be found, reviewed, approved, and updated.

## Group
REQ-05 Project Documentation Management

## Criticality
Must have

## Description
The user can create a UI mockup as a document type with prefix MOCK stored under `docs/specifications/ui/mockups/`. A mockup realizes a UI specification: it carries a title, an associated task, references to other documents, and an implementation reference to one or more UI specifications by spec id. Mockups are editable, are not versioned by Papyrus (version control is provided by git), and can be associated to tickets (the ticket is recorded in the mockup and the mockup is referenced by the ticket). Mockups can be listed, viewed, and edited; they can be deprecated and restored with their ID preserved, with deprecation visibility matching UI specifications (hidden from lists, search, and the web UI). A mockup's status is one of: pending (waiting user approval), approved, or deprecated. A mockup may supersede an older mockup; supersession is performed by command only, `papyrus mockup supersede <old> <new>`; the superseded mockup becomes deprecated and records a back-reference to its successor. Status transitions are blocked while a mockup is superseded. Restore copies a deprecated mockup back to its original folder and updates its status field in the HTML. Status transitions follow the linear table pending -> approved -> deprecated: an approved mockup may be deprecated via `deprecate` or `supersede`, and a deprecated mockup is restorable.

## Acceptance Criteria
- A mockup can be created with the required title, associated task, and reference(s) to UI specification(s) by spec id.
- Mockups are stored under `docs/specifications/ui/mockups/` with IDs using the MOCK prefix.
- A mockup records its associated ticket and references to other documents within its HTML content.
- Mockups can be listed, viewed, and edited.
- A mockup's status is one of pending (waiting user approval), approved, or deprecated.
- Deprecation hides a mockup from lists, search, and the web UI; restoration returns it to its original folder with its ID preserved and updates its status field in the HTML.
- A mockup may supersede an older mockup; supersession is performed only via the command `papyrus mockup supersede <old> <new>`, the superseded mockup becomes deprecated and records a back-reference to its successor.
- Status transitions are blocked for a mockup that has been superseded.
- Status transitions follow the linear table pending -> approved -> deprecated; an approved mockup may be deprecated via `deprecate` or `supersede`, and a deprecated mockup is restorable.
- Mockups are not versioned by Papyrus; version control is provided by git.
- A mockup's metadata is carried in its HTML content and never in YAML front matter; mockup HTML files are excluded from the front-matter requirement (REQ-05.010).

## Traceability
- `docs/specifications/ui/mockups/README.md` — mockup registry; status values pending/approved
- `docs/specifications/ui/browser-requirements-ui.md` — MOCK-0001 package
- User decision 2026-11-08: UI mockups are excluded from YAML front matter; the HTML-embedded metadata remains their single source.
- User decision 2026-11-08: supersession is performed by command only (`papyrus mockup supersede <old> <new>`); status transitions follow the linear table pending -> approved -> deprecated.
- REQ-05.010 — Record Metadata in YAML Front Matter (front-matter scope excludes mockups)
- README.md, `papyrus(1)` → COMMANDS → ui-spec and Documents lifecycle (lines 538–583, 750–757) — pattern reference

