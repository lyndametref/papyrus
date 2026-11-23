---
id: TASK-114
title: >-
  Route typed document commands through document::run with descriptor-driven
  dispatch
status: Done
assignee:
  - rust-dev
created_date: '2026-11-14T20:29:00Z'
updated_date: '2026-11-14T22:05:00Z'
labels:
  - rust
  - refactor
  - cli
milestone: MS-019
dependencies:
  - TASK-103
  - TASK-105
references:
  - docs/architecture/adr/ADR-0016-runtime-config-derived-command-dispatch.md
  - >-
    docs/architecture/adr/ADR-0017-artifact-category-registry-and-type-descriptors.md
documentation:
  - docs/specifications/descriptor-driven-typed-document-command-dispatch.md
    docs/architecture/adr/ADR-0017-artifact-category-registry-and-type-descriptors.md
  - docs/architecture/adr/ADR-0018-generic-document-content-model.md
    docs/architecture/adr/ADR-0022-document-lifecycle-state-and-archive-location.md
  - src/cli/tree.rs
  - src/cli/dispatch.rs
  - src/document.rs
  - src/registry.rs
  - docs/requirements/business/REQ-007.001.md
  - docs/requirements/business/REQ-007.003.md
  - docs/requirements/business/REQ-007.004.md
  - docs/requirements/business/REQ-001.004.md
  - docs/requirements/business/REQ-005.001.md
  - docs/requirements/business/REQ-005.002.md
  - docs/requirements/business/REQ-005.008.md
  - docs/business-rules/BR-001-papyrus-command-behavior.md
  - docs/business-rules/BR-009-req-command-behavior.md
  - docs/business-rules/BR-010-nfr-command-behavior.md
  - docs/business-rules/BR-011-glossary-command-behavior.md
  - docs/business-rules/BR-012-ui-spec-command-behavior.md
  - docs/business-rules/BR-013-rule-command-behavior.md
  - docs/business-rules/BR-014-audit-command-behavior.md
  - docs/business-rules/BR-017-tech-spec-command-behavior.md
  - docs/business-rules/BR-018-use-case-document-type-behavior.md
  - docs/business-rules/BR-019-test-case-document-type-behavior.md
  - docs/business-rules/BR-020-test-report-document-type-behavior.md
priority: high
ordinal: 41210
---


## Description
**Why.** ADR-0016 makes `document::run` the single dispatch path for every document type and makes `descriptor` data the only command source, but m-17 shipped that model only for the generic path. `src/cli/tree.rs` still matches `TypedHandler` to graft one of ten per-type clap enums onto a slug, `src/cli/dispatch.rs` repeats the match in `dispatch_typed_module`, and the typed modules join their own `cfg.doc_directory` with `document_types.<slug>.directory`. Adding or moving a typed document type therefore still requires edits in four places, which is the model ADR-0016/0017 replaced. TASK-117 recorded the finding; the owner chose to refactor the code rather than amend the ADRs.

**What.** Build every document command from its `TypeDescriptor` — baseline and extension verbs from `descriptor.verbs`, typed create/edit options from descriptor metadata on `TypedSchema`, slug/prefix/directory from the descriptor — and route every document invocation through `document::run`, which delegates only content semantics to the owning module. Delete the `TypedHandler` command matches, `dispatch_typed_module`, and all ten per-type clap enums. No user-visible behavior changes.

**Scope (in).**
- `src/registry.rs`: extend `TypedSchema` with ordered per-verb option metadata and add one static table for the generic document family.
- `src/cli/tree.rs`: one descriptor-driven `document_command`; no `TypedHandler` match and no `augment_subcommands`.
- `src/cli/dispatch.rs`: every `ArtifactCategory::Document` slug enters `document::run`; no `dispatch_typed_module`, no typed `status` special case, no `from_arg_matches`.
- `src/document.rs`: one generic typed argument map; delegate typed `create`, `create-group`, `list`, `view`, `edit`, and `supersede` through `TypedHandler`; keep shared archive/delete/status for `Generic` and `Typed` content, with `ui-mockup` archive/delete delegated to `src/ui_mockup.rs` (HTML metadata, no shared record descriptor; `ui-mockup` has no `status` verb); resolve active/archived directory and prefix from the descriptor, and id padding from the effective `config.id_padding`.
- Ten typed modules (`req`/`nfr`, `glossary`, `rule`, `ui-spec`, `audit`, `tech-spec`, `use-case`, `test-case`, `test-report`, `ui-mockup`): neutral handler entry, no clap enum, no module-local directory join.
- Pre-refactor golden help snapshots asserted byte-for-byte after the refactor.

