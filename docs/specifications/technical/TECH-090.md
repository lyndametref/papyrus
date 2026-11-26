---
id: TECH-090
title: "Backlog: Tasks"
created_date: '2026-11-25T20:42:10Z'
updated_date: '2026-11-25T20:42:11Z'
labels: []
references:
  - REQ-002
  - REQ-002.001
  - REQ-002.002
status: draft
---
# TECH-090 Backlog: Tasks

## Description
Covers the task record, its attributes, its command family, the configured task
statuses, its archive, restore, and delete lifecycle, and the task-specific view
and list surface. Milestones, releases, the Backlog board, and the shared
command dispatch are outside this specification.

## Record

| Attribute | Rule |
|---|---|
| Identifier | Configured task prefix plus the shared `id_padding`; three digits with default configuration |
| Title | Required; first positional argument of `create` |
| Description | Optional, multiline; `-` reads the value from standard input |
| Acceptance criteria | Repeatable |
| Dependencies | Repeatable |
| Milestone | Optional; at most one |
| Labels | Repeatable; free-form classification |
| References | Repeatable; may name any managed record, no type excluded |
| Assignees | List |
| Ordinal | Optional non-negative whole number |
| Status | Configured task status |
| Dates | Created date and updated date |

## Command family

`papyrus task create|edit|archive|restore|delete|list|view`.

- `papyrus task create <TITLE>` creates a task and reports its identifier.
- Title is required and is the first positional argument.
- Description, acceptance criteria, dependencies, milestone, labels, references,
  assignees, and ordinal are optional.
- Repeatable options accept multiple values on both create and edit.
- `papyrus task create --help` lists the options.

## Identifier and status

- A task identifier uses the configured task prefix and the shared `id_padding`;
  with default configuration the prefix is `TASK` and the number is three digits
  wide.
- Task statuses are configured; a new task starts at the configured default task
  status, `ToDo` with default configuration.
- The final task status is the last entry of the configured task status list.
- A task is unfinished when its status is not the final task status.

## Ordinal

- The ordinal is a non-negative whole number giving the task its execution order.
- It is optional on create; when omitted it is derived from the current maximum.

## Edit

- `papyrus task edit <ID>` changes the title, description, status, milestone,
  assignees, labels, references, or ordinal.
- An edit can clear the milestone.
- Repeatable add-options append acceptance criteria and dependencies to the
  existing sets rather than replacing them.
- `--editor` opens the task body in the configured editor.
- An option-only edit updates the metadata without opening an editor.

## Archive, restore, and delete

- `papyrus task archive <ID>` takes the task off the working set and moves it to
  the backlog archive folder for tasks.
- Archive preserves the identifier, the status, the dependencies, and the
  references.
- `papyrus task restore <ID>` returns an archived task to the working set with
  the status it had before archiving and the same identifier.
- `papyrus task delete <ID>` permanently removes a task. A task that is not
  archived is archived first.
- Deletion asks for confirmation, and `--force` skips the prompt.

## View

- `papyrus task view <ID>` shows a task; `papyrus task <ID>` shows the same task.
  The identifier shorthand exists for tasks only.
- A view shows the identifier, title, status, description, acceptance criteria,
  milestone, assignees, labels, references, dependencies, ordinal, and the
  created and updated dates.
- `--ac` prints only the acceptance criteria: one criterion per line in plain
  text, and an array under `--json`.

## List

- `papyrus task list` prints tasks as plain text by default and honours the
  global `--json` option.
- Filters: status, milestone, release, label, assignee, and free text.
- A release filter includes the tasks whose milestone belongs to that release.
- Filters combine with one another.
- `--limit` caps the number of rows.
- `--sort <field>` orders the rows by the named field.
- `--watch` keeps the list open and refreshes it when records change.

## Readiness

- `--ready` restricts the task list to unfinished tasks whose every dependency
  has reached the final task status.
- Readiness is derived at read time from the stored dependencies and is never
  stored as a field.
- It fails closed: a dependency that is missing, unknown, ambiguous,
  self-referential, part of a cycle, or not in the final status blocks its
  dependent task.
- An archived dependency counts as resolved only when it is in the final status.
- `--ready` combines with the other documented task-list filters.
