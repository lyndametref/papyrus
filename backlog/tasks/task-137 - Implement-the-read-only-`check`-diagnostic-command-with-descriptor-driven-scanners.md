---
id: TASK-137
title: >-
  Implement the read-only `check` diagnostic command with descriptor-driven
  scanners
status: To Do
assignee:
  - '@rust-dev'
created_date: '2026-11-15T16:11:00Z'
labels:
  - rust
  - cli
  - diagnostics
milestone: m-21
dependencies: []
references:
  - docs/requirements/business/REQ-01.008.md
  - >-
    docs/architecture/adr/ADR-0029-read-only-corpus-diagnostics-with-duplicate-id-repair.md
  - src/lib.rs
  - src/main.rs
  - src/commands.rs
  - src/registry.rs
priority: high
ordinal: 41500
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why.** REQ-01.008 adds `papyrus check` and ADR-0029 is Accepted. Three gaps block it today: `lib.rs::run` validates the configuration fail-closed before dispatch, so the command cannot diagnose a broken configuration; `CommandOutput` cannot carry a non-zero exit, so findings would have to be reported as an error; and no read-only corpus enumeration exists, because `record::resolve::list_in_dir` relocates records and therefore writes.

**What.** A `check` built-in command, a `src/check/` module with descriptor-driven read-only scanners, a tolerant self-load, a stable findings model, exit 4, and `--json`.

**In scope.**
- `BuiltinCmd::Check` wired into dispatch and the command tree. Flags are added by TASK-138; the reserved slug, help text, and `--help` golden are TASK-139.
- `lib.rs::run` gains an `is_check` branch beside `is_config`: the tree is built with no registry, so `check` works with an invalid configuration.
- Stage A: tolerant self-load through the same loader the `config get|set|unset|list` repair surfaces use; a load error becomes one `config-invalid` finding; only on success run `Config::validate(Some(project_dir))`, one finding per diagnostic line, and treat a registry build failure as a finding.
- Stage B: corpus scanners over `registry.descriptors()` for enabled **and** disabled types, read only through the existing parsers and projection.
- A new read-only primitive `record::scan::family_files(ctx, descriptor)` that lists active, archived, and read-only legacy directories without relocating, migrating, or adopting front matter.
- Finding kinds: `duplicate-id` (canonical ID per family, group and member spaces kept apart), `self-dependency`, `dependency-cycle` (iterative DFS, one finding per cycle, canonical member order), `dangling-milestone`, `dangling-release`, `frontmatter-conflict` (`id`/`title` against the `# <ID> <title>` heading, and the retired task sections `## Status`, `## Assignee`, `## Milestone`, `## Labels`, `## Dependencies` plus `## Release` on a milestone; an unreadable section is a conflict, agreement is not, decisions are excluded), `config-invalid`.
- `CommandOutput.exit_code` (0 by default); `main` prints the output and exits with it; `AppError` keeps its mapping (I/O 1, usage 2).
- Exit 4 whenever the run produced at least one finding, 0 otherwise; `--json` emits the findings object, never the error envelope.
- The report names the checks skipped when the configuration is invalid.
- `deprecated/` is never read or inventoried and produces no finding; a task dependency that resolves to no task is not a finding.
- Tests: one characterization fixture per finding kind written by the real writer; invalid-configuration run; deterministic ordering golden; text and `--json` goldens; exit-code tests; a read-only test that every scanned file keeps its bytes and mtime.

**Out of scope.** `--fix` and `--force` (TASK-138), reserved slug and help and README (TASK-139, TASK-140), dependency-reference findings, a browser surface, and any migration of legacy folders.

**Assumptions.** ADR-0029 decision 5 is the detection contract and decision 6 the finding shape and order. `config-invalid` is one finding per diagnostic line. A missing task dependency stays an open question recorded by ADR-0029, not a finding.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 `papyrus check` runs on a valid project and prints a plain findings report, exiting 0 when there are no findings.
- [ ] #2 Scanners read every registry descriptor, including disabled document types, through the existing parsers and projection, and never relocate, migrate, or adopt a record.
- [ ] #3 All seven finding kinds are detected: duplicate-id, self-dependency, dependency-cycle, dangling-milestone, dangling-release, frontmatter-conflict, and config-invalid.
- [ ] #4 Duplicate IDs are compared per family, keeping grouped-family group and member spaces apart, across decisions and every configured document type.
- [ ] #5 Front-matter conflicts cover the body heading and the retired task and milestone sections; agreement is not reported, an unreadable section is, and decision body sections are excluded.
- [ ] #6 A broken configuration does not abort the run: configuration findings are reported, the checks that need document_types are skipped and named as skipped, and the independent checks still run.
- [ ] #7 deprecated/ folders are neither read nor inventoried and produce no finding, and a task dependency that resolves to no task is not a finding.
- [ ] #8 CommandOutput.exit_code is honoured by main: a run with findings exits 4, a clean run exits 0, and --json emits the findings object rather than the error envelope.
- [ ] #9 Findings are deterministically ordered and both output forms match their golden fixtures.
- [ ] #10 A read-only test asserts that every scanned file keeps its bytes and modification time.
<!-- AC:END -->
