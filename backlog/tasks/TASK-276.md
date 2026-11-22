---
id: TASK-276
title: "Implement versioned immutable vendor assets and ETag revalidation for app assets"
created_date: '2026-11-20T17:43:20Z'
updated_date: '2026-11-22T21:27:28Z'
labels: []
references:
  - ADR-038
  - ADR-0027
  - ADR-0026
  - NFR-007.003
  - ADR-0025
  - ADR-0005
  - TECH-007
  - TECH-008
  - TECH-013
  - TECH-024
  - TECH-027
  - TECH-069
  - TC-181
  - TC-187
  - TC-189
status: Done
assignee:
  - rust-dev
milestone: MS-036
dependencies:
  - TASK-272
ordinal: 60
documentation:
  - src/browser/assets.rs
  - src/browser/http.rs
  - src/browser/routes.rs
  - src/browser/render.rs
  - src/browser/math.rs
  - src/browser/vendor/MANIFEST.md
  - decisions/ADR-038-cacheable-immutable-browser-assets.md
  - tests/browser_cli.rs
  - tests/backlog_discovery_smoke.rs
  - tests/search_browser_smoke.rs
  - tests/release_document_types_smoke.rs
  - tests/technical_specifications_smoke.rs
  - tests/use_cases_test_cases_test_reports_smoke.rs
  - scripts/build-deb.sh
  - packaging/deb/usr/share/doc/papyrus/copyright
type: implementation
---

## Description
## Context
Every response is emitted by `src/browser/http.rs::write_response`, which hard-codes `Cache-Control: no-store`; the embedded table in `src/browser/assets.rs` serves 26 files (three first-party files, KaTeX 0.16.11 CSS, script and 20 fonts, and Mermaid 11.17.2), and all paths are unversioned. The vendor bytes already live under `src/browser/vendor/<package>/<version>/` and are recorded in `src/browser/vendor/MANIFEST.md`; `assets::lookup` is a byte-exact linear scan over compile-time `include_bytes!` entries. `src/browser/render.rs` and `src/browser/math.rs` reference the unversioned vendor URLs.

## Scope
Implement ADR-038 and NFR-007.003. Exactly three cache classes exist and the response kind decides the class:

- Immutable vendor: every file under `/assets/<package>/<asset-version>/` returns `Cache-Control: public, max-age=31536000, immutable`, with no `ETag` and never a 304.
- Revalidated first-party: `/assets/app.css`, `/assets/app.js`, and `/assets/nav-state.js` return `Cache-Control: no-cache` and one strong `ETag`; a matching `If-None-Match` on GET or HEAD returns a bodyless 304.
- No-store: every HTML, JSON, error, 404, and 405 response keeps `Cache-Control: no-store`, no validator, and never a 304.

Vendor URLs mirror `src/browser/vendor/<package>/<asset-version>/` exactly and the version segment is a literal compile-time table entry, equal to the vendor directory name and the manifest `version` column, not the Papyrus package version. The unversioned vendor paths are removed rather than aliased. First-party paths stay unversioned. The ETag is a strong validator of the form `"<64 lowercase hex characters>"` containing the SHA-256 of the exact served bytes, computed once per process from the embedded bytes and sharing one SHA-256 implementation with the manifest digest test. Conditional evaluation happens after the method check and exact lookup and applies only to a first-party hit: all `If-None-Match` lines are combined, `*` and an exact strong tag match, a weak or unknown or malformed tag does not match, and a match produces the fixed bodyless 304 header set.

The response writer emits the class from the `Response`; the default class is no-store, and no handler or route can supply `Cache-Control` through the extra-header list. Assets only: no record-derived response becomes cacheable, no compression or `Vary` is introduced, and the 404/405 matrix, MIME types, `Content-Length`, CSP, and security headers are unchanged. Retarget the existing header assertions named in ADR-038 decision 26 and add the new assertions of decision 27.

