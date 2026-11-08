# Adopt the Backlog.md On-Disk Layout for Task and Milestone Records

## Status

Accepted — approved by explicit user confirmation on 2026-11-08

## Date

2026-11-08 (status: Accepted, 2026-11-08)

## Context

NFR-04.003 requires Papyrus task and milestone files to be readable, listable, and updatable by Backlog.md without loss of the fields Backlog.md requires. Milestone files may carry the general front-matter keys and `release` beyond `id` and `title`; those additive keys must be accepted and preserved by Backlog.md. REQ-05.010's traceability records the originating instruction verbatim: "insure the compatibility with task and milestone from backlog md, only acceptable discrpency is the addition of releases for milestone. check the files created with backlog md in @backlog/ and how the folder is structured and implement the same behavior in papyrus".

ADR-0007 decides that front matter carries the metadata and that existing records are migrated lazily, per record, on first read and first write. This ADR decides the surrounding on-disk contract, because front matter alone does not make the files interchangeable: path, filename, ID derivation, body shape, status vocabulary, and configuration precedence all differ today.

A first draft of this ADR proposed adopting Backlog.md task and milestone IDs directly (`TASK-18`, `m-0`) as Papyrus IDs. The user reversed that: Papyrus keeps its documented ID format (`TASK-001`, `MS-001`, configured prefix plus `id_padding`) and **translates** to and from Backlog.md storage IDs at the storage boundary (REQ-01.005). Decision 4 below is that translation, and it supersedes the earlier draft's decision 4.

Observed Backlog.md state in this repository (the only in-project evidence available):

- Layout: `backlog/tasks/<task-N> - <slug>.md`, `backlog/milestones/<m-N> - <slug>.md`, `backlog/docs/`, `backlog/drafts/`, `backlog/archive/tasks/`, `backlog/archive/milestones/`, `backlog/archive/drafts/`, plus `backlog/config.yml` and `backlog/.locks/`.
- Task filename stem: lowercase prefix, unpadded number, ` - ` separator, slugified title. Front-matter `id` is the uppercase prefix and the unpadded number (`id: TASK-18` while the file is `task-18 - ....md`).
- Milestone filename stem and front-matter `id` are the same lowercase form (`m-0`); front matter carries only `id` and `title`. Milestone numbering starts at `m-0`.
- Task references are written in storage form: `milestone: m-12`, `dependencies: [TASK-31, TASK-35]`.
- Task body carries no `#` heading and uses marker comments: `<!-- SECTION:DESCRIPTION:BEGIN -->`/`END`, `<!-- AC:BEGIN -->` with numbered items `- [ ] #1 ...`, `<!-- DOD:BEGIN -->`, `<!-- SECTION:PLAN:BEGIN -->`, `<!-- SECTION:NOTES:BEGIN -->`, `<!-- COMMENTS:BEGIN -->`.
- `backlog/config.yml` defines `task_prefix: "task"`, `default_status: "To Do"`, `statuses: ["To Do", "Initiated", "Refined", "Implemented", "Validated", "Done"]`, `date_format: yyyy-mm-dd`.
- Backlog.md has no release file format.
- Existing task numbers are not contiguous (18, 19, 20, 26, 48–60), and milestones are contiguous from 0 to 12.

Observed Papyrus state in `src/`:

- Tasks are written as `<TASK_PREFIX>-<padded number>.md` with a `# <ID> <title>` heading and `## Status`, `## Milestone`, `## Assignee`, `## Labels`, `## Dependencies`, `## Acceptance Criteria`, `## Comments` sections. Milestones are written as `<MS-001>.md` with `## Release`.
- Archived records go to `backlog/tasks/archived/` and `backlog/milestones/archived/`.
- `papyrus.config.yml` defaults are `task_prefix: "TASK"`, `milestone_prefix: "MS"`, `id_padding: 3`, `statuses: ["ToDo", ...]`, `default_status: "ToDo"`.
- Consequence today: Papyrus cannot resolve a Backlog.md-created task at all. `browser::records::build_index` and `task::view` require a stem of `<PREFIX>-<digits>`; `task-18 - Complete-...` does not match, and `TASK-18.md` does not exist. NFR-04.003 currently fails in both directions.

