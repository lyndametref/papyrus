---
id: TASK-287
title: "Run the quality gate audit and collect release evidence"
created_date: '2026-11-20T17:44:13Z'
updated_date: '2026-11-23T11:29:26Z'
labels: []
references:
  - NFR-007.001
  - NFR-007.002
  - NFR-007.003
  - TECH-069
  - REL-001
  - ADR-043
  - TECH-008
  - AUD-150
status: ToDo
assignee:
  - quality-gatekeeper
milestone: MS-037
dependencies:
  - TASK-285
  - TASK-286
  - TASK-316
ordinal: 80
documentation:
  - tests/no_drift.rs
  - tests/document_type_no_drift.rs
  - tests/no_descriptor_leaks.rs
  - src/record/mod.rs
  - tests/build_package_guard.rs
  - tests/benchmark_baseline.rs
  - docs/benchmarks/MS-036-results.md
  - Cargo.toml
  - Cargo.lock
  - docs/audits/AUD-145.md
  - docs/audits/AUD-148.md
type: audit
---

## Description
## Context
The release decision needs one place that shows every gate passed. MS-035 and MS-036 gates are recorded in AUD-146, AUD-149, and AUD-150; this task runs the release-wide quality gate on the MS-037 tree and collects the evidence for REL-001.

## Scope
Run and record, on the pinned toolchain (rust 1.99.1), against the committed verified tree:
- `cargo test` (full suite), `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`.
- `tests/no_drift.rs`, `tests/document_type_no_drift.rs`, `tests/no_descriptor_leaks.rs`, `src/record/mod.rs::read_path_guard_tests`, `tests/browser_config_types_cli.rs`, `tests/build_package_guard.rs`.
- `cargo test --release --test benchmark_baseline -- --ignored --nocapture` (NFR-007.001, NFR-007.002, NFR-007.003) or a citation of `docs/benchmarks/MS-036-results.md` when the host is unchanged; record the host and the hardware deviation.
- `stat -c %s target/release/papyrus` against 12,582,912 bytes.
- Production LOC against the 28,000 internal target, explicitly informational and never a gate (TECH-008 section 2).
- Test counts, with the note that counts are not a gate.
- A no-new-crate check over `Cargo.toml` and `Cargo.lock` (dev and build dependencies included; current baseline: clap, serde, serde_json, yaml_serde; 36 `Cargo.lock` packages).
- NFR-007.001, NFR-007.002, and NFR-007.003 results from the MS-036 run, with the 6 physical and 12 logical core, 15.3 GiB hardware deviation recorded beside them; do not state that NFR-007.001 or NFR-007.002 is satisfied.
- Record hygiene: resolve the duplicate AUD-145 and AUD-148 records (one in the active audit directory, one in the archived audit directory) so one record per ID remains and `papyrus check` reports no `duplicate-id` finding for them; record which copy was retained.

## Out of scope
- Do not fix product code, requirements, rules, ADRs, specifications, mockups, or test cases. A failed gate is a finding with a disposition.
- Do not produce the release package (TASK-288) and do not edit MS-036 text.

## Definition of done
One audit record of type `quality` referencing REL-001 contains the gate table, the binary size, production LOC (informational), test counts, the NFR-007.001 to NFR-007.003 results with the hardware deviation, the no-new-crate result, the duplicate-audit resolution, severity counts, and a verdict. The record is referenced from a task comment; any blocking finding is reported to the orchestrator.
## Acceptance Criteria
- [ ] cargo test result is recorded with pass or fail and counts; clippy with -D warnings and cargo fmt --check are clean.
- [ ] All guard suites and the characterization and equivalence suites pass and are named in the evidence.
- [ ] The release binary size is recorded and is at most 12,582,912 bytes.
- [ ] Production LOC is recorded against the 28,000 internal target and is explicitly labelled informational and not a gate.
- [ ] The no-new-crate result is recorded: Cargo.toml (including dev and build) and Cargo.lock add no crate or dependency edge.
- [ ] NFR-007.001 and NFR-007.002 measured p50 and p95 are recorded with the host and the hardware deviation (6 physical and 12 logical cores, 15.3 GiB versus the 4 vCPU and 8 GiB reference) beside every result, and neither NFR is stated as satisfied.
- [ ] NFR-007.003 cache-class results are recorded: immutable vendor assets, revalidated app assets, and no-store HTML and JSON.
- [ ] The duplicate AUD-145 and AUD-148 records are resolved to one record per ID, and papyrus check reports no duplicate-id finding for either.
- [ ] Exactly one audit record of type quality referencing REL-001 records the evidence, severity counts, and verdict.
- [ ] No blocking finding is open; each blocker or high is reported to the orchestrator.
