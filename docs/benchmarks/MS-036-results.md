# MS-036 post-refactor benchmark results (NFR-007.001, NFR-007.002, NFR-007.003)

> **TASK-313 update:** the read path was optimised and re-measured after this TASK-278 record. The post-optimisation results and verdicts are appended under "TASK-313 read-path optimisation re-measurement" at the end of this document. The TASK-278 figures and verdicts below remain the pre-optimisation record.

Status: recorded measurement. Measurement only; no production behaviour was changed. The full NFR-007 protocol was run on the available host, which deviates from the NFR-007 reference hardware (6 physical / 12 logical cores and 15.3 GiB instead of 4 vCPU / 8 GiB). **NFR-007.001 and NFR-007.002 are not marked satisfied by this run.** A pass below means only that the measured p95 was within the threshold on the available host.

- Run dates: `2026-11-22T22:39:59+01:00` to `2026-11-22T22:47:08+01:00`
- Measured production tree commit: `235c9cd9128205bf0cdde9218e005ea2af5c05d5` (last commit that changed `src`, `packaging`, `scripts`, `Cargo.toml` or `Cargo.lock`)
- Harness commit: `6bf99686ab0dff45f96a238390adbfc9eb4337ef` (last committed revision of `tests/benchmark_baseline.rs`; working tree modified during this run: true)
- Measured HEAD: `25173c795a6f0cf9a83cc6e8fbd75940fba1ea45`
- rustc: `rustc 1.99.1 (58cf8f9c3 2026-10-13)`
- Build profile: release (lto=true, strip=true)
- Harness command: `cargo test --release --test benchmark_baseline -- --ignored --nocapture benchmark_ms036`
- Host: Intel(R) Core(TM) i7-9750H CPU @ 2.60GHz with 6 physical and 12 logical cores, 15.3 GiB RAM, SSD (mapper/linux-root), kernel `Linux 7.2.15+deb14-amd64 x86_64`, Debian GNU/Linux forky/sid.
- Hardware deviation: NFR-007 reference hardware: 4 vCPU, 8 GiB RAM, SSD, Linux, one server process, loopback HTTP, release build with LTO and strip; measured host as above. The deviation is repeated beside every result table.

## Method

One release `papyrus --dir <corpus> browser --port <free> --no-open --debug` process per run on `127.0.0.1`, one unmeasured readiness request, then every route measured in sequence. Every request uses its own loopback TCP connection because the server sends `Connection: close`. Server-observed latency is the `GET <path> <status> <n>ms` debug log line (millisecond resolution); wall time is client `Instant` around connect/request/read. Percentiles are nearest-rank.

Band A is the full NFR-007 protocol: 10 warm-up requests followed by 100 measured sequential requests. Band B is 2 warm-up + 20 measured and is selected only when the first five measured samples of run A have p50 >= 1000 ms. Those five probe samples count toward the measured total, and band A tops up to ten warm-ups. Run B reuses run A's band, warm-up count and measured count. Both runs A and B were executed for all three profiles; the 1,000-record single-run exemption was not needed because neither run approached 90 minutes.

The first request after a cold start is excluded (one unmeasured readiness GET `/` before the protocol). No request in this corpus changes the configuration, so no configuration reload is triggered.

## Corpora

| Profile | Active records | Files (records + config) | Corpus hash (identical across runs A and B) | Composition |
|---|---|---|---|---|
| `records-100` | 100 | 101 | `7b88012297517150ed40e5fc1cbc6b4f74e5915aea0f6e61f92155d514e5404e` | tasks 14, milestones 5, releases 2, decisions 5, requirements 21 (1 group + 20 members), UI mockups 5, NFRs 6 (1 group + 5 members), audit 4, glossary 6, rules 6, tech specs 6, test cases 5, test reports 3, UI specs 3, use cases 4, notes 5 |
| `records-382` | 382 | 383 | `16852d24b236c29d9a109ec5c706645bf3ab5a60615c120a3dcf6e76cb816a15` | tasks 53, milestones 19, releases 8, decisions 19, requirements 77 (1 group + 76 members), UI mockups 19, NFRs 20 (1 group + 19 members), audit 15, glossary 23, rules 23, tech specs 23, test cases 19, test reports 11, UI specs 11, use cases 15, notes 27 |
| `records-1000` | 1000 | 1001 | `46d64349137f7017f6ee04bcfd6a3d3042b7b6bc23245749dac400804581988f` | tasks 140, milestones 50, releases 20, decisions 50, requirements 201 (1 group + 200 members), UI mockups 50, NFRs 51 (1 group + 50 members), audit 40, glossary 60, rules 60, tech specs 60, test cases 50, test reports 30, UI specs 30, use cases 40, notes 68 |

