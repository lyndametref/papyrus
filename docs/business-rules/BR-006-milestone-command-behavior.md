---
id: "BR-006:"
title: "Papyrus milestone Command Behavior"
created_date: '2026-11-15T13:32:07Z'
updated_date: '2026-11-16T16:57:06Z'
labels: []
references: []
status: active
---
# BR-006: Papyrus milestone Command Behavior

## Description

The `papyrus milestone` command manages milestones with the verbs `create`, `list`, `view`, `edit`, `close`, and `delete`, where `papyrus milestone close <id>` is the only transition of the fixed `open -> closed` lifecycle; a milestone has no archive or restore command, no configurable status list, and no `--status` transition option.

## Rationale

Based on REQ-03.001 — Manage Milestones and REQ-07.001 — Organize Artifacts into Three Categories, this rule exists because a milestone is a backlog artifact whose lifecycle is fixed by the category model rather than configurable: the state is stored in the front-matter `status` field, `close` is the single command that moves a milestone from `open` to `closed`, and document archive semantics never apply to a milestone. The milestone vocabulary is therefore fixed at `open` and `closed`, and configuration cannot add, remove, or rename either value.

## Applicability

Applies to every invocation of `papyrus milestone` and to every milestone record.

### Command surface

- `create <title> [-d <description>] [--release <id>]`
- `list`
- `view <id>`
- `edit <id> [--title <title>] [-d <description>] [--release <id>] [--no-release [<id>]]`
- `close <id>` is the only transition command and moves the milestone from `open` to `closed`.
- `delete <id> [--force]` permanently deletes the milestone; `--force` skips the confirmation prompt.
- No `--status` option performs the transition, and no archive or restore verb is declared.

### Lifecycle

- Fixed `open -> closed`, reached only through `papyrus milestone close <id>`; `statuses` or `default_status` configuration cannot change it.
- A milestone starts `open` and its state is stored in the front-matter `status` field.
- `closed` is lifecycle completion, not an archive state; closing changes the front-matter value and there is no archive or restore command for a milestone.
- A milestone may be assigned to zero or one release; a release may group multiple milestones. Assigning a release does not change the milestone lifecycle.
