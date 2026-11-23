---
id: TASK-283
title: "Extend the no-drift and descriptor-leak guards for the three-handler architecture"
created_date: '2026-11-20T17:44:01Z'
updated_date: '2026-11-23T06:54:17Z'
labels: []
references:
  - ADR-036
  - ADR-039
  - TECH-072
  - ADR-043
  - TECH-008
  - MS-037
  - ADR-0020
status: Done
assignee:
  - rust-dev
milestone: MS-037
dependencies:
  - TASK-282
ordinal: 40
documentation:
  - tests/no_drift.rs
  - tests/document_type_no_drift.rs
  - tests/no_descriptor_leaks.rs
  - src/record/mod.rs
  - src/init.rs
  - src/config.rs
  - src/registry.rs
  - tests/browser_config_types_cli.rs
  - Cargo.toml
  - Cargo.lock
  - scripts/package-out.sh
  - scripts/build-package.sh
  - scripts/build-deb.sh
  - tests/build_package_guard.rs
type: implementation
---

## Description
## Context
The refactor is one deliverable with three category handlers (`ArtifactCategory`: `Backlog`, `Decision`, `Document`) and one descriptor-driven document engine (ADR-036, ADR-039, ADR-043; TECH-072). Existing guards already cover much of this: `tests/document_type_no_drift.rs` D-13..D-18, `tests/no_descriptor_leaks.rs`, `src/record/mod.rs::read_path_guard_tests`, and `tests/browser_config_types_cli.rs`. This task extends them to the MS-037 invariants and the release gates.

## Scope
Extend the production-line guards (skip `#[cfg(test)]`) so the default `cargo test` run fails with `file:line: symbol` in the message when any of the following reappears:

1. `TypedHandler` has a variant beyond `Task`, `Milestone`, `Release`, `Decision` (D-16 exists) or a document `TypedHandler`/`RequirementKind` appears in production (D-17 exists). Keep the Backlog and Decision variants explicit.
2. `ArtifactCategory` has a variant beyond `Backlog`, `Decision`, `Document` (`src/registry.rs:25-29`). Add a D-19 guard.
3. A per-type document module returns: keep D-13 over the deleted modules and keep `src/ui_mockup.rs`.
4. A per-type conflict or not-found renderer returns: no `conflict_page_*`/`*_not_found_page` definition or call site beyond `render::document_conflict_page` and `render::document_not_found_page`. The negative list in `tests/browser_config_types_cli.rs:690-728` is the source; move or duplicate it into the guard suite so the guard owns the invariant.
5. A document slug literal selects behaviour anywhere in `src/` except `SEED_DOCUMENT_TYPES` in `src/config.rs`; tests and `docs/` exempt. D-05 currently scans only `src/browser/`, `src/cli/`, and `src/document.rs`. Widen it to all of `src/`, and record the allowlist. Known production offender: `src/init.rs:21-34` (`DOCUMENT_ARTIFACTS`, a literal 11-slug list). Per the ADR-0020 clarification of 2026-11-23, `DOCUMENT_ARTIFACTS` must derive from `SEED_DOCUMENT_TYPES` in seed order; no allowlist exception is recorded, and a test must assert position-by-position equality with the seed.
6. A configuration load or registry construction returns on a per-record/per-request read path: keep `src/record/mod.rs::read_path_guard_tests` and extend its module list when a new read module appears.
7. A new crate or dependency edge: parse `Cargo.toml` (dependencies, dev-dependencies, build-dependencies) and `Cargo.lock`, and fail on any package or edge beyond the recorded baseline (`clap`, `serde`, `serde_json`, `yaml_serde`; dev `serde_json`; 36 `Cargo.lock` packages).
8. The release binary exceeds 12 MiB: record `stat -c %s target/release/papyrus` and fail above 12,582,912 bytes (TECH-008 section 2).
9. The packaging `--out` guard implements ADR-043 decision 7 as amended 2026-11-23 on the canonicalised path (`realpath -m`): an in-repository `--out` is permitted only when it is strictly below `<repo>/.tmp/` (scratch, removed before the work ends) or strictly below `<repo>/builds/` with at least one segment after `builds/` (publish location, git-ignored, never staged, committed, or published); refuse `<repo>`, `<repo>/builds` itself, and every other in-repository path. Keep the outside-repository refusals (filesystem root, system directories, `$HOME` and its subtree, the repository root, and any parent of the repository root). Update the usage text and diagnostics in `scripts/package-out.sh`, `scripts/build-package.sh`, and `scripts/build-deb.sh`, and extend `tests/build_package_guard.rs` to cover both permitted roots and each refusal with exit code 2.

