# MS-032 pre-refactor benchmark baseline

Status: recorded (TASK-241). This is the pre-refactor baseline; no production behaviour was changed. It uses the reduced-sample protocol of MS-032 AC-8 because the full NFR-007 protocol does not complete on the pre-refactor engine.

- Dates: `2026-11-21T20:28:21+01:00` to `2026-11-21T21:49:20+01:00`

- Git commit: `c34cf77a209d8af09b759b4534e146fb617cc2aa`

- rustc: `rustc 1.99.1 (58cf8f9c3 2026-10-13)`

- Build profile: release (lto=true, strip=true)

- Harness command: `cargo test --release --test benchmark_baseline -- --ignored --nocapture`

## Baseline-only status

The reduced sample counts come from pre-refactor per-request latency, not measurement uncertainty: the recorded reconnaissance measured 74.37 s per inventory request at 1,000 records and 23.90 s per grouped-member view at 1,000 records. A p95 computed from fewer than 100 samples is **baseline-only** and is never compared against the NFR-007.001 or NFR-007.002 thresholds. MS-036 measures the post-refactor engine with the full NFR-007 protocol of 10 warm-up + 100 measured requests per route, so this baseline deviation never enters the acceptance comparison. NFR-007.001 to NFR-007.003 are unchanged.

Owner-approved deviation for `records-1000`: this profile was measured with a single run (run A only) because the pre-refactor engine needs roughly 75 s per inventory request at 1,000 records (measured p50 75,682 ms), which makes a second run impractical. Its single-run p95 is baseline-only and is never compared against the NFR-007.001 or NFR-007.002 thresholds. The 100 and 382 profiles keep two runs (A and B).

## Host

Intel(R) Core(TM) i7-9750H CPU @ 2.60GHz with 6 physical and 12 logical cores, 15.3 GiB RAM, SSD (mapper/linux-root), kernel `Linux 7.2.15+deb14-amd64 x86_64`, Debian GNU/Linux forky/sid.

## Method

The harness generates a deterministic Papyrus project under `./.tmp/benchmark/<profile>/` (deleted and recreated for every run), boots one real `papyrus --dir <corpus> browser --port <free> --no-open --debug` process on `127.0.0.1`, performs one unmeasured readiness request, then measures every measured route with the protocol below. Every request uses its own loopback TCP connection because the server sends `Connection: close`.

Band selection is deterministic. Run A performs two common warm-ups, then the first five measured samples. Their p50 selects band A (p50 < 1000 ms, 10 warm-up + 100 measured) or band B (p50 >= 1000 ms, 2 warm-up + 20 measured); those five samples count toward the measured total. Band A then tops up to ten warm-ups before the remaining samples. Run B reuses the same band, warm-up count and measured count. Percentiles are nearest-rank over the measured samples.

- Server-observed latency: the `{method} {path} {status} {n}ms` line printed by `src/browser/mod.rs::handle_connection` when `--debug` is set. Millisecond resolution.
- Wall time: client-side `std::time::Instant` around connect, request, and read. Microsecond resolution.
- Routes: inventory `/`; SingleMarkdown view/edit (`NOTE-001`); GroupedMarkdown group and member view/edit (`REQ-001`, `REQ-001.001`); SingleHtmlMockup view/edit (`MOCK-001`); and the member with the unresolved group reference (`REQ-001.004`). No form POST is sent because it writes records.
- At 1,000 records the band-B routes are limited to the representative set: inventory, one grouped-member view, one SingleMarkdown view, one edit, and the mockup view. Missing routes are listed as omitted and are never extrapolated.
- `records-1000` was measured with run A only (owner-approved); the 100 and 382 profiles were measured twice.

## Corpora

| Profile | Active records | Files (records + config) | Corpus hash (A = B) | Composition |
|---|---|---|---|---|
| `records-100` | 100 | 101 | `7b88012297517150ed40e5fc1cbc6b4f74e5915aea0f6e61f92155d514e5404e` | tasks 14, milestones 5, releases 2, decisions 5, requirements 21 (1 group + 20 members), UI mockups 5, NFRs 6 (1 group + 5 members), audit 4, glossary 6, rules 6, tech specs 6, test cases 5, test reports 3, UI specs 3, use cases 4, notes 5 |
| `records-382` | 382 | 383 | `16852d24b236c29d9a109ec5c706645bf3ab5a60615c120a3dcf6e76cb816a15` | tasks 53, milestones 19, releases 8, decisions 19, requirements 77 (1 group + 76 members), UI mockups 19, NFRs 20 (1 group + 19 members), audit 15, glossary 23, rules 23, tech specs 23, test cases 19, test reports 11, UI specs 11, use cases 15, notes 27 |
| `records-1000` | 1000 | 1001 | `46d64349137f7017f6ee04bcfd6a3d3042b7b6bc23245749dac400804581988f` | tasks 140, milestones 50, releases 20, decisions 50, requirements 201 (1 group + 200 members), UI mockups 50, NFRs 51 (1 group + 50 members), audit 40, glossary 60, rules 60, tech specs 60, test cases 50, test reports 30, UI specs 30, use cases 40, notes 68 |

