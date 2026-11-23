---
id: TASK-087
title: Implement papyrus import backlog
status: Done
assignee: []
created_date: '2026-11-11T20:43:00Z'
updated_date: '2026-11-13T06:24:00Z'
labels:
  - import
  - native-layout
  - release-1.0
milestone: MS-017
dependencies:
  - TASK-078
  - TASK-083
  - TASK-084
  - TASK-085
references:
  - docs/specifications/import-backlog-md-records.md
  - docs/specifications/papyrus-native-record-layout-and-single-id-space.md
  - docs/architecture/adr/ADR-0010-papyrus-native-on-disk-record-layout.md
  - docs/architecture/adr/ADR-0011-one-way-backlog-md-import.md
  - docs/architecture/adr/ADR-0012-papyrus-owned-front-matter-metadata.md
  - >-
    docs/architecture/adr/ADR-0013-shared-record-envelope-without-a-storage-layer.md
  - docs/requirements/business/REQ-001.002.md
  - docs/requirements/business/REQ-001.003.md
  - docs/requirements/business/REQ-001.004.md
  - docs/requirements/business/REQ-001.005.md
  - docs/requirements/business/REQ-005.010.md
  - docs/requirements/non-functional/NFR-004.002.md
  - docs/requirements/non-functional/NFR-004.004.md
priority: high
type: feature
ordinal: 39090
---


## Description
Implement the one-way Backlog.md import command `papyrus import backlog` (ADR-0011; ADR-0013 stage 6): the only bridge from a Backlog.md store into the Papyrus-native layout (ADR-0010). The command plans all IDs before writing, reuses the shared front-matter/document parser and record::store writer, and never persists a source ID or mapping store.

Scope (in):
- CLI subcommand `papyrus import backlog --from <source-project-dir> [--source-backlog-dir <dir>] [--dry-run] [--allow-existing] [--report <path>]`; target must be an initialized project (REQ-001.003).
- Import `backlog/tasks`, `backlog/archive/tasks`, `backlog/milestones`, `backlog/archive/milestones`.
- Exclude `docs`, `drafts`, `decisions`, `config.yml`, and locks; count/report them as excluded (NFR-004.004).
- Phase 1: enumerate and sort source records; allocate consecutive Papyrus IDs per family from max(existing target number in active/archive/legacy/deprecated locations)+1, minimum 1; milestones start at 1; no source number preserved.
- Phase 2: resolve references, validate/normalize statuses and timestamps, validate destinations.
- Phase 3: write through record::store::write_atomic; mid-import failure aborts with a summary of records already written.
- Field/body mapping, status normalization, collision/duplicate handling, dry run, containment, and exit codes per docs/specifications/import-backlog-md-records.md.

Scope (out):
- Importing `docs`, `drafts`, `decisions`, or Backlog.md configuration.
- Release records, re-import reconciliation, sync, or reverse export.
- UI/browser changes; no new crate or dependency (ADR-0005).

Design deliverable:
- Feature specification: docs/specifications/import-backlog-md-records.md
- ADRs: ADR-0010, ADR-0011, ADR-0012, ADR-0013
- Requirements: REQ-001.002, REQ-001.003, REQ-001.004, REQ-001.005, REQ-005.010, NFR-004.002, NFR-004.004
- Downstream: TASK-97, TASK-99, TASK-100

Assumptions:
- Assumption: implementation stays in the single crate; any wrapper script only forwards arguments (ADR-0011 decision 1).
- Assumption: target statuses come only from Papyrus configuration; an unrecognized source status is preserved verbatim with a warning (never coerced).
- Assumption: the source store is opened read-only and left byte-identical; no mapping file, sidecar, marker, or extra front-matter key is written.
- Assumption: no UI design or mockup is required because the command has text output only.
- Assumption: no material ambiguity remains; open questions are none (resolved by ADR-0011 on 2026-11-11).

Verification: cargo build/test pass, including no_drift; dry-run writes nothing; source directory byte-identical after import; tests cover the ADR-0011 downstream constraint 8 matrix.

