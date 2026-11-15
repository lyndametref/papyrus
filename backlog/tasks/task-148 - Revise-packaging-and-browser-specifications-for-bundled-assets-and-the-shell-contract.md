---
id: TASK-148
title: >-
  Revise packaging and browser specifications for bundled assets and the shell
  contract
status: To Do
assignee:
  - '@business-analyst'
created_date: '2026-11-15T16:14:00Z'
labels:
  - docs
  - specifications
  - blocked
milestone: m-21
dependencies:
  - TASK-142
  - TASK-144
  - TASK-145
  - TASK-146
references:
  - docs/requirements/business/REQ-06.016.md
  - docs/requirements/business/REQ-06.017.md
  - docs/requirements/business/REQ-06.015.md
  - docs/architecture/adr/ADR-0025-bundled-katex-client-side-math-rendering.md
  - >-
    docs/architecture/adr/ADR-0027-vendored-embedded-browser-assets-and-packaging-notices.md
  - >-
    docs/architecture/adr/ADR-0028-browser-shell-shared-list-layout-content-width-and-collapsible-navigation.md
  - docs/specifications
priority: medium
ordinal: 41600
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Gate.** ADR-0026 is **Proposed**. The asset, font, width, navigation, and list clauses of this task may be written from the accepted ADRs, but the diagram and `style-src 'unsafe-inline'` clauses are written only after formal acceptance; until then the specification records the accepted math state and names the Mermaid ADR as proposed.

**Why.** ADR-0025, ADR-0027, and ADR-0028 change contracts that existing specifications state differently: several packaging and verification pages assert that no served CSS contains `url(` and that no page references a web font, the browser pages describe per-family list markup, and the content-width and navigation rules have no specification page. ADR-0025 and ADR-0027 assign that wording revision to the business analyst and tech writer.

**What.** Publish the browser shell specification and correct the affected packaging and browser specifications to the landed contract.

**In scope.**
- New UI specification page for the browser shell: the single shared list layout contract (fixed column order, family slots, action placement, hierarchical rows), the server-side width classes (`Capped`, `Full`, `bleed`), collapsible navigation with `localStorage` persistence and default expanded, no-JS behaviour, accessibility, and supported-browser checks.
- Correct every specification that asserts "no web font", "no `url(`", or an asset-free package: bundled same-origin fonts, the static asset table, the digest manifest, the single copyright file with third-party sections, `no-store`, and the 12 MiB budget.
- Record the CSP per response class: `font-src 'self'` for math, and the diagram `style-src` clause only as accepted.
- Point the browser specifications that describe per-family list markup at the shared layout (REQ-06.016) and remove superseded wording rather than leaving two accounts.
- Revise the packaging verification specification to check asset digests, notices, and offline rendering instead of the removed "no web font" assertions.
- Regenerate the affected specification and ADR indexes.

**Out of scope.** Requirement and ADR text; implementation (TASK-141 to TASK-149); release notes; the man page; any new product decision.

**Assumptions.** Specifications follow the landed, accepted contract, as TASK-127 and TASK-136 did. Where code and requirement disagree, the requirement wins and the conflict is reported instead of being resolved in prose.

**UI gate.** Not applicable: this task edits documentation only.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A browser shell UI specification page states the shared list contract, the width classes, the navigation persistence, no-JS behaviour, accessibility, and browser checks.
- [ ] #2 Every specification that asserted no web font, no url(, or an asset-free package now describes bundled same-origin fonts, the asset table, the digest manifest, and the single copyright file with third-party sections.
- [ ] #3 The CSP is recorded per response class, with font-src self for math and the diagram style clause only as accepted.
- [ ] #4 Browser pages that describe per-family list markup point at the shared layout, and superseded wording is removed rather than duplicated.
- [ ] #5 Packaging verification wording checks digests, notices, and offline rendering instead of the removed no-web-font assertions.
- [ ] #6 Affected specification and ADR indexes are regenerated, and no requirement or ADR text is edited.
- [ ] #7 The diagram clauses are written only after ADR-0026 acceptance, and the pre-acceptance wording names the ADR as proposed.
<!-- AC:END -->
