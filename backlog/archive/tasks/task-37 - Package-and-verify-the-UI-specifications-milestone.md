---
id: TASK-37
title: Package and verify the UI specifications milestone
status: Done
assignee: []
created_date: '2026-11-04T19:20:00Z'
updated_date: '2026-11-06T17:48:00Z'
labels:
  - packaging
milestone: m-6
dependencies:
  - TASK-34
  - TASK-35
  - TASK-24
priority: medium
type: chore
ordinal: 24400
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Package and verify UI-specification management, search, and browser workflows in an installable increment.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 amd64 Debian package installs and provides UI-specification workflows.
- [ ] #2 Smoke tests verify UI-specification management, search, and browser.
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
TASK-37 packaging & verification complete. Env: Debian Forky, glibc 2.43, rust 1.99.1, dpkg 1.23.11, fakeroot. Date 2026-11-06 UTC. Package papyrus_0.1.0_amd64.deb rebuilt. Gates: build --release pass, test 258 pass, clippy 0 warnings, fmt clean. Smoke: ui_specifications_smoke 2/2 pass. Isolated install/uninstall pass. Limitations: offline unshare not run; no real-GUI; config keys verified on built binary. No new deps.

Package size 869420 bytes. Isolated fakeroot dpkg -i PASS (host unchanged).
<!-- SECTION:NOTES:END -->
