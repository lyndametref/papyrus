---
id: TASK-107
title: Update init defaults and seed document statuses
status: Done
assignee:
  - rust-dev
created_date: '2026-11-13T21:26:00Z'
updated_date: '2026-11-14T14:45:00Z'
labels:
  - rust
milestone: MS-018
dependencies:
  - TASK-100
  - TASK-104
references:
  - REQ-07.002
  - REQ-01.006
  - REQ-07.001
  - REQ-05.008
  - REQ-01.001
  - ADR-0019
  - ADR-0024
documentation:
  - docs/specifications/project-initialization-and-configuration.md
  - docs/specifications/interactive-init-walkthrough-for-artifact-activation.md
  - docs/requirements/business/REQ-07.002.md
  - docs/requirements/business/REQ-01.006.md
  - >-
    docs/architecture/adr/ADR-0019-fail-closed-document-type-configuration-validation.md
priority: high
ordinal: 41060
---


## Description
**Why.** `Config::default()` still seeds the removed `deploy` document type, so every generated configuration and every configuration that relies on the default document-type map (a file with no `document_types` key) offers a type REQ-07.002 removed from the default set. The generated configuration also declares no `statuses` for the default document types, so the document status model stays implicit (`active`/`archived` fallback in the descriptor) instead of the declared, inspectable and editable vocabulary REQ-07.001 requires. `papyrus init` is the surface that writes the default set (REQ-07.002), and its interactive walkthrough (REQ-01.006) must offer exactly that set and disable entries from it.

**What changes.**

- `Config::default()` document set: remove `deploy`; keep the other eleven entries with their current prefixes and directories (`req`/`REQ`/`requirements/business/`, `nfr`/`NFR`/`requirements/non-functional/`, `glossary`/`GLO`/`glossary/`, `rule`/`RUL`/`business-rules/`, `ui-spec`/`UI`/`specifications/ui/`, `ui-mockup`/`MOCK`/`specifications/ui/mockups/`, `audit`/`AUD`/`audit-records/`, `tech-spec`/`TECH`/`technical/`, `use-case`/`UC`/`use-cases/`, `test-case`/`TC`/`test-cases/`, `test-report`/`TR`/`test-reports/`).
- Seed `statuses: [active, archived]` in that order on every front-matter-backed default entry: `req`, `nfr`, `glossary`, `rule`, `ui-spec`, `audit`, `tech-spec`, `use-case`, `test-case`, `test-report`.
- `ui-mockup` keeps no `statuses` declaration (a `statuses` key on it is invalid configuration, REQ-05.008).
- No default entry declares `default_status`; new records start at the effective default `active`.
- `init`'s selectable document list drops `deploy` (eleven entries); core artifacts task/milestone/release and decision types `adr`/`bdr` are unchanged.
- `init` persists the generated configuration through the existing `config::to_config_yaml` renderer introduced by TASK-111, so seeded lists are written as flow sequences `[active, archived]` (the same form `config set` writes and `config get`/`config list` print), not as block sequences.
- Interactive walkthrough activation: disabling a document type writes `enabled: false` and keeps that entry's prefix, directory, and seeded status list; it creates no directory and touches no existing files. Written activation equals the list state at confirm time.
- Non-interactive `init` writes the same default set, all entries enabled.
- `papyrus init --force` replaces any existing configuration file — including one that fails ADR-0019 validation — with this generated default set, and the written file passes the validator. Without `--force`, an existing file still fails with exit 4 and writes nothing.
- Browser configuration summary: drop `deploy` from the fixed document-type display order, so a user-added `deploy` renders through the generic sorted "unknown enabled keys" fallback instead of a reserved slot.
- Test fixtures that currently exercise a generic document handler with `deploy` through the merged default set must declare the `deploy` entry explicitly in their configuration; this keeps them working and doubles as coverage that `deploy` remains addable as an ordinary user-defined type.

**In scope.** `src/config.rs` (default map + its unit tests); `src/init.rs` (selectable artifact list, generated YAML rendering, init tests); `src/browser/render.rs` (document-type display order); affected tests in `src/config.rs`, `src/document.rs`, `tests/cli.rs`, `tests/cli_dispatch.rs`, `tests/browser_cli.rs`; the two feature specifications named below.

