# BR-005: Papyrus task Command Behavior

## Description

The papyrus task command performs the behavior stated in its Papyrus manpage entry (README.md, papyrus(1) → COMMANDS → task); behavior not stated there is unspecified.

## Rationale

Based on REQ-02.001 — Create and Describe Tasks and REQ-02.002 — View and Edit Tasks and REQ-02.003 — List and Filter Tasks and REQ-02.004 — Archive, Restore, and Delete Tasks, this rule exists because the documented task behavior is the contract for papyrus task; README.md is the sole behavior source.

## Applicability

Applies to all invocations of `papyrus task` and the project data they manage. The excerpt below reproduces relevant manpage behavior verbatim; heading identifies README section and exact source line range.

### papyrus(1): task — `README.md` lines 109–228

```text
task

        Manage tasks (TASK-001, ...). A task may be assigned to zero or one milestone, may depend on zero or more tasks, and may reference zero or more business requirements, non-functional requirements, UI specifications, technical specifications, audit records, and business rules.
       
       papyrus task create <title> [options]
       papyrus task list [options]
       papyrus task view <id>
       papyrus task <id>
       papyrus task edit <id> [options]
       papyrus task archive <id>
       papyrus task restore <id>
       papyrus task delete <id> [--force]
       
       create
       Create a task.

          -d, --description <text>
          Task description.

          --ac <criteria>
          Acceptance criterion. Repeatable.

          --depends-on <id>
           Task dependency. Repeatable; tasks may depend on zero or more tasks.

          --milestone <id>
          Parent milestone.

          --label <label>
          Label. Repeatable.

          --assignee <name>
          Assignee.

       list
       List tasks.

          -s, --status <status>
          Filter by status. See STATUSES.

          --milestone <id>
          Filter by milestone.

          --release <id>
          Filter by release. Collects tasks belonging to milestones that belong to the release.

          --label <label>
          Filter by label.

          --assignee <name>
          Filter by assignee.

          --search <text>
          Filter by text.

          --limit <n>
          Maximum number of results.

          --sort <field>
          Sort field, for example id, created, or status.

          --watch
          Keep the list open and refresh on changes.

       view <id>
       View a task.

       edit <id>
       Edit a task.

          --title <title>
          Update the title.

          -d, --description <text>
          Update the description.

          --ac <criteria>
          Add an acceptance criterion. Repeatable.

          --comment <text>
          Add a comment.

          --comment-author <name>
          Set the comment author.

          --status <status>
          Set the status.

          --depends-on <id>
          Add a dependency. Repeatable.

          --milestone <id>
          Set the parent milestone.

          --no-milestone [<id>]
          Remove the specified milestone; without an ID, remove all milestone assignments.

          --label <label>
          Add a label. Repeatable.

          --assignee <name>
          Set the assignee.

          --editor
          Open the task body in $VISUAL or $EDITOR. All repeatable options add to the existing set; field flags update metadata without launching an editor.

       archive <id>
       Move a task off the board. The ID and every dependency are preserved.

       restore <id>
       Return an archived task to the board with its previous status.

       delete <id> [--force]
       Permanently delete a task. An archived task can be deleted; an active task is archived first.

          --force
          Skip confirmation prompt.

       task view <id> and task <id> are equivalent.
```
