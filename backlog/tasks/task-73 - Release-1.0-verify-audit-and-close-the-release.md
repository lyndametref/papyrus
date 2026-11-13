---
id: TASK-73
title: 'Release 1.0: verify, audit, and close the release'
status: To Do
assignee: []
created_date: '2026-11-08T15:06:00Z'
updated_date: '2026-11-13T11:51:00Z'
labels:
  - release
milestone: m-14
dependencies:
  - TASK-20
  - TASK-82
  - TASK-91
  - TASK-98
  - TASK-99
references:
  - docs/requirements/business/REQ-06.013.md
priority: low
type: task
ordinal: 40000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Close Release 1.0 after the release scope was extended on 2026-11-11 with the Papyrus-native record layout, the single ID space, and the one-way Backlog.md import (ADR-0010 to ADR-0013), the configuration and browser requirements adopted the same day, and the migration of this repository own store. The release gate verifies the extended scope, re-runs the mandatory security audit over it, confirms that the upgrade notes and the one-time import are in place, compiles the release report, and records explicit user approval.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Every task in milestone m-14 is Done, and the feature milestone m-16 (Release 1.0 - Native Layout, Import, and Browser Refinements) is fully Done, including TASK-20 (m-12) and TASK-82 (m-15).
- [ ] #2 The Debian package builds and installs on the supported targets, and the full test suite, clippy, and formatting checks pass.
- [ ] #3 The security-officer release audit is re-run over the extended scope (native layout, import, relocation, browser configuration view); no Critical or High finding is unresolved and remaining findings have documented owners.
- [ ] #4 The upgrade notes document the one-time import for users, naming the affected directories, the command, and the renumbering, and the import is verified against an isolated sample Backlog.md project; this repository own store is explicitly out of release scope and was not imported.
- [ ] #5 This ticket records the audit summary, the release version, and explicit user approval, and the release report is compiled.
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Security audit (security-officer, 2026-11-11): Pass with findings. No Critical/High. Medium (1): symlink traversal in task/milestone resolution (owner rust-dev, NFR-04.002). Low (3): unauthenticated non-loopback browser bind (rust-dev); path disclosure in diagnostics (rust-dev); trimming normalization vs rejection (rust-dev + requirements-engineer). Info (3): CLI non-atomic writes (accepted risk); plaintext HTTP (loopback default); cargo audit unavailable (CI follow-up). Secrets: none. Dependencies: 4 direct (clap, serde, serde_json, yaml_serde), crates.io, checksums verified, GPL-compatible. NFR coverage: NFR-02.001/002, NFR-04.001/002/003, NFR-01.003, NFR-06.002 addressed. Release version: 0.1.0 (Cargo.toml). Package: papyrus_0.1.0_amd64.deb (1136804 bytes). Tests: 565 passed. Clippy: 0 warnings. Build: release ok.

Release scope extension (planner, 2026-11-11): milestone m-14 gained TASK-83 to TASK-100 for the accepted architecture decisions ADR-0010 to ADR-0013 (native record layout and single ID space, one-way Backlog.md import), the requirement changes of 2026-11-11, the configuration work, the browser work, and the import of this repository own store. TASK-73 now depends on every one of them; the dependency list also still carries TASK-20 (m-12) and TASK-82 (m-15). The security audit text and the release report recorded above were produced before the extension and are superseded: the audit must be re-run over the extended scope and the report recompiled. The release must ship the native layout and papyrus import backlog together and must publish the one-time import in its upgrade notes (ADR-0010 decision 11, ADR-0011 decision 9).

Revision (planner, 2026-11-11, user decision): the import of this repository own backlog store is deferred and out of Release 1.0 scope. TASK-100 left milestone m-14, was archived as deferred, and is no longer a dependency of this gate; the gate now depends on TASK-83 to TASK-99 plus TASK-20 and TASK-82. The release still satisfies ADR-0010 decision 11 and ADR-0011 decision 9: the native layout and papyrus import backlog ship together (TASK-88, TASK-92), the import is verified against an isolated sample Backlog.md project (TASK-98), and the upgrade notes document the one-time import for users (TASK-99). This repository own store keeps its Backlog.md layout after the release.

Restructure (planner, 2026-11-11, user decision): feature and specification work moved out of the release milestone into the dedicated feature milestone m-16, so m-14 holds only this release gate. The gate now depends on TASK-91, TASK-98, and TASK-99 in m-16 (the last tasks of the feature milestone) plus TASK-20 and TASK-82. Rationale: more requirements may still arrive before 1.0, and the release milestone stays open and isolated so the feature scope can grow without redefining the gate. TASK-91 is listed explicitly because nothing else in m-16 depends on it.

