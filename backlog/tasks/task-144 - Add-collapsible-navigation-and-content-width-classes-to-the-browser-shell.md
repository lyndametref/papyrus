---
id: TASK-144
title: Add collapsible navigation and content-width classes to the browser shell
status: To Do
assignee:
  - '@frontend-dev'
created_date: '2026-11-15T16:13:00Z'
labels:
  - browser
  - css
  - accessibility
milestone: m-21
dependencies:
  - TASK-141
references:
  - docs/requirements/business/REQ-06.015.md
  - >-
    docs/architecture/adr/ADR-0028-browser-shell-shared-list-layout-content-width-and-collapsible-navigation.md
  - src/browser/app.css
  - src/browser/app.js
  - src/browser/assets.rs
priority: high
ordinal: 41570
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why.** REQ-06.015 requires a control on every page that collapses the left navigation and reclaims its width, with the state persisted in `localStorage` and defaulting to expanded, and it requires the 1040 px cap to stay everywhere except the backlog board and mockup full-width regions. ADR-0028 fixes the mechanics; today `app.js` has a sidebar toggle that exists only below 480 px, is not persisted, and is hidden at wider widths.

**What.** The navigation toggle, its persistence and no-flash script, the width classes, and the accessible behaviour.

**In scope.**
- One `<button class="nav-toggle" type="button" aria-expanded="true" aria-controls="site-nav">` in the shell header on every page, keyboard operable with a visible focus ring, with `aria-expanded` and `aria-controls` kept in sync.
- Collapsed state sets `data-nav="collapsed"` on `<html>`; CSS sets the sidebar grid column to `0`, removes the main column border, and the nav is removed from the accessibility tree with `hidden`, so the reclaimed width is real layout width.
- New asset `/assets/nav-state.js` (about 15 lines) loaded synchronously in `<head>` to apply the stored value before first paint; `localStorage` key `papyrus.nav` with values `collapsed` and `expanded`; storage exceptions are caught and ignored.
- `app.js` stays deferred and handles only the toggle and the existing form guard; the automatic sub-480 px collapse is removed so it cannot compete with the persisted preference.
- Default state is expanded at every viewport width when no stored value exists.
- Width: `.app` keeps `max-width: 1040px; margin: 0 auto; padding: 0 24px 56px`; `.app--full` lifts the cap; a `.bleed` region inside a capped page uses full width with its own `overflow-x: auto` wrapper. The board route becomes `PageWidth::Full`; the mockup full-width class exists without a preview route.
- No page gains a horizontal scrollbar; wide regions scroll inside their own container.
- Tests: route-level width assertions (`Capped` for every list, detail, and edit route, `Full` for the board), a preset collapsed state that survives navigation and reload, a JavaScript-disabled run with the nav present and usable, no inline styles or inline scripts, and the CSP unchanged by this task.
- Verify at the 320, 480, and 720 px breakpoints and in Firefox 100+, Edge 130+, and Safari 18+ (NFR-05.001).

**Out of scope.** The shell and list refactor (TASK-141, TASK-142); the board action (TASK-147); diagram and math rendering (TASK-145, TASK-146); a mockup preview route (needs its own ADR); changes to `script-src`.

**Assumptions.** ADR-0028 decisions 3 to 8 are the contract. Toggle placement in the header is a design refinement and is approved through the UI gate below.

**UI gate.** This task changes page chrome on every page. A UI mockup for the header toggle and collapsed layout must be approved through `ui-designer` before merge, or the refiner records a no-mockup waiver.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A nav toggle with aria-expanded and aria-controls is present and keyboard operable on every page.
- [ ] #2 Collapsing removes the sidebar from the grid and the accessibility tree and reclaims its width for page content.
- [ ] #3 The collapsed or expanded state persists in localStorage under papyrus.nav across navigation and reload, with no flash of the wrong state and no server state or cookie.
- [ ] #4 The default state is expanded at every viewport width, and the automatic sub-480 px collapse is removed.
- [ ] #5 With JavaScript disabled the navigation is expanded and usable, and storage exceptions do not break the page.
- [ ] #6 Standard pages stay capped at 1040px and the board route renders full width; a mockup full-width class exists without a preview route.
- [ ] #7 A bleed region uses full width inside a capped page with its own overflow container, and no page gains a horizontal scrollbar.
- [ ] #8 Route tests assert Capped for every list, detail, and edit route and Full only for the board, and the CSP is unchanged.
- [ ] #9 An approved mockup for the header toggle and collapsed layout, or a recorded no-mockup waiver, exists before merge.
<!-- AC:END -->
