---
id: TASK-143
title: Add the vendored browser asset table, digest manifest, and package notices
status: To Do
assignee:
  - '@rust-dev'
created_date: '2026-11-15T16:13:00Z'
labels:
  - browser
  - packaging
  - rust
milestone: m-21
dependencies: []
references:
  - >-
    docs/architecture/adr/ADR-0027-vendored-embedded-browser-assets-and-packaging-notices.md
  - docs/architecture/adr/ADR-0025-bundled-katex-client-side-math-rendering.md
  - src/browser/http.rs
  - src/browser/mod.rs
  - scripts/build-deb.sh
  - packaging/deb/usr/share/doc/papyrus/copyright
priority: high
ordinal: 41560
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why.** ADR-0025 and ADR-0026 need vendored browser assets inside the single binary, and ADR-0027 is Accepted. The current asset model cannot serve them: `mod.rs` embeds exactly `app.css` and `app.js` and matches two hard-coded paths, `Response::text` cannot carry binary bytes, and the package notice file carries no third-party sections. Several existing tests assert that no served CSS contains `url(` and that no page references a web font, which the KaTeX stylesheet must change.

**What.** The static asset table, binary-safe responses, the vendored asset layout, the digest manifest, and the package notices.

**In scope.**
- New `src/browser/assets.rs` with `Asset { path, mime, bytes }`, `ASSETS`, and `lookup`; exact-match routing, `404` on a miss for every method, `405` for a hit with a method other than `GET`/`HEAD`, `HEAD` sending `Content-Length` with no body. No prefix matching, directory expansion, percent-decoding, or path join.
- `Response::bytes(status, content_type, &'static [u8])` in `src/browser/http.rs`, with the MIME types the served assets need.
- `src/browser/vendor/<package>/<version>/` containing unmodified upstream files, `LICENSE`, and `UPSTREAM.md` (package, version, upstream URL, retrieval date). Add KaTeX 0.16.11 only: `katex.min.js`, `katex.min.css`, and the 20 woff2 font files. Mermaid is added by TASK-146.
- `src/browser/vendor/MANIFEST.md` with package, version, licence, upstream URL, byte size, and SHA-256 per asset, and a test that recomputes every embedded digest.
- Move `app.css` and `app.js` onto the table; keep the zero-dependency, `include_bytes!`/`include_str!` model and a checkout that builds offline.
- `packaging/deb/usr/share/doc/papyrus/copyright` gains one section per bundled asset with licence identifier and full text or reference; the docs set stays exactly one file and `scripts/build-deb.sh` keeps its shape.
- Retarget the offline-reference tests: every `url(` in every served stylesheet must resolve to an asset-table entry, no served CSS may contain `@import`, no page may reference an origin outside `/assets/`, and no `.map` file is embedded.
- `Cache-Control: no-store` is retained; no `ETag`, compression, range request, or long-lived cache is added.

**Out of scope.** Math and diagram pre-passes and CSP changes (TASK-145, TASK-146); the release binary-size assertion and packaged verification (TASK-149); any Rust crate, npm build, or asset fetched at build time.

**Assumptions.** ADR-0027 decisions 1 to 10 are the contract; KaTeX licence review (MIT code, SIL OFL-1.1 fonts) passes before merge, and MathJax with SVG output is the recorded fallback if it does not. The 12 MiB budget is asserted by TASK-149.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A static asset table with exact-match lookup serves every embedded asset, returning 404 on a miss, 405 for a non-GET or non-HEAD method, and a bodyless HEAD with Content-Length.
- [ ] #2 Binary responses are supported with the MIME types the assets need, and no request path is ever joined to a filesystem path.
- [ ] #3 KaTeX 0.16.11 code, CSS, and woff2 fonts are committed unmodified under a versioned vendor directory with LICENSE and UPSTREAM.md.
- [ ] #4 MANIFEST.md records package, version, licence, upstream URL, size, and SHA-256 per asset, and a test fails on any digest mismatch.
- [ ] #5 app.css and app.js are served from the table, and cargo build remains the whole build with no crate, npm step, or network access.
- [ ] #6 The package copyright file carries one section per bundled asset and the docs set remains a single file.
- [ ] #7 Retargeted offline tests assert that every served url( resolves to a table entry, no served CSS contains @import, no page references a remote origin, and no source map is embedded.
- [ ] #8 Cache-Control no-store is retained and no cache, ETag, compression, or range behaviour is added.
<!-- AC:END -->
