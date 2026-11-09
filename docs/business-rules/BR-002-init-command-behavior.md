# BR-002: Papyrus init Command Behavior

## Description

The papyrus init command performs the behavior stated in its Papyrus manpage entry (README.md, papyrus(1) → COMMANDS → init); behavior not stated there is unspecified.

## Rationale

Based on REQ-01.001 — Initialize a Papyrus Project, this rule exists because the documented init behavior is the contract for papyrus init; README.md is the sole behavior source.

## Applicability

Applies to all invocations of `papyrus init` and the project data they manage. The excerpt below reproduces relevant manpage behavior verbatim; heading identifies README section and exact source line range.

### papyrus(1): init — `README.md` (init section)

```text
init

       Initialize a project in the directory where papyrus init is run. If --dir <path> is given, initialize in that directory instead; the directory is created if it does not exist. Initialization creates the backlog store, the documentation structure, and papyrus.config.yml.

       papyrus init <name> [--no-git] [--force]

       <name> is the project name stored in the configuration.

       In an interactive terminal, papyrus init walks through all available artifacts (task, milestone, release, decision types, and document types), lets you choose which to activate, and prompts for the ID prefix and storage folder of each activated artifact. Core artifacts (task, milestone, release) are always activated. Non-interactive or CI environments (no TTY) use default selections and defaults without prompting. Invalid prefixes or folders are rejected and re-prompted with hints naming the default equivalent value.

       --no-git
       Do not initialize a Git repository.

       --force
       Allow initialization in a non-empty directory. Existing files are never overwritten.
```
