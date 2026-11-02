---
id: TASK-3
title: Manage business and non-functional requirements from the CLI
status: Done
assignee: []
created_date: '2026-10-31T22:25:00Z'
updated_date: '2026-11-02T19:07:00Z'
labels: []
milestone: m-0
dependencies:
  - TASK-2
references:
  - docs/requirements/business/REQ-05.md
  - docs/requirements/business/REQ-05.001.md
  - docs/requirements/business/REQ-05.002.md
  - docs/requirements/business/REQ-05.007.md
  - docs/requirements/non-functional/NFR-02.002.md
  - docs/requirements/non-functional/NFR-04.001.md
  - docs/requirements/non-functional/NFR-04.002.md
  - docs/business-rules/BR-009-req-command-behavior.md
  - docs/business-rules/BR-010-nfreq-command-behavior.md
  - docs/specifications/business-and-non-functional-requirements-management.md
type: feature
ordinal: 3000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Implement the `req` and `nfreq` commands so contributors can create, list, view, edit, deprecate, and restore business requirements and non-functional requirements, including their required groups, with stable IDs and text-file persistence inside the project directory.

Scope (in):
- `papyrus req create <title> [--title <title>] [--story <text>] [-d <description>] --group <id>` and `papyrus req create-group --title <title> [--story <text>] [-d <description>]` (BR-009, REQ-05.001).
- `papyrus req list`, `view <id>`, `edit <id> [--title <title>] [--story <text>] [-d <description>] [--editor]`, `deprecate <id>`, `restore <id>` (BR-009).
- `papyrus nfreq create <title> [-d <description>] --group <id>` and `papyrus nfreq create-group --title <title> [-d <description>]` (BR-010, REQ-05.002).
- `papyrus nfreq list`, `view <id>`, `edit <id> [--title <title>] [-d <description>] [--editor]`, `deprecate <id>`, `restore <id>` (BR-010).
- Requirement-to-group cardinality 1..1: requirements are created only with a required group, belong to exactly one group, and groups are type-specific (REQ-05.001, REQ-05.002).
- Stable IDs preserved across deprecation and restoration; group IDs and per-group sequence IDs (REQ-05.001, REQ-05.002, REQ-05.007).
- Deprecation moves files to the type's `deprecated/` folder and hides them from lists, search, and the web UI; restoration returns them with the same ID (REQ-05.001, REQ-05.002, REQ-05.007).
- UTF-8 text-only persistence inside the project directory; input sanitization by normalization/escaping (NFR-02.002, NFR-04.001, NFR-04.002).
- Common CLI controls and outcomes inherited from TASK-2 (REQ-01.004).

Scope (out):
- Search and browser access for requirements (TASK-5, TASK-6).
- Other record-management commands (TASK-8 onward).
- Requirement references from tasks/decisions (later tickets).
- UI mockups or web UI.

