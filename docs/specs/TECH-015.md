---
id: TECH-015
title: "Browser Navigation and Presentation"
created_date: '2026-11-27T20:35:39Z'
updated_date: '2026-11-27T20:35:39Z'
labels: []
references:
  - REQ-006.010
  - REQ-006.011
  - REQ-006.014
  - REQ-006.016
status: verified
---
# TECH-015 Browser Navigation and Presentation

## Description
Covers the implementation of the shared browser surface: the descriptor-driven templates, shared list layout, column sorting, collapsible groups, status-filter validation, the reduced-row variant, and server-side reference links. Routes are specified by the routing specification; HTML rendering is specified by the rendering specification; command-line lists are separate.

## Descriptor-driven surface

- One template set serves every document type: list, single-record view, edit form, submission result, conflict page, and not-found page.
- The type's descriptor selects the model-specific view sections and field renderers; there is no per-type handler, page, or string.
- The descriptor is built from one validated configuration snapshot when the server starts or reloads; request handlers read the snapshot and never the configuration file.
- Each handler receives the descriptor for the requested type and uses its `record_name`, `plural`, configuration order, summary, slots, and field name, help, sequence position, required flag, kind, and verbs.
- Adding a configured type adds a descriptor entry and its routes; no template changes.
- Backlog and decision records keep their own handlers and templates and are not served by the document surface.

## Shared list layout

- One list template fixes the column order: ID, Title, Status, the type's declared slots in configured order, Actions.
- A slot is a property key resolved against each record at render time; slot keys are validated against the reserved and structural keys when the configuration loads.
- A family supplies cell values only; it never supplies a table, header row, or column order.
- Archived records are filtered out before the rows are built, so no archived file contributes a row.

## Column sorting

- Every data column header is a link carrying `sort=<key>` and an optional `dir=asc|desc`; the server sorts the rows before rendering, so sorting works without JavaScript.
- Sortable keys are `id`, `title`, `status`, and each declared slot key; the Actions column is not sortable.
- The first activation of a column sorts ascending; activating the same column again reverses the direction; activating another column starts at ascending.
- Sorting is stable, with identifier ascending as the tie-break.
- The active column carries `aria-sort` with the current direction on its header cell.
- An unknown sort key or direction is rejected with the shared 400 error page and renders no list.
- Sorting composes with the status filter and with collapsed groups.

## Collapsible groups

- In a `grouped-record` list, every group row carries a toggle. The list is fully collapsed unless a group is named in the request.
- Without JavaScript, the toggle is a link to the list URL with `expanded=<group-id>`; the parameter is repeatable, and the server renders member rows only for the named groups.
- With JavaScript, the toggle flips the `hidden` attribute on the group's member rows without a reload and updates `aria-expanded`.
- The toggle exposes `aria-expanded` and is operable from the keyboard.
- An `expanded` value that names no group in the current list is ignored.
- Toggling affects only that group's member rows; the group row and every other group are unchanged.
- In a grouped list, sorting orders member rows within each group; group order is never changed by sorting or collapsing.

## Status filter

- The list reads one `status` query parameter; its allowed set is `active` plus the type's declared custom statuses in configured order.
- `archived` is not in the allowed set; an unknown or out-of-set value is rejected with the shared 400 error page and no list is rendered.
- A type with no custom status accepts only `active`.
- The filter selects from the non-archived record set before rows are built; it never reads an archive folder.

## Reduced-row variant

- The glossary descriptor selects a reduced row variant; no other type selects it.
- The variant is a branch inside the shared row renderer, not a separate page or template: it renders labels as a tag list and adds no date or reference cell.
- The glossary single-record view and the rows of every other type keep the full variant.

## Reference links

- Links are resolved and rendered server-side. For a reference value, the renderer looks up the identifier in the enabled type registry and emits `/records/{id}` only when a record resolves; otherwise it emits the value as text.
- The lookup is the same identifier index used by route resolution, so a link and a directly requested URL resolve to the same record.
- The `href` is built from the resolved identifier and never from the stored value, so stored content cannot inject a scheme or path.
- One reference renderer serves every reference-bearing property: a task's dependencies and milestone, a release's milestones, a milestone's release and tasks, a record's `references`, a member's owning group, and a mockup's task and specifications.
- A list-valued field renders links when its field metadata restricts values to record identifiers; a newly declared such field needs no link code.
- An unresolved reference is rendered as plain text and produces no error.
- Links are local; no link points outside the application, and navigation uses native browser history rather than a client-side router.
- Command-line output prints reference values as text and is unaffected.

## No-JavaScript baseline

- The list, column sorting, group collapsing, the status filter, the reduced-row variant, and every reference link are server-rendered and work without JavaScript.
- No client-side router, data fetch, or script-generated markup is required for navigation.
