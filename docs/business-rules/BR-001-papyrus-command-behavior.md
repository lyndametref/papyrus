# BR-001: Papyrus Command Behavior

## Description

Each Papyrus command and subcommand performs the behavior stated in its Papyrus manpage entry; behavior not stated there is unspecified.

## Rationale

REQ-01 — Project Setup and CLI Access; REQ-02 — Task and Board Management; REQ-03 — Milestone and Release Planning; REQ-04 — Decision Records; REQ-05 — Project Documentation Management; and REQ-06 — Search and Local Web Interface ground the documented command areas. `README.md` is the sole behavior source for this rule.

## Applicability

Applies to all Papyrus command invocations and the project data they manage. The excerpts below reproduce relevant manpage behavior verbatim. Each heading identifies the README section and exact source line range.

### papyrus(1): SYNOPSIS, DESCRIPTION, GLOBAL OPTIONS, and COMMANDS — `README.md` lines 20–705

```text
SYNOPSIS

       papyrus <command> [options]

       papyrus <command> <subcommand> [options]

DESCRIPTION

       papyrus manages a project's backlog and long-term documentation from a single command-line interface. It combines:

       - **Backlog management** - tasks, milestones, releases, and decisions.
        - **Documentation management** - business and non-functional requirements, glossary terms, UI and technical specifications, audit records, and business rules.

       All project data lives in the project directory selected with --dir (default: the current working directory). IDs are generated automatically and remain stable across archive/restore and deprecate/restore cycles.

GLOBAL OPTIONS

       --json
       Emit stable, machine-readable JSON. Applies to every command that produces output, including errors.

       --dir <path>
       Project directory. Default: the current working directory.

       --config <file>
       Configuration file. Default: papyrus.config.yml in the project directory.

       --no-color
       Disable ANSI color output. The NO_COLOR environment variable has the same effect.

       --version
       Print the version and exit.

       -h, --help
       Print help and exit. Use papyrus <command> --help for command help.

COMMANDS

init

       Initialize a project in the directory where papyrus init is run. If --dir <path> is given, initialize in that directory instead; the directory is created if it does not exist. Initialization creates the backlog store, the documentation structure, and papyrus.config.yml.
       
       papyrus init <name> [--no-git] [--force]
       
       <name> is the project name stored in the configuration.

       --no-git
       Do not initialize a Git repository.

       --force
       Allow initialization in a non-empty directory. Existing files are never overwritten.

config

       Read and edit the configuration. Keys use dotted notation for nested values, for example document_types.req.enabled.
       
       papyrus config list
       papyrus config get <key>
       papyrus config set <key> <value>
       papyrus config unset <key>
       
       list
       Print all effective keys, including defaults.

       get <key>
       Print one effective value.

       set <key> <value>
       Persist one value. The value is parsed as a YAML scalar.

       unset <key>
       Remove one key and fall back to its default.

       Precedence is described in papyrus-config(5).

browser

       Serve the local web UI over both the backlog and documentation stores.
       
       papyrus browser [--host <host>] [--port <port>] [--no-open]
       
       --host <host>
       Bind address. Default: 127.0.0.1. A non-loopback address is allowed but prints a warning.

       --port <port>
       TCP port. Default: 6420.

       --no-open
       Do not open the default browser automatically.

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

milestone

       Manage milestones (MS-001, ...). A milestone may be assigned to zero or one release.
       
       papyrus milestone create <title> [-d <description>] [--release <id>]
       papyrus milestone list
       papyrus milestone view <id>
        papyrus milestone edit <id> [--title <title>] [-d <description>] [--release <id>] [--no-release [<id>]]
       papyrus milestone archive <id>
       papyrus milestone delete <id> [--force]
       
       create
       Create a milestone.

          -d, --description <text>
          Milestone description.

          --release <id>
          Assign the milestone to a release.

       list
       List milestones.

       view <id>
       View a milestone.

       edit <id>
       Edit a milestone.

          --title <title>
          Update the title.

          -d, --description <text>
          Update the description.

          --release <id>
          Assign the milestone to a release.

          --no-release [<id>]
          Remove the specified release; without an ID, remove all release assignments.

       archive <id>
       Archive a milestone.

       delete <id> [--force]
       Permanently delete a milestone.

          --force
          Skip confirmation prompt.

release

       Manage releases (REL-001, ...). A release may contain multiple milestones; each milestone may be assigned to zero or one release.
       
       papyrus release create <title>
       papyrus release list
       papyrus release view <id>
       papyrus release edit <id> [--title <title>] [-d <description>]
       
       create
       Create a release.

       list
       List releases.

       view <id>
       View a release.

       edit <id>
       Edit a release.

          --title <title>
          Update the title.

          -d, --description <text>
          Update the description.

decision

       Manage decisions (ADR-001, BDR-001, ...). Decisions may reference zero or more business requirements and zero or more non-functional requirements, and may be superseded by zero or one decision of the same type.
       
        papyrus decision create <title> [-t, --decision-type <type>] [-d <decision>] [--context <text>] [--consequences <text>]
        papyrus decision list [-t, --decision-type <type>]
       papyrus decision view <id>
       papyrus decision edit <id> [--title <title>] [-d <decision>]
                                     [--status <status>] [--context <text>] [--consequences <text>]
                                     [--superseded-by <id>]
       papyrus decision archive <id>
       
       create
       Create a decision.

           -t, --decision-type <type>
          Decision type. Built-in types: adr (default) and bdr. Additional types are configured in papyrus-config(5).

          -d, --decision <text>
          The decision which was taken.

          --context <text>
          Context of the decision.

          --consequences <text>
          Consequences of the decision.

       list
       List decisions.

           -t, --decision-type <type>
           Filter by decision type.

       view <id>
       View a decision.

       edit <id>
       Edit a decision.

          --title <title>
          Update the title.

          -d, --decision <text>
          The decision which was taken.

          --status <status>
          Set the status.

          --context <text>
          Context of the decision.

          --consequences <text>
          Consequences of the decision.

          --superseded-by <id>
           Mark this decision as superseded by at most one other decision of the same type. Cannot be combined with --status.

       archive <id>
       Archive a decision.

       Decision records follow the Nygard ADR template: Title, Status, Decision, Context, and Consequences. Statuses are proposed, accepted, rejected, deprecated, and superseded.

req

         Manage business requirements (REQ-...). Requirement-to-group cardinality is 1..1; each group contains business requirements only.

       
        papyrus req create <title> [--title <title>] [--story <text>] [-d <description>] --group <id>
       papyrus req create-group --title <title> [--story <text>] [-d <description>]
       papyrus req list
       papyrus req view <id>
       papyrus req edit <id> [--title <title>] [--story <text>] [-d <description>] [--editor]
       papyrus req deprecate <id>
       papyrus req restore <id>
       

       create
       Create a business requirement.

          --title <title>
          Requirement title.

          --story <text>
          User story.

          -d, --description <text>
          Description.

          --group <id>
           Add the requirement to its required business-requirement group.

       create-group
        Create a business-requirement group. It can contain business requirements only.

          --title <title>
          Requirement group title.

          --story <text>
          User story.

          -d, --description <text>
          Description.

       list
       List business requirements.

       view <id>
       View a business requirement.

       edit <id>
       Edit a business requirement.

          --title <title>
          Update the title.

          --story <text>
          Update the user story.

          -d, --description <text>
          Update the description.

          --editor
          Open the document body in $VISUAL or $EDITOR.

       deprecate <id>
       Move a business requirement to the deprecated folder. Deprecated documents are hidden from lists, search, and the web UI.

       restore <id>
       Return a deprecated business requirement to its original folder. The ID is preserved.

nfreq

          Manage non-functional requirements (NFR-...). Requirement-to-group cardinality is 1..1; each group contains non-functional requirements only. Non-functional requirements may be referenced by tasks, decisions, and technical specifications.

       
        papyrus nfreq create <title> [-d <description>] --group <id>
       papyrus nfreq create-group --title <title> [-d <description>]
       papyrus nfreq list
       papyrus nfreq view <id>
       papyrus nfreq edit <id> [--title <title>] [-d <description>] [--editor]
       papyrus nfreq deprecate <id>
       papyrus nfreq restore <id>
       

       create
       Create a non-functional requirement.

          -d, --description <text>
          Requirement description.

          --group <id>
           Add the requirement to its required non-functional-requirement group.

       create-group
        Create a non-functional-requirement group. It can contain non-functional requirements only.

          --title <title>
          Requirement group title.

          -d, --description <text>
          Description.

       list
       List non-functional requirements.

       view <id>
       View a non-functional requirement.

       edit <id>
       Edit a non-functional requirement.

          --title <title>
          Update the title.

          -d, --description <text>
          Update the description.

          --editor
          Open the document body in $VISUAL or $EDITOR.

       deprecate <id>
       Move a non-functional requirement to the deprecated folder. Deprecated documents are hidden from lists, search, and the web UI.

       restore <id>
       Return a deprecated non-functional requirement to its original folder. The ID is preserved.

glossary

       Manage glossary terms (GLO-...).

       
       papyrus glossary create --term <term> --definition <text>
       papyrus glossary list
       papyrus glossary view <id>
       papyrus glossary edit <id> [--term <term>] [--definition <text>] [--editor]
       papyrus glossary deprecate <id>
       papyrus glossary restore <id>
       

       create
       Create a glossary term.

          --term <term>
          Glossary term.

          --definition <text>
          Definition of the term.

       list
       List glossary terms.

       view <id>
       View a glossary term.

       edit <id>
       Edit a glossary term.

          --term <term>
          Update the glossary term.

          --definition <text>
          Update the definition.

          --editor
          Open the document body in $VISUAL or $EDITOR.

       deprecate <id>
       Move a glossary term to the deprecated folder. Deprecated documents are hidden from lists, search, and the web UI.

       restore <id>
       Return a deprecated glossary term to its original folder. The ID is preserved.

ui-spec

        Manage UI specifications (UI-...).

       
       papyrus ui-spec create <title> --view <view> --description <text>
       papyrus ui-spec list
       papyrus ui-spec view <id>
       papyrus ui-spec edit <id> [--view <view>] [--description <text>] [--editor]
       papyrus ui-spec deprecate <id>
       papyrus ui-spec restore <id>
       

       create
       Create a UI specification.

          --view <view>
          Associate the specification with a view.

          --description <text>
          Description of the UI specification.

       list
       List UI specifications.

       view <id>
       View a UI specification.

       edit <id>
       Edit a UI specification.

          --view <view>
          Update the associated view.

          --description <text>
          Update the description.

          --editor
          Open the document body in $VISUAL or $EDITOR.

       deprecate <id>
        Move a UI specification to the deprecated folder. Deprecated documents are hidden from lists, search, and the web UI.

       restore <id>
        Return a deprecated UI specification to its original folder. The ID is preserved.

rule

       Manage business rules (RUL-...).

       
        papyrus rule create <title> --description <text> --rationale <text> --applicability <text> [--requirement <id>...]
       papyrus rule list
       papyrus rule view <id>
        papyrus rule edit <id> [--title <title>] [-d <description>] [--requirement <id>...] [--no-requirement [<id>]] [--editor]
       papyrus rule deprecate <id>
       papyrus rule restore <id>
        

        create
        Create a business rule.

          --description <text>
          State the business rule precisely.

          --rationale <text>
          Explain why the rule exists, referencing its requirement(s).

          --applicability <text>
          Describe when and where the rule applies.

          --requirement <id>
           Reference a business requirement. Repeatable; zero or more may be specified.

       list
        List business rules.

       view <id>
        View a business rule.

       edit <id>
        Edit a business rule.

          --title <title>
          Update the title.

          -d, --description <text>
          Update the description.

          --requirement <id>
           Add a business requirement to the existing list. Repeatable.

          --no-requirement [<id>]
           Remove the specified business requirement; if no ID is given, remove all business-requirement references.

          --editor
          Open the document body in $VISUAL or $EDITOR.

       deprecate <id>
        Move a business rule to the deprecated folder. Deprecated documents are hidden from lists, search, and the web UI.

       restore <id>
       Return a deprecated business rule to its original folder. The ID is preserved.

audit

        Manage audit records (AUD-001, ...).
       
       papyrus audit create --title <title> --content <text> [--audit-type <type>]
       papyrus audit list
       papyrus audit view <id>
       papyrus audit edit <id> [--title <title>] [--content <text>] [--audit-type <type>] [--editor]
       papyrus audit deprecate <id>
       papyrus audit restore <id>
       
       create
       Create an audit record.

          --title <title>
          Audit record title.

          --content <text>
          Audit record content.

          --audit-type <type>
          Audit record type. Built-in types: security (default) and licence; additional types are configurable.

       list
       List audit records.

       view <id>
       View an audit record.

       edit <id>
       Edit an audit record.

          --title <title>
          Update the title.

          --content <text>
          Update the content.

          --audit-type <type>
          Set the audit record type.

          --editor
          Open the audit record content in $VISUAL or $EDITOR.

       deprecate <id>
       Deprecate an audit record.

       restore <id>
       Restore a deprecated audit record.

search

       Search tasks, milestones, releases, decisions, and documents in one pass.
       
        papyrus search <query> [--type <type>]... [--limit <n>]
        
        <query>
        Free-text search term or phrase. Quote phrases that contain spaces.

        --type <type>...
        Repeatable. Restrict to record types: task, milestone, release, decision, or doc. Default: all types.

       --limit <n>
       Maximum number of results.
```

