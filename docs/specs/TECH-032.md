---
id: TECH-032
title: "Record Views"
created_date: '2026-11-27T20:35:44Z'
updated_date: '2026-11-28T08:25:50Z'
labels: []
references:
  - REQ-002.002
  - REQ-004.003
  - REQ-005.008
  - REQ-005.017
status: verified
---
# TECH-032 Record Views

## Description
Covers the console single-record view for the three record categories: document types, the built-in backlog types, and decision types. Console list output is a separate specification; browser presentation is also separate.

## Shared rules

- A view renders one record. Plain text is the default output and `--json` carries the values without console formatting.
- `--no-color` disables the console styling; the content stays readable.
- References are present on the single-record view of every category.
- A record that is unknown or of a type that is not configured is not viewable, and no command returns it. An archived record is viewable.
- An archived record is not editable: an edit is rejected with a validation outcome and writes nothing.

## Document views

- A view renders one section per declared field, in field-sequence order.
- The section heading is the field's name.
- A field with no value contributes no section. A required field with no stored value is rendered as an empty section.
- Field content is Markdown rendered with the console's advanced formatting, so headings, emphasis, lists, tables, code, and quotes are formatted for the terminal rather than printed as raw source.
- A `formatted-record` view renders the fixed metadata block instead of field sections (REQ-005.008).
- A `grouped-record` group record shows its members as a read-only list in identifier order, and a member record shows a read-only link to its group; membership is never a form control.

## Task view

- A task view shows the identifier, title, status, description, acceptance criteria, milestone, assignees, labels, references, dependencies, ordinal, and the created and updated dates.
- `--ac` prints only the acceptance criteria: one criterion per line in plain text and an array under `--json`.

## Milestone view

- A milestone view shows the identifier, title, status, description, assigned release, acceptance criteria, labels, and references.
- `--ac` prints only the acceptance criteria: one criterion per line in plain text and an array under `--json`.

## Release view

- A release view shows the identifier, title, status, description, labels, and references.

## Decision view

- A decision view shows the identifier, title, type, status, decision text, context, consequences, creation date, update date, labels, and references.
- A superseded decision's view names its successor.
- An archived decision is viewable.