**Out of scope.** Business-rule text updates (TASK-115); README/upgrade notes (TASK-116); end-to-end release and package verification (TASK-117); changes to document record behavior, archive/status semantics, or the ADR-0019 validator rules; `config set`/`get`/`list` semantics (TASK-111); walkthrough presentation and navigation (TASK-104); adding any new document type or changing existing prefixes/directories.

**Assumptions.**

- Assumption: `Config::default()` is the single source of the default set, so a configuration file without a `document_types` key receives the same eleven entries and the same seeded status lists as `init` writes (ADR-0019 decision 8: whole-map default when the key is absent).
- Assumption: the seeded status order is exactly `active`, `archived`, and that order is preserved in the file and in `config list`.
- Assumption: statuses are seeded on disabled entries too, because the walkthrough's disable only flips `enabled`.
- Assumption: dropping `deploy` from the browser display order is cosmetic; a user-added `deploy` is still shown, in the sorted fallback position.
- Assumption: existing configuration files that contain a `deploy` entry and/or omit document `statuses` continue to load and validate exactly as today; only the generated defaults change.

**Dependencies.** TASK-105 (artifact registry and type descriptors, completed), TASK-109 (fail-closed document-type validation, completed), and TASK-111 (`config::to_config_yaml` flow renderer, completed). Decisions: ADR-0019 (fail-closed validation, default-map semantics, `ui-mockup` status restriction, `init --force` repair), ADR-0024 (flow-sequence rendering of list-valued keys).

## Acceptance Criteria
- [x] `Config::default().document_types` contains exactly `req`, `nfr`, `glossary`, `rule`, `ui-spec`, `ui-mockup`, `audit`, `tech-spec`, `use-case`, `test-case`, `test-report`; each keeps its current prefix and directory; `deploy` is absent.
- [x] Every front-matter-backed default entry (`req`, `nfr`, `glossary`, `rule`, `ui-spec`, `audit`, `tech-spec`, `use-case`, `test-case`, `test-report`) has `statuses == Some(vec!["active", "archived"])` in that order; `ui-mockup` has `statuses == None`; no default entry has `default_status`.
- [x] `papyrus init <name> --no-git` on an empty directory writes `papyrus.config.yml` with exactly the eleven document-type slugs; each entry except `ui-mockup` contains `statuses: [active, archived]` in flow form; `ui-mockup` has no `statuses` key; no entry anywhere has a `default_status` key.
- [x] The file written by `init` passes `Config::validate` (ADR-0019); `papyrus config get document_types.req.statuses` prints `[active, archived]` and `papyrus config list` prints the same form for every seeded entry.
- [x] The interactive walkthrough list offers core task/milestone/release, decisions `adr`/`bdr`, and exactly the eleven default document types in configuration-key order; `deploy` is not offered and cannot be selected.
- [x] Disabling a document type in the walkthrough writes `enabled: false` for that slug, keeps its prefix, directory, and `[active, archived]` statuses, creates no directory for it, and the activation written equals the list state at Enter.
- [x] A configuration file with no `document_types` key yields the same eleven-entry default set with the seeded statuses (whole-map default, ADR-0019 decision 8), and an explicitly declared `deploy` entry is accepted by the validator and dispatched as a user-defined document type.
- [x] `papyrus init --force` over a configuration that fails ADR-0019 validation exits 0, replaces the file with the generated default set, leaves no invalid entry behind, and the result validates; without `--force` an existing file still exits 4 and writes nothing.
- [x] Tests assert the exact default map, the seeded status lists and their order, the absent `statuses` on `ui-mockup`, the absence of `deploy`, walkthrough disabling, and `--force` repair; existing assertions and fixtures that assumed `deploy` in the defaults (`src/config.rs` init-force test, `src/document.rs` generic-handler fixtures, `tests/cli_dispatch.rs` default-help slug list, `tests/browser_cli.rs` document-type summaries) are updated.
- [x] `docs/specifications/project-initialization-and-configuration.md` and `docs/specifications/interactive-init-walkthrough-for-artifact-activation.md` describe the eleven-entry default set, the seeded document status model, and `init --force` repair.

## Definition of Done
- [x] #1 All ten acceptance criteria verified with objective evidence (test name or command output recorded in the ticket).
- [x] #2 `cargo test` passes, including the updated `src/config.rs`, `src/init.rs`, `src/document.rs`, `tests/cli.rs`, `tests/cli_dispatch.rs`, and `tests/browser_cli.rs` suites.
- [x] #3 `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` are clean.
- [x] #4 `grep -n '"deploy"' src/config.rs src/init.rs src/browser/render.rs` returns no default-set occurrence.
- [x] #5 Both feature specifications list the eleven-entry default set and the seeded status model, with no remaining `deploy` entry in the init default set.
- [x] #6 No requirement or business-rule file under `docs/requirements/` or `docs/business-rules/` was modified.


