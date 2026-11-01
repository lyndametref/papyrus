# BR-008: Papyrus decision Command Behavior

## Description

The papyrus decision command performs the behavior stated in its Papyrus manpage entry (README.md, papyrus(1) → COMMANDS → decision); behavior not stated there is unspecified.

## Rationale

Based on REQ-04.001 — Create and Inspect Decisions and REQ-04.002 — Update, Supersede, and Archive Decisions, this rule exists because the documented decision behavior is the contract for papyrus decision; README.md is the sole behavior source.

## Applicability

Applies to all invocations of `papyrus decision` and the project data they manage. The excerpt below reproduces relevant manpage behavior verbatim; heading identifies README section and exact source line range.

### papyrus(1): decision — `README.md` lines 306–367

```text
decision

       Manage decisions (ADR-001, BDR-001, ...). Decisions may reference zero or more business requirements and zero or more non-functional requirements, and may be superseded by zero or one decision of the same type.
       
        papyrus decision create <title> [-t, --decision-type <type>] [-d <decision>] [--context <text>] [--consequences <text>]
        papyrus decision list [-t, --decision-type <type>]
       papyrus decision view <id>
       papyrus decision edit <id> [--title <title>] [-d <decision>]
                                     [--status <status>] [--context <text>] [--consequences <text>]
                                     [--superseded-by <id>]
       papyrus decision archive <id>
       
       create
       Create a decision.

           -t, --decision-type <type>
          Decision type. Built-in types: adr (default) and bdr. Additional types are configured in papyrus-config(5).

          -d, --decision <text>
          The decision which was taken.

          --context <text>
          Context of the decision.

          --consequences <text>
          Consequences of the decision.

       list
       List decisions.

           -t, --decision-type <type>
           Filter by decision type.

       view <id>
       View a decision.

       edit <id>
       Edit a decision.

          --title <title>
          Update the title.

          -d, --decision <text>
          The decision which was taken.

          --status <status>
          Set the status.

          --context <text>
          Context of the decision.

          --consequences <text>
          Consequences of the decision.

          --superseded-by <id>
           Mark this decision as superseded by at most one other decision of the same type. Cannot be combined with --status.

       archive <id>
       Archive a decision.

       Decision records follow the Nygard ADR template: Title, Status, Decision, Context, and Consequences. Statuses are proposed, accepted, rejected, deprecated, and superseded.
```
