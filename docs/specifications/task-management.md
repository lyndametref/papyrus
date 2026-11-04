# Feature Specification — Task Management from the CLI

**Ticket:** TASK-9
**Milestone:** m-2 (Backlog Discovery and Management)
**Status:** Refined (execution-ready)
**UI mockups:** none

## Purpose

Deliver the `task` commands so a project contributor can create, inspect, update, list, filter, sort, archive, restore, and delete tasks, with stable IDs, configurable workflow status, and text-file persistence inside the project directory. This ticket implements REQ-02, REQ-02.001, REQ-02.002, REQ-02.003, and REQ-02.004, together with the documented command contract in BR-005. It provides the task records that later tickets search (TASK-10) and expose in the browser (TASK-11).

## Scope

### In scope

- `papyrus task create <title> [options]` (BR-005, REQ-02.001).
- `papyrus task list [options]`, including all documented filters, `--limit`, `--sort`, and `--watch` (BR-005, REQ-02.003).
- `papyrus task view <id>` and the equivalent `papyrus task <id>` (BR-005, REQ-02.002).
- `papyrus task edit <id> [options]` (BR-005, REQ-02.002).
- `papyrus task archive <id>`, `restore <id>`, and `delete <id> [--force]` (BR-005, REQ-02.004).
- Task identity: stable IDs of the form `<task_prefix>-<number>` (default `TASK-001`) using configured `task_prefix` and `id_padding` (REQ-02.001, BR-005).
- Task workflow status drawn from configured `statuses`, with new tasks using configured `default_status` (default `ToDo`) (REQ-02.001, BR-005, REQ-01.005).
- Task-to-milestone assignment: zero or one milestone per task (REQ-02, REQ-02.001, BR-005).
- Task dependencies: zero or more task IDs (REQ-02, REQ-02.001, BR-005).
- Task fields: title, description, repeatable acceptance criteria, labels, assignee, comments (REQ-02.001, REQ-02.002, BR-005).
- Release filtering that collects tasks whose milestone belongs to the requested release (REQ-02.003, BR-005).
- Persistence as UTF-8 text only inside the project directory (NFR-04.001, NFR-04.002).
- Input sanitization by normalization/escaping before use (NFR-02.002).
- Common CLI controls and outcomes inherited from TASK-2: `--json`, `--no-color`/`NO_COLOR`, help/version, exit statuses 0/1/2/3/4 (REQ-01.004).

### Out of scope

- Search and browser access for tasks — TASK-10 and TASK-11.
- Packaging and verification of the backlog-discovery milestone — TASK-12.
- Managing references from a task to business requirements, non-functional requirements, UI/technical specifications, audit records, or business rules via CLI flags — BR-005 defines no such flags; reference management is deferred to later tickets (consistent with TASK-3).
- Editing or removing individual acceptance criteria, labels, or dependencies once added — BR-005 only documents append behavior for these repeatable options.
- UI mockups or web UI — none for this ticket; CLI output only.
- New configuration keys beyond those already delivered by TASK-2 (`task_prefix`, `id_padding`, `backlog_directory`, `statuses`, `default_status`).

## Behavior

### 1. Project and configuration context

1. Project directory and configuration file selection follow the precedence and canonicalization rules delivered by TASK-2 (REQ-01.003, NFR-04.002).
2. For every command in this ticket, reads and writes stay within the canonicalized project directory (NFR-04.002).
3. The effective configuration supplies `task_prefix` (default `TASK`), `id_padding` (default 3), `backlog_directory` (default `backlog/`), `statuses` (default `ToDo, Initiated, Refined, Implemented, Validated, Done`), and `default_status` (default `ToDo`).
4. Task records are stored under the configured backlog directory: active tasks in `<backlog_directory>/tasks/`, archived tasks in `<backlog_directory>/tasks/archived/`. These directories are created as needed; `init` already creates the active task directory.

