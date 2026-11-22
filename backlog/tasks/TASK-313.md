---
id: TASK-313
title: "Optimise the read path to meet the NFR-007 targets at 1,000 records"
created_date: '2026-11-22T21:49:04Z'
updated_date: '2026-11-22T21:49:04Z'
labels:
  - performance
references:
  - NFR-007.001
  - NFR-007.002
  - ADR-037
  - MS-036
status: ToDo
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