The forces are: Backlog.md must operate on the same files with no conversion step and no parallel store (NFR-04.001, NFR-04.003); Papyrus must keep its documented, configurable ID format and stable IDs (REQ-01.005); IDs must survive archive, restore, and title changes; and the two conventions must not become two identities for one record.

## Decision

1. **For task and milestone *storage*, the Backlog.md on-disk contract is authoritative.** Papyrus writes and reads the Backlog.md path, filename, front-matter field set, and body layout. Papyrus-specific metadata is limited to the milestone key `release` (plus the general keys, which are additive and tolerated per NFR-04.003); no other extra key, heading, or section is added to task or milestone files. Record *identity* is not adopted from Backlog.md: it is translated (decision 4).

2. **Paths.**
   - Active tasks: `<backlog_directory>/tasks/`.
   - Active milestones: `<backlog_directory>/milestones/`.
   - Archived tasks: `<backlog_directory>/archive/tasks/`.
   - Archived milestones: `<backlog_directory>/archive/milestones/`.
   - Releases keep their Papyrus-only path (`<backlog_directory>/releases/`) because Backlog.md has no equivalent.
   - Legacy read support: `tasks/archived/` and `milestones/archived/` are still read so existing Papyrus archives stay visible; on first read or first write a legacy archive is converted to the Backlog.md archive path and is never written back to the legacy path.

3. **Filename and stem resolution.**
   - `<lowercase storage prefix>-<n> - <slug>.md`, where `<n>` is the storage number without padding and the slug is the title reduced to Backlog.md's convention (lowercase words joined by `-`, punctuation dropped).
   - The `- <slug>` suffix is cosmetic. A record is resolved by its storage prefix and number, never by slug; a stale slug is not a conflict.
   - Title edits do not rename the file. Renaming would move the file a browser session or an in-flight edit is addressing, and would invalidate the ADR-0006 version token path; the front-matter `title` is authoritative and Backlog.md resolves by `id`.
   - Reading accepts every form either tool has produced: case-insensitive prefix match, unpadded or zero-padded number, optional ` - <slug>` suffix. The ID is the front matter `id` when present, otherwise the stem-derived ID, normalized per decision 4.

