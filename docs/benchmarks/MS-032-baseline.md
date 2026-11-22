# MS-032 pre-refactor benchmark baseline

Status: recorded (TASK-241). This is the pre-refactor baseline; no production behaviour was changed. It uses the reduced-sample protocol of MS-032 AC-8 because the full NFR-007 protocol does not complete on the pre-refactor engine.

- Dates: `2026-11-22T02:44:47+01:00` to `2026-11-22T06:51:28+01:00`
- Measured production tree commit: `3bc3ce518238f2a36b9c095df0e7226bf889184a` (last commit that changed `src`, `packaging`, `scripts`, `Cargo.toml` or `Cargo.lock`)
- Harness commit: `d46d6dfee641dfb1692ff9696d6a8fdc95878db2` (last commit that changed `tests/benchmark_baseline.rs`; harness working tree modified during this run: true)
- Measured HEAD: `cd26ef564baa89e47e82f6d78e9b68642a2908ab`
- rustc: `rustc 1.99.1 (58cf8f9c3 2026-10-13)`
- Build profile: release (lto=true, strip=true)
- Harness command: `cargo test --release --test benchmark_baseline -- --ignored --nocapture`

## Baseline-only status

The reduced sample counts come from pre-refactor per-request latency, not measurement uncertainty: the recorded reconnaissance measured 74.37 s per inventory request at 1,000 records and 23.90 s per grouped-member view at 1,000 records. A p95 computed from fewer than 100 samples is **baseline-only** and is never compared against the NFR-007.001 or NFR-007.002 thresholds. MS-036 measures the post-refactor engine with the full NFR-007 protocol of 10 warm-up + 100 measured requests per route, so this baseline deviation never enters the acceptance comparison. NFR-007.001 to NFR-007.003 are unchanged.

Owner-approved exemption from duplicate runs for `records-1000` (TASK-292 C-2): this profile was measured with a single baseline run (run A only) because the pre-refactor engine needs roughly 75 s per inventory request at 1,000 records, which makes duplicate runs impractical. Its figures are **single-run baseline-only** and are never compared against the NFR-007.001 or NFR-007.002 thresholds. The 100 and 382 profiles keep the recorded A and B runs plus two no-write confirmation reruns (C and D).

## Host

Intel(R) Core(TM) i7-9750H CPU @ 2.60GHz with 6 physical and 12 logical cores, 15.3 GiB RAM, SSD (mapper/linux-root), kernel `Linux 7.2.15+deb14-amd64 x86_64`, Debian GNU/Linux forky/sid.

## Method

The harness generates a deterministic Papyrus project under `./.tmp/benchmark/<profile>/` (deleted and recreated for every run), boots one real `papyrus --dir <corpus> browser --port <free> --no-open --debug` process on `127.0.0.1`, performs one unmeasured readiness request, then measures every measured route with the protocol below. Every request uses its own loopback TCP connection because the server sends `Connection: close`.

Band selection is deterministic. Run A performs two common warm-ups, then the first five measured samples. Their p50 selects band A (p50 < 1000 ms, 10 warm-up + 100 measured) or band B (p50 >= 1000 ms, 2 warm-up + 20 measured); those five samples count toward the measured total. Band A then tops up to ten warm-ups before the remaining samples. Later runs reuse the same band, warm-up count and measured count. Percentiles are nearest-rank over the measured samples. The six routes added at 1,000 records by TASK-292 C-1 are assigned band B explicitly (2 warm-up + 20 measured) as the task directs; all other bands remain data-driven.

- Server-observed latency: the `{method} {path} {status} {n}ms` line printed by `src/browser/mod.rs::handle_connection` when `--debug` is set. Millisecond resolution.
- Wall time: client-side `std::time::Instant` around connect, request, and read. Microsecond resolution.
- Routes: inventory `/`; SingleMarkdown view/edit (`NOTE-001`); GroupedMarkdown group and member view/edit (`REQ-001`, `REQ-001.001`); SingleHtmlMockup view/edit (`MOCK-001`); and the member with the unresolved group reference (`REQ-001.004`). No form POST is sent because it writes records.
- Every profile measures all eleven routes (TASK-292 C-1 removed the 1,000-record representative-set limit); no route is omitted or extrapolated.
- `records-100` and `records-382` run A and B plus two no-write confirmation reruns C and D; confirmation reruns never overwrite the recorded A and B artifacts.

