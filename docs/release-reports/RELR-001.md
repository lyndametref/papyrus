---
id: RELR-001
title: "Release 1.0.0 report"
created_date: '2026-11-23T14:22:46Z'
updated_date: '2026-11-23T14:22:50Z'
labels: []
references:
  - REL-001
status: issued
---
# RELR-001 Release 1.0.0 report

## Description
# Release 1.0.0 report

## Summary

REL-001 delivers the descriptor-driven document engine refactor: one field-driven engine over three document models, a seed table as the single definition of built-in types, nested configuration with selector paths, one browser surface for every document type, and a read path fast enough to serve 1,000 records in under 250 ms on the measurement host. Milestones MS-030 to MS-037 are complete.

## Version and artifact

| Item | Value |
|---|---|
| Version | 1.0.0 |
| Release commit | 38736ae |
| Artifact | builds/1.0.0/.build/papyrus/papyrus_1.0.0_amd64.deb |
| Size | 2,429,284 bytes |
| SHA-256 | b2d747ac017a65659b5147300775d423b95961c3b611bfbc733df09c57ddf43c |
| Packaged binary | /usr/bin/papyrus, 8,285,368 bytes, stripped, prints papyrus 1.0.0 |
| Inventory | /usr/bin/papyrus 0755 and /usr/share/doc/papyrus/copyright 0644 only |
| Verification build | builds/1.0.0-rc.1 (SUPERSEDED, discarded, not a release artifact) |

Install: sudo dpkg -i builds/1.0.0/.build/papyrus/papyrus_1.0.0_amd64.deb

## Scope delivered

- MS-030: milestone ordering and browser cleanup markers.
- MS-031: governance — RE-010, ADR-036 to ADR-044, 27 requirements, 13 business rules, 3 NFRs, 16 TECH sections, 94 approved test cases, Document Type in the domain model.
- MS-032: baseline — 254-case characterization corpus, benchmark harness, corrected renderer and route inventory.
- MS-033: record model foundation — owned descriptors, SEED_DOCUMENT_TYPES, FieldSpec, nested config with document_types diagnostics, selector paths. Inventory at 382 records fell from 11.8 s to 2.3 s.
- MS-034: CLI engine — one field-driven handler replacing nine typed modules (net minus 5,685 lines), uniform title contract, selector-path config, 144 attributed help goldens.
- MS-035: browser engine — one list, view, edit, conflict and 404 surface, seed-declared presentation, per-role reference visibility, semantic equivalence against the pre-refactor baseline for 62 pages.
- MS-036: performance — request snapshot, mtime serving snapshot, single-scan inventory, versioned immutable assets with ETag/304. At 1,000 records on the measurement host: inventory p95 247 ms and grouped view/edit 63 to 66 ms, down from 76.5 s and 23.9 s.
- MS-037: cleanup, documentation, audits and the single release package.

## Quality gates

| Gate | Record | Result |
|---|---|---|
| Code audit | AUD-155 (reissue of AUD-152) | pass for the code delta; artifact rebuilt from HEAD, digest b2d747ac |
| Security audit | AUD-153 and the reissue | pass with findings; 0 critical, 0 high, 0 medium after fixes; non-loopback bind risk accepted by ADR-044 |
| Quality audit | AUD-154 | pass with findings; AC-1 to AC-6 met after packaging |
| Milestone verification | AUD-150 (MS-036), TASK-289 (MS-037) | pass |
| Tests | 1446 passed, 0 failed, 27 ignored, 70 suites | green with and without EDITOR set |
| Lints | clippy -D warnings, cargo fmt | clean |
| Guards | no_drift 16, document_type_no_drift 27, no_descriptor_leaks 6, build_package_guard 18 | pass |
| Dependencies | 36 locked packages, 4 direct plus 1 dev | no new crates |
| Binary size | 8,285,368 bytes | under the 12 MiB gate |
| Production LOC | 34,168 | above the 28,000 informational target; not a gate |

## Known limitations

1. NFR-007.001 is measured-only, not satisfied: inventory p95 247 ms against 250 ms at 1,000 records, 3 ms headroom, on 6 physical and 12 logical cores with 15.3 GiB versus the 4 vCPU and 8 GiB reference host.
2. NFR-007.002 is measured-only, not satisfied: view and edit p95 63 to 66 ms against 100 ms at 1,000 records on the same host.
3. Performance was measured on tree bb9c152, before the MS-037 cleanup and hardening; no re-measurement was run on the final tree.
4. NFR-007.003 passes; the vendor second load is a browser-cache simulation, not a measured network transfer.
5. Non-loopback browser binding is unauthenticated with no session, authorization or TLS; the risk is opt-in, warned at startup, documented, and accepted in ADR-044.
6. cargo audit is unavailable offline; dependency posture is 4 direct plus 1 dev dependency, 36 locked crates.io packages, and --locked builds.
7. The installed host binary at /usr/bin/papyrus may still be 0.1.0; install the 1.0.0 package to upgrade.
8. One intermittent test flake was observed in the browser port-allocation helper; no product failure. A follow-up exists.
9. Historical archived-task frontmatter conflicts remain in TASK-098, TASK-111 and TASK-112; no duplicate audit IDs remain.
10. Supported platform: Debian Forky amd64 with libc6 2.34 or newer. No man page is packaged; the repository README carries the papyrus(1) and papyrus-config(5) reference sections.
11. Low-severity follow-ups recorded: YAML anchor scanner is lexical best-effort, duplicated destination-occupied helper, ADR-044 decision 8 wording, release build runs twice, deferred tasks TASK-213 to TASK-215.

## Verification performed

- Fresh release build from the release commit reproduces the packaged binary byte-for-byte.
- Isolated install with fakeroot dpkg: metadata Version 1.0.0, package state installed, binary reports papyrus 1.0.0, config list and a create and edit round trip succeed.
- Characterization: 254 cases; 62 document pages semantically equivalent to the pre-refactor baseline, 192 cases byte-strict, 0 payload deltas.
- Security probes: foreign Host 400, foreign Origin 403, traversal 400, symlink escape 404, unversioned vendor 404, app-asset 304, immutable vendor headers, HTML no-store with CSP and nosniff.
- Archive and relocation refuse an occupied destination; atomic writes leave no temporary files.

## Approval

This report is issued for the release approval decision. The release is not complete until the owner approves it.
