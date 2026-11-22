---
id: TASK-313
title: "Optimise the read path to meet the NFR-007 targets at 1,000 records"
created_date: '2026-11-22T21:49:04Z'
updated_date: '2026-11-22T22:24:24Z'
labels:
  - performance
references:
  - NFR-007.001
  - NFR-007.002
  - ADR-037
  - MS-036
status: Done
assignee:
  - rust-dev
milestone: MS-036
dependencies:
  - TASK-277
  - TASK-278
ordinal: 75
documentation:
  - tests/benchmark_baseline.rs
  - docs/benchmarks/MS-036-results.md
  - docs/benchmarks/MS-032-baseline.md
  - src/browser/mod.rs
  - src/browser/records.rs
  - src/browser/http.rs
  - src/browser/routes.rs
type: implementation
---

## Description
## Context
TASK-278 measured NFR-007 on the available host (6 physical / 12 logical cores, 15.3 GiB) with the full protocol. Inventory at 1,000 records is 550 ms p95 against a 250 ms target; GroupedMarkdown group and member view and edit are 118 to 126 ms p95 against 100 ms; the unresolved-reference view and edit are 119 to 120 ms p95 against 100 ms. SingleMarkdown view and edit are 38 ms, SingleHtmlMockup is 4 ms, and NFR-007.003 passes. This host is faster than the NFR-007 reference hardware, so the misses are a read-path cost, not a hardware artefact. MS-036 AC-1 and AC-2 remain unmet until these routes pass.

## Scope
Profile and optimise the read path so the NFR-007 targets are met on this host:
- Inventory at 1,000 records: p95 at or below 250 ms, from 550 ms. Likely work: parse only front matter for list rows instead of full documents; remove redundant per-record work; reduce allocation and escaping cost; keep one listing per directory.
- GroupedMarkdown group and member view and edit, and the unresolved-reference view and edit, at 1,000 records: p95 at or below 100 ms, from roughly 120 ms. Likely work: build the member list from front-matter titles only, with no body parse; reuse the request snapshot's cached parses; avoid re-reading the group record per row.

Out of scope: any functional or visual change, any new route, cache class, template, client script, or crate; re-measuring on other hardware; editing NFR records; repairing the packaging or verification work of TASK-279.

## Measurement
Reuse the TASK-278 harness and protocol unchanged: 10 warm-up plus 100 measured sequential requests where the route is under 1 s, two runs, one server process, loopback HTTP, release build with LTO and strip. Measure the 100, 382, and 1,000 record profiles for every affected route. Append a results table to the MS-036 benchmark results document with the route, profile, p50 and p95, the target, and the pass or miss; record the file at docs/benchmarks/MS-036-results.md in the task documentation field.

## Definition of done
All acceptance criteria pass; the affected routes are measured within their targets or a miss is recorded with the measured figure and the remaining obstacle; no behaviour change; the full suite, clippy, fmt, and characterization replay are green; the results table is appended to the MS-036 benchmark results document; evidence is recorded on this task.

