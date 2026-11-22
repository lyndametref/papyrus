---
id: TASK-314
title: "Correct AUD-148 findings in MS-036 performance work"
created_date: '2026-11-22T22:37:37Z'
updated_date: '2026-11-22T23:07:11Z'
labels: []
references:
  - AUD-148
  - MS-036
  - NFR-007.001
  - NFR-007.002
  - NFR-007.003
  - ADR-037
  - ADR-038
status: In-Progress
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
## Comments
- **rust-dev**: AUD-148 corrections implemented on branch task-TASK-314; status left In-Progress; no git commit by instruction. F1: an externally archived record still in its active directory is now absent from inventory counts and grouped member rows; the shared accept_listed predicate rejects a listed path that no longer exists after read-path relocation and both the count and the front-matter-only index use it; tests inventory_count_excludes_a_record_archived_in_its_active_directory, grouped_rows_exclude_a_record_archived_in_its_active_directory and t314_externally_archived_record_is_absent_from_inventory_and_group_page. F2: build-deb.sh runs check_asset_versions on the normal assembly path before any output, and check-assets-only exits after the same check; build_package_guard tests f2_vendor_version_directory_mismatch_fails_a_normal_build, f2_manifest_version_mismatch_fails_a_normal_build and f2_copyright_version_mismatch_fails_a_normal_build each run a normal build and assert exit 1, the class message and no output directory. F3: one shared project_front_matter projection supplies title precedence, labels, references, status fallback and group link to the full loader and the front-matter-only index; tests grouped_light_title_matches_the_full_loader_when_titles_differ and t314_group_member_title_matches_the_list. F4: reason 304 is Not Modified and the integration helper asserts the full status line HTTP 1.1 304 Not Modified for every 304. F5: narrowing recorded; snapshot resolution stays canonical-file-stem keyed so a front-matter id that differs from the stem is not addressable by that id; regression test frontmatter_id_mismatch_is_not_addressable_by_the_frontmatter_id and a resolve doc note. F6: harness records production_worktree_modified and production_diff_hash (SHA-256 of git diff binary against the production commit) in every run JSON, the summary header and ms036-second-load.json plus git commit and harness commit; raw files regenerated by the post-correction run with production commit 3305a3110bdefd7412893cdea6c4a31a2d6c9b49, dirty true, diff hash b7852f3ad3eb128bab3fec3efb9b764b38a708a114d6d931db1bb3c39b6b76b5, HEAD 9eee1f86eea324b2e03603ffc6efe34bb040c831 and harness commit 6e071ca277f2095c5ff1c22c7a5352a38dc40f19; the results sentence now says the harness adds no crate and does not change production source while it runs and that the measured tree is identified by commit, dirty flag and diff hash. F7: one shared acceptance predicate accept_listed and one shared projection helper project_front_matter replace the duplicated rules. F8: removed dead scan_indexes and is_built; inventory tests use scan_counts and build_index_with_snapshot. F9: browser_snapshot.rs now uses item_source symbol and block assertions instead of substring counts. F10: NFR-007.003 verdict states the vendor second load is a browser-cache simulation, not a measured transfer, with its basis: immutable vendor URL treated as a cache hit and the recorded basis is the full first-load 200 bytes returned to a conditional If-None-Match, never a 304. F11: NFR-007.001 verdict records the 2 ms headroom to 250 ms on a host faster than the reference and states it is thin and not evidence of reference-hardware compliance. F12: results-doc Connection behaviour wording corrected: Connection close on every response and always has been; no keep-alive is in place; no code change. Re-measured after F1 and F3 with the full NFR-007 protocol at 1000 records (ms, p50 and p95): inventory 242 and 248 unchanged; view-grouped-group 65 and 65; view-grouped-member 63 and 64; view-unresolved-reference 63 and 65; edit-grouped-member 63 and 64; every grouped route stays within 1 ms of TASK-313 so the corrections add no measurable cost; NFR-007.001 measured pass at 248 ms, NFR-007.002 measured pass on every route below 100 ms, NFR-007.003 measured pass. Checks: cargo test with EDITOR and VISUAL unset 1397 passed, 0 failed, 27 ignored; cargo clippy --all-targets -- -D warnings clean; cargo fmt --check clean; no new crate, route, template, asset or client script. Note: the session instruction forbids git commit; raw files were regenerated from the working tree identified by the recorded production commit, dirty flag and diff hash so the measured tree is exact and reproducible; the orchestrator should commit on task-TASK-314.
- **orchestrator**: Acceptance met: shared acceptance predicate and front-matter projection close the archived-record leak and the grouped title divergence; asset-version guard on the normal assembly path; 304 Not Modified; snapshot narrowing recorded; benchmark provenance with dirty flag and diff hash regenerated; dead code and brittle audits removed; 1397 tests pass with targets still met.
- **orchestrator**: Reopened for the close-out residual: empty front-matter title must fall back to the heading, and the benchmark must be re-measured on a clean committed tree so provenance matches.
- **rust-dev**: Fixed empty front-matter title fallback; benchmark pending.
- **rust-dev**: rust-dev close-out residual: in the browser records module projected_title, an empty or whitespace-only front-matter title is now treated as absent so tasks and milestones fall back to the heading, then the file stem, matching the previous loader. Regression test empty_front_matter_title_falls_back_to_heading_then_stem covers a task with an empty title and heading, a task with an empty title and no heading (stem), a milestone with a whitespace-only title, and a document record with an empty title. Full cargo test with EDITOR and VISUAL unset is green, cargo clippy all-targets with deny warnings is clean, cargo fmt check is clean. The clean-tree benchmark re-measurement is still pending.
