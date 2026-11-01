# BR-011: Papyrus glossary Command Behavior

## Description

The papyrus glossary command performs the behavior stated in its Papyrus manpage entry (README.md, papyrus(1) → COMMANDS → glossary); behavior not stated there is unspecified.

## Rationale

Based on REQ-05.003 — Manage Glossary Terms, this rule exists because the documented glossary behavior is the contract for papyrus glossary; README.md is the sole behavior source.

## Applicability

Applies to all invocations of `papyrus glossary` and the project data they manage. The excerpt below reproduces relevant manpage behavior verbatim; heading identifies README section and exact source line range.

### papyrus(1): glossary — `README.md` lines 492–537

```text
glossary

       Manage glossary terms (GLO-...).

       
       papyrus glossary create --term <term> --definition <text>
       papyrus glossary list
       papyrus glossary view <id>
       papyrus glossary edit <id> [--term <term>] [--definition <text>] [--editor]
       papyrus glossary deprecate <id>
       papyrus glossary restore <id>
       

       create
       Create a glossary term.

          --term <term>
          Glossary term.

          --definition <text>
          Definition of the term.

       list
       List glossary terms.

       view <id>
       View a glossary term.

       edit <id>
       Edit a glossary term.

          --term <term>
          Update the glossary term.

          --definition <text>
          Update the definition.

          --editor
          Open the document body in $VISUAL or $EDITOR.

       deprecate <id>
       Move a glossary term to the deprecated folder. Deprecated documents are hidden from lists, search, and the web UI.

       restore <id>
       Return a deprecated glossary term to its original folder. The ID is preserved.
```
