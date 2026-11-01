# BR-006: Papyrus milestone Command Behavior

## Description

The papyrus milestone command performs the behavior stated in its Papyrus manpage entry (README.md, papyrus(1) → COMMANDS → milestone); behavior not stated there is unspecified.

## Rationale

Based on REQ-03.001 — Manage Milestones, this rule exists because the documented milestone behavior is the contract for papyrus milestone; README.md is the sole behavior source.

## Applicability

Applies to all invocations of `papyrus milestone` and the project data they manage. The excerpt below reproduces relevant manpage behavior verbatim; heading identifies README section and exact source line range.

### papyrus(1): milestone — `README.md` lines 229–278

```text
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
```