The files count includes the configuration file, so it is active records + 1. Each profile was regenerated for every run and reproduced the same hash and file count.

## Sample counts and bands

Every count below the full 10 warm-up + 100 measured protocol is band B and is caused by pre-refactor per-request latency, not measurement uncertainty. A `*` on a p95 later in this document marks a baseline-only figure computed from fewer than 100 samples.

### `records-100` (100 active records, 2 runs)

| Route | Band | Warm-up | Measured | Reason |
|---|---|---|---|---|
| inventory | B | 2 | 20 | band B: p50 >= 1000 ms before reduction; 20 measured < 100 full-protocol samples because of pre-refactor per-request latency, not measurement uncertainty |
| view-single-markdown | A | 10 | 100 | band A: full NFR-007 protocol (10 warm-up + 100 measured) |
| view-grouped-group | A | 10 | 100 | band A: full NFR-007 protocol (10 warm-up + 100 measured) |
| view-grouped-member | A | 10 | 100 | band A: full NFR-007 protocol (10 warm-up + 100 measured) |
| view-single-html-mockup | A | 10 | 100 | band A: full NFR-007 protocol (10 warm-up + 100 measured) |
| view-unresolved-reference | A | 10 | 100 | band A: full NFR-007 protocol (10 warm-up + 100 measured) |
| edit-single-markdown | A | 10 | 100 | band A: full NFR-007 protocol (10 warm-up + 100 measured) |
| edit-grouped-group | A | 10 | 100 | band A: full NFR-007 protocol (10 warm-up + 100 measured) |
| edit-grouped-member | A | 10 | 100 | band A: full NFR-007 protocol (10 warm-up + 100 measured) |
| edit-single-html-mockup | A | 10 | 100 | band A: full NFR-007 protocol (10 warm-up + 100 measured) |
| edit-unresolved-reference | A | 10 | 100 | band A: full NFR-007 protocol (10 warm-up + 100 measured) |

### `records-382` (382 active records, 2 runs)

| Route | Band | Warm-up | Measured | Reason |
|---|---|---|---|---|
| inventory | B | 2 | 20 | band B: p50 >= 1000 ms before reduction; 20 measured < 100 full-protocol samples because of pre-refactor per-request latency, not measurement uncertainty |
| view-single-markdown | A | 10 | 100 | band A: full NFR-007 protocol (10 warm-up + 100 measured) |
| view-grouped-group | B | 2 | 20 | band B: p50 >= 1000 ms before reduction; 20 measured < 100 full-protocol samples because of pre-refactor per-request latency, not measurement uncertainty |
| view-grouped-member | B | 2 | 20 | band B: p50 >= 1000 ms before reduction; 20 measured < 100 full-protocol samples because of pre-refactor per-request latency, not measurement uncertainty |
| view-single-html-mockup | A | 10 | 100 | band A: full NFR-007 protocol (10 warm-up + 100 measured) |
| view-unresolved-reference | B | 2 | 20 | band B: p50 >= 1000 ms before reduction; 20 measured < 100 full-protocol samples because of pre-refactor per-request latency, not measurement uncertainty |
| edit-single-markdown | A | 10 | 100 | band A: full NFR-007 protocol (10 warm-up + 100 measured) |
| edit-grouped-group | B | 2 | 20 | band B: p50 >= 1000 ms before reduction; 20 measured < 100 full-protocol samples because of pre-refactor per-request latency, not measurement uncertainty |
| edit-grouped-member | B | 2 | 20 | band B: p50 >= 1000 ms before reduction; 20 measured < 100 full-protocol samples because of pre-refactor per-request latency, not measurement uncertainty |
| edit-single-html-mockup | A | 10 | 100 | band A: full NFR-007 protocol (10 warm-up + 100 measured) |
| edit-unresolved-reference | B | 2 | 20 | band B: p50 >= 1000 ms before reduction; 20 measured < 100 full-protocol samples because of pre-refactor per-request latency, not measurement uncertainty |

