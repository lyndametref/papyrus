---
id: TASK-241
title: "Implement the benchmark harness and record pre-refactor baselines at 100 and 1,000 records"
created_date: '2026-11-20T17:40:57Z'
updated_date: '2026-11-22T01:41:09Z'
labels: []
references:
  - NFR-007.001
  - NFR-007.002
  - NFR-007.003
  - TECH-069
  - TC-178
  - TC-179
  - TC-182
status: Done
assignee:
  - rust-dev
milestone: MS-032
dependencies:
  - TASK-240
ordinal: 40
documentation:
  - docs/benchmarks
type: implementation
---

## Description
Context: the pre-refactor engine is far outside the NFR-007.001 and NFR-007.002 targets. The only recorded numbers are inventory 14.2 s and `/doc/REQ-007.001` view 1.8 s at 382 active records (MS-032 description; NFR-007.001 and NFR-007.002 rationale). A reproducible harness is needed both to re-measure that baseline and to compare every later milestone against it.

Grounding in the current code:
- The server logs one line per request only in debug mode: `{method} {path} {status} {n}ms` in `src/browser/mod.rs::handle_connection` (`src/browser/mod.rs:317` to `320`, gated on `state.debug`). This is the server-observed latency the NFRs ask for. No timing header exists.
- There is no `benches/` directory, no criterion dependency, and no benchmark target in `Cargo.toml`. Integration tests boot the real binary with `env!("CARGO_BIN_EXE_papyrus")`, a free loopback port, `browser --no-open`, and raw `TcpStream` requests (`tests/list_characterization_capture.rs`, `tests/list_characterization_replay.rs`, `tests/browser_shell_cli.rs`). The harness follows that convention; it adds no crate.
- The server closes every connection after one response (`Connection: close` in `src/browser/http.rs::write_response`), so "sequential requests" means one connection per request.
- Known root causes the baseline must make visible, and that later milestones remove: `src/record/migrate.rs::ensure_front_matter` calls `managed_descriptor`, which calls `load_cfg_for` and rebuilds `ArtifactRegistry` for every record; `src/browser/records.rs::resolve_id` rescans descriptor directories for each referenced ID, making index construction O(n^2); `ServerState::config()` (`src/browser/mod.rs:155`) reloads configuration on every request; no index cache exists.

Scope:

1. Harness `tests/benchmark_baseline.rs`, marked `#[ignore]`, run with `cargo test --release --test benchmark_baseline -- --ignored --nocapture`. It is not part of the default `cargo test` run.

2. Corpora generated into `./.tmp/benchmark/<profile>/` (deleted and recreated per run, never committed): profiles `records-100`, `records-382` and `records-1000`. Generation is deterministic: fixed record bytes, no wall-clock value, the same twelve-type configuration as the characterization corpus, backlog and decision records for route coverage. At 1,000 records the corpus contains one GroupedMarkdown type with a 200-member group and one SingleHtmlMockup type with 50 records; the two smaller profiles use the same mix scaled down. Corpus hash: sha256 over sorted `<relative path>\0<sha256(bytes)>` lines.

3. Per profile: boot one server (`papyrus --dir <corpus> browser --port <free> --no-open --debug`), read the startup line, make one readiness request that is not measured, then measure routes: inventory `/`; view `/doc/NOTE-001` (SingleMarkdown), `/doc/REQ-001` (GroupedMarkdown group), `/doc/REQ-001.001` (grouped member), `/doc/MOCK-001` (SingleHtmlMockup), and the member carrying the unresolved reference; edit `GET /doc/<same ID>/edit` for the same set. Form POST is not measured because it writes records.

4. Protocol per route: 10 warm-up requests, then 100 measured sequential requests (band A), reduced to 2 warm-up and 20 measured requests where the route p50 is at or above 1 s (band B), per the reduced-sample protocol below. Server-observed latency comes from the debug log line above (millisecond resolution); wall time comes from a client-side `Instant` (microsecond resolution). Report p50 and p95 for both, per route and per profile, each beside the measured count it was computed from.

