---
id: TASK-98
title: >-
  Verify the native layout, import, relocation, and single ID space end to end
  and package the release build
status: To Do
assignee: []
created_date: '2026-11-11T20:44:00Z'
updated_date: '2026-11-11T20:57:00Z'
labels:
  - verification
  - packaging
  - release-1.0
milestone: m-16
dependencies:
  - TASK-97
references:
  - >-
    docs/architecture/adr/ADR-0013-shared-record-envelope-without-a-storage-layer.md
  - docs/requirements/non-functional/NFR-04.004.md
  - docs/requirements/non-functional/NFR-03.001.md
priority: high
type: task
ordinal: 39150
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
ADR-0013 stage 8 is the gate on any release that claims NFR-04.004, and the release also needs a package that installs. The individual implementation tasks test their own behaviour, but nothing yet proves the combined properties that the renunciation depends on: no Backlog-shaped write anywhere, one identity on every surface, import fidelity against a real source store, and relocation that never loses a folder. This task provides that evidence and the installable artifact.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Adoption is idempotent and byte-identical for records that already carry front matter, and a no-op write changes nothing.
- [ ] #2 No code path writes a Backlog.md shape; an isolated sample Backlog.md project fixture is invisible to every surface before import and fully readable after it.
- [ ] #3 CLI text, --json, browser JSON, browser HTML, and browser URLs expose the Papyrus ID only.
- [ ] #4 Import fidelity, excluded scope, and the upgrade path are verified against an isolated sample Backlog.md project created for the test, not against this repository own store; relocation atomicity and conflict behaviour are verified.
- [ ] #5 Verification neither imports nor modifies this repository own backlog store, which stays in the Backlog.md layout after the release.
- [ ] #6 Search indexes whole-file text including front matter, and the documented search behaviour matches.
- [ ] #7 cargo build --release, the full test suite, cargo clippy --all-targets -- -D warnings, and cargo fmt --check all pass.
- [ ] #8 The Debian package builds, installs, and uninstalls on the supported target, and the installed binary runs the import and serves the browser.
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Scope note (planner, 2026-11-11): the user deferred the import of this repository own store out of Release 1.0. Verify the import against an isolated sample Backlog.md project fixture only; this repository store is not part of the verification evidence.
<!-- SECTION:NOTES:END -->