The 1,000-record profile contains one GroupedMarkdown type with exactly 200 members and one SingleHtmlMockup type with exactly 50 records; the harness regenerates the corpus deterministically for every run and asserts both counts.

## Sample counts

### `records-100` (100 active records, 2 runs)

| Route | Band | Warm-up | Measured | Run B same |
|---|---|---|---|---|
| inventory | A | 10 | 100 | 10 / 100 |
| view-single-markdown | A | 10 | 100 | 10 / 100 |
| view-grouped-group | A | 10 | 100 | 10 / 100 |
| view-grouped-member | A | 10 | 100 | 10 / 100 |
| view-single-html-mockup | A | 10 | 100 | 10 / 100 |
| view-unresolved-reference | A | 10 | 100 | 10 / 100 |
| edit-single-markdown | A | 10 | 100 | 10 / 100 |
| edit-grouped-group | A | 10 | 100 | 10 / 100 |
| edit-grouped-member | A | 10 | 100 | 10 / 100 |
| edit-single-html-mockup | A | 10 | 100 | 10 / 100 |
| edit-unresolved-reference | A | 10 | 100 | 10 / 100 |

### `records-382` (382 active records, 2 runs)

| Route | Band | Warm-up | Measured | Run B same |
|---|---|---|---|---|
| inventory | A | 10 | 100 | 10 / 100 |
| view-single-markdown | A | 10 | 100 | 10 / 100 |
| view-grouped-group | A | 10 | 100 | 10 / 100 |
| view-grouped-member | A | 10 | 100 | 10 / 100 |
| view-single-html-mockup | A | 10 | 100 | 10 / 100 |
| view-unresolved-reference | A | 10 | 100 | 10 / 100 |
| edit-single-markdown | A | 10 | 100 | 10 / 100 |
| edit-grouped-group | A | 10 | 100 | 10 / 100 |
| edit-grouped-member | A | 10 | 100 | 10 / 100 |
| edit-single-html-mockup | A | 10 | 100 | 10 / 100 |
| edit-unresolved-reference | A | 10 | 100 | 10 / 100 |

### `records-1000` (1000 active records, 2 runs)

| Route | Band | Warm-up | Measured | Run B same |
|---|---|---|---|---|
| inventory | A | 10 | 100 | 10 / 100 |
| view-single-markdown | A | 10 | 100 | 10 / 100 |
| view-grouped-group | A | 10 | 100 | 10 / 100 |
| view-grouped-member | A | 10 | 100 | 10 / 100 |
| view-single-html-mockup | A | 10 | 100 | 10 / 100 |
| view-unresolved-reference | A | 10 | 100 | 10 / 100 |
| edit-single-markdown | A | 10 | 100 | 10 / 100 |
| edit-grouped-group | A | 10 | 100 | 10 / 100 |
| edit-grouped-member | A | 10 | 100 | 10 / 100 |
| edit-single-html-mockup | A | 10 | 100 | 10 / 100 |
| edit-unresolved-reference | A | 10 | 100 | 10 / 100 |

## Results at 100 records

Measured on 6 physical / 12 logical cores, 15.3 GiB RAM, SSD (mapper/linux-root), kernel `Linux 7.2.15+deb14-amd64 x86_64`; NFR-007 reference is 4 vCPU / 8 GiB. Every figure below is a measured figure on the available host, not NFR-007.001 or NFR-007.002 satisfaction.

| Route | Band | n | Server p50 (ms) | Server p95 (ms) | MS-032 baseline p50 | MS-032 baseline p95 | Δ p50 (ms) | Δ p95 (ms) |
|---|---|---|---|---|---|---|---|---|
| inventory | A | 100 | 55 | 60 | 1048 (B, n=20) | 1060 | -993 | -1000 |
| view-single-markdown | A | 100 | 3 | 3 | 44 (A, n=100) | 45 | -41 | -42 |
| view-grouped-group | A | 100 | 13 | 14 | 311 (A, n=100) | 320 | -298 | -306 |
| view-grouped-member | A | 100 | 12 | 13 | 341 (A, n=100) | 345 | -329 | -332 |
| view-single-html-mockup | A | 100 | 0 | 1 | 1 (A, n=100) | 1 | -1 | +0 |
| view-unresolved-reference | A | 100 | 12 | 16 | 343 (A, n=100) | 346 | -331 | -330 |
| edit-single-markdown | A | 100 | 3 | 3 | 44 (A, n=100) | 45 | -41 | -42 |
| edit-grouped-group | A | 100 | 12 | 13 | 310 (A, n=100) | 318 | -298 | -305 |
| edit-grouped-member | A | 100 | 12 | 13 | 342 (A, n=100) | 351 | -330 | -338 |
| edit-single-html-mockup | A | 100 | 1 | 2 | 1 (A, n=100) | 3 | +0 | -1 |
| edit-unresolved-reference | A | 100 | 12 | 13 | 343 (A, n=100) | 351 | -331 | -338 |

