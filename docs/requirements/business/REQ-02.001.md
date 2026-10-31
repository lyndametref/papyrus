# REQ-02.001 Create and Describe Tasks

## User Story
As a project contributor, I want to create tasks with planning details so that work and its completion expectations can be recorded.

## Group
REQ-02 Task and Board Management

## Criticality
Must have

## Description
The user can create a task with a title and optional description, repeatable acceptance criteria, dependencies, labels, and an assignee, and can associate it with a milestone. A task may also reference zero or more business requirements, non-functional requirements, UI specifications, technical specifications, audit records, and business rules. Task IDs use the configured task prefix and padding; the default example is `TASK-001`. New tasks use the configured default status, which defaults to `ToDo`.

## Acceptance Criteria
- `papyrus task create <title>` creates a task and returns/records its task ID.
- Description, acceptance criteria, dependencies, milestone, labels, and assignee can be supplied with the documented options; repeatable options accept multiple values.
- The assigned milestone and dependencies are represented on the created task.
- A task can reference zero or more of each documented requirement, specification, audit-record, and business-rule type.
- With default settings, the task ID follows the `TASK-001` format and its status is `ToDo`.

## Traceability
- README.md, `papyrus(1)` → DESCRIPTION and COMMANDS → task create (lines 28–33, 109–142)
- README.md, `papyrus-config(5)` → General and Workflow keys (lines 949–989)
