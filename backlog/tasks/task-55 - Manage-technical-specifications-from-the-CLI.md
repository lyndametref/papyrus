---
id: TASK-55
title: Manage technical specifications from the CLI
status: Done
assignee: []
created_date: '2026-11-04T19:21:00Z'
updated_date: '2026-11-06T08:12:00Z'
labels:
  - cli
  - documentation
milestone: m-4
dependencies:
  - TASK-13
references:
  - docs/requirements/business/REQ-05.009.md
  - docs/requirements/business/REQ-05.007.md
  - docs/requirements/non-functional/NFR-02.002.md
  - docs/requirements/non-functional/NFR-04.001.md
  - docs/requirements/non-functional/NFR-04.002.md
  - docs/business-rules/BR-017-tech-spec-command-behavior.md
  - docs/business-rules/BR-016-configuration-keys-and-precedence.md
  - docs/domain-model/business-glossary/technical-specification.md
documentation:
  - docs/specifications/technical-specifications-management.md
priority: medium
type: feature
ordinal: 17500
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Implement the `tech-spec` commands so contributors can create, list, view, edit, deprecate, and restore technical specifications, with stable IDs and text-file persistence inside the project directory.

Scope (in):
- `papyrus tech-spec create <title> --description <text>` (BR-017, REQ-05.009).
- `papyrus tech-spec list` (BR-017, REQ-05.009).
- `papyrus tech-spec view <id>` (BR-017, REQ-05.009).
- `papyrus tech-spec edit <id> [--title <title>] [--description <text>] [--editor]` (BR-017, REQ-05.009).
- `papyrus tech-spec deprecate <id>` and `papyrus tech-spec restore <id>` (BR-017, REQ-05.007).
- Stable IDs of the form <prefix>-<number> (default TECH-001) using configured document_types.tech-spec prefix and id_padding; IDs are not reused across active and deprecated folders (REQ-05.009, REQ-05.007, BR-017, BR-016).
- Technical-specification fields: title and description (REQ-05.009, BR-017).
- Deprecation moves files to the type deprecated folder and hides them from lists, search, and the web UI; restoration returns them with the same ID and content (REQ-05.007, BR-017).
- UTF-8 text-only persistence inside the project directory; input sanitization by normalization/escaping (NFR-02.002, NFR-04.001, NFR-04.002).
- Common CLI controls and outcomes inherited from TASK-2 (REQ-01.004).

Scope (out):
- Search and browser access for technical specifications (TASK-56, TASK-57).
- Packaging and verification of the technical-specifications milestone (TASK-59).
- Deleting a technical specification (BR-017 defines no delete command).
- Dedicated CLI flags for referencing non-functional requirements or tasks (BR-017 defines none); such references appear as literal text in the title or description.
- UI mockups or web UI.
- New configuration keys beyond those delivered by TASK-2 (document_types.tech-spec).

Dependencies: TASK-13 (Done).
Specification: docs/specifications/technical-specifications-management.md.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 papyrus tech-spec create <title> --description <text> creates a technical specification and prints the generated ID.
- [ ] #2 Both <title> and --description are required; a missing argument or unknown option exits 2 (usage).
- [ ] #3 A technical-specification ID has the form <prefix>-<number> (default TECH-001) using the configured document_types.tech-spec prefix and id_padding, and is not reused across active and deprecated folders.
- [ ] #4 Created files are UTF-8 Markdown named <ID>.md in <doc_directory>/<document_types.tech-spec.directory> (default docs/technical/) with content '# <ID> <title>' then '## Description', ending with a single newline.
- [ ] #5 papyrus tech-spec list shows active technical specifications (ID and title) in ID ascending order and excludes deprecated ones; only top-level files matching <prefix>-<digits>.md are records.
- [ ] #6 papyrus tech-spec view <id> prints the full persisted file text; deprecated specifications remain viewable; an unknown ID exits 3.
- [ ] #7 papyrus tech-spec edit <id> [--title <title>] [--description <text>] [--editor] changes only the supplied fields, keeps other fields unchanged, and leaves the ID and folder unchanged; --editor opens the file in $VISUAL or $EDITOR; editing a deprecated specification keeps it deprecated; an unknown ID exits 3.
- [ ] #8 papyrus tech-spec deprecate <id> moves the file to the type deprecated/ folder; an already-deprecated specification exits 4 and an unknown ID exits 3.
- [ ] #9 papyrus tech-spec restore <id> returns a deprecated specification to the active folder with the same ID and unchanged content; an already-active specification exits 4 and an unknown ID exits 3.
- [ ] #10 Deprecated specifications are hidden from list, search, and the web UI; restored specifications reappear in list.
- [ ] #11 Title, description, and ID inputs are sanitized by trimming and rejection of empty values, control characters, and path separators; a value that cannot be normalized exits 4 and writes nothing.
- [ ] #12 All reads and writes stay within the canonicalized project directory; boundary violations exit 1, and IDs are resolved by exact case-sensitive file-stem match rather than path concatenation.
- [ ] #13 When the tech-spec document type is disabled, create and list report a validation error (exit 4) and no records are created or shown.
- [ ] #14 --json output shapes are: create {id}, list [{id,title}], view {id,path,content}, and edit/deprecate/restore {status,id}; --no-color/NO_COLOR, help/version, and exit statuses 0/1/2/3/4 behave as delivered by TASK-2.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Implementation compiles and unit/integration tests pass.
- [ ] #2 Tests cover create, list, view, edit, deprecate, restore, ID generation and preservation, sanitization, project-directory boundary, and the non-recursive record scan.
- [ ] #3 Feature specification docs/specifications/technical-specifications-management.md reviewed and aligned.
- [ ] #4 No direct backlog file edits; all changes via backlog CLI.
<!-- DOD:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add `tech-spec` subcommand to the clap command enum and dispatch through Context.
2. Implement a tech_spec module for generated IDs, file naming, active/deprecated folders, and sanitization.
3. Implement create: generate the next ID, write the technical-specification Markdown file, print the ID.
4. Implement list and view for active specifications, including JSON output.
5. Implement edit with --title/--description updates and --editor support.
6. Implement deprecate and restore with file moves and ID preservation.
7. Add tests for specification lifecycle, IDs, persistence, sanitization, and project-directory boundary.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Assumptions:
- Rust 1.99.1 via rustup and Cargo are available (ADR-0001); single-crate layout retained (ADR-0004).
- IDs are generated by scanning active and deprecated folders; id_padding (default 3) applies; no ID reuse.
- tech-spec uses configured document_types.tech-spec prefix and directory; disabled type rejects creation.
- init already creates the tech-spec directory and deprecated/ subfolder.
- Technical-specification files follow the template in the specification (heading plus ## Description); tests verify field presence rather than byte-for-byte formatting.
- --title and --description are single-line sanitized inputs; multi-line editing is available through --editor.
- Deprecating/restoring an already-deprecated/active specification is a validation error (exit 4).
- list shows active specifications in ID order.

Resolved open questions:
- OQ-1: technical-specification file template uses '## Description' (BR-017 fixes no section names).
- OQ-2: edits to deprecated specifications are allowed, including field changes.

Remaining open question:
- OQ-3: whether --description may be multi-line from the CLI is unspecified; single-line sanitization applies, with --editor for multi-line.
<!-- SECTION:NOTES:END -->