These 100-record figures are recorded for comparison only; no NFR-007 threshold applies at 100 records.

## Results at 382 records

Measured on 6 physical / 12 logical cores, 15.3 GiB RAM, SSD (mapper/linux-root), kernel `Linux 7.2.15+deb14-amd64 x86_64`; NFR-007 reference is 4 vCPU / 8 GiB. Every figure below is a measured figure on the available host, not NFR-007.001 or NFR-007.002 satisfaction.

| Route | Band | n | Server p50 (ms) | Server p95 (ms) | MS-032 baseline p50 | MS-032 baseline p95 | Δ p50 (ms) | Δ p95 (ms) |
|---|---|---|---|---|---|---|---|---|
| inventory | A | 100 | 203 | 206 | 11776 (B, n=20) | 11971 | -11573 | -11765 |
| view-single-markdown | A | 100 | 15 | 15 | 555 (A, n=100) | 565 | -540 | -550 |
| view-grouped-group | A | 100 | 48 | 50 | 3552 (B, n=20) | 3629 | -3504 | -3579 |
| view-grouped-member | A | 100 | 45 | 46 | 3666 (B, n=20) | 3736 | -3621 | -3690 |
| view-single-html-mockup | A | 100 | 1 | 2 | 1 (A, n=100) | 1 | +0 | +1 |
| view-unresolved-reference | A | 100 | 45 | 46 | 3680 (B, n=20) | 3723 | -3635 | -3677 |
| edit-single-markdown | A | 100 | 15 | 16 | 557 (A, n=100) | 564 | -542 | -548 |
| edit-grouped-group | A | 100 | 45 | 47 | 3553 (B, n=20) | 3611 | -3508 | -3564 |
| edit-grouped-member | A | 100 | 45 | 46 | 3692 (B, n=20) | 3734 | -3647 | -3688 |
| edit-single-html-mockup | A | 100 | 1 | 2 | 1 (A, n=100) | 1 | +0 | +1 |
| edit-unresolved-reference | A | 100 | 45 | 46 | 3665 (B, n=20) | 3735 | -3620 | -3689 |

The MS-032 382-record baseline used band B and 20 samples for several routes, so its p95 is baseline-only; the deltas are indicative, not a threshold verdict.

## Results at 1,000 records and NFR comparison

Measured on 6 physical / 12 logical cores, 15.3 GiB RAM, SSD (mapper/linux-root), kernel `Linux 7.2.15+deb14-amd64 x86_64`; NFR-007 reference is 4 vCPU / 8 GiB. Every figure below is a measured figure on the available host, not NFR-007.001 or NFR-007.002 satisfaction.

A pass means the measured p95 was below the threshold on the available host. It is **not** NFR-007.001 or NFR-007.002 satisfaction; the reference hardware was not available.

