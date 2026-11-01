# BR-012: Papyrus ui-spec Command Behavior

## Description

The papyrus ui-spec command performs the behavior stated in its Papyrus manpage entry (README.md, papyrus(1) → COMMANDS → ui-spec); behavior not stated there is unspecified.

## Rationale

Based on REQ-05.004 — Manage UI Specifications, this rule exists because the documented ui-spec behavior is the contract for papyrus ui-spec; README.md is the sole behavior source.

## Applicability

Applies to all invocations of `papyrus ui-spec` and the project data they manage. The excerpt below reproduces relevant manpage behavior verbatim; heading identifies README section and exact source line range.

### papyrus(1): ui-spec — `README.md` lines 538–583

```text
ui-spec

        Manage UI specifications (UI-...).

       
       papyrus ui-spec create <title> --view <view> --description <text>
       papyrus ui-spec list
       papyrus ui-spec view <id>
       papyrus ui-spec edit <id> [--view <view>] [--description <text>] [--editor]
       papyrus ui-spec deprecate <id>
       papyrus ui-spec restore <id>
       

       create
       Create a UI specification.

          --view <view>
          Associate the specification with a view.

          --description <text>
          Description of the UI specification.

       list
       List UI specifications.

       view <id>
       View a UI specification.

       edit <id>
       Edit a UI specification.

          --view <view>
          Update the associated view.

          --description <text>
          Update the description.

          --editor
          Open the document body in $VISUAL or $EDITOR.

       deprecate <id>
        Move a UI specification to the deprecated folder. Deprecated documents are hidden from lists, search, and the web UI.

       restore <id>
        Return a deprecated UI specification to its original folder. The ID is preserved.
```
