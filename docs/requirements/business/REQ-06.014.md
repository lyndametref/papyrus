# REQ-06.014 Display Config-Defined Document Types in the Browser

## User Story
As a project contributor, I want the browser to serve any document type declared in configuration so that a document type added for the CLI is also usable in the local web interface.

## Group
REQ-06 Search and Local Web Interface

## Criticality
Should have

## Description
The local web interface serves list, detail, and edit views for every enabled document type declared in `document_types`, not only for the default types shipped with Papyrus. A document type added by configuration (REQ-07.002) becomes available in the browser without a code change, using the same navigation, list, detail, and edit behavior as the default document types.

Views use the document type's configured folder and prefix, the record metadata projection of REQ-06.004, and the single-record layout of REQ-06.012. Every listed record shows its `status` value as stored, including a custom status (REQ-07.001). The list view offers a status filter whose options are exactly `active` plus the type's declared custom statuses, in configured order; selecting a value lists only the records carrying that value, and a filter value outside that set is rejected rather than applied as an unfiltered or guessed match. The `archived` value is not offered because archived records are not listed (REQ-05.007), so a type that declares no custom statuses offers only `active`.

A config-defined type uses the generic content model (REQ-07.004), so its edit view offers title, description, labels, and references; `status` is not editable there because it is lifecycle- and configuration-owned (REQ-05.011). Archived records are hidden and unreachable, and the same page is returned for an archived record and an unknown address (REQ-05.007). A disabled type has no views, and its records are not listed or reachable. Where a document type has a field set beyond the generic model, that field set continues to apply to the default type.

## Acceptance Criteria
- The browser serves a list view and a detail view for every enabled document type declared in `document_types`.
- A document type added by configuration gains its browser views without a code change.
- A config-defined type offers an edit view with title, description, labels, and references.
- Views use the type's configured folder and prefix.
- Each listed record shows its `status` value as stored, including a configured custom status.
- The list view offers a status filter whose options are exactly `active` plus the type's declared custom statuses in configured order; selecting a value lists only records carrying that value.
- A type that declares no custom statuses offers only `active` in the status filter, and `archived` is never an option because archived records are not listed.
- A status filter value outside the offered set is rejected rather than applied (NFR-02.002).
- `status` is not an editable field in the browser (REQ-05.011).
- Detail views use the metadata projection of REQ-06.004 and the layout of REQ-06.012.
- Archived records are not listed, are not reachable, and return the same page as an unknown address.
- A disabled document type has no views, and its records are neither listed nor reachable.
- The views require no network access (NFR-01.003).

## Open Questions
- Whether the browser navigation is grouped by the three artifact categories or lists document types flat; the page route and navigation entry remain design decisions, as in REQ-06.003.
- Whether a config-defined type can declare an icon or display name; current requirement defines none.

## Traceability
- Source: customer-feedback handoff — in-session user feedback, Item 10 (needs clarification), raw quote:
  - "any type of document can be configured in the config file. ... the document type artifacts can be created updated or deleted from the command line with papyrus <document-slug>."
- Project-owner decision 2026-11-13 (refinement): the browser must serve full list/detail/edit views for config-defined document types, not only the CLI.
- Project-owner decision 2026-11-13 (D5): the browser shows document status values, including custom statuses, and its status filter/labels follow the statuses declared for the type; `status` is not editable in the browser.
- REQ-05.007 — Manage Document Archive State and Deletion
- REQ-05.011 — Maintain Record Metadata Through All Commands (`status` not editable)
- REQ-06.002 — Serve the Local Web Interface
- REQ-06.003 — Describe Managed Artifacts and Their Lifecycles in the Browser
- REQ-06.004 — Use Front-Matter Metadata in CLI and Browser Views
- REQ-06.012 — Define Single Record View Layout
- REQ-07.001 — Organize Artifacts into Three Categories (document status model)
- REQ-07.002 — Configure Document Types and Slugs (declared statuses)
- REQ-07.004 — Provide a Generic Content Model for Config-Defined Document Types
- NFR-01.002 — Display, View, and Edit All Papyrus Content in a Local Browser
- NFR-01.003 — Use the Browser Command and Interface Without Internet
- NFR-02.002 — Sanitize Input (invalid filter values rejected)
