---
id: TECH-011
title: "Record Create and Edit"
created_date: '2026-11-27T20:35:38Z'
updated_date: '2026-11-28T08:25:50Z'
labels: []
references:
  - REQ-002.001
  - REQ-002.005
  - REQ-002.006
  - REQ-004.001
  - REQ-005.015
  - REQ-007.003
status: verified
---
# TECH-011 Record Create and Edit

## Description
Covers creating and editing a record across the three record categories: the title contract, attribute and field options, repeatable values, multiline input, the editor, clearing, appending, and content immutability. Identifiers, statuses, dates, command families, and views are separate specifications.

## Create

- Every create takes the title as the first positional `TITLE`; the title is required, single-line, and non-empty, and `create` reports the assigned identifier.
- A backlog or decision record accepts its attributes as options, taken from the attributes of its built-in type.
- A document record accepts the fixed title plus one option per declared field whose verbs include `create`, taken from the type's field sequence for the record's role.
- A field default declared in configuration is written on create when the field is absent.
- Repeatable options accept multiple values and may be repeated.
- A multiline attribute or field value accepts `-` to read the value from standard input on create and edit, including the decision text, the context, and the consequences.
- `--label` and `--reference` are the general repeatable metadata options.
- No create option sets the status; a new record starts at the status its category defines.
- `create --help` lists the options for the type.

## Edit

- `edit` changes the title through `--title`, every attribute or field whose verbs include `edit`, the labels, and the references.
- A field or attribute whose verbs exclude `edit` is never writable after creation.
- An optional attribute can be cleared; a required attribute cannot.
- Repeatable add-options append to the existing set instead of replacing it.
- `--editor` opens the record body in the configured editor; an option-only edit updates the metadata without opening the editor.
- An edit can never clear the title.
- No edit option sets the status; the status verbs of the category perform status changes.
- An archived record is not editable: an edit is rejected with a validation outcome and writes nothing.
- A browser save applies the same create and edit rules.

## Document fields

- Document edit options come from the type's field sequence for the record's role.
- A field with no edit option is never writable after creation.
- Required and clearing rules, `list` and `multiline` handling, and the `--editor` contract follow the field schema.
- A field outside the editable verb set is rejected when supplied.

## Decision immutability

- A decision can be created and edited only while its status is `proposed`.
- An edit of a decision changes the title, the decision text, the context, the consequences, the labels, or the references.
- Once the decision has left `proposed`, its content, title, labels, and references are frozen; an edit is rejected with a validation outcome and writes nothing.

## Validation

- A create missing a required title, attribute, or field writes no file.
- An invalid value, an unknown option, or a rejected edit writes nothing.
- A patch that changes nothing performs no write.
