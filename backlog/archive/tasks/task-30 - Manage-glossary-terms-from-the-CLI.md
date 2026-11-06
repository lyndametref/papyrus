---
id: TASK-30
title: Manage glossary terms from the CLI
status: Done
assignee: []
created_date: '2026-11-04T19:20:00Z'
updated_date: '2026-11-05T20:52:00Z'
labels:
  - cli
  - documentation
milestone: m-4
dependencies:
  - TASK-13
references:
  - docs/requirements/business/REQ-05.003.md
  - docs/requirements/business/REQ-05.007.md
  - docs/requirements/non-functional/NFR-02.002.md
  - docs/requirements/non-functional/NFR-04.001.md
  - docs/requirements/non-functional/NFR-04.002.md
  - docs/business-rules/BR-011-glossary-command-behavior.md
documentation:
  - docs/specifications/glossary-terms-management.md
priority: medium
type: feature
ordinal: 17000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Implement the `glossary` commands so contributors can create, list, view, edit, deprecate, and restore glossary terms, with stable IDs and text-file persistence inside the project directory.

Scope (in):
- `papyrus glossary create --term <term> --definition <text>` (BR-011, REQ-05.003).
- `papyrus glossary list` (BR-011, REQ-05.003).
- `papyrus glossary view <id>` (BR-011, REQ-05.003).
- `papyrus glossary edit <id> [--term <term>] [--definition <text>] [--editor]` (BR-011, REQ-05.003).
- `papyrus glossary deprecate <id>` and `papyrus glossary restore <id>` (BR-011, REQ-05.007).
- Stable IDs of the form <prefix>-<number> (default GLO-001) using configured document_types.glossary prefix and id_padding; IDs are not reused across active and deprecated folders (REQ-05.003, REQ-05.007, BR-011, BR-016).
- Glossary fields: term (title) and definition (REQ-05.003, BR-011).
- Deprecation moves files to the type deprecated folder and hides them from lists, search, and the web UI; restoration returns them with the same ID (REQ-05.003, REQ-05.007, BR-011).
- UTF-8 text-only persistence inside the project directory; input sanitization by normalization/escaping (NFR-02.002, NFR-04.001, NFR-04.002).
- Common CLI controls and outcomes inherited from TASK-2 (REQ-01.004).

Scope (out):
- Search and browser access for glossary terms (TASK-31, TASK-23).
- Packaging and verification of the glossary milestone (TASK-33).
- Deleting a glossary term (BR-011 defines no delete command).
- UI mockups or web UI.

Dependencies: TASK-13 (Done).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 papyrus glossary create --term <term> --definition <text> creates a glossary term and prints its generated ID.
- [ ] #2 Both --term and --definition are required; missing either exits 2 (usage).
- [ ] #3 A glossary ID has the form <prefix>-<number> (default GLO-001) using the configured document_types.glossary prefix and id_padding, and is not reused across active and deprecated folders.
- [ ] #4 papyrus glossary list shows active glossary terms (ID and term) in ID order and excludes deprecated terms.
- [ ] #5 papyrus glossary view <id> prints the full persisted text; deprecated terms are viewable; unknown IDs exit 3.
- [ ] #6 papyrus glossary edit <id> [--term <term>] [--definition <text>] [--editor] updates only the supplied fields; --editor opens the body in $VISUAL or $EDITOR; deprecated terms remain deprecated.
- [ ] #7 papyrus glossary deprecate <id> moves the file to the type's deprecated/ folder; already-deprecated exits 4; unknown ID exits 3.
- [ ] #8 papyrus glossary restore <id> returns a deprecated term to its original folder with the same ID; already-active exits 4; unknown ID exits 3.
- [ ] #9 Deprecated terms are hidden from list, search, and the web UI; restored terms retain their IDs.
- [ ] #10 All glossary files follow the repository glossary-file style (heading plus ## Definition); files are UTF-8 Markdown under the configured glossary directory or its deprecated/ subfolder; no binary data is written.
- [ ] #11 User-controlled input (term, definition) is sanitized by normalization/escaping; unnormalizable values exit 4.
- [ ] #12 All reads and writes stay within the canonicalized project directory; boundary violations exit 1.
- [ ] #13 Common CLI controls (--json, --no-color/NO_COLOR, help/version) and exit statuses 0/1/2/3/4 behave as delivered by TASK-2.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Implementation compiles and unit/integration tests pass.
- [ ] #2 Tests cover create, list, view, edit, deprecate, restore, ID generation and preservation, sanitization, and project-directory boundary.
- [ ] #3 Feature specification docs/specifications/glossary-terms-management.md reviewed and aligned.
- [ ] #4 No direct backlog file edits; all changes via backlog CLI.
<!-- DOD:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add `glossary` subcommand to the clap command enum and dispatch through Context.
2. Implement a glossary module for generated IDs, file naming, active/deprecated folders, and sanitization.
3. Implement create: generate the next ID, write the glossary Markdown file, print the ID.
4. Implement list and view for active terms, including JSON output.
5. Implement edit with field updates and --editor support.
6. Implement deprecate and restore with file moves and ID preservation.
7. Add tests for term lifecycle, IDs, persistence, sanitization, and project-directory boundary.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Assumptions:
- Rust 1.99.1 via rustup and Cargo are available (ADR-0001).
- Single-crate layout is retained; a glossary module may be added within the crate.
- IDs are generated by scanning active and deprecated folders; id_padding (default 3) applies.
- Glossary uses configured document_types.glossary prefix and directory; disabled type rejects creation.
- init already creates the glossary directory and deprecated/ subfolder.
- Glossary files follow the repository glossary-file style (heading plus ## Definition); tests verify field presence rather than byte-for-byte formatting.
- --term and --definition are single-line sanitized inputs; multi-line editing is available through --editor.
- Deprecating/restoring an already-deprecated/active term is a validation error (exit 4).
- list shows active terms in ID order.

Resolved open questions:
- OQ-1: glossary files follow the template in the specification (heading plus ## Definition).
- OQ-2: edits to deprecated terms are allowed, including field changes.

Remaining open question:
- OQ-3: whether --definition may be multi-line from the CLI is unspecified; single-line sanitization applies, with --editor for multi-line.
<!-- SECTION:NOTES:END -->
