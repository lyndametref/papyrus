# NFR-02.002 Sanitize Input

## Description
Papyrus shall sanitize every user-supplied value (IDs, paths, titles, free text) from all CLI and browser inputs before using it, covering all command types. Input that fails the per-command-type sanitization criteria shall be rejected rather than silently cleaned.

## Rationale
Input sanitization is the specific security control requested for Papyrus.

## Verification Method
Review and test Papyrus input-handling paths for every command type to confirm all user-supplied values are sanitized before use and that input failing the per-command-type criteria is rejected (not silently cleaned).
