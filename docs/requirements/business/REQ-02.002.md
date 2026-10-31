# REQ-02.002 View and Edit Tasks

## User Story
As a project contributor, I want to inspect and update a task so that its current information and progress remain useful.

## Group
REQ-02 Task and Board Management

## Criticality
Must have

## Description
The user can view a task using either `task view <id>` or `task <id>`, and update its title, description, status, milestone, assignee, or add acceptance criteria, comments, dependencies, and labels. Repeated add-options append to the existing set. Field flags update metadata without opening an editor; `--editor` opens the task body in the selected editor.

## Acceptance Criteria
- `papyrus task view <id>` and `papyrus task <id>` display the same task.
- Documented task edit options update the indicated task fields; repeatable options add entries.
- `--editor` opens the task body in `$VISUAL` or `$EDITOR`.
- Field flags update metadata without launching an editor.

## Traceability
- README.md, `papyrus(1)` → COMMANDS → task view/edit (lines 173–213), ENVIRONMENT (lines 856–857)
