---
id: TASK-89
title: >-
  Adopt front matter for tasks and milestones and remove the Backlog.md body
  conversion
status: Implemented
assignee: []
created_date: '2026-11-11T20:42:00Z'
updated_date: '2026-11-12T20:34:00Z'
labels:
  - front-matter
  - native-layout
  - release-1.0
milestone: m-16
dependencies:
  - TASK-88
references:
  - docs/architecture/adr/ADR-0012-papyrus-owned-front-matter-metadata.md
  - >-
    docs/architecture/adr/ADR-0013-shared-record-envelope-without-a-storage-layer.md
  - docs/requirements/business/REQ-05.010.md
  - docs/requirements/business/REQ-05.011.md
  - docs/specifications/papyrus-native-record-layout-and-single-id-space.md
  - >-
    docs/specifications/migrate-document-record-types-to-front-matter-lazily-on-first-read-and-first-write.md
  - docs/specifications/shared-record-envelope-and-front-matter-patcher.md
priority: high
type: feature
ordinal: 39060
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Implement ADR-0013 Stage 4 for tasks and milestones (ADR-0012 decisions 4, 5, 8 and 9; REQ-05.010, REQ-05.011). `src/record/migrate.rs` still converts a legacy task or milestone body into the Backlog.md layout on read/write through `convert_task`, `convert_milestone` and `is_legacy_shape`. That target shape was renounced by NFR-04.004, so adoption must be front-matter-only and the conversion must be deleted together with its tests and fixtures. Task and milestone creation also still emits a `# <ID> <title>` heading and the retired metadata sections.

This ticket:
- adds the front-matter block on the first single-record read (block only; body byte-identical) and on the first write (block plus retirement of the legacy metadata sections and the title heading);
- derives front-matter values from the legacy body (task status, assignee, milestone, labels and dependencies; milestone release) instead of writing defaults over them;
- makes new task and milestone files front-matter-only, with no title heading and no retired sections;
- reads task and milestone metadata from front matter, falling back to a legacy section only when the key is absent;
- refreshes `updated_date` once per write and never changes `created_date`;
- deletes the conversion, its action/diff machinery, and its tests and fixtures;
- fixes the document envelope so a heading-less task or milestone body parses `## Description` and later sections instead of consuming the first section as a heading.

Adoption remains per record, idempotent, and never renames or relocates a file. Bulk `list` commands do not adopt. Out of scope: the browser save path (removed by TASK-97), the Backlog.md import (TASK-92), document record types (already adopted by TASK-68), and any bulk migration command. Browser task/milestone saves may keep the old body shape until TASK-97; this ticket must not add new browser serialization.

**Assumptions**
- Assumption: the task/milestone `# <ID> <title>` heading is a legacy metadata carrier. Read adoption leaves it in place; the first write removes it (REQ-05.010 heading policy; ADR-0012 decision 5).
- Assumption: read adoption derives values from legacy sections because writing defaults would hide an existing status, assignee, milestone, labels, dependencies or release; the body itself stays byte-identical (ADR-0012 decision 9).
- Assumption: bulk `list` is not a single-record read and does not adopt, which preserves the "no bulk pass" rule (ADR-0012 decision 9).
- Assumption: an adopted task with no `## Status` uses the configured `default_status`; an adopted task with no ordinal uses `max(observed)+1`, consistent with create (ADR-0012 decision 10).
- Assumption: `created_date` for an adopted task or milestone is the file modification time, documented as approximate and never recomputed (ADR-0012 decision 9).
- Assumption: a retired section whose value cannot be parsed keeps its section and emits a warning; a duplicate section whose key is already present in front matter is removed and the front-matter value wins (ADR-0012 decision 4 and the "duplicate storage" risk note).
- Assumption: browser task/milestone saves are out of scope because ADR-0013 stage 7 (TASK-97) removes that path.
- Assumption: key completion for partially front-mattered records is removed to satisfy idempotency (ADR-0012 decision 9). This replaces the current `dropped_task_keys_are_readded_on_next_read` behaviour and its test.
- Assumption: no UI mockup is required; the change is a CLI read/write-path and file-format change.

