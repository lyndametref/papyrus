# Feature Specification — Verify Backlog.md Round-Trip Interoperability

**Ticket:** TASK-71
**Milestone:** m-12 (Final Cross-Type Integration)
**Status:** Refined (execution-ready)
**UI mockups:** none

## Purpose

Empirically verify NFR-04.003: Papyrus task and milestone files remain readable, listable, and updatable by Backlog.md in both directions, including translated identity (REQ-01.005) and the additive milestone general keys and `release` (REQ-05.010), on newly created records and on records migrated lazily on first read and on first write, covering both migration input shapes. This is the Backlog.md round-trip gate and is a verification task only: no new product behavior is added.

## Scope

### In scope

- Empirical two-direction verification of task and milestone records:
  - Backlog.md-created task/milestone → Papyrus list/view/edit.
  - Papyrus-created or migrated task/milestone → Backlog.md list/view/update.
- Translated identity checks: storage ID ↔ display ID (task offset `0`, milestone offset `1`), including `milestone: m-12` ↔ `MS-013` and `dependencies: [TASK-31]` ↔ `TASK-031`.
- Additive milestone general keys and `release` tolerated and preserved by Backlog.md.
- Newly created records and records produced by lazy migration, from both input shapes (Backlog.md storage shape and legacy Papyrus shape), including idempotency across repeated reads and writes.
- Recording any field rejected, rewritten, or dropped by either tool as a blocker with the exact command and output.
- Recording verification evidence (environment, versions, timestamps in UTC, commands, observed output) in the TASK-71 ticket notes.

### Out of scope

