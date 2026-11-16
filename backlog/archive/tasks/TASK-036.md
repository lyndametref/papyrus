---
id: TASK-036
title: Package and verify the UI specifications milestone
status: Done
assignee: []
created_date: '2026-11-04T19:20:00Z'
updated_date: '2026-11-06T17:48:00Z'
labels:
  - packaging
milestone: MS-007
dependencies:
  - TASK-033
  - TASK-034
  - TASK-024
priority: medium
type: chore
ordinal: 24400
---


## Description
Package and verify UI-specification management, search, and browser workflows in an installable increment.

## Acceptance Criteria
- [ ] amd64 Debian package installs and provides UI-specification workflows.
- [ ] Smoke tests verify UI-specification management, search, and browser.


## Notes
TASK-37 packaging & verification complete. Env: Debian Forky, glibc 2.43, rust 1.99.1, dpkg 1.23.11, fakeroot. Date 2026-11-06 UTC. Package papyrus_0.1.0_amd64.deb rebuilt. Gates: build --release pass, test 258 pass, clippy 0 warnings, fmt clean. Smoke: ui_specifications_smoke 2/2 pass. Isolated install/uninstall pass. Limitations: offline unshare not run; no real-GUI; config keys verified on built binary. No new deps.

Package size 869420 bytes. Isolated fakeroot dpkg -i PASS (host unchanged).
