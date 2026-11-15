---
id: TASK-138
title: Implement `check --fix` duplicate-ID repair with refusals and confirmation
status: To Do
assignee:
  - '@rust-dev'
created_date: '2026-11-15T16:12:00Z'
labels:
  - rust
  - cli
  - diagnostics
  - tests
milestone: m-21
dependencies:
  - TASK-137
references:
  - docs/requirements/business/REQ-01.008.md
  - >-
    docs/architecture/adr/ADR-0029-read-only-corpus-diagnostics-with-duplicate-id-repair.md
  - src/record/store.rs
  - src/record/identity.rs
  - src/record/patch.rs
priority: high
ordinal: 41510
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why.** REQ-01.008 and ADR-0029 decision 7 settle `--fix` as a duplicate-ID-only repair that goes through the single write path (ADR-0013 decision 7). No ID-rewrite primitive exists today, and the repair must be safe for identity uniqueness while never rewriting a reference.

**What.** The `--fix` and `--force` flags on `check`, the deterministic keeper rule, ID reassignment through the allocator, the refusal rules, the confirmation prompt, and the tests that pin the accepted semantics.

**In scope.**
- `BuiltinCmd::Check { fix, force }`; `--force` without `--fix` is a usage error with exit 2.
- Keeper rule: the record with the earliest `created_date` keeps the ID; equal or missing dates break on the lowest project-relative path.
- Every other record in the group receives the next free number of its own family and shape from `record::identity::allocate_next`, computed over the active, archived, and legacy directories; prefix and padding come from the descriptor; the file is renamed to the new stem.
- New `record::store::reassign_id(ctx, path, new_id) -> Result<PathBuf>`: patch front-matter `id` and the `# <ID> <title>` heading where the type keeps one, write the new path atomically, then remove the old path, refreshing `updated_date` once. No other module writes.
- Refusals, reported and left untouched: a record whose front-matter `id` and file stem disagree; a keeper or member in a read-only legacy directory; a duplicate **group** ID in a grouped family (member-level duplicates are still repaired).
- No inbound reference rewriting: the prompt lists every affected reference (referring family, record, project-relative path, field, value) and the final report repeats the list.
- One confirmation prompt per run over all planned repairs, read from stdin (`y`/`yes` confirms, anything else cancels with no write); `--force` skips it; when stdin is not a terminal and `--force` is absent, nothing is written and the run exits 2 naming `--force`.
- Concurrency: the file is re-read before each repair and its version token compared with the scan-time token; a changed file is skipped and reported.
- Exit 4 still applies after successful repairs; a second `--fix` on a repaired corpus writes nothing.
- `--fix` repairs nothing else: no dangling references, cycles, self-dependencies, front-matter conflicts, invalid configuration, or `deprecated/` migration.
- Tests: keeper and tie-break, refusals, reference listing before the write with the document untouched after it, non-TTY exit 2, `--force` bypass, idempotence, a crash between the two write steps leaving a readable corpus, and a repaired corpus whose second run is clean.

**Out of scope.** Reference rewriting, `deprecated/` migration, new exit codes, a browser repair surface, and any repair of the other finding kinds.

**Assumptions.** ADR-0029 decisions 7 to 9 are the contract. The prompt is std-only and needs no terminal crate. `--yes` from earlier requirement wording is superseded by `--force`.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 `check --fix` repairs duplicate record IDs only and leaves every other finding reported and unresolved.
- [ ] #2 The record with the earliest created_date keeps the ID, and equal or missing dates break on the lowest project-relative path.
- [ ] #3 Every other record in the group receives the next free family ID from the allocator and its file is renamed to the new stem.
- [ ] #4 record::store::reassign_id writes the new path atomically before removing the old path and is the only writer used by the repair.
- [ ] #5 A record whose front-matter id and file stem disagree, a duplicate whose keeper or member is in a read-only legacy directory, and a grouped-family group ID are refused and left untouched.
- [ ] #6 Inbound references are never rewritten, and the prompt and the final report list every affected reference with referring family, record, path, field, and value.
- [ ] #7 Each repair run prompts for confirmation once, `--force` skips the prompt, and `--force` without `--fix` is a usage error with exit 2.
- [ ] #8 When stdin is not a terminal and `--force` is absent, nothing is written and the run exits 2 naming `--force`.
- [ ] #9 A repair whose file changed after the scan is skipped and reported, and a second `--fix` on a repaired corpus writes nothing.
- [ ] #10 A run that repaired findings still exits 4, and a crash between the two write steps leaves a readable corpus.
<!-- AC:END -->
