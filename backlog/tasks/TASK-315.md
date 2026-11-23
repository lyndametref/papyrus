---
id: TASK-315
title: "Correct AUD-151 release audit findings"
created_date: '2026-11-23T07:52:49Z'
updated_date: '2026-11-23T08:42:02Z'
labels: []
references:
  - AUD-151
  - MS-037
  - REL-001
  - TASK-286
status: Done
assignee:
  - rust-dev
milestone: MS-037
dependencies:
  - TASK-284
ordinal: 65
documentation:
  - src/record/store.rs
  - src/record/relocate.rs
  - src/record/identity.rs
  - src/config.rs
  - src/document.rs
  - src/registry.rs
  - src/ui_mockup.rs
  - src/browser/records.rs
  - tests/document_type_no_drift.rs
  - docs/release-1.0-upgrade-notes.md
  - tests/build_package_guard.rs
  - tests/config_relocation_cli.rs
  - tests/milestone_archive_cli.rs
type: implementation
---

## Description
## Context
AUD-151 (code audit, REL-001) reports corrections required for MS-037: 1 blocker, 4 high, 5 medium, 2 low. This single task carries every correction and must be complete before TASK-286 (security audit) runs. TASK-285 (code audit) remains In-Progress at ordinal 60 and is not reordered.

## Scope (AUD-151 order, each with its regression test)

### Blocker
- B1: archive_inner, relocate_if_archived, archive_content and the mockup writer overwrite an existing archived record of the same ID. Refuse when the destination exists, name the destination in the error, leave both files untouched, and add regression tests for all four paths (archive, automatic relocation, archive_content, mockup writer).

### High
- H1: the D-06 documentation guard counts the upgrade-notes migration table as a static command inventory and fails the suite. Narrow the guard to real command inventories or rewrite the table, then rerun the suite green.
- H2: a valid field key can collide with generated clap argument ids and panic (title_flag, and tags with no_tags). Namespace generated argument ids (field_, remove_) or validate every generated id and long name against the fixed engine arguments; test title_flag and no_<list-key>.
- H3: id reservation loops forever when a record-like name is occupied by a directory or symlink (GLO-001.md directory). Treat any occupied candidate as a conflict and advance, or fail with a bounded error; test the directory and mockup targets.
- H4: mockup refs are written raw into an HTML comment and accept angle brackets and a double dash. Encode refs with the shared comment encoder or reject those characters; add an injection round-trip test.

### Medium
- M1: CLI create and edit ignore the declared field value domain (for example audit-type bogus is accepted). Apply the shared domain validation to both surfaces with CLI tests.
- M2: mockup saves re-read the record with unwrap_or_default and can write content derived from an empty string. Use the stored source or fail closed with no write; add a regression test.
- M3: config set relocates folders before persisting the config and config writes are non-atomic. Write the config atomically, order or roll back the relocation, and propagate the old-config load error.
- M4: the editor test depends on the ambient EDITOR. Clear EDITOR and VISUAL in the child environment.
- M5: the packaged-script guard test is flaky with ExecutableFileBusy (14 passes in 15 runs). Execute through bash or ensure the handle is closed so the test is non-flaky.

### Low
- L1: production relocation honours test-injection environment variables. Gate them behind cfg(test).
- L2: dead wrappers remain and the registry module documentation is stale. Remove the dead wrappers or document them, and fix the stale registry module documentation.

## Constraints
- No behaviour change beyond these corrections; no new crate, asset, template, route, or client script.
- Do not edit milestones, requirements, NFRs, or audit records from this task; record evidence as task comments.
- Keep the AUD-151 ordering when recording results.

## Definition of done
Every finding has a recorded result and test, or a recorded narrowing with a test; the suite, clippy, and fmt are green; no behaviour change beyond the corrections; AUD-151 is addressed point by point; evidence is recorded on this task.
## Acceptance Criteria
- [ ] B1: archive, automatic relocation, archive_content, and the mockup writer refuse to overwrite an existing archived destination, name it, and leave both files untouched; each path has a regression test.
- [ ] H1: the D-06 documentation guard is scoped to real command inventories (or the migration table is rewritten) so the suite is green.
- [ ] H2: generated clap argument ids are namespaced or validated against the fixed engine arguments; tests cover title_flag and no_<list-key> without a panic.
- [ ] H3: any occupied id candidate (directory, symlink) is treated as a conflict and advances or fails with a bounded error; tests cover a directory and a mockup target.
- [ ] H4: mockup refs are encoded with the shared comment encoder or rejected, and an injection round-trip test passes.
- [ ] M1: CLI create and edit apply the declared field value domain, with CLI tests on both surfaces.
- [ ] M2: a mockup save never derives content from an empty re-read; it uses the stored source or fails closed with no write, covered by a regression test.
- [ ] M3: config writes are atomic, relocation is ordered or rolled back, and an old-config load error is propagated, with tests.
- [ ] M4: the editor test clears EDITOR and VISUAL in the child environment.
- [ ] M5: the packaged-script guard test no longer fails with ExecutableFileBusy in repeated runs.
- [ ] L1: relocation test hooks are gated behind cfg(test).
- [ ] L2: dead wrappers are removed or documented and the stale registry module documentation is fixed.
- [ ] The full test suite, cargo clippy --all-targets -- -D warnings, and cargo fmt --check are green.
- [ ] No behaviour change beyond the AUD-151 corrections; AUD-151 is addressed point by point in the task evidence and is ready for re-audit.
## Comments
- **rust-dev**: -
- **orchestrator**: Acceptance met: B1 archive overwrite refused on all four paths, H1 guard scoped so the suite is green, H2 generated clap ids namespaced and validated, H3 bounded reservation against directories and symlinks, H4 mockup refs encoded, M1 field domains on CLI, M2 fail-closed mockup saves, M3 atomic config with rollback, M4 and M5 hermetic tests, L1 hooks gated, L2 dead code removed; 1427 tests pass.
- **orchestrator**: Reopened for close-out residuals: C-1 ambient editor in tests, C-2 non-atomic init config write, C-3 unused aliases, C-4 whole-file guard exemption, plus browser field-domain parity.
- **rust-dev**: Close-out residuals fixed. C-1: config_editor_json_null_when_unset clears EDITOR and VISUAL, suite also run green with EDITOR=nano. C-2: init writes the config through config::write_config_file; an injected write failure regression proves the previous configuration stays intact and no temporary file remains. C-3: removed the unused patch apply alias, the two relocation aliases, and their re-export after verifying no call sites. C-4: no_drift config.rs exemptions for fs::rename, write_atomic and .tmp are scoped to the write_config_file block; a planted record rename in the same module is reported. Parity: one shared FieldValues domain validator in document.rs is used by the CLI engine and by the browser form and JSON document save paths; a browser test saving audit_type bogus is rejected with 422 and writes nothing. Full suite green env -u EDITOR -u VISUAL and with EDITOR=nano; clippy all targets D warnings clean; fmt check clean.
- **orchestrator**: Close-out residuals closed: C-1 hermetic editor test (suite green with EDITOR=nano), C-2 atomic init config write with regression, C-3 unused aliases removed, C-4 guard exemption scoped, and field-domain validation unified across CLI and browser saves; 1430 tests pass.