4. **Two ID spaces, one record identity (supersedes the earlier draft's "adopt Backlog.md IDs").**

   | Record type | Papyrus ID (display and input) | Backlog.md ID (storage) | Number relation |
   |---|---|---|---|
   | Task | `TASK-001` — configured `task_prefix`, zero-padded to `id_padding` | file stem `task-1`; front-matter `id: TASK-1` (uppercase, unpadded) | display number = storage number + 0 |
   | Milestone | `MS-001` — configured `milestone_prefix`, padded | file stem and front-matter `id: m-0` (lowercase, unpadded) | display number = storage number + 1 |
   | Release | `REL-001` — Papyrus only | none | not applicable |

   The relation is an affine per-type translation, not a lookup table:
   - **storage → display:** `prefix(display) + zero-pad(storage_number + offset, id_padding)`.
   - **display → storage:** strip the display prefix (case-insensitive), parse the number, subtract the offset, render as `<storage prefix>-<n>` unpadded; the front-matter `id` form keeps Backlog.md's own casing per type as observed above.
   - **Offsets are fixed constants per type in the record descriptor:** `0` for tasks (both tools number tasks from 1) and `1` for milestones (Backlog.md numbers milestones from `m-0`, Papyrus milestone IDs start at `MS-001`; `m-0` → `MS-001`, `m-12` → `MS-013`). They are not configuration keys and are not configurable, because a user-tunable offset could silently renumber every displayed ID.
   - **Display width (resolved 2026-11-08).** When a storage number's digit count exceeds `id_padding`, the display ID uses the wider number; padding is a minimum, never truncated or errored. No re-padding on disk and no error is raised.
   - **The mapping is derived, never persisted.** There is no mapping file, no sidecar, and no extra front-matter key. The storage ID in the filename and front matter is the only persisted identity; the display ID is computed on every read. Nothing can drift out of sync, and no migration must maintain a table.
   - **Bijection and stability.** Within one type the translation is monotone and injective (same prefix, same offset, padding is display-only), so translate-then-translate-back returns the original value for every existing record. IDs are never renumbered by archive, restore, deprecate, supersede, or title change; a create only ever allocates a number not present in either space (including archived files).
   - **What is shown and where.** CLI `list`/`view` (plain text and `--json`), browser list and detail views, browser URLs, and validation messages use the Papyrus ID. The storage ID is never presented as the record identity; where a diagnostic needs it, it is labelled separately (`storage id: task-18 — file: ...`). Reference values (`milestone:`, `dependencies:`) are stored in storage form and translated for display, input validation, and cross-record resolution.
   - **Input accepts display IDs only (resolved 2026-11-08).** Papyrus commands accept only the Papyrus display format; a Backlog.md storage ID (`task-18`, `m-0`, `TASK-18`) supplied as input is rejected with a hint naming the equivalent Papyrus ID (`TASK-018`, `MS-001`). Storage IDs are never accepted as command input.
   - **Generation is symmetric.** On create, Papyrus allocates the next storage number for the type (after the current maximum in both active and archived locations) and derives the display ID from it; the number is written once and never recomputed.
   - **Consequence for existing projects.** Display numbers follow storage numbers, so this repository's existing records appear as `TASK-018` … `TASK-060` and `MS-001` … `MS-013` (from `task-18` … `task-60` and `m-0` … `m-12`). The documented `TASK-001` example describes a project whose first task has storage number 1; it is not a promise that the first *visible* record is 001.

5. **Front matter.** Tasks carry exactly the Backlog.md key set and order (ADR-0007): `id`, `title`, `status`, `assignee`, `created_date`, `updated_date`, `labels`, `milestone`, `dependencies`, `references`, `documentation`, `type`, `ordinal`; `assignee` is a YAML list. Milestones carry `id`, `title`, the general Papyrus keys `created_date`, `updated_date`, `labels`, `references`, and `release`, matching REQ-05.010 and NFR-04.003. For tasks and milestones the `id` value, and every `milestone:`/`dependencies:` value, is the storage form from decision 4.

6. **Body.** No `#` heading. Task bodies preserve Backlog.md's marker comments and section order; a newly created task is written with `## Description` (SECTION markers) and `## Acceptance Criteria` (AC markers, numbered items). Edits patch only the affected lines and never regenerate the body from a model, so markers, numbering, notes, plan, definition of done, comments, and unknown sections survive. Milestone bodies keep `## Description` in Backlog.md's plain form.

7. **Status vocabulary and layered configuration.** When `<backlog_directory>/config.yml` exists it is read (never written) and is authoritative for the storage layer: `statuses`, `default_status`, the storage `task_prefix`, `date_format`, and numbering. `papyrus.config.yml` is authoritative for the presentation layer (`task_prefix` casing, `milestone_prefix`, `id_padding`) and for every non-task record type. The two layers must describe the same type set: a load-time validation rejects a configuration whose task/milestone display prefixes are missing or cannot map one-to-one onto the storage prefixes. If no Backlog.md configuration exists, Papyrus uses its own prefixes for both layers and still writes unpadded storage numbers so the files stay Backlog.md-compatible. Status values are validated against the Backlog.md list, so Papyrus must not write `ToDo` into a project configured with `To Do`.

8. **Existing records are converted lazily, per record, on first read and first write (ADR-0007 decision 9), not by a bulk pass.** Conversion happens on first access of each legacy Papyrus task/milestone file (`tasks/TASK-018.md`, `milestones/MS-001.md`) and legacy archive (`tasks/archived/`, `milestones/archived/`): it moves the record to the Backlog.md path with the storage filename and storage `id`, and adds any missing front-matter keys. It is ID-preserving: the storage number is taken from the existing Papyrus number (offset applied), the display ID is unchanged before and after. A dry run and a per-file inventory remain available as a reporting and verification aid, but they are not a prerequisite bulk step. Conflicts in decision 9 are detected and reported per record; the conversion refuses to proceed past a conflict rather than choosing a winner.

   Two input shapes must be distinguished, because they do not permit the same guarantee:
   - **Already in the Backlog.md storage shape** (this repository's tasks and milestones): the block is added or extended, timestamp keys are added, and the body, marker comments, and section order stay byte-identical. A task whose front matter is already complete is not rewritten at all.
   - **Legacy Papyrus shape** (heading, `## Status`/`## Assignee`/`## Milestone`/`## Labels`/`## Dependencies` sections, no markers): converting such a file to the storage shape necessarily changes the body — the heading is dropped, the retired metadata sections are removed after their values move to front matter, and `## Description`/`## Acceptance Criteria` are mapped into Backlog.md's marker sections with every unknown section and free-text block appended verbatim, in file order. This is the only migration path permitted to alter a task/milestone body, it happens on first read or first write, and it is reported per file in the dry-run inventory so the change is reviewable. Confirmed (REQ-05.011, user decision 2026-11-08): the byte-identical guarantee applies to bodies already in Backlog.md shape; legacy bodies are converted with a reported per-file diff.

9. **Conflicts and duplicates are detected, reported, and never resolved silently.** Every comparison normalizes both sides first (case-insensitive prefix, padding-insensitive number, offset applied). Caused cases and their outcomes:

   | Situation | Meaning | Outcome |
   |---|---|---|
   | Two files resolve to the same storage ID (`task-18 - a.md`, `task-18 - b.md`; or `TASK-018.md` and `task-18 - ....md`) | Same record written twice | Duplicate: listing shows one entry flagged invalid, `view` by that ID fails with the candidate paths, writes refuse, the lazy conversion stops on that record and reports it. Never merged, never deleted. |
   | Front-matter `id` and filename disagree after normalization (`task-18 - x.md` with `id: TASK-19`) | Corrupt or hand-edited file | Read succeeds using the front-matter `id` (canonical, decision 3); the record is flagged invalid and writes refuse until fixed. |
   | Two files, different numbers, same slug | Distinct records that happen to share a title | Not a conflict; slug is never identity. |
   | A legacy Papyrus file and a Backlog.md file with different numbers but the same title | Possibly the same work tracked twice | Distinct records; the migration reports the pair for a human decision; no automatic equivalence, no merge, no renumber. |
   | A visible ID requested that maps to no storage file | Unknown record | Normal not-found error, listing the expected storage path once for diagnosis. |
   | A number already used in the other ID space | Already a record | Create allocates past every number in both spaces, active and archived. |

   No conflict path deletes, overwrites, or renumbers a record; each reports the paths and the two IDs so a human resolves it.

10. **Interoperability and translation are verified empirically, not assumed.** The NFR-04.003 gate is extended to cover identity: Backlog.md creates `task-61` → Papyrus lists it as `TASK-061` and `view TASK-061` opens it; Papyrus creates `TASK-062` → Backlog.md lists `task-62` with the same title; `milestone: m-12` displays as `MS-013` and `view MS-013` resolves it; `dependencies: [TASK-31]` displays as `TASK-031`. `release` on milestones and the additive general keys must be proven tolerated by Backlog.md's reader before the change is accepted, and the same verification runs on records migrated lazily on first read and first write.

11. **Backlog.md `docs` and `drafts` remain Backlog-owned.** Papyrus does not read or write those directories unless a later requirement makes them managed record types. They are also outside the migration.

12. **IDs are immutable in both spaces.** No operation renumbers, re-pads into the file, or changes the storage prefix of an existing record. A configuration change to display prefix or padding changes the *display* of all records at once and is an explicit, documented action with no write to disk; the storage IDs, and therefore the bijection, are untouched.

## Consequences

**Positive:**

- NFR-04.003 becomes achievable in both directions, and this repository's existing task and milestone files become visible to Papyrus.
- One on-disk convention for task and milestone records removes the two parallel stores, while Papyrus keeps the ID format its documentation and users rely on (REQ-01.005).
- Because the mapping is derived, adopting the Backlog.md layout requires no mapping file, no extra key, and no reconciliation job.
- Archive layout matches the tool that created the archive, so neither tool loses track of archived work, and display IDs survive archive and restore.
- Status vocabulary is shared, so a Papyrus status change appears on the Backlog.md board.
- Reference values stored in storage form resolve in the tool that owns the reference, while users still see Papyrus IDs.

**Negative:**

- Two ID spaces now exist in every surface: documentation, error messages, JSON, and tests must state which space they mean. This is inherent to the product decision, not removable by design.
- Existing projects do not start at `TASK-001`; display numbers mirror storage numbers and may be sparse (`TASK-018`, `TASK-048`). Users who expect contiguous IDs will be surprised.
- The storage and display layers are configured separately and can be misconfigured; only the load-time validation prevents a mapping that is not one-to-one.
- The simple rule "the ID is the filename" no longer holds: reading a filename now requires normalization and offsetting, and every diagnostic has to say which ID space it is using.
- Filename slugs become stale after title edits, because files are not renamed.
- Configuration precedence is layered and must be documented per setting.

**Risks and mitigations:**

- *Two ID spaces confuse users or leak into output*: one projection type supplies the display ID everywhere; a test asserts that no CLI or browser surface prints a storage ID as identity; diagnostics label the storage ID explicitly.
- *Translation bugs (off-by-one on milestones, casing, padding)*: translation is a single pure function in `record::identity` with property tests (round-trip identity for every offset/padding combination, monotonicity, rejection of malformed input); the empirical round trip in decision 10 is the acceptance gate.
- *Mapping persisted accidentally by a later change*: the contract states the mapping is derived and the record has no key for it; a storage ID differing from the pattern is a validation error, not an invitation to add a table.
- *Collision when a Backlog.md file and a Papyrus file denote the same record*: decision 9 detects it by normalized storage ID, stops the migration, and refuses writes; nothing is merged, and the report names both paths.
- *Migration of records that use either format*: conversion on first read or first write reads both shapes (ADR-0007 decision 8), preserves the number, and refuses to guess when both exist; the dry-run inventory shows every rename before it happens.
- *Legacy body conversion losing content*: the legacy-to-storage conversion (decision 8) drops a heading and retires metadata sections; unknown sections and free text are appended verbatim, the dry-run inventory records the before/after body for each converted file, and the conversion is a one-time path with no second use in the writer.
- *Backlog.md rejects or rewrites a file Papyrus wrote*: the NFR-04.003 two-direction verification runs before the change is accepted; any rejected field is a blocker, not a known limitation.
- *Storage front-matter casing asymmetry (task `TASK-18` upper vs milestone `m-0` lower)*: this is observed Backlog.md behavior, encoded as-is; the translation normalizes on read, and a round-trip test confirms Backlog.md accepts what Papyrus writes. If Backlog.md tolerates a uniform form, normalizing is a later simplification, not a correctness requirement.
- *Status validation blocking legitimate edits*: the accepted status set comes from configuration; an unknown observed status is reported and preserved, not normalized.
- *Slug collisions*: resolved by number, never by slug; a duplicate stem is a validation error, not a silent overwrite.
- *A changed display prefix or padding silently rewriting user expectations*: the change is detected at load and reported; storage IDs are untouched, so nothing on disk breaks.

## Resolved open questions (user decisions 2026-11-08)

1. Resolved (user decision 2026-11-08): Papyrus commands accept only the Papyrus display format; a Backlog.md storage ID (`task-18`, `m-0`, `TASK-18`) supplied as input is rejected with a hint naming the equivalent Papyrus ID. Not blocking.
2. Resolved (user decision 2026-11-08): the milestone offset is fixed `+1` (`m-0` → `MS-001`, `m-12` → `MS-013`); not configurable. Not blocking.
3. Resolved (user decision 2026-11-08): when a storage number's digit count exceeds `id_padding`, the display ID uses the wider number; padding is a minimum, never truncated or errored. Not blocking.
4. Resolved (user decision 2026-11-08): confirmed via REQ-05.011 — the byte-identical guarantee covers bodies already in Backlog.md shape; legacy bodies are converted with a reported per-file diff. Blocking for the task/milestone migration task only, not for the design.

## Alternatives Considered

| Option | Outcome | Reason |
|---|---|---|
| Adopt the Backlog.md layout for storage and translate IDs at the boundary (this ADR) | Selected | Satisfies NFR-04.003 in both directions while keeping the Papyrus ID contract of REQ-01.005. |
| Adopt Backlog.md IDs directly as Papyrus IDs (earlier draft of this ADR) | Superseded by product decision | It works mechanically but discards the configured prefix, `id_padding`, and the documented `TASK-001`/`MS-001` examples. |
| Keep the Papyrus layout and add only front matter | Rejected | Backlog.md cannot see `<TASK-001>.md`, `tasks/archived/`, or `## Status`; NFR-04.003 fails. |
| Dual-write a Backlog.md-compatible file next to the Papyrus file | Rejected | Two sources of truth, two files to keep in sync, violates NFR-04.001's single text store. |
| Persist an ID mapping table (sidecar file or extra front-matter key) | Rejected | A second store that can drift, must be migrated, and would have to be repaired when files are edited by Backlog.md; a derived affine map needs neither. |
| Make the display prefix and offset fully configurable per project | Rejected | Adds renumbering hazards with no observed need; display prefixes and padding are already configurable, the offset is a fixed property of each tool's numbering. |
| Convert on import/export only | Rejected | Requires a conversion step, which NFR-04.003 explicitly rules out. |
| Use `papyrus.config.yml` as the sole configuration and ignore `backlog/config.yml` | Rejected | The two configurations already disagree on statuses and prefixes; Backlog.md would break on files Papyrus writes. |
| Adopt Backlog.md layout for every record type (requirements, decisions, docs) | Rejected | Backlog.md has no format for those; Papyrus's document conventions stay authoritative there. |

## References

- `docs/requirements/non-functional/NFR-04.003.md` — interoperability requirement and verification method.
- `docs/requirements/business/REQ-01.005.md` — stable IDs, two ID spaces, bidirectional translation, reference translation.
- `docs/requirements/business/REQ-05.010.md` — front-matter field map and Backlog.md compatibility statement.
- `docs/requirements/business/REQ-03.001.md`, `REQ-03.002.md` — milestone and release identity and lifecycle.
- `docs/architecture/adr/ADR-0007-yaml-front-matter-as-single-record-metadata-source.md` — front-matter field map, canonical key order, lazy per-record migration.
- `docs/architecture/adr/ADR-0009-shared-record-envelope-and-unified-write-path.md` — one writer, ID translation at the store boundary, migration staging.
- Observed files: `backlog/config.yml`, `backlog/tasks/*.md`, `backlog/milestones/*.md`, `backlog/archive/`.
- Implementation: `src/task.rs` (`active_dir`, `archived_dir`, `create`, `view`, `list`), `src/milestone.rs` (`active_dir`, `create`), `src/browser/records.rs` (`build_index` stem filter, `resolve_id`), `src/config.rs` (`Config::default`).
