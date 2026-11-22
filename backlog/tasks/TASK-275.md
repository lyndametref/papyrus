---
id: TASK-275
title: "Build the inventory from one registry-wide scan"
created_date: '2026-11-20T17:43:20Z'
updated_date: '2026-11-22T20:18:03Z'
labels: []
references:
  - ADR-037
  - NFR-007.001
  - TECH-043
  - TECH-072
  - NFR-007.002
  - TC-178
status: ToDo
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