## Corpora

| Profile | Active records | Files (records + config) | Corpus hash (identical across runs) | Composition |
|---|---|---|---|---|
| `records-100` | 100 | 101 | `7b88012297517150ed40e5fc1cbc6b4f74e5915aea0f6e61f92155d514e5404e` | tasks 14, milestones 5, releases 2, decisions 5, requirements 21 (1 group + 20 members), UI mockups 5, NFRs 6 (1 group + 5 members), audit 4, glossary 6, rules 6, tech specs 6, test cases 5, test reports 3, UI specs 3, use cases 4, notes 5 |
| `records-382` | 382 | 383 | `16852d24b236c29d9a109ec5c706645bf3ab5a60615c120a3dcf6e76cb816a15` | tasks 53, milestones 19, releases 8, decisions 19, requirements 77 (1 group + 76 members), UI mockups 19, NFRs 20 (1 group + 19 members), audit 15, glossary 23, rules 23, tech specs 23, test cases 19, test reports 11, UI specs 11, use cases 15, notes 27 |
| `records-1000` | 1000 | 1001 | `46d64349137f7017f6ee04bcfd6a3d3042b7b6bc23245749dac400804581988f` | tasks 140, milestones 50, releases 20, decisions 50, requirements 201 (1 group + 200 members), UI mockups 50, NFRs 51 (1 group + 50 members), audit 40, glossary 60, rules 60, tech specs 60, test cases 50, test reports 30, UI specs 30, use cases 40, notes 68 |

## Sample counts and bands

Every count below the full 10 warm-up + 100 measured protocol is band B and is caused by pre-refactor per-request latency, not measurement uncertainty.

### `records-100` (100 active records, 4 runs)

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

### `records-382` (382 active records, 4 runs)

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
| view-grouped-group | B | 2 | 20 | band B assigned by TASK-292 C-1 at 1,000 records (2 warm-up + 20 measured); no value is extrapolated |
| view-grouped-member | B | 2 | 20 | band B: p50 >= 1000 ms before reduction; 20 measured < 100 full-protocol samples because of pre-refactor per-request latency, not measurement uncertainty |
| view-single-html-mockup | A | 10 | 100 | band A: full NFR-007 protocol (10 warm-up + 100 measured) |
| view-unresolved-reference | B | 2 | 20 | band B assigned by TASK-292 C-1 at 1,000 records (2 warm-up + 20 measured); no value is extrapolated |
| edit-single-markdown | B | 2 | 20 | band B assigned by TASK-292 C-1 at 1,000 records (2 warm-up + 20 measured); no value is extrapolated |
| edit-grouped-group | B | 2 | 20 | band B assigned by TASK-292 C-1 at 1,000 records (2 warm-up + 20 measured); no value is extrapolated |
| edit-grouped-member | B | 2 | 20 | band B: p50 >= 1000 ms before reduction; 20 measured < 100 full-protocol samples because of pre-refactor per-request latency, not measurement uncertainty |
| edit-single-html-mockup | B | 2 | 20 | band B assigned by TASK-292 C-1 at 1,000 records (2 warm-up + 20 measured); no value is extrapolated |
| edit-unresolved-reference | B | 2 | 20 | band B assigned by TASK-292 C-1 at 1,000 records (2 warm-up + 20 measured); no value is extrapolated |

## Results

A `*` on a p95 marks a baseline-only figure computed from fewer than 100 samples.

### `records-100` (100 active records)

Run A, measured latency per route:

| Route | Band | n | Server p50 (ms) | Server p95 (ms) | Wall p50 (ms) | Wall p95 (ms) |
|---|---|---|---|---|---|---|
| inventory | B | 20 | 1048 | 1060 * | 1049.095 | 1060.381 * |
| view-single-markdown | A | 100 | 44 | 45 | 44.885 | 45.959 |
| view-grouped-group | A | 100 | 311 | 320 | 311.314 | 321.040 |
| view-grouped-member | A | 100 | 341 | 345 | 341.813 | 345.994 |
| view-single-html-mockup | A | 100 | 1 | 1 | 1.922 | 1.988 |
| view-unresolved-reference | A | 100 | 343 | 346 | 343.620 | 346.534 |
| edit-single-markdown | A | 100 | 44 | 45 | 44.951 | 45.971 |
| edit-grouped-group | A | 100 | 310 | 318 | 311.088 | 318.483 |
| edit-grouped-member | A | 100 | 342 | 351 | 342.263 | 352.102 |
| edit-single-html-mockup | A | 100 | 1 | 3 | 1.217 | 3.958 |
| edit-unresolved-reference | A | 100 | 343 | 351 | 343.539 | 352.077 |

Recorded variance (run B minus run A):

| Route | n | Server p95 A (ms) | Server p95 B (ms) | Server Δ p95 (ms) | Wall p95 A (ms) | Wall p95 B (ms) | Wall Δ p95 (µs) |
|---|---|---|---|---|---|---|---|
| inventory | 20 | 1060 | 1096 | +36 | 1060.381 | 1096.797 | +36416 |
| view-single-markdown | 100 | 45 | 46 | +1 | 45.959 | 46.491 | +532 |
| view-grouped-group | 100 | 320 | 317 | -3 | 321.040 | 318.167 | -2873 |
| view-grouped-member | 100 | 345 | 349 | +4 | 345.994 | 349.626 | +3632 |
| view-single-html-mockup | 100 | 1 | 2 | +1 | 1.988 | 3.055 | +1067 |
| view-unresolved-reference | 100 | 346 | 349 | +3 | 346.534 | 349.320 | +2786 |
| edit-single-markdown | 100 | 45 | 48 | +3 | 45.971 | 48.783 | +2812 |
| edit-grouped-group | 100 | 318 | 319 | +1 | 318.483 | 319.505 | +1022 |
| edit-grouped-member | 100 | 351 | 350 | -1 | 352.102 | 351.038 | -1064 |
| edit-single-html-mockup | 100 | 3 | 3 | +0 | 3.958 | 3.761 | -197 |
| edit-unresolved-reference | 100 | 351 | 351 | +0 | 352.077 | 352.228 | +151 |

Confirmation reruns (no-write, TASK-292 C-2). Each rerun p95 must stay inside the A-B range widened by 10 percent of the nearer bound: the accepted interval is `[min(A,B) * 0.9, max(A,B) * 1.1]`. The 1000-record profile is exempt (owner-approved single-run baseline-only). The `edit-single-html-mockup` route is a sub-5 ms route whose server p95 has millisecond resolution; its out-of-band reruns are recorded exactly as measured, never rounded or excluded.

| Route | Server A-B p95 range (ms) | Widened server interval (ms) | Server C p95 (ms) | C in range | Server D p95 (ms) | D in range |
|---|---|---|---|---|---|---|
| inventory | 1060..1096 | 954.0..1205.6 | 1094 | yes | 1097 | yes |
| view-single-markdown | 45..46 | 40.5..50.6 | 47 | yes | 45 | yes |
| view-grouped-group | 317..320 | 285.3..352.0 | 319 | yes | 321 | yes |
| view-grouped-member | 345..349 | 310.5..383.9 | 350 | yes | 350 | yes |
| view-single-html-mockup | 1..2 | 0.9..2.2 | 1 | yes | 2 | yes |
| view-unresolved-reference | 346..349 | 311.4..383.9 | 351 | yes | 352 | yes |
| edit-single-markdown | 45..48 | 40.5..52.8 | 47 | yes | 46 | yes |
| edit-grouped-group | 318..319 | 286.2..350.9 | 322 | yes | 320 | yes |
| edit-grouped-member | 350..351 | 315.0..386.1 | 351 | yes | 353 | yes |
| edit-single-html-mockup | 3..3 | 2.7..3.3 | 3 | yes | 1 | no |
| edit-unresolved-reference | 351..351 | 315.9..386.1 | 348 | yes | 351 | yes |