Reduced-sample protocol (MS-032 amendment): the full NFR-007 protocol of 10 warm-up + 100 measured requests per route does not complete on the pre-refactor engine. Measured reconnaissance on the benchmark host: inventory 1.04 s at 100 records, 11.36 s at 382, 74.37 s at 1,000; grouped-member view 23.90 s at 1,000; the full protocol over 11 routes, 2 runs and 3 profiles would run for roughly 15 to 16 hours. Bands:
- Band A, route p50 below 1 s: 10 warm-up + 100 measured requests.
- Band B, route p50 at or above 1 s: 2 warm-up + 20 measured requests.
The first five measured samples of run A decide the band per route and profile, count toward the total, and run B reuses the same band and counts. p50 and p95 are reported for every route and profile; a p95 from fewer than 100 samples is labelled baseline-only and is never compared against the NFR-007.001 or NFR-007.002 thresholds. MS-036 keeps the full NFR-007 protocol for the post-refactor measurement, so this deviation never enters the acceptance comparison.

5. Two runs per profile (A and B) in one session, so the recorded variance is the per-route p95 difference between A and B.

6. Evidence of the root causes, without touching `src/`: the recorded inventory p50/p95 curve across the 100, 382 and 1,000 profiles (growth far above linear is the expected signature of the per-record config and registry rebuild plus the O(n^2) index), and a short note in the summary document that cites the four call sites listed above and states which request path exercises each.

7. Output: `docs/benchmarks/MS-032-baseline.md` (method, host, protocol, sample counts, one table per profile with p50 and p95 for server and wall, the 382-record comparison, and the root-cause note) and `docs/benchmarks/raw/<profile>-run-{a,b}.json` (machine-readable result). Every result records: profile, active record count, corpus hash, exact harness command, git commit, rustc version, build profile (release, LTO and strip), host (CPU model, physical and logical cores, RAM, disk type, kernel, distribution) and the date with timezone. The sample-count table records, per route and profile, the band, the warm-up count, the measured count, and the reason for any count below the full protocol. The summary states that MS-036 measures with the full NFR-007 protocol and that the reduced counts are baseline-only.

8. Historical figures: transcribe `inventory 14.2 s` and `/doc/REQ-007.001 view 1.8 s at 382 active records` with their provenance (MS-032; NFR-007.001 and NFR-007.002 rationale), then place the measured `records-382` numbers next to them. The historical corpus is not preserved, so exact equality is not expected and no corpus parameter is tuned to approach the historical value. The `/doc/REQ-007.001` view is represented by `REQ-001` in the generated corpus; say so explicitly.

9. Reference hardware: compare the measured host against the NFR-007.001 description (4 vCPU, 8 GiB RAM, SSD, Linux, one server process, loopback HTTP, release build with LTO and strip) and flag every deviation in the summary document.

10. Headless: no browser process, `--no-open`, no browser automation, no new crate.

Constraints: no production behaviour change. No file under `src/`, `packaging/` or `scripts/` and no `[dependencies]` or `[profile.release]` entry changes. The full `cargo test` suite stays green.

Out of scope: fixing the performance problems (later milestones); changing the server; comparing against post-refactor builds.

Definition of done: three profiles generated and measured twice each at the sample counts of the reduced-sample protocol, results committed under `docs/benchmarks/`, the sample-count table and its reason present, the MS-036 full-protocol note present, the 382-record comparison and the root-cause note present, and the default suite green.
## Acceptance Criteria

