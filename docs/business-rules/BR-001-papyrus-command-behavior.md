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
config
browser
task
milestone
release
decision
req
nfreq
glossary
ui-spec
rule
audit
search

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

        active -> released

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