### papyrus(1): STATUSES — `README.md` lines 707–756

```text
STATUSES

Tasks

        ToDo -> Initiated -> Refined -> Implemented -> Validated -> Done

        ToDo
       Initial status.

       Initiated
       Scope confirmed.

       Refined
       Ready for implementation.

       Implemented
       Work complete.

       Validated
       Reviewed and accepted.

        Done
        Completion approved by the user. This is the only terminal status.

Milestones

        active -> archived

        Archiving a milestone is separate from task workflow status.

Releases

        active -> archived

        Releases group milestones.

Decisions

       Nygard ADR statuses:

       proposed -> accepted | rejected
       accepted -> deprecated | superseded

Documents

        Business requirements, non-functional requirements, glossary terms,
        UI specifications, and business rules share this lifecycle:

        active -> deprecated

```

### papyrus(1): FILES — `README.md` lines 757–816

```text
FILES

        papyrus.config.yml
        Project configuration. Its location can be overridden with --config
        or PAPYRUS_CONFIG. See papyrus-config(5).

        backlog/tasks/
        Task records. The backlog_directory setting controls the parent path.

        backlog/milestones/
        Milestone records. The backlog_directory setting controls the parent path.

        backlog/releases/
        Release records. The backlog_directory setting controls the parent path.

        decisions/
        Decision records. The decision_directory setting controls this path.

        docs/requirements/business/
        Business requirements. Path is doc_directory plus
        document_types.req.directory.

        <doc_directory>/<document_types.req.directory>/deprecated/
        Deprecated business requirements, under the configured requirement folder.

        docs/requirements/non-functional/
        Non-functional requirements. Path is doc_directory plus
        document_types.nfr.directory.

        <doc_directory>/<document_types.nfr.directory>/deprecated/
        Deprecated non-functional requirements, under the configured requirement folder.

        docs/glossary/
        Glossary terms. Path is doc_directory plus
        document_types.glossary.directory.

        <doc_directory>/<document_types.glossary.directory>/deprecated/
        Deprecated glossary terms, under the configured glossary folder.

        docs/specifications/ui/
        UI specifications. Path is controlled by doc_directory and
        document_types.ui-spec.directory.

        <doc_directory>/<document_types.ui-spec.directory>/deprecated/
        Deprecated UI specifications, under the configured UI specifications folder.

        docs/business-rules/
        Business rules. Path is doc_directory plus
        document_types.rule.directory.

        <doc_directory>/<document_types.rule.directory>/deprecated/
        Deprecated business rules, under the configured business rules folder.

        docs/audit-records/
        Audit records. Path is doc_directory plus
        document_types.audit.directory.

        <doc_directory>/<document_types.audit.directory>/deprecated/
        Deprecated audit records, under the configured audit records folder.

```