| Route | Band | n | Server p50 (ms) | Server p95 (ms) | Threshold (ms) | Measured verdict | MS-032 baseline p50 | MS-032 baseline p95 | Δ p50 | Δ p95 |
|---|---|---|---|---|---|---|---|---|---|---|
| inventory | A | 100 | 542 | 550 | 250 | MISS | 75546 (B, n=20) | 76532 | -75004 | -75982 |
| view-single-markdown | A | 100 | 38 | 39 | 100 | pass (measured) | 2939 (B, n=20) | 2998 | -2901 | -2959 |
| view-grouped-group | A | 100 | 124 | 126 | 100 | MISS | 23142 (B, n=20) | 23648 | -23018 | -23522 |
| view-grouped-member | A | 100 | 118 | 121 | 100 | MISS | 23431 (B, n=20) | 23950 | -23313 | -23829 |
| view-single-html-mockup | A | 100 | 4 | 4 | 100 | pass (measured) | 2 (A, n=100) | 2 | +2 | +2 |
| view-unresolved-reference | A | 100 | 118 | 120 | 100 | MISS | 23651 (B, n=20) | 23861 | -23533 | -23741 |
| edit-single-markdown | A | 100 | 38 | 39 | 100 | pass (measured) | 2942 (B, n=20) | 2973 | -2904 | -2934 |
| edit-grouped-group | A | 100 | 118 | 120 | 100 | MISS | 23200 (B, n=20) | 23642 | -23082 | -23522 |
| edit-grouped-member | A | 100 | 118 | 120 | 100 | MISS | 23596 (B, n=20) | 23820 | -23478 | -23700 |
| edit-single-html-mockup | A | 100 | 4 | 4 | 100 | pass (measured) | 3 (A, n=100) | 3 | +1 | +1 |
| edit-unresolved-reference | A | 100 | 118 | 119 | 100 | MISS | 23504 (B, n=20) | 23777 | -23386 | -23658 |

The MS-032 1,000-record baseline used band B (20 samples) for every route except the two HTML mockup routes, so its p95 values are baseline-only. The 1,000-record inventory baseline was 75,546 ms p50 / 76,532 ms p95.

## NFR verdicts

- **NFR-007.001 (inventory p95 < 250 ms at 1,000 records): measured MISS.** Measured server p95 550 ms on the available host. Measured on 6 physical / 12 logical cores, 15.3 GiB RAM, SSD (mapper/linux-root), kernel `Linux 7.2.15+deb14-amd64 x86_64`; NFR-007 reference is 4 vCPU / 8 GiB. Every figure below is a measured figure on the available host, not NFR-007.001 or NFR-007.002 satisfaction.
- **NFR-007.002 (view and edit p95 < 100 ms at 1,000 records): measured MISS.** The following measured routes were at or above 100 ms:
  - `view-grouped-group`: p95 126 ms against 100 ms.
  - `view-grouped-member`: p95 121 ms against 100 ms.
  - `view-unresolved-reference`: p95 120 ms against 100 ms.
  - `edit-grouped-group`: p95 120 ms against 100 ms.
  - `edit-grouped-member`: p95 120 ms against 100 ms.
  - `edit-unresolved-reference`: p95 119 ms against 100 ms.
  These misses are reported as measured and are not smoothed or presented as a pass.
- **NFR-007.003 (cache contract and repeat visit): measured pass.** The second-load measurement below records zero vendor body bytes on the second load, bodyless 304 responses for the first-party app assets, and full `no-store` HTML and JSON bodies on every repeat load. This is a measured pass on the available host, not a claim about the reference hardware.

## NFR-007.003 second-load transfer

Measured on 6 physical / 12 logical cores, 15.3 GiB RAM, SSD (mapper/linux-root), kernel `Linux 7.2.15+deb14-amd64 x86_64`; NFR-007 reference is 4 vCPU / 8 GiB. Every figure below is a measured figure on the available host, not NFR-007.001 or NFR-007.002 satisfaction.

Corpus: dedicated deterministic cache corpus, hash `dca390095965e3239a7d6bcecc26300bb9b040c6e3a94c427cf51473aeb11913`, 4 files. Method: A browser-cache simulation: immutable vendor URLs are cache hits on the second load; revalidated first-party assets send If-None-Match and accept a bodyless 304; HTML and JSON are re-fetched with no-store. A conditional probe on every URL records the exact 304/200 matrix. KaTeX WOFF2 URLs named by the served stylesheet are included as math-page vendor assets.