### 2. Tasks — create

1. `papyrus task create <title> [options]` creates a task.
2. `<title>` is required and is sanitized before use.
3. `-d, --description <text>` sets an optional description, sanitized before use.
4. `--ac <criteria>` adds an acceptance criterion. Repeatable; each value is sanitized before use and appended to the existing set.
5. `--depends-on <id>` adds a task dependency. Repeatable; each value is sanitized before use and appended to the existing set. Each dependency ID must identify an existing task (active or archived); an unknown ID fails with exit code 3. A task cannot depend on itself (exit code 4).
6. `--milestone <id>` optionally assigns the task to a milestone. The milestone ID is sanitized and must identify an existing milestone (active or archived); an unknown ID fails with exit code 3.
7. `--label <label>` adds a label. Repeatable; each value is sanitized before use and appended to the existing set.
8. `--assignee <name>` sets the assignee, sanitized before use.
9. The task ID is generated as `<task_prefix>-<number>`, padded to `id_padding` digits (default `TASK-001`), where `<number>` is one greater than the highest existing task number found in both the active and `archived/` folders. IDs are never reused.
10. A created task uses the configured `default_status` (default `ToDo`) and has no milestone, dependencies, labels, assignee, description, acceptance criteria, or comments unless supplied.
11. On success the generated task ID is printed (text mode) or returned as JSON.

### 3. Tasks — view

1. `papyrus task view <id>` and `papyrus task <id>` display the same task.
2. Viewing prints the full persisted text of the task identified by `<id>`.
3. Viewing an archived task by ID is allowed and prints its persisted text.
4. Viewing an unknown ID fails with exit code 3.
5. `--json` returns the ID, path, and content.

### 4. Tasks — edit

1. `papyrus task edit <id> [options]` updates only the supplied fields; omitted fields retain their previous values.
2. `--title <title>` updates the title.
3. `-d, --description <text>` updates the description.
4. `--ac <criteria>` adds an acceptance criterion. Repeatable; appended to the existing set.
5. `--comment <text>` adds a comment. Repeatable.
6. `--comment-author <name>` sets the author used for comments added by the same invocation; if omitted, the comment author is recorded as `unknown`.
7. `--status <status>` sets the workflow status. The value must be one of the configured `statuses`; an unknown status fails with exit code 4.
8. `--depends-on <id>` adds a dependency. Repeatable; appended to the existing set. Each dependency ID must identify an existing task; an unknown ID fails with exit code 3. A task cannot depend on itself (exit code 4).
9. `--milestone <id>` sets the parent milestone. The milestone must exist; an unknown ID fails with exit code 3.
10. `--no-milestone [<id>]` removes a milestone assignment: with an ID, it removes that specific assignment (failing with exit code 4 if the task is not currently assigned to that milestone); without an ID, it removes any existing assignment.
11. `--milestone` and `--no-milestone` are mutually exclusive; supplying both fails with exit code 4.
12. `--label <label>` adds a label. Repeatable; appended to the existing set.
13. `--assignee <name>` sets the assignee.
14. `--editor` opens the task body in `$VISUAL` or `$EDITOR` (per effective configuration). Field flags update metadata without launching an editor.
15. All repeatable options add to the existing set; field flags update metadata without launching an editor.
16. Editing an archived task is allowed and keeps it archived.
17. Editing an unknown ID fails with exit code 3.

### 5. Tasks — list and filter

