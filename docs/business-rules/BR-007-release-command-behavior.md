# BR-007: Papyrus release Command Behavior

## Description

The papyrus release command performs the behavior stated in its Papyrus manpage entry (README.md, papyrus(1) → COMMANDS → release); behavior not stated there is unspecified.

## Rationale

Based on REQ-03.002 — Manage Releases, this rule exists because the documented release behavior is the contract for papyrus release; README.md is the sole behavior source.

## Applicability

Applies to all invocations of `papyrus release` and the project data they manage. The excerpt below reproduces relevant manpage behavior verbatim; heading identifies README section and exact source line range.

### papyrus(1): release — `README.md` lines 279–305

```text
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
```
