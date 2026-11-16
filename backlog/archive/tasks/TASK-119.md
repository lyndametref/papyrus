---
id: TASK-119
title: Guard init defaults and registry drift and finish archive terminology
status: Done
assignee:
  - rust-dev
created_date: '2026-11-14T20:31:00Z'
updated_date: '2026-11-15T11:00:00Z'
labels:
  - rust
  - tests
milestone: MS-019
dependencies:
  - TASK-115
references:
  - >-
    docs/architecture/adr/ADR-0022-document-lifecycle-state-and-archive-location.md
  - docs/specifications/interactive-init-walkthrough-for-artifact-activation.md
  - REQ-05.007
  - REQ-07.001
documentation:
  - src/init.rs
  - src/config.rs
priority: low
ordinal: 41260
---


## Description
**Why.** Two drift risks survived m-17. First, `init` writes the eleven document types from the literal `DOCUMENT_ARTIFACTS` array in `src/init.rs` while `Config::default()` seeds `document_types` independently in `src/config.rs`; nothing fails if one changes alone, so a new default type can be creatable through config inference yet missing from `init` (or the reverse). Second, seven typed modules still name a helper `active_or_deprecated`, `src/ui_mockup.rs` still names its archived-path bindings `deprecated`, and a requirement error still says `group is deprecated`, although the delivered lifecycle is `active` → `archived` with `deprecated/` folders inert (ADR-0022); the stale vocabulary invites a future reader to reintroduce deprecated behavior.

**What.** Add one equality test that ties init defaults to the default configuration through a single ordered seeding list in `src/config.rs`, and rename the stale identifiers and messages to archived terminology without changing behavior.

**In scope.**
- Test: one unit test in the `src/init.rs` test module (it needs the private `DOCUMENT_ARTIFACTS` const). Assertions: the eleven slugs equal the canonical ordered seeding list in `src/config.rs` position by position; the slug set equals `Config::default().document_types` keys; eleven unique slugs on the init side; eleven keys on the config side; no `deploy` slug on either side. It must fail if either side gains, loses, reorders, or renames an entry.
- `src/config.rs`: promote the inline `docs` seeding table used by `Config::default()` to one ordered module-level constant (crate-visible) and build `document_types` from it. Effective output must be unchanged: same keys, prefixes, directories, `enabled: true`, and `statuses` (`Some([active, archived])` for the ten front-matter types, `None` for `ui-mockup`).
- Renames only: `active_or_deprecated` → `active_or_archived`, plus the `deprecated` path parameter and local bindings → `archived`, in `src/audit.rs`, `src/ui_spec.rs`, `src/test_report.rs`, `src/rule.rs`, `src/test_case.rs`, `src/tech_spec.rs`, `src/use_case.rs`.
- `src/ui_mockup.rs`: `resolve_record`'s `deprecated` parameter and the `deprecated` / `move_to_deprecated` bindings → `archived` / `move_to_archived`.
- `src/requirement.rs`: `deprecated`, `group_deprecated_path`, `group_deprecated_safe`, `group_deprecated` → archived names; message `group is deprecated: <id>` → `group is archived: <id>` with the same `AppError::Validation` variant, so the exit code and behavior are unchanged.
- `src/browser/render.rs`: the lifecycle sentence "active → deprecated (restored via restore)" becomes archived wording with no `deprecated` and no `restore` claim; the "deprecated or unknown" comments in `src/browser/mod.rs` and `src/browser/render.rs` say archived.
- Manual source audit of every remaining `deprecated` occurrence in `src/`, classified in the final summary.

**Out of scope.**
- Lifecycle behavior: no status, folder, validation, or migration change; `deprecated/` folders stay inert; `ensure_dirs` calls stay where they are.
- The decision-record status `deprecated` (Nygard `proposed|accepted|rejected|deprecated|superseded`) and its messages in `src/decision.rs`, `src/registry.rs`, and the browser decision pages: legitimate, unchanged.
- Adding types, changing init output, a `DECISION_ARTIFACTS`/decision-type drift guard, or TASK-112 defaults.
- A new `tests/no_drift.rs` scanner check for the word `deprecated`; the audit is manual and recorded in the final summary.
- Reopening m-17 tasks.

**Assumptions.**
- Assumption: the config-side ordered seeding table is the canonical slug order, because `document_types` is a `BTreeMap` and cannot carry order; ordering equality is therefore asserted against that table, and set equality against the live default map.
- Assumption: no current test asserts `group is deprecated` or the browser lifecycle sentence, so no existing assertions need updating; new assertions appear only in the drift test.
- Assumption: archived requirement groups stay a validation error; only the message wording changes.