1. `papyrus task list` lists active tasks only; archived tasks are hidden.
2. `-s, --status <status>` restricts results to tasks whose status equals the given value.
3. `--milestone <id>` restricts results to tasks assigned to the given milestone.
4. `--release <id>` restricts results to tasks whose assigned milestone belongs to the given release. Tasks with no milestone are excluded.
5. `--label <label>` restricts results to tasks carrying the given label.
6. `--assignee <name>` restricts results to tasks assigned to the given assignee.
7. `--search <text>` restricts results to tasks whose persisted text contains the given text (case-insensitive).
8. `--limit <n>` caps the number of results. `<n>` must be a positive integer; invalid values are usage errors (exit code 2).
9. `--sort <field>` sorts results by the requested field. Supported fields are `id` (default), `title`, `status`, `milestone`, `assignee`, and `created`. An unknown field fails with exit code 4.
10. `--watch` keeps the list open and re-renders it when task files change; the process exits successfully on interrupt.
11. Output includes at least the ID and title of each task in the selected order.
12. `--json` wraps the output as a stable JSON array.

### 6. Tasks — archive, restore, and delete

1. `papyrus task archive <id>` moves the task file from the active folder to the `archived/` folder. Archiving an already-archived task fails with exit code 4.
2. Archiving an unknown ID fails with exit code 3.
3. Archiving preserves the task ID, status, dependencies, and all other fields.
4. `papyrus task restore <id>` moves an archived task back to the active folder with the same ID and its previous status. Restoring an active task fails with exit code 4.
5. Restoring an unknown ID fails with exit code 3.
6. `papyrus task delete <id> [--force]` permanently deletes the task file (from either the active or `archived/` folder).
7. Deleting an active task archives it first and then deletes the archived file.
8. Without `--force`, the command prompts for confirmation on standard input and aborts (exit 0, no change) unless the user confirms.
9. With `--force`, the confirmation prompt is skipped.
10. Deleting an unknown ID fails with exit code 3.

### 7. Status and lifecycle

1. Task workflow statuses are exactly the configured `statuses` (default `ToDo -> Initiated -> Refined -> Implemented -> Validated -> Done`).
2. New tasks start at the configured `default_status` (default `ToDo`).
3. `Done` is the only terminal status; the command does not enforce transition ordering and accepts any configured status.
4. Archiving is record management and is independent of workflow status.

### 8. Dependencies

1. A task may depend on zero or more tasks.
2. Dependencies are stored as task IDs on the dependent task.
3. The CLI only adds dependencies; it does not remove individual dependencies.
4. A dependency is preserved across archive, restore, edit, and status changes.
5. Deleting a task does not automatically remove references to it from other tasks' dependency lists (Assumption: dangling dependency references are preserved and reported on view).

### 9. Milestone assignment

1. A task is assigned to zero or one milestone.
2. Assigning a task to a milestone requires that milestone to exist.
3. Removing or deleting a task does not modify the milestone file.
4. A milestone may be referenced by multiple tasks.
5. Deleting or archiving a milestone does not automatically clear task assignments (Assumption: dangling milestone references are preserved and reported on view).

### 10. Persistence and sanitization

1. Tasks are stored as UTF-8 Markdown text files named `<ID>.md` in their respective folders (NFR-04.001).
2. File content uses the following style: a top-level heading `# <ID> <title>`, then `## Description` (when provided), `## Status` (always), `## Milestone` (when assigned), `## Assignee` (when set), `## Labels` (when any, one per line as `- <label>`), `## Dependencies` (when any, one per line as `- <id>`), `## Acceptance Criteria` (when any, one per line as `- [ ] <criteria>`), and `## Comments` (when any, one entry per comment as `- **<author>**: <text>`). Empty optional fields are omitted. Status is always present.
3. All user-controlled input (title, description, status, milestone ID, dependency IDs, labels, assignee, acceptance criteria, comments, comment author) is sanitized by normalization/escaping before use (NFR-02.002).
4. Values that cannot be normalized into a valid form fail with exit code 4 (NFR-02.002).
5. No reads or writes occur outside the canonicalized project directory (NFR-04.002).
6. No binary data is written (NFR-04.001).

### 11. Common CLI controls and outcomes

