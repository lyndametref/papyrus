---
id: "BR-007:"
title: "Papyrus release Command Behavior"
created_date: '2026-11-14T18:39:51Z'
updated_date: '2026-11-16T16:57:06Z'
labels: []
references: []
status: active
---
# BR-007: Papyrus release Command Behavior

## Description

The `papyrus release` command manages releases with the verbs `create`, `list`, `view`, `edit`, and `close`, where `papyrus release close <id>` is the only transition of the fixed `planned -> released` lifecycle; releases are not documents and have no archive, restore, or delete command.

## Rationale

Based on REQ-003.002 — Manage Releases and REQ-007.001 — Organize Artifacts into Three Categories, this rule exists because a release is a backlog artifact whose lifecycle is fixed by the category model rather than configurable: the state is stored in the front-matter `status` field, `close` is the single command that moves a release from `planned` to `released`, and document archive semantics never apply to a release.

## Applicability

Applies to every invocation of `papyrus release` and to every release record.

### Command surface

- `create`, `list`, `view`, and `edit` (`--title`, `-d/--description`).
- `close <id>` is the only transition command and moves the release from `planned` to `released`.
- No `--status` option performs the transition, and no archive, restore, delete, or other document verb is declared.

### Lifecycle

- Fixed `planned -> released`, reached only through `papyrus release close <id>`; `statuses` or `default_status` configuration cannot change it.
- `released` represents delivery completion and is distinct from archiving; releases are not archived.
- Release status is stored in the front-matter `status` field and is neither a document main status nor configurable, and a release is never moved to a document `archived/` folder.
- A release groups milestones; each milestone may be assigned to at most one release.