### `records-1000` (1000 active records, 1 run)

| Route | Band | Warm-up | Measured | Reason |
|---|---|---|---|---|
| inventory | B | 2 | 20 | band B: p50 >= 1000 ms before reduction; 20 measured < 100 full-protocol samples because of pre-refactor per-request latency, not measurement uncertainty |
| view-single-markdown | B | 2 | 20 | band B: p50 >= 1000 ms before reduction; 20 measured < 100 full-protocol samples because of pre-refactor per-request latency, not measurement uncertainty |
| view-grouped-member | B | 2 | 20 | band B: p50 >= 1000 ms before reduction; 20 measured < 100 full-protocol samples because of pre-refactor per-request latency, not measurement uncertainty |
| view-single-html-mockup | A | 10 | 100 | band A: full NFR-007 protocol (10 warm-up + 100 measured) |
| edit-grouped-member | B | 2 | 20 | band B: p50 >= 1000 ms before reduction; 20 measured < 100 full-protocol samples because of pre-refactor per-request latency, not measurement uncertainty |

Omitted at this profile (no measurement, no extrapolation):

| Route | Path | Reason |
|---|---|---|
| view-grouped-group | `/doc/REQ-001` | band-B representative-set limit at 1,000 records |
| view-unresolved-reference | `/doc/REQ-001.004` | band-B representative-set limit at 1,000 records |
| edit-single-markdown | `/doc/NOTE-001/edit` | band-B representative-set limit at 1,000 records |
| edit-grouped-group | `/doc/REQ-001/edit` | band-B representative-set limit at 1,000 records |
| edit-single-html-mockup | `/doc/MOCK-001/edit` | band-B representative-set limit at 1,000 records |
| edit-unresolved-reference | `/doc/REQ-001.004/edit` | band-B representative-set limit at 1,000 records |

## Results

Run A is shown for every profile. `run B` variance is shown for the 100 and 382 profiles; `records-1000` has a single run (owner-approved). A `*` on a p95 marks a baseline-only figure computed from fewer than 100 samples.

### `records-100` (100 active records)

Run A, measured latency per route:

| Route | Band | n | Server p50 (ms) | Server p95 (ms) | Wall p50 (ms) | Wall p95 (ms) |
|---|---|---|---|---|---|---|
| inventory | B | 20 | 1041 | 1051 * | 1041.716 | 1052.076 * |
| view-single-markdown | A | 100 | 44 | 46 | 45.180 | 46.537 |
| view-grouped-group | A | 100 | 309 | 322 | 309.907 | 323.023 |
| view-grouped-member | A | 100 | 339 | 345 | 339.400 | 345.943 |
| view-single-html-mockup | A | 100 | 1 | 1 | 2.129 | 2.177 |
| view-unresolved-reference | A | 100 | 340 | 348 | 340.913 | 348.310 |
| edit-single-markdown | A | 100 | 44 | 45 | 45.051 | 45.651 |
| edit-grouped-group | A | 100 | 312 | 319 | 312.428 | 319.488 |
| edit-grouped-member | A | 100 | 341 | 353 | 342.112 | 353.655 |
| edit-single-html-mockup | A | 100 | 1 | 1 | 1.217 | 1.568 |
| edit-unresolved-reference | A | 100 | 343 | 351 | 343.963 | 351.720 |

Recorded variance (run B minus run A):

| Route | n | Server p95 A (ms) | Server p95 B (ms) | Server Δ p95 (ms) | Wall p95 A (ms) | Wall p95 B (ms) | Wall Δ p95 (ms) |
|---|---|---|---|---|---|---|---|
| inventory | 20 | 1051 | 1099 | +48 | 1052.076 | 1099.352 | +47276 |
| view-single-markdown | 100 | 46 | 45 | -1 | 46.537 | 46.151 | -386 |
| view-grouped-group | 100 | 322 | 319 | -3 | 323.023 | 319.270 | -3753 |
| view-grouped-member | 100 | 345 | 351 | +6 | 345.943 | 351.846 | +5903 |
| view-single-html-mockup | 100 | 1 | 3 | +2 | 2.177 | 3.801 | +1624 |
| view-unresolved-reference | 100 | 348 | 352 | +4 | 348.310 | 352.354 | +4044 |
| edit-single-markdown | 100 | 45 | 46 | +1 | 45.651 | 46.828 | +1177 |
| edit-grouped-group | 100 | 319 | 319 | +0 | 319.488 | 319.679 | +191 |
| edit-grouped-member | 100 | 353 | 352 | -1 | 353.655 | 352.960 | -695 |
| edit-single-html-mockup | 100 | 1 | 1 | +0 | 1.568 | 2.094 | +526 |
| edit-unresolved-reference | 100 | 351 | 350 | -1 | 351.720 | 350.645 | -1075 |