- [ ] The benchmark harness is an ignored integration test invoked by the documented release command, and the default cargo test run does not execute it.
- [ ] The harness generates deterministic corpora of exactly 100, 382 and 1,000 active records from the same twelve-type configuration, and the 1,000-record profile contains one GroupedMarkdown group with 200 members and 50 SingleHtmlMockup records.
- [ ] Each profile boots one server process with no browser on loopback and measures inventory, document view for all three models plus the unresolved-reference record, and the edit form, excluding the cold start, with the per-route sample counts of the reduced-sample protocol: 10 warm-up + 100 measured where the route p50 is below 1 s, 2 warm-up + 20 measured where it is at or above 1 s.
- [ ] Server-observed latency is read from the debug request log line in the browser handler and wall time from a client-side monotonic clock, and both p50 and p95 are reported per route and profile.
- [ ] Results are recorded in the benchmark summary document and the raw result files with profile, active record count, corpus hash, exact harness command, git commit, rustc version, build profile with LTO and strip, host description and the date with timezone (MS-032 AC-1, AC-4).
- [ ] The 382-record profile is measured and recorded beside the historical 14.2 s inventory and 1.8 s view figures with their provenance and an explicit statement that the historical corpus is not preserved, and no profile is tuned to match a historical number (MS-032 AC-4).
- [ ] The measured host is compared with the NFR-007.001 reference hardware description and every deviation is flagged.
- [ ] Two runs per profile are recorded and the per-route p95 difference between run A and run B is stated as the recorded variance.
- [ ] The summary document records the inventory p50 and p95 curve across 100, 382 and 1,000 records and cites ensure_front_matter with managed_descriptor and load_cfg_for, resolve_id, the per-request ServerState configuration load, and the absent index cache as the root causes each measured path exercises.
- [ ] Two consecutive runs of one profile produce the same corpus hash and the same corpus file count.
- [ ] No production behaviour change: no file under the source, packaging or scripts directories and no dependency or release-profile entry in the manifest is introduced, and the full suite is green after the harness lands (MS-032 AC-7).
- [ ] Band selection and reporting are deterministic: the first five measured samples of run A select the band per route and profile and count toward the total, run B reuses the same band and counts, and p50 and p95 are reported beside the measured count they were computed from. The summary document and the raw result files record band, warm-up count and measured count per route and profile (MS-032 AC-8).
- [ ] The summary document states that the reduced sample counts come from pre-refactor per-request latency (74.37 s per inventory request at 1,000 records), not measurement uncertainty; that a p95 from fewer than 100 samples is baseline-only and is never compared against the NFR-007.001 or NFR-007.002 thresholds; and that the MS-036 post-refactor measurement uses the full NFR-007 protocol of 10 warm-up + 100 measured requests per route, so the baseline deviation never enters the acceptance comparison. NFR-007.001 to NFR-007.003 are unchanged (MS-032 AC-9).

