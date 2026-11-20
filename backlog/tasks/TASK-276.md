---
id: TASK-276
title: "Implement versioned immutable vendor assets and ETag revalidation for app assets"
created_date: '2026-11-20T17:43:20Z'
updated_date: '2026-11-20T17:43:20Z'
labels: []
references:
  - ADR-038
  - ADR-0027
  - ADR-0026
  - NFR-007.003
status: ToDo
assignee:
  - rust-dev
milestone: MS-036
dependencies:
  - TASK-272
ordinal: 60
documentation: []
type: implementation
---

## Description
Context: every response currently sends no-store, including 3.57 MB of Mermaid and 275 KB of KaTeX, so every page load re-downloads them.

Scope: serve vendor files from versioned URLs with Cache-Control immutable and a long max-age; serve app.css and app.js with no-cache plus a strong ETag and a 304 bodyless path; keep HTML and JSON no-store. Assets only: no record response becomes cacheable. No compression. Derive the asset version from the packaged build. Retarget the existing header assertions: no-store moves from every response to HTML and JSON, with new assertions for immutable vendor caching and app asset 304 revalidation.
