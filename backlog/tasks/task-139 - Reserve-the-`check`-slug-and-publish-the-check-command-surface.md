---
id: TASK-139
title: Reserve the `check` slug and publish the check command surface
status: Done
assignee:
  - '@rust-dev'
created_date: '2026-11-15T16:12:00Z'
updated_date: '2026-11-15T18:27:00Z'
labels:
  - rust
  - cli
  - config
milestone: m-21
dependencies:
  - TASK-137
  - TASK-138
references:
  - docs/requirements/business/REQ-07.002.md
  - docs/requirements/business/REQ-01.004.md
  - >-
    docs/architecture/adr/ADR-0029-read-only-corpus-diagnostics-with-duplicate-id-repair.md
  - docs/specifications/project-diagnostics-check.md
  - src/cli/builtin.rs
  - src/cli/tree.rs
  - src/config.rs
  - src/check/mod.rs
  - tests/cli_help_golden.rs
priority: high
ordinal: 41520
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why.** `check` is a built-in command name, so a configured document type must never take it (REQ-07.002), and the command needs a published surface that matches accepted ADR-0029 decision 1. TASK-137/138 landed the working command (`src/check/*`, `BuiltinCmd::Check { fix, force }`), but `check` is still missing from `cli::BUILTIN_COMMANDS` — so configuration validation still accepts a `check` document slug — and `papyrus check --help` still prints only the one-line `about` plus the flag list.

**What.** Add `check` to the single reserved-name list, freeze the help text, and lock both with tests and goldens.

**In scope.**
1. `src/cli/builtin.rs`: add `BuiltinEntry { name: "check" }` to `BUILTIN_COMMANDS`, after `config`. That array is the only reserved-name source — `src/config.rs` (reserved-name loop in `Config::validate`) and `src/cli/tree.rs` (pre-build guard) both read it. Do not add a second list or hard-code `"check"` anywhere else.
2. The resulting diagnostics, already formatted by the existing code as `document_types.<slug>: slug conflicts with built-in type <builtin>`, must be exactly:
   - `document_types.check: slug conflicts with built-in type check`
   - `document_types.Check: slug conflicts with built-in type check`
   - `document_types.CHECK: slug conflicts with built-in type check`
   They surface as `AppError::Validation` (exit 4) from every fail-closed command, and as the `detail` of a `config-invalid` finding from `check`.
3. `papyrus check --help` prints the frozen long description in `docs/specifications/project-diagnostics-check.md` §1.1 verbatim: synopsis, `--fix`, `--force`, the seven finding kinds, exit statuses 0/4/1/2, tolerant configuration behaviour, and the read-only default.
4. The command tree built with no registry (the `lib.rs::run` path used for `check`) exposes `check` and its full help; the tree built from a registry still lists `check` once as a built-in and never as a document-family command.
5. Tests proving AC1-AC5, and goldens for `papyrus --help` and `papyrus check --help`.

**Where the surface is documented.** The command surface, the seven finding kinds, and the exit-status table live in `docs/specifications/project-diagnostics-check.md` (sections 1, 4-5, 8); §1.1 fixes the help text this task publishes. README and the business rules (BR-001, BR-005, BR-016) are TASK-140.

**Out of scope.** README and business-rule prose (TASK-140); browser/packaging specification prose (TASK-148); requirement text, ADRs, release notes, and man-page content (ADR-0021 is still Proposed); new flags, subcommands, or output formats; any change to the diagnostic behaviour landed in TASK-137/138.