## Out of scope
- Do not implement the document engine, CLI, or browser. Packaging changes are limited to the `--out` guard, its usage text and diagnostics, and `tests/build_package_guard.rs` as described here.
- Do not duplicate TASK-287: TASK-287 runs these guards and records the release evidence; this task makes them exist and pass.
- Do not edit requirements, rules, ADRs, specifications, mockups, test cases, dependencies, ordinals, or milestone scope.

## Assumptions
- The 36-package `Cargo.lock` set is captured at refinement time; the guard records the baseline in its source.
- The 12 MiB guard runs against an existing `target/release/papyrus` produced by `cargo build --release`.
- `src/init.rs::DOCUMENT_ARTIFACTS` is an enumeration, not dispatch; it must derive from `SEED_DOCUMENT_TYPES` in seed order (ADR-0020 clarification, 2026-11-23), and the existing init test continues to assert order and set equality.
- `builds/` is git-ignored, so neither permitted in-repository root (`builds/` below one segment and `.tmp/`) is a tracked repository location.

## Definition of done
The new and extended guards fail on a planted violation and pass on the current tree; `DOCUMENT_ARTIFACTS` derives from `SEED_DOCUMENT_TYPES` in seed order and its test passes; the packaging guard accepts both permitted roots and refuses every other in-repository path with exit code 2; `cargo test`, `cargo clippy --all-targets -- -D warnings`, and `cargo fmt --check` are green; a comment names each guard, its file, and the failure message format.

