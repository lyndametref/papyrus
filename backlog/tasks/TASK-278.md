---
id: TASK-278
title: "Re-run the benchmark suite and record NFR-007.001 and NFR-007.002 results"
created_date: '2026-11-20T17:43:20Z'
updated_date: '2026-11-22T21:49:24Z'
labels: []
references:
  - NFR-007.001
  - NFR-007.002
  - NFR-007.003
  - ADR-037
  - RE-010
  - TC-178
  - TC-179
  - TC-181
  - TC-182
  - TC-187
  - TC-189
  - MS-036
  - MS-037
status: Done
assignee:
  - rust-dev
milestone: MS-036
dependencies:
  - TASK-275
  - TASK-277
ordinal: 80
documentation:
  - tests/benchmark_baseline.rs
  - docs/benchmarks/MS-032-baseline.md
  - docs/benchmarks
  - src/browser/mod.rs
  - src/browser/http.rs
  - decisions/ADR-037-in-server-configuration-and-registry-snapshot.md
  - decisions/ADR-038-cacheable-immutable-browser-assets.md
type: implementation
---

## Description
## Context
The milestone exit is numeric, so the post-refactor measurement must be produced the same way as the MS-032 baseline, but with the full NFR-007 protocol. `docs/benchmarks/MS-032-baseline.md` records the pre-refactor engine with the reduced baseline protocol: inventory at 100, 382, and 1,000 records is 1,048, 11,776, and 75,546 ms p50, and the 1,000-record figures are baseline-only because they use 20 samples. MS-032 AC-9 states that MS-036 uses the full NFR-007 protocol, so no reduced-sample p95 from this run enters the acceptance comparison.

## Recorded decisions (planner, 2026-11-22T20:22:22Z)
- Reference hardware: NFR-007.001 and NFR-007.002 name 4 vCPU and 8 GiB RAM, and no such host is available. The only available host has 6 physical / 12 logical cores and 15.3 GiB. This task measures on the available host with the full protocol and records the hardware deviation beside every result. The results are reported as measured figures with the deviation; they do not mark NFR-007.001 or NFR-007.002 satisfied, and MS-036 does not claim either NFR. The release report (MS-037) presents the figures with the deviation as a caveat for the owner, who decides between re-measurement on reference hardware and accepting the deviation. This resolves the open hardware question left in acceptance criterion #2.
- Second-load scope: the repeat-visit measurement stays in this task and covers one page with a renderable diagram, one page with math, and a cache simulation that honours the served Cache-Control and ETag, for NFR-007.003.
- Packaging: MS-036 is an internal checkpoint, so no RC version is determined and no package is retained (MS-036 AC-9); this task produces no packaging artifact.

## Scope
Run `tests/benchmark_baseline.rs` at the 100 and 1,000 record profiles on the available host recorded above, one server process, loopback HTTP, release build with LTO and strip. For every measured inventory, view, and edit route, use the full NFR-007 protocol: 10 warm-up requests followed by 100 measured sequential requests, excluding the first request after a cold start and any request that triggers a configuration reload. Record server-observed p50 and p95 for every route, plus the harness command, commit SHA, rustc version, host CPU and RAM, disk, kernel, build profile, corpus definition, corpus hash, sample counts, and the hardware deviation from the NFR-007 reference hardware.

At 1,000 records the comparison is: inventory p95 against 250 ms (NFR-007.001) and view and edit p95 against 100 ms for every measured document model (NFR-007.002), including SingleMarkdown, the GroupedMarkdown group and member, SingleHtmlMockup, and the record with an unresolved reference. Compare every 1,000-record route with the corresponding MS-032 baseline figure and record an explicit pass or miss against the threshold per route, always beside the hardware deviation. A pass is a measured figure within the threshold on the available host; it is not NFR-007.001 or NFR-007.002 satisfaction. Record the 100-record p50 and p95 for the same routes without treating them as a threshold verdict.

Also record second-load bytes transferred. Load one page with a renderable diagram and one page with math twice under a simulated cache that honours the served Cache-Control and ETag, and record the first-load and second-load body bytes and headers. The second load must transfer zero vendor body bytes; the first-party app assets must revalidate with 304 and empty bodies; HTML and JSON must remain no-store and be re-fetched with bodies. The simulation method, the pages used, and the numbers are recorded with the result.

Keep `Connection: close` as is unless the measurement shows a need. If it does, record the evidence and return to the planner rather than changing connection behaviour in this task. Any threshold miss is reported to the orchestrator and planner with the failing route and numbers; this task does not repair the preceding implementation tasks and does not edit NFR records.

## Assumptions
- The available host is 6 physical / 12 logical cores with 15.3 GiB RAM, SSD, Linux, one server process, loopback HTTP, release build with LTO and strip. The NFR-007 reference hardware of 4 vCPU and 8 GiB is not available; the deviation is recorded beside every result and no NFR is marked satisfied.
- The harness corpus generator and the MS-032 raw JSON remain the comparison basis; no corpus parameter is tuned to approach a target.