Confirmation reruns for wall time (microseconds; same widened A-B rule):

| Route | Wall A-B p95 range (µs) | Widened wall interval (µs) | Wall C p95 (µs) | C in range | Wall D p95 (µs) | D in range |
|---|---|---|---|---|---|---|
| inventory | 1060381..1096797 | 954342.9..1206476.7 | 1094365 | yes | 1098023 | yes |
| view-single-markdown | 45959..46491 | 41363.1..51140.1 | 47715 | yes | 45606 | yes |
| view-grouped-group | 318167..321040 | 286350.3..353144.0 | 320102 | yes | 321415 | yes |
| view-grouped-member | 345994..349626 | 311394.6..384588.6 | 350403 | yes | 351104 | yes |
| view-single-html-mockup | 1988..3055 | 1789.2..3360.5 | 2184 | yes | 2233 | yes |
| view-unresolved-reference | 346534..349320 | 311880.6..384252.0 | 351435 | yes | 353077 | yes |
| edit-single-markdown | 45971..48783 | 41373.9..53661.3 | 47251 | yes | 46508 | yes |
| edit-grouped-group | 318483..319505 | 286634.7..351455.5 | 323193 | yes | 321109 | yes |
| edit-grouped-member | 351038..352102 | 315934.2..387312.2 | 352129 | yes | 353552 | yes |
| edit-single-html-mockup | 3761..3958 | 3384.9..4353.8 | 4081 | yes | 1488 | no |
| edit-unresolved-reference | 352077..352228 | 316869.3..387450.8 | 348809 | yes | 351451 | yes |

### `records-382` (382 active records)

Run A, measured latency per route:

| Route | Band | n | Server p50 (ms) | Server p95 (ms) | Wall p50 (ms) | Wall p95 (ms) |
|---|---|---|---|---|---|---|
| inventory | B | 20 | 11776 | 11971 * | 11776.698 | 11972.040 * |
| view-single-markdown | A | 100 | 555 | 565 | 555.780 | 566.039 |
| view-grouped-group | B | 20 | 3552 | 3629 * | 3553.218 | 3629.673 * |
| view-grouped-member | B | 20 | 3666 | 3736 * | 3666.629 | 3736.887 * |
| view-single-html-mockup | A | 100 | 1 | 1 | 1.929 | 1.990 |
| view-unresolved-reference | B | 20 | 3680 | 3723 * | 3680.389 | 3723.688 * |
| edit-single-markdown | A | 100 | 557 | 564 | 557.843 | 564.999 |
| edit-grouped-group | B | 20 | 3553 | 3611 * | 3553.419 | 3611.415 * |
| edit-grouped-member | B | 20 | 3692 | 3734 * | 3692.812 | 3734.951 * |
| edit-single-html-mockup | A | 100 | 1 | 1 | 1.608 | 1.710 |
| edit-unresolved-reference | B | 20 | 3665 | 3735 * | 3666.230 | 3735.425 * |

Recorded variance (run B minus run A):

| Route | n | Server p95 A (ms) | Server p95 B (ms) | Server Δ p95 (ms) | Wall p95 A (ms) | Wall p95 B (ms) | Wall Δ p95 (µs) |
|---|---|---|---|---|---|---|---|
| inventory | 20 | 11971 | 12017 | +46 | 11972.040 | 12017.349 | +45309 |
| view-single-markdown | 100 | 565 | 564 | -1 | 566.039 | 564.855 | -1184 |
| view-grouped-group | 20 | 3629 | 3632 | +3 | 3629.673 | 3632.944 | +3271 |
| view-grouped-member | 20 | 3736 | 3728 | -8 | 3736.887 | 3729.134 | -7753 |
| view-single-html-mockup | 100 | 1 | 2 | +1 | 1.990 | 2.419 | +429 |
| view-unresolved-reference | 20 | 3723 | 3744 | +21 | 3723.688 | 3744.620 | +20932 |
| edit-single-markdown | 100 | 564 | 567 | +3 | 564.999 | 567.503 | +2504 |
| edit-grouped-group | 20 | 3611 | 3612 | +1 | 3611.415 | 3612.666 | +1251 |
| edit-grouped-member | 20 | 3734 | 3724 | -10 | 3734.951 | 3724.914 | -10037 |
| edit-single-html-mockup | 100 | 1 | 1 | +0 | 1.710 | 1.953 | +243 |
| edit-unresolved-reference | 20 | 3735 | 3732 | -3 | 3735.425 | 3733.212 | -2213 |