Dependencies: TASK-2 (Done).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 papyrus req create-group --title <title> [--story <text>] [-d <description>] creates a business-requirement group and prints its generated ID.
- [ ] #2 papyrus nfreq create-group --title <title> [-d <description>] creates a non-functional-requirement group and prints its generated ID.
- [ ] #3 A group ID has the form <prefix>-<number> using the configured document_types prefix and id_padding, and is not reused.
- [ ] #4 papyrus req create <title> [--title <title>] [--story <text>] [-d <description>] --group <id> creates a business requirement only when --group references an existing same-type group.
- [ ] #5 papyrus nfreq create <title> [-d <description>] --group <id> creates a non-functional requirement only when --group references an existing same-type group.
- [ ] #6 A requirement ID has the form <group-id>.<sequence> using id_padding for the sequence, and is unique within its group.
- [ ] #7 Creating a requirement without --group, or with a missing or mismatched-type group, fails with the appropriate exit code (2 for usage, 3 for not found, 4 for validation).
- [ ] #8 req list and nfreq list show active groups and requirements (ID and title) in ID order and exclude deprecated records.
- [ ] #9 req view <id> and nfreq view <id> print the full persisted text; unknown IDs exit 3.
- [ ] #10 req edit <id> and nfreq edit <id> update only the supplied fields; --editor opens the body in $VISUAL or $EDITOR; group membership is immutable.
- [ ] #11 req deprecate <id> and nfreq deprecate <id> move the file to the type's deprecated/ folder.
- [ ] #12 req restore <id> and nfreq restore <id> return a deprecated requirement to its original folder with the same ID.
- [ ] #13 Deprecated requirements are hidden from list, search, and the web UI; restored requirements retain their IDs.
- [ ] #14 All requirement and group files follow the repository requirement-file style, excluding Criticality and unsupported fields; files are UTF-8 Markdown under the configured type directory or its deprecated/ subfolder; no binary data is written.
- [ ] #15 User-controlled input (title, story, description, group ID) is sanitized by normalization/escaping; unnormalizable values exit 4.
- [ ] #16 All reads and writes stay within the canonicalized project directory; boundary violations exit 1.
- [ ] #17 Common CLI controls (--json, --no-color/NO_COLOR, help/version) and exit statuses 0/1/2/3/4 behave as delivered by TASK-2.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 All acceptance criteria are verified by automated tests or documented manual checks.
- [ ] #2 cargo build --release and cargo test pass with the pinned Rust 1.99.1 toolchain.
- [ ] #3 cargo clippy --all-targets -- -D warnings and cargo fmt --check pass.
- [ ] #4 req and nfreq create/list/view/edit/deprecate/restore behaviors are verified, including required groups and ID preservation.
- [ ] #5 UTF-8 text-only persistence and project-directory boundary (including symlinks) are verified.
- [ ] #6 Input sanitization (normalization/escaping) is covered by tests for representative inputs.
- [ ] #7 Build, packaging, install, and smoke-test steps remain documented and pass.
- [ ] #8 Acceptance criteria reviewed and signed off.
<!-- DOD:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add `req` and `nfreq` subcommands to the clap command enum and dispatch through Context.
2. Implement a shared document-store helper for generated IDs, file naming, active/deprecated folders, and sanitization.
3. Implement create-group for req and nfreq: generate the next group ID, write the group Markdown file, print the ID.
4. Implement create: validate --group, generate the next per-group sequence ID, record group reference, write the requirement Markdown file, print the ID.
5. Implement list and view for active requirements, including JSON output.
6. Implement edit with field updates and --editor support.
7. Implement deprecate and restore with file moves and ID preservation.
8. Add tests for group/requirement lifecycle, IDs, persistence, sanitization, and project-directory boundary.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Assumptions:
- Rust 1.99.1 via rustup and Cargo are available (ADR-0001).
- Single-crate layout is retained; req/nfreq modules may be added within the crate.
- IDs are generated by scanning active and deprecated folders; id_padding (default 3) applies to group numbers and per-group sequences.
- Group IDs and requirement IDs use configured document_types prefixes/directories; disabled types reject creation.
- --editor resolves $VISUAL then $EDITOR via effective configuration.
- Requirement files follow the repository requirement-file style, excluding Criticality and fields not supported by documented command options.
- Deprecating/restoring an already-deprecated/active requirement is a validation error (exit 4).
- Deprecating a group does not cascade to its requirements; creation against a deprecated group is rejected.
- list shows groups and requirements in ID order.
- Group membership is immutable after creation.

Resolved open questions:
- OQ-1: generated files follow repository requirement-file style, excluding Criticality and unsupported fields.
- OQ-2: req list and nfreq list display both groups and requirements.
- OQ-3: group membership is immutable; edit does not support --group.

Remaining open question:
- OQ-4: child sequence padding convention is not explicitly documented; id_padding is applied.
<!-- SECTION:NOTES:END -->