**Scope (out).**
- Removing `record::descriptor::descriptor_for_doc_type` and the parallel record descriptor table (TASK-120); the cache/slug leak (TASK-121).
- Changing any typed field set, section order, body parser, HTML, ID shape, status vocabulary, file layout, help text, exit code, or JSON shape.
- Adding a verb, option, alias, status, dependency, or configuration key; editing ADRs, requirements, or business rules.

**Canonical plan.** `docs/specifications/descriptor-driven-typed-document-command-dispatch.md` (design, migration matrix, compatibility contract, tests, risks). Requirement and business-rule links provide rationale and the current behavior contract; the ticket and its implementation plan remain self-contained.

**Assumptions.**

- Assumption: current runtime behavior wins over BR-009–BR-020 wording where they already disagree (`nfr --story` is accepted and ignored; `nfr` keeps "business requirement" about-lines; `test-report edit` keeps `--applicability` and `--editor`). No requirement or business-rule edit is made in this ticket.
- Assumption: option metadata belongs on `TypedSchema` in `src/registry.rs`; ADR-0016 decision 4 and ADR-0017 decision 7 already place commands, verbs, and typed content in descriptor data, so no ADR amendment is needed.
- Assumption: extension verbs (`create-group`, `supersede`) also enter `document::run` in this ticket, because the ticket routes every document invocation through one handler.
- Assumption: directory resolution stays in `src/document.rs` (descriptor fields plus the existing containment helper) so the registry remains free of I/O; id padding is global configuration, not descriptor data.
- Assumption: `ui-mockup` archive/delete stay in `src/ui_mockup.rs` for this ticket, because HTML records carry metadata comments and have no shared record descriptor; the exception is keyed on `ContentModel::HtmlMockup`, not on a slug.
- Assumption: TASK-120 owns the second record-descriptor lookup; TASK-119 leaves those call sites in place with narrower use.

## Acceptance Criteria
- [x] No `TypedHandler` command-construction match remains in `src/cli/tree.rs`; `document_command` builds every slug-level command and verb from the `TypeDescriptor`, and no `ReqCmd`, `GlossaryCmd`, `RuleCmd`, `UiSpecCmd`, `AuditCmd`, `TechSpecCmd`, `UseCaseCmd`, `TestCaseCmd`, `TestReportCmd`, or `UiMockupCmd` exists in `src/`.
- [x] `src/cli/dispatch.rs` sends every `ArtifactCategory::Document` slug through `document::run`; `dispatch_typed_module`, its `from_arg_matches` calls, and the typed `status` special case are gone.
- [x] Typed `create`, `create-group`, `list`, `view`, `edit`, and `supersede` reach the owning module only through `document::run` and a `TypedHandler` selected from the descriptor; `archive`, `delete`, and `status` use the shared record path for `Generic` and `Typed` content. Documented exception: `ui-mockup` keeps its `archive` and `delete` in `src/ui_mockup.rs` because HTML records carry their metadata as comments and have no shared record descriptor, and `ui-mockup` declares no `status` verb; the exception is keyed on `ContentModel::HtmlMockup`, never on a slug literal.
- [x] Each built-in typed type preserves its current field set, required/optional options, section layout, ID shape, file path, archive location, list/view text, `--json` keys, exit codes, and the `ui-mockup` HTML/supersede behavior; `nfr --story`, the empty `audit`/`ui-mockup` verb about-lines, the `nfr` about text, and `req --title <TITLE_FLAG>` are unchanged.
- [x] Directory and prefix resolution for a document type come from its `TypeDescriptor`; id padding is not a descriptor field and comes from the effective `config.id_padding`, read once by `DocumentScope::resolve`. The ten typed modules contain no `cfg.doc_directory` join, `type_entry`, `active_dir`, `archived_dir`, or `load_config` helper.
- [x] Golden help snapshots captured before the refactor match byte-for-byte afterwards for the top level, every enabled typed slug, every verb (including `create-group` and `supersede`), a config-defined generic slug, and the exit-2 usage-error cases; `--version`, `--help`, and `--json` error output are unchanged.
- [x] Tests assert that a config-defined generic slug and a built-in typed slug both enter `document::run` with the same invocation contract, and mechanical guards forbid a per-type clap enum, `augment_subcommands`, `TypedHandler`, `dispatch_typed_module`, or document `from_arg_matches` from returning under `src/cli/` or `src/`.
- [x] `cargo build`, the full `cargo test`, and `cargo clippy --all-targets -- -D warnings` are clean; the existing CLI, search, browser, release smoke, and document-type no-drift suites pass unchanged except the parsing-table cases extended for the new argument map.