- Release records, Backlog.md `docs`/`drafts`, and all non-task/non-milestone record types (NFR-04.003 scope).
- Product code changes; any defect found is recorded as a blocker and handled by a separate task. A minimal fix is permitted only if explicitly routed and reported separately.
- UI mockups and browser UI changes.
- Corpus-wide migration or modification of repository records (verification uses a scratch project; identity checks may use read-only commands against the project's own backlog).

## Verification environment

- Papyrus binary built from the current source (`target/debug/papyrus` or `target/release/papyrus`).
- Backlog.md CLI version 1.55.3 (or the version available in the verification environment).
- A scratch project created under `./.tmp/` (project convention; no system temporary directories) using `papyrus init ... --no-git` or `backlog init ... --no-git`, isolated from the repository's own backlog.
- Identity checks may use read-only Backlog.md/Papyrus commands against the repository's own `backlog/` (notably `m-12`/`MS-013` and `TASK-31`/`TASK-031`).
- Evidence timestamps use ISO-8601 UTC with timezone (for example `2026-11-09T20:30:00Z`).

## Procedure

### A. Backlog.md → Papyrus (task)

1. In the scratch project, create a task with Backlog.md:
   - `backlog task create "Interop Task BL" -d "probe" --labels interop --ac "first criterion"`.
2. Record the resulting storage ID (for example `task-<n>`).
3. Run `papyrus task list` and confirm the translated display ID (for example `TASK-<n>`) is shown.
4. Run `papyrus task view TASK-<n>` and confirm exit code 0 and that title, status, labels, acceptance criteria, and description match the Backlog.md record.
5. Run `papyrus task edit TASK-<n> --title "Interop Task BL Updated"`; then run `backlog task view task-<n>` and confirm the title change is visible to Backlog.md.
6. Repeat the view commands and confirm the file is byte-identical after the first pass (idempotency).

### B. Backlog.md → Papyrus (milestone)

1. In the scratch project, create a milestone with Backlog.md:
   - `backlog milestone add "Interop Mile BL" -d "probe"`.
2. Record the resulting storage ID (for example `m-<n>`).
3. Run `papyrus milestone list` and confirm the translated display ID (for example `MS-<n+1>`) is shown.
4. Run `papyrus milestone view MS-<n+1>` and confirm exit code 0 and that title and description match.
5. Use Papyrus to assign general keys and a release (if a release exists) to that milestone; then run `backlog milestone list`/`backlog milestone ...` and confirm Backlog.md still reads the file and the additive keys remain in the file (record any key dropped by Backlog.md as a blocker).

### C. Papyrus → Backlog.md (task)

1. In the scratch project, create a task with Papyrus:
   - `papyrus task create "Interop Task PAP" -d "probe"`.
2. Record the resulting display ID (for example `TASK-<n>`).
3. Run `backlog task list` immediately and record whether the record is visible. If it is not visible, record the exact command and output as a blocker (this is the round-trip gate).
4. Run `papyrus task list` (or `view`) to allow any lazy migration, then run `backlog task list` and `backlog task view <storage-id>`; record visibility and field fidelity (title, status, labels, dependencies, references, documentation, type, ordinal).
5. Run `backlog task edit <storage-id> -t "Interop Task PAP Updated"`; then run `papyrus task view TASK-<n>` and confirm the title change is visible to Papyrus (or record any dropped field as a blocker).

### D. Papyrus → Backlog.md (milestone)

1. In the scratch project, create a milestone with Papyrus:
   - `papyrus milestone create "Interop Mile PAP" -d "probe"`.
2. Record the resulting display ID and run `backlog milestone list`; record visibility and field fidelity.
3. Use Backlog.md to update the milestone (rename or archive) and confirm Papyrus still reads it and preserves additive keys/release (or record any dropped field as a blocker).

### E. Migrated records (both input shapes)

1. Seed the scratch project with:
   - A legacy Papyrus-shaped task file (no front matter, `# <ID> <title>` heading, retired metadata sections).
   - A storage-shaped task file (complete Backlog.md front matter and marker body).
   - A legacy Papyrus-shaped milestone file.
   - A storage-shaped milestone file (Backlog.md front matter with only `id` and `title`, no general keys).
2. Trigger Papyrus reads (`papyrus task list`/`view`, `papyrus milestone list`/`view`) and confirm migration occurs and is reported.
3. Run Backlog.md list/view/update against the migrated files and confirm readability and field fidelity.
4. Repeat Papyrus and Backlog.md reads/writes and confirm the files are byte-identical after the first migration (idempotency).

### F. Identity checks (repository backlog, read-only)

1. Run `papyrus milestone view MS-013` and confirm it resolves to the record whose storage identity is `m-12` (milestone offset `1`).
2. Run `papyrus task view TASK-031` (or the task whose dependency is stored as `TASK-31`) and confirm the dependency displays as `TASK-031` (task offset `0`).
3. Run `backlog task view task-<n>` / `backlog milestone view m-<n>` for a known record and confirm Papyrus displays the translated ID.

### G. Blocker recording

1. For every field rejected, rewritten, dropped, or not preserved by either tool, record in the ticket notes:
   - the exact command run,
   - the file content before and after,
   - the observed output/error,
   - the affected field and record type.
2. TASK-71 is not marked Done while any blocker remains open.

## Acceptance criteria

1. Verification evidence shows a Backlog.md-created task is listed by Papyrus under its translated display ID and `papyrus task view <ID>` resolves with matching title, status, assignee, labels, milestone, dependencies, references, documentation, type, and ordinal (or any mismatch is recorded as a blocker).
2. Verification evidence shows a Papyrus-created or migrated task is listed by Backlog.md with the same title and that Backlog.md view/edit preserve or are documented to default the required fields (or any dropped field is recorded as a blocker).
3. Verification evidence shows milestone `m-12` maps to `MS-013`, `papyrus milestone view MS-013` resolves, and a dependency stored as `TASK-31` displays as `TASK-031`.
4. Verification evidence shows Backlog.md list/view/update commands accept a milestone file carrying the general keys plus `release` and preserve those keys (or any dropped key is recorded as a blocker with exact command and output).
5. Verification covers newly created records and records migrated from both input shapes, with idempotency demonstrated by byte-identical output on a second pass.
6. Any field rejected, rewritten, or dropped by either tool is recorded as a blocker with exact command and output; TASK-71 is not Done while such a blocker exists.
7. Verification evidence records environment details (Papyrus binary, Backlog.md version, date/time in UTC) and the scratch project location.

## Assumptions

- Assumption: verification is performed manually and evidence is recorded in the ticket notes; no new product code is added by this ticket.
- Assumption: the Backlog.md CLI is available in the verification environment at version 1.55.3.
- Assumption: scratch projects are created under `./.tmp/` and removed after verification, including on failure.
- Assumption: repository records are not modified except for read-only identity checks; any necessary edits use a scratch project.
- Assumption: a missing optional field after a Backlog.md round trip is acceptable if Papyrus re-applies its default on read; a changed non-default value is a blocker.
- Assumption: status values are carried verbatim; a status value rejected by either tool is a blocker.

## Open questions

- OQ-1: Whether Backlog.md should be expected to preserve Papyrus-only fields such as `documentation` and `type` when they carry non-default values, or whether Papyrus should tolerate their loss as defaults. The verification records observed behavior; resolution may require a product decision if Backlog.md drops non-default values.
- OQ-2: Whether milestone identity checks should be performed against the repository's own `m-12`/`MS-013` or against a freshly created milestone in a scratch project. The repository record is used for read-only verification; if it has changed, use a scratch record and record the substitution.
- OQ-3: Whether an automated regression test (invoking both CLIs) should be added after manual verification passes. Not required by this ticket; if desired, track separately.

## Traceability

| Artifact | Reference |
|---|---|
| Ticket | TASK-71 |
| Requirements | REQ-01.005, REQ-05.010, REQ-05.011, NFR-04.003 |
| ADRs | ADR-0007, ADR-0008, ADR-0009 |
| Related tickets | TASK-69 (Done), TASK-70 (Done), TASK-20 |
| Source | src/record/*, src/task.rs, src/milestone.rs |
| Tests | tests/migrate_records_cli.rs, tests/no_drift.rs |
| UI mockups | none |