**Assumptions.**
- Assumption: ADR-0029 decision 1 flag names (`--fix`, `--force`) and exit codes 0/4/1/2 are final, so the help text can be frozen now.
- Assumption: clap 4 is built without the `wrap_help` feature, so `long_about` lines print exactly as authored and the pinned block is the golden text.
- Assumption: the short `about` (`Run read-only project diagnostics`) is unchanged, so `tests/golden/cli-help/top.json` stays byte-identical.
- Assumption: no default type or CLI-help fixture uses the slug `check`, so the new reservation breaks no existing project fixture.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 A project whose configuration declares a document type with the slug `check`, `Check`, or `CHECK` is rejected with a validation outcome (exit 4) whose diagnostic line is exactly `document_types.<key>: slug conflicts with built-in type check`, where `<key>` is the configured spelling.
- [x] #2 The reserved-name comparison is case-insensitive and derives from `cli::BUILTIN_COMMANDS`, which now contains `check`; on such a project `papyrus config unset document_types.<key>` and `papyrus init --force` both still succeed, and after either repair the fail-closed commands run again.
- [x] #3 `papyrus check --help` exits 0 and prints the specification §1.1 long description verbatim, covering the generated synopsis `Usage: papyrus check [OPTIONS]`, `--fix`, `--force`, the seven finding kinds `duplicate-id`, `self-dependency`, `dependency-cycle`, `dangling-milestone`, `dangling-release`, `frontmatter-conflict`, `config-invalid`, and exit statuses 0, 4, 1, and 2.
- [x] #4 The same help text states the read-only default (nothing is written without `--fix`) and the tolerant configuration behaviour (an invalid configuration is reported as `config-invalid` findings, and the document-type checks are named as skipped).
- [x] #5 `papyrus::cli::tree(None)` exposes a `check` subcommand whose rendered help contains the AC3/AC4 text; the tree built from a registry lists `check` exactly once and never as a document-family command; a `check` document descriptor is rejected by the tree guard with the AC1 diagnostic.
- [x] #6 `tests/golden/cli-help/check.json` is added, `tests/golden/cli-help/top.json` still matches, and `cargo test --test cli_help_golden` passes without `PAPYRUS_UPDATE_CLI_HELP`.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 `check` is in `cli::BUILTIN_COMMANDS`; configuration validation and the tree guard reject `check`/`Check`/`CHECK` with the pinned diagnostics; no second reserved-name list exists.
- [x] #2 `papyrus check --help` matches specification §1.1 verbatim, and the short `about` and the `--fix`/`--force` argument help are unchanged.
- [x] #3 `tests/golden/cli-help/check.json` is added and the whole golden set passes without regeneration; `top.json` is byte-identical and no other golden changed.
- [x] #4 New and updated tests cover AC1-AC5; `cargo test` passes, and `cargo fmt --check` plus `cargo clippy --all-targets -- -D warnings` are clean.
- [x] #5 No new crate or dependency; no browser route, no JSON API, no library surface beyond `run`; `--fix`/`--force` behaviour and the existing check tests and goldens are unchanged.
- [x] #6 No README, business-rule, requirement, or ADR edit; only `docs/specifications/project-diagnostics-check.md` §1.1 and its status line changed.
<!-- DOD:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Contract: `docs/specifications/project-diagnostics-check.md` §1.1 (frozen help text) and §8 (exit codes); REQ-07.002 (reserved names, case-insensitive); ADR-0029 decisions 1, 3, and 11. No requirement, ADR, README, or business-rule file is edited.

1. Reserved name — `src/cli/builtin.rs`
   - Add `BuiltinEntry { name: "check" }` to `BUILTIN_COMMANDS`, after `config`, keeping the array in clap enum order.
   - No other production change: `src/config.rs` already compares with `eq_ignore_ascii_case` and formats the diagnostic; `src/cli/tree.rs` already applies the same guard to document descriptors. Both consume the array.
   - Exact diagnostics to assert, character for character:
     - `document_types.check: slug conflicts with built-in type check`
     - `document_types.Check: slug conflicts with built-in type check`
     - `document_types.CHECK: slug conflicts with built-in type check`
   - `Config::validate` returns every diagnostic joined by `\n`; fail-closed callers wrap the whole string in `AppError::Validation` (exit 4), while `check` splits it into one `config-invalid` finding per line.

