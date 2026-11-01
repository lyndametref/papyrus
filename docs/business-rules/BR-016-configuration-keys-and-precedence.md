# BR-016: Papyrus Configuration Keys and Precedence

## Description

The papyrus configuration file papyrus.config.yml defines keys and precedence as stated in papyrus-config(5); behavior not stated there is unspecified.

## Rationale

Based on REQ-01.002 — Read and Edit Project Configuration and REQ-01.003 — Select Project and Configuration Context, this rule exists because configuration keys and precedence are defined by the manpage; README.md is the sole behavior source.

## Applicability

Applies to papyrus configuration files (papyrus.config.yml) and the project data they govern. The excerpt below reproduces relevant manpage behavior verbatim; heading identifies README section and exact source line range.

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
