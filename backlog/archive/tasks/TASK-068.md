---
id: TASK-068
title: 'Release 1.0: verify, audit, and close the release'
status: ToDo
assignee: []
created_date: '2026-11-08T15:06:00Z'
updated_date: '2026-11-13T15:35:00Z'
labels:
  - release
milestone: MS-015
dependencies:
  - TASK-020
  - TASK-077
  - TASK-086
  - TASK-093
  - TASK-094
references:
  - docs/requirements/business/REQ-006.013.md
priority: low
type: task
ordinal: 1000
---


## Description
Close Release 1.0 after the release scope was extended on 2026-11-11 with the Papyrus-native record layout, the single ID space, and the one-way Backlog.md import (ADR-0010 to ADR-0013), the configuration and browser requirements adopted the same day, and the migration of this repository own store. The release gate verifies the extended scope, re-runs the mandatory security audit over it, confirms that the upgrade notes and the one-time import are in place, compiles the release report, and records explicit user approval.

## Acceptance Criteria
- [x] Every task in milestone m-14 is Done, and the feature milestone m-16 (Release 1.0 - Native Layout, Import, and Browser Refinements) is fully Done, including TASK-20 (m-12) and TASK-82 (m-15).
- [x] The Debian package builds and installs on the supported targets, and the full test suite, clippy, and formatting checks pass.
- [x] The security-officer release audit is re-run over the extended scope (native layout, import, relocation, browser configuration view); no Critical or High finding is unresolved and remaining findings have documented owners.
- [x] The upgrade notes document the one-time import for users, naming the affected directories, the command, and the renumbering, and the import is verified against an isolated sample Backlog.md project; this repository own store is explicitly out of release scope and was not imported.
- [ ] This ticket records the audit summary, the release version, and explicit user approval, and the release report is compiled.


## Notes
Release 1.0 close-out (2026-11-13T13:30+01:00).

Security audit (security-officer, 2026-11-13T13:09+01:00, supersedes the 2026-11-11 audit): Pass with findings. 0 Critical, 0 High, 3 Medium, 7 Low, 7 Info.
Medium: M1 import source symlink escape (owner rust-dev, NFR-004.002); M2 init config symlink write (owner rust-dev, NFR-004.002); M3 unauthenticated local browser read/write (owner rust-dev, accepted single-user loopback risk).
Low: L1 relocation release-build test hooks (rust-dev); L2 dry-run creates dirs (rust-dev); L3 import field sanitization (rust-dev); L4 path disclosure (rust-dev); L5 editor env value in config view (frontend-dev); L6 trimming vs rejection (rust-dev + requirements-engineer); L7 mockup HTML escaping (rust-dev/frontend-dev).
Info: I1 config writes not atomic; I2 plaintext HTTP loopback; I3 GET adoption writes; I4 unsigned committed .deb; I5 cargo audit unavailable; I6 Done docs tasks with unchecked ACs (process exception); I7 --report can overwrite in-project file.
Secrets: none. Dependencies: 35 third-party crates, crates.io checksums, 0 OSV advisories, GPL-compatible. NFR: NFR-001.003, NFR-002.001, NFR-004.001/004, NFR-006.001/002 addressed; NFR-002.002 and NFR-004.002 gaps documented.

Architecture readiness (architect, 2026-11-13): ready. All ADRs Accepted/superseded correctly. Constraint check PASS for native layout, single ID space, one-way import, shared envelope, stage 8 gate. G1: ADR-0011 decision 2 and import spec §1.6 still say 1 runtime/validation; shipped 0/1/2/4 per REQ-001.004; deferred to follow-up ADR-0014 + spec correction (not release-blocking).

Release checks: cargo build --release OK; cargo test 633 passed / 0 failed / 0 ignored; cargo clippy --all-targets -- -D warnings 0 warnings; cargo fmt --check OK. Package papyrus_0.1.0_amd64.deb 1,197,168 bytes SHA-256 96acf96b672522bbd718dab94552e1e817b1743745a11ad42a5d75744a0985b2; isolated fakeroot dpkg install/remove/purge verified.