## Acceptance Criteria
- [ ] #1 D-16 and D-17 still assert exactly `Task`, `Milestone`, `Release`, `Decision` and no document `TypedHandler` or `RequirementKind`; the failure message names the offending file and line.
- [ ] #2 A new guard asserts `ArtifactCategory` has exactly `Backlog`, `Decision`, `Document` and no other variant.
- [ ] #3 D-13 still fails on any deleted per-type document module and still requires `ui_mockup`.
- [ ] #4 A guard fails on any per-type conflict or not-found renderer definition or call site beyond `document_conflict_page` and `document_not_found_page`; the `browser_config_types_cli` negative list is not weakened.
- [ ] #5 The slug guard scans all of `src`, allows document slug literals only in `SEED_DOCUMENT_TYPES` in the config module, and fails with `file:line: text` on a planted `match` on a slug literal in a new module.
- [ ] #6 `DOCUMENT_ARTIFACTS` in `crate::init` derives from `SEED_DOCUMENT_TYPES` in seed order; a test asserts position-by-position equality with the seed, no second document slug list remains, the slug guard is green, and the init order and set tests still pass.
- [ ] #7 The read-path guard still fails when `load_effective_config` or `ArtifactRegistry::build` appears in a read module, and covers every top-level read module and the record module tree.
- [ ] #8 A no-new-crate guard fails when a dependency is added to `Cargo.toml` (including dev and build) or a package or edge is added to `Cargo.lock`, naming the added crate.
- [ ] #9 Each new guard has a planted-violation self-test, and the whole guard suite passes on the current tree.
- [ ] #10 `cargo test`, `cargo clippy --all-targets -- -D warnings`, and `cargo fmt --check` are green.
- [ ] #11 A binary-budget guard records the release `papyrus` binary size and fails above 12,582,912 bytes.
- [ ] #12 The packaging `--out` guard accepts an in-repository path strictly below `<repo>/.tmp/` or strictly below `<repo>/builds/` with at least one segment after `builds/`, and refuses `<repo>`, `<repo>/builds` itself, and every other in-repository path with exit code 2.
- [ ] #13 `scripts/package-out.sh`, `scripts/build-package.sh`, and `scripts/build-deb.sh` state the permitted `.tmp/` and `builds/<segment>/...` roots and each refusal in their usage text and diagnostics.
- [ ] #14 `tests/build_package_guard.rs` covers both permitted roots and each refusal (repository root, `builds/` itself, other in-repository paths, and the outside-repository refusals), and asserts a refused path is never created or cleaned.
## Comments
- **rust-dev**: TASK-283 evidence: guards exist and pass on the current tree; status left In-Progress; no git commit by instruction. Guards: `tests/document_type_no_drift.rs` D-05 scans every production `src/**.rs` line and allows document slug literals only inside the `SEED_DOCUMENT_TYPES` declaration block in `src/config.rs` (range computed by bracket depth), reporting `file:line: text`; a planted `match` on `"req"` in a new module reports `src/new_module.rs:3: "req" => 1,`, and a second test rejects a planted second slug list. D-17 keeps the no-document-TypedHandler/RequirementKind scan. D-16 now reports `src/registry.rs:<line>: TypedHandler must have exactly the four explicit backlog/decision variants`; D-19 asserts `src/registry.rs:<line>: ArtifactCategory is exactly Backlog, Decision, Document`; both share `enum_variants` with a planted-variant self-test. D-20 scans definitions and call sites for any `conflict_page`/`not_found_page` token and allows only `document_conflict_page`, `document_not_found_page` and `backlog_decision_conflict_page` (the existing `browser_config_types_cli` negative list is unchanged). D-13 still fails on deleted per-type modules and requires `ui_mockup`. The `record/mod.rs` read-path guard still auto-covers every top-level read module and the record tree. `tests/no_drift.rs` adds a no-new-crate guard parsing Cargo.toml dependencies/dev-dependencies/build-dependencies (baseline clap, serde, serde_json, yaml_serde; dev serde_json; no build deps) and the full 36-package Cargo.lock set, naming added and removed crates, with planted manifest and lockfile additions rejected; it adds a binary-budget guard failing above 12,582,912 bytes with a planted oversized-size self-test, and measured `target/release/papyrus` is 8,245,400 bytes. `DOCUMENT_ARTIFACTS` in `src/init.rs` is no longer a literal list: `const fn document_artifacts_from_seed` derives it from `SEED_DOCUMENT_TYPES` in seed order, the existing position-by-position, set and order tests still pass, and a new test fails if init production restates any slug. Packaging ADR-043 decision 7 as amended 2026-11-23: `scripts/package-out.sh` canonicalises with `realpath -m` and permits an in-repository `--out` only when strictly below `<repo>/.tmp/` or strictly below `<repo>/builds/` with a non-empty remainder, refusing `<repo>`, `<repo>/builds` itself, every other in-repository path, the filesystem root, system directories, `$HOME` and its subtree (both permitted roots stay usable under `$HOME`) and any parent of the repository root with exit 2 before creation or cleaning. The `package-out.sh` header, `package_out_policy` and refusal messages, `build-package.sh --help` and `build-deb.sh --help` state both permitted roots and each refusal. `tests/build_package_guard.rs` validates and prepares both permitted roots, refuses the repository root, `builds` itself, other in-repository paths, canonicalised escapes (`builds/../x`, `builds/.`), the repository-root parent, `/`, `/usr/...` and `$HOME` plus a `$HOME` subdirectory with `--force`, asserts refused paths are never created, and asserts existing refused directories keep their markers; a usage-text test asserts the three scripts state `.tmp/`, `builds/<segment>`, the repository-root and outside-repository refusals. Checks: `env -u EDITOR -u VISUAL cargo test` 1414 passed, 0 failed, 27 ignored (the unset `EDITOR`/`VISUAL` is required by the existing `config_editor_json_null_when_unset` test); `cargo clippy --all-targets -- -D warnings` clean; `cargo fmt --check` clean; `cargo build --release --locked` built; no crate added and `cargo audit` is not installed in this environment.
- **orchestrator**: Acceptance met: DOCUMENT_ARTIFACTS derived from the seed, packaging out guard permits only .tmp and builds with a version segment with usage text in all three scripts, all-src slug scan, three-category and renderer guards with planted self-tests, no-new-crate and 12 MiB binary guards; 1414 tests pass.
