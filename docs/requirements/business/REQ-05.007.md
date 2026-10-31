# REQ-05.007 Manage Documentation Record Lifecycle

## User Story
As a project maintainer, I want to deprecate and restore supported documentation records so that outdated documents can be hidden without losing their identity.

## Group
REQ-05 Project Documentation Management

## Criticality
Must have

## Description
For business requirements, non-functional requirements, glossary terms, UI specifications, business rules, and audit records, the man page documents deprecate and restore commands. Deprecated documents are stored in a `deprecated/` subfolder for their document type. The man page explicitly says deprecation hides requirements, glossary terms, UI specifications, and business rules from lists, search, and the web UI; it does not spell out the same visibility effects for audit records.

## Acceptance Criteria
- Each documented document type with deprecate/restore commands can be deprecated and restored.
- Deprecated documents are stored in that document type's `deprecated/` subfolder.
- For requirements, glossary terms, UI specifications, and business rules, deprecation hides them from lists, search, and the web UI.
- Restoring a deprecated business requirement, non-functional requirement, glossary term, or UI specification preserves its ID and returns it to its original folder.

## Traceability
- README.md, `papyrus(1)` → COMMANDS → req, nfreq, glossary, ui-spec, rule, audit (lines 458–719), FILES (lines 803–844), Documents lifecycle (lines 778–784)
