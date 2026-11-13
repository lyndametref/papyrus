# REQ-05.008 Manage UI Mockups

## User Story
As a project contributor, I want to maintain UI mockups associated with UI specifications and tickets so that UI design artifacts can be found, reviewed, approved, updated, and retired.

## Group
REQ-05 Project Documentation Management

## Criticality
Must have

## Description
The user can create a UI mockup as a document type declared by the default `document_types` entry `ui-mockup`, with prefix MOCK, stored under `docs/specifications/ui/mockups/`. A mockup realizes a UI specification: it carries a title, an associated task, references to other documents, and an implementation reference to one or more UI specifications by spec id. Mockups are editable, are not versioned by Papyrus (version control is provided by git), and can be associated to tickets (the ticket is recorded in the mockup and the mockup is referenced by the ticket). Mockups can be listed, viewed, and edited, and can be archived or deleted.

A mockup's status is one of: pending (waiting user approval), approved, or archived. Status transitions follow the linear table pending -> approved -> archived: an approved mockup is archived by `papyrus ui-mockup archive <id>` or by supersession, and an archived mockup has no restore command (REQ-05.007). A mockup may supersede an older mockup; supersession is performed by command only, `papyrus ui-mockup supersede <old> <new>`; the superseded mockup becomes archived and records a back-reference to its successor. Status transitions are blocked while a mockup is superseded.

Mockups are the document type that keeps its metadata in HTML rather than YAML front matter. Their status, associated task, and references remain stored in the mockup's HTML content, and mockup HTML files carry no front matter (REQ-05.010). Archiving sets that HTML status to `archived` and moves the file into the `archived/` subfolder of the mockup directory, keeping the ID.

## Acceptance Criteria
- A mockup can be created with the required title, associated task, and reference(s) to UI specification(s) by spec id.
- Mockups are stored under `docs/specifications/ui/mockups/` with IDs using the MOCK prefix.
- A mockup records its associated ticket and references to other documents within its HTML content.
- Mockups can be listed, viewed, and edited.
- A mockup's status is one of pending (waiting user approval), approved, or archived.
- Archiving a mockup sets its HTML status to `archived`, moves it to the `archived/` subfolder, hides it from lists, search, and the web UI, and preserves its ID.
- No restore command exists for mockups (REQ-05.007).
- A mockup may supersede an older mockup; supersession is performed only via the command `papyrus ui-mockup supersede <old> <new>`, the superseded mockup becomes archived and records a back-reference to its successor.
- Status transitions are blocked for a mockup that has been superseded.
- Status transitions follow the linear table pending -> approved -> archived.
- `papyrus ui-mockup delete <id>` permanently removes a mockup, archiving an active mockup first, with confirmation and `--force` as for other document types (REQ-05.007).
- Mockups are not versioned by Papyrus; version control is provided by git.
- A mockup's metadata is carried in its HTML content and never in YAML front matter; mockup HTML files are excluded from the front-matter requirement (REQ-05.010).

## Traceability
- `docs/specifications/ui/mockups/README.md` — mockup registry; status values pending/approved
- `docs/specifications/ui/browser-requirements-ui.md` — MOCK-0001 package
- User decision 2026-11-08: UI mockups are excluded from YAML front matter; the HTML-embedded metadata remains their single source.
- User decision 2026-11-08: supersession is performed by command only (`papyrus mockup supersede <old> <new>`); status transitions follow the linear table pending -> approved -> deprecated (state name superseded by the 2026-11-13 decision below).
- Project-owner decision 2026-11-13: mockups keep HTML metadata but adopt the `archived` state in place of `deprecated`; archiving moves the file to `archived/`; no restore command exists.
- REQ-05.007 — Manage Document Archive State and Deletion
- REQ-05.010 — Record Metadata in YAML Front Matter (front-matter scope excludes mockups)
- REQ-07.002 — Configure Document Types and Slugs
- README.md, `papyrus(1)` → COMMANDS → ui-spec and Documents lifecycle (lines 538–583, 750–757) — pattern reference
