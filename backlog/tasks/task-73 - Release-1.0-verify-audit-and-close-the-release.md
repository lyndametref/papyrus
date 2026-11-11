---
id: TASK-73
title: 'Release 1.0: verify, audit, and close the release'
status: To Do
assignee: []
created_date: '2026-11-08T15:06:00Z'
updated_date: '2026-11-11T08:57:00Z'
labels:
  - release
milestone: m-14
dependencies:
  - TASK-20
  - TASK-82
references:
  - docs/requirements/business/REQ-05.010.md
  - docs/requirements/business/REQ-05.011.md
  - docs/requirements/business/REQ-06.003.md
  - docs/requirements/business/REQ-06.004.md
  - docs/requirements/non-functional/NFR-04.003.md
  - >-
    docs/architecture/adr/ADR-0007-yaml-front-matter-as-single-record-metadata-source.md
  - >-
    docs/architecture/adr/ADR-0008-adopt-backlog-md-on-disk-layout-for-tasks-and-milestones.md
  - >-
    docs/architecture/adr/ADR-0009-shared-record-envelope-and-unified-write-path.md
  - docs/specifications/release-lifecycle.md
  - docs/requirements/business/REQ-01.006.md
  - docs/requirements/business/REQ-01.007.md
  - docs/requirements/business/REQ-06.005.md
priority: high
type: task
ordinal: 40000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Close the first complete Papyrus release after the final cross-type integration milestone is Done. Run release-level build and test checks, delegate the mandatory end-of-release audit to security-officer, compile the release report, and record the audit summary and explicit user approval here.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Every m-12 task, including TASK-20, is Done.
- [ ] #2 The Debian package builds and installs on the supported targets and the full test suite passes.
- [ ] #3 The security-officer release audit is complete; no Critical or High finding is unresolved and remaining findings have documented owners.
- [ ] #4 This ticket records the audit summary, the release version, and explicit user approval.
- [ ] #5 The release report is compiled.
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Security audit (security-officer, 2026-11-11): Pass with findings. No Critical/High. Medium (1): symlink traversal in task/milestone resolution (owner rust-dev, NFR-04.002). Low (3): unauthenticated non-loopback browser bind (rust-dev); path disclosure in diagnostics (rust-dev); trimming normalization vs rejection (rust-dev + requirements-engineer). Info (3): CLI non-atomic writes (accepted risk); plaintext HTTP (loopback default); cargo audit unavailable (CI follow-up). Secrets: none. Dependencies: 4 direct (clap, serde, serde_json, yaml_serde), crates.io, checksums verified, GPL-compatible. NFR coverage: NFR-02.001/002, NFR-04.001/002/003, NFR-01.003, NFR-06.002 addressed. Release version: 0.1.0 (Cargo.toml). Package: papyrus_0.1.0_amd64.deb (1136804 bytes). Tests: 565 passed. Clippy: 0 warnings. Build: release ok.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Release 1.0 (v0.1.0) — Release report. Date: 2026-11-11 09:57:01 CET. Milestone: m-14 (Release 1.0). Included tasks (20 Done): TASK-18 (unified search), TASK-19 (browser access all content), TASK-20 (content-management packaging), TASK-60 (final cross-type browser UI mockup), TASK-66 (front-matter metadata & lifecycle browser UI), TASK-67 (one metadata projection), TASK-68 (lazy front-matter migration), TASK-69 (Backlog.md layout migration), TASK-70 (remove legacy serializers, no-drift), TASK-71 (Backlog.md round-trip), TASK-72 (managed artifacts & lifecycles page), TASK-74 (id_padding 1-6), TASK-75 (interactive init walkthrough), TASK-76 (browser skills page design review), TASK-77 (browser skills page), TASK-78 (use cases/test cases/test reports CLI), TASK-79 (browser UI mockup MOCK-0013), TASK-80 (search UC/TC/TR), TASK-81 (browser UC/TC/TR), TASK-82 (m-15 packaging). Requirement coverage: REQ-01, REQ-05 (007,010-014), REQ-06 (001-005); NFR-01, NFR-02, NFR-03, NFR-04, NFR-05, NFR-06. Build/test: cargo build --release OK; cargo test 565 passed, 0 failed; cargo clippy --all-targets -- -D warnings OK (0 warnings); cargo fmt --check OK. Package: papyrus_0.1.0_amd64.deb (1136804 bytes), papyrus 0.1.0, amd64, Depends libc6 (>=2.34), GPL-3.0-or-later. Isolated install/uninstall verified on Debian Forky. Security audit: Pass with findings — no Critical/High. Medium (1): symlink traversal in task/milestone resolution (owner rust-dev). Low (3): non-loopback browser auth, path disclosure, trimming normalization. Info (3): CLI non-atomic writes, plaintext HTTP, cargo audit unavailable. Accepted risks recorded. Known limitations: man pages (NFR-03.003) not implemented; real-GUI browser not driven (markup-level only); non-Forky releases not verified. Deferred work: follow-up tickets for security findings (optional). Recommended git tag: v0.1.0 (not executed). Explicit user approval: pending.
<!-- SECTION:FINAL_SUMMARY:END -->
