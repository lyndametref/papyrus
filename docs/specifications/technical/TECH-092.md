---
id: TECH-092
title: Decisions
created_date: '2026-11-25T20:42:11Z'
updated_date: '2026-11-25T20:53:58Z'
labels: []
references:
  - REQ-004
  - REQ-004.001
  - REQ-004.002
  - REQ-004.003
status: draft
---
# TECH-092 Decisions

## Description
Covers decision records: their configured types, the type-specific command
family, the Nygard content template, the fixed status vocabulary and transitions,
supersession, content immutability, archive behavior, and the list and view
surface of a decision type. Document types and unified search are outside scope.

## Decision-Type Configuration

- Decision types are configured in `decision_types`, a map keyed by the type
  slug; presence in the configuration is what makes a type available, there is
  no `enabled` key, and each entry declares a prefix and a directory.
- The decision-type seed contains exactly two entries; criticality is an
  attribute of the built-in type, not a configuration key:

| Slug | Criticality | Prefix | Directory | Statuses |
|---|---|---|---|---|
| `adr` | Must have | `ADR` | `adr` | the five shared statuses |
| `bdr` | Must have | `BDR` | `bdr` | the five shared statuses |

- `init` offers both seed types and writes the selected one or both.
- The directory is the subfolder used when both types are configured; with
  exactly one configured type it is not applied.
- The directory may be changed by editing the configuration before decisions of
  that type exist.

## Command family

- The slug is the command name: the `adr` type is managed as `papyrus adr ...`
  and the `bdr` type as `papyrus bdr ...`.
- Each type exposes `create`, `list`, `view`, `edit`, `accept`, `reject`,
  `deprecate`, `supersede`, `archive`, and `delete`.
- The command selects the type. No `--decision-type` option exists and there is
  no shared `decision` command family.

## Identifiers and folder layout

- A decision identifier uses its type's configured prefix and the shared
  `id_padding`; numbering runs per type, and the type of an existing decision is
  derived from its identifier prefix, not from the slug.
- There is one main decision folder. With exactly one configured type, its
  records are stored directly in the main folder and archived in its `archive/`
  subfolder; the type's directory is not applied.
- With two or more configured types, each type's records are stored in the
  subfolder named by its directory, and archived in that subfolder's `archive/`.

## Changing the type set

| Change | Outcome |
|---|---|
| Add a type while at least two types are configured | Accepted; the new type receives its own subfolder; no existing record moves |
| Add the second type to a project with exactly one type | Rejected with a validation outcome; no file moves |
| Remove a configured type | Rejected with a validation outcome; no file moves |
| Change the prefix of a type that still has decisions | Rejected with a validation outcome; no file moves |

## Content

- `papyrus <slug> create <TITLE>` creates a decision of the configured type from
  the Nygard template: a title, the decision text, the context, and the
  consequences.
- The decision text, the context, and the consequences are multiline and accept
  `-` to read the value from standard input.
- Labels and references are optional general metadata; a reference may name any
  managed record, including business and non-functional requirements.
- A new decision starts `proposed`, and its status is stored in the `## Status`
  body section.

## Status vocabulary and transitions

The vocabulary is fixed, shared by every decision type, and not configurable per
type: `proposed`, `accepted`, `rejected`, `deprecated`, `superseded`. A status
moves only through the status verbs of the decision's own command family, and no
edit option sets it. Each verb performs exactly one transition.

| Command | Transition |
|---|---|
| `papyrus <slug> accept <ID>` | `proposed` to `accepted` |
| `papyrus <slug> reject <ID>` | `proposed` to `rejected` |
| `papyrus <slug> deprecate <ID>` | `accepted` to `deprecated` |
| `papyrus <slug> supersede <ID> --by <SUCCESSOR-ID>` | `accepted` to `superseded` |

- A verb applied from a status other than its required starting status is
  rejected with a validation outcome and writes nothing.
- A decision cannot be accepted or rejected twice, and a rejected decision cannot
  be accepted or deprecated.

## Supersession

- A decision is superseded by exactly one successor, of the same type.
- `supersede` records the successor identifier in the front-matter
  `superseded_by` key and sets the status to `superseded`.
- A successor of a different type, an unknown successor, or a second
  supersession is rejected and writes nothing.

## Content immutability

- `papyrus <slug> edit <ID>` changes the title, decision text, context,
  consequences, labels, or references while the status is `proposed`, and never
  changes the status.
- Once the decision has left `proposed`, its content, title, labels, and
  references are frozen; an edit is rejected with a validation outcome and
  writes nothing.
- `papyrus <slug> delete <ID>` permanently removes a decision while the status is
  `proposed`; confirmation is required unless `--force` is given. A decision
  that has left `proposed` cannot be deleted and is archived instead.

## Archive

- `papyrus <slug> archive <ID>` moves the record's file into the `archive/`
  subfolder of the folder that holds it, keeps its identifier, and changes no
  status.
- Archiving is allowed only when the status is `rejected`, `superseded`, or
  `deprecated`; it is rejected with a validation outcome and writes nothing for
  `proposed` and `accepted`.
- Archiving changes no status and keeps the identifier. An archived decision is
  excluded from the list, from search, and from the browser, and no command
  returns it to the decision folder; there is no restore command.

## Dates

- The creation date is written at creation, never changes afterwards, and is
  shown as the decision's date.
- The update date is when the status was last changed: a status verb refreshes
  it, while a content edit of a `proposed` decision does not.

## View and list

- `papyrus <slug> view <ID>` shows the identifier, title, type, status, decision
  text, context, consequences, creation date, update date, labels, and
  references; a superseded decision's view names its successor.
- A decision moved to the archive folder is not viewable: the command and the
  browser treat it like an unknown identifier.
- `papyrus <slug> list` lists the decisions of that type, each row showing the
  identifier, title, type, status, creation date, and update date.
- Archived decisions are excluded, and a cross-type listing is provided by
  search, not by a shared command.
- The list prints plain text by default and honours the global `--json` option.
