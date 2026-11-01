# BR-013: Papyrus rule Command Behavior

## Description

The papyrus rule command performs the behavior stated in its Papyrus manpage entry (README.md, papyrus(1) → COMMANDS → rule); behavior not stated there is unspecified.

## Rationale

Based on REQ-05.005 — Manage Business Rules, this rule exists because the documented rule behavior is the contract for papyrus rule; README.md is the sole behavior source.

## Applicability

Applies to all invocations of `papyrus rule` and the project data they manage. The excerpt below reproduces relevant manpage behavior verbatim; heading identifies README section and exact source line range.

### papyrus(1): rule — `README.md` lines 584–641

```text
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
```
