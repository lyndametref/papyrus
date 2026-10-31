# REQ-05.003 Manage Glossary Terms

## User Story
As a project contributor, I want to maintain glossary terms and definitions so that project terminology can be looked up consistently.

## Group
REQ-05 Project Documentation Management

## Criticality
Must have

## Description
The user can create glossary terms with a term and definition, list and view terms, edit either field, and deprecate or restore terms. Deprecated terms are hidden from lists, search, and the web UI; restored terms retain their IDs.

## Acceptance Criteria
- A glossary term can be created with the required term and definition.
- Terms can be listed, viewed, and edited for term and definition.
- Deprecation hides a term from lists, search, and the web UI.
- Restoration returns a term to its original folder with its ID preserved.

## Traceability
- README.md, `papyrus(1)` → COMMANDS → glossary and Documents lifecycle (lines 520–564, 778–784)
