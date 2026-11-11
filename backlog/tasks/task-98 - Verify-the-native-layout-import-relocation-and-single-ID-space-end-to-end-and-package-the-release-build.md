---
id: TASK-98
title: >-
  Verify the native layout, import, relocation, and single ID space end to end
  and package the release build
status: To Do
assignee: []
created_date: '2026-11-11T20:44:00Z'
labels:
  - verification
  - packaging
  - release-1.0
milestone: m-14
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
- [ ] #2 No code path writes a Backlog.md shape; a fixture Backlog store is invisible to every surface before import and fully readable after it.
- [ ] #3 CLI text, --json, browser JSON, browser HTML, and browser URLs expose the Papyrus ID only.
- [ ] #4 Import fidelity and excluded scope are verified against a fixture corpus, and relocation atomicity and conflict behaviour are verified.
- [ ] #5 Search indexes whole-file text including front matter, and the documented search behaviour matches.
- [ ] #6 cargo build --release, the full test suite, cargo clippy --all-targets -- -D warnings, and cargo fmt --check all pass.
- [ ] #7 The Debian package builds, installs, and uninstalls on the supported target, and the installed binary runs the import and serves the browser.
<!-- AC:END -->