### papyrus(1): ENVIRONMENT and EXIT STATUS — `README.md` lines 817–846

```text
ENVIRONMENT

       PAPYRUS_DIR
       Default project directory. Overridden by --dir.

       PAPYRUS_CONFIG
       Default configuration file. Overridden by --config.

       NO_COLOR
       Disable color output when set to any value.

       EDITOR, VISUAL
        Editor used by document-type edit commands and other interactive edit commands.

EXIT STATUS

       0
       Success.

       1
       Runtime error.

       2
       Usage error.

       3
       Record not found.

       4
       Validation error or conflict.
```

### papyrus-config(5): DESCRIPTION, PRECEDENCE, and KEYS — `README.md` lines 902–999

```text
DESCRIPTION

       papyrus.config.yml is a YAML file that controls project paths, ID formats, workflow statuses, document types, and the local web UI.

       The file is read from the project directory (--dir) unless --config or PAPYRUS_CONFIG names another file.

PRECEDENCE

       From highest to lowest:

       1. Command-line options.
       2. Environment variables (PAPYRUS_DIR, PAPYRUS_CONFIG).
       3. papyrus.config.yml in the project directory.
       4. Built-in defaults.

KEYS

General

       project_name
       Project name. Default: the directory name.

       task_prefix
       Task ID prefix. Default: TASK.

       milestone_prefix
       Milestone ID prefix. Default: MS.

       release_prefix
       Release ID prefix. Default: REL.

        id_padding
        Positive integer for ID zero-padding width. Default: 3 (for example TASK-001). IDs wider than the padding are not truncated.

       date_format
       Date format. Default: yyyy-mm-dd.

       editor
       Editor command. Default: $VISUAL, then $EDITOR.

Paths

       backlog_directory
       Backlog store. Default: backlog/.

       decision_directory
       Decision records. Default: decisions/, outside the backlog store.

       doc_directory
       Active documentation store. Default: docs/.

       Each document type has its own configured directory relative to this path.
       Deprecated documents are stored in a deprecated/ subfolder within
       their type directory.

Workflow

       statuses
       Ordered task statuses. Default: ["ToDo", "Initiated", "Refined", "Implemented", "Validated", "Done"].
       Use unique values; default_status must match one of them.

       default_status
       Status assigned to new tasks. Default: ToDo.

Decisions

       decision_types
       Decision types and their ID prefixes. Default:

         
         decision_types:
           adr: { prefix: ADR, enabled: true }
           bdr: { prefix: BDR, enabled: true }
         

         Additional types may be added. A disabled type is rejected by papyrus decision create and hidden from lists and search.

Documents

       document_types
       Document type keys, ID prefixes, directories relative to doc_directory,
       and enabled flags. Deprecated documents are stored in a deprecated/
       subfolder of each type directory. Default:

       
       document_types:
         req:      { prefix: REQ, directory: requirements/business/,       enabled: true }
         nfr:      { prefix: NFR, directory: requirements/non-functional/, enabled: true }
         glossary: { prefix: GLO, directory: glossary/,                    enabled: true }
         rule:     { prefix: RUL, directory: business-rules/,              enabled: true }
         ui-spec:  { prefix: UI,  directory: specifications/ui/,           enabled: true }
         audit:    { prefix: AUD, directory: audit-records/,                enabled: true }
         deploy:   { prefix: DEP, directory: architecture/deployment/,     enabled: true }
       

       The nfreq command uses the nfr type key. Additional types may
       be added. Disabled types cannot be created and are hidden from lists and search.

```
