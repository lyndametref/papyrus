---
id: TASK-275
title: "Build the inventory from one registry-wide scan"
created_date: '2026-11-20T17:43:20Z'
updated_date: '2026-11-22T21:14:14Z'
labels: []
references:
  - ADR-037
  - NFR-007.001
  - TECH-043
  - TECH-072
  - NFR-007.002
  - TC-178
status: Done
assignee:
  - rust-dev
milestone: MS-036
dependencies:
  - TASK-274
ordinal: 50
documentation:
  - src/browser/mod.rs
  - src/browser/records.rs
  - src/registry.rs
  - docs/benchmarks/MS-032-baseline.md
  - tests/benchmark_baseline.rs
  - tests/browser_cli.rs
type: implementation
---

## Description
## Context
`inventory_page` in `src/browser/mod.rs` builds one card per inventory entry and calls `enabled_index` for each card. `enabled_index` calls `records::build_index` for a descriptor, which reads the active directory and calls `load_record` for every file, and the Backlog and Decision aggregate cards rescan every descriptor in their category. The inventory therefore repeats the same directory reads and record parses per card and per aggregate. `docs/benchmarks/MS-032-baseline.md` records the pre-refactor inventory at 1,000 records as 75,546 ms p50 and 76,532 ms p95 (20 samples, baseline-only), and attributes the super-linear rise to per-record configuration and registry rebuilds plus family rescans.

## Scope
Build the inventory from one registry-wide scan over the already-loaded records. One inventory request reads each enabled descriptor's active directory at most once and parses each active record at most once, then derives every card count, every aggregate count, and the Backlog and Decision aggregate lists from that one result. Do not call `build_index` once per card and do not rescan a category for an aggregate card.

Keep archive filtering, disabled-type exclusion, merged group and member ordering, the `INVENTORY_AGGREGATE_AFTER` placement, reference visibility, card titles, notes, hrefs, and the rendered inventory HTML identical. Render from the request snapshot with no per-record `resolve_id`, directory rescan, or configuration read. The performance verdict is TASK-278; this task records a before-and-after measurement from the MS-032 harness and leaves the full-protocol comparison to TASK-278.

## Assumptions
- TASK-273 and TASK-274 provide the request-scoped record snapshot and the serving snapshot; this task does not introduce a second inventory cache across requests.
- The corpus and harness are the MS-032 ones; a reduced or smoke measurement is labelled with its warm-up and measured counts and is never presented as an NFR-007 pass.

