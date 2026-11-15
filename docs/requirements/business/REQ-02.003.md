# REQ-02.003 List and Filter Tasks

## User Story
As a project contributor, I want to find tasks by their attributes and text so that I can focus on relevant work.

## Group
REQ-02 Task and Board Management

## Criticality
Must have

## Description
The task list supports filtering by status, milestone, release, label, assignee, and text, limiting the number of results, and sorting by a selected field. A release filter includes tasks belonging to milestones assigned to that release. Watch mode keeps the list open and refreshes it on changes.

`--ready` restricts the list to unfinished tasks whose every dependency is resolved to the project's configured final task status. The final task status is the last entry of the configured task `statuses` list (REQ-01.002); it is not a separately configured key. A task is unfinished when its status is not that final status. Readiness is derived at read time from the stored `dependencies` front matter (REQ-05.010); no readiness field is stored and no data shape changes. A dependency that is missing, unknown, ambiguous, self-referential, part of a cycle, or not in the final status blocks the task (fail-closed). An archived dependency counts as resolved only when it is in the final status.

`--plain` forces non-interactive plain-text output for the task list. It is mutually exclusive with the global `--json` option: specifying both is a usage error. `--plain` may be combined with `--ready` and with the other filters.

## Acceptance Criteria
- Each documented task-list filter restricts results to matching tasks.
- A release filter includes tasks on milestones assigned to that release.
- `--limit` caps the number of results and `--sort <field>` sorts by the requested field.
- `--watch` keeps the list open and refreshes it when changes occur.
- `--ready` restricts results to unfinished tasks whose every dependency is resolved to the configured final task status.
- A dependency that is missing, unknown, ambiguous, self-referential, cyclic, or not in the final status blocks its dependent task.
- An archived dependency counts as resolved only when it is in the final status.
- Readiness is derived at read time and is not stored in task front matter.
- `--plain` forces non-interactive plain-text task-list output.
- Specifying both `--plain` and `--json` is a usage error (exit code 2, REQ-01.004).
- `--ready` and `--plain` may be combined with each other and with the other documented filters.

## Traceability
- README.md, `papyrus(1)` → COMMANDS → task list (lines 143–171)
- REQ-01.002 — Read and Edit Project Configuration (configured task `statuses`; final status is the last entry)
- REQ-01.004 — Use Common CLI Controls and Outcomes (usage error for mutually exclusive options)
- REQ-02.001 — Create and Describe Tasks (`dependencies` on task records)
- REQ-05.010 — Record Metadata in YAML Front Matter (`dependencies` front matter)
- Maintainer decision 2026-11-15 (in-session feedback): `--ready` and `--plain` scoped to `papyrus task list` only; `--plain` mutually exclusive with `--json`; readiness fail-closed.
