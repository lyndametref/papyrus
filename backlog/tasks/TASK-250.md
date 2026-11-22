---
id: TASK-250
title: "Remove per-record configuration and registry construction from the read path"
created_date: '2026-11-20T17:41:37Z'
updated_date: '2026-11-22T11:14:29Z'
labels: []
references:
  - ADR-037
  - ADR-036
  - TECH-043
  - TECH-040
  - NFR-007.001
  - TC-160
  - TC-171
  - TC-193
status: Done
assignee:
  - rust-dev
milestone: MS-033
dependencies:
  - TASK-249
ordinal: 70
documentation:
  - src/record/migrate.rs
  - src/record/store.rs
  - src/record/resolve.rs
  - src/browser/records.rs
  - docs/specifications/technical/TECH-043.md
type: implementation
---

## Description
Context: MS-033 start commit c3103fc. The read path rebuilds configuration and the registry for every record:
- `src/record/migrate.rs:81 managed_descriptor(project_dir, path)` calls `load_cfg_for` (line 1206), which runs `commands::load_effective_config` and then `ArtifactRegistry::build`; `ensure_front_matter_internal` (line 187) calls it on every read, and `is_managed_path` (line 71) calls it too.
- `src/record/store.rs` calls `ensure_front_matter_quiet` at lines 155, 385, and 630 and `managed_descriptor` at lines 157, 231, and 473, each time rebuilding the configuration and the registry. `src/record/resolve.rs:124` does the same and also calls `load_effective_config` directly at line 118.
- `src/browser/records.rs::load_record` (line 308) already takes `cfg: &Config` and `registry: &ArtifactRegistry`, but `resolve_id` (line 697) calls `descriptor_directory` and `fs::read_dir` once per looked-up ID, and `build_index` walks each descriptor directory.
- `src/lib.rs` builds the configuration and registry once at phase 2 and passes only the registry to `cli::dispatch`, so callers cannot supply both.
- Benchmark root cause recorded in `docs/benchmarks/`: `migrate::ensure_front_matter` calls `managed_descriptor` and `load_cfg_for` at least twice per record (TECH-043 sections 2 and 3).

Scope:
1. Change `managed_descriptor` and `is_managed_path` in `src/record/migrate.rs` to take `&Config` and `&ArtifactRegistry`, and delete `load_cfg_for` and every per-record `load_effective_config` and `ArtifactRegistry::build` call from `src/record/`.
2. Thread `&Config` and `&ArtifactRegistry` into `ensure_front_matter`, `ensure_front_matter_quiet`, `ensure_front_matter_canonical`, the read and write entry points of `src/record/store.rs`, and `src/record/resolve.rs`. Callers supply the one registry built at `src/lib.rs` phase 2; `cli::dispatch` receives the `Config` alongside the registry as internal plumbing only.
3. Update the callers that today pass only `ctx`: the document modules (`src/requirement.rs`, `src/glossary.rs`, `src/rule.rs`, `src/audit.rs`, `src/ui_spec.rs`, `src/ui_mockup.rs`, `src/tech_spec.rs`, `src/use_case.rs`, `src/test_case.rs`, `src/test_report.rs`, `src/task.rs`, `src/milestone.rs`, `src/release.rs`, `src/decision.rs`) and `src/browser/records.rs`. Typed module signatures may change internally; their user-visible behaviour may not.
4. Title resolution uses the supplied registry: `browser::records::resolve_id` resolves the descriptor through `registry.descriptor` and `registry.record_descriptor` and never builds a registry or loads configuration. The per-ID directory scan remains in this milestone and is TASK-273's scope (ADR-037 decisions 21 to 25), so no index, request snapshot, or cache is added here.
5. Keep the mtime cache as the only reload point (owner decision 11). This task adds no cache and no snapshot; the serving snapshot is TASK-274 in MS-036.
6. Behaviour and output bytes are unchanged: adoption keys and order, migration and conflict outcomes, list, view, edit, and JSON payloads, status codes, and browser pages.

Invariants:
- No read path builds a configuration or a registry per record. One configuration load and one registry build per process or request serve every record in that pass (MS-033 AC-5, TECH-043 section 2).
- Old configuration files must parse to the same effective document-type descriptor set, and no record or page content changes (MS-033 AC-1, AC-7).
- Every write still re-reads the live file for its version token, and `record::store` stays the only writer (ADR-006, ADR-013).

Out of scope: the request-scoped record snapshot, a config fingerprint, and the serving snapshot (TASK-273 and TASK-274, MS-036); the inventory scan (TASK-275); the document engine and typed module deletion (MS-034). No dependency, ordinal, milestone, or assignee changes.