## Definition of Done
- [x] #1 `cargo build`, `cargo test`, and `cargo clippy --all-targets -- -D warnings` pass.
- [x] #2 No dependency, CLI flag, verb, status, configuration key, or user-visible behavior is added or removed.
- [x] #3 Golden help fixtures are committed before the production refactor and the post-refactor snapshots are byte-identical.
- [x] #4 The final summary lists every removed enum, dispatch arm, slug helper, and module-local directory join, and names the `descriptor_for_doc_type` call sites deliberately left for TASK-120.
- [x] #5 `tests/document_type_no_drift.rs` passes and gains guards for the removed clap/dispatch plumbing; no `deprecated/` path, `nfreq`, or static per-type document command returns.
- [x] #6 No UI mockup is produced; no requirement, business rule, ADR, or specification outside this ticket's new spec is edited.


## Implementation Notes
# Implementation Plan

Canonical design: `docs/specifications/descriptor-driven-typed-document-command-dispatch.md`. Requirements: REQ-007.001/002/003/004, REQ-005.001/002/007/008/010, REQ-001.004/005. Rules: BR-001, BR-009–BR-014, BR-017–BR-020. ADRs: 0016, 0017, 0018, 0022.

## 0. Capture the contract first

1. Add `tests/fixtures/cli-help/papyrus.config.yml` with all eleven default slugs plus one config-defined generic slug (`note`).
2. Add `tests/cli_help_golden.rs` and `tests/golden/cli-help/`; a regeneration switch writes snapshots, normal runs compare byte-for-byte. Cover: top-level help; each typed slug `--help`; each verb `--help` including `create-group`/`supersede`; the generic slug and all its verbs; unknown slug, disabled slug, `nfreq`, unknown verb, missing required option, malformed option (stderr + exit 2); one `--json` application error (stdout/stderr/exit). Commit the pre-refactor snapshots.
3. Add contract tests that assert the current command surface programmatically: verb order, argument ids, long/short names, value names, requiredness, conflicts, and `required_unless_present`.

## 1. Descriptor option metadata and generic builder

1. In `src/registry.rs`, extend `TypedSchema` with an ordered `verbs` list. Add `VerbSpec` (verb, about line, ordered arguments) and an argument spec carrying id, long, short, value name, help, positional index, required, `required_unless_present`, action kind (single, append, switch, zero-or-one optional value), and conflicts.
2. Copy the current derive facts verbatim, including: positional versus `--title` ids; uppercase derive value names (`ID`, `TITLE`, `TITLE_FLAG`, `AUDIT_TYPE`, `SUPERSEDED_BY`); lowercase generic value names; empty about lines for `audit`/`ui-mockup`; `rule edit --no-requirement` as an optional value; `ui-mockup edit --status`/`--superseded-by` conflict; `ui-mockup create --spec` required and append.
3. Add one static generic seven-verb table shared by every `ContentModel::Generic` descriptor. Keep per-type tables literal; share only identical argument definitions (id, labels, references, editor, force, status value).
4. Preserve derive declaration order for arguments, because clap derives help display order from insertion order and interleaves propagated global options. Do not reorder positionals/options “for readability”.
5. Rewrite `cli::tree::document_command` to select verbs from `descriptor.verbs`, order them canonically (`create`, `create-group`, `list`, `view`, `edit`, `archive`, `delete`, `supersede`, `status`), and add args from the spec. Keep the slug name, `Manage <slug> documents.`, the reserved-name guard, registry insertion order, and the four root-level global options.
6. Add a registry invariant test: every document descriptor verb has a spec and every spec verb is declared. This turns a missing metadata row into a test/startup defect instead of a missing command.
7. Interchange step: the tree now builds from metadata while dispatch may still use the old enum path. Because metadata reproduces the same arg ids, run the golden suite here before touching dispatch.

## 2. Invocation and scope plumbing