Revision (planner, 2026-11-11, user decision): TASK-99 (upgrade notes) moved into the release milestone m-14 at ordinal 39160, between the feature milestone and this gate. m-14 now holds TASK-99 and this ticket only, and both are Low priority per the user; m-16 keeps its 16 tasks at High. Dependencies are unchanged in substance: this gate still depends on TASK-99, TASK-98, TASK-91, TASK-20, and TASK-82, and TASK-99 still depends on TASK-83 and TASK-92 in m-16, so m-14 depends on m-16 and never the reverse.

Requirement change (planner, 2026-11-11): REQ-06.013 (per-milestone kanban board in the Backlog view) is planned into m-16 as TASK-101 (specification, 39035), TASK-102 (mockup revision and approval, 39045), and TASK-103 (implementation, 39145). No change to this gate's dependency list: TASK-103 depends on TASK-97 and TASK-98 now depends on TASK-103, so the gate's existing dependency on TASK-98 covers the new work transitively, and the gate's acceptance criteria (m-16 fully Done plus build, tests, re-audit, upgrade notes) already require it.
<!-- SECTION:NOTES:END -->

## Comments

<!-- COMMENTS:BEGIN -->
author: @task-refiner
created: 2026-11-13T11:51:00Z

---

Escalation from TASK-99 (2026-11-13): the exit-code wording in `docs/specifications/import-backlog-md-records.md` §1.6 item 6 and `docs/architecture/adr/ADR-0011-one-way-backlog-md-import.md` decision 6 is stale. Both say "`0` when every in-scope source record was imported (or planned, for a dry run); `1` runtime/validation failure". Shipped behaviour follows REQ-01.004 with four codes: `0` success/planned, `1` runtime failure (including refusal without `--allow-existing` and skipped records), `2` usage error, `4` validation/conflict (e.g. `--report` resolving outside the project directory) - confirmed by `src/main.rs`, `src/error.rs` (`Runtime => 1`, `Validation => 4`) and `src/import.rs`. TASK-99's upgrade notes publish the shipped mapping; the specification and ADR need a release-owner correction or an ADR amendment. Not edited by TASK-99, which is documentation-scoped and must not change ADR or requirement text.

---
<!-- COMMENTS:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Release 1.0 (v0.1.0) — Release report. Date: 2026-11-11 09:57:01 CET. Milestone: m-14 (Release 1.0). Included tasks (20 Done): TASK-18 (unified search), TASK-19 (browser access all content), TASK-20 (content-management packaging), TASK-60 (final cross-type browser UI mockup), TASK-66 (front-matter metadata & lifecycle browser UI), TASK-67 (one metadata projection), TASK-68 (lazy front-matter migration), TASK-69 (Backlog.md layout migration), TASK-70 (remove legacy serializers, no-drift), TASK-71 (Backlog.md round-trip), TASK-72 (managed artifacts & lifecycles page), TASK-74 (id_padding 1-6), TASK-75 (interactive init walkthrough), TASK-76 (browser skills page design review), TASK-77 (browser skills page), TASK-78 (use cases/test cases/test reports CLI), TASK-79 (browser UI mockup MOCK-0013), TASK-80 (search UC/TC/TR), TASK-81 (browser UC/TC/TR), TASK-82 (m-15 packaging). Requirement coverage: REQ-01, REQ-05 (007,010-014), REQ-06 (001-005); NFR-01, NFR-02, NFR-03, NFR-04, NFR-05, NFR-06. Build/test: cargo build --release OK; cargo test 565 passed, 0 failed; cargo clippy --all-targets -- -D warnings OK (0 warnings); cargo fmt --check OK. Package: papyrus_0.1.0_amd64.deb (1136804 bytes), papyrus 0.1.0, amd64, Depends libc6 (>=2.34), GPL-3.0-or-later. Isolated install/uninstall verified on Debian Forky. Security audit: Pass with findings — no Critical/High. Medium (1): symlink traversal in task/milestone resolution (owner rust-dev). Low (3): non-loopback browser auth, path disclosure, trimming normalization. Info (3): CLI non-atomic writes, plaintext HTTP, cargo audit unavailable. Accepted risks recorded. Known limitations: man pages (NFR-03.003) not implemented; real-GUI browser not driven (markup-level only); non-Forky releases not verified. Deferred work: follow-up tickets for security findings (optional). Recommended git tag: v0.1.0 (not executed). Explicit user approval: pending.
<!-- SECTION:FINAL_SUMMARY:END -->
