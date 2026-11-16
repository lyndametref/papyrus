---
id: TASK-083
title: Adopt the Papyrus-native record layout and single ID space
status: Done
assignee: []
created_date: '2026-11-11T20:42:00Z'
updated_date: '2026-11-12T20:12:00Z'
labels:
  - native-layout
  - identity
  - release-1.0
milestone: MS-017
dependencies:
  - TASK-078
  - TASK-079
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
  - docs/specifications/decision-records-management.md
  - docs/specifications/release-lifecycle.md
  - docs/specifications/browser-decision-records.md
  - docs/business-rules/BR-007-release-command-behavior.md
  - docs/business-rules/BR-008-decision-command-behavior.md
  - docs/requirements/business/REQ-04.002.md
priority: high
type: feature
ordinal: 39050
---


## Description
Implement Stage 2 of ADR-0013: replace the bidirectional display/storage identity with a single Papyrus ID space and adopt the Papyrus-native on-disk layout in code for every managed family.

`record::identity` keeps only parse/format/validate/allocate of a Papyrus ID; storage prefixes, offsets, `to_storage_id`, `to_display_id`, and all their call sites are removed. Descriptors resolve directories from configuration keys (no `concat!` or literal path roots such as `backlog/tasks`, `backlog/milestones`, `archive/tasks`, or `docs/decisions`). Writers emit `<ID>.md` and decisions `<ID>-<kebab-slug>.md`, and never construct a Backlog.md stem. Readers tolerate `<PREFIX>-<digits>` optionally followed by `-<slug>` or ` - <slug>`, with case-insensitive prefix matching and optional leading zeros; document families match their full dotted ID before an optional slug; Backlog-shaped stems (an all-lowercase prefix matching a non-all-lowercase configured prefix with an unpadded number, e.g. `task-18`, `m-0`) resolve to no record. The front-matter `id` is authoritative and a mismatch is flagged with writes refused. Allocation scans every active, canonical archive, `archived/`, deprecated, and legacy-alias location of a family with no offset; milestones start at 1; a prefix or `id_padding` change affects only records created afterwards.

Family lifecycle:
- Tasks and milestones: archive to `<backlog_directory>/archive/tasks/` and `<backlog_directory>/archive/milestones/`; task restore returns to active; no milestone restore/unarchive.
- Decisions: creation writes `<ID>-<kebab-slug>.md`; all commands resolve and report the canonical ID across `<decision_directory>/` and `<decision_directory>/archived/`; the documented `papyrus decision archive` moves the file to `archived/` keeping its name; deprecation is the `## Status` line in place and never moves; no decision restore.
- Document records: `<ID>.md` in the configured type directory; read resolves the canonical ID including legacy `<ID>-<slug>.md` names; deprecate/restore moves to/from `deprecated/` keeping the file name; no archive.
- Releases: `<ID>.md` in the configured release directory, read by canonical ID; releases are never archived and no release archive path or command is added.

Legacy alias: `<backlog_directory>/tasks/archived/` and `<backlog_directory>/milestones/archived/` are read as archived records and are never written by an archive. Archiving a record already in the canonical archive or either alias fails as already archived without any write, move, or removal; only restore may move it, to the active directory, unchanged. No read or write path relocates or renames a record: the lazy legacy relocation path in `src/record/migrate.rs` is deleted here, while its legacy body conversion is deleted by TASK-89. `backlog/config.yml` is no longer read; statuses come only from Papyrus configuration and an unrecognised status is preserved and reported.

Out of scope: adding lifecycle commands (no release archive/restore, no milestone restore/unarchive, no decision restore); corpus-wide renaming of existing records; the one-way import itself (TASK-92); legacy body conversion (TASK-89); whole-folder configuration relocation (TASK-90).

