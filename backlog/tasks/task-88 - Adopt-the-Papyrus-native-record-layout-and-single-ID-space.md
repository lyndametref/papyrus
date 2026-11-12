---
id: TASK-88
title: Adopt the Papyrus-native record layout and single ID space
status: Implemented
assignee: []
created_date: '2026-11-11T20:42:00Z'
updated_date: '2026-11-12T17:19:00Z'
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
priority: high
type: feature
ordinal: 39050
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Implement Stage 2 of ADR-0013: replace the bidirectional display/storage identity with a single Papyrus ID space and adopt the Papyrus-native on-disk layout in code. record::identity keeps only parse/format/validate/allocate of a Papyrus ID; storage prefixes, offsets, to_storage_id, to_display_id, and all their call sites are removed. Descriptors resolve directories from configuration keys (no concat! or literal path roots such as backlog/tasks, backlog/milestones, archive/tasks, or docs/decisions). Writers emit <ID>.md (decisions <ID>-<kebab-slug>.md) and never construct a Backlog.md stem. Readers tolerate <PREFIX>-<digits> optionally followed by -<slug> or  - <slug>, with case-insensitive prefix matching and optional leading zeros; Backlog-shaped stems (an all-lowercase prefix matching a non-all-lowercase configured prefix with an unpadded number, e.g. task-18, m-0) resolve to no record. The front-matter id is authoritative and a mismatch is flagged with writes refused. Allocation scans every active, archived, and deprecated location of a family with no offset; milestones start at 1; a prefix or id_padding change affects only records created afterwards. Archive, restore, deprecate, and restore are moves that keep the file name and ID; legacy tasks/archived/ and milestones/archived/ are read as archived records. backlog/config.yml is no longer read; statuses come only from Papyrus configuration and an unrecognised status is preserved and reported.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 record::identity exposes only Papyrus ID parse, format, validate, and allocate; storage prefixes, offsets, to_storage_id, to_display_id, and their call sites are gone.
- [ ] #2 Descriptor directory values are configuration keys resolved through Config and the project directory; no module hardcodes backlog/tasks, backlog/milestones, archive/tasks, or docs/decisions, and descriptor.rs contains no concat! path literals.
- [ ] #3 New task, milestone, and release records are written as <ID>.md under the configured directories, and no code path constructs or writes a Backlog.md stem.
- [ ] #4 Reading recognises <PREFIX>-<digits>[-slug] and <PREFIX>-<digits> - <slug> stems with case-insensitive prefix matching and optional leading zeros; Backlog-shaped stems (all-lowercase prefix matching a non-all-lowercase configured prefix with an unpadded number, e.g. task-18, m-0) resolve to no record; the front-matter id is authoritative and a stem mismatch is flagged with writes refused.
- [ ] #5 Allocation scans every active, archived, and deprecated location of a family with no offset, milestones start at 1, and a prefix or id_padding change affects only records created afterwards.
- [ ] #6 Archive, restore, deprecate, and restore are moves that keep the file name and ID, and the legacy tasks/archived/ and milestones/archived/ locations are read as archived records.
- [ ] #7 backlog/config.yml is no longer read; statuses come only from Papyrus configuration, and an unrecognised status is preserved and reported.
- [ ] #8 Tests cover allocation, stem resolution including case-insensitive variants, archive and deprecate round trips, and that Backlog-shaped stems (task-18, m-0) resolve to no record.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 All acceptance criteria verified by code review and tests.
- [ ] #2 cargo build succeeds with no warnings introduced by this change.
- [ ] #3 cargo test passes, including rewritten identity, stem-resolution, allocation, and archive/deprecate round-trip tests.
- [ ] #4 A mechanical grep confirms no to_storage_id, to_display_id, storage_prefix, TASK_OFFSET, MILESTONE_OFFSET, backlog/config.yml read, or concat! path literals in src/.
- [ ] #5 No module hardcodes backlog/tasks, backlog/milestones, archive/tasks, or docs/decisions.
- [ ] #6 Descriptor directories resolve from configuration and pass Context::ensure_within.
- [ ] #7 Milestones start at 1; allocation scans all active, archived, and deprecated locations with no offset.
- [ ] #8 Backlog-shaped stems (task-18, m-0, and other all-lowercase unpadded forms of a non-lowercase configured prefix) resolve to no record.
- [ ] #9 No requirement or business-rule files were modified.
<!-- DOD:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Rewrite IdSpec in src/record/identity.rs: single `prefix` field; remove `storage_prefix`, `frontmatter_prefix`, `offset`. Keep `kind` and `padding`. Update constructors (task/milestone/default) to take one prefix.
2. Replace translation methods with Papyrus-only parse/format/validate: `parse_id` returns the number, `format_id` returns `<PREFIX>-<padded>`, `validate_input` accepts only the configured Papyrus prefix (rejects any other form, including former storage forms). Remove `to_storage_id`, `to_display_id`, `parse_display_number`, `parse_storage_number`, `to_frontmatter_id`.
3. Remove module-level translation free functions and constants: `TASK_OFFSET`, `MILESTONE_OFFSET`, `DEFAULT_TASK_STORAGE_PREFIX`, `DEFAULT_MILESTONE_STORAGE_PREFIX`, `to_storage_id`, `to_display_id`, and related helpers. Keep generic Papyrus-ID helpers.
4. Update allocation: remove milestone `+1` offset and `m-0` base; base becomes 1 for every family. `next_milestone_number` scans Papyrus IDs only. `allocate_from_observed_ids` uses the single prefix.
5. Update RecordKind defaults: milestones start at 1.
6. Descriptor (src/record/descriptor.rs): replace `directory: &'static str` (literal path) with a configuration key (backlog/decision/doc plus fixed family subpath). Remove all hardcoded `backlog/tasks`, `backlog/milestones`, `backlog/releases`, `docs/decisions`, `docs/...` and any `concat!` path construction. Provide a resolver mapping descriptor + Config + project dir to a PathBuf passing Context::ensure_within.
7. Update src/record/store.rs (and context.rs) to resolve directories through the descriptor/config resolver; ensure_within on every path.
8. src/task.rs: collapse three-location resolution to configured active + archive locations plus legacy alias read; remove storage translation calls; use Papyrus IDs directly; implement tolerant stem resolution and front-matter-id authority; archive aliases.
9. src/milestone.rs: same as task.rs.
10. src/release.rs: verify no storage translation; uses generic helpers with release prefix.
11. src/migrate.rs: remove storage-translation calls (or delete legacy conversion code as part of TASK-89/TASK-97); at minimum remove to_storage_id/to_display_id usage.
12. src/browser/* and src/commands.rs: replace to_display_id/to_storage_id with identity parse/format; carry Papyrus ID only; remove storage-ID fields from views/JSON.
13. src/init.rs and src/config.rs: stop reading backlog/config.yml; statuses only from Papyrus config. Preserve unrecognised status verbatim and report.
14. Tolerant stem resolution: implement case-insensitive prefix match with optional leading zeros and optional `-slug` or ` - slug` suffix; exclude Backlog-shaped stems (all-lowercase prefix matching a non-all-lowercase configured prefix with an unpadded number, e.g. task-18, m-0) so they resolve to no record. Add helper in identity or store.
15. Front-matter id authority: on read, compare normalised stem ID to front-matter id; on mismatch flag record invalid and refuse writes.
16. Archive/restore/deprecate/restore: implement as file moves keeping name and ID; handle legacy archive aliases as archived locations.
17. Legacy archive aliases: read tasks/archived/ and milestones/archived/ as archived.
18. Rewrite tests: identity translation tests to single-ID tests; allocation tests (milestones start at 1); stem resolution tests including case-insensitive variants and Backlog-shaped exclusion; archive/deprecate round trips; conflict detection.
19. Run cargo build and cargo test; fix compile errors from removed symbols.
20. Mechanical grep verification: no to_storage_id/to_display_id/storage_prefix/TASK_OFFSET/MILESTONE_OFFSET/backlog/config.yml or concat! path literals in src/.
<!-- SECTION:PLAN:END -->
