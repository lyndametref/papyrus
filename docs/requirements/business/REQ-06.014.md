---
id: REQ-06.014
title: "Display Config-Defined Document Types in the Browser"
created_date: '2026-11-13T20:57:05Z'
updated_date: '2026-11-20T20:59:20Z'
labels: []
references:
  - RE-010
  - ADR-036
  - ADR-037
  - ADR-038
  - ADR-039
---
# REQ-06.014 Display Config-Defined Document Types in the Browser

## User Story
As a project contributor, I want the browser to serve any document type declared in configuration so that a document type added for the CLI is also usable in the local web interface.

## Group
REQ-06 Search and Local Web Interface

## Criticality
Should have

## Description
The local web interface serves list, view, edit, conflict, and not-found pages for every enabled document type declared in `document_types`, not only for the default types shipped with Papyrus. A document type added by configuration (REQ-07.002) becomes available in the browser without a code change. Every document record type — a seed type or a user-defined type, under any document model — is served by one descriptor-driven document surface; backlog and decision records keep their bespoke surfaces (RE-010 D4).

The document surface has one list builder, one view builder, one edit builder, one form submission handler, one conflict builder, and one not-found builder. The browser document path contains no per-type dispatch, no slug matching, no per-type handler variant, and no document entry in a static presentation table. Its only variation comes from the type's descriptor and the field map built for the record's model and role. Navigation, list, view, edit, conflict, and not-found output for documents are produced by that one surface.

Views use the document type's configured folder and prefix, the record field map of REQ-06.004, and the single-record layout of REQ-06.012. The list uses the shared list layout of REQ-06.016 and its fixed columns. A document type's slots are its declared seed slots, and no document field becomes a list column.

**GroupedMarkdown.** A `GroupedMarkdown` record is rendered by its role. A group record renders the `group.fields` sections, then an engine-owned read-only member list (member ID, title, and link) in ID order. A member record renders the `member.fields` sections and a read-only link to its group. Membership is never a form control and never a field, and the group link cannot be changed through the browser. The list renders the group row and its member rows through the shared hierarchical row variant in merged group-then-member order.

**SingleHtmlMockup.** A `SingleHtmlMockup` record uses the same document head, chrome, and reference disclosure as every other document, with a fixed metadata block instead of field sections: `status` (`pending`, `approved`, or `archived`), `task`, `specs`, `references`, and `superseded_by`, all read from the HTML metadata comment. The edit form offers the title and the list-valued `specs` and `references` controls only; `status`, `task`, and `superseded_by` are read-only there, there is no field sequence, and there is no `status` control.

**Presentation facts.** Every presentation fact of a document type comes from the seed descriptor, and a user-defined type uses the documented default presentation values: the label, plural, order, summary, API aliases, type alias, slots, reference visibility, and each field's label, help, sequence position, required flag, kind, and verbs. The list heading, count noun, empty state, inventory note, page title, navigation label, sidebar entry, form label, form hint, view section heading, and required marker are derived from the descriptor. A field's storage `heading` (for example `## User Story`) is never rendered as a display heading; the visible section heading is the field `label`. The browser may format a value it is given, but it may not hold a per-type string or a per-type list.

A document record's content is rendered from its field map: one section per non-empty field in sequence order, headed by the field label, with a list rendered as a list. A required field that is empty renders as an empty section rather than being omitted. Undeclared front-matter keys already on disk follow the field sections as read-only metadata rows. There is no per-type summary card and no repetition of the document type.

The edit form is one builder. It renders the fixed title control, one control per declared field whose `editable` is true in sequence order, the general `labels` and `references` controls, read-only `status`, `created_date`, and `updated_date`, and the hidden version token. A field whose `verbs` exclude `edit` is rendered read-only and is never writable. A submission with a stale version token renders the same edit form with the submitted values and a conflict notice and returns `409` without writing. A document ID that does not resolve returns one not-found page with `404`, the same page for an unknown ID, an archived record, and a disabled type.

Every listed record shows its `status` value as stored, including a custom status (REQ-07.001). The list view offers a status filter whose options are exactly `active` plus the type's declared custom statuses, in configured order; selecting a value lists only the records carrying that value, and a filter value outside that set is rejected rather than applied. The `archived` value is not offered because archived records are not listed (REQ-05.007), so a type that declares no custom statuses offers only `active`.

A user-defined type defaults to `SingleMarkdown` (REQ-07.004); its edit view offers the fixed title control, its declared fields (by default one optional `description` field), `labels`, and `references`. `status` is not editable in the browser because it is lifecycle- and configuration-owned (REQ-05.011). Archived records are hidden and unreachable, and a disabled type has no views and its records are neither listed nor reachable.

The document surface is built from one validated configuration and registry snapshot and requires no network access (ADR-037, NFR-01.003). Transport, routes, security headers, cache classes, and the `no-store` policy for HTML and JSON are unchanged (ADR-038).