| Page | Class | URL | First-load status | First body bytes | Second-load status | Second-load body bytes | Cache-Control | ETag |
|---|---|---|---|---|---|---|---|---|
| diagram | revalidated-first-party | `/assets/app.css` | 200 | 17880 | 304 | 0 | no-cache | "ba45bcdda8f99bf643f537109e00d51f519901625fc143f24550ef6f83b04a4c" |
| diagram | revalidated-first-party | `/assets/app.js` | 200 | 12953 | 304 | 0 | no-cache | "5b2ce904b04ca0f270f632b0e9921259913957a4509c3fe1802ac7b955f073f7" |
| diagram | immutable-vendor | `/assets/mermaid/11.17.2/mermaid.min.js` | 200 | 3572661 | 200 | 0 | public, max-age=31536000, immutable | none |
| diagram | revalidated-first-party | `/assets/nav-state.js` | 200 | 679 | 304 | 0 | no-cache | "c12c7fe53d844706c7c085f2c9ad6fc6aa784a3ba236895259b486d993bd961d" |
| math | revalidated-first-party | `/assets/app.css` | 200 | 17880 | 304 | 0 | no-cache | "ba45bcdda8f99bf643f537109e00d51f519901625fc143f24550ef6f83b04a4c" |
| math | revalidated-first-party | `/assets/app.js` | 200 | 12953 | 304 | 0 | no-cache | "5b2ce904b04ca0f270f632b0e9921259913957a4509c3fe1802ac7b955f073f7" |
| math | immutable-vendor | `/assets/katex/0.16.11/fonts/KaTeX_AMS-Regular.woff2` | 200 | 28076 | 200 | 0 | public, max-age=31536000, immutable | none |
| math | immutable-vendor | `/assets/katex/0.16.11/fonts/KaTeX_Caligraphic-Bold.woff2` | 200 | 6912 | 200 | 0 | public, max-age=31536000, immutable | none |
| math | immutable-vendor | `/assets/katex/0.16.11/fonts/KaTeX_Caligraphic-Regular.woff2` | 200 | 6908 | 200 | 0 | public, max-age=31536000, immutable | none |
| math | immutable-vendor | `/assets/katex/0.16.11/fonts/KaTeX_Fraktur-Bold.woff2` | 200 | 11348 | 200 | 0 | public, max-age=31536000, immutable | none |
| math | immutable-vendor | `/assets/katex/0.16.11/fonts/KaTeX_Fraktur-Regular.woff2` | 200 | 11316 | 200 | 0 | public, max-age=31536000, immutable | none |
| math | immutable-vendor | `/assets/katex/0.16.11/fonts/KaTeX_Main-Bold.woff2` | 200 | 25324 | 200 | 0 | public, max-age=31536000, immutable | none |
| math | immutable-vendor | `/assets/katex/0.16.11/fonts/KaTeX_Main-BoldItalic.woff2` | 200 | 16780 | 200 | 0 | public, max-age=31536000, immutable | none |
| math | immutable-vendor | `/assets/katex/0.16.11/fonts/KaTeX_Main-Italic.woff2` | 200 | 16988 | 200 | 0 | public, max-age=31536000, immutable | none |
| math | immutable-vendor | `/assets/katex/0.16.11/fonts/KaTeX_Main-Regular.woff2` | 200 | 26272 | 200 | 0 | public, max-age=31536000, immutable | none |
| math | immutable-vendor | `/assets/katex/0.16.11/fonts/KaTeX_Math-BoldItalic.woff2` | 200 | 16400 | 200 | 0 | public, max-age=31536000, immutable | none |
| math | immutable-vendor | `/assets/katex/0.16.11/fonts/KaTeX_Math-Italic.woff2` | 200 | 16440 | 200 | 0 | public, max-age=31536000, immutable | none |
| math | immutable-vendor | `/assets/katex/0.16.11/fonts/KaTeX_SansSerif-Bold.woff2` | 200 | 12216 | 200 | 0 | public, max-age=31536000, immutable | none |
| math | immutable-vendor | `/assets/katex/0.16.11/fonts/KaTeX_SansSerif-Italic.woff2` | 200 | 12028 | 200 | 0 | public, max-age=31536000, immutable | none |
| math | immutable-vendor | `/assets/katex/0.16.11/fonts/KaTeX_SansSerif-Regular.woff2` | 200 | 10344 | 200 | 0 | public, max-age=31536000, immutable | none |
| math | immutable-vendor | `/assets/katex/0.16.11/fonts/KaTeX_Script-Regular.woff2` | 200 | 9644 | 200 | 0 | public, max-age=31536000, immutable | none |
| math | immutable-vendor | `/assets/katex/0.16.11/fonts/KaTeX_Size1-Regular.woff2` | 200 | 5468 | 200 | 0 | public, max-age=31536000, immutable | none |
| math | immutable-vendor | `/assets/katex/0.16.11/fonts/KaTeX_Size2-Regular.woff2` | 200 | 5208 | 200 | 0 | public, max-age=31536000, immutable | none |
| math | immutable-vendor | `/assets/katex/0.16.11/fonts/KaTeX_Size3-Regular.woff2` | 200 | 3624 | 200 | 0 | public, max-age=31536000, immutable | none |
| math | immutable-vendor | `/assets/katex/0.16.11/fonts/KaTeX_Size4-Regular.woff2` | 200 | 4928 | 200 | 0 | public, max-age=31536000, immutable | none |
| math | immutable-vendor | `/assets/katex/0.16.11/fonts/KaTeX_Typewriter-Regular.woff2` | 200 | 13568 | 200 | 0 | public, max-age=31536000, immutable | none |
| math | immutable-vendor | `/assets/katex/0.16.11/katex.min.css` | 200 | 23335 | 200 | 0 | public, max-age=31536000, immutable | none |
| math | immutable-vendor | `/assets/katex/0.16.11/katex.min.js` | 200 | 275414 | 200 | 0 | public, max-age=31536000, immutable | none |
| math | revalidated-first-party | `/assets/nav-state.js` | 200 | 679 | 304 | 0 | no-cache | "c12c7fe53d844706c7c085f2c9ad6fc6aa784a3ba236895259b486d993bd961d" |
| plain | revalidated-first-party | `/assets/app.css` | 200 | 17880 | 304 | 0 | no-cache | "ba45bcdda8f99bf643f537109e00d51f519901625fc143f24550ef6f83b04a4c" |
| plain | revalidated-first-party | `/assets/app.js` | 200 | 12953 | 304 | 0 | no-cache | "5b2ce904b04ca0f270f632b0e9921259913957a4509c3fe1802ac7b955f073f7" |
| plain | revalidated-first-party | `/assets/nav-state.js` | 200 | 679 | 304 | 0 | no-cache | "c12c7fe53d844706c7c085f2c9ad6fc6aa784a3ba236895259b486d993bd961d" |

