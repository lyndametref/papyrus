---
id: TASK-34
title: Manage UI specifications from the CLI
status: Done
assignee: []
created_date: '2026-11-04T19:20:00Z'
updated_date: '2026-11-06T07:10:00Z'
labels:
  - cli
  - documentation
milestone: m-4
dependencies:
  - TASK-13
references:
  - docs/requirements/business/REQ-05.004.md
  - docs/requirements/business/REQ-05.007.md
  - docs/requirements/non-functional/NFR-02.002.md
  - docs/requirements/non-functional/NFR-04.001.md
  - docs/requirements/non-functional/NFR-04.002.md
  - docs/business-rules/BR-012-ui-spec-command-behavior.md
  - docs/business-rules/BR-016-configuration-keys-and-precedence.md
documentation:
  - docs/specifications/ui-specifications-management.md
priority: medium
type: feature
ordinal: 17100
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Implement the `ui-spec` commands so contributors can create, list, view, edit, deprecate, and restore UI specifications (title, associated view, description), with stable IDs and text-file persistence inside the project directory.

Scope (in):
- `papyrus ui-spec create <title> --view <view> --description <text>` (BR-012, REQ-05.004).
- `papyrus ui-spec list` (BR-012, REQ-05.004).
- `papyrus ui-spec view <id>` (BR-012, REQ-05.004).
- `papyrus ui-spec edit <id> [--view <view>] [--description <text>] [--editor]` (BR-012, REQ-05.004).
- `papyrus ui-spec deprecate <id>` and `papyrus ui-spec restore <id>` (BR-012, REQ-05.007).
- Stable IDs of the form <prefix>-<number> (default UI-001) using configured document_types.ui-spec prefix and id_padding; IDs are not reused across active and deprecated folders (REQ-05.004, REQ-05.007, BR-012, BR-016).
- UI-specification fields: title, view, description; file template `# <ID> <title>`, `## View`, `## Description` (REQ-05.004, BR-012).
- Deprecation moves files to the type deprecated folder and hides them from lists, search, and the web UI; restoration returns them with the same ID and content (REQ-05.007, BR-012).
- UTF-8 text-only persistence inside the project directory; input sanitization by normalization/escaping (NFR-02.002, NFR-04.001, NFR-04.002).
- Common CLI controls and outcomes inherited from TASK-2 (REQ-01.004).

Scope (out):
- Unified search for UI specifications (TASK-35).
- Browser listing, viewing, and editing of UI specifications and the browser design review (TASK-24, TASK-36); mockup authoring or linking MOCK files to records.
- Packaging and verification of the UI-specifications milestone (TASK-37).
- Deleting a UI specification (BR-012 defines no delete command).
- Changing the default ui-spec directory (`specifications/ui/`) or adding configuration keys beyond those delivered by TASK-2.

