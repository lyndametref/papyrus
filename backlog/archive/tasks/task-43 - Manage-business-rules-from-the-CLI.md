---
id: TASK-43
title: Manage business rules from the CLI
status: Done
assignee: []
created_date: '2026-11-04T19:20:00Z'
updated_date: '2026-11-06T07:35:00Z'
labels:
  - cli
  - documentation
milestone: m-4
dependencies:
  - TASK-13
references:
  - docs/requirements/business/REQ-05.005.md
  - docs/requirements/business/REQ-05.007.md
  - docs/requirements/non-functional/NFR-02.002.md
  - docs/requirements/non-functional/NFR-04.001.md
  - docs/requirements/non-functional/NFR-04.002.md
  - docs/business-rules/BR-013-rule-command-behavior.md
  - docs/business-rules/BR-016-configuration-keys-and-precedence.md
documentation:
  - docs/specifications/business-rules-management.md
priority: medium
type: feature
ordinal: 17300
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Implement the `rule` commands so contributors can create, list, view, edit, deprecate, and restore business rules, with stable IDs and text-file persistence inside the project directory.

Scope (in):
- `papyrus rule create <title> --description <text> --rationale <text> --applicability <text> [--requirement <id>...]` (BR-013, REQ-05.005).
- `papyrus rule list` (BR-013, REQ-05.005).
- `papyrus rule view <id>` (BR-013, REQ-05.005).
- `papyrus rule edit <id> [--title <title>] [-d <description>] [--requirement <id>...] [--no-requirement [<id>]] [--editor]` (BR-013, REQ-05.005).
- `papyrus rule deprecate <id>` and `papyrus rule restore <id>` (BR-013, REQ-05.007).
- Stable IDs of the form <prefix>-<number> (default RUL-001) using configured document_types.rule prefix and id_padding; IDs are not reused across active and deprecated folders (REQ-05.005, REQ-05.007, BR-013, BR-016).
- Business-rule fields: title, description, rationale, applicability, and zero or more business-requirement references (REQ-05.005, BR-013).
- Deprecation moves files to the type deprecated folder and hides them from lists, search, and the web UI; restoration returns them with the same ID and content (REQ-05.005, REQ-05.007, BR-013).
- UTF-8 text-only persistence inside the project directory; input sanitization by normalization/escaping (NFR-02.002, NFR-04.001, NFR-04.002).
- Common CLI controls and outcomes inherited from TASK-2 (REQ-01.004).

Scope (out):
- Search and browser access for business rules (TASK-44, TASK-25).
- Packaging and verification of the business-rules milestone (TASK-46).
- Deleting a business rule (BR-013 defines no delete command).
- UI mockups or web UI.
- New configuration keys beyond those delivered by TASK-2 (document_types.rule).