## Acceptance Criteria
- [x] A unit test in the src/init.rs test module asserts DOCUMENT_ARTIFACTS and the default document type set agree exactly: the eleven slugs match the canonical ordered seeding list in src/config.rs position by position, and the slug set matches Config::default().document_types keys; the test fails if either side gains, loses, reorders, or renames an entry
- [x] The same test asserts there is no deploy slug on either side, eleven unique slugs in DOCUMENT_ARTIFACTS, and eleven distinct Config::default().document_types keys, so no duplicate or extra slug can pass
- [x] src/config.rs exposes the eleven-type seeding table as one ordered crate-visible constant and Config::default() builds document_types from it, producing the same keys, prefixes, directories, enabled flags, and status lists as before, with no config key and no loader behavior change
- [x] No document-lifecycle identifier or user-visible message in src/ uses deprecated vocabulary: active_or_deprecated and its deprecated path bindings are renamed in the seven typed modules, ui_mockup.rs archived-path bindings are renamed, requirement.rs local bindings and the group is deprecated message use archived wording, and the browser lifecycle sentence states active to archived without claiming restore
- [x] Behavior is unchanged after the renames: no status, folder, validation, or migration outcome differs, error variants and exit codes are identical, generated papyrus.config.yml content is unchanged, and the full test suite passes
- [x] A source audit of grep -rn deprecated src/ shows remaining hits only in the three documented categories - legacy pre-front-matter migration, inert deprecated/ folder handling and its tests, and the five Nygard decision statuses kept by ADR-0022 - each listed with file and line in the final summary

## Definition of Done
- [x] #1 cargo build, cargo test, and cargo clippy are clean; the new drift test fails if a slug is added, removed, or reordered on either side (verified locally by a temporary edit, reverted before finishing).
- [x] #2 No new dependency, public type, or configuration key is introduced, and generated papyrus.config.yml content is unchanged.
- [x] #3 The final summary lists every renamed identifier and message with its file, the drift test name and location, and the classified inventory of remaining deprecated hits (legacy migration, inert folder and tests, Nygard decision status), each with file and line.
- [x] #4 No change to status values, folders, validation outcomes, migration behavior, or the decision-record deprecated status.


## Implementation Notes
1. In src/config.rs, promote the inline eleven-entry docs seeding table to one ordered crate-visible constant (slug, prefix, directory, has_status) and make Config::default() iterate it. Compare the emitted default map before and after to confirm identical keys, prefixes, directories, enabled flags, and status lists.
2. Add a unit test in the src/init.rs test module named document_artifacts_match_config_default_document_types:
   a. init side: DOCUMENT_ARTIFACTS (schema order) as a Vec<&str>.
   b. ordered reference: the config constant's slugs in declaration order; assert element-wise equality with (a), so an add, remove, reorder, or rename on either side fails.
   c. live default map: Config::default().document_types keys sorted; assert equality with (a) sorted, so a config-only extra or loss fails.
   d. size and uniqueness: DOCUMENT_ARTIFACTS has eleven entries and eleven distinct values; the default map has eleven keys.
   e. no deploy: neither DOCUMENT_ARTIFACTS nor the default map contains the slug deploy.
   f. status sanity: ui-mockup has no statuses and the ten front-matter types declare [active, archived], so the guard also covers the seeded shape the init output test asserts.
3. Rename the stale helper and bindings in the seven typed modules (audit, ui_spec, test_report, rule, test_case, tech_spec, use_case): active_or_deprecated to active_or_archived, and every deprecated path parameter and local to archived, including the scope.archived clone and the ensure_dirs argument.
4. Rename the same stale vocabulary in src/ui_mockup.rs: resolve_record's deprecated parameter, the three deprecated locals, and move_to_deprecated to move_to_archived.
5. Rename src/requirement.rs bindings (deprecated, group_deprecated_path, group_deprecated_safe, group_deprecated) and change the message to group is archived: <id>, keeping AppError::Validation so the exit code is unchanged.
6. Update the browser lifecycle sentence in src/browser/render.rs to state active then archived with no deprecated and no restore claim, and align the two comments in src/browser/mod.rs and src/browser/render.rs that describe a hidden record as deprecated.
7. Run cargo fmt, cargo clippy, cargo test. Then grep -rn deprecated src/ and classify every remaining hit into legacy migration, inert folder and tests, or Nygard decision status; if any hit falls outside those three, either rename it or record it as an open question instead of blocking silently.

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Init/registry drift guard and archive terminology complete.

Drift guard: init::tests::document_artifacts_match_config_default_document_types (src/init.rs) compares init::DOCUMENT_ARTIFACTS element-wise with the new ordered config::DEFAULT_DOCUMENT_TYPES const, set-equal with Config::default().document_types, asserts 11 entries, no deploy, no duplicates, ui-mockup without statuses and the ten front-matter types with [active, archived]. Proven to fail on reorder, rename, duplicate, removal, config-extra and deploy mutations; generated YAML is byte-identical to the previous output.

Renames: active_or_deprecated -> active_or_archived and deprecated params/locals -> archived in audit, ui_spec, test_report, rule, test_case, tech_spec, use_case; ui_mockup move_to_deprecated -> move_to_archived plus locals; requirement group_deprecated_* -> group_archived_* and message 'group is archived: {}' (same Validation/exit 4); browser lifecycle sentence active -> archived without restore, plus related comments and five test assertion strings.

Remaining deprecated inventory (12 hits, all legitimate): decision status — browser/routes.rs:167, browser/render.rs:4255, registry.rs:1975, decision.rs:319, decision.rs:329, decision.rs:768, decision.rs:769; inert deprecated/ folder handling/tests — record/migrate.rs:1681, 1682, 1685, 1697; legacy-path doc comment — record/descriptor.rs:189.

Evidence: cargo test 836 passed / 0 failed / 22 ignored; clippy and fmt clean; behavior unchanged (statuses, folders, validation, migration, decision status).