Per-page totals (bytes):

| Page | First-load asset body | First-load vendor | First-load app | Second-load vendor | Second-load app | Second-load HTML | Second-load JSON |
|---|---|---|---|---|---|---|---|
| diagram | 3604173 | 3572661 | 31512 | 0 | 0 | 3129 | 2 |
| math | 590053 | 558541 | 31512 | 0 | 0 | 3194 | 2 |
| plain | 31512 | 0 | 31512 | 0 | 0 | 2932 | 2 |

Conditional matrix: every immutable vendor asset answered a conditional `If-None-Match` request with a full `200` and the first-load bytes, never a `304`; every `/assets/app.css`, `/assets/app.js`, and `/assets/nav-state.js` request with the strong ETag answered `304` with an empty body, the same ETag and `no-cache`, and no `Content-Type` or `Content-Length`; a stale tag answered `200` with the full body; HTML pages and `/api/tasks` answered `200` with a full body and `no-store` even when sent `If-None-Match`. The first-load and second-load headers are recorded in `docs/benchmarks/raw/ms036-second-load.json`.

## Connection behaviour

`Connection: close` remains on every response. The measurement did not show a need to change it: every route completed with one sequential request per connection, and the second-load measurement reuses the browser cache rather than a persistent connection. No connection behaviour was changed in this task.

## Recorded result files

- `docs/benchmarks/raw/ms036-records-100-run-a.json`
- `docs/benchmarks/raw/ms036-records-100-run-b.json`
- `docs/benchmarks/raw/ms036-records-382-run-a.json`
- `docs/benchmarks/raw/ms036-records-382-run-b.json`
- `docs/benchmarks/raw/ms036-records-1000-run-a.json`
- `docs/benchmarks/raw/ms036-records-1000-run-b.json`
- `docs/benchmarks/raw/ms036-second-load.json`

## Reproducibility

The run is reproducible from the recorded command and corpus hashes. The corpus generator deletes and recreates `./.tmp/benchmark/<profile>/` for every run; a second run must reproduce the same file count and SHA-256 manifest. No corpus parameter was tuned toward a target. The measurement changes no production source and adds no crate.

## TASK-313 read-path optimisation re-measurement

Status: post-optimisation measurement of the read path on the same available host. The production read path was changed by TASK-313; every other row, header, status code and payload is unchanged (characterization replay green). **NFR-007.001 and NFR-007.002 remain unmarked**: this run is a measured pass on the available host, not satisfaction of the reference-hardware requirement.

