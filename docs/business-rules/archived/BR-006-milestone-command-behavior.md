---
id: "BR-006:"
title: "Papyrus milestone Command Behavior"
created_date: '2026-11-15T13:32:07Z'
updated_date: '2026-11-16T20:58:34Z'
labels: []
references:
  - RE-002
status: archived
---
# BR-006: Papyrus milestone Command Behavior

## Description

The `papyrus milestone` command manages milestones with the verbs `create`, `list`, `view`, `edit`, `close`, `archive`, and `delete`. `papyrus milestone close <id>` moves an `open` milestone to `closed`; `papyrus milestone archive <id>` closes an open milestone if needed and then moves its file to `backlog/archive/milestones/`, marking it `archived`. The milestone lifecycle is fixed as `open -> closed -> archived`; there is no restore command, no configurable status list, and no `--status` transition option.

## Rationale

Based on REQ-003.001 — Manage Milestones and REQ-007.001 — Organize Artifacts into Three Categories, this rule exists because a milestone is a backlog artifact whose lifecycle is fixed by the category model rather than configurable: the state is stored in the front-matter `status` field (REQ-005.010), `close` moves `open` to `closed`, and `archive` closes an open milestone if needed before moving its file to `backlog/archive/milestones/` and marking it `archived` (RE-002). Archived milestones are hidden from active listings and there is no restore command; configuration cannot add, remove, or rename lifecycle values.

## Applicability

Applies to every invocation of `papyrus milestone` and to every milestone record.

### Command surface

- `create <title> [-d <description>] [--release <id>]`
- `list`
- `view <id>`
- `edit <id> [--title <title>] [-d <description>] [--release <id>] [--no-release [<id>]]`
- `close <id>` moves an `open` milestone to `closed`.
- `archive <id>` closes an open milestone if needed, then moves it to `backlog/archive/milestones/` and marks it `archived`; archived milestones are hidden from active listings.
- `delete <id> [--force]` permanently deletes the milestone; `--force` skips the confirmation prompt.
- No `--status` option performs the transition, and no restore verb is declared.

### Lifecycle

- Fixed `open -> closed -> archived`: a milestone starts `open`; `closed` is reached only through `papyrus milestone close <id>`; `archived` is reached only through `papyrus milestone archive <id>`, which closes first if needed.
- A milestone starts `open` and its state is stored in the front-matter `status` field.
- `closed` is not an archive state; `archive` moves the file to `backlog/archive/milestones/` and marks it `archived`.
- Archived milestones are hidden from active listings and there is no restore command.
- A milestone may be assigned to zero or one release; a release may group multiple milestones. Assigning a release does not change the milestone lifecycle.
- `statuses` or `default_status` configuration cannot change the lifecycle, and milestone archive uses the backlog archive location rather than the document-type archive flow.

## Requirements
- REQ-003.001
