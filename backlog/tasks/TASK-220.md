---
id: TASK-220
title: "Write ADR-C: cacheable immutable browser assets with validators"
created_date: '2026-11-20T17:36:52Z'
updated_date: '2026-11-20T19:49:23Z'
labels: []
references:
  - ADR-0005
  - ADR-0026
  - ADR-0027
  - NFR-007.003
  - TECH-072
  - REQ-006.014
status: Done
assignee:
  - architect
milestone: MS-031
dependencies:
  - TASK-218
ordinal: 40
documentation:
  - decisions/ADR-038-cacheable-immutable-browser-assets.md
  - decisions/ADR-0027-vendored-embedded-browser-assets-and-packaging-notices.md
  - decisions/ADR-0005-embedded-dependency-free-http-server-and-server-rendered-ui.md
  - decisions/ADR-0026-mermaid-only-offline-diagram-rendering.md
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
## Refinement inputs (TASK-290)

### ADR inventory and amendment targets (`decisions/`)
Inputs: ADR-0027 d8 (`decisions/ADR-0027-vendored-embedded-browser-assets-and-packaging-notices.md`), ADR-0005 no-store-on-every-response clause (`decisions/ADR-0005-embedded-dependency-free-http-server-and-server-rendered-ui.md`), ADR-0026 d4 (`decisions/ADR-0026-mermaid-only-offline-diagram-rendering.md`).
Output path: `decisions/ADR-038-cacheable-immutable-browser-assets.md`. If the project declines asset caching, ADR-038 must record the explicit exclusion required by NFR-007.003.

### Refinement assumptions / definition of done
The source of truth is RE-010 D1-D14 (accepted, not reopenable) and MS-031 AC-2/AC-3. No decision is renumbered, softened, split, or reopened. Done means every acceptance criterion passes, the record is accepted and user-verified, and no source, test, or packaging file changes (MS-031 AC-10).
## Acceptance Criteria
- [ ] ADR-038 exists at `decisions/ADR-038-cacheable-immutable-browser-assets.md` with status accepted (MS-031 AC-2C).
- [ ] ADR-038 supersedes ADR-0027 decision 8 and the no-store-on-every-response clause of ADR-0005, and amends ADR-0026 decision 4 for Mermaid loading (MS-031 AC-2C).
- [ ] ADR-038 defines versioned immutable vendor URLs, app.css and app.js no-cache with ETag and a 304 path, and no-store for HTML and JSON, limited to assets with no compression (MS-031 AC-2C, RE-010 D12).
- [ ] ADR-038 states the exact test retargeting: existing header assertions move from every response to HTML and JSON responses, with new assertions for vendor immutable caching and app asset 304 revalidation (MS-031 AC-2C).
- [ ] ADR-038 records the risk that a vendor asset version bump is required to invalidate the cache and how packaging computes the version; no source, test, or packaging file changes (MS-031 AC-10).
## Comments
- **architect**: ADR-038 accepted at decisions ADR-038 file; supersedes ADR-0027 decision 8 and the ADR-0005 no-store clause; amends ADR-0026 decision 4; papyrus check reports no new findings.
- **orchestrator**: Acceptance met: ADR-038 accepted; supersedes ADR-0027 d8 and the ADR-0005 no-store clause only, amends ADR-0026 d4; defines immutable versioned vendor caching, app-asset no-cache with ETag and 304, HTML and JSON no-store, test retargeting and version-bump risk.
