---
id: TASK-314
title: "Correct AUD-148 findings in MS-036 performance work"
created_date: '2026-11-22T22:37:37Z'
updated_date: '2026-11-22T22:37:37Z'
labels: []
references:
  - AUD-148
  - MS-036
  - NFR-007.001
  - NFR-007.002
  - NFR-007.003
  - ADR-037
  - ADR-038
status: ToDo
assignee:
  - rust-dev
milestone: MS-036
dependencies:
  - TASK-313
ordinal: 85
documentation:
  - src/browser/records.rs
  - src/browser/render.rs
  - src/browser/http.rs
  - src/browser/assets.rs
  - src/browser/math.rs
  - src/browser/sha256.rs
  - src/record/frontmatter.rs
  - src/record/migrate.rs
  - src/record/store.rs
  - src/record/view.rs
  - scripts/build-deb.sh
  - scripts/build-package.sh
  - tests/browser_cli.rs
  - tests/browser_snapshot.rs
  - tests/build_package_guard.rs
  - tests/benchmark_baseline.rs
  - docs/benchmarks/MS-036-results.md
  - docs/benchmarks/raw
  - decisions/ADR-038-cacheable-immutable-browser-assets.md
type: implementation
---

## Description
## Context
AUD-148 (code audit, MS-036) reports corrections required: 0 blocker, 2 high, 4 medium, 6 low. This single task carries every correction and must be complete before TASK-279 (MS-036 milestone verification) proceeds.

## Scope (AUD-148 order)

### High
- F1: an externally archived record still present in its active directory is counted by inventory and shown in grouped member rows, while the full index excludes it. Treat a relocated record as absent, by comparing the canonical path with the listed target or by returning the original target, and add archive tests for inventory counts and grouped rows.
- F2: the build-path asset-version guard runs only with the opt-in `--check-assets-only` flag. Call it at the start of the normal `scripts/build-deb.sh` assembly path and test each mismatch class (vendor, manifest, copyright) through a normal build.

### Medium
- F3: the front-matter-only grouped index uses a different title precedence from the full loader (front-matter title first versus heading first), so a member can show different titles on the group page and in the list. Align precedence through one shared projection.
- F4: every 304 response carries the `Internal Server Error` reason phrase. Add the `Not Modified` arm and assert the status line.
- F5: a record whose front-matter id differs from its file stem is no longer addressable by that id. Resolve it, or record the narrowing with a regression test.
- F6: benchmark provenance does not identify the measured production tree for the post-optimisation run, and the second-load JSON has no commit provenance. Record a worktree-dirty flag and diff hash, regenerate raw files after the source commit, add provenance to the second-load JSON, and correct the results sentence.
- F7: the front-matter-only paths duplicate acceptance and projection rules and have already diverged. Extract one shared front-matter projection and one acceptance predicate; this is the structural fix for F1 and F3.

### Low
- F8: remove dead code left after the inventory move (`scan_indexes`, `is_built`).
- F9: replace brittle substring-count source audits in the snapshot tests with behavioural or symbol assertions.
- F10: state in the verdict that the NFR-007.003 vendor second load is simulated, not measured, and give its basis.
- F11: record the thin inventory headroom (2 ms on a host faster than the reference hardware).
- F12: correct the MS-036 AC-8 verification wording: the server sends `Connection: close` and always has; no keep-alive is in place.

## Constraints
- No behaviour change beyond these corrections; no new crate, asset, template, route, or client script.
- Do not edit milestones, requirements, NFRs, or audit records from this task; record evidence as task comments.
- Regenerate benchmark raw files only after the source commit so provenance identifies the measured tree.
- Keep the AUD-148 ordering when recording results.

## Definition of done
Every finding has a recorded result and test, or a recorded narrowing with a test; the suite, clippy, and fmt are green; no behaviour change beyond the corrections; AUD-148 is addressed point by point; evidence is recorded on this task.

## Acceptance Criteria
- [ ] Each AUD-148 finding F1 to F12 has a recorded result on this task and a regression test, or an explicitly recorded narrowing with a test.
- [ ] F1: an externally archived record still present in its active directory is treated as absent by inventory counts and grouped member rows; archive tests cover both.
- [ ] F2: the asset-version guard runs on the normal build-deb assembly path, not only with --check-assets-only; each mismatch class is tested through a normal build.
- [ ] F3: front-matter-only title precedence matches the full loader through one shared projection; a regression test pins the parity.
- [ ] F4: 304 responses carry the Not Modified reason phrase and the status line is asserted.
- [ ] F5: a record whose front-matter id differs from its file stem is resolved, or the narrowing is recorded with a regression test.
- [ ] F6: benchmark provenance records a worktree-dirty flag and diff hash, raw files are regenerated after the source commit, provenance is added to the second-load JSON, and the results sentence is corrected.
- [ ] F7: one shared front-matter projection and one shared acceptance predicate replace the duplicated rules that diverged in F1 and F3.
- [ ] F8-F12: dead code is removed, brittle substring-count audits become behavioural or symbol assertions, the simulated vendor second-load basis is stated, the thin inventory headroom is recorded, and the keep-alive wording is corrected.
- [ ] The full test suite, cargo clippy --all-targets -- -D warnings, and cargo fmt --check are green.
- [ ] No behaviour change beyond the AUD-148 corrections; existing route, payload, status, header, and exit-code contracts are otherwise unchanged.
- [ ] AUD-148 is addressed point by point in the task evidence and is ready for re-audit.
