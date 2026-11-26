---
id: TECH-095
title: "Browser Navigation, Lists, and Record Views"
created_date: '2026-11-25T20:42:11Z'
updated_date: '2026-11-25T20:42:11Z'
labels: []
references:
  - REQ-005.010
  - REQ-005.017
  - REQ-006.004
  - REQ-006.010
  - REQ-006.014
  - REQ-006.016
  - NFR-007.002
status: draft
---
# TECH-095 Browser Navigation, Lists, and Record Views

## Description
Covers how the browser exposes every configured document type: the shared
document surface and its routes, the shared list layout and row content, the
`GroupedMarkdown` list variant, front-matter metadata presentation,
single-record field sections, and record reference links. It does not cover the
edit form, the Backlog board, console verbosity, or math and diagram rendering.

## One Surface for Every Configured Type

- Each document type declared in the configuration is served by one document
  surface covering its list, single-record view, edit form, submission,
  conflict, and not-found pages (REQ-006.014).
- The surface varies only with the type's document model and descriptor: there
  is no per-type handler, no per-type page, and no per-type string
  (REQ-006.014).
- A type added by configuration becomes available without a code change
  (REQ-006.014).
- Backlog and decision records keep their own surfaces (REQ-006.014).
- Every presentation fact comes from the type's configuration entry: label,
  plural, order, summary, type alias, API aliases, slots, reference visibility,
  and each field's label, help, sequence position, required flag, kind, and
  verbs (REQ-006.014).
- A display heading is always the field label; a field's storage heading never
  appears as one (REQ-006.014).
- The document surface is built from one validated configuration snapshot and
  requires no network access (REQ-006.014).

## List Status Filter

- The document list offers a status filter whose options are `active` plus the
  type's declared custom workflow statuses in configured order
  (REQ-006.014).
- `archived` is never offered, because archived records are not listed
  (REQ-006.014).
- A filter value outside the offered set is rejected rather than applied
  (REQ-006.014).
- A type that declares no custom status shows only `active` (REQ-006.014).

## Shared List Layout

- Every browser list view — tasks, milestones, releases, decisions, and every
  document type — uses one shared list layout (REQ-006.016).
- The layout fixes the title cell, the row and column structure, and the
  placement of actions; family-specific content fills declared slots instead of
  replacing the layout, and no family uses a bespoke list template
  (REQ-006.016).
- A document list uses the shared layout, the type's declared slots, and the row
  content below (REQ-006.014, REQ-006.016).
- Archived records appear in no list (REQ-006.016, REQ-005.017).

## Row Content

- A list shows one row per record (REQ-005.017).
- The title cell carries the record title and its labels (REQ-005.017,
  REQ-006.016).
- The row carries the values of the family's declared slots; a document type's
  slots come from its configuration entry (REQ-006.016, REQ-005.017).
- The row carries the record's reference list only when the type's reference
  visibility is `visible`; with `hidden` the row omits that list while the
  labels still render (REQ-005.017).
- Slot values and the visibility value come from the type's configuration entry
  (REQ-005.017, REQ-006.016).
- The glossary landing page is the one deliberate reduced row of the shared
  layout (REQ-006.016).

## GroupedMarkdown List Variant

- A list of a `GroupedMarkdown` type renders group rows and read-only member
  rows through the hierarchical row variant (REQ-006.016).
- Group rows and member rows are merged in group-then-member order: group order
  first, then member order within each group (REQ-005.017, REQ-006.016).
- Member rows are read-only in the list (REQ-005.017).

## Metadata Presentation

- The browser list and the single-record view show the same metadata values that
  the record stores; neither derives a value from the record body when the
  metadata defines it, and neither renames a stored value for display
  (REQ-006.004).
- Both views show at least the identifier, the title, the created and updated
  dates, the labels, the references, and every declared field of the record's
  type and role (REQ-006.004).
- The record's `status` is shown as stored, including a custom status declared
  for its type (REQ-006.004).
- `created_date` and `updated_date` are stored as quoted ISO-8601 UTC strings of
  the form `'YYYY-MM-DDTHH:MM:SSZ'` and are displayed without renaming
  (REQ-005.010, REQ-006.004).
- The `id` value and every reference value use the record identifier, with no
  second identifier space (REQ-005.010).
- A field declared with both a heading and a front-matter key displays its body
  section; body and front-matter values are not merged, and a divergence already
  on disk is left as stored (REQ-005.010).
- An undeclared metadata key already on disk is shown read-only after the field
  sections (REQ-006.014).

## Single-Record Field Sections

- A Markdown record view renders one section per declared field in
  field-sequence order, headed by the field label (REQ-005.017,
  REQ-006.014).
- A field without a value produces no section; the exception is a required
  field with no stored value, which appears as an empty section (REQ-005.017).
- The References disclosure is present on the single-record view of every model,
  whatever the reference visibility of its type (REQ-005.017).
- A `GroupedMarkdown` group record shows its members as a read-only list in
  identifier order, and a member record shows a read-only link to its group;
  membership is never a form control (REQ-006.014).
- A `SingleHtmlMockup` record shows its fixed metadata block instead of field
  sections (REQ-006.014).

## Reference Links

- Every reference the browser renders to another Papyrus record is a hyperlink,
  whatever its role (REQ-006.010).
- The rule covers a task's dependencies and milestone, a release's milestones, a
  milestone's release and tasks, a record's `references`, a member record's
  owning group, and a mockup's task and specifications (REQ-006.010).
- The rule also covers the items of any declared list-valued field whose values
  are record identifiers (REQ-006.010).
- The rule follows the reference itself, not the field that carries it, so a
  newly declared field of record identifiers is linked without a browser change
  (REQ-006.010).
- Activating a link opens the referenced record's view; the browser back button
  returns to the page the link was followed from (REQ-006.010).
- A reference that does not resolve to an existing record is rendered as plain
  text and raises no error (REQ-006.010).
- Links are generated locally and require no network access (REQ-006.010).
- The rule is browser-only; command-line output prints reference values as text
  (REQ-006.010).

## Unviewable Records

- A record that is unknown, archived, or of a type that is not configured is not
  viewable; the browser returns one shared not-found page for all three cases
  (REQ-005.017, REQ-006.014).

## View Latency at 1,000 Records

- A document view page is served with a p95 latency below 100 ms when 1,000
  active records exist (NFR-007.002).
- The measurement uses the corpus, hardware, warm-up, measured-request count,
  reporting, and exclusions of the inventory-latency run, with 100 measured
  requests per document model to the view route (NFR-007.002).
- The measured set includes one record with an unresolved reference and one
  member of the 200-member group (NFR-007.002).
- The run and the measured figures are recorded with the release
  (NFR-007.002).