## Implementation Notes
1. `src/config.rs`: in `Config::default()`, remove `("deploy", "DEP", "architecture/deployment/")` from the document array; build each entry with `statuses: Some(vec!["active".into(), "archived".into()])` for the ten front-matter types and `statuses: None` for `ui-mockup`; leave `default_status: None` everywhere. Extend the config unit tests with an exact default-map test and per-entry status assertions.
2. `src/init.rs`: reduce `DOCUMENT_ARTIFACTS` to the eleven slugs (drop `deploy`) and `SELECTABLE_ARTIFACT_COUNT` follows from the constants; update the row-order test expectation. Keep `initial_activation()` deriving from `Config::default()`.
3. `src/init.rs` persistence: after `config.validate(Some(&project_dir))`, serialize with `yaml_serde::to_value(&config)`, render with `crate::config::to_config_yaml`, and write that string instead of `yaml_serde::to_string`, mapping both failures to `AppError::Runtime`; keep full-file replacement, the existing conflict check, and directory creation logic. Confirm `--force` output passes the validator.
4. `src/browser/render.rs`: remove `"deploy"` from `known_doc_order` so it is treated as an unknown enabled key (sorted fallback).
5. Test fixtures: in `src/document.rs` tests, make the shared `default_config` helper declare the `deploy` entry explicitly (`prefix: DEP`, `directory: architecture/deployment/`, `enabled: true`, `statuses: [active, archived]`) so every existing generic-handler test keeps using `deploy` as a user-defined type; add one assertion that the declared deploy entry validates and dispatches.
6. Update affected expectations: `src/config.rs::init_force_repairs_an_invalid_configuration` asserts `deploy` is absent (and the eleven slugs are present); `tests/cli_dispatch.rs::generated_help_lists_enabled_slugs_for_default_config` drops `deploy`; the two `tests/browser_cli.rs` enabled-document-type summaries drop `deploy`.
7. Add tests: exact default map; non-interactive `init` file content (eleven slugs, flow-form statuses, no `ui-mockup` statuses, no `default_status`, deploy absent, validates); scripted-key walkthrough run asserting `deploy` is not rendered and disabling a type keeps statuses; `init --force` over an invalid file asserting the generated set and successful validation.
8. Update the two specifications listed in AC #10 (already revised during refinement; verify the text still matches the implementation).
9. Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test`.

Test plan.

- Unit, src/config.rs: exact default document-type map; per-entry seeded statuses and order; `ui-mockup` without statuses; no `default_status`; `Config::default()` validates.

- Unit, src/init.rs: constants and rendered list contain the eleven default document types and no `deploy`; scripted-key run with one document toggled off asserts `enabled: false` plus retained prefix, directory, and statuses; non-interactive run asserts the written YAML text (eleven slugs, flow-form `statuses: [active, archived]`, no `ui-mockup` statuses, no `default_status`, no `deploy`).

- Integration, tests/cli.rs: `init` on an empty directory produces a file that parses and validates; `init --force` over a validator-rejecting file exits 0, validates, and contains no leftover invalid entry; without `--force` exit 4 and the file is byte-identical.

- Integration, tests/cli_dispatch.rs and tests/browser_cli.rs: default help slug list and the configuration summary show the eleven slugs and no `deploy`; a user-declared `deploy` entry is still dispatched/rendered.

- Regression: the src/document.rs generic-handler suite passes with `deploy` declared explicitly in its fixture.

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Default set is exactly req, nfr, glossary, rule, ui-spec, ui-mockup, audit, tech-spec, use-case, test-case, test-report (no deploy); the ten front-matter types seed statuses [active, archived], ui-mockup seeds none, no document entry seeds default_status. init persists the set through to_config_yaml (flow lists), the walkthrough offers exactly those 11 slugs and disabling only flips enabled, and init --force replaces an invalid configuration with the validated default set. Declared deploy still resolves to a generic descriptor. Evidence: cargo test 738 passed / 0 failed / 22 ignored (init::tests:: 23, --test cli init_ 7); cargo fmt --check and clippy -D warnings clean; grep shows deploy only in explicit user-declared tests; specs updated; no requirement or business-rule files changed.
