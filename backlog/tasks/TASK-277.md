---
id: TASK-277
title: "Load Mermaid lazily and stop sending it no-store"
created_date: '2026-11-20T17:43:20Z'
updated_date: '2026-11-22T21:35:25Z'
labels: []
references:
  - ADR-038
  - ADR-0026
  - NFR-007.003
  - ADR-0025
  - ADR-0027
  - TC-181
  - TC-189
status: Done
assignee:
  - frontend-dev
milestone: MS-036
dependencies:
  - TASK-276
ordinal: 70
documentation:
  - src/browser/math.rs
  - src/browser/render.rs
  - src/browser/app.js
  - src/browser/http.rs
  - src/browser/assets.rs
  - decisions/ADR-0026-mermaid-only-offline-diagram-rendering.md
  - decisions/ADR-038-cacheable-immutable-browser-assets.md
  - tests/browser_cli.rs
  - tests/browser_diagram_cli.rs
  - tests/browser_math_cli.rs
type: implementation
---

## Description
## Context
`src/browser/math.rs` holds `HEAD_ASSETS` for KaTeX and `MERMAID_ASSETS` for Mermaid, and `src/browser/render.rs::page_assets` already includes KaTeX only when a page has math and Mermaid only when a page has a renderable diagram. `src/browser/app.js` renders only `figure[data-diagram="mermaid"]` figures and keeps the server-rendered `<pre>` fallback when the bundle is absent or fails. `Response::html_diagram` selects the diagram CSP. The remaining gaps are the unversioned Mermaid and KaTeX URLs and the no-store asset response, both owned by TASK-276; this task completes the lazy, diagram-only loading contract and proves it.

## Scope
Load Mermaid only when a document contains a renderable `mermaid` figure, using the versioned immutable vendor URL from TASK-276. Keep the deferred classic-script load, the single vendored file, the 11.17.2 pin, the load order, the server-rendered fallback, and the existing render output unchanged. Do not introduce a dynamic import, module script, service worker, Cache API, prefetch, preload, or client-side asset cache. KaTeX remains available exactly where math exists and follows the same immutable vendor caching rules. A page without a renderable diagram must request no Mermaid bytes at all. The cache verdict is TASK-276; this task proves that Mermaid and KaTeX use the immutable class and that no diagram page sends them no-store.

## Assumptions
- "Lazy" means the existing deferred, diagram-only classic-script inclusion from ADR-0026 and ADR-038 decision 3; no new loading mechanism is introduced.
- The Mermaid and KaTeX bundles and their notices, manifest rows, and SHA-256 values are unchanged apart from the served path version segment.

## Definition of done
All acceptance criteria pass; tests prove zero Mermaid requests on non-diagram pages and exactly one on diagram pages; the rendered figure output and no-script fallback are unchanged; the full suite, `cargo clippy --all-targets -- -D warnings`, and `cargo fmt --check` are clean; evidence is recorded on this task.
## Acceptance Criteria
- [ ] A page with at least one renderable mermaid figure includes exactly one Mermaid script reference, uses the versioned immutable vendor URL from TASK-276, and marks the script defer; the page requests that script exactly once.
- [ ] A page without a renderable mermaid figure includes no Mermaid URL and makes no request to the Mermaid path, including no prefetch, preload, dynamic import, module import, or app script fetch.
- [ ] Mermaid stays the deferred classic script pinned at 11.17.2, one file, with an unchanged SHA-256, manifest row, licence, and notice; no dynamic import, module script, service worker, Cache API, or client-side asset cache is introduced.
- [ ] Render output is unchanged: the same figure markup, server-rendered pre fallback, render-error note, link neutralisation, theme and configuration, unique diagram identifiers, and diagram ordering; without JavaScript the source remains readable.
- [ ] KaTeX availability is unchanged: the KaTeX head fragment is included only when the page has math and references the versioned immutable KaTeX URLs; a math-only page requests no Mermaid, and a page without math requests no KaTeX.
- [ ] Mermaid and KaTeX responses use the immutable vendor class from TASK-276: public, max-age 31536000, immutable, no ETag, and never a 304; no diagram or math page sends a no-store header for these assets.
- [ ] The diagram page CSP is unchanged and no new CSP directive, inline script, or client script is introduced.
- [ ] Tests assert zero Mermaid requests on non-diagram pages and exactly one on diagram pages, and compare page and render output with the pre-change fixture apart from versioned vendor URLs; the browser, diagram, math, and asset suites pass.
- [ ] No behaviour change beyond the versioned URLs and cache headers: status codes, page HTML semantics, record content, offline behaviour, and the 404 and 405 matrix are unchanged; cargo clippy --all-targets -- -D warnings and cargo fmt --check are clean.
## Comments
- **rust-dev**: TASK-277 rust-dev: test-only proof added.
- **rust-dev**: No production-code delta needed: page_assets already gates MERMAID_ASSETS on has_diagram and TASK-276 supplies the versioned immutable URL.
- **rust-dev**: Files changed: browser diagram and math tests.
- **rust-dev**: Measured: diagram NOTE-012 first load 3572661 vendor B + 31512 first-party B, second load 0 asset B; math NOTE-017 first load 298749 KaTeX B + 31512 first-party B, second load 0 with zero Mermaid; plain NOTE-019 first load 0 vendor B + 31512 first-party B, second load 0.
- **rust-dev**: Cache classes: vendor public,max-age=31536000,immutable with no ETag and no 304; first-party no-cache with strong ETag and 304; pages no-store.
- **rust-dev**: Mermaid stays one deferred 11.17.2 classic script (3572661 B) with unchanged manifest digest, licence, and notice.
- **rust-dev**: Assertions added: exactly one deferred Mermaid reference and request on diagram pages; zero Mermaid refs and requests on 10 non-diagram pages with no preload, prefetch, module, dynamic import, or app fetch; KaTeX only on math pages with immutable no-ETag responses.
- **rust-dev**: Also: the page asset set normalised to the pre-version fixture differs only by vendor URL version segments; figure markup, pre fallback, CSP, link neutralisation, and diagram ordering are unchanged.
- **rust-dev**: Checks: cargo test EXIT 0 full suite with env -u EDITOR -u VISUAL (host EDITOR=nano breaks one unrelated pre-existing CLI test); cargo clippy --all-targets -- -D warnings clean; cargo fmt --check clean; characterization_replay 17 of 17 green with no corpus delta.
- **rust-dev**: Branch task TASK-277. No git commit, merge, checkout, or push was performed, per instruction. Task status left In-Progress.
- **orchestrator**: Acceptance met: diagram pages include exactly one deferred versioned Mermaid reference, non-diagram pages none; math pages keep KaTeX; vendor Mermaid and KaTeX are immutable with second-load transfer zero; production code already gated on diagram presence so the task adds proof and measurements; suite, clippy and fmt green.
