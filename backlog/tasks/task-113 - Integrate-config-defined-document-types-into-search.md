---
id: TASK-113
title: Integrate config-defined document types into search
status: Done
assignee:
  - rust-dev
created_date: '2026-11-13T21:26:00Z'
updated_date: '2026-11-14T15:08:00Z'
labels:
  - rust
milestone: m-17
dependencies:
  - TASK-107
  - TASK-110
documentation:
  - docs/specifications/search-config-defined-document-types.md
  - docs/requirements/business/REQ-06.001.md
  - docs/requirements/business/REQ-05.007.md
  - docs/requirements/business/REQ-07.002.md
  - docs/requirements/business/REQ-06.004.md
  - >-
    docs/architecture/adr/ADR-0017-artifact-category-registry-and-type-descriptors.md
  - >-
    docs/architecture/adr/ADR-0022-document-lifecycle-state-and-archive-location.md
priority: high
ordinal: 41070
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Search still walks a hard-coded set of document keys (`DOC_KEYS` in `src/search.rs`), so records of a type added in configuration cannot be found even though the CLI manages them. This task replaces that list with the enabled `Document` descriptors of the startup artifact registry and keeps the documented search contract unchanged.

Scope: iterate `ArtifactRegistry::enabled(ArtifactCategory::Document)`; resolve each type active directory from `doc_directory` plus the descriptor directory; derive the candidate extension, ID parse shape, and title source from the descriptor (`content_model`, `id_shape`, prefix, shared front-matter/heading projection, HTML mockup title); index only the active directory; exclude disabled types, archived files, and records whose stored main status is archived; keep whole-text case-insensitive substring matching, ID ordering, `--limit`, text/JSON output, and exit codes from `docs/specifications/search-complete-unified.md`; thread the startup registry into `search::run`; add unit and integration tests.

Out of scope: new `--type` values or a per-document-type filter, a `papyrus <slug> search` command, ranking/scoring, new options or output formats, browser search (TASK-114), registry-model or record-storage changes.

Assumptions:
- "Per-type search" means the documented type-restricted search (`--type doc` and the four non-document values), not a per-slug filter; REQ-06.001 and BR-015 fix the five documented values.
- Matching stays whole persisted text; "same metadata projection" refers to ID and title derivation only.
- Search auto-relocates an archived-status record found in the active directory (REQ-05.007); an archived UI mockup is excluded by its HTML status without relocation.

Behavior contract: `docs/specifications/search-config-defined-document-types.md`.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Search enumerates enabled Document descriptors from the startup artifact registry instead of a hard-coded document key list; no static DOC_KEYS and no per-slug search branch remains.
- [x] #2 Directory, file extension, ID parse shape, and title source come from the descriptor (doc_directory plus relative directory, content_model, id_shape, prefix, shared front-matter/heading projection); a config-defined record is searchable by title, description, labels, and references with the unchanged whole-text case-insensitive matching, ID ordering, and output format of built-in document types.
- [x] #3 Default search and --type doc include records of every enabled config-defined document type; --type task|milestone|release|decision never return them; no new --type value, per-slug filter, or papyrus <slug> search command is added.
- [x] #4 Adding, renaming, re-pointing, or disabling a document type in configuration changes results on the next invocation without a code change or rebuild; a disabled type yields no results and exits 0.
- [x] #5 Only the active directory is indexed: archived/ and any other subdirectory are never scanned, a record whose stored main status is archived is not returned and is auto-relocated per REQ-05.007, relocation failure exits 4 with the source named, and a custom-status record stays searchable.
- [x] #6 Tests cover a user-added generic type end to end (unified and --type doc hits by title, description, labels, references; JSON shape and path), disabled exclusion, archived and legacy-subdirectory exclusion, archived-status auto-relocation, custom-status visibility, configuration changes, and an unchanged built-in req/nfr/rule/ui-mockup regression fixture.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 cargo build succeeds.
- [x] #2 cargo test and cargo clippy are clean; the new T-113 test cases pass and the existing search suite is unchanged and green.
- [x] #3 No DOC_KEYS, no module-local document-slug list, and no per-slug match arm remain in src/search.rs (grep/review check).
- [x] #4 search performs no write on the active directory except the REQ-05.007 auto-relocation; it creates, moves, or deletes nothing else.
- [x] #5 docs/specifications/search-config-defined-document-types.md is linked and matches the delivered behavior.
- [x] #6 Any deferred item or open question is recorded in the task notes with an owning task.
<!-- DOD:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Thread the startup ArtifactRegistry into search::run through cli::dispatch; do not build a second registry.
2. Delete DOC_KEYS and collect_type; add one descriptor-driven collect_document_type using descriptor.directory (doc_directory plus relative), content_model (extension and HTML title), id_shape (Grouped/Dotted/Plain parse), and prefix.
3. Read each candidate through the existing boundary check, call record::store::relocate_if_archived before matching, report a successful move on stderr, skip candidates that leave the active directory, map relocation failure to exit-4 validation, and exclude archived HTML-mockup status.
4. Keep collect_backlog_store, collect_decision_store, sorting, --limit, and text/JSON rendering unchanged.
5. Add integration tests T-113-01..13 from the specification and a unit assertion that the collected type set equals enabled Document descriptors; run cargo test and cargo clippy.
<!-- SECTION:PLAN:END -->

## Comments

<!-- COMMENTS:BEGIN -->
author: @task-refiner
created: 2026-11-14T14:51:00Z

---

Refinement 2026-11-14: corrected "per-type search" to the documented type-restricted search (--type doc plus the four non-document values). REQ-06.001 and BR-015 fix the five-value vocabulary; no per-slug filter or `papyrus <slug> search` is added. Added active-directory-only auto-relocation per REQ-05.007/ADR-0022 decision 12 and a spec at docs/specifications/search-config-defined-document-types.md.

---
<!-- COMMENTS:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Search now derives every document type from the startup ArtifactRegistry; the static DOC_KEYS list and collect_type are deleted. One descriptor-driven collector handles default and config-defined types (directory, content model, ID shape, prefix), keeps backlog/decision collectors, sorting, --limit and text/JSON output unchanged. Only active directories are indexed; archived records and archived/deprecated folders are invisible; archived-in-active records auto-relocate per REQ-05.007 with exit 4 on failure; disabled types produce no results. Evidence: cargo test 753 passed / 0 failed / 22 ignored (new tests/search_config_types_cli.rs 13/13, search_cli 52 passed); clippy -D warnings and fmt clean.
<!-- SECTION:FINAL_SUMMARY:END -->
