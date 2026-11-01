# BR-014: Papyrus audit Command Behavior

## Description

The papyrus audit command performs the behavior stated in its Papyrus manpage entry (README.md, papyrus(1) → COMMANDS → audit); behavior not stated there is unspecified.

## Rationale

Based on REQ-05.006 — Manage Audit Records and REQ-05.007 — Manage Documentation Record Lifecycle, this rule exists because the documented audit behavior is the contract for papyrus audit; README.md is the sole behavior source.

## Applicability

Applies to all invocations of `papyrus audit` and the project data they manage. The excerpt below reproduces relevant manpage behavior verbatim; heading identifies README section and exact source line range.

### papyrus(1): audit — `README.md` lines 642–691

```text
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
          Audit record type. Built-in types: security (default), licence, and architecture; additional types are configurable.

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
```
