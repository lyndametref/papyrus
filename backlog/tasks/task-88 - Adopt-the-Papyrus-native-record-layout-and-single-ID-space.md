---
id: TASK-88
title: Adopt the Papyrus-native record layout and single ID space
status: Implemented
assignee: []
created_date: '2026-11-11T20:42:00Z'
updated_date: '2026-11-12T19:26:00Z'
labels:
  - native-layout
  - identity
  - release-1.0
milestone: m-16
dependencies:
  - TASK-83
  - TASK-84
references:
  - docs/architecture/adr/ADR-0010-papyrus-native-on-disk-record-layout.md
  - >-
    docs/architecture/adr/ADR-0013-shared-record-envelope-without-a-storage-layer.md
  - docs/requirements/business/REQ-01.005.md
  - docs/requirements/non-functional/NFR-04.004.md
  - docs/specifications/papyrus-native-record-layout-and-single-id-space.md
  - docs/requirements/business/REQ-01.002.md
  - docs/requirements/business/REQ-01.007.md
  - docs/requirements/business/REQ-05.010.md
  - docs/requirements/non-functional/NFR-04.002.md
  - docs/requirements/business/REQ-03.001.md
priority: high
type: feature
ordinal: 39050
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Implement Stage 2 of ADR-0013: replace the bidirectional display/storage identity with a single Papyrus ID space and adopt the Papyrus-native on-disk layout in code. record::identity keeps only parse/format/validate/allocate of a Papyrus ID; storage prefixes, offsets, to_storage_id, to_display_id, and all their call sites are removed. Descriptors resolve directories from configuration keys (no concat! or literal path roots such as backlog/tasks, backlog/milestones, archive/tasks, or docs/decisions). Writers emit <ID>.md (decisions <ID>-<kebab-slug>.md) and never construct a Backlog.md stem. Readers tolerate <PREFIX>-<digits> optionally followed by -<slug> or  - <slug>, with case-insensitive prefix matching and optional leading zeros; Backlog-shaped stems (an all-lowercase prefix matching a non-all-lowercase configured prefix with an unpadded number, e.g. task-18, m-0) resolve to no record. The front-matter id is authoritative and a mismatch is flagged with writes refused. Allocation scans every active, archived, and deprecated location of a family with no offset; milestones start at 1; a prefix or id_padding change affects only records created afterwards. Archive and restore are file moves that keep the file name and ID and use the canonical archive directories <backlog_directory>/archive/tasks/ and <backlog_directory>/archive/milestones/; the legacy tasks/archived/ and milestones/archived/ directories are read as archived records and are never written by an archive. No read or write path relocates or renames a record: the lazy legacy relocation path in src/record/migrate.rs is deleted here, while its legacy body conversion is deleted by TASK-89. backlog/config.yml is no longer read; statuses come only from Papyrus configuration and an unrecognised status is preserved and reported. Out of scope: adding a milestone unarchive command. No milestone restore command exists and REQ-03.001 defines none; this task implements only the milestone archive move.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 record::identity exposes only Papyrus ID parse, format, validate, and allocate; storage prefixes, offsets, to_storage_id, to_display_id, the StorageInput error variant, and their call sites are gone.
- [ ] #2 Descriptor directory values are configuration keys resolved through Config and the project directory; no module hardcodes backlog/tasks, backlog/milestones, archive/tasks, docs/decisions, or a concrete decision directory such as docs/architecture/adr, and descriptor.rs contains no concat! path literals.
- [ ] #3 New task, milestone, and release records are written as <ID>.md under the configured directories, and no record-creation or native write path constructs or writes a Backlog.md stem.
- [ ] #4 Reading recognises <PREFIX>-<digits>[-slug] and <PREFIX>-<digits> - <slug> stems with case-insensitive prefix matching and optional leading zeros; Backlog-shaped stems (all-lowercase prefix matching a non-all-lowercase configured prefix with an unpadded number, e.g. task-18, m-0) resolve to no record; the front-matter id is authoritative and a stem mismatch is flagged with writes refused.
- [ ] #5 Allocation scans every active, archived (including the legacy alias locations), and deprecated location of a family with no offset, milestones start at 1, and a prefix or id_padding change affects only records created afterwards.
- [ ] #6 Archive is a file move that keeps the file name and ID into <backlog_directory>/archive/tasks/ for tasks and <backlog_directory>/archive/milestones/ for milestones; document deprecate/restore moves between the active and deprecated/ directories; decisions are not moved.
- [ ] #7 The legacy <backlog_directory>/tasks/archived/ and <backlog_directory>/milestones/archived/ locations are read as archived records, are never written by an archive, and a task restored from either is moved to the active directory unchanged.
- [ ] #8 No read or write path relocates or renames a record; the lazy legacy relocation path in src/record/migrate.rs (storage_target_path and its call sites, tests, and fixtures) is deleted, and the legacy body conversion remains for TASK-89.
- [ ] #9 backlog/config.yml is no longer read; statuses come only from Papyrus configuration, and an unrecognised status is preserved and reported.
- [ ] #10 Tests cover allocation, stem resolution including case-insensitive variants, task archive/restore round trip, milestone archive, document deprecate/restore round trip, legacy-alias restore without relocation, and that Backlog-shaped stems (task-18, m-0) resolve to no record.
- [ ] #11 No milestone unarchive command is added; the milestone lifecycle remains create, list, view, edit, archive, and delete (REQ-03.001).
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 All acceptance criteria verified by code review and tests.
- [ ] #2 cargo build succeeds with no warnings introduced by this change.
- [ ] #3 cargo test passes, including rewritten identity, stem-resolution, allocation, archive/restore, deprecate/restore, and legacy-alias tests.
- [ ] #4 A mechanical grep confirms no to_storage_id, to_display_id, storage_prefix, StorageInput, TASK_OFFSET, MILESTONE_OFFSET, backlog/config.yml read, storage_target_path, or concat! path literals in src/.
- [ ] #5 A mechanical grep confirms no module hardcodes full record directory paths (backlog/tasks, backlog/milestones, backlog/releases, archive/tasks, archive/milestones, archive/releases, docs/decisions, or a concrete decision directory such as docs/architecture/adr).
- [ ] #6 Descriptor directories resolve from configuration and pass Context::ensure_within.
- [ ] #7 Milestones start at 1; allocation scans all active, archived (including legacy aliases), and deprecated locations with no offset.
- [ ] #8 Backlog-shaped stems (task-18, m-0, and other all-lowercase unpadded forms of a non-lowercase configured prefix) resolve to no record.
- [ ] #9 Archive writes only to <backlog_directory>/archive/<family>/; legacy tasks/archived/ and milestones/archived/ are read-only aliases except for restore of a task to the active directory.
- [ ] #10 The specification docs/specifications/papyrus-native-record-layout-and-single-id-space.md and this ticket agree on the archive target, legacy-alias behavior, and the Backlog-shaped exclusion rule.
- [ ] #11 No requirement or business-rule files were modified.
<!-- DOD:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Rewrite IdSpec in src/record/identity.rs: single prefix field; remove storage_prefix, frontmatter_prefix, offset. Keep kind and padding. Update constructors (task/milestone/default) to take one prefix.