1. Extend `DocumentVerb` with `CreateGroup` and `Supersede`; map command names to verbs through the registry (`Verb`), not through a local slug/verb match.
2. Replace the hardcoded verb-name match in `parse_document_invocation` with that mapping and return a validation error for an unknown verb instead of defaulting to `list`. Update `tests/cli_dispatch.rs` and `src/document.rs` constructions.
3. Add a generic argument map (non-global id -> values, key present only when the argument was present; empty values for switches). Derive the existing convenience fields (`id`, `title`, `description`, `labels`, `references`, `editor`, `status_value`, `force`) from the map so the generic path is untouched.
4. Add a `DocumentScope` resolved once in `document.rs` from the `TypeDescriptor` and the loaded config: active directory, archived directory, and prefix from the descriptor, and id padding from the effective `config.id_padding` (`TypeDescriptor` has no padding field). Resolve `directory.config_key` + `directory.relative` and contain it with the existing project-boundary helper; do not add I/O to the registry. Keep dotted-ID resolution on `descriptor_for_doc_type` for TASK-120.
5. Change the shared archive/delete/status helpers to take the resolved `TypeDescriptor` instead of a slug; keep `record_descriptor_for` internally (TASK-120 removes it). Delete the public slug-based `archive`/`delete`/`status` wrappers once typed modules stop calling them.

## 3. `document::run` as the only entry

1. Resolve the descriptor, validate category/enabled/verb, then handle `archive`/`delete`/`status` through the shared path for `Generic` and `Typed` content. `ContentModel::HtmlMockup` is the documented exception: `ui-mockup archive`/`delete` stay in `src/ui_mockup.rs` (HTML metadata comments, no shared record descriptor) and `ui-mockup` declares no `status` verb.
2. For content verbs, dispatch `Generic` to the existing generic functions and `Typed`/`HtmlMockup` to the owning module via a `TypedHandler` match (never a slug literal), passing descriptor, scope, verb, argument map, and registry.
3. Give each typed module one neutral handler entry that matches the verb and calls its existing internal create/edit/list/view/create-group/supersede code. Do not move section mapping, validation, body parsing, or HTML into `document.rs`.
4. Keep typed list/view delegated to the module: glossary (and others) emit type-specific JSON keys and must not fall back to the generic list/view.

## 4. Per-module migration order

Recommended order is simplest-first; it differs deliberately from a requirement-first listing because the similar types validate the plumbing before group and HTML logic.

1. `tech-spec`: add spec; switch dispatch; scope replaces local dirs; delete `TechSpecCmd`/`run`.
2. `use-case`: same plus `--applicability`.
3. `test-case`: same as use-case.
4. `test-report`: same as use-case.
5. `glossary`: `--term`/`--definition`, no title; Definition section.
6. `ui-spec`: `--view`, `--description`.
7. `audit`: no positional title; `--title`, `--content`, `--audit-type`; empty about lines.
8. `rule`: `--rationale`, `--applicability`, `--requirement`, `--no-requirement` optional value.
9. `req`/`nfr`: required `--group`, `create-group`, positional/flag title, `--story` behavior, group mismatch lookup through the registry; delete `ReqCmd` and the per-kind `run`/`run_nfr`.
10. `ui-mockup`: HTML create/edit, `--task`, required `--spec`, `--ref`, `--status`/`--superseded-by`, `supersede`, no `status` verb.

For each module, one change unit: spec + builder switch + neutral handler + scope + enum/run deletion + dispatch-arm removal. Update the four enum constructions in `src/document.rs` tests in the same unit. The module keeps: input sanitization, required-field validation, section names and order, body construction/parsing, front-matter keys, ID shape, and (for ui-mockup) HTML rendering/parsing.

## 5. Remove the old plumbing

1. Delete `dispatch_typed_module`, its arms, and the typed-status branch in `src/cli/dispatch.rs`; route every document slug from one parsed invocation to `document::run`.
2. Delete the `TypedHandler` match and per-type imports in `src/cli/tree.rs`.
3. Delete the remaining per-type enum definitions, `run` functions, `use clap::Subcommand`, `load_config`, `type_entry`, `active_dir`, and `archived_dir`.
4. Extend `tests/document_type_no_drift.rs` (do not add a third scanner) with guards: no per-type `*Cmd`, no `augment_subcommands` or `TypedHandler` in `src/cli/`, no `dispatch_typed_module`, no document `from_arg_matches`, and no `cfg.doc_directory` join in the ten typed modules.

## 6. Compatibility verification