## Definition of done
Every acceptance criterion passes; the run is reproducible from the recorded command and corpus hash; the full NFR-007 sample counts are recorded per route; the threshold comparison per route, the hardware deviation beside every result, and the second-load cache evidence are recorded on this task; no NFR-007.001 or NFR-007.002 satisfaction is claimed; no source or behaviour change is made; evidence is recorded.
## Acceptance Criteria
- [ ] #1 Harness run recorded: the MS-032 harness command, release build profile with LTO and strip, one server process, loopback HTTP, commit SHA, rustc version, host CPU and RAM, disk, kernel, and date are recorded with the results.
- [ ] #2 Reference hardware: the run uses the NFR-007 reference hardware of 4 vCPU, 8 GiB RAM, SSD, Linux, one server process, loopback HTTP, and a release build with LTO and strip; if a different host is used, the deviation is recorded and the result is not presented as an NFR pass until the planner resolves the hardware question.
- [ ] #3 Corpus: the 100 and 1,000 active-record profiles are used; the 1,000 profile includes a GroupedMarkdown type with a 200-member group and a SingleHtmlMockup type with 50 records; the corpus definition, file count, and hash are recorded and the deterministic corpus is regenerated by the harness.
- [ ] #4 Full protocol: every measured inventory, view, and edit route performs 10 warm-up requests followed by 100 measured sequential requests; the first request after a cold start and any request that triggers a configuration reload are excluded; the recorded output shows 10 warm-ups and 100 measured samples for every route, and a route with fewer than 100 measured samples is not compared against the NFR and blocks this task.
- [ ] #5 Inventory threshold: at 1,000 records the server-observed inventory p95 is below 250 ms; p50 and p95 are recorded.
- [ ] #6 View and edit threshold: at 1,000 records the server-observed p95 is below 100 ms for every measured document model, covering SingleMarkdown, GroupedMarkdown group, GroupedMarkdown member, SingleHtmlMockup, and the record with an unresolved reference; p50 and p95 are recorded for view and for edit.
- [ ] #7 100-record results: server-observed p50 and p95 are recorded for the same inventory, view, and edit routes at 100 records and are compared with the baseline without being presented as a threshold verdict.
- [ ] #8 Baseline comparison: every 1,000-record route is compared with its MS-032 baseline figure, including the inventory baseline of 75,546 ms p50 and 76,532 ms p95, and an explicit pass or fail against the NFR threshold is recorded per route.
- [ ] #9 Second-load transfer: one page with a renderable diagram and one page with math are loaded twice under a cache that honours Cache-Control and ETag; the first-load and second-load body bytes and headers are recorded; the second load transfers zero vendor body bytes, the first-party app assets revalidate with 304 and empty bodies, and HTML and JSON remain no-store and are re-fetched with bodies.
- [ ] #10 Connection behaviour: Connection close remains on every response; if the measurement indicates a need for keep-alive, the evidence is recorded and the question is returned to the planner without changing connection behaviour in this task.
- [ ] #11 No functional change: this task measures only; any implementation fix belongs to an earlier task or the planner; the full suite, cargo clippy --all-targets -- -D warnings, and cargo fmt --check are clean.
- [ ] #12 Results recorded on this task: command, corpus and hash, host, sample counts, per-route p50 and p95, NFR pass or fail, baseline comparison, second-load cache numbers, and the measurement method are recorded so the run is reproducible; any miss is reported to the orchestrator and planner and is not hidden or presented as a baseline-only pass.
- [ ] #13 Hardware deviation decision (resolves and supersedes the reference-hardware clause of AC 2): the run uses the available host of 6 physical and 12 logical cores with 15.3 GiB RAM instead of the 4 vCPU and 8 GiB NFR-007 reference hardware; the full 10 warm-up and 100 measured protocol is unchanged; the hardware deviation is recorded beside every route result and in the run header; each threshold comparison is recorded as a measured pass or miss on the available host and is never presented as satisfying NFR-007.001 or NFR-007.002; the milestone does not mark either NFR satisfied, and the figures with the deviation are handed to MS-037 for the release report and the owner decision.
## Comments
- **planner**: planner: hardware decision, 2026-11-22T20:22:22Z. NFR-007.001 and NFR-007.002 name 4 vCPU and 8 GiB; the available host is 6 physical and 12 logical cores with 15.3 GiB. Measure on the available host with the full 10 warm-up and 100 measured protocol, record the deviation beside every result, report measured figures with the deviation, and do not mark either NFR satisfied (AC #13 supersedes the reference-hardware clause of AC #2). MS-037 presents the figures with the caveat for the owner. Second-load measurement stays in this task: one diagram page, one math page, and a cache simulation honouring the served Cache-Control and ETag, for NFR-007.003.
- **orchestrator**: Acceptance met: full NFR-007 protocol measured on the available host with the deviation recorded; inventory 1000 p95 550 ms (miss vs 250), grouped view and edit 118 to 126 ms (miss vs 100), single-markdown 38 ms and mockup 4 ms pass, NFR-007.003 pass with zero second-load vendor bytes; raw JSON and MS-036-results.md recorded.
- **rust-dev**: test
