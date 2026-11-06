# Feature Specification — UI Specifications Milestone Packaging and Verification

**Ticket:** TASK-37
**Milestone:** m-6 (UI Specifications)
**Status:** Refined (execution-ready)
**UI mockups:** none new; MOCK-0005 approved remains the UI specifications category reference.

## Purpose

Package and verify UI-specification management, search, and browser workflows in an installable increment. Mirrors the glossary milestone (TASK-33) pattern.

## Scope

In scope: cargo build/test/clippy/fmt, build-deb.sh, isolated install/uninstall, installed-binary smoke (lifecycle, search, browser, config keys), tests/ui_specifications_smoke.rs, docs/packaging.md TASK-37 section, ticket evidence. Out of scope: product behavior changes, new dependencies.

## Behavior

Same structure as docs/specifications/glossary-terms-milestone-packaging-and-verification.md sections 1-5, adapted: document_types.ui-spec default `specifications/ui/`, prefix UI, CLI `ui-spec`, search doc type, browser ?type=ui-spec, API /api/ui-specs.

## Tests

1. `milestone_ui_specifications_flow`: init, ui-spec create/list/view/edit/deprecate/restore, quoted-phrase search doc limit json, browser startup/list/view/edit/API/assets/320px/no-external-refs.
2. `installed_layout_has_no_runtime_asset_dependency`: binary with empty cwd, search+browser work.

## Documentation and evidence

docs/packaging.md TASK-37 section with environment, date/timezone UTC, package file/size, gate outcomes, smoke outcomes, limitations. Ticket notes same.
