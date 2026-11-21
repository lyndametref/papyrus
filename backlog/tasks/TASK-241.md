---
id: TASK-241
title: "Implement the benchmark harness and record pre-refactor baselines at 100 and 1,000 records"
created_date: '2026-11-20T17:40:57Z'
updated_date: '2026-11-21T16:44:49Z'
labels: []
references:
  - NFR-007.001
  - NFR-007.002
  - NFR-007.003
  - TECH-069
  - TC-178
  - TC-179
  - TC-182
status: ToDo
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
Context: the pre-refactor engine is far outside the NFR-007.001 and NFR-007.002 targets. The only recorded numbers are inventory 14.2 s and `/doc/REQ-07.001` view 1.8 s at 382 active records (MS-032 description; NFR-007.001 and NFR-007.002 rationale). A reproducible harness is needed both to re-measure that baseline and to compare every later milestone against it.

Grounding in the current code:
- The server logs one line per request only in debug mode: `{method} {path} {status} {n}ms` in `src/browser/mod.rs::handle_connection` (`src/browser/mod.rs:317` to `320`, gated on `state.debug`). This is the server-observed latency the NFRs ask for. No timing header exists.
- There is no `benches/` directory, no criterion dependency, and no benchmark target in `Cargo.toml`. Integration tests boot the real binary with `env!("CARGO_BIN_EXE_papyrus")`, a free loopback port, `browser --no-open`, and raw `TcpStream` requests (`tests/list_characterization_capture.rs`, `tests/list_characterization_replay.rs`, `tests/browser_shell_cli.rs`). The harness follows that convention; it adds no crate.
- The server closes every connection after one response (`Connection: close` in `src/browser/http.rs::write_response`), so "sequential requests" means one connection per request.
- Known root causes the baseline must make visible, and that later milestones remove: `src/record/migrate.rs::ensure_front_matter` calls `managed_descriptor`, which calls `load_cfg_for` and rebuilds `ArtifactRegistry` for every record; `src/browser/records.rs::resolve_id` rescans descriptor directories for each referenced ID, making index construction O(n^2); `ServerState::config()` (`src/browser/mod.rs:155`) reloads configuration on every request; no index cache exists.

Scope:

1. Harness `tests/benchmark_baseline.rs`, marked `#[ignore]`, run with `cargo test --release --test benchmark_baseline -- --ignored --nocapture`. It is not part of the default `cargo test` run.

2. Corpora generated into `./.tmp/benchmark/<profile>/` (deleted and recreated per run, never committed): profiles `records-100`, `records-382` and `records-1000`. Generation is deterministic: fixed record bytes, no wall-clock value, the same twelve-type configuration as the characterization corpus, backlog and decision records for route coverage. At 1,000 records the corpus contains one GroupedMarkdown type with a 200-member group and one SingleHtmlMockup type with 50 records; the two smaller profiles use the same mix scaled down. Corpus hash: sha256 over sorted `<relative path>\0<sha256(bytes)>` lines.

3. Per profile: boot one server (`papyrus --dir <corpus> browser --port <free> --no-open --debug`), read the startup line, make one readiness request that is not measured, then measure routes: inventory `/`; view `/doc/NOTE-001` (SingleMarkdown), `/doc/REQ-001` (GroupedMarkdown group), `/doc/REQ-001.001` (grouped member), `/doc/MOCK-001` (SingleHtmlMockup), and the member carrying the unresolved reference; edit `GET /doc/<same ID>/edit` for the same set. Form POST is not measured because it writes records.

4. Protocol per route: 10 warm-up requests, then 100 measured sequential requests. Server-observed latency comes from the debug log line above (millisecond resolution); wall time comes from a client-side `Instant` (microsecond resolution). Report p50 and p95 for both, per route and per profile.

5. Two runs per profile (A and B) in one session, so the recorded variance is the per-route p95 difference between A and B.

6. Evidence of the root causes, without touching `src/`: the recorded inventory p50/p95 curve across the 100, 382 and 1,000 profiles (growth far above linear is the expected signature of the per-record config and registry rebuild plus the O(n^2) index), and a short note in the summary document that cites the four call sites listed above and states which request path exercises each.

