---
id: REQ-05.002
title: "Manage Non-Functional Requirements"
created_date: '2026-11-13T20:16:54Z'
updated_date: '2026-11-20T20:47:08Z'
labels: []
references:
  - RE-010
  - ADR-036
  - ADR-039
  - ADR-035
  - REQ-07.001
  - REQ-07.002
  - REQ-07.003
  - REQ-07.004
  - REQ-05.007
  - REQ-05.010
  - REQ-05.011
---
# REQ-05.002 Manage Non-Functional Requirements

## User Story
As a project contributor, I want to manage non-functional requirements in their own groups so that non-functional needs can be documented separately from business requirements.

## Group
REQ-05 Project Documentation Management

## Criticality
Must have

## Description
`nfr` is a `GroupedMarkdown` document type (REQ-07.004). A **group record** carries a plain ID `NFR-<n>`; a **member record** carries the dotted ID `NFR-<n>.<m>`, where `<n>` is the sequence of its owning group and `<m>` its position in that group (REQ-01.005). Groups contain non-functional requirements only, and each requirement belongs to exactly one group. The family is addressed by the slug `nfr` (`papyrus nfr`); the former `nfreq` command name is not retained (REQ-07.002).

**Declared fields.** The type's field sets are seed data and are not declared in project configuration (ADR-036 decisions 26, 27, and 29):

- `group.fields` — `story` stored under the heading `## User Story`, and `description` stored under the heading `## Description`.
- `member.fields` — `story` (`## User Story`) and `description` (`## Description`).

Both fields are optional and body-only: either may be omitted or cleared, and each value is stored in the record body under its declared heading. The field sequence is the order of the fields everywhere — body sections, CLI options and help, and the browser form (ADR-036 decision 20). The record title is not a field (REQ-05.010). No other field belongs to this type; adding or changing a field is a configuration file edit (REQ-07.002).

**Grouped create.** `papyrus nfr create-group <TITLE>` creates a group record. `papyrus nfr create <TITLE> --group <GROUP-ID>` creates a member and requires an existing group of the same type, supplied through the group slot or `--group`. An existing active group of the same type is reused: a member create never creates a group and no group is created implicitly. An unknown group, an archived group, or a group of another document type is rejected with one validation outcome (REQ-01.004), and nothing is written (ADR-036 decision 9).

**Title.** The title contract is uniform for every document type (RE-010 D13, REQ-07.003): the title is the first positional `TITLE` on `create` and `create-group`; `--title` is accepted on `create`, `create-group`, and `edit`; the title is single-line and non-empty, is required on create, and can never be cleared by an edit. Supplying both the positional `TITLE` and `--title` on one invocation is rejected and writes nothing. No document command accepts `--term`.

**Group link and membership.** The owning group is the reserved front-matter key `group`, written and maintained only by the engine: it is not a field, no option sets or changes it, and a member's group link cannot change after creation (ADR-036 decisions 9 and 23). Membership is therefore immutable.

**Listing and references.** A list shows the group records and their members in merged order — group order, then member order within a group (ADR-036 decision 9). Reference visibility for this type is `hidden` for both roles, so a group or member list row renders no reference list; labels still render in the Title cell, and the single-document view keeps its References disclosure (ADR-035 as amended by ADR-039 decision 8, ADR-036 decision 29).

**Lifecycle.** Group and member records carry front-matter `status` with the value `active`, a declared custom status, or `archived`; the `status` and `archive` verbs are the only writers, and no create, edit, or field option sets it (REQ-07.001, REQ-05.011). Archiving or deleting a group archives, deletes, or otherwise changes none of its members, and archiving a member preserves its ID (REQ-05.007). Deleting a record permanently removes it under the common document deletion rules (REQ-05.007).

