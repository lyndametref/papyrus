---
id: TASK-38
title: Manage UI mockups from the CLI
status: Done
assignee: []
created_date: '2026-11-04T19:20:00Z'
updated_date: '2026-11-06T07:23:00Z'
labels:
  - cli
  - documentation
milestone: m-4
dependencies:
  - TASK-13
references:
  - docs/requirements/business/REQ-05.008.md
  - docs/requirements/business/REQ-05.007.md
  - docs/requirements/non-functional/NFR-02.002.md
  - docs/requirements/non-functional/NFR-04.001.md
  - docs/requirements/non-functional/NFR-04.002.md
  - docs/business-rules/BR-016-configuration-keys-and-precedence.md
documentation:
  - docs/specifications/ui-mockups-management.md
priority: medium
type: feature
ordinal: 17200
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Implement the `ui-mockup` commands so contributors can create, list, view, edit, deprecate, restore, and supersede UI mockups (numbered MOCK-xxxx HTML review packages), with stable IDs and text-file persistence inside the project directory.

Scope (in):
- `papyrus ui-mockup create <title> --task <task> --spec <spec-id>... [--ref <doc>...]` (REQ-05.008).
- `papyrus ui-mockup list` (REQ-05.008).
- `papyrus ui-mockup view <id>` (REQ-05.008).
- `papyrus ui-mockup edit <id> [--title] [--task] [--spec...] [--ref...] [--status] [--superseded-by] [--editor]` (REQ-05.008).
- `papyrus ui-mockup deprecate <id>` and `restore <id>` (REQ-05.007, REQ-05.008).
- Supersession via `--superseded-by` (REQ-05.008).
- Stable IDs MOCK-<number> using configured document_types.ui-mockup prefix/id_padding; no reuse (REQ-05.008, BR-016).
- Fields: title, associated task, references, UI spec ids (REQ-05.008).
- Status vocabulary pending/approved/deprecated; transitions per spec (REQ-05.008).
- Deprecation moves files to deprecated/ and hides from lists/search/web UI (REQ-05.007, REQ-05.008).
- Restoration preserves ID and updates status field in HTML (REQ-05.008).
- UTF-8 HTML persistence inside project directory; sanitization (NFR-02.002, NFR-04.001, NFR-04.002).
- Common CLI controls/outcomes inherited from TASK-2 (REQ-01.004).

Scope (out):
- Search and browser access for mockups (TASK-39, TASK-40).
- Packaging/verification (TASK-42).
- Browser design review (TASK-41).
- Generating full visual review-package content (authored by @ui-designer).
- Deleting a mockup (no delete command).
- New configuration keys beyond document_types.ui-mockup.