### `records-382` (382 active records)

Run A, measured latency per route:

| Route | Band | n | Server p50 (ms) | Server p95 (ms) | Wall p50 (ms) | Wall p95 (ms) |
|---|---|---|---|---|---|---|
| inventory | B | 20 | 11756 | 11975 * | 11757.041 | 11975.744 * |
| view-single-markdown | A | 100 | 554 | 566 | 554.889 | 566.828 |
| view-grouped-group | B | 20 | 3566 | 3613 * | 3566.538 | 3613.680 * |
| view-grouped-member | B | 20 | 3682 | 3718 * | 3682.604 | 3718.703 * |
| view-single-html-mockup | A | 100 | 2 | 2 | 2.424 | 2.479 |
| view-unresolved-reference | B | 20 | 3693 | 3727 * | 3693.483 | 3727.582 * |
| edit-single-markdown | A | 100 | 554 | 568 | 554.662 | 568.219 |
| edit-grouped-group | B | 20 | 3514 | 3612 * | 3514.757 | 3612.386 * |
| edit-grouped-member | B | 20 | 3684 | 3723 * | 3685.142 | 3723.469 * |
| edit-single-html-mockup | A | 100 | 1 | 1 | 2.040 | 2.104 |
| edit-unresolved-reference | B | 20 | 3648 | 3730 * | 3648.460 | 3730.727 * |

Recorded variance (run B minus run A):

| Route | n | Server p95 A (ms) | Server p95 B (ms) | Server Δ p95 (ms) | Wall p95 A (ms) | Wall p95 B (ms) | Wall Δ p95 (ms) |
|---|---|---|---|---|---|---|---|
| inventory | 20 | 11975 | 12125 | +150 | 11975.744 | 12125.568 | +149824 |
| view-single-markdown | 100 | 566 | 565 | -1 | 566.828 | 565.868 | -960 |
| view-grouped-group | 20 | 3613 | 3630 | +17 | 3613.680 | 3630.725 | +17045 |
| view-grouped-member | 20 | 3718 | 3758 | +40 | 3718.703 | 3759.146 | +40443 |
| view-single-html-mockup | 100 | 2 | 1 | -1 | 2.479 | 1.991 | -488 |
| view-unresolved-reference | 20 | 3727 | 3759 | +32 | 3727.582 | 3760.139 | +32557 |
| edit-single-markdown | 100 | 568 | 567 | -1 | 568.219 | 567.537 | -682 |
| edit-grouped-group | 20 | 3612 | 3644 | +32 | 3612.386 | 3644.505 | +32119 |
| edit-grouped-member | 20 | 3723 | 3756 | +33 | 3723.469 | 3756.344 | +32875 |
| edit-single-html-mockup | 100 | 1 | 2 | +1 | 2.104 | 2.450 | +346 |
| edit-unresolved-reference | 20 | 3730 | 3783 | +53 | 3730.727 | 3783.859 | +53132 |

### `records-1000` (1000 active records)

Run A, measured latency per route:

| Route | Band | n | Server p50 (ms) | Server p95 (ms) | Wall p50 (ms) | Wall p95 (ms) |
|---|---|---|---|---|---|---|
| inventory | B | 20 | 75682 | 76507 * | 75683.083 | 76507.625 * |
| view-single-markdown | B | 20 | 2980 | 2994 * | 2980.624 | 2994.441 * |
| view-grouped-member | B | 20 | 23746 | 24154 * | 23746.744 | 24154.684 * |
| view-single-html-mockup | A | 100 | 2 | 2 | 2.945 | 2.992 |
| edit-grouped-member | B | 20 | 23787 | 24008 * | 23787.658 | 24008.887 * |

Single run (owner-approved deviation): run B was not executed for this profile because of pre-refactor per-request latency. The single-run p95 is baseline-only and is never compared against the NFR-007 thresholds.

## Inventory scaling curve