## Acceptance Criteria
- [x] papyrus import backlog --from <dir> [--source-backlog-dir <dir>] [--dry-run] [--allow-existing] [--report <path>] exists with the documented options and exit codes, and the target project must be initialized.
- [x] Every destination path is contained in the project directory, and --report accepts only a path inside the project.
- [x] Import covers backlog/tasks, backlog/archive/tasks, backlog/milestones, and backlog/archive/milestones; docs, drafts, decisions, config.yml, and locks are not read as records and are reported as excluded.
- [x] Phase one allocates consecutive Papyrus numbers per family from the maximum present in every active, archived, and deprecated location, minimum 1, with milestones starting at 1; no source number is preserved and no mapping file, sidecar, marker, or extra front-matter key is written.
- [x] Field mapping follows ADR-0011 decision 5: Papyrus id, verbatim title, normalized status with a warning on mismatch, assignee as a YAML list, validated or approximated timestamps, translated milestone and dependency references with warnings for unresolvable entries, copied references, documentation, type, and ordinal, preserved unknown keys, and no milestone release value.
- [x] Body mapping converts the documented marker sections into the native section layout, drops acceptance-criteria ordinals and marker comments, and appends unknown sections and free text verbatim.
- [x] Collision handling matches ADR-0011 decision 7: same-title records import with a possible-duplicate warning, a non-empty target family requires --allow-existing, source anomalies are reported, and a mid-import failure aborts with a summary of the records already written.
- [x] A dry run performs the full plan and writes nothing, and the source directory is byte-identical after a real import.
- [x] Reading reuses the shared front-matter and document parser and writing reuses record::store, with no Backlog-specific parser, serializer, or direct file write in the import module.
- [x] Command help matches the specification, and tests cover active and archived records, reference translation, dangling references, status mismatch, duplicate source numbers, same-title targets, existing-target refusal, dry run, and source immutability.

## Definition of Done
- [x] #1 Implementation in src/import.rs (or src/import/mod.rs), src/lib.rs command dispatch, and tests; record writes only through record::store::write_atomic.
- [x] #2 cargo build succeeds with no warnings introduced by this change.
- [x] #3 cargo test passes, including no_drift checks and import-specific tests.
- [x] #4 Dry run performs phases 1-2 and writes nothing; source directory is byte-identical after a real import.
- [x] #5 --report writes only inside the project directory; destination paths pass Context::ensure_within.
- [x] #6 Command help and README/papyrus(1) document the command, options, one-way/non-idempotent caveat, and excluded scope.
- [x] #7 Mechanical grep confirms no Backlog-specific parser/serializer and no source storage ID persisted in imported records.
- [x] #8 All acceptance criteria verified by code review and tests.


## Implementation Notes
1. Add CLI surface: `Commands::Import { action: ImportCmd }`, `ImportCmd::Backlog { from, source_backlog_dir, dry_run, allow_existing, report }`, help text, dispatch in `run()` to `import::run()` after `build_context`; register `pub mod import;`.
2. Create `src/import.rs` with `run(ctx, cmd)`, source path resolution, and report writer.
3. Resolve source backlog directory (default `<from>/backlog`), canonicalize, verify existence/readability; enumerate the four in-scope source locations; count excluded `docs`, `drafts`, `archive/drafts`, `decisions`, and skipped non-candidate files.
4. Implement source stem parsing for `<prefix>-<number>` with optional slug and cross-check against source front-matter `id`; report duplicate/unparseable source numbers and stem/id mismatches as source anomalies.
5. Read each candidate with `record::frontmatter::parse` and `record::document::parse`; require `title`; parse status, assignee, timestamps, labels, milestone, dependencies, references, documentation, type, ordinal, and unknown front-matter keys.
6. Phase 1: per family, sort by source number (ties by path), compute base from `record::identity::allocate_next` over `record::descriptor` read directories (tasks/milestones: active, archive, legacy alias; no offset), allocate consecutive target IDs, and build the in-memory source->Papyrus map.
7. Phase 2: normalize status against configured statuses; validate/replace timestamps; translate milestone and dependency references with warnings; validate ordinal; preserve unknown keys; validate `--allow-existing` against existing family records; validate `--report` containment; print/collect the plan.
8. Phase 3: for each planned record, build the native body (marker conversion for tasks; single Description for milestones), patch front matter starting from the source raw with target values, and write via `record::store::write_atomic` to the allocated native path under active or archived directory; abort on first write failure with a written-records summary.
9. Implement dry-run to execute phases 1-2 and print/report the plan without phase 3.
10. Implement collision/duplicate handling: same-title possible-duplicate warnings, existing-family refusal without `--allow-existing`, source anomaly reporting, mid-import abort summaries; never overwrite/renumber/merge.
11. Implement body marker conversion (`SECTION:DESCRIPTION`, `AC` with ordinal removal, `DOD`, `SECTION:PLAN`->`## Implementation Notes`, `SECTION:NOTES`->`## Notes`, `COMMENTS`->`## Comments`; unknown sections/free text appended verbatim; milestone single Description).
12. Implement containment: destination/report paths via `Context::ensure_within`; source symlink escape refusal; no source writes.
13. Update command help and README/papyrus(1) documentation with the command, options, one-way/non-idempotent caveat, and excluded scope.
14. Add integration tests in `tests/import_backlog_cli.rs` covering the ADR-0011 downstream constraint 8 matrix and no_drift compatibility.
15. Run `cargo build`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`; fix findings.


## Notes
Refined TASK-92 against ADR-0010, ADR-0011, ADR-0012, ADR-0013, NFR-004.004, REQ-001.002..REQ-001.005, REQ-005.010, NFR-004.002, and docs/specifications/import-backlog-md-records.md. Scope, ACs, assumptions, dependencies, DoD, and implementation plan aligned; no implementation performed in this pass.
