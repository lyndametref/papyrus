# REQ-06.017 Render Math Formulas and Diagrams in the Browser

## User Story
As a project contributor, I want the browser to render math formulas and diagrams embedded in record content so that technical documentation is legible without leaving Papyrus.

## Group
REQ-06 Search and Local Web Interface

## Criticality
Should have

## Description
The local web interface renders math formulas (for example, TeX/LaTeX math) and diagrams (for example, PlantUML- or Mermaid-style diagrams) that are embedded in managed record content when it displays record bodies, so that authors can include mathematical notation and diagrams in Markdown content and readers see rendered output rather than raw source. Rendering happens in the browser only: CLI output and any other surface display the source text unchanged. Rendering works fully offline — all rendering assets are bundled with the application and no network request, external service, or server-side rendering process is required. The choice of rendering library or framework is an implementation decision; this requirement specifies capability and constraints (offline operation, single-binary distribution, and licensing) rather than a named library. Rendered output conforms to the browser compatibility targets (NFR-05.001) and to the content-width rules of REQ-06.015, under which diagram or visualization content may use the full available window width while surrounding text remains capped. Rendering failures degrade gracefully: unrenderable or malformed input is shown as its source text (or a clearly marked fallback) instead of breaking the page. Any rendering dependency must be open source and must permit linking in a GPLv3-or-later application (NFR-06.002), and rendered content must not result in arbitrary script execution (NFR-02.002).

## Acceptance Criteria
- Browser record views render embedded math formulas in record content.
- Browser record views render embedded diagrams in record content.
- CLI output and other surfaces show the source text unchanged; rendering is browser-only.
- Rendering requires no network access and works fully offline.
- All rendering assets are bundled with the application; no external service, CDN, or server-side rendering process is used.
- Unrenderable or malformed input degrades gracefully, showing the source text or a clearly marked fallback without breaking the page.
- Rendered diagram or visualization content may use the full available window width while adjacent text content stays capped at 1040px (REQ-06.015).
- Rendering works in the supported browsers (NFR-05.001).
- Any rendering dependency is open source and permits GPLv3-or-later linking (NFR-06.002).
- Rendered content does not result in arbitrary script execution (NFR-02.002).

## Traceability
- Source: customer-feedback handoff — Key Account Manager, Item 11, raw quote:
  - "support math formula display with mathTeX or similar (standard choice is part of the task) and diagram framework like plant uml and mermaid"
- Product decisions 2026-11-15 (in-session): rendering scope is the browser only; rendering must work fully offline; recorded as a REQ-06 capability requirement only, with constraints already covered by existing NFRs.
- REQ-06.002 — Serve the Local Web Interface
- REQ-06.015 — Control Browser Content Width and Navigation Visibility (diagram/visualization content may use full window width)
- NFR-01.002 — Display, View, and Edit All Papyrus Content in a Local Browser
- NFR-01.003 — Use the Browser Command and Interface Without Internet
- NFR-02.002 — Sanitize Input
- NFR-03.001 — Implement Papyrus in Rust (single-binary distribution)
- NFR-05.001 — Display the Browser Interface in Named Browsers
- NFR-06.002 — Use Open-Source Dependencies That Permit GPLv3-or-Later Linking