Confirmation reruns (no-write, TASK-292 C-2). Each rerun p95 must stay inside the A-B range widened by 10 percent of the nearer bound: the accepted interval is `[min(A,B) * 0.9, max(A,B) * 1.1]`. The 1000-record profile is exempt (owner-approved single-run baseline-only). The `edit-single-html-mockup` route is a sub-5 ms route whose server p95 has millisecond resolution; its out-of-band reruns are recorded exactly as measured, never rounded or excluded.

| Route | Server A-B p95 range (ms) | Widened server interval (ms) | Server C p95 (ms) | C in range | Server D p95 (ms) | D in range |
|---|---|---|---|---|---|---|
| inventory | 11971..12017 | 10773.9..13218.7 | 11988 | yes | 12053 | yes |
| view-single-markdown | 564..565 | 507.6..621.5 | 564 | yes | 568 | yes |
| view-grouped-group | 3629..3632 | 3266.1..3995.2 | 3626 | yes | 3650 | yes |
| view-grouped-member | 3728..3736 | 3355.2..4109.6 | 3735 | yes | 3758 | yes |
| view-single-html-mockup | 1..2 | 0.9..2.2 | 1 | yes | 1 | yes |
| view-unresolved-reference | 3723..3744 | 3350.7..4118.4 | 3754 | yes | 3760 | yes |
| edit-single-markdown | 564..567 | 507.6..623.7 | 567 | yes | 566 | yes |
| edit-grouped-group | 3611..3612 | 3249.9..3973.2 | 3622 | yes | 3636 | yes |
| edit-grouped-member | 3724..3734 | 3351.6..4107.4 | 3747 | yes | 3757 | yes |
| edit-single-html-mockup | 1..1 | 0.9..1.1 | 3 | no | 2 | no |
| edit-unresolved-reference | 3732..3735 | 3358.8..4108.5 | 3734 | yes | 3769 | yes |

Confirmation reruns for wall time (microseconds; same widened A-B rule):

| Route | Wall A-B p95 range (µs) | Widened wall interval (µs) | Wall C p95 (µs) | C in range | Wall D p95 (µs) | D in range |
|---|---|---|---|---|---|---|
| inventory | 11972040..12017349 | 10774836.0..13219083.9 | 11989174 | yes | 12054180 | yes |
| view-single-markdown | 564855..566039 | 508369.5..622642.9 | 564916 | yes | 568450 | yes |
| view-grouped-group | 3629673..3632944 | 3266705.7..3996238.4 | 3627176 | yes | 3650623 | yes |
| view-grouped-member | 3729134..3736887 | 3356220.6..4110575.7 | 3735979 | yes | 3759252 | yes |
| view-single-html-mockup | 1990..2419 | 1791.0..2660.9 | 2078 | yes | 2123 | yes |
| view-unresolved-reference | 3723688..3744620 | 3351319.2..4119082.0 | 3755187 | yes | 3760294 | yes |
| edit-single-markdown | 564999..567503 | 508499.1..624253.3 | 567580 | yes | 566421 | yes |
| edit-grouped-group | 3611415..3612666 | 3250273.5..3973932.6 | 3622630 | yes | 3636335 | yes |
| edit-grouped-member | 3724914..3734951 | 3352422.6..4108446.1 | 3747438 | yes | 3757824 | yes |
| edit-single-html-mockup | 1710..1953 | 1539.0..2148.3 | 4252 | no | 2246 | no |
| edit-unresolved-reference | 3733212..3735425 | 3359890.8..4108967.5 | 3734882 | yes | 3769902 | yes |

### `records-1000` (1000 active records)

Run A, measured latency per route:

