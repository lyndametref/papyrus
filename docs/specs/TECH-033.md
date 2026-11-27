---
id: TECH-033
title: "Record Lists"
created_date: '2026-11-27T20:35:45Z'
updated_date: '2026-11-27T20:35:45Z'
labels: []
references:
  - REQ-002.002
  - REQ-004.003
  - REQ-005.017
status: verified
---
# TECH-033 Record Lists

## Description
Covers console list output for the three record categories: document types, the built-in backlog types, and decision types. Console single-record views are a separate specification; browser lists, layout, and chrome are also separate.

## Shared rules

- A list shows one line per record: the identifier, the title, and the labels.
- Archived records appear in no list.
- Plain text is the default output and `--json` carries the values without console formatting.

## Document lists

- Each line shows the identifier, the title, and the labels.
- A `grouped-record` list shows its group records and their members merged into one sequence, in group order and then member order within each group.
- Console lists do not render declared slots; slots are a browser column.

## Task list

- `papyrus task list` prints one line per task and honours the global `--json` option.
- Filters: status, milestone, release, label, assignee, and free text.
- A release filter includes the tasks whose milestone belongs to that release.
- Filters combine with one another.
- `--limit` caps the number of lines.
- `--sort <field>` orders the lines by the named field.
- `--watch` keeps the list open and refreshes it when records change.

## Readiness

- `--ready` restricts the task list to unfinished tasks whose every dependency has reached the final task status.
- Readiness is derived at read time from the stored dependencies and is never stored as a field.
- It fails closed: a dependency that is missing, unknown, ambiguous, self-referential, part of a cycle, or not in the final status blocks its dependent task.
- An archived dependency counts as resolved only when it is in the final status.
- `--ready` combines with the other documented task-list filters.

## Milestone and release lists

- `papyrus milestone list` and `papyrus release list` print one line per record.
- They accept no filter beyond the global options.

## Decision lists

- `papyrus <slug> list` lists the decisions of that type, one line per record.
- A line shows the identifier, the title, the labels, the type, the status, the creation date, and the update date.
- Archived decisions are excluded, and a cross-type listing is provided by search, not by a shared command.