Dependencies: TASK-13 (Done).
Specification: docs/specifications/ui-mockups-management.md.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 papyrus ui-mockup create <title> --task <task> --spec <spec-id>... creates a mockup and prints its generated ID.
- [ ] #2 Title, --task, and at least one --spec are required; a missing argument exits 2 (usage).
- [ ] #3 A mockup ID has the form MOCK-<number> using the configured document_types.ui-mockup prefix and id_padding, and is not reused across active and deprecated folders.
- [ ] #4 Created files are UTF-8 HTML named <ID>.html under the configured mockups directory with status pending and the title, task, spec ids, and references recorded in the HTML.
- [ ] #5 papyrus ui-mockup list shows active mockups (ID and title) in ID ascending order and excludes deprecated mockups and non-record files.
- [ ] #6 papyrus ui-mockup view <id> prints the full persisted HTML; deprecated mockups are viewable; an unknown ID exits 3.
- [ ] #7 papyrus ui-mockup edit <id> updates only supplied fields, keeps other fields unchanged, leaves ID and folder unchanged, supports --editor, and keeps deprecated mockups deprecated; an unknown ID exits 3.
- [ ] #8 Status edits follow the documented transition table; blocked transitions exit 4.
- [ ] #9 papyrus ui-mockup deprecate <id> moves the file to deprecated/; already-deprecated exits 4; unknown ID exits 3; superseded mockups are blocked.
- [ ] #10 papyrus ui-mockup restore <id> returns a deprecated mockup to the active folder with the same ID and an updated status field; already-active exits 4; unknown ID exits 3; superseded mockups are blocked.
- [ ] #11 A mockup may supersede an older mockup via --superseded-by: the superseded mockup becomes deprecated and records a back-reference to its successor; self-reference and unknown successor are rejected.
- [ ] #12 Status transitions are blocked for a superseded mockup.
- [ ] #13 Title, task, spec, and ref inputs are sanitized by trimming and rejection of empty values, control characters, and path separators; unnormalizable values exit 4 and write nothing.
- [ ] #14 All reads and writes stay within the canonicalized project directory; boundary violations exit 1; IDs are resolved by exact case-sensitive file-stem match.
- [ ] #15 When the ui-mockup document type is disabled, create and list report a validation error (exit 4).
- [ ] #16 Common CLI controls (--json, --no-color/NO_COLOR, help/version) and exit statuses 0/1/2/3/4 behave as delivered by TASK-2.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Implementation compiles and unit/integration tests pass.
- [ ] #2 Tests cover create, list, view, edit, deprecate, restore, supersede, ID generation and preservation, sanitization, project-directory boundary, and non-recursive record scan.
- [ ] #3 Feature specification docs/specifications/ui-mockups-management.md reviewed and aligned.
- [ ] #4 No direct backlog file edits; all changes via backlog CLI.
<!-- DOD:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add `ui-mockup` subcommand (clap variant UiMockup) covering create/list/view/edit/deprecate/restore, dispatched through Context like ui-spec.
2. Implement a ui_mockup module resolving document_types.ui-mockup (prefix MOCK, directory specifications/ui/mockups/), active/deprecated directories, and Context::ensure_within on every path.
3. Implement create: sanitize title/task/specs/refs, scan active and deprecated folders for highest prefix number, generate padded ID, ensure directories, write minimal HTML stub with status pending, print ID.
4. Implement list (top-level <prefix>-<digits>.html only, ID+title, sorted by ID, active only) and view (active then deprecated, full HTML, unknown -> exit 3).
5. Implement edit for --title/--task/--spec/--ref/--status/--superseded-by/--editor with field preservation and transition/supersede rules.
6. Implement deprecate/restore using file renames with already-deprecated/already-active conflicts (exit 4) and unknown IDs (exit 3), updating status field on restore.
7. Implement supersede: deprecate target, record back-reference, block further transitions.
8. Implement documented --json shapes and disabled-type rejection; add tests for lifecycle, IDs, sanitization, non-recursive scan, supersede, and project-directory boundary.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Assumptions:
- Rust 1.99.1 via rustup and Cargo available (ADR-0001); single-crate layout retained (ADR-0004).
- IDs generated by scanning active and deprecated folders; id_padding (default 3) applies; no ID reuse.
- Mockups are HTML (*.html) files; list/view/deprecate/restore do not recurse, so README.md and wireframe.css are never treated as records.
- Default directory specifications/ui/mockups/ is shared with registry and wireframe assets; record set limited to <prefix>-<digits>.html at top level.
- init already creates the mockups directory and deprecated/ subfolder; create/deprecate ensure they exist if absent.
- Title/task/spec/ref are single-line sanitized inputs; multi-line bodies available through --editor.
- Already-deprecated/already-active state conflicts exit 4; list order by ID ascending.
- Editing deprecated mockups allowed subject to transition rules.
- Mockups are not versioned by Papyrus beyond git.

Resolved open questions: none fully resolved.

Remaining open questions (see specification OQ-1..OQ-9): command/config key name, absence of a dedicated BR, HTML status field format, supersede mechanism, full transition table, restore target status, task/ref semantics, whether create requires at least one --spec, and browser mockup tracking.
<!-- SECTION:NOTES:END -->