## Acceptance Criteria
- [x] `record::identity` exposes only Papyrus ID parse, format, validate, and allocate; storage prefixes, offsets, `to_storage_id`, `to_display_id`, the `StorageInput` error variant, and all their call sites are gone.
- [x] Descriptor directory values are configuration keys resolved through `Config` and the project directory; no module hardcodes `backlog/tasks`, `backlog/milestones`, `backlog/releases`, `archive/tasks`, `archive/milestones`, `archive/releases`, `docs/decisions`, or a concrete decision directory such as `docs/architecture/adr`; `descriptor.rs` contains no `concat!` path literals; every resolved read and write path passes `Context::ensure_within`.
- [x] New records are written with native file names: tasks, milestones, releases, and document records as `<ID>.md`; decisions as `<ID>-<kebab-slug>.md` with the slug derived once from the creation title; no record-creation or native write path constructs or writes a Backlog.md stem, and no writer renames an existing file.
- [x] Read resolution accepts `<PREFIX>-<digits>` optionally followed by `-<slug>` or ` - <slug>`, with case-insensitive prefix matching and optional leading zeros; document families match their full dotted ID before an optional slug so `REQ-01.001` and `REQ-01.002` stay distinct; Backlog-shaped stems (all-lowercase prefix matching a non-all-lowercase configured prefix with an unpadded number, e.g. `task-18`, `m-0`) resolve to no record; the front-matter `id` is authoritative, a stem mismatch flags the record invalid and refuses writes, and every reported ID is the canonical ID, never the slugged stem.
- [x] Allocation scans every active, canonical archive, `archived/`, deprecated, and legacy-alias location of a family with no offset (tasks and milestones include the legacy alias; decisions include `archived/`; document records include `deprecated/`; releases use the active directory only); milestones start at 1; a prefix or `id_padding` change affects only records created afterwards.
- [x] Task archive moves the file to `<backlog_directory>/archive/tasks/` and milestone archive to `<backlog_directory>/archive/milestones/`, keeping the file name and ID; task restore moves a record from the canonical archive or either legacy alias to the active directory; no milestone restore or unarchive command is added.
- [x] The documented `papyrus decision archive` moves a decision file to `<decision_directory>/archived/`, keeping its slugged file name and ID; decision deprecation is the `## Status` line in place and never moves a file; no decision restore command is added.
- [x] Decision `list`, `view`, `edit`, `archive`, and supersede resolution use the canonical ID across `<decision_directory>/` and `<decision_directory>/archived/` regardless of file slug; `list` reports `ADR-0010` for `ADR-0010-<slug>.md`, never the stem; an existing `<ID>.md` decision remains readable, editable, and archivable under its existing name and is never renamed.
- [x] Document `deprecate` and `restore` move a record between its active type directory and `deprecated/`, keeping the file name and ID; document reads resolve the canonical ID including an existing legacy `<ID>-<slug>.md` name and report the canonical ID; no document archive command is added and no document is renamed.
- [x] Release `create`, `list`, `view`, and `edit` use the configured release directory and resolve records by canonical ID with native `<ID>.md` names; releases are never archived, no release archive or restore command is added, and no `<backlog_directory>/archive/releases/` location is read or written.
- [x] Legacy aliases `<backlog_directory>/tasks/archived/` and `<backlog_directory>/milestones/archived/` are read as archived records; archiving a record already in the canonical archive or either alias fails as already archived with no write, move, or removal, including no lazy front-matter write; archive never writes into an alias; only restore moves a record out, to the active directory, with the file name and ID unchanged.
- [x] Duplicate detection is keyed by canonical ID across a family's active, archive, `archived/`, legacy-alias, and deprecated locations; a duplicate flags the record invalid and refuses writes, including archive and deprecate.
- [x] No read or write path relocates or renames a record; `storage_target_path` and its call sites, tests, and fixtures are deleted; the legacy body conversion remains for TASK-89; `ensure_front_matter_canonical` returns the same path it was given.
- [x] `backlog/config.yml` is no longer read; statuses come only from Papyrus configuration, and an unrecognised status is preserved verbatim and reported.
- [x] Tests cover allocation, stem resolution including case-insensitive, slugged, and dotted-ID variants, decision slug creation and `list`/`view`/`archive` by canonical ID, document deprecate/restore with a legacy slugged name, release resolution, legacy-alias archive refusal and restore-only move, duplicate detection, and Backlog-shaped stems (`task-18`, `m-0`) resolving to no record.
- [x] No lifecycle command is added or inferred: the milestone lifecycle remains create, list, view, edit, archive, and delete (REQ-03.001), releases remain `active -> released` with no archive (BR-007), and decisions have no restore (BR-008).