## Acceptance Criteria
- [ ] Inventory p95 at 1,000 records is measured at or below 250 ms with the TASK-278 harness and protocol (10 warm-up plus 100 measured sequential requests, two runs), down from 550 ms.
- [ ] Grouped group and member view and edit p95 at 1,000 records are measured at or below 100 ms with the same harness and protocol, down from 118 to 126 ms.
- [ ] Unresolved-reference view and edit p95 at 1,000 records are measured at or below 100 ms with the same harness and protocol, down from 119 to 120 ms.
- [ ] SingleMarkdown view and edit and SingleHtmlMockup routes stay within their targets and do not regress against the TASK-278 figures.
- [ ] Measurements cover the 100, 382, and 1,000 record profiles for every affected route and are appended as a results table to the MS-036 benchmark results document.
- [ ] No behaviour change: record output, JSON payloads, rendered HTML, status codes, headers, and the characterization replay are unchanged.
- [ ] The full suite, cargo clippy --all-targets -- -D warnings, and cargo fmt --check are clean.
- [ ] If a target is still missed after the optimisation, the measured figure and the remaining obstacle are recorded instead of a claimed pass.
## Comments
- **rust-dev**: TASK-313 read-path optimisation complete in the working tree. No commit was made per instruction.
- **rust-dev**: Profile before optimisation at 1,000 records: inventory 550 ms p95; grouped group and member view and edit 118-126 ms; unresolved-reference view and edit 119-120 ms; single-markdown 38-39 ms; mockup 4 ms. Inventory was dominated by full document parse plus the read projection and field map for 1,000 files. Grouped and unresolved routes built a full sibling index for 200 or 201 records, and edit built an index it never read. Each per-record store read re-read the file for relocation and migration probes and parsed YAML up to four times.
- **rust-dev**: Changes, no new crates and no cross-request cache: inventory uses RecordSnapshot::scan_counts with one listing per descriptor directory and one validated front-matter read per listed file, counting only. Grouped view and edit use build_grouped_index_with_snapshot with front-matter-only sibling records and reuse a full record already in the request snapshot. View and edit no longer build an index unless the page reads one: grouped family rows or an owning group link, task dependents. read_snapshot_source reads once for the common case and falls back to the shared store for adoption, archived relocation, blocking conflicts, malformed front matter or read errors, with the same undeclared-status warning. RecordView::from_front_matter and frontmatter::value_string reuse parsed front matter; requirement_link uses one projection per member row. scan_migration_conflicts reads and parses each candidate once instead of twice.
- **rust-dev**: Measurement: TASK-278 MS-036 harness unchanged, full protocol, all routes band A with 10 warm-up plus 100 measured, two runs, profiles records-100 records-382 records-1000, release LTO strip, one server process, loopback HTTP. Command: cargo test --release --test benchmark_baseline -- --ignored --nocapture benchmark_ms036. Tables appended to the MS-036 results document under TASK-313 read-path optimisation re-measurement; raw ms036 JSON files regenerated.
- **rust-dev**: Before to after p95 at 1,000 records, run A and run B, target, verdict: inventory 550 to 248 and 248 ms, target 250, PASS measured. view-grouped-group 126 to 65 and 66 ms, target 100, PASS measured. view-grouped-member 121 to 65 and 64 ms, target 100, PASS measured. view-unresolved-reference 120 to 64 and 65 ms, target 100, PASS measured. edit-grouped-group 120 to 1 and 2 ms, target 100, PASS measured. edit-grouped-member 120 to 67 and 64 ms, target 100, PASS measured. edit-unresolved-reference 119 to 64 and 64 ms, target 100, PASS measured. view-single-markdown 39 to 8 and 8 ms, target 100, PASS measured. edit-single-markdown 39 to 8 and 8 ms, target 100, PASS measured. view-single-html-mockup 4 to 1 and 1 ms, target 100, PASS measured. edit-single-html-mockup 4 to 1 and 1 ms, target 100, PASS measured. The 100 and 382 profiles are in the appended tables without threshold verdicts. NFR-007.001 and NFR-007.002 remain unmarked because the reference hardware was not used.
- **rust-dev**: Verification: full suite 1,388 passed, 0 failed, 27 ignored. Cargo clippy all-targets with warnings denied is clean. Cargo fmt check is clean. Characterization replay, list replay, browser snapshot, document field-map and no-drift suites are green. One unrelated CLI environment test, config_editor_json_null_when_unset, fails when EDITOR is set to nano; the suite was run with EDITOR and VISUAL unset as in the recorded run.
- **rust-dev**: Files changed: browser mod, records and render; record frontmatter, migrate, store and view; the MS-036 results document; regenerated ms036 raw JSON. No production behaviour, route, status, header or payload change was observed. Working tree left uncommitted on branch task TASK-313 per instruction. Task status remains In-Progress.
- **orchestrator**: Acceptance met: front-matter-only counts for inventory and grouped indexes, indexes built only when read, single-read snapshot source with fallback, member rows from one projection; at 1000 records inventory p95 550 to 248 ms and grouped routes 118 to 126 down to 63 to 67 ms, all targets met on the available host with the deviation recorded; 1388 tests pass.
