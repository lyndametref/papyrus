---
id: TECH-088
title: Seed
created_date: '2026-11-25T20:42:10Z'
updated_date: '2026-11-25T20:42:10Z'
labels: []
references:
  - REQ-005.016
status: draft
---
# TECH-088 Seed

## Description
Defines the document-type Seed: the built-in document types that `init` offers
for selection, their models, prefixes, field sequences, reference visibility,
and workflow statuses, and the init-time lifecycle. The decision-type Seed is a
separate family and is not specified here.

## Definition

- The Seed is the built-in list of default record types.
- It has two families: the document-type Seed, defined here, and the
  decision-type Seed.
- This specification is the only definition of the built-in document types; no
  other requirement, table, or module defines a built-in type, its fields, or
  its defaults.

## Init Lifecycle

- `init` is the only command that reads the Seed.
- The walkthrough presents the Seed for selection during `init`.
- A selected entry is written into `papyrus.config.yml` fully expanded.
- An unselected entry is not written at all.
- A run without an interactive terminal writes every entry.
- After `init` the Seed is irrelevant: no command, view, search, validation, or
  default resolution reads it.
- The project configuration is then the only source of document types, and a
  document type exists if and only if the configuration declares it.
- There is no `enabled` key: presence in the configuration is the only switch,
  and removing an entry removes the type.

## Full Expansion

A written entry carries: document model, field sequence, identifier prefix,
directory, custom workflow status set, reference visibility, list slots, and
field defaults.

The criticality of a built-in type is a requirement attribute, not a
configuration key.

## Entries

The document-type Seed contains exactly these eleven entries.

| Slug | Model | Criticality | Prefix | Field sequence | References | Statuses |
|---|---|---|---|---|---|---|
| `req` | `GroupedMarkdown` | Must have | `REQ` | group and member: `story`, `description` | hidden | none |
| `nfr` | `GroupedMarkdown` | Must have | `NFR` | group and member: `story`, `description` | hidden | none |
| `glossary` | `SingleMarkdown` | Must have | `GLO` | `definition` (required) | visible | none |
| `rule` | `SingleMarkdown` | Must have | `BR` | `description` (required, create and edit), `rationale` (required, create only), `applicability` (required, create only), `requirements` (list, option `requirement`) | visible | none |
| `ui-spec` | `SingleMarkdown` | Must have | `UI` | `view`, `description` | visible | none |
| `tech-spec` | `SingleMarkdown` | Must have | `TECH` | `description` | visible | none |
| `audit` | `SingleMarkdown` | Must have | `AUD` | `audit_type` (front-matter key `type`, option `audit-type`, default `security`), `content` (required) | visible | none |
| `use-case` | `SingleMarkdown` | Should have | `UC` | `description`, `applicability` | visible | none |
| `test-case` | `SingleMarkdown` | Should have | `TC` | `description`, `applicability` | visible | none |
| `test-report` | `SingleMarkdown` | Should have | `TR` | `description`, `applicability` | visible | none |
| `ui-mockup` | `SingleHtmlMockup` | Must have | `MOCK` | none (`fields: []`) | visible | fixed `pending`, `approved`, `archived` |

## Field Rules

- Every field is a body field under its own heading, except where the table
  names a front-matter key. `audit_type` is stored at the front-matter key
  `type`.
- A field is optional unless the table marks it required.
- `rule.description` is required on create and on edit.
- `rule.rationale` and `rule.applicability` are required on create only.
- `rule.requirements` is a list whose option is `requirement`.
- `audit.audit_type` has the option `audit-type` and the default `security`.
- `ui-mockup` declares no fields.

## Statuses and Identifiers

- No entry declares `active` or `archived` in a custom status list: `active`
  describes any non-archived state and `archived` is the storage state.
- The `ui-mockup` states are fixed `pending`, `approved`, and `archived` and
  are not configurable.
- Every other entry declares no custom workflow status.
- The prefix of each entry is the identifier prefix of its records.
