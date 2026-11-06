---
id: TASK-61
title: Add missing ui-mockup document type and FILES entries to README.md
status: Done
assignee: []
created_date: '2026-11-06T08:44:00Z'
updated_date: '2026-11-06T09:27:00Z'
labels:
  - documentation
milestone: m-4
dependencies:
  - TASK-38
references:
  - docs/specifications/ui-mockups-management.md
  - docs/business-rules/BR-016-configuration-keys-and-precedence.md
modified_files:
  - README.md
priority: medium
type: chore
ordinal: 17600
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Update README.md (papyrus(1) manpage) to document the already-implemented `ui-mockup` command and its `ui-mockup` document type. These entries are missing even though TASK-38 and BR-016 define prefix MOCK and directory specifications/ui/mockups/.

Scope (in):
- Add a `ui-mockup` command section to README.md (papyrus(1)), placed after the `ui-spec` section and before `tech-spec`, documenting:
  - Synopsis:
    - papyrus ui-mockup create <title> --task <task> --spec <spec-id>... [--ref <doc>...]
    - papyrus ui-mockup list
    - papyrus ui-mockup view <id>
    - papyrus ui-mockup edit <id> [--title <title>] [--task <task>] [--spec <spec-id>...] [--ref <doc>...] [--status <status>] [--superseded-by <id>] [--editor]
    - papyrus ui-mockup deprecate <id>
    - papyrus ui-mockup restore <id>
  - Option descriptions for --task, --spec, --ref, --title, --status, --superseded-by, and --editor, consistent with docs/specifications/ui-mockups-management.md and the ui-spec section style.
- In both document_types default blocks (explanatory block and EXAMPLE), add immediately after the `ui-spec` line:
  ui-mockup: { prefix: MOCK, directory: specifications/ui/mockups/, enabled: true }
- In the FILES section (papyrus(1)), add immediately after the ui-spec active and deprecated entries:
  - Active UI mockups entry for docs/specifications/ui/mockups/ (path is doc_directory plus document_types.ui-mockup.directory).
  - Deprecated UI mockups entry for <doc_directory>/<document_types.ui-mockup.directory>/deprecated/.
- Preserve existing formatting and HTML structure.

Scope (out):
- No source code, tests, or other documentation changes.
- No changes to generated indexes.
- No change to command behavior; documentation only.

Assumptions:
- README.md is the canonical manpage source; there is no separate generated copy.
- Command syntax and field semantics are as implemented in TASK-38 and documented in docs/specifications/ui-mockups-management.md.
- The ui-mockup entry is placed after ui-spec in each document_types block and FILES section to keep related UI entries grouped.
- Only README.md is modified.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 README.md documents the papyrus ui-mockup command (create/list/view/edit/deprecate/restore) consistently with the implemented CLI and the ui-spec section.
- [ ] #2 Both default document_types blocks (explanatory and EXAMPLE) include: ui-mockup: { prefix: MOCK, directory: specifications/ui/mockups/, enabled: true }.
- [ ] #3 The FILES section lists the active and deprecated paths for UI mockups under specifications/ui/mockups/.
- [ ] #4 Only README.md is modified; no source code, tests, or other documentation files change.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 README.md updated with the ui-mockup command section and all required document_types and FILES entries.
- [ ] #2 Changes reviewed for consistency with the implemented CLI and docs/specifications/ui-mockups-management.md.
- [ ] #3 Modified-files check confirms only README.md was changed.
<!-- DOD:END -->