- Measured tree: `b51bd66a9330d164f2b57debe864d24e9743ed83` plus the TASK-313 working-tree changes in `src/browser` and `src/record` (release build, LTO and strip); the measurement harness `tests/benchmark_baseline.rs` was unmodified (`harness_worktree_modified: false`).
- Run dates: `2026-11-22T23:19:03+01:00` to `2026-11-22T23:22:14+01:00`
- Harness command: `cargo test --release --test benchmark_baseline -- --ignored --nocapture benchmark_ms036`
- Build profile: `release (lto=true, strip=true)`; rustc `rustc 1.99.1 (58cf8f9c3 2026-10-13)`
- Host and hardware deviation: unchanged from the run header above (6 physical / 12 logical cores, 15.3 GiB; NFR-007 reference 4 vCPU / 8 GiB). Every figure is a measured figure on the available host.
- Protocol: full NFR-007 protocol. All routes, all profiles, two runs, every route band A with 10 warm-up and 100 measured sequential requests (no route was assigned band B).
- Corpora: unchanged deterministic corpora; 1,000-record corpus hash `46d64349137f7017f6ee04bcfd6a3d3042b7b6bc23245749dac400804581988f`, 1,001 files.
- Raw result files: `docs/benchmarks/raw/ms036-{records-100,records-382,records-1000}-run-{a,b}.json` and `ms036-second-load.json`. The TASK-313 run regenerated these files; the pre-optimisation TASK-278 values remain in git history at `6e071ca`.

### All measured profiles

| Profile | Route | Band | n | Run A p50 (ms) | Run A p95 (ms) | Run B p50 (ms) | Run B p95 (ms) | Target (ms) | Verdict |
|---|---|---|---|---|---|---|---|---|---|
| `records-100` | inventory | A | 100 | 26 | 27 | 25 | 26 | n/a | recorded (no threshold) |
| `records-100` | view-single-markdown | A | 100 | 1 | 1 | 2 | 3 | n/a | recorded (no threshold) |
| `records-100` | view-grouped-group | A | 100 | 7 | 12 | 7 | 7 | n/a | recorded (no threshold) |
| `records-100` | view-grouped-member | A | 100 | 7 | 7 | 7 | 7 | n/a | recorded (no threshold) |
| `records-100` | view-single-html-mockup | A | 100 | 0 | 2 | 1 | 2 | n/a | recorded (no threshold) |
| `records-100` | view-unresolved-reference | A | 100 | 7 | 7 | 7 | 15 | n/a | recorded (no threshold) |
| `records-100` | edit-single-markdown | A | 100 | 2 | 2 | 1 | 2 | n/a | recorded (no threshold) |
| `records-100` | edit-grouped-group | A | 100 | 1 | 2 | 2 | 2 | n/a | recorded (no threshold) |
| `records-100` | edit-grouped-member | A | 100 | 7 | 15 | 7 | 13 | n/a | recorded (no threshold) |
| `records-100` | edit-single-html-mockup | A | 100 | 0 | 1 | 0 | 1 | n/a | recorded (no threshold) |
| `records-100` | edit-unresolved-reference | A | 100 | 7 | 15 | 7 | 7 | n/a | recorded (no threshold) |
| `records-382` | inventory | A | 100 | 92 | 94 | 92 | 94 | n/a | recorded (no threshold) |
| `records-382` | view-single-markdown | A | 100 | 3 | 4 | 3 | 3 | n/a | recorded (no threshold) |
| `records-382` | view-grouped-group | A | 100 | 26 | 27 | 25 | 26 | n/a | recorded (no threshold) |
| `records-382` | view-grouped-member | A | 100 | 25 | 25 | 25 | 27 | n/a | recorded (no threshold) |
| `records-382` | view-single-html-mockup | A | 100 | 1 | 1 | 1 | 1 | n/a | recorded (no threshold) |
| `records-382` | view-unresolved-reference | A | 100 | 25 | 25 | 24 | 25 | n/a | recorded (no threshold) |
| `records-382` | edit-single-markdown | A | 100 | 3 | 3 | 3 | 3 | n/a | recorded (no threshold) |
| `records-382` | edit-grouped-group | A | 100 | 1 | 1 | 1 | 1 | n/a | recorded (no threshold) |
| `records-382` | edit-grouped-member | A | 100 | 25 | 26 | 25 | 25 | n/a | recorded (no threshold) |
| `records-382` | edit-single-html-mockup | A | 100 | 3 | 3 | 2 | 2 | n/a | recorded (no threshold) |
| `records-382` | edit-unresolved-reference | A | 100 | 25 | 25 | 25 | 25 | n/a | recorded (no threshold) |
| `records-1000` | inventory | A | 100 | 242 | 248 | 243 | 248 | 250 | pass (measured) |
| `records-1000` | view-single-markdown | A | 100 | 8 | 8 | 8 | 8 | 100 | pass (measured) |
| `records-1000` | view-grouped-group | A | 100 | 64 | 65 | 64 | 66 | 100 | pass (measured) |
| `records-1000` | view-grouped-member | A | 100 | 63 | 65 | 63 | 64 | 100 | pass (measured) |
| `records-1000` | view-single-html-mockup | A | 100 | 1 | 1 | 1 | 1 | 100 | pass (measured) |
| `records-1000` | view-unresolved-reference | A | 100 | 63 | 64 | 63 | 65 | 100 | pass (measured) |
| `records-1000` | edit-single-markdown | A | 100 | 8 | 8 | 8 | 8 | 100 | pass (measured) |
| `records-1000` | edit-grouped-group | A | 100 | 1 | 1 | 2 | 2 | 100 | pass (measured) |
| `records-1000` | edit-grouped-member | A | 100 | 63 | 67 | 63 | 64 | 100 | pass (measured) |
| `records-1000` | edit-single-html-mockup | A | 100 | 1 | 1 | 1 | 1 | 100 | pass (measured) |
| `records-1000` | edit-unresolved-reference | A | 100 | 62 | 64 | 63 | 64 | 100 | pass (measured) |