## Definition of Done
- [x] #1 All acceptance criteria are verified by code review and tests.
- [x] #2 `cargo build` succeeds with no warnings introduced by this change.
- [x] #3 `cargo test` passes, including rewritten identity, stem-resolution, allocation, archive/restore, deprecate/restore, legacy-alias, decision, document, and release tests.
- [x] #4 A mechanical grep confirms no `to_storage_id`, `to_display_id`, `storage_prefix`, `StorageInput`, `TASK_OFFSET`, `MILESTONE_OFFSET`, `backlog/config.yml` read, `storage_target_path`, or `concat!` path literals in `src/`.
- [x] #5 A mechanical grep confirms no module hardcodes full record directory paths (`backlog/tasks`, `backlog/milestones`, `backlog/releases`, `archive/tasks`, `archive/milestones`, `archive/releases`, `docs/decisions`, or a concrete decision directory such as `docs/architecture/adr`).
- [x] #6 Descriptor directories resolve from configuration and every resolved path passes `Context::ensure_within`.
- [x] #7 Milestones start at 1; allocation scans all active, archive, `archived/`, deprecated, and legacy-alias locations per family with no offset.
- [x] #8 Backlog-shaped stems (`task-18`, `m-0`, and other all-lowercase unpadded forms of a non-lowercase configured prefix) resolve to no record.
- [x] #9 Archive targets are `<backlog_directory>/archive/tasks/`, `<backlog_directory>/archive/milestones/`, and `<decision_directory>/archived/`; `archive/releases` is neither read nor written; the legacy aliases are never archive destinations.
- [x] #10 Decision behavior is verified end to end: a new decision file carries a kebab slug, `ADR-0010-<slug>.md` resolves and is listed/viewed as `ADR-0010`, archive keeps the file name, an existing `<ID>.md` decision stays unrenamed, and a `## Status` deprecation does not move the file.
- [x] #11 Document behavior is verified end to end: a legacy `<ID>-<slug>.md` name resolves and reports the canonical ID, and deprecate/restore keeps the file name and ID.
- [x] #12 Legacy-alias behavior is verified: archiving a record already in `tasks/archived/` or `milestones/archived/` fails without modifying it, and restore is the only operation that moves it to the active directory.
- [x] #13 The specification `docs/specifications/papyrus-native-record-layout-and-single-id-space.md` and this ticket agree on archive targets, legacy-alias behavior, the Backlog-shaped exclusion, the decision file-name convention, deprecation versus archive, and release non-archival.
- [x] #14 No requirement or business-rule files were modified.


