# BR-015: Papyrus search Command Behavior

## Description

The papyrus search command performs the behavior stated in its Papyrus manpage entry (README.md, papyrus(1) → COMMANDS → search); behavior not stated there is unspecified.

## Rationale

Based on REQ-06.001 — Search Project Records, this rule exists because the documented search behavior is the contract for papyrus search; README.md is the sole behavior source.

## Applicability

Applies to all invocations of `papyrus search` and the project data they manage. The excerpt below reproduces relevant manpage behavior verbatim; heading identifies README section and exact source line range.

### papyrus(1): search — `README.md` lines 692–706

```text
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