## Comments
- **task-refiner**: Refined by task-refiner for MS-032: description grounded in the current routes, renderers, harness conventions and root-cause call sites; acceptance criteria added; references and documentation set. No dependency, ordinal or milestone change. Status left ToDo because papyrus.config.yml declares statuses [ToDo, In-Progress, Done].
- **planner**: Planner amendment for MS-032 AC-1 and AC-4 plus the new AC-8 and AC-9: the pre-refactor baseline uses a reduced-sample protocol. Band A is 10 warm-up and 100 measured requests where the route p50 is below 1 s; band B is 2 warm-up and 20 measured requests at or above 1 s; the first five measured samples of run A decide the band and run B reuses it. Reason: the full NFR-007 protocol needs roughly 15 to 16 hours on the pre-refactor engine, whose per-request latency is the defect MS-036 removes. p50 and p95 stay mandatory; a p95 from fewer than 100 samples is baseline-only and is never compared against the NFR-007 thresholds. MS-036 keeps the full NFR-007 protocol and NFR-007.001 to NFR-007.003 are unchanged. No ordinal, dependency, milestone or assignee change; status left In-Progress.
- **rust-dev**: TASK-241 implementation evidence (rust-dev). Delivered tests benchmark_baseline.rs (ignored release harness), docs benchmarks MS-032-baseline.md and raw JSON. Reduced-sample protocol: band A 10 warm-up + 100 measured below 1 s, band B 2 warm-up + 20 measured at or above 1 s; the first five run A samples choose the band and count, and run B reuses band and counts. Owner-approved deviation: records-1000 run A only; its single-run p95 is baseline-only; 100 and 382 have two runs. records-1000 band-B routes limited to the representative set (inventory, grouped-member view, single-markdown view, one edit, mockup view); six omitted routes are recorded and never extrapolated. Host Intel i7-9750H, 6 physical and 12 logical cores, 15.3 GiB RAM, NVMe SSD, Debian forky sid, kernel 7.2.15; release lto and strip; rustc 1.99.1; commit c34cf77. Baseline-only: any p95 under 100 samples is never compared with NFR-007.001 or NFR-007.002; reduced counts come from pre-refactor latency (74.37 s inventory at 1000), not measurement uncertainty; MS-036 uses the full 10+100 protocol; NFR-007.001 to NFR-007.003 unchanged. Root causes evidenced: ensure_front_matter with managed_descriptor and load_cfg_for plus family conflict rescan; resolve_id directory rescans; per-request ServerState config load; no index cache. Inventory p50 1041, 11756 and 75682 ms at 100, 382 and 1000. Corpus hashes 7b88012297517150ed40e5fc1cbc6b4f74e5915aea0f6e61f92155d514e5404e, 16852d24b236c29d9a109ec5c706645bf3ab5a60615c120a3dcf6e76cb816a15 and 46d64349137f7017f6ee04bcfd6a3d3042b7b6bc23245749dac400804581988f. Suite 1136 passed, 0 failed, 25 ignored; clippy and fmt clean. No src, packaging or scripts change.
- **rust-dev**: records-100 table, 2 runs, server p50 and p95 ms with band and warmup+measured: inventory B 2+20 1041 1051 baseline-only; view-single-markdown A 10+100 44 46; view-grouped-group A 10+100 309 322; view-grouped-member A 10+100 339 345; view-single-html-mockup A 10+100 1 1; view-unresolved-reference A 10+100 340 348; edit-single-markdown A 10+100 44 45; edit-grouped-group A 10+100 312 319; edit-grouped-member A 10+100 341 353; edit-single-html-mockup A 10+100 1 1; edit-unresolved-reference A 10+100 343 351.
- **rust-dev**: records-382 table, 2 runs, server p50 and p95 ms with band and warmup+measured: inventory B 2+20 11756 11975 baseline-only; view-single-markdown A 10+100 554 566; view-grouped-group B 2+20 3566 3613 baseline-only; view-grouped-member B 2+20 3682 3718 baseline-only; view-single-html-mockup A 10+100 2 2; view-unresolved-reference B 2+20 3693 3727 baseline-only; edit-single-markdown A 10+100 554 568; edit-grouped-group B 2+20 3514 3612 baseline-only; edit-grouped-member B 2+20 3684 3723 baseline-only; edit-single-html-mockup A 10+100 1 1; edit-unresolved-reference B 2+20 3648 3730 baseline-only.
- **rust-dev**: records-1000 table, single run A, server p50 and p95 ms with band and warmup+measured: inventory B 2+20 75682 76507 baseline-only; view-single-markdown B 2+20 2980 2994 baseline-only; view-grouped-member B 2+20 23746 24154 baseline-only; view-single-html-mockup A 10+100 2 2; edit-grouped-member B 2+20 23787 24008 baseline-only. Omitted: view-grouped-group, view-unresolved-reference, edit-single-markdown, edit-grouped-group, edit-single-html-mockup, edit-unresolved-reference. The single-run p95 is baseline-only and is never compared with the NFR-007 thresholds.
- **orchestrator**: Acceptance met: benchmark harness and deterministic 100, 382 and 1000-record corpora, pre-refactor baselines recorded with band and sample counts (inventory p50 1041 ms, 11756 ms and 75682 ms), 1000-profile run-A-only deviation recorded, root causes evidenced, suite green with 1136 tests passing.
- **rust-dev**: TASK-292 C-5 rationale for the TASK-241 test infrastructure changes: the project-empty fixture deletions restore the documented invariant that the pinned empty project contains only papyrus.config.yml and no record files; the three removed files had arrived in unrelated commits.
- **rust-dev**: TASK-292 C-5 rationale for the TASK-241 test infrastructure changes: version-token normalisation maps the mtime-derived token in edit and conflict pages and JSON payloads to the {{VERSION}} placeholder, because the token changes on every copy and would make byte replay flaky.