Dependencies: TASK-13 (Done).
Specification: docs/specifications/ui-specifications-management.md.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 papyrus ui-spec create <title> --view <view> --description <text> creates a UI specification and prints the generated ID.
- [ ] #2 Title, --view, and --description are all required on create; a missing argument or an unknown option (for example --title) exits 2 (usage).
- [ ] #3 IDs have the form <prefix>-<number> using the configured document_types.ui-spec prefix (default UI) and id_padding (default 3, for example UI-001); the number is one greater than the highest number found in the active and deprecated folders and IDs are never reused.
- [ ] #4 Created files are UTF-8 Markdown named <ID>.md in <doc_directory>/<document_types.ui-spec.directory> (default docs/specifications/ui/) with content '# <ID> <title>', then '## View', then '## Description', ending with a single newline.
- [ ] #5 papyrus ui-spec list prints active UI specifications (ID and title) in ID ascending order and excludes deprecated ones; only top-level files matching <prefix>-<digits>.md are records, so subfolders (for example mockups/) and non-matching files are ignored.
- [ ] #6 papyrus ui-spec view <id> prints the full persisted file text; deprecated specifications remain viewable; an unknown ID exits 3.
- [ ] #7 papyrus ui-spec edit <id> [--view <view>] [--description <text>] [--editor] changes only the supplied fields, keeps other fields unchanged, and leaves the ID and folder unchanged; --editor opens the file in $VISUAL or $EDITOR; editing a deprecated specification keeps it deprecated; an unknown ID exits 3.
- [ ] #8 The title has no dedicated edit option; it is only changeable through --editor, which rewrites the file without renaming it.
- [ ] #9 papyrus ui-spec deprecate <id> moves the file to the type deprecated/ folder; an already-deprecated specification exits 4 and an unknown ID exits 3.
- [ ] #10 papyrus ui-spec restore <id> returns a deprecated specification to the active folder with the same ID and unchanged content; an already-active specification exits 4 and an unknown ID exits 3.
- [ ] #11 Deprecated specifications are hidden from list (and therefore from later search and web UI integrations); restored specifications reappear in list.
- [ ] #12 Title, view, description, and ID inputs are sanitized by trimming and rejection of empty values, control characters, and path separators; a value that cannot be normalized exits 4 and writes nothing.
- [ ] #13 All reads and writes stay within the canonicalized project directory; boundary violations exit 1, and IDs are resolved by exact case-sensitive file-stem match rather than path concatenation.
- [ ] #14 When the ui-spec document type is disabled, create and list report a validation error (exit 4) and no records are created or shown.
- [ ] #15 --json output shapes are: create {id}, list [{id,title}], view {id,path,content}, and edit/deprecate/restore {status,id}; --no-color/NO_COLOR, help/version, and exit statuses 0/1/2/3/4 behave as delivered by TASK-2.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Implementation compiles and unit/integration tests pass.
- [ ] #2 Tests cover create, list, view, edit, deprecate, restore, ID generation and preservation, sanitization, project-directory boundary, and the non-recursive record scan.
- [ ] #3 Feature specification docs/specifications/ui-specifications-management.md reviewed and aligned.
- [ ] #4 No direct backlog file edits; all changes via backlog CLI.
<!-- DOD:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add a `ui-spec` subcommand (clap variant UiSpec) covering create/list/view/edit/deprecate/restore, dispatched through Context like the glossary command.
2. Implement a ui_spec module resolving document_types.ui-spec (prefix UI, directory specifications/ui/), active and deprecated directories, and Context::ensure_within on every path.
3. Implement create: sanitize title/view/description, scan active and deprecated folders for the highest prefix number, generate the padded ID, ensure directories, write '# <ID> <title>' + '## View' + '## Description', print the ID.
4. Implement list (top-level <prefix>-<digits>.md only, ID+title, sorted by ID, active only) and view (active then deprecated, full text, unknown -> exit 3).
5. Implement edit for --view/--description, keeping unsupplied fields, leaving ID/folder unchanged, and --editor through $VISUAL/$EDITOR.
6. Implement deprecate/restore using file renames with already-deprecated/already-active conflicts (exit 4) and unknown IDs (exit 3).
7. Implement the documented --json shapes and disabled-type rejection; add tests for lifecycle, IDs, sanitization, non-recursive scan, and project-directory boundary.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Specification: docs/specifications/ui-specifications-management.md (Refined).

Assumptions:
- Rust 1.99.1 via rustup and Cargo are available (ADR-0001); single-crate layout retained (ADR-0004).
- IDs are generated by scanning active and deprecated folders; id_padding (default 3) applies; no ID reuse.
- BR-012 fixes no section names; the file template uses '# <ID> <title>', '## View', '## Description', following the TASK-30 pattern. Tests check field presence, not byte-for-byte formatting.
- list/view/deprecate/restore do not recurse; only top-level <prefix>-<digits>.md files are records, so specifications/ui/mockups/ and UI description documents are never treated as records.
- The default ui-spec directory specifications/ui/ is shared in this repository with UI description documents and mockups/; relocation is a configuration decision, out of scope.
- init already creates the ui-spec directory and deprecated/ subfolder; create and deprecate ensure they exist if absent.
- Title, --view, and --description are single-line sanitized inputs; multi-line bodies are available through --editor.
- Already-deprecated/already-active state conflicts exit 4; list order is by ID ascending; edits to deprecated specifications are allowed.
- Disabled ui-spec type: create and list exit 4 (consistent with TASK-30), so no records are created or shown.

Resolved open questions:
- OQ-1: create takes <title> positionally and requires --view and --description (BR-012 synopsis, REQ-05.004) -> all three required.
- OQ-2: edit exposes only --view, --description, --editor; there is no --title option, so the title is editable only through --editor (BR-012).
- OQ-3: file body layout unspecified by BR-012 -> '## View' then '## Description' (OQ-1 of the specification, non-blocking).
- OQ-4: linking MOCK ids to UI specifications is unspecified and out of scope; browser work belongs to TASK-24/TASK-36 (OQ-2 of the specification, non-blocking).
<!-- SECTION:NOTES:END -->