## Implementation Notes
1. Rewrite `IdSpec` in `src/record/identity.rs`: one prefix, padding, and kind; remove `storage_prefix`, `frontmatter_prefix`, and `offset`. Keep parse/format/validate/allocate of Papyrus IDs only.
2. Remove `to_storage_id`, `to_display_id`, `parse_display_number`, `parse_storage_number`, `to_frontmatter_id`, the `StorageInput` variant, `TASK_OFFSET`, `MILESTONE_OFFSET`, storage-prefix constants, and their call sites.
3. Allocation: minimum 1 for every family, no offsets, scan active, canonical archive, `archived/`, legacy alias, and `deprecated/` locations as applicable to the family; milestones start at 1.
4. `src/record/descriptor.rs`: replace literal directories with configuration keys; add active, canonical archive, legacy alias, and deprecated resolvers; remove every full path literal and `concat!`; apply `Context::ensure_within`.
5. `src/record/store.rs` and `src/context.rs`: resolve directories through descriptors/configuration; ensure containment on every read and write.
6. Tolerant stem helper: case-insensitive prefix, optional leading zeros, optional `-<slug>` or ` - <slug>`; document families match the full dotted ID before an optional slug; exclude Backlog-shaped stems (`task-18`, `m-0`); all-lowercase configured prefixes are native.
7. Front-matter authority: compare normalized stem ID and front-matter `id`; on mismatch mark the record invalid and refuse writes.
8. `src/task.rs`: resolve active, canonical archive, and legacy alias; archive moves to `<backlog>/archive/tasks/`; restore moves to active from either archive location; archiving a record already archived or in an alias fails before any write.
9. `src/milestone.rs`: same resolution and archive to `<backlog>/archive/milestones/`; do not add a restore/unarchive command.
10. `src/decision.rs`: derive a kebab slug from the creation title and write `<ID>-<kebab-slug>.md`; resolve by canonical ID across `<decision_directory>/` and `archived/`; make `list`, `view`, `edit`, `archive`, and supersede use the canonical ID; archive keeps the file name; `## Status` deprecation never moves; duplicate keyed by canonical ID.
11. Document modules (`src/requirement.rs`, `src/glossary.rs`, `src/rule.rs`, `src/audit.rs`, `src/tech_spec.rs`, `src/use_case.rs`, `src/test_case.rs`, `src/test_report.rs`, `src/req.rs`, `src/nfreq.rs`): resolve active and `deprecated/` from configuration; resolve canonical IDs including existing `<ID>-<slug>.md` names; report canonical IDs; deprecate/restore keep the file name; no archive.
12. `src/release.rs`: verify native `<ID>.md`, configuration-driven directory, and canonical-ID reads; add no archive path or lifecycle command.
13. `src/record/migrate.rs`: delete `storage_target_path`, its call sites, archive-target branch, tests, and fixtures; replace the concrete decision-directory literal with the resolved decision descriptor; keep the legacy body conversion for TASK-89; `ensure_front_matter_canonical` returns its input path.
14. `src/browser/*`: derive canonical IDs from stems (including slugged decision/document names), remove storage-ID fields and translation, and keep routes/JSON on Papyrus IDs.
15. `src/init.rs` and `src/config.rs`: stop reading `backlog/config.yml`; statuses only from Papyrus configuration; preserve and report an unrecognised status verbatim.
16. Duplicate detection keyed by canonical ID across a family's active, archive, `archived/`, legacy-alias, and deprecated locations; refuse archive/deprecate and other writes on a duplicate.
17. Tests: identity, stem resolution (case, padding, slug, dotted IDs, Backlog exclusion), allocation, task archive/restore, milestone archive, decision create/list/view/archive with slug, document deprecate/restore with legacy slugged name, release resolution, legacy-alias archive refusal and restore-only move, duplicates.
18. Run `cargo build` and `cargo test`; fix compile errors from removed symbols.
19. Mechanical grep for removed symbols, `backlog/config.yml`, `storage_target_path`, `concat!` path literals, and hardcoded record directories including `archive/releases` and the concrete decision directory.


## Notes
Implemented native family behaviour: decisions write <ID>-<kebab-slug>.md and resolve/list/report the canonical ID across active and archived while keeping existing <ID>.md names; document families resolve tolerant dotted/slugged stems, preserve file names on deprecate/restore, and refuse writes on stem/front-matter mismatch; releases resolve by canonical ID from the configured directory and allocate from the active directory only (archive/releases neither read nor written); task and milestone archive refuse records already in the canonical archive or a legacy alias before any write.

Verification: cargo build OK; cargo test 566 passed / 0 failed; cargo clippy --all-targets -- -D warnings OK; cargo fmt --check OK. Mechanical grep confirms no removed storage-ID symbols, no backlog/config.yml read, no storage_target_path, no concat! in descriptor.rs; hardcoded path literals remain only in test fixtures.

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Adopted the Papyrus-native record layout and single ID space across every managed family. Added shared canonical-ID/stem resolution (tolerant case, padding, slug, dotted IDs, Backlog-shaped exclusion) and wired it through decisions, document records, releases, browser indexing, search, and allocation. Decisions now create kebab-slug file names, report canonical IDs, and archive without renaming; existing unslugged decisions stay readable/editable/archivable. Document deprecate/restore preserve file names and refuse mismatched writes. Releases use native <ID>.md names, active-only allocation, and no archive path. Task/milestone archive from the canonical archive or a legacy alias now fails without any write. Added regression tests for tolerant release/decision/document reads, release active-only allocation, and legacy-alias archive refusal. All checks pass.