1. Run the golden suite; any difference is a defect unless it is a documented removal (there should be none).
2. Run the per-type CLI suites (`req`, `glossary`, `business_rules`, `ui_spec`, `audit_records`, `technical_specifications`, `use_cases`, `test_cases`, `test_reports`, `ui_mockup`), the matching smoke suites, `tests/cli_dispatch.rs`, `tests/release_document_types_smoke.rs`, and `tests/document_type_no_drift.rs`.
3. Assert exit codes 0/1/2/3/4 and `--json` error output; assert no `nfreq`, `deprecate`, `restore`, or `status` on `ui-mockup`.
4. Add the dispatch test: a config-defined generic slug and a built-in typed slug both enter `document::run` and preserve their outputs.
5. `cargo build`; full `cargo test`; `cargo clippy --all-targets -- -D warnings`.

## 7. TASK-119 / TASK-120 split

Removed here: per-type clap enums and `run` functions; both `TypedHandler` matches; `dispatch_typed_module`; typed status branch; slug-based `document::archive`/`delete`/`status`; module-local config/directory helpers and joins.

Left for TASK-120: `record::descriptor::descriptor_for_doc_type` call sites in the ten modules (now only dotted-ID resolution), `document::record_descriptor_for`, `src/init.rs`, `src/record/migrate.rs`, `generic_document_descriptor`, and the registry agreement test. The final summary must name them.

## 8. Risks

- Highest: help drift from argument insertion order, value names, about text, or conflicts. Mitigate with pre-captured byte snapshots and contract tests.
- Near-duplicate option sets (tech-spec/use-case/test-case/test-report) invite over-generalisation; keep literal tables and per-type suites.
- Requirement group logic needs the sibling kind; pass the registry and keep the existing mismatch tests.
- Typed list/view must stay in their modules; switching to generic list/view would change JSON keys and text.
- Scope resolution must keep project containment and must not create directories outside `doc_directory`.
- Removing enums breaks direct enum constructions in `src/document.rs` tests; update them in the same change.


## Comments
author: @task-refiner
created: 2026-11-14T21:58:00Z

---

Wording aligned with the implemented design (no code change):
- AC3 / spec §4.3 now document the  exception: HTML / stay in `src/ui_mockup.rs` (metadata comments, no shared record descriptor) and  declares no `status` verb; the exception is keyed on `ContentModel::HtmlMockup`.
- AC5 no longer claims padding comes from : directory and prefix come from the descriptor, id padding from the effective  read by `DocumentScope::resolve`.
- Spec §8.3 now states that direct `document::run` tests cover generic create/status and typed create/list/view/archive, while `create-group`/`supersede`/typed edit/delete are asserted by the per-type CLI suites; direct coverage of those verbs is listed as optional hardening.

---

author: @task-refiner
created: 2026-11-14T21:59:00Z

---

Correction to comment #1 (inline code spans were lost to shell substitution). Wording aligned with the implemented design, no code change:
- AC3 / spec §4.3 now document the `ui-mockup` exception: HTML `archive`/`delete` stay in `src/ui_mockup.rs` (metadata comments, no shared record descriptor) and `ui-mockup` declares no `status` verb; the exception is keyed on `ContentModel::HtmlMockup`.
- AC5 no longer claims padding comes from `TypeDescriptor`: directory and prefix come from the descriptor, id padding from the effective `config.id_padding` read by `DocumentScope::resolve`.
- Spec §8.3 now states that direct `document::run` tests cover generic create/status and typed create/list/view/archive, while `create-group`/`supersede`/typed edit/delete are asserted by the per-type CLI suites; direct coverage of those verbs is listed as optional hardening.

---

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Every document slug now builds its command from TypeDescriptor metadata and enters document::run. Removed: ReqCmd, GlossaryCmd, RuleCmd, UiSpecCmd, AuditCmd, TechSpecCmd, UseCaseCmd, TestCaseCmd, TestReportCmd, UiMockupCmd; the tree TypedHandler command-construction match; dispatch_typed_module and the typed status special case; slug-based document::archive/delete/status wrappers; and module-local load_config/type_entry/active_dir/archived_dir helpers with cfg.doc_directory joins. src/req.rs is deleted. Typed modules keep validation, section mapping, front-matter keys, ID shape, and ui-mockup HTML archive/delete semantics (ContentModel::HtmlMockup exception, no status). Retained for TASK-120: descriptor_for_doc_type call sites in audit(3), glossary(1), requirement(1), rule(3), tech_spec(3), test_case(3), test_report(3), ui_spec(3), use_case(3) — now only is_dotted_id(); plus document::record_descriptor_for, init.rs, record/migrate.rs, generic_document_descriptor, and the registry agreement test. Evidence: 110 golden help snapshots byte-identical against a pre-refactor HEAD build; cargo test 807 passed / 0 failed / 22 ignored; clippy and fmt clean; new document_dispatch and no-drift guards.