## Acceptance Criteria
- `nfr` is a `GroupedMarkdown` document type whose model comes from the seed table and is never declared in configuration (REQ-07.004).
- A group record has a plain ID `NFR-<n>`; a member record has the dotted ID `NFR-<n>.<m>` naming its group's sequence (REQ-01.005).
- The type declares `group.fields` and `member.fields` from the seed, and a top-level `fields` declaration on the type is rejected (REQ-07.002).
- The group and member field sets are each `story` under the heading `## User Story` and `description` under the heading `## Description`, and no other field is declared.
- `story` and `description` are optional on both roles: a create that omits either succeeds, and an edit may supply or clear either.
- Each field value is stored in the record body under its declared heading, and neither field declares a front-matter key.
- Field sequence position governs body section order, CLI option and help order, and browser form order (ADR-036 decision 20).
- `papyrus nfr create-group <TITLE>` creates a group record with the given title.
- `papyrus nfr create <TITLE> --group <GROUP-ID>` creates a member only when the group exists, is active, and is of the `nfr` type.
- An existing active group of the same type is reused, and a member create never creates a group.
- An unknown, archived, or other-type group is rejected with one validation outcome and writes no file.
- The title is the first positional `TITLE` on `create` and `create-group`, `--title` is accepted on `create`, `create-group`, and `edit`, and no `--term` option exists.
- A create without a title, with an empty title, with a multi-line title, or with both the positional `TITLE` and `--title` is rejected and writes no file; an edit cannot clear the title.
- The group link is the reserved front-matter `group` key, maintained only by the engine; it is not a field, and no create or edit option sets or changes it.
- A member's group link cannot be changed after creation.
- A list shows group records and their members in group order, then member order.
- Reference visibility is `hidden` for both roles: a group or member row renders no reference list, while labels still render and the single-document view keeps its References disclosure.
- Group and member records carry front-matter `status`, and only the `status` and `archive` verbs write it; no create or edit option sets it (REQ-07.001, REQ-05.011).
- Archiving or deleting a group changes no member's file, status, or ID.
- Archiving a member hides it from lists, search, and the web UI, moves it to the type's `archived/` subfolder, and preserves its ID (REQ-05.007).
- Deleting a member permanently removes it under the document deletion rules (REQ-05.007).
- The family command is `papyrus nfr`; no `papyrus nfreq` alias exists.

## Open Questions
- Whether records whose group link is stored in a legacy `## Group` body section are migrated to the reserved front-matter `group` key. This requirement fixes the target link location only, and no accepted source defines a migration.

## Traceability
- Source: customer-feedback handoff — in-session user feedback, Item 10; and the accepted owner decisions of RE-010 (not reopenable): D6 (only `req` and `nfr` are grouped), D7 (immutable membership, engine-owned link, no cascade, merged order), D8 (reference visibility is seed data), D10 (presentation facts in the seed), D13 (title contract).
- ADR-036 — Config-declared document field schema and one descriptor model: decision 7 (grouped model rules), decision 9 (plain and dotted IDs, required active same-type group, engine-owned link, no cascade, merged order), decisions 18-20 (required, verbs, sequence position), decisions 23-24 (reserved keys and title), decisions 26-29 (role field sets and seed table: `nfr` has the same field sets as `req`), decision 32 (seed fill).
- ADR-039 — One descriptor-driven document view and edit surface: decision 15 (role field map), decision 20 (role-aware rendering and read-only member list), decision 30 (seed presentation facts), decision 32 (seed reference visibility).
- ADR-035 — Per-family reference visibility in the shared browser list layout, as amended by ADR-039 decision 8.
- ADR-0017 — Artifact category registry and type descriptors (decisions 5, 6, and 8 remain in force; decisions 1-4, 7, 9, and 10 replaced by ADR-036).
- ADR-0018 — Generic document content model (superseded in full by ADR-036).
- REQ-07.001 — Organize Artifacts into Three Categories (document status model).
- REQ-07.002 — Configure Document Types and Slugs (role blocks, field items, `nfr` slug, no `nfreq` alias).
- REQ-07.003 — Dispatch Config-Defined Document Commands (`create-group`, `--group`, field options, title contract).
- REQ-07.004 — Provide One Schema-Driven Document Content Model with Three Document Models (grouped model, IDs, immutable membership).
- REQ-01.004 — Use Common CLI Controls and Outcomes (validation outcome).
- REQ-01.005 — Assign Stable Record IDs (plain and hierarchical IDs).
- REQ-05.007 — Manage Document Archive State and Deletion.
- REQ-05.010 — Record Metadata in YAML Front Matter (reserved general keys, `group`).
- REQ-05.011 — Maintain Record Metadata Through All Commands (`status` exemption, field options).
- BR-010 — Papyrus nfr Command Behavior (needs revision for the seed-declared fields, the grouped IDs, and the title contract).
