# NFR-02.002 Sanitize Input

## Description
Papyrus shall sanitize input before using it.

## Rationale
Input sanitization is the specific security control requested for Papyrus.

## Verification Method
Review and test Papyrus input-handling paths to confirm input is sanitized before use.

## Open Questions
The input sources in scope, the sanitization criteria, and the expected handling of input that does not meet those criteria have not been specified.
