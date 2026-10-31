# REQ-02.003 List and Filter Tasks

## User Story
As a project contributor, I want to find tasks by their attributes and text so that I can focus on relevant work.

## Group
REQ-02 Task and Board Management

## Criticality
Must have

## Description
The task list supports filtering by status, milestone, release, label, assignee, and text, limiting the number of results, and sorting by a selected field. A release filter includes tasks belonging to milestones assigned to that release. Watch mode keeps the list open and refreshes it on changes.

## Acceptance Criteria
- Each documented task-list filter restricts results to matching tasks.
- A release filter includes tasks on milestones assigned to that release.
- `--limit` caps the number of results and `--sort <field>` sorts by the requested field.
- `--watch` keeps the list open and refreshes it when changes occur.

## Traceability
- README.md, `papyrus(1)` → COMMANDS → task list (lines 143–171)
