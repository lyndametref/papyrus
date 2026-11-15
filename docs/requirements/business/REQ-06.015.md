---
id: REQ-06.015
title: "Control Browser Content Width and Navigation Visibility"
created_date: '2026-11-15T15:46:30Z'
updated_date: '2026-11-15T15:57:53Z'
labels: []
references: []
---
# REQ-06.015 Control Browser Content Width and Navigation Visibility

## User Story
As a project contributor, I want browser pages to use the full window width where wide content needs it and to let me hide the left navigation bar, so that wide content is legible without changing the readable layout of ordinary pages.

## Group
REQ-06 Search and Local Web Interface

## Criticality
Should have

## Description
By default, browser pages keep the existing maximum width of 1040px, centered, with horizontal padding, so long-form text and forms stay readable. The cap is lifted only where wide content requires it:

- The backlog kanban board view (REQ-06.013) renders without the 1040px cap and uses the full available window width.
- UI mockup content uses a dedicated full-width layout class that lifts the 1040px cap; the mockup files themselves are unchanged, and no mockup preview route or endpoint is introduced by this requirement.
- On any page, diagram or visualization content may occupy the full available window width even when the surrounding text content remains capped at 1040px.

All other browser pages, including long prose, forms, and detail views, keep the 1040px cap.

The browser also provides a user control that collapses and expands the left navigation bar. Collapsing it removes the navigation bar from the layout and reclaims its width for page content. The control is available on every page, and the collapsed or expanded state persists across page navigation and page reloads within the local web interface, stored in the browser's `localStorage`. The default state is expanded, matching the current layout, and it applies whenever no stored value is present. Exact padding, breakpoints, animation, and the control's placement are design decisions.

## Acceptance Criteria
- Standard browser pages remain capped at 1040px, centered, with horizontal padding.
- The backlog kanban board view renders without the 1040px cap and uses the full available window width.
- A dedicated full-width layout class exists that lifts the 1040px cap for UI mockup content, and the mockup files themselves are unchanged.
- No mockup preview route or endpoint is introduced by this requirement.
- Diagram or visualization content on a page may use the full available window width while adjacent text content stays capped at 1040px.
- A control is available on every page to collapse and expand the left navigation bar.
- Collapsing the left navigation bar removes it from the layout and reclaims its width for page content.
- The collapsed or expanded state is stored in the browser's `localStorage` and persists across page navigation and page reloads within the local web interface.
- The default navigation state is expanded when no stored value is present.
- No other browser page loses the 1040px cap.

## Traceability
- Source: customer-feedback handoff — Key Account Manager, Item 5, raw quote:
  - "make the browser view use the full available width of the window"
- Product decisions 2026-11-15 (in-session): keep the 1040px cap except for the backlog kanban board, diagram/visualization content (diagram full width, text capped), and UI mockup views, which use a full-width class only; hiding the left navigation bar is a user toggle.
- Project-owner decision 2026-11-15: no mockup preview route is in scope — only the full-width layout class applies; the navigation collapse state is persisted in `localStorage` with the default expanded.
- REQ-06.002 — Serve the Local Web Interface
- REQ-06.009 — Set Browser Page Title Branding (shared header present on every page)
- REQ-06.013 — Present the Backlog as a Per-Milestone Kanban Board (uncapped board)
- REQ-06.017 — Render Math Formulas and Diagrams in the Browser (diagram/visualization content rendering)
- NFR-01.002 — Display, View, and Edit All Papyrus Content in a Local Browser
- NFR-05.001 — Display the Browser Interface in Named Browsers
