---
id: TASK-150
title: Verify the diagnostics, shell, and rendering release end to end
status: To Do
assignee:
  - '@rust-dev'
created_date: '2026-11-15T16:14:00Z'
labels:
  - verification
  - cli
  - browser
  - blocked
milestone: m-21
dependencies:
  - TASK-140
  - TASK-146
  - TASK-148
  - TASK-149
references:
  - docs/requirements/business/REQ-01.008.md
  - docs/requirements/business/REQ-06.015.md
  - docs/requirements/business/REQ-06.016.md
  - docs/requirements/business/REQ-06.017.md
  - docs/requirements/business/REQ-06.013.md
priority: high
ordinal: 41620
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Gate.** This task depends on TASK-146 and therefore inherits the ADR-0026 acceptance gate. It does not start while ADR-0026 is Proposed.

**Why.** The requirement set spans the CLI, the browser shell, the packaged assets, and two offline renderers. Each task has its own tests, but the acceptance criteria are cross-surface: the same corpus must be diagnosable, repairable, uniformly browsable, and readable with math and diagrams while staying offline and inside the package budget.

**What.** One end-to-end verification of the delivered release increment, from a fresh project to the installed package.

**In scope.**
- Diagnostics loop: a clean project exits 0; introduce a duplicate ID and a dangling milestone; `check` exits 4 and names both findings; `--fix` repairs the duplicate only, lists the affected references, and exits 4; the second run exits 0; a deliberately broken configuration is reported by `check` while every other command still fails closed.
- Browser: every family list uses the shared layout; the board renders full width; the nav collapses, persists across navigation and reload, and works without JavaScript; math and Mermaid render offline; a PlantUML fence shows source with a note; malformed math and diagram input fall back to source.
- Security: the exact CSP per response class with `script-src 'self'`, `font-src 'self'`, and the accepted `style-src`; hostile fixtures produce no script, handler, or remote reference.
- Package: install the `.deb` on the supported target, verify the notices and the docs set, the binary budget, the offline behaviour, and the absence of any runtime asset directory.
- Documentation: README and the business rules describe the shipped `check` surface, and the specifications match the landed shell, asset, and rendering contract.
- Record every measured value (binary size, `.deb` size, browser versions) and every deviation in the task.
- Confirm no release milestone or release ticket was created by this milestone, and report the release-vehicle decision to the owner through `orchestrator`.

**Out of scope.** New features or fixes beyond defects this verification uncovers, which are reported as follow-ups; release notes; a formal security audit; PlantUML support.

**Assumptions.** Verification runs on the ADR-0003 target release. A defect found here is reported with its owning task; it does not silently expand this task's scope.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The full diagnostics loop passes: clean 0, findings 4, duplicate-only repair, listed references, clean second run, and a broken configuration reported without blocking check.
- [ ] #2 Other commands still fail closed while the configuration is invalid, and init --force and config remain the repair paths.
- [ ] #3 Every family list uses the shared layout and the board renders full width from the installed package.
- [ ] #4 The navigation collapses, persists across navigation and reload, and works without JavaScript.
- [ ] #5 Math and Mermaid render fully offline, while PlantUML and malformed input show source text with a note.
- [ ] #6 The final CSP per response class matches the accepted ADRs, and hostile fixtures produce no script, handler, or remote reference.
- [ ] #7 Package contents, notices, binary budget, and offline behaviour are verified on the supported target release.
- [ ] #8 README and business rules describe the shipped check surface and the specifications match the landed contract.
- [ ] #9 Measured sizes and browser versions are recorded, and no release milestone or release ticket is created.
<!-- AC:END -->