1. `--version`, `-h`/`--help`, `--json`, `--no-color`/`NO_COLOR`, and exit statuses 0/1/2/3/4 behave as delivered by TASK-2 (REQ-01.004).
2. Missing or invalid usage (including missing required title, mutually exclusive flags, invalid `--limit`) exits 2 when it is a usage error, or 4 when it is a validation error, following clap's classification.
3. Unknown record, milestone, or dependency ID exits 3.
4. Validation/conflict (unnormalizable input, mutually exclusive flags, already-archived task, unknown status, self-dependency) exits 4.
5. Filesystem/IO failure or project-directory boundary violation exits 1.

## Inputs and outputs

- `task create`: inputs title plus optional description, acceptance criteria, dependencies, milestone, labels, assignee; output is the generated task ID (text) or JSON.
- `task list`: output is IDs and titles of active tasks (text table or JSON array), subject to filters, sorting, and limit.
- `task view <id>` / `task <id>`: output is the full file text (text) or JSON object containing ID, path, and content.
- `task edit <id>`: inputs optional fields and flags; output minimal success text or JSON.
- `task archive <id>`: moves file; minimal success output.
- `task restore <id>`: moves file back; minimal success output.
- `task delete <id> [--force]`: deletes file; minimal success output (or confirmation prompt without `--force`).

## Validation and error behavior

- Missing required title → exit code 2 (usage).
- Unknown task, milestone, or dependency ID → exit code 3 (not found).
- Invalid status, unnormalizable input, mutually exclusive flags, already-archived/active state conflict, self-dependency, unknown sort field → exit code 4 (validation/conflict).
- Invalid `--limit` or other usage error → exit code 2.
- IO failure or boundary violation → exit code 1.
- `--json` error output includes the exit-relevant error information, as delivered by TASK-2.

## Assumptions

- Assumption: Rust 1.99.1 via rustup and Cargo are available (ADR-0001).
- Assumption: `init` already creates `backlog/tasks/`; this ticket creates `backlog/tasks/archived/` on first archive if absent.
- Assumption: archive moves the task file into `backlog/tasks/archived/` rather than recording status in place, consistent with milestone handling (TASK-8) and keeping archived tasks out of `list`.
- Assumption: task dependencies are not validated for cycles; a task may not depend on itself.
- Assumption: dependency IDs must identify existing tasks (active or archived); otherwise the command fails with exit code 3.
- Assumption: milestone and release references that become dangling are preserved and reported on view rather than silently cleared.
- Assumption: `--watch` refreshes by polling the task directory; the exact interval and filesystem-event mechanism are implementation choices.
- Assumption: sort field `created` uses the task file's creation time when available and falls back to ID ordering when it is not.
- Assumption: comment authorship defaults to `unknown` when `--comment-author` is omitted.
- Assumption: no UI mockups are required; this is a CLI-only ticket.
- Assumption: single-crate layout (`src/main.rs` + `src/lib.rs`) is retained; a new `task` module may be added within the crate without a new ADR.

## Open questions

- OQ-1: Whether `--watch` should exit on a key press in addition to interrupt is unspecified; this specification assumes interrupt (Ctrl+C / SIGINT) only.
- OQ-2: Whether dependency removal should be supported is not documented in BR-005; this specification does not add it.
- OQ-3: Whether task references to requirements, specifications, audit records, and business rules should become CLI-managed is deferred; BR-005 defines no flags for them.
- OQ-4: The exact archive folder name and whether archived tasks remain editable are implementation choices documented as assumptions above.

## Traceability

| Artifact | Reference |
|---|---|
| Requirement | REQ-02, REQ-02.001, REQ-02.002, REQ-02.003, REQ-02.004 |
| Non-functional | NFR-02.002, NFR-04.001, NFR-04.002 |
| Business rule | BR-005 (task command behavior) |
| ADR | ADR-0001 (Rust), ADR-0002 (amd64 .deb) |
| Ticket | TASK-9 |
| UI mockups | none |
| Related specification | docs/specifications/milestone-and-release-management.md (TASK-8) |