Build-path asset-version guard (ADR-038 decision 30, in scope because MS-036 owns asset versioning): `scripts/build-deb.sh` rejects a vendor-directory and manifest version mismatch before assembling the package. For every vendored package it enumerates the version directories under `src/browser/vendor/<package>/<version>/`, reads the `version` column of `src/browser/vendor/MANIFEST.md`, and reads the version cited in the `copyright` section. It fails the build with a clear stderr diagnostic when a package has no version directory, more than one version directory, a manifest row whose version differs, or a copyright section citing a different version. The check is a build-time verification of already committed data: no new file, build script, code generation, environment variable, network access, duplicate assembly, or tracked-file edit is introduced, and the asset version stays independent of the package version.

## Assumptions
- The asset version is the packaged asset version from the vendor directory and manifest, not a build timestamp and not the Papyrus package version.
- The production SHA-256 implementation may be moved out of the manifest test module so runtime and test share one implementation; no new crate is added.
- The ADR-038 decision 30 build-time refusal of a vendor-directory and manifest-version mismatch is in scope here: this task adds the script check in the build path plus its test, in addition to the runtime version-binding test.

## Definition of done
All acceptance criteria pass; the retargeted and new header assertions cover every class and the 404/405 matrix; bodies, MIME types, Content-Length, status codes, and record responses are unchanged; the build-path guard passes on the committed tree and rejects every mismatch class in test fixtures; the full suite, `cargo clippy --all-targets -- -D warnings`, and `cargo fmt --check` are clean; evidence is recorded on this task.
## Acceptance Criteria
- [ ] #1 Three cache classes exist and the response kind decides the class: immutable vendor, revalidated first-party, and no-store; the default class for a response that names none is no-store, only the asset table produces the other two classes, and no handler or route can supply a Cache-Control value through the extra-header list.
- [ ] #2 Vendor URLs are versioned: every vendor file is served from a literal compile-time path with the form assets package asset-version file; app.css, app.js, and nav-state.js keep their unversioned paths; there is no runtime version interpolation, prefix match, path join, or directory listing, and lookup stays byte-exact.
- [ ] #3 Version binding: every vendor path version segment equals the vendor directory name under the packaged tree and the manifest version column, and is not the Papyrus package version; a test asserts that the served path version, the manifest row version, and the embedded directory agree for every vendor entry.
- [ ] #4 Unversioned vendor paths are gone: the unversioned KaTeX stylesheet, KaTeX script, KaTeX font, and Mermaid script paths return 404 for GET, HEAD, and POST, with no redirect, alias, or duplicate table row.
- [ ] #5 Immutable vendor 200: every vendor entry on GET and HEAD returns 200 with the exact header set of Server, Content-Type, Content-Length, Connection: close, Cache-Control public max-age 31536000 immutable, X-Content-Type-Options nosniff, Referrer-Policy no-referrer, and X-Frame-Options DENY, and with no ETag, Last-Modified, Expires, Vary, Content-Encoding, or CSP.
- [ ] #6 Immutable ignores conditionals: a GET or HEAD on a vendor URL with If-None-Match of star, an exact tag, a weak tag, or a malformed value returns the full 200 and never a 304.
- [ ] #7 Revalidated first-party 200: app.css, app.js, and nav-state.js on GET and HEAD return 200 with Cache-Control no-cache and one strong ETag of the form double quote plus 64 lowercase hex characters plus double quote that equals the SHA-256 of the exact served bytes; the ETag is computed once per process, and no Last-Modified, Vary, or Content-Encoding is emitted.
- [ ] #8 Revalidated first-party 304 matrix: an exact tag on GET or HEAD returns a bodyless 304 with the same ETag and no-cache, and with no Content-Type, Content-Length, or CSP; star returns 304; a weak tag, an unknown tag, a malformed element, or a stale tag returns the full 200; a comma-separated list containing the exact tag returns 304; two separate If-None-Match header lines are combined and match.
- [ ] #9 No-store stays: the inventory page, a document view, a document edit, an API list, an API document read, an HTML 404, a JSON 404, a 405, and every error response carry Cache-Control no-store, no ETag, and never a 304; no record-derived response becomes cacheable and no response body or header becomes a function of a previous request.
- [ ] #10 Method and error matrix: PUT, DELETE, OPTIONS, and POST on a vendor URL or a first-party URL return 405 with Allow GET, HEAD before any conditional handling; an unknown asset path returns 404 for every method; 404, 405, and error responses are no-store, carry no ETag, and never return 304.
- [ ] #11 No compression: no response of any class carries Content-Encoding or Vary, no Accept-Encoding negotiation changes a response, and Transfer-Encoding remains rejected.
- [ ] #12 Page references use versioned vendor URLs: a math page references the versioned KaTeX stylesheet and script, a diagram page references the versioned Mermaid script, no served page references an unversioned vendor path, and the stylesheet reference-resolution test resolves every relative font URL under the versioned KaTeX directory.
- [ ] #13 The existing header assertions named in ADR-038 decision 26 are retargeted and the new assertions of decision 27 are added; the full suite, cargo clippy --all-targets -- -D warnings, and cargo fmt --check are clean.
- [ ] #14 No behaviour change beyond cache headers: asset bodies, MIME types, Content-Length values, status codes, the 404 and 405 matrix, record pages, JSON payloads, CSP, and the three security headers are unchanged.
- [ ] #15 Build-path asset-version guard (ADR-038 decision 30): the Debian assembly script rejects a vendor-directory and manifest version mismatch before assembling the package; it enumerates the version directories under the vendor tree, reads the version column of the asset manifest and the version cited in the copyright section, and exits non-zero with a clear stderr diagnostic when a package has no version directory, more than one version directory, a manifest row whose version differs, or a copyright section citing a different version; an automated test asserts the guard passes on the committed tree and fails on a fixture for each mismatch class; the asset version is not derived from the package version and no tracked file is edited by the check.
## Comments
- **planner**: planner: MS-036 owns asset versioning, so the ADR-038 decision 30 build-time guard is in scope here: AC #15 adds the script check in the Debian assembly build path (vendor version directories vs manifest version column vs copyright version) plus its test. No RC or package work; no tracked file edited by the check.
- **rust-dev**: Progress: implementation complete
- **rust-dev**: Cache classes implemented.
- **rust-dev**: Cache classes: vendor immutable; app no-cache; pages no-store.
- **rust-dev**: App assets use SHA-256 ETag.
- **rust-dev**: HTML JSON errors and 404 and 405 stay no-store.
- **rust-dev**: TASK-276 implemented at commit 235c9cd. Cache: versioned vendor URLs are public, max-age=31536000, immutable with no ETag; app.css, app.js and nav-state.js are no-cache with strong SHA-256 ETags computed once at browser startup; HTML, JSON, error, 404 and 405 responses stay no-store. A strong If-None-Match match on GET and HEAD returns a bodyless 304 with no Content-Type, Content-Length or CSP; weak, stale and malformed tags and all vendor conditionals return 200. Pages and the manifest use versioned vendor URLs; unversioned vendor paths return 404. The Debian build script checks each vendor version directory against the manifest version column and the copyright citation before assembly, failing non-zero with a stderr diagnostic on mismatch. cargo test --all-targets, clippy -D warnings and fmt --check are green; the guard passes the committed tree and rejects no-version, multiple-version, manifest-version and copyright-version fixtures.
- **orchestrator**: Acceptance met: three cache classes (immutable versioned vendor, no-cache with strong SHA-256 ETag and 304 for app assets, no-store for HTML and JSON), version binding with unversioned 404, no compression or Vary, and the build-time vendor, manifest and copyright guard with mismatch tests; suite, clippy and fmt green.
