# BR-010: Papyrus nfreq Command Behavior

## Description

The papyrus nfreq command performs the behavior stated in its Papyrus manpage entry (README.md, papyrus(1) → COMMANDS → nfreq); behavior not stated there is unspecified.

## Rationale

Based on REQ-05.002 — Manage Non-Functional Requirements, this rule exists because the documented nfreq behavior is the contract for papyrus nfreq; README.md is the sole behavior source.

## Applicability

Applies to all invocations of `papyrus nfreq` and the project data they manage. The excerpt below reproduces relevant manpage behavior verbatim; heading identifies README section and exact source line range.

### papyrus(1): nfreq — `README.md` lines 436–491

```text
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
```
