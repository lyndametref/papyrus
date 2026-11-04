---
id: TASK-9
title: Manage tasks and task lifecycle from the CLI
status: Done
assignee: []
created_date: '2026-10-31T22:25:00Z'
updated_date: '2026-11-04T19:30:00Z'
labels: []
milestone: m-2
dependencies:
  - TASK-8
references:
  - docs/requirements/business/REQ-02.md
  - docs/requirements/business/REQ-02.001.md
  - docs/requirements/business/REQ-02.002.md
  - docs/requirements/business/REQ-02.003.md
  - docs/requirements/business/REQ-02.004.md
  - docs/requirements/non-functional/NFR-02.002.md
  - docs/requirements/non-functional/NFR-04.001.md
  - docs/requirements/non-functional/NFR-04.002.md
  - docs/business-rules/BR-005-task-command-behavior.md
  - docs/specifications/task-management.md
ordinal: 9000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Implement `papyrus task` commands (create, list, view, edit, archive, restore, delete) per REQ-02/REQ-02.001-004 and BR-005, with stable task IDs, configurable workflow status, task-to-milestone assignment, dependencies, labels, assignee, acceptance criteria, comments, filtering, sorting, watch mode, archive/restore/delete lifecycle, and UTF-8 text persistence inside the project directory.

Scope and behavior are defined in docs/specifications/task-management.md. CLI contract is BR-005 (verbatim manpage excerpt). Task records are stored under `<backlog_directory>/tasks/` (active) and `<backlog_directory>/tasks/archived/` (archived).

Dependencies: TASK-8 (milestones/releases) completed. This ticket is prerequisite for TASK-10 (search) and TASK-11 (browser).

Assumptions: see specification Assumptions section. Open questions: see specification Open questions section (OQ-1..OQ-4). UI mockups: none (CLI-only).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 papyrus task create <title> creates a task and returns/records its task ID; description, acceptance criteria, dependencies, milestone, labels, and assignee can be supplied with the documented options; repeatable options accept multiple values.
- [ ] #2 Task IDs use the configured task prefix and padding (default TASK-001) and new tasks use the configured default status (default ToDo); IDs are never reused across active and archived tasks.
- [ ] #3 papyrus task view <id> and papyrus task <id> display the same task; documented edit options update the indicated fields; repeatable options append; --editor opens the task body in $VISUAL or $EDITOR; field flags update metadata without launching an editor.
- [ ] #4 Task list supports status, milestone, release, label, assignee, and text filters; --limit caps results; --sort sorts by the requested field; --watch keeps the list open and refreshes on changes.
- [ ] #5 A release filter includes tasks whose assigned milestone belongs to that release.
- [ ] #6 Task dependency, milestone assignment, archive, restore, delete, and confirmation behaviors match BR-005: archive preserves ID, dependencies, and status; restore returns the previous status; delete archives an active task first; --force skips confirmation.
- [ ] #7 Task data is stored in project-local UTF-8 text files under backlog/tasks/ and backlog/tasks/archived/; user input is sanitized before use; no reads or writes occur outside the project directory except during init.
- [ ] #8 Common CLI controls and outcomes inherited from TASK-2 apply: --json, --no-color/NO_COLOR, help/version, and exit statuses 0/1/2/3/4.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Implementation compiles with cargo build
- [ ] #2 Unit and integration tests for task commands pass
- [ ] #3 CLI behavior matches README manpage and BR-005
- [ ] #4 Feature specification updated at docs/specifications/task-management.md
- [ ] #5 No reads or writes outside the project directory except during init
<!-- DOD:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Quality review (gatekeeper): Ready to close. All AC #1-#8 pass; DoD pass. cargo build, cargo test (5/5 task_cli, 41 total), clippy -D warnings, fmt --check all pass. Scope: src/task.rs, src/lib.rs, tests/task_cli.rs, spec doc. NFR-02.002/NFR-04.001/NFR-04.002 addressed.
<!-- SECTION:NOTES:END -->