2. Replace translation methods with Papyrus-only parse/format/validate: parse_id returns the number, format_id returns <PREFIX>-<padded>, validate_input accepts only the configured Papyrus prefix. Remove to_storage_id, to_display_id, parse_display_number, parse_storage_number, to_frontmatter_id.

3. Remove module-level translation free functions and constants: TASK_OFFSET, MILESTONE_OFFSET, DEFAULT_TASK_STORAGE_PREFIX, DEFAULT_MILESTONE_STORAGE_PREFIX, to_storage_id, to_display_id, and related helpers. Keep generic Papyrus-ID helpers.

4. Update allocation: remove milestone +1 offset and m-0 base; base becomes 1 for every family. next_milestone_number scans Papyrus IDs only. allocate_from_observed_ids uses the single prefix and scans active, canonical archive, and legacy alias locations.

5. Update RecordKind defaults: milestones start at 1.

6. Descriptor (src/record/descriptor.rs): replace directory: &'static str (literal path) with a configuration key (backlog/decision/doc plus fixed family subpath). Add resolution for the active, archive (archive/tasks, archive/milestones, archive/releases), legacy alias (tasks/archived, milestones/archived), and deprecated locations as fixed subpaths of the configured root. Remove all hardcoded full paths and any concat! path construction.

7. Update src/record/store.rs (and context.rs) to resolve directories through the descriptor/config resolver; ensure_within on every path.

8. src/task.rs: resolve from configured active + canonical archive locations, plus the legacy alias read; implement tolerant stem resolution and front-matter-id authority; archive moves into <backlog_directory>/archive/tasks/; restore moves from the canonical archive or either legacy alias into the active directory, keeping the file name and ID.

9. src/milestone.rs: same resolution and stem rules; archive moves into <backlog_directory>/archive/milestones/. Do not add a milestone unarchive command (out of scope).

10. src/release.rs: verify no storage translation; uses generic helpers with the release prefix.

11. src/record/migrate.rs: remove storage-translation calls and delete the relocation path — storage_target_path and every call site, the archive-target conflict/creation branch, and the relocation tests and fixtures. Replace the concrete decision-directory literal (docs/architecture/adr) in managed_descriptor and corpus scanning with the decision descriptor resolution. Keep the legacy body conversion and lazy front-matter adoption for TASK-89; ensure_front_matter_canonical must return the same path it was given.

12. src/browser/* and src/commands.rs: replace to_display_id/to_storage_id with identity parse/format; carry Papyrus ID only; remove storage-ID fields from views/JSON.

13. src/init.rs and src/config.rs: stop reading backlog/config.yml; statuses only from Papyrus config. Preserve unrecognised status verbatim and report.

14. Tolerant stem resolution: case-insensitive prefix match with optional leading zeros and optional -slug or ' - slug' suffix; exclude Backlog-shaped stems (all-lowercase prefix matching a non-all-lowercase configured prefix with an unpadded number, e.g. task-18, m-0) so they resolve to no record; an all-lowercase configured prefix is treated as native. Add helper in identity or store.

15. Front-matter id authority: on read, compare normalised stem ID to front-matter id; on mismatch flag record invalid and refuse writes.

16. Archive/deprecate are moves that keep the file name and ID: task and milestone archive into the canonical archive directory, document deprecate/restore between active and deprecated/, decisions are not moved. Detect duplicates across active, archive, and deprecated locations and refuse writes.

17. Legacy alias handling: read tasks/archived/ and milestones/archived/ as archived; never write them from an archive; restore from an alias moves the file to the active directory.

18. Rewrite tests: single-ID identity tests; allocation tests (milestones start at 1, legacy aliases scanned); stem resolution including case-insensitive variants and Backlog-shaped exclusion; task archive/restore round trips; milestone archive; document deprecate/restore; legacy-alias restore; conflict detection.

19. Run cargo build and cargo test; fix compile errors from removed symbols.

20. Mechanical grep verification: no to_storage_id/to_display_id/storage_prefix/TASK_OFFSET/MILESTONE_OFFSET/backlog/config.yml/storage_target_path/concat! path literals and no hardcoded full record directory paths.
<!-- SECTION:PLAN:END -->