Dependencies: TASK-13 (Done).
Specification: docs/specifications/business-rules-management.md.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 papyrus rule create <title> --description <text> --rationale <text> --applicability <text> [--requirement <id>...] creates a business rule and prints the generated ID.
- [ ] #2 Title, --description, --rationale, and --applicability are all required on create; a missing argument or unknown option exits 2 (usage).
- [ ] #3 A business-rule ID has the form <prefix>-<number> (default RUL-001) using the configured document_types.rule prefix and id_padding, and is not reused across active and deprecated folders.
- [ ] #4 Created files are UTF-8 Markdown named <ID>.md in <doc_directory>/<document_types.rule.directory> (default docs/business-rules/) with content '# <ID> <title>', then '## Description', '## Rationale', '## Applicability', and, when references exist, '## Requirements', ending with a single newline.
- [ ] #5 papyrus rule list prints active business rules (ID and title) in ID ascending order and excludes deprecated ones; only top-level files matching <prefix>-<digits>.md are records.
- [ ] #6 papyrus rule view <id> prints the full persisted file text; deprecated rules remain viewable; an unknown ID exits 3.
- [ ] #7 papyrus rule edit <id> [--title <title>] [-d <description>] [--requirement <id>...] [--no-requirement [<id>]] [--editor] changes only the supplied fields, keeps other fields unchanged, and leaves the ID and folder unchanged; --editor opens the file in $VISUAL or $EDITOR; editing a deprecated rule keeps it deprecated; an unknown ID exits 3.
- [ ] #8 The rationale and applicability have no dedicated edit options; they are only changeable through --editor, which rewrites the file without renaming it.
- [ ] #9 papyrus rule deprecate <id> moves the file to the type deprecated/ folder; an already-deprecated rule exits 4 and an unknown ID exits 3.
- [ ] #10 papyrus rule restore <id> returns a deprecated rule to the active folder with the same ID and unchanged content; an already-active rule exits 4 and an unknown ID exits 3.
- [ ] #11 Deprecated rules are hidden from list, search, and the web UI; restored rules reappear in list.
- [ ] #12 Title, description, rationale, applicability, requirement references, and ID inputs are sanitized by trimming and rejection of empty values, control characters, and path separators; a value that cannot be normalized exits 4 and writes nothing.
- [ ] #13 All reads and writes stay within the canonicalized project directory; boundary violations exit 1, and IDs are resolved by exact case-sensitive file-stem match rather than path concatenation.
- [ ] #14 When the rule document type is disabled, create and list report a validation error (exit 4) and no records are created or shown.
- [ ] #15 Requirement references: --requirement adds a business-requirement ID (repeatable, de-duplicated); --no-requirement <id> removes that ID; --no-requirement without an ID removes all references.
- [ ] #16 --json output shapes are: create {id}, list [{id,title}], view {id,path,content}, and edit/deprecate/restore {status,id}; --no-color/NO_COLOR, help/version, and exit statuses 0/1/2/3/4 behave as delivered by TASK-2.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Implementation compiles and unit/integration tests pass.
- [ ] #2 Tests cover create, list, view, edit, deprecate, restore, ID generation and preservation, requirement references add/remove, sanitization, project-directory boundary, and the non-recursive record scan.
- [ ] #3 Feature specification docs/specifications/business-rules-management.md reviewed and aligned.
- [ ] #4 No direct backlog file edits; all changes via backlog CLI.
<!-- DOD:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add `rule` subcommand to the clap command enum and dispatch through Context.
2. Implement a rule module for generated IDs, file naming, active/deprecated folders, requirement references, and sanitization.
3. Implement create: generate the next ID, write the business-rule Markdown file, print the ID.
4. Implement list and view for active rules, including JSON output.
5. Implement edit with field updates, requirement add/remove, and --editor support.
6. Implement deprecate and restore with file moves and ID preservation.
7. Add tests for rule lifecycle, IDs, persistence, requirement references, sanitization, and project-directory boundary.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Assumptions:
- Rust 1.99.1 via rustup and Cargo are available (ADR-0001); single-crate layout retained (ADR-0004).
- IDs are generated by scanning active and deprecated folders; id_padding (default 3) applies; no ID reuse.
- rule uses configured document_types.rule prefix and directory; disabled type rejects creation.
- init already creates the rule directory and deprecated/ subfolder.
- Business-rule files follow the template in the specification (heading plus ## Description/Rationale/Applicability/Requirements); tests verify field presence rather than byte-for-byte formatting.
- --description, --rationale, and --applicability are single-line sanitized inputs; multi-line editing is available through --editor.
- Deprecating/restoring an already-deprecated/active rule is a validation error (exit 4).
- list shows active rules in ID order.
- Requirement references are sanitized IDs; existence is not verified by this command.

Resolved open questions:
- OQ-1: business-rule file template uses '## Description', '## Rationale', '## Applicability', and '## Requirements' (when references exist).
- OQ-2: edits to deprecated rules are allowed, including field changes.

Remaining open question:
- OQ-3: whether --description/--rationale/--applicability may be multi-line from the CLI is unspecified; single-line sanitization applies, with --editor for multi-line.
<!-- SECTION:NOTES:END -->