**Open questions:** none. The behaviour above is fixed by REQ-05.010, REQ-05.011, ADR-0012 and ADR-0013; the two implementation interpretations (heading removal on write and removal of key completion) are recorded as assumptions and justified by the cited decisions.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A task or milestone that starts without front matter and is read through a single-record path (view, edit, archive, restore, delete) gains a front-matter block at byte 0 with every documented key for its type; the body after the block is byte-identical (no heading removal, no section removal, no re-rendering, no rename or relocation) and one report line `<path>\tadded-front-matter\t<keys>` is printed. Bulk list commands do not adopt.
- [ ] #2 Read adoption derives existing values instead of defaults: `id` and `title` from the `# <ID> <title>` heading (or the file stem and an empty title); task `status` from `## Status` or the configured default status; `assignee` from `## Assignee` as a YAML list; `milestone` from `## Milestone`; `labels` and `dependencies` from their list sections; milestone `release` from `## Release`; task `ordinal` from `max(observed)+1`; `documentation` and `type` empty or null; `created_date` from the file modification time; `updated_date` from one UTC clock value. Keys appear in canonical order (general keys, then type-specific keys).
- [ ] #3 The first write to a task or milestone that lacked front matter results in a file with the front-matter block, no `# <ID> <title>` heading, and no retired metadata section (task: `## Status`, `## Assignee`, `## Milestone`, `## Labels`, `## Dependencies`; milestone: `## Release`), except a section kept by criterion 4 because its value cannot be parsed. Values from the removed sections are stored in front matter, and content sections (`## Description`, `## Acceptance Criteria`, `## Comments`, and every other non-retired section) are preserved byte-for-byte.
- [ ] #4 A write to a task or milestone that already has front matter removes every retired metadata section and the title heading when present, without adding or reordering unrelated front-matter keys. A section whose value cannot be parsed for its key (a scalar section with more than one non-empty line, or a list item that is empty after stripping `- `) is left in place and a warning naming the record path and section is printed; the write still succeeds and the other sections are retired. When a key is already present in front matter, the front-matter value wins and the duplicate section is removed.
- [ ] #5 Adoption and retirement are idempotent: a record that already carries front matter and has no retired sections is returned byte-identically by a read; a write that changes no requested field and has nothing to retire performs no write and does not refresh `updated_date`; a second write after retirement does not touch the body.
- [ ] #6 Key completion is not performed: a record that already has a front-matter block but is missing one or more documented keys is not modified by adoption on read or write; missing keys read as empty or default and are left for an explicit edit.
- [ ] #7 New task and milestone files created by `papyrus task create` and `papyrus milestone create` contain no `# <ID> <title>` heading and no retired metadata sections. A task carries `id`, `title`, `created_date`, `updated_date`, `labels`, `references`, `status`, `assignee` (YAML list), `milestone`, `dependencies`, `ordinal`, `documentation` and `type`; a milestone carries the general keys plus `release`; the canonical key order is used.
- [ ] #8 Task and milestone readers take `status`, `assignee`, `milestone`, `labels`, `dependencies`, `documentation`, `type`, `ordinal` and `release` from front matter, falling back to the legacy section only when the key is absent. Title edits patch front matter only and never add or update a body heading; list, filter and sort output is unchanged for front-matter records.
- [ ] #9 `updated_date` is refreshed by every in-scope task and milestone write (create, edit, adoption, archive, restore) from one UTC clock value per write; `created_date` is written once and never changed; both are quoted `'YYYY-MM-DDTHH:MM:SSZ'` values.
- [ ] #10 The document envelope parses a heading-less task or milestone body correctly: only a line starting with `# ` is treated as the title heading, so a body that starts with `## Description` keeps that section and every later section readable and renderable.
- [ ] #11 The legacy-to-Backlog conversion is deleted: `convert_task`, `convert_milestone`, `is_legacy_shape`, the `converted-legacy-to-backlog-md` action, the conversion-only section helpers and diff machinery (`unified_diff`, `diff_lines`, `longest_common_subsequence`, `MigrationEntry.diff`), the `would-convert` dry-run action, and their tests and fixtures are gone. No read or write path re-renders a task or milestone body into a Backlog.md layout.
- [ ] #12 `migrate_corpus` and its dry run still traverse descriptor directories without writing and report `would-migrate` for a record without a block, `would-retire-legacy-sections` for a blocked record that still has retired sections, and `already-migrated` otherwise; a dry run leaves every file byte-identical.
- [ ] #13 Tests cover read adoption with body byte identity, derived adoption values, first-write retirement and heading removal, retirement on an already-blocked record, the keep-section warning, idempotent read and no-op write, partial front matter not being completed, new-record layout, front-matter reads and filters, archive/restore timestamp refresh, and removal of the conversion symbols and dry-run actions; `cargo build` and `cargo test` pass.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 All acceptance criteria are verified by code review and tests with objective evidence.
- [ ] #2 `cargo build` succeeds with no warnings introduced by this change.
- [ ] #3 `cargo test` passes, including the rewritten migration, task and milestone unit and integration tests.
- [ ] #4 A mechanical grep confirms no `convert_task`, `convert_milestone`, `is_legacy_shape`, `converted-legacy-to-backlog-md`, `would-convert`, `unified_diff`, `diff_lines`, `longest_common_subsequence` or `MigrationEntry.diff` remains in `src/`.
- [ ] #5 A review confirms no task or milestone create/edit path writes a title heading or a retired metadata section; the only such strings left in `src/` are legacy-read and retirement handling.
- [ ] #6 No new dependency is added; `yaml_serde` remains the only YAML dependency.
- [ ] #7 No requirement or business-rule files under `docs/requirements/` or `docs/business-rules/` are modified.
- [ ] #8 This ticket and `docs/specifications/papyrus-native-record-layout-and-single-id-space.md` (section 6, no shape conversion) agree on adoption, retirement and the absence of body conversion.
- [ ] #9 No UI mockup is required because the change is a CLI read/write-path and file-format change.
<!-- DOD:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. `src/record/patch.rs`: add an explicit `ClearHeading` patch operation (remove the optional heading line; no-op when absent) and apply it in `apply_patch`; update the patch tests and the operation-variant test.
2. `src/record/document.rs`: make `parse` treat only a line starting with `# ` as the title heading, so a heading-less body starting with `## Description` is parsed as sections; add round-trip tests for heading-less task/milestone bodies.
3. `src/record/migrate.rs`: add legacy-extraction helpers (`section_scalar`, `section_list`, heading id/title) and `adoption_ops` that build the full canonical key set for a task or milestone with values derived per acceptance criterion 2 (configured `default_status`, `max(observed)+1` ordinal, file mtime for `created_date`, one `utc_now` for `updated_date`).
4. `src/record/migrate.rs`: in `ensure_front_matter_internal`, keep path resolution, containment and conflict checks; for a record that already has front matter return it byte-identically (delete the missing-key completion and its `migrate_content` call); for a record without a block apply `adoption_ops`, write atomically and print the report line. Adoption never converts a body.
5. `src/record/migrate.rs`: add `retire_legacy_body_ops(doc, desc, path)` returning patch operations plus warnings. Task: `## Status`, `## Assignee`, `## Milestone`, `## Labels`, `## Dependencies`; milestone: `## Release`; both: `ClearHeading`. Map scalar and list values; clear a section when its key is present (front matter wins) or set from the section when the key is absent; keep a section and warn when its value cannot be parsed.
6. `src/record/migrate.rs`: apply the retirement operations to the post-edit document in memory so the final content is written once. Replace the task/milestone conversion branch with adoption-only. Delete `is_legacy_shape`, `convert_task`, `convert_milestone`, conversion-only section helpers, `unified_diff`, `diff_lines`, `longest_common_subsequence`, `MigrationEntry.diff`, and the `converted-legacy-to-backlog-md`/`would-convert` action paths. Update `migrate_corpus`, `process_file` and `migrate_file` for `would-migrate`, `would-retire-legacy-sections` and `already-migrated`.
7. `src/record/descriptor.rs`: reduce `TASK_SECTIONS` and `MILESTONE_SECTIONS` to the content sections that remain after retirement (task: Description, Acceptance Criteria, Comments; milestone: Description); keep retired-section names in the migration module.
8. `src/task.rs`: `body_document` stops emitting the heading and the Status/Milestone/Assignee/Labels/Dependencies sections. `create` keeps its front-matter operations. `task_from_document` reads status, assignee, milestone, labels and dependencies from front matter with a section fallback; add a front-matter list helper. `edit` removes `SetHeadingTitle` and the conditional retired-section `SetSection`/`ClearSection` operations, applies `retire_legacy_body_ops` after the edit operations, and writes once with one refreshed `updated_date`. `archive` and `restore` apply retirement plus one `updated_date` refresh before the move.
9. `src/milestone.rs`: `body_document` stops emitting the heading and `## Release`; `milestone_from_document` and `milestone_release` read `release` from front matter first; `edit` removes heading/section synchronisation, applies retirement and writes once; `archive` refreshes `updated_date` and retires sections before the move.
10. `src/record/store.rs`: verify the read path calls the adoption helper and never rewrites a body; keep the read-adoption and write-retirement responsibilities separated and atomic.
11. Rewrite the unit tests in `src/record/migrate.rs` (read adoption and body identity, derived values, write retirement and heading removal, keep-section warning, idempotency, partial front matter no-op, dry run) and keep the conflict, duplicate-ID and read-failure tests.
12. Add/update task and milestone unit tests: new-record layout and key order, assignee as a YAML list, front-matter reads and list filters, legacy edit retirement, title edit without a heading, archive/restore timestamp refresh.
13. Update `tests/migrate_records_cli.rs` to assert front-matter-only adoption and retirement, remove the Backlog.md round-trip wording and expectations, and keep Backlog-shaped stem invisibility. Adjust any other CLI test that inspects a task or milestone body after create/edit.
14. Run `cargo build` and `cargo test`; fix failures. Run the mechanical greps from the Definition of Done and confirm no requirement or business-rule file changed.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented front-matter-only adoption/retirement for tasks and milestones: ClearHeading patch op, heading-less document parsing, adoption_ops/retire_legacy_body_ops, create/edit/archive wiring, removed legacy conversion and diff machinery, updated specs and CLI tests. cargo build, cargo test (all suites), cargo clippy --all-targets -D warnings and cargo fmt --check pass.
<!-- SECTION:NOTES:END -->