Assumptions:
- Passing `&Config` and `&ArtifactRegistry` from `lib.rs` through `cli::dispatch` is acceptable internal plumbing; no command, option, output, or exit code changes.
- `RecordDescriptor::declared_statuses` and the other record facts keep reading the supplied `Config`, not a reloaded one.
- A record of a configuration-defined type still resolves after the configuration file changes between building the registry and the read, because the supplied registry is the descriptor source for that pass.

Definition of done: `src/record/` contains no configuration load or registry build; the read-path signatures require both values; the full suite, the characterization replay, and the browser and CLI suites pass unchanged; `cargo clippy --all-targets -- -D warnings` is clean.
## Acceptance Criteria
- [ ] The record module contains no production call to `load_effective_config`, `load_effective_value`, `ArtifactRegistry::build`, or `load_cfg_for`; a source-guard test fails if one returns, and it passes on the final tree (MS-033 AC-5, TECH-043 section 2).
- [ ] `managed_descriptor`, `is_managed_path`, `ensure_front_matter`, `ensure_front_matter_quiet`, `ensure_front_matter_canonical`, the store read and write entry points, and `record::resolve` require `&Config` and `&ArtifactRegistry`; the crate does not compile when either is omitted (TECH-043 section 2).
- [ ] A record of a configuration-defined type still reads after `papyrus.config.yml` is removed or made invalid between building the registry and the read: the read resolves through the supplied `Config` and registry and does not reload configuration (MS-033 AC-5).
- [ ] `browser::records::resolve_id` and `load_record` resolve their descriptor through the supplied registry and never build a registry or load configuration; the per-ID directory scan is unchanged and remains TASK-273's scope (ADR-037 decisions 21 to 25).
- [ ] No cache, snapshot, fingerprint, or reload point is introduced: the mtime cache stays the only reload point (owner decision 11), and the serving snapshot remains TASK-274 in MS-036.
- [ ] The characterization replay, the list replay, the CLI-help goldens, and the browser, CLI, JSON, and migration suites pass byte-identically, so adoption keys and order, record bytes, page output, JSON payloads, and status codes are unchanged (MS-033 AC-7, TC-160, TC-193).
- [ ] Every write still reads the live file for its version token and writes through `record::store`; the stale-token case still returns 409 and writes nothing (ADR-006, ADR-013, TC-171).
- [ ] `cargo test` reports zero failures and `cargo clippy --all-targets -- -D warnings` is clean.
## Comments
- **task-refiner**: Refined for MS-033: description grounded in migrate.rs managed_descriptor and load_cfg_for, the per-record loads in store.rs and resolve.rs, browser records load_record and resolve_id, and lib.rs phase 2; 8 testable acceptance criteria added; TECH-043 documented. The request-scoped record snapshot and the serving snapshot are explicitly deferred to MS-036. No dependency, ordinal, milestone, or assignee change.
- **rust-dev**: rust-dev implementation complete on branch task-TASK-250 (uncommitted per orchestrator instruction). One config load and one registry build now serve each pass. Record migrate managed_descriptor and is_managed_path take Config plus ArtifactRegistry; ensure_front_matter, ensure_front_matter_quiet, ensure_front_matter_canonical, migrate_content, scan_migration_conflicts, migrate_corpus and migrate_file thread them. Record store read, read_document, read_document_normalized, write_atomic, archive, delete, set_status, relocate_if_archived, archive_content and reassign_id require both. Record resolve resolve_in_dirs, list_in_dir, duplicate_ids and describe_record require both. Cli dispatch receives Config with the phase 2 registry; task, milestone, release, decision, document, import and search receive both. Browser load_record and resolve_id use the supplied config and registry only. Guard tests scan production sources in src record and src browser records for load_effective_config, load_effective_value, ArtifactRegistry::build, load_cfg_for and fn load_cfg. New tests prove a config-defined record reads after papyrus.config.yml is removed or made invalid between registry build and read. Checks: env -u EDITOR -u VISUAL cargo test passed (1214 tests, 65 suites); cargo clippy --all-targets -- -D warnings clean; cargo fmt --check clean. Measurement on the 382-record corpus (smoke, inventory request): server p50 fell from the pre-refactor recorded 11776-11905 ms to 2245-2287 ms, about 5.2x faster; corpus hash matched 16852d24b236c29d9a109ec5c706645bf3ab5a60615c120a3dcf6e76cb816a15.
- **orchestrator**: Acceptance met: per-record config and registry construction removed from CLI and browser read paths, guard tests enforce it, config-removed and config-invalid read tests pass; suite 1214 tests green; 382-record inventory p50 11205 ms to 2266 ms.
