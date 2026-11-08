# Feature Specification — Translate Papyrus and Backlog.md IDs at the Storage Boundary

**Ticket:** TASK-64
**Milestone:** m-12 (Final Cross-Type Integration)
**Status:** Refined (execution-ready)
**UI mockups:** none

## Purpose

Deliver ADR-0009 Stage 2: a single pure translation layer in `src/record/identity` that converts between Papyrus display IDs (`TASK-001`, `MS-001`, configured prefix plus `id_padding`) and Backlog.md storage IDs (`task-1`, `m-0`, unpadded with Backlog.md's own casing) at the storage boundary, including reference-value translation, storage-number allocation across active and archived records, and conflict detection. Commands continue to accept and display Papyrus IDs only. This ticket implements REQ-01.005 (two ID spaces, bidirectional mapping, reference translation, input rejection of storage IDs), supports NFR-04.003 (interoperability), and is the prerequisite for Stage 4 migration of tasks and milestones (TASK-69).

## Scope

### In scope

- A new `src/record/identity` module (file or directory) that is the **only** location of ID parsing, normalization, formatting, and translation in the crate (ADR-0009 decision 7).
- Bidirectional, derived (never persisted) translation between Papyrus display IDs and Backlog.md storage IDs for **task** and **milestone** records only.
- Fixed per-type offsets: task `0`, milestone `1`. Offsets are constants in the identity module, not configuration keys.
- Configured display prefix and `id_padding` (minimum width; wider numbers are never truncated or errored).
- Backlog.md storage prefix for tasks from the storage-layer configuration (default `task`); milestone storage prefix `m` (Backlog.md convention).
- Backlog.md storage-form casing: task storage front-matter `id` `TASK-<n>` (uppercase, unpadded); milestone storage `id` `m-<n>` (lowercase, unpadded); filename stems `task-<n>` / `m-<n>`.
- Reference-value translation for `milestone:` and `dependencies:` (storage form on disk, display form in commands and output).
- Storage-number allocation considering active and archived records in both ID spaces; create never reuses a number.
- Conflict detection per ADR-0008 decision 9: duplicate storage IDs, front-matter/filename mismatch, and related cases; never merge, delete, or renumber.
- Input validation: commands accept display IDs only; a Backlog.md storage ID supplied as input is rejected with a hint naming the equivalent display ID.
- Diagnostic labelling: any diagnostic that needs the storage ID labels it separately (`storage id: ...`); no CLI text/JSON, browser view, or URL presents a storage ID as the record identity.
- Unit and property tests for round-trip identity, monotonicity, injectivity, and rejection of malformed IDs across prefix, padding, and offset combinations.
- No-drift grep checks: no ID formatting outside `src/record/identity`.

### Out of scope

- Lazy migration of tasks and milestones to the Backlog.md layout and storage IDs — TASK-69 (Stage 4), which consumes this module.
- CLI option exposure for front-matter fields — TASK-65.
- Removal of legacy per-module parsers/writers and no-drift enforcement beyond checks — TASK-70.
- Browser UI changes and metadata projection — TASK-66, TASK-67, TASK-72.
- Release records (Papyrus-only, no storage counterpart) and all other non-task/non-milestone record types (display-only IDs).
- Corpus migration or any write that changes an existing record body.
- New configuration keys; offsets are not configurable.

## Behavior

### 1. Module ownership and no-drift

1. `src/record/identity` is the sole location of ID parsing, normalization, formatting, and translation. No other module may format an ID string, strip a prefix, pad a number, or apply an offset.
2. The module exposes pure functions operating on strings and numbers; it performs no filesystem I/O except where an allocator helper is explicitly given directory inputs by its caller. Translation itself is pure and total over valid inputs.
3. Every record type's identity is representable through this module. For types without a storage counterpart (release and all document records), the module provides display-only parse/format and no translation.
4. A grep check asserts no ID-formatting code exists outside `src/record/identity`.

### 2. ID spaces

Two ID spaces exist per ADR-0008 decision 4:

| Record type | Papyrus display ID | Backlog.md storage ID (filename stem / front-matter `id`) | Offset (display = storage + offset) |
|---|---|---|---|
| Task | `<task_prefix>-<padded n>` (e.g. `TASK-001`) | `task-<n>` stem; front-matter `id: TASK-<n>` (uppercase, unpadded) | 0 |
| Milestone | `<milestone_prefix>-<padded n>` (e.g. `MS-001`) | `m-<n>` stem and front-matter `id` (lowercase, unpadded) | 1 |
| Release | `<release_prefix>-<padded n>` | none | not applicable |
| Other records | configured prefix + padded number | none | not applicable |

1. The storage prefix for tasks is the storage-layer value (Backlog.md `task_prefix`, default `task`); the display prefix is the presentation-layer value (Papyrus `task_prefix`, default `TASK`). The two are paired by configuration validation (ADR-0008 decision 7).
2. The milestone storage prefix is the Backlog.md convention `m` and is not a Papyrus configuration key.
3. Offsets are fixed constants per type in the identity module: `0` for tasks, `1` for milestones. They are not configuration keys and are not user-tunable.

### 3. Display format

1. A display ID is `<PREFIX>-<number>` where `<PREFIX>` is the configured display prefix for the record type and `<number>` is the storage number plus the offset, zero-padded to at least `id_padding` digits.
2. Padding is a minimum width: when a storage number's digit count exceeds `id_padding`, the display ID uses the wider number. No truncation and no error is raised.
3. Prefix matching on input is case-insensitive.
4. A display number must be a positive integer greater than the offset (so the storage number is `>= 0` for milestones? milestone storage starts at 0; display starts at 1). More precisely: storage number `n >= 0`; display number `n + offset`. A display number less than `offset` is invalid.

### 4. Storage format

1. A storage ID is `<storage_prefix>-<n>` where `<storage_prefix>` is the storage-layer prefix and `n` is the unpadded non-negative integer storage number.
2. For tasks, the front-matter `id` uses Backlog.md's observed casing `TASK-<n>` (uppercase prefix, unpadded number) while the filename stem uses `task-<n>` (lowercase). The identity module normalizes both to the same storage number on read.
3. For milestones, the front-matter `id` and filename stem are both `m-<n>` (lowercase).
4. The storage ID is never presented as the record's identity in any output.

### 5. Translation

1. `display → storage`: strip the display prefix (case-insensitive), parse the number, subtract the offset, render as `<storage_prefix>-<n>` (unpadded) using the type's storage casing rules above.
2. `storage → display`: strip the storage prefix (case-insensitive), parse the number, add the offset, render as `<display_prefix>-<zero-padded number>` (minimum `id_padding`).
3. The mapping is an affine per-type function: no lookup table, no sidecar, no persisted mapping key.
4. `translate(storage) → display → storage` and `translate(display) → storage → display` return the original value for every valid ID of the type (round-trip identity).
5. Translation is monotone and injective within one type (same prefix, same offset, padding display-only).
6. Malformed input (wrong prefix, non-numeric, missing separator, empty number, out-of-range) is rejected with a validation error, never silently coerced.

### 6. Normalization

1. Normalization collapses case, padding, and casing asymmetry so that `task-18`, `TASK-18`, `task-018`, and the front-matter form `TASK-18` all resolve to storage number 18 for a task.
2. Normalization is applied before any comparison, conflict check, or reference resolution.
3. A record is identified by its normalized storage ID; the slug in a filename is never part of identity.

### 7. Reference values (`milestone:`, `dependencies:`)

1. Reference values are stored on disk in storage form (`m-12`, `task-31` / `TASK-31` per type casing).
2. On read they are translated to display form for command output, browser views, and JSON.
3. On input they are accepted only in display form and translated to storage form before being written.
4. A reference resolves to a record in both Papyrus and Backlog.md after translation.
5. A reference whose storage form maps to no record is a not-found error (exit code 3), consistent with existing commands.
6. `milestone:` and `dependencies:` values pass through the same identity module; no separate reference parser exists.

### 8. Input validation

1. Papyrus commands accept only the Papyrus display format as a record ID input.
2. A Backlog.md storage ID supplied as input (`task-18`, `m-0`, or the front-matter casing `TASK-18`) is rejected with a hint naming the equivalent display ID (`TASK-018`, `MS-001`).
3. The rejection is a validation error (exit code 4) and names both the supplied storage ID and the equivalent display ID.
4. A storage ID appearing in a URL is rejected like any unknown ID (no special-casing beyond the hint).
5. Storage IDs are never accepted as command input, including in `--milestone`, `--depends-on`, `--release`, view/edit/archive/restore/delete arguments, and search filters.

### 9. Storage-number allocation

1. On create, Papyrus allocates the next storage number for the type.
2. Allocation scans both active and archived locations for the type and considers every file whose name or front-matter `id` normalizes to a storage number of that type, including files in Backlog.md shape and legacy Papyrus shape (legacy display number converted via the reverse offset).
3. The allocated number is `max(observed storage numbers) + 1`, or `0` (milestone) / `1` (task) when no records exist. (Task storage numbers start at 1; milestone storage numbers start at 0.)
4. A number present in either ID space (active or archived, any shape) is never reused.
5. The allocated storage number is written once and never recomputed; the display ID is derived from it at read time.
6. Allocation is deterministic given the same corpus.

### 10. Conflict detection

Every comparison normalizes both sides first (case-insensitive prefix, padding-insensitive number, offset applied). Outcomes per ADR-0008 decision 9:

| Situation | Meaning | Outcome |
|---|---|---|
| Two files resolve to the same storage ID | Same record written twice | Duplicate: listing shows one entry flagged invalid; `view` by that ID fails naming candidate paths; writes refuse; lazy conversion stops on that record and reports it. Never merged, deleted, or renumbered. |
| Front-matter `id` and filename disagree after normalization | Corrupt or hand-edited file | Read succeeds using the front-matter `id` (canonical); record flagged invalid; writes refuse until fixed. |
| Two files, different numbers, same slug | Distinct records sharing a title | Not a conflict; slug is never identity. |
| Legacy Papyrus file and Backlog.md file with different numbers but same title | Possibly same work tracked twice | Distinct records; migration reports the pair for a human decision; no automatic equivalence, merge, or renumber. |
| Requested visible ID maps to no storage file | Unknown record | Normal not-found error (exit code 3), listing the expected storage path once for diagnosis. |
| Number already used in the other ID space | Already a record | Create allocates past every number in both spaces, active and archived. |

1. No conflict path deletes, overwrites, or renumbers a record.
2. Each conflict report names both paths and both IDs (display and storage) so a human resolves it.
3. Conflict detection is pure over a list of candidate (path, raw id) pairs and is reusable by allocation and migration.

### 11. Diagnostics and output

1. CLI text, `--json`, browser list/detail views, and URLs use the display ID as the record identity.
2. The `metadata` object (stable public contract) carries the storage ID under `metadata.id`; no second top-level identity field is introduced.
3. Where a diagnostic needs the storage ID, it is labelled separately, e.g. `storage id: task-18 — file: backlog/tasks/task-18 - ....md`.
4. A test asserts no CLI or browser surface prints a storage ID in the identity position.

### 12. Error behavior

1. Malformed ID (bad prefix, non-numeric, missing separator, empty number, out of range) → validation error, exit code 4.
2. Unknown record → not found, exit code 3.
3. Storage ID supplied as input → validation error, exit code 4, with hint naming the equivalent display ID.
4. A reference that cannot be resolved → not found, exit code 3.
5. Conflicting files are reported, not silently resolved; affected writes refuse until resolved.
6. No panic on malformed input.

## Inputs and outputs

- Input: a record type, an ID string, an `id_padding`, display/storage prefixes, and (for allocation) the active and archived directories.
- Output: a storage ID string, a display ID string, a normalized storage number, an allocation result, or a validation/conflict error.
- No user-facing command output is added by this ticket; the module is consumed by later tasks.

## Validation and error behavior

- Malformed ID → exit code 4.
- Unknown record → exit code 3.
- Storage ID as input → exit code 4 with hint naming the equivalent display ID.
- Unresolvable reference → exit code 3.
- Conflict → affected write refuses; report names paths and both IDs.

## Assumptions

- Assumption: `src/record/identity` is a module inside the existing crate; no new crate or dependency is added (ADR-0004, ADR-0005).
- Assumption: offsets are fixed constants (`0` task, `1` milestone) and are not configuration keys (ADR-0008 decision 4).
- Assumption: task storage prefix defaults to `task` and milestone storage prefix is `m`; display prefixes and padding come from `papyrus.config.yml` (ADR-0008 decisions 4 and 7).
- Assumption: the storage ID is never persisted as a mapping; translation is derived on every read.
- Assumption: conflict detection is pure and reusable by allocation and by the Stage 4 migration (TASK-69).
- Assumption: no corpus record is modified by this ticket; only the identity module and its tests are added.
- Assumption: UI mockups are excluded; this ticket has no UI design (no MOCK ids).

## Open questions

- OQ-1: Whether the module is a single file `src/record/identity.rs` or a directory `src/record/identity/` with submodules (types, parse, translate, allocate, conflicts). Either is acceptable provided the no-drift grep checks pass and callers use `record::identity` as the entry point.
- OQ-2: Whether allocation scans raw directory entries itself or receives a caller-provided list of observed (path, raw id) pairs. The latter keeps the module free of I/O and is preferred, but the implementation may choose, provided the "active and archived, both shapes" contract holds.

## Traceability

| Artifact | Reference |
|---|---|
| Ticket | TASK-64 |
| Requirements | REQ-01.005, REQ-05.010, NFR-04.003 |
| ADRs | ADR-0008 (decisions 4, 7, 8, 9), ADR-0009 (decisions 1, 2, 7, 8 Stage 2) |
| Related tickets | TASK-63 (dependency, Done), TASK-65, TASK-67, TASK-69, TASK-70, TASK-71 |
| Source | src/record/ (stage 1 envelope), src/task.rs (legacy scan_max, create, view), src/milestone.rs, src/config.rs |
| Tests | property tests in src/record/identity (to be added) |
