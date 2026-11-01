# BR-009: Papyrus req Command Behavior

## Description

The papyrus req command performs the behavior stated in its Papyrus manpage entry (README.md, papyrus(1) → COMMANDS → req); behavior not stated there is unspecified.

## Rationale

Based on REQ-05.001 — Manage Business Requirements, this rule exists because the documented req behavior is the contract for papyrus req; README.md is the sole behavior source.

## Applicability

Applies to all invocations of `papyrus req` and the project data they manage. The excerpt below reproduces relevant manpage behavior verbatim; heading identifies README section and exact source line range.

### papyrus(1): req — `README.md` lines 368–435

```text
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
```
