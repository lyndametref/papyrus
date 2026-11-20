---
id: TASK-220
title: "Write ADR-C: cacheable immutable browser assets with validators"
created_date: '2026-11-20T17:36:52Z'
updated_date: '2026-11-20T17:36:52Z'
labels: []
references:
  - ADR-0005
  - ADR-0026
  - ADR-0027
  - NFR-07.003
  - TECH-072
status: ToDo
assignee:
  - architect
milestone: MS-031
dependencies:
  - TASK-218
ordinal: 40
documentation:
  - decisions/ADR-038-cacheable-immutable-browser-assets.md
type: architecture
---

## Description
Context: every response currently carries no-store, including 3.57 MB of Mermaid and 275 KB of KaTeX. The bootstrap cost dominates first paint in the browser and is unrelated to record reads.

Scope: author ADR-038, "Cacheable immutable browser assets with validators".
- Supersede ADR-0027 decision 8 and the no-store-on-every-response clause of ADR-0005.
- Amend ADR-0026 decision 4 for Mermaid loading.
- Define versioned vendor URLs served with immutable caching derived from the packaged asset version.
- Define app.css and app.js as no-cache with an ETag and a 304 path.
- Limit cacheability to assets: HTML and JSON responses stay no-store.
- State that no compression is introduced in this change.
- Define the exact test retargeting: existing header assertions move from every response to HTML/JSON responses, and new assertions cover vendor immutable caching and app asset 304 revalidation.
- Record the risk that a version bump is required to invalidate a vendor asset, and how packaging computes the version.
