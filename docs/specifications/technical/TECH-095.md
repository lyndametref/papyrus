---
id: TECH-095
title: "Browser Navigation and Presentation"
created_date: '2026-11-25T20:42:11Z'
updated_date: '2026-11-27T16:35:38Z'
labels: []
references:
  - REQ-006.010
  - REQ-006.014
  - REQ-006.016
  - NFR-007.002
status: verified
---
# TECH-095 Browser Navigation and Presentation

## Description
Covers how the browser exposes configured document types: the shared document surface and its routes, the shared list layout, the list status filter, reference links, and view latency. List row content, single-record view content, metadata projection, and the edit form are separate specifications.

## One surface for every configured type

- Each document type declared in the configuration is served by one document surface covering its list, single-record view, edit form, submission, conflict, and not-found pages (REQ-006.014).
- The surface varies only with the type's record model and descriptor: there is no per-type handler, no per-type page, and no per-type string (REQ-006.014).
- A type added by configuration becomes available without a code change (REQ-006.014).
- Backlog and decision records keep their own surfaces (REQ-006.014).
- Every presentation fact comes from the type's configuration entry: `record_name`, `plural`, configuration order, summary, slots, and each field's name, help, sequence position, required flag, kind, and verbs (REQ-006.014).
- The document surface is built from one validated configuration snapshot and requires no network access (REQ-006.014).

## List status filter

- The document list offers a status filter whose options are `active` plus the type's declared custom workflow statuses in configured order (REQ-006.014).
- `archived` is never offered, because archived records are not listed (REQ-006.014).
- A filter value outside the offered set is rejected rather than applied (REQ-006.014).
- A type that declares no custom status shows only `active` (REQ-006.014).

## Shared list layout

- Every browser list view — tasks, milestones, releases, decisions, and every document type — uses one shared list layout (REQ-006.016).
- The layout fixes the title cell, the row and column structure, and the placement of actions; family-specific content fills declared slots instead of replacing the layout, and no family uses a bespoke list template (REQ-006.016).
- A document list uses the shared layout and the type's declared slots (REQ-006.014, REQ-006.016).
- The glossary landing page is the one deliberate reduced row of the shared layout (REQ-006.016).

## Reference links

- Every reference the browser renders to another Papyrus record is a hyperlink, whatever its role (REQ-006.010).
- The rule covers a task's dependencies and milestone, a release's milestones, a milestone's release and tasks, a record's `references`, a member record's owning group, and a mockup's task and specifications (REQ-006.010).
- The rule also covers the items of any declared list-valued field whose values are record identifiers (REQ-006.010).
- The rule follows the reference itself, not the field that carries it, so a newly declared field of record identifiers is linked without a browser change (REQ-006.010).
- Activating a link opens the referenced record's view; the browser back button returns to the page the link was followed from (REQ-006.010).
- A reference that does not resolve to an existing record is rendered as plain text and raises no error (REQ-006.010).
- Links are generated locally and require no network access (REQ-006.010).
- The rule is browser-only; command-line output prints reference values as text (REQ-006.010).

## View latency at 1,000 records

- A document view page is served with a p95 latency below 100 ms when 1,000 active records exist (NFR-007.002).
- The measurement uses the corpus, hardware, warm-up, measured-request count, reporting, and exclusions of the inventory-latency run, with 100 measured requests per document model to the view route (NFR-007.002).
- The measured set includes one record with an unresolved reference and one member of the 200-member group (NFR-007.002).
- The run and the measured figures are recorded with the release (NFR-007.002).