## Definition of done
All acceptance criteria pass; the one-scan count is proven by a test seam or instrumentation; the rendered inventory is identical to the pre-change output; the before and after numbers and their sample basis are recorded on this task; the full suite, `cargo clippy --all-targets -- -D warnings`, and `cargo fmt --check` are clean; no user-visible behaviour changes beyond latency.
## Acceptance Criteria
- [ ] One registry-wide scan: one inventory request reads each enabled descriptor's active directory at most once and parses each active record at most once, regardless of the number of cards, aggregate cards, or references.
- [ ] No per-card rebuild: the per-card enabled_index and build_index calls are gone from the inventory path, and every card count, aggregate count, and Backlog and Decision aggregate list comes from the same scan result.
- [ ] Identical output: card membership, order including the INVENTORY_AGGREGATE_AFTER placement, titles, notes, counts, hrefs, aggregate placement, and the rendered inventory HTML are unchanged from the pre-change output.
- [ ] Identical filtering and ordering: archive filtering, disabled-type exclusion, merged group and member ordering, and reference visibility are unchanged.
- [ ] No per-record resolution or configuration read on the inventory or list path: rendering from the request snapshot performs no records::resolve_id, directory rescan, YAML parse, validation, or registry construction per record.
- [ ] Before numbers are recorded with their source and sample basis: the MS-032 baseline records inventory at 1,000 records as 75,546 ms p50 and 76,532 ms p95 from 20 samples, baseline-only, and at 382 records as 11,776 ms p50 and 11,971 ms p95.
- [ ] After numbers are recorded from the MS-032 harness at 100 and 1,000 records on the recorded host, with the corpus definition, warm-up count, measured count, and build profile; a p95 from fewer than 100 samples is labelled baseline-only and is never presented as an NFR-007 pass.
- [ ] The after inventory p95 at 1,000 records is materially below the recorded baseline p95 of 76,532 ms; the final NFR-007.001 verdict and threshold comparison are recorded by TASK-278.
- [ ] Tests prove the one-scan and one-parse counts and compare the inventory output with a characterization fixture; the full suite, cargo test, cargo clippy --all-targets -- -D warnings, and cargo fmt --check are clean; no user-visible behaviour changes beyond latency.
## Comments
- **rust-dev**: rust-dev implementation evidence. Branch task TASK-275; uncommitted per orchestrator instruction (no commit, merge, checkout, or push). Design: RecordSnapshot gains scan_indexes(), built on the TASK-273 listing and parse cache; it lists each enabled descriptor active directory at most once, parses each listed record at most once, and finalizes one Index per enabled descriptor from the shared records. inventory_page calls scan_indexes once and derives every document card count, the Backlog count, and the Decisions aggregate sum from that map; enabled_index and build_index_with_snapshot are gone from the inventory path. List and view paths keep their single descriptor index. Migration conflicts are also request-scoped: migration_conflicts() caches scan_migration_conflicts per descriptor family, and new ensure_front_matter_quiet_with_conflicts and read_document_normalized_with_conflicts thread the cached result, so rendering performs one conflict scan per family instead of one per record; CLI and write paths keep the per-read scan. Counters (unit test registry_scan_lists_each_directory_and_parses_each_record_once): listing_count equals the enabled directory count, parse_count 5 for 2 tasks + 1 milestone + 2 requirements, conflict_scan_count 3 families, all unchanged on a second scan and a cached resolve; scan output matches the per-descriptor index builder. Output: characterization_replay 17 of 17 including the http inventory and http empty-inventory byte and semantic gates, and browser_cli t19a_inventory_hub pass, so membership, INVENTORY_AGGREGATE_AFTER order, titles, notes, counts, hrefs, archive and disabled filtering, merged ordering, and reference visibility are unchanged. Measurement: MS-032 harness inventory-only, PAPYRUS_BENCH_NO_WRITE=1 PAPYRUS_BENCH_PROFILES=records-382,records-1000 PAPYRUS_BENCH_ROUTES=inventory, release LTO+strip, recorded host; corpora 382 (hash 16852d24b236c29d9a109ec5c706645bf3ab5a60615c120a3dcf6e76cb816a15) and 1000 (hash 46d64349137f7017f6ee04bcfd6a3d3042b7b6bc23245749dac400804581988f). Both selected band A by p50, 10 warm-up + 100 measured; 382 ran A-D, 1000 ran A only (owner-approved single-run exemption). 382 run A server p50 205 ms, p95 211 ms (runs B, C, D p95 211, 210, 219) vs baseline p50 11776, p95 11971 baseline-only (20 samples): -98.26% p50, -98.24% p95, 57.4x and 56.7x. 1000 run A server p50 547 ms, p95 556 ms vs baseline p50 75546, p95 76532 baseline-only (20 samples): -99.28% p50, -99.27% p95, 138.1x and 137.6x. After p95 comes from 100 samples, not baseline-only; full NFR verdict is TASK-278. Checks: env -u EDITOR -u VISUAL cargo test --all-targets 1372 passed, 0 failed, 69 suites; cargo clippy --all-targets -- -D warnings clean; cargo fmt --check clean. Files: browser records.rs, browser mod.rs, record migrate.rs, record store.rs, tests benchmark_baseline.rs (route filter env for route-limited no-write runs). Security: no new unsafe, no new crates, snapshot stays request-scoped and display-only, writes still read live bytes and token. Gaps: none in task scope; 1000 p95 556 ms remains above NFR-007.001 250 ms and is for TASK-278 and TASK-279 to record and verify.
- **orchestrator**: Acceptance met: one registry-wide scan with one listing per enabled directory and at most one parse per record, conflict scans cached per family, inventory renders from the single map; 1372 tests pass; 1000-record inventory p95 76532 ms to 556 ms (-99.3%), still above the 250 ms NFR target pending TASK-278 verdict.