| Active records | Band | n | Server p50 (ms) | Server p95 (ms) | Wall p50 (ms) | Wall p95 (ms) |
|---|---|---|---|---|---|---|
| 100 | B | 20 | 1041 | 1051 * | 1041.716 | 1052.076 * |
| 382 | B | 20 | 11756 | 11975 * | 11757.041 | 11975.744 * |
| 1000 | B | 20 | 75682 | 76507 * | 75683.083 | 76507.625 * |

The per-record inventory cost rises from roughly 10 ms at 100 records to roughly 31 ms at 382 and roughly 76 ms at 1,000. That super-linear growth is the recorded signature of the per-record configuration/registry rebuild plus the family rescan described under root causes.

## Comparison with the historical 382-record figures

Historical figures, provenance `MS-032 description; NFR-007.001 and NFR-007.002 rationale`: inventory `/` 14.2 s and `/doc/REQ-07.001` view 1.8 s at 382 active records. The historical corpus is not preserved, so exact equality is neither expected nor targeted; no corpus parameter was tuned to approach these values.

The historical `/doc/REQ-07.001` view is represented by `REQ-001` in the generated corpus (the requirement family group; the member route `REQ-001.001` is the closest individual member route and is also recorded below).

| Metric | Historical (382 records) | Measured `records-382` run A |
|---|---|---|
| Inventory server-observed p50 | 14.2 s (single figure) | 11756 ms (band B, n=20) |
| Inventory server-observed p95 | 14.2 s (single figure) | 11975 * ms (band B, n=20) |
| Requirement group view (`REQ-001`) server p50 | 1.8 s (single figure) | 3566 ms (band B, n=20) |
| Grouped member view (`REQ-001.001`) server p50 | 1.8 s (single figure) | 3682 ms (band B, n=20) |
| Grouped member view (`REQ-001.001`) server p95 | 1.8 s (single figure) | 3718 * ms (band B, n=20) |

## Root causes exercised by the measured paths

The current engine has no index cache. The measured paths exercise these call sites:

1. `src/record/migrate.rs::ensure_front_matter` calls `managed_descriptor`, which calls `load_cfg_for` and rebuilds `ArtifactRegistry` for every record. The same path calls `scan_migration_conflicts`, which rescans and reads the whole family for every record, so loading a family is super-linear. Inventory `/` loads every record of every enabled type, so each request pays this cost once per record. Every view and edit request rebuilds the index of the record's type and pays the same per-record cost for that type.
2. `src/browser/records.rs::resolve_id` rescans descriptor directories for each referenced ID. Inventory and the view/edit index resolve milestone, release, and superseded-by titles per record; the view route also resolves the record's own references. The measured single-markdown (`NOTE-001`) and unresolved-reference (`REQ-001.004`) routes exercise the unresolved lookup.
3. `ServerState::config()` (`src/browser/mod.rs`) reloads the effective configuration on every request. Every measured route calls it before it reads a record.
4. There is no index cache: inventory rebuilds every family index on each request, and view/edit rebuild the index of the viewed family. The inventory curve across 100, 382 and 1,000 records is therefore the direct signature of root causes 1, 2, and 4.

## Reference hardware comparison

NFR-007.001 reference: 4 vCPU, 8 GiB RAM, SSD, Linux, one server process, loopback HTTP, release build with LTO and strip.

| Reference | Measured | Result |
|---|---|---|
| 4 vCPU | 6 physical / 12 logical cores (Intel(R) Core(TM) i7-9750H CPU @ 2.60GHz) | deviation |
| 8 GiB RAM | 15.3 GiB | deviation |
| SSD | SSD (mapper/linux-root) | match |
| Linux | Debian GNU/Linux forky/sid (kernel `Linux 7.2.15+deb14-amd64 x86_64`) | match |
| One server process | one `papyrus browser` process | match |
| Loopback HTTP | `127.0.0.1` | match |
| Release build with LTO and strip | release (lto=true, strip=true) | match |

Deviations flagged: the measured host has more cores and roughly twice the reference RAM. The disk is an NVMe SSD, which matches the reference SSD class. These deviations mean the measured absolute latencies are not directly comparable to the reference-hardware targets; the baseline is recorded on this host so later milestones can compare same-host deltas.

## Recorded result files

- `docs/benchmarks/raw/records-100-run-a.json`
- `docs/benchmarks/raw/records-100-run-b.json`
- `docs/benchmarks/raw/records-382-run-a.json`
- `docs/benchmarks/raw/records-382-run-b.json`
- `docs/benchmarks/raw/records-1000-run-a.json`