## Comments
author: @task-refiner
created: 2026-11-13T11:51:00Z

---

Escalation from TASK-99 (2026-11-13): the exit-code wording in `docs/specifications/import-backlog-md-records.md` §1.6 item 6 and `docs/architecture/adr/ADR-0011-one-way-backlog-md-import.md` decision 6 is stale. Both say "`0` when every in-scope source record was imported (or planned, for a dry run); `1` runtime/validation failure". Shipped behaviour follows REQ-001.004 with four codes: `0` success/planned, `1` runtime failure (including refusal without `--allow-existing` and skipped records), `2` usage error, `4` validation/conflict (e.g. `--report` resolving outside the project directory) - confirmed by `src/main.rs`, `src/error.rs` (`Runtime => 1`, `Validation => 4`) and `src/import.rs`. TASK-99's upgrade notes publish the shipped mapping; the specification and ADR need a release-owner correction or an ADR amendment. Not edited by TASK-99, which is documentation-scoped and must not change ADR or requirement text.

---

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Release 1.0 (v0.1.0) release report — READY FOR APPROVAL. Date: 2026-11-13T13:30+01:00. Milestone: m-14 (Release 1.0); feature milestone m-16 fully Done (19/19).

Included work: m-16 TASK-83..98, TASK-101, TASK-102, TASK-103, TASK-87 (native layout and single ID space, front-matter adoption, atomic folder relocation, init/startup config, papyrus import backlog, browser record view/NFR grouping/branding/links, browser config view, lifecycle per-artifact config, quiet console + --debug, no-drift refactor, kanban board); TASK-99 (Release 1.0 upgrade notes); TASK-20 (m-12) and TASK-82 (m-15) previously Done.

Requirements coverage: REQ-001.002/003/004/005/006/007, REQ-005.010/011, REQ-006.002-013 (incl. REQ-006.013 board), REQ-002.001/002, REQ-003.001; NFR-001.002/003, NFR-002.001/002, NFR-003.002, NFR-004.001/002/004, NFR-005.001, NFR-006.001/002.

Build/test: cargo build --release OK; cargo test 633 passed, 0 failed, 0 ignored; cargo clippy --all-targets -- -D warnings 0 warnings; cargo fmt --check OK. Package: papyrus_0.1.0_amd64.deb (1,197,168 bytes, SHA-256 96acf96b672522bbd718dab94552e1e817b1743745a11ad42a5d75744a0985b2), installed and purged in isolated fakeroot dpkg root on Debian Forky amd64.

Security audit (2026-11-13): Pass with findings. 0 Critical, 0 High, 3 Medium (M1 import source symlink escape; M2 init config symlink write; M3 unauthenticated local loopback browser), 7 Low, 7 Info. No unresolved Critical/High; every finding has a documented owner. Accepted risks: single-user local threat model; loopback plaintext HTTP; unsigned committed .deb; GET-time front-matter adoption.

Architecture readiness: all ADRs Accepted or correctly superseded; native layout, single ID space, one-way import, shared envelope and stage 8 verification constraints satisfied.

Known limitations and follow-ups: no real-GUI browser driving; non-Forky Debian targets not verified; several Done m-16 documentation tasks carry unchecked checkboxes (TASK-83/84/85/86/101) — content verified, bookkeeping exception; repository own backlog store explicitly out of scope and not imported. Follow-up owner architect, after release: ADR-0014 to correct ADR-0011 decision 2 / import spec §1.6 exit-code wording (shipped 0/1/2/4 per REQ-001.004). Confirmed: TASK-99 upgrade notes match TASK-98 upgrade-path evidence.

Approval status: awaiting explicit user approval of this release report; the no-approval-mode instruction covers milestone task gates, not release completion. Recommended Git tag: v0.1.0 (not created).