7. Output: `docs/benchmarks/MS-032-baseline.md` (method, host, protocol, one table per profile with p50 and p95 for server and wall, the 382-record comparison, and the root-cause note) and `docs/benchmarks/raw/<profile>-run-{a,b}.json` (machine-readable result). Every result records: profile, active record count, corpus hash, exact harness command, git commit, rustc version, build profile (release, LTO and strip), host (CPU model, physical and logical cores, RAM, disk type, kernel, distribution) and the date with timezone.

8. Historical figures: transcribe `inventory 14.2 s` and `/doc/REQ-07.001 view 1.8 s at 382 active records` with their provenance (MS-032; NFR-007.001 and NFR-007.002 rationale), then place the measured `records-382` numbers next to them. The historical corpus is not preserved, so exact equality is not expected and no corpus parameter is tuned to approach the historical value. The `/doc/REQ-07.001` view is represented by `REQ-001` in the generated corpus; say so explicitly.

9. Reference hardware: compare the measured host against the NFR-007.001 description (4 vCPU, 8 GiB RAM, SSD, Linux, one server process, loopback HTTP, release build with LTO and strip) and flag every deviation in the summary document.

10. Headless: no browser process, `--no-open`, no browser automation, no new crate.

Constraints: no production behaviour change. No file under `src/`, `packaging/` or `scripts/` and no `[dependencies]` or `[profile.release]` entry changes. The full `cargo test` suite stays green.

Out of scope: fixing the performance problems (later milestones); changing the server; comparing against post-refactor builds.

Definition of done: three profiles generated and measured twice each, results committed under `docs/benchmarks/`, the 382-record comparison and the root-cause note present, and the default suite green.

## Acceptance Criteria

- [ ] The benchmark harness is an ignored integration test invoked by the documented release command, and the default cargo test run does not execute it.
- [ ] The harness generates deterministic corpora of exactly 100, 382 and 1,000 active records from the same twelve-type configuration, and the 1,000-record profile contains one GroupedMarkdown group with 200 members and 50 SingleHtmlMockup records.
- [ ] Each profile boots one server process with no browser on loopback and measures inventory, document view for all three models plus the unresolved-reference record, and the edit form, with 10 warm-up requests and 100 measured sequential requests per route, excluding the cold start.
- [ ] Server-observed latency is read from the debug request log line in the browser handler and wall time from a client-side monotonic clock, and both p50 and p95 are reported per route and profile.
- [ ] Results are recorded in the benchmark summary document and the raw result files with profile, active record count, corpus hash, exact harness command, git commit, rustc version, build profile with LTO and strip, host description and the date with timezone (MS-032 AC-1, AC-4).
- [ ] The 382-record profile is measured and recorded beside the historical 14.2 s inventory and 1.8 s view figures with their provenance and an explicit statement that the historical corpus is not preserved, and no profile is tuned to match a historical number (MS-032 AC-4).
- [ ] The measured host is compared with the NFR-007.001 reference hardware description and every deviation is flagged.
- [ ] Two runs per profile are recorded and the per-route p95 difference between run A and run B is stated as the recorded variance.
- [ ] The summary document records the inventory p50 and p95 curve across 100, 382 and 1,000 records and cites ensure_front_matter with managed_descriptor and load_cfg_for, resolve_id, the per-request ServerState configuration load, and the absent index cache as the root causes each measured path exercises.
- [ ] Two consecutive runs of one profile produce the same corpus hash and the same corpus file count.
- [ ] No production behaviour change: no file under the source, packaging or scripts directories and no dependency or release-profile entry in the manifest is introduced, and the full suite is green after the harness lands (MS-032 AC-7).

## Comments
- **task-refiner**: Refined by task-refiner for MS-032: description grounded in the current routes, renderers, harness conventions and root-cause call sites; acceptance criteria added; references and documentation set. No dependency, ordinal or milestone change. Status left ToDo because papyrus.config.yml declares statuses [ToDo, In-Progress, Done].
