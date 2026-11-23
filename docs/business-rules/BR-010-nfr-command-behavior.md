---
id: "BR-010:"
title: "Papyrus nfr Command Behavior"
created_date: '2026-11-14T18:39:51Z'
updated_date: '2026-11-20T21:13:34Z'
labels: []
references: []
status: active
---
# BR-010: Papyrus nfr Command Behavior

## Description

`papyrus nfr` is the command family generated from the `GroupedMarkdown` `nfr` descriptor: `create-group` writes a group record with a plain ID, `create` writes a dotted-ID member only under an existing active group of the same type, the engine-owned group link is immutable and never a field or option, archive and delete cascade to nothing, and lists merge group rows then member rows with no reference list.

## Rationale

Based on REQ-005.002 — Manage Non-Functional Requirements, REQ-007.003 — Dispatch Config-Defined Document Commands, REQ-007.004 — Provide One Schema-Driven Document Content Model with Three Document Models, and REQ-005.007 — Manage Document Archive State and Deletion, this rule exists because `nfr` is one of only two `GroupedMarkdown` types, so the generation step must derive the extra `create-group` verb, the required `--group` on create, the plain and dotted ID shapes, the engine-owned group link, the non-cascading archive, and the merged ordering from the descriptor and the seed rather than from a per-type module. Non-functional requirements are kept in their own groups so they never mix with business requirements, so accepting a cross-type or archived group, exposing a second group link, or cascading an archive would silently corrupt the requirement corpus; RE-010 D7 fixes these invariants as accepted and not reopenable.

## Applicability

Applies to every invocation of `papyrus nfr` and to every non-functional-requirement group and member record, in projects where the `nfr` entry is enabled.

### Type entry

- `nfr` is the seed `GroupedMarkdown` entry (prefix `NFR`, directory `requirements/non-functional/`); its model, roles, and field sets are seed data and are never declared in configuration.
- `group.fields` and `member.fields` are each `story` (`## User Story`) then `description` (`## Description`), and a top-level `fields` on the entry is rejected.
- The family is addressed by the slug `nfr`: there is no `nfreq` command and no `nfreq` alias.
- Renaming the entry key renames the command and moves no existing file; prefix, directory, statuses, slots, aliases, and presentation facts are entry data (BR-016).
- IDs use the entry prefix and `id_padding` and are preserved on archive.

### Verbs

- The model generates `create`, `list`, `view`, `edit`, `archive`, `delete`, and `status <id> <value>`, plus `create-group` and a required `--group` on `create`; no other extension verb exists (BR-001).
- `--story` and `--description` are offered in field-sequence order for the role the invocation targets; both fields are optional, so a create that supplies only a title succeeds, and `--editor` edits the body.
- The title is the first positional `TITLE` on `create` and `create-group` and `--title` on `create`, `create-group`, and `edit`; it is single-line and non-empty, required on create, never clearable by an edit, and no `--term` option exists.

### Groups and members

- `create-group <TITLE>` writes one group record with a plain ID `NFR-<n>` and never writes a member.
- `create <TITLE> --group <GROUP-ID>` writes one member with the dotted ID `NFR-<n>.<m>`, where `<n>` is the owning group's sequence and `<m>` its position in that group.
- The named group must already exist, be `active`, and belong to `nfr`: an unknown group, an archived group, and a group of another document type are each rejected with one validation outcome that writes nothing.
- A member create reuses an existing active group of the same type and never creates a group implicitly.
- The owning group is the reserved front-matter `group` key, written and maintained only by the engine; it is not a field, no option sets or changes it, and an edit cannot move a member, so membership is immutable after creation.

### Lists and references

- A list renders the group records and their members merged in group order, then member order within each group, and a group row carries its read-only member list.
- Reference visibility is `hidden` for both roles: a group or member list row renders no reference list, while labels still render in the Title cell and the single-document view keeps its References disclosure.

### Status and archive

- Status is generated for both roles: `active`, `archived`, or a configured custom status; creation writes the configured `default_status` or `active`, and only `status` and `archive` write it, so no create, edit, or field option sets it.
- `archive` sets `archived`, moves the file into the entry's `archived/` subfolder, keeps the ID, discards any previous custom status value, and has no inverse command.
- `status` never archives: `status <id> archived` is rejected with a validation outcome naming `archive` as the only way, and `status` on an archived record is rejected without a move.
- Archiving or deleting a group changes no member's file, status, or ID; a member is archived or deleted only by a command that names that member, and deleting a record is permanent with confirmation, with `--force` skipping the prompt.
- Decision records and the fixed milestone and release lifecycles are separate and unaffected.
