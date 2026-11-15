---
id: TASK-149
title: Verify packaged assets, offline rendering, and the binary budget
status: To Do
assignee:
  - '@rust-dev'
created_date: '2026-11-15T16:14:00Z'
labels:
  - verification
  - packaging
  - browser
milestone: m-21
dependencies:
  - TASK-142
  - TASK-143
  - TASK-144
  - TASK-145
  - TASK-147
references:
  - >-
    docs/architecture/adr/ADR-0027-vendored-embedded-browser-assets-and-packaging-notices.md
  - docs/architecture/adr/ADR-0025-bundled-katex-client-side-math-rendering.md
  - >-
    docs/architecture/adr/ADR-0028-browser-shell-shared-list-layout-content-width-and-collapsible-navigation.md
  - scripts/build-deb.sh
  - tests/browser_cli.rs
priority: high
ordinal: 41610
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why.** ADR-0027 makes the package contents, the licence notices, the digests, and the binary-size budget part of the release contract, and ADR-0025 makes offline font delivery part of the browser contract. The current packaging verification still asserts a two-file asset model with no fonts and no `url(` in served CSS, so it cannot pass and would not notice a broken notice file or an asset that is not embedded.

**What.** Verify the packaged binary and the served assets against the accepted asset, width, and navigation contracts, short of the diagram work gated by ADR-0026.

**In scope.**
- Build the `.deb` and assert `dpkg-deb -c` lists the binary and exactly one copyright document, with the expected modes and no runtime asset directory, configuration file, or man page.
- Assert the copyright file carries the Papyrus GPL-3.0-or-later statement plus one section per bundled asset, and that the KaTeX licence identifiers and text are present.
- Assert every embedded asset digest matches `MANIFEST.md`, the playground of offline tests passes, and every served `url(` resolves to an asset-table entry with no `@import` and no remote origin.
- Assert the release binary stays within the 12 MiB budget and record the binary and `.deb` sizes in the task.
- Install the package in an isolated environment and verify that a math page renders with its bundled fonts, that a page without math requests no KaTeX asset, and that the page still shows source text when JavaScript is disabled.
- Verify the width contract and navigation from the installed binary: capped standard pages, the full-width board, the mockup full-width class present without a route, and the nav state surviving navigation and reload.
- Smoke `papyrus check` from the installed binary on a clean and a duplicate-ID fixture project.
- Confirm no `deprecated/`-folder access, no network access, and no new crate or build step in the release build.

**Out of scope.** Mermaid assets and the diagram CSP clause (gated by ADR-0026 and verified by TASK-150); specification prose (TASK-148); release notes; a release ticket.

**Assumptions.** The 12 MiB budget from ADR-0027 decision 6 is final. A size assertion that cannot run in CI is executed as a documented release check with the measured value recorded in the task.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 dpkg-deb -c lists the binary and exactly one copyright document with the expected modes and no runtime asset directory.
- [ ] #2 The copyright file carries the Papyrus licence plus one section per bundled asset, including the KaTeX licence identifiers and text.
- [ ] #3 Every embedded asset digest matches the manifest and the offline asset tests pass, with every url( resolving to a table entry.
- [ ] #4 The release binary is within the 12 MiB budget, and the binary and package sizes are recorded.
- [ ] #5 A math page renders with bundled fonts after installing the package, a page without math loads no math asset, and JavaScript-disabled pages show source text.
- [ ] #6 Standard pages stay capped and the board renders full width from the installed binary, and the nav state survives navigation and reload.
- [ ] #7 papyrus check runs from the installed binary on a clean and on a duplicate-ID fixture project.
- [ ] #8 No network access, deprecated-folder access, new crate, or build-toolchain step is required by the release build.
<!-- AC:END -->