## Acceptance Criteria
- The browser serves a list view and a detail view for every enabled document type declared in `document_types`.
- A document type added by configuration gains its browser views without a code change.
- Every document type, seed or user-defined and under any document model, is served by one descriptor-driven document surface for list, view, edit, conflict, not-found, and form submission.
- The browser document path contains no per-type dispatch, no slug match, no per-type handler variant, and no document entry in a static presentation table.
- A config-defined type offers an edit view with the fixed title, its declared fields (by default `description`), `labels`, and `references`.
- Views use the type's configured folder and prefix.
- A view renders one section per non-empty field in descriptor sequence order, headed by the field label; a required empty field renders as an empty section.
- A field's storage `heading` is never rendered as a display heading; the visible section heading is the field label.
- Undeclared front-matter keys on disk are rendered as read-only metadata rows after the field sections.
- A `GroupedMarkdown` group record renders its group field sections and a read-only member list in ID order; a member record renders its member field sections and a read-only link to its group.
- Membership is never a form control or field, and the group link is not editable in the browser.
- A grouped list renders group rows and member rows through the shared hierarchical row variant in merged group-then-member order.
- A `SingleHtmlMockup` view uses the shared head, chrome, and reference disclosure with a fixed metadata block of `status`, `task`, `specs`, `references`, and `superseded_by` read from the HTML metadata comment, and no field sections.
- A `SingleHtmlMockup` edit form offers the title and the `specs` and `references` controls only; `status`, `task`, and `superseded_by` are read-only, and there is no field sequence and no `status` control.
- Every presentation fact (labels, headings, field order, help, slots, aliases, reference visibility, and required markers) comes from the seed descriptor or the documented default for a user-defined type; the browser holds no per-type string or list for a document.
- The edit form renders the fixed title control, editable declared fields in sequence order, `labels`, `references`, read-only `status`, `created_date`, and `updated_date`, and the hidden version token; a field whose verbs exclude `edit` is read-only.
- A stale version token renders the same edit form with the submitted values and a conflict notice and returns `409` without writing.
- An unresolved document ID returns one not-found page with `404`, the same page for an unknown ID, an archived record, and a disabled type.
- Each listed record shows its `status` value as stored, including a configured custom status.
- The list view offers a status filter whose options are exactly `active` plus the type's declared custom statuses in configured order; selecting a value lists only records carrying that value.
- A type that declares no custom statuses offers only `active` in the status filter, and `archived` is never an option because archived records are not listed.
- A status filter value outside the offered set is rejected rather than applied (NFR-02.002).
- `status` is not an editable field in the browser (REQ-05.011).
- Detail views use the field map and metadata projection of REQ-06.004 and the layout of REQ-06.012.
- Archived records are not listed, are not reachable, and return the same page as an unknown address.
- A disabled document type has no views, and its records are neither listed nor reachable.
- The views require no network access (NFR-01.003); transport, route matching, security headers, cache classes, and `no-store` on HTML and JSON are unchanged (ADR-037, ADR-038).

## Open Questions
- Whether a document type can declare an icon or display name; the seed presentation facts define none.

## Traceability
- Source: customer-feedback handoff — in-session user feedback, Item 10 (needs clarification), raw quote:
  - "any type of document can be configured in the config file. ... the document type artifacts can be created updated or deleted from the command line with papyrus <document-slug>."
- RE-010 — Descriptor-driven document engine refactor (accepted, not reopenable): D4 (backlog and decision surfaces stay bespoke), D5 (`DocumentModel` variants), D6 (grouped and HTML models are seed-only), D9 (rendered HTML is semantically equivalent; record, JSON, and status codes are strict), D10 (presentation facts live in the config seed), D13 (fixed title contract).
- ADR-039 — One descriptor-driven document view and edit surface (decisions 10-13 one document surface; 14-18 field map; 19-25 rendering rules; 26-28 one form submission; 29-31 presentation facts from the seed; 32-33 reference visibility; downstream constraints 1-9).
- ADR-036 — Config-declared document field schema and one descriptor model (decisions 4-10 one descriptor and three models; 11-27 field vocabulary and role sets; 28-31 seed table and seed-only models).
- ADR-037 — In-server configuration and registry snapshot (the document surface is built from one validated configuration and registry snapshot; no per-request configuration reload).
- ADR-038 — Cacheable immutable browser assets (transport, cache classes, and `no-store` on HTML and JSON are unchanged).
- ADR-0020 — Browser routing and the bespoke backlog and decision surfaces (as amended by ADR-039 decisions 2-6).
- ADR-0028 — Uniform browser list layout (as amended by ADR-039 decisions 5-6: document slots come from the seed).
- ADR-0033 — Uniform single-document browser view layout (as amended by ADR-039 decision 7).
- REQ-05.007 — Manage Document Archive State and Deletion
- REQ-05.011 — Maintain Record Metadata Through All Commands (`status` not editable)
- REQ-06.002 — Serve the Local Web Interface
- REQ-06.004 — Use Front-Matter Metadata in CLI and Browser Views
- REQ-06.012 — Define Single Record View Layout
- REQ-06.016 — Present a Uniform List View Across Artifact Families
- REQ-06.019 — Control Reference Visibility in Document-Type Browser List Views
- REQ-07.001 — Organize Artifacts into Three Categories (document status model)
- REQ-07.002 — Configure Document Types and Slugs
- REQ-07.004 — Provide One Schema-Driven Document Content Model with Three Document Models
- NFR-01.002 — Display, View, and Edit All Papyrus Content in a Local Browser
- NFR-01.003 — Use the Browser Command and Interface Without Internet
- NFR-02.002 — Sanitize Input (invalid filter values rejected)