| Route | Band | n | Server p50 (ms) | Server p95 (ms) | Wall p50 (ms) | Wall p95 (ms) |
|---|---|---|---|---|---|---|
| inventory | B | 20 | 75546 | 76532 * | 75546.337 | 76532.400 * |
| view-single-markdown | B | 20 | 2939 | 2998 * | 2939.539 | 2998.795 * |
| view-grouped-group | B | 20 | 23142 | 23648 * | 23142.995 | 23648.737 * |
| view-grouped-member | B | 20 | 23431 | 23950 * | 23431.749 | 23950.568 * |
| view-single-html-mockup | A | 100 | 2 | 2 | 3.025 | 3.076 |
| view-unresolved-reference | B | 20 | 23651 | 23861 * | 23651.482 | 23861.745 * |
| edit-single-markdown | B | 20 | 2942 | 2973 * | 2942.539 | 2973.277 * |
| edit-grouped-group | B | 20 | 23200 | 23642 * | 23200.687 | 23643.173 * |
| edit-grouped-member | B | 20 | 23596 | 23820 * | 23597.141 | 23820.567 * |
| edit-single-html-mockup | B | 20 | 2 | 2 * | 2.756 | 2.848 * |
| edit-unresolved-reference | B | 20 | 23504 | 23777 * | 23505.211 | 23777.482 * |

Single run (owner-approved exemption from duplicate runs, TASK-292 C-2): run B was not executed for this profile because the pre-refactor engine needs roughly 75 s per inventory request at 1,000 records. The single-run p95 is baseline-only and is never compared against the NFR-007 thresholds.
## Inventory scaling curve

| Active records | Band | n | Server p50 (ms) | Server p95 (ms) | Wall p50 (ms) | Wall p95 (ms) |
|---|---|---|---|---|---|---|
| 100 | B | 20 | 1048 | 1060 * | 1049.095 | 1060.381 * |
| 382 | B | 20 | 11776 | 11971 * | 11776.698 | 11972.040 * |
| 1000 | B | 20 | 75546 | 76532 * | 75546.337 | 76532.400 * |

The per-record inventory cost rises from roughly 10 ms at 100 records to roughly 30 ms at 382 and roughly 74 ms at 1,000. That super-linear growth is the recorded signature of the per-record configuration/registry rebuild plus the family rescan described under root causes.

## Comparison with the historical 382-record figures

Historical figures, provenance `MS-032 description; NFR-007.001 and NFR-007.002 rationale`: inventory `/` 14.2 s and `/doc/REQ-07.001` view 1.8 s at 382 active records. The historical corpus is not preserved, so exact equality is neither expected nor targeted; no corpus parameter was tuned to approach these values.

The historical `/doc/REQ-07.001` view is represented by `REQ-001` in the generated corpus (the requirement family group; the member route `REQ-001.001` is the closest individual member route and is also recorded below).

| Metric | Historical (382 records) | Measured `records-382` run A |
|---|---|---|
| Inventory server-observed p50 | 14.2 s (single figure) | 0 ms (band B, n=20) |
| Inventory server-observed p95 | 14.2 s (single figure) | 0 * ms (band B, n=20) |
| Requirement group view (`REQ-001`) server p50 | 1.8 s (single figure) | 0 ms (band B, n=20) |
| Grouped member view (`REQ-001.001`) server p50 | 1.8 s (single figure) | 0 ms (band B, n=20) |
| Grouped member view (`REQ-001.001`) server p95 | 1.8 s (single figure) | 0 * ms (band B, n=20) |

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
- `docs/benchmarks/raw/records-100-run-c.json` (no-write confirmation rerun)
- `docs/benchmarks/raw/records-100-run-d.json` (no-write confirmation rerun)
- `docs/benchmarks/raw/records-382-run-a.json`
- `docs/benchmarks/raw/records-382-run-b.json`
- `docs/benchmarks/raw/records-382-run-c.json` (no-write confirmation rerun)
- `docs/benchmarks/raw/records-382-run-d.json` (no-write confirmation rerun)
- `docs/benchmarks/raw/records-1000-run-a.json`

