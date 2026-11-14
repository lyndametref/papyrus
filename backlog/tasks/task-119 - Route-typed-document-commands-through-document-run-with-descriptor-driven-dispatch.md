---
id: TASK-119
title: >-
  Route typed document commands through document::run with descriptor-driven
  dispatch
status: To Do
assignee:
  - rust-dev
created_date: '2026-11-14T20:29:00Z'
updated_date: '2026-11-14T20:29:00Z'
labels:
  - rust
  - refactor
  - cli
milestone: m-18
dependencies: []
references:
  - docs/architecture/adr/ADR-0016-runtime-config-derived-command-dispatch.md
  - >-
    docs/architecture/adr/ADR-0017-artifact-category-registry-and-type-descriptors.md
documentation:
  - docs/architecture/adr/ADR-0016-registry-derived-cli-dispatch.md
  - src/cli/tree.rs
  - src/cli/dispatch.rs
  - src/document.rs
priority: high
ordinal: 41210
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why.** ADR-0016 makes `document::run` the single dispatch path for every document type, but m-17 shipped only the generic path that way: `src/cli/tree.rs` still matches `TypedHandler` to graft a per-type clap enum onto the slug, and `src/cli/dispatch.rs` still calls `dispatch_typed_module` for typed content models, which repeats a second match and hands control to per-type `run` functions. Those modules also join their own directories from `cfg.doc_directory` plus `document_types.<slug>.directory`. The result is four places that must change to add or move a type, which is the model ADR-0016/0017 replaced. TASK-117 recorded the checkpoint finding; the owner chose to refactor the code rather than amend ADR-0016/0017.

**What.** Build every document command from its `TypeDescriptor` — baseline verbs from `descriptor.verbs`, typed create/edit options from `descriptor.content_model`, and slug/prefix/directory from the descriptor — and route every document invocation through `document::run`, which delegates to the type module only for content semantics. Delete the `TypedHandler` command matches in `src/cli/tree.rs` and `src/cli/dispatch.rs` and the per-type clap enums/dispatch arms they feed.

**In scope.**

- `src/cli/tree.rs`: one descriptor-driven `document_command` for every content model; no `TypedHandler` match.
- `src/cli/dispatch.rs`: document slugs reach `document::run` only; `dispatch_typed_module` removed.
- `src/document.rs` and the typed modules (`req`, `glossary`, `rule`, `ui_spec`, `audit`, `tech_spec`, `use_case`, `test_case`, `test_report`, `ui_mockup`): delegate create/edit/list/view/archive/delete/status content work to the module through a descriptor-selected handler; directory and prefix resolution moves to the descriptor.
- Behavior preservation: identical help text, flags, exit codes, field sets, sections, IDs, and file layout; `--json` unchanged.

**Out of scope.**

- Removing `record::descriptor::descriptor_for_doc_type` (TASK-120) and the descriptor cache or slug leak (TASK-121) — this task may leave both in place, with fewer consumers.
- Editing ADR-0016/0017, requirements, business rules, or specifications; no new document verb, status, or configuration key.
- Changing what a typed module writes: its field set, section order, content parser, and serialization stay as they are.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 No `TypedHandler` command-construction match remains in `src/cli/tree.rs` and no per-type clap enum (`ReqCmd`, `GlossaryCmd`, `RuleCmd`, `UiSpecCmd`, `AuditCmd`, `TechSpecCmd`, `UseCaseCmd`, `TestCaseCmd`, `TestReportCmd`, `UiMockupCmd`) is used to build a slug command
- [ ] #2 `src/cli/dispatch.rs` sends every `ArtifactCategory::Document` slug through `document::run`; `dispatch_typed_module` and any per-type status special case are gone
- [ ] #3 Typed create/edit/list/view/archive/delete/status still work through `document::run` delegation, preserving each type field set, section layout, ID shape, file path, exit codes, and `--json` shape
- [ ] #4 Directory and prefix resolution for a document type come from its `TypeDescriptor`/`ArtifactRegistry`, not from a module-local join of `cfg.doc_directory` with `document_types.<slug>.directory`
- [ ] #5 Existing help output for every default and config-defined slug is unchanged except for text that reflected the removed per-type plumbing; `--help` exit codes and ordering match the current snapshot tests
- [ ] #6 All existing CLI, dispatch, search, browser, and document-type no-drift tests pass unchanged; new or updated tests assert that a config-defined typed-like slug and a built-in typed slug both reach `document::run`
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 `cargo build`, `cargo test`, and `cargo clippy` are clean.
- [ ] #2 No new dependency, flag, verb, status, or configuration key is introduced.
- [ ] #3 The final summary names every removed enum/dispatch arm and states the remaining descriptor duplication deliberately left for TASK-120.
- [ ] #4 `tests/document_type_no_drift.rs` passes; no `deprecated/` path, `nfreq`, or static per-type command reappears.
<!-- DOD:END -->