### Before and after at 1,000 records

Pre-optimisation figures are the TASK-278 full-protocol results recorded above. Post-optimisation figures are both TASK-313 runs. A pass is a measured p95 within the threshold on the available host; it is not NFR-007.001 or NFR-007.002 satisfaction.

| Route | TASK-278 p95 (ms) | TASK-313 run A p50/p95 (ms) | TASK-313 run B p50/p95 (ms) | Target (ms) | Verdict | Δ p95 vs TASK-278 (run A / run B) |
|---|---|---|---|---|---|---|
| inventory | 550 | 242 / 248 | 243 / 248 | 250 | pass (measured) | -302 / -302 |
| view-single-markdown | 39 | 8 / 8 | 8 / 8 | 100 | pass (measured) | -31 / -31 |
| view-grouped-group | 126 | 64 / 65 | 64 / 66 | 100 | pass (measured) | -61 / -60 |
| view-grouped-member | 121 | 63 / 65 | 63 / 64 | 100 | pass (measured) | -56 / -57 |
| view-single-html-mockup | 4 | 1 / 1 | 1 / 1 | 100 | pass (measured) | -3 / -3 |
| view-unresolved-reference | 120 | 63 / 64 | 63 / 65 | 100 | pass (measured) | -56 / -55 |
| edit-single-markdown | 39 | 8 / 8 | 8 / 8 | 100 | pass (measured) | -31 / -31 |
| edit-grouped-group | 120 | 1 / 1 | 2 / 2 | 100 | pass (measured) | -119 / -118 |
| edit-grouped-member | 120 | 63 / 67 | 63 / 64 | 100 | pass (measured) | -53 / -56 |
| edit-single-html-mockup | 4 | 1 / 1 | 1 / 1 | 100 | pass (measured) | -3 / -3 |
| edit-unresolved-reference | 119 | 62 / 64 | 63 / 64 | 100 | pass (measured) | -55 / -55 |

### Route notes

- **Inventory p95 248 ms in both 1,000-record runs against 250 ms**: measured pass at 1,000 records; the two 1,000-record runs measured 248 ms and 248 ms p95, down from 550 ms. Inventory now builds one file-name listing per descriptor directory, validates each listed file through the same front-matter read contract, and counts without parsing document bodies, read projections or field maps.
- **Grouped group, grouped member and unresolved-reference view and edit**: p95 64 to 67 ms at 1,000 records against 100 ms, down from 118 to 126 ms. Group and member pages build their family rows from front-matter-only records; the page record itself is still read in full, and a record already parsed in full is reused from the request snapshot. Edit pages no longer build a family index unless a grouped member needs its owning group link.
- **SingleMarkdown view and edit** p95 8 ms at 1,000 records (from 39 ms); **SingleHtmlMockup view and edit** p95 1 ms. Both remain well inside target and neither regressed against TASK-278.
- **NFR-007.003 second-load transfer** is unchanged: zero vendor body bytes on the second load, bodyless 304 responses for first-party assets, and full `no-store` HTML and JSON. The same `ms036-second-load.json` evidence was regenerated by this run.

### Remaining misses

None on the available host for the measured routes. NFR-007.001 and NFR-007.002 are still not marked satisfied because the NFR-007 reference hardware (4 vCPU / 8 GiB) was not used; the figures remain measured passes with the recorded hardware deviation.