2. Help text — `src/cli/builtin.rs`
   - On the `Check` variant, add `#[command(long_about = "<block from spec §1.1>")]` using a raw string and copy the block byte for byte. Keep the existing doc-comment `about` (`Run read-only project diagnostics`) and the existing argument help texts (`Repair duplicate record IDs.`; `Skip the repair confirmation prompt; requires --fix.`).
   - Do not add `override_usage`, `after_help`, or a version note. Keep `requires = "fix"` on `--force`; do not hand-roll the `--force`-without-`--fix` check.

3. Tree guard — no production change expected
   - `src/cli/tree.rs` already rejects a document descriptor whose slug matches a built-in case-insensitively, with `document_types.<slug>: slug conflicts with built-in type <builtin>`. Cover it with the step-5 test; do not duplicate the list or move the guard.

4. Goldens — `tests/cli_help_golden.rs`, `tests/golden/cli-help/`
   - Add one case: `("check", ["check", "--help"])`, placed after `top`/`version`, so `golden_snapshot_set_is_complete` keeps its exact-count assertion.
   - Regenerate with `PAPYRUS_UPDATE_CLI_HELP=1 cargo test --test cli_help_golden`. `check.json` is the only intended addition; `top.json` must stay byte-identical — if it moves, stop and check whether the short `about` was edited.
   - No other golden in that directory changes; `err-*`, `json-*`, `slug-*`, and `verb-*` stay as they are.

5. Tests
   - `src/config.rs`, `reserved_slug_conflicts_are_case_insensitive`: add `("check","check")`, `("Check","check")`, `("CHECK","check")` and compare full diagnostic lines, not substrings.
   - `tests/tree_rebuild.rs` (or a sibling test module): `tree(None)` yields a command whose `find_subcommand("check")` is `Some` and whose `render_help()` contains `Usage: papyrus check` and `config-invalid`; the tree from the default registry lists `check` exactly once; a `Config` carrying a `check` document entry produces a registry whose `tree(Some(&reg))` returns the AC1 diagnostic.
   - `tests/check_surface_cli.rs` (new; reuse the `tests/check_cli.rs` project-harness pattern): on an initialized project, hand-write a `document_types.check` entry (prefix `CHECK`, directory `checks/`); assert `papyrus task list` exits 4 and stderr carries the pinned diagnostic; `papyrus check --json` exits 4 with a `config-invalid` finding whose `detail` is the same line; `papyrus config unset document_types.check` exits 0 and persists; the fail-closed command then exits 0. Separately, `papyrus init --force` on a `check`-slug project exits 0 and rewrites the default configuration.
   - `tests/check_cli.rs`: keep the existing `missing_configuration_is_reported_as_a_finding` help assertion and add one that `check --help` with an invalid configuration contains the frozen finding-kind list and the exit-status sentence.

6. Specification
   - `docs/specifications/project-diagnostics-check.md` §1.1 is already written by this refinement. Update only the `Status` line after the golden lands; nothing else in the specification changes.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
check added to cli::BUILTIN_COMMANDS (single reserved-name source, case-insensitive), frozen CHECK_LONG_ABOUT from spec 1.1, new tests/golden/cli-help/check.json, tests/check_surface_cli.rs for three spellings plus config unset / init --force repair, tree_rebuild tests for the registry-free and registry trees. Quality review: pass; 914 tests pass, clippy/fmt clean, goldens pass without regeneration, top.json byte-identical.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
The check slug is now reserved: document_types.check/Check/CHECK fail validation with 'document_types.<key>: slug conflicts with built-in type check', and check reports the same line as a config-invalid finding instead of aborting. Repair works via config unset or init --force. papyrus check --help publishes the frozen surface (synopsis, --fix, --force, seven finding kinds, exit 0/4/1/2, tolerant configuration, read-only default) and remains available with no or broken configuration because the command tree is built without a registry. Locked by the new check.json help golden plus surface and tree tests; top.json unchanged.
<!-- SECTION:FINAL_SUMMARY:END -->
