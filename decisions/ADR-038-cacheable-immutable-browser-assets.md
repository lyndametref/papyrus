---
id: ADR-038
title: "Cacheable immutable browser assets"
created_date: '2026-11-20T19:47:59Z'
updated_date: '2026-11-20T19:48:00Z'
labels: []
references:
  - RE-010
  - ADR-0005
  - ADR-0025
  - ADR-0026
  - ADR-0027
  - ADR-0028
  - REQ-006.014
  - NFR-007.003
  - TECH-007
  - TECH-008
  - TECH-013
  - TECH-024
  - TECH-027
  - TECH-072
superseded_by: null
---
# ADR-038 Cacheable immutable browser assets

## Status
accepted
## Decision
## Supersession and amendments

1. This ADR supersedes ADR-0027 decision 8, and only that decision. The retention of `Cache-Control: no-store`, the prohibition of cache semantics and `ETag`, and the statement that serving vendor assets with a long-lived cache is a separate decision that needs its own ADR are superseded. The no-compression and no-range clauses of the same decision survive and are restated in decisions 23 and 24. ADR-0027 decisions 1-7, 9, and 10 — the asset table, binary-safe bodies, the vendored version directories, the manifest digest test, the licence notices, the weight budget, and the load discipline — remain in force unchanged, and ADR-0027 keeps its accepted status because the supersession is by decision, not in full.
2. This ADR supersedes one clause of ADR-0005: the `Cache-Control: no-store` entry in the response-header list of its security-posture decision bullet, and only that entry. Every other entry of that list (CSP, `X-Frame-Options: DENY`, `X-Content-Type-Options: nosniff`, `Referrer-Policy: no-referrer`, `Host`/`Origin` validation, no authentication, loopback bind, no outbound network), the transport rules (one request per connection, `Connection: close`, no keep-alive, no pipelining, no TLS), and the record, render, and route baseline remain in force. ADR-0005 keeps its accepted status.
3. This ADR amends ADR-0026 decision 4 for Mermaid loading, without superseding the record or the decision. The served path gains the asset-version segment of decision 8, and the sentence stating that `no-store` is unchanged and that a diagram page re-sends the 3.4 MiB bundle on each navigation is retired. The 11.17.2 pin, the single file, the SHA-256, the `LICENSE`/`UPSTREAM.md`/manifest obligations, the deferred classic-script load, the load order, and the rule that the bundle is linked only on pages with a renderable `mermaid` figure are unchanged.
4. Nothing else is superseded or amended here. ADR-0025 decision 2 records the KaTeX serve paths; those paths gain the version segment under decision 8, and the literal paths it records are updated by its own amendment rather than here. No clause about routing, exact matching, MIME types, CSP, any header other than `Cache-Control` and `ETag`, notices, digests, binary budgets, or record content changes.

## Cache classes

5. Every response belongs to exactly one cache class, and the class follows from what the response *is*, never from the route that happened to produce it:

   | Class | Responses | `Cache-Control` | Validator | `304` possible |
   |---|---|---|---|---|
   | Immutable vendor | every file under `/assets/<package>/<asset-version>/` | `public, max-age=31536000, immutable` | the URL only; no `ETag` | no |
   | Revalidated first-party | `/assets/app.css`, `/assets/app.js`, `/assets/nav-state.js` | `no-cache` | strong `ETag` | yes |
   | No-store | every HTML response, every JSON response, every error and not-found response, and anything that is not a file in the asset table | `no-store` | none | no |

6. The exact header sets are fixed:

   - Immutable vendor `200`: `Server: papyrus`, `Content-Type` (the asset's MIME type), `Content-Length`, `Connection: close`, `Cache-Control: public, max-age=31536000, immutable`, `X-Content-Type-Options: nosniff`, `Referrer-Policy: no-referrer`, `X-Frame-Options: DENY`. No `ETag`, no `Last-Modified`, no `Expires`, no `Vary`, no `Content-Encoding`, no CSP.
   - Revalidated first-party `200`: the same set, with `Cache-Control: no-cache` and one strong `ETag`.
   - Revalidated first-party `304`: `Server: papyrus`, `Connection: close`, `Cache-Control: no-cache`, the same `ETag`, `X-Content-Type-Options: nosniff`, `Referrer-Policy: no-referrer`, `X-Frame-Options: DENY`. No `Content-Type`, no `Content-Length`, no body, no CSP.
   - No-store: unchanged from today — `Content-Type`, `Content-Length`, `Connection: close`, `Cache-Control: no-store`, the three security headers, and the CSP on HTML responses only.

7. `max-age` is exactly one year (`31536000` seconds) and `immutable` is the RFC 8246 directive. `public` is correct because the interface has no credentials, cookies, sessions, or per-client variability (ADR-0005), so no shared cache can leak anything. Only the values above are emitted; no handler supplies its own `Cache-Control`, and no other `Cache-Control` directive or parameter (`private`, `must-revalidate`, `stale-while-revalidate`, `s-maxage`) is used.

## Versioned vendor URLs

8. Every vendored file is served from a path that carries its packaged asset version:

   ```
   /assets/<package>/<asset-version>/<file>
   ```

   The path mirrors the vendored directory `src/browser/vendor/<package>/<asset-version>/` exactly, so the served path is the embedding path with `src/browser/vendor` replaced by `/assets`. For the current inventory that is:

   | Served path | Embedded file |
   |---|---|
   | `/assets/katex/0.16.11/katex.min.css` | `vendor/katex/0.16.11/katex.min.css` |
   | `/assets/katex/0.16.11/katex.min.js` | `vendor/katex/0.16.11/katex.min.js` |
   | `/assets/katex/0.16.11/fonts/<name>.woff2` (20 files) | `vendor/katex/0.16.11/fonts/<name>.woff2` |
   | `/assets/mermaid/11.17.2/mermaid.min.js` | `vendor/mermaid/11.17.2/mermaid.min.js` |

   The first-party files `/assets/app.css`, `/assets/app.js`, and `/assets/nav-state.js` keep their unversioned paths; they are revalidated, not versioned.

9. `<asset-version>` is the packaged asset version: the version of the vendored package as it is embedded in this binary, which is the version directory name, which equals the manifest `version` column and, for an unpatched file, the upstream release version. It is **not** the Papyrus package version. A Papyrus release that does not change an asset does not change that asset's URL, and a Papyrus version bump alone does not invalidate a vendor cache. A vendored file that is patched under ADR-0027 decision 3 takes a new asset version identifier that is not an upstream release — `<upstream-version>-p<n>` — with a new directory, a new manifest row, and an updated notice.

10. The version segment is a literal part of the table entry, not a runtime value: each path is a compile-time constant next to its `include_bytes!`, exactly as today. There is no per-request version interpolation, no version negotiation from a request header or a query parameter, no prefix match, no path join, and no directory listing. `assets::lookup` stays a byte-exact scan, and a hit keeps the ADR-0027 method rules (GET/HEAD `200`, any other method `405` with `Allow: GET, HEAD`, a miss `404` for every method).

11. The unversioned vendor paths are removed, not aliased. `/assets/katex/katex.min.css`, `/assets/katex/katex.min.js`, `/assets/katex/fonts/<name>.woff2`, and `/assets/mermaid/mermaid.min.js` return `404` after this change. There is no redirect, no duplicate table row, and no second URL for the same bytes: a second URL would make the cache key ambiguous and double the reference surface the offline-reference tests must check.

12. Pages reference versioned vendor URLs and nothing else. A math page links `/assets/katex/0.16.11/katex.min.css` and `/assets/katex/0.16.11/katex.min.js`; a diagram page links `/assets/mermaid/11.17.2/mermaid.min.js`; `app.css`, `app.js`, and `nav-state.js` keep their current references. KaTeX's 60 relative `url(fonts/…)` references resolve against the versioned directory, so `/assets/katex/0.16.11/katex.min.css` loads `/assets/katex/0.16.11/fonts/…`, still byte-identical to upstream and still every entry of the asset table.

13. Immutable responses never carry a validator and never answer `304`. A conditional request to a vendor URL is answered with the full `200` and body: the URL is the validator, so there is nothing to revalidate.

## First-party assets and revalidation

14. The revalidated class is exactly the three first-party files: `/assets/app.css`, `/assets/app.js`, and `/assets/nav-state.js`. They change with Papyrus, not with an upstream release, so a version segment would have to change on every release; a validator gives the same correctness with one cheap round trip. `nav-state.js` is included because it is a first-party asset served from the same table; leaving it on `no-store` would create a third class for first-party files with no reason.

15. `Cache-Control: no-cache` means "a cache may store this response but must revalidate before reuse". There is no `max-age`, because there is no age at which the bytes are known to be current. `no-cache` without a `max-age` is the contract; `max-age=0, must-revalidate` is not used, because the NFR names `no-cache` and the two differ in the presence of a freshness lifetime.

16. The `ETag` is a strong validator with the exact syntax `"<64 lowercase hex characters>"`: a double quote, the SHA-256 of the exact served byte sequence, a double quote. No `W/` prefix, no vendor-specific format, no version string. SHA-256 is used because the manifest digest test already computes SHA-256 over every embedded asset and compares it with `src/browser/vendor/MANIFEST.md`; the validator and the digest therefore come from one implementation, and no second digest is stored.

17. The validator is computed once per process from the embedded bytes and held for the process lifetime; it is not computed per request, not read from configuration, and not a second hard-coded constant that could drift from the bytes. Hashing the three first-party files is a one-time cost over roughly 31 KB.

18. `If-None-Match` is the only conditional request header that has any effect, and only for the revalidated class on `GET` and `HEAD`:

   - All `If-None-Match` header lines are combined into one comma-separated list, as HTTP requires, and each element is trimmed of spaces and tabs.
   - `*` matches the current representation.
   - A strong entity-tag matches only when it is byte-identical to the asset's `ETag`, quotes included.
   - A weak tag (`W/"…"`) never matches: `If-None-Match` on `GET` uses the strong comparison function, and this server issues no weak validators.
   - An unparseable element or an unknown tag does not match; the response is the normal `200` with the full body.
   - A match produces `304`; no match produces the normal `200`.
   - `If-Modified-Since`, `If-Match`, `If-Unmodified-Since`, and `If-Range` are ignored, and no `Last-Modified` is issued, because there is no file mtime to expose and the ETag is the only validator.

19. The `304` is bodyless and terminal: the connection still closes, no `Content-Length: 0` is emitted, and the response does not carry `Content-Type` or a CSP. `HEAD` with a matching `If-None-Match` also returns `304`. A conditional `POST` to an asset path is still `405` with `Allow: GET, HEAD`; method rejection happens before conditional evaluation.

## Writer and method rules

20. The hard-coded `Cache-Control: no-store` line in the response writer is removed. Each `Response` carries one cache class, the writer emits exactly one `Cache-Control` line and, when the class is revalidated, one `ETag` line from it, and no route or handler can supply `Cache-Control` through the extra-header list. The default class for every response constructor that does not name a class is no-store, so a new page cannot become cacheable by omission; only the asset table produces the immutable and revalidated classes.

21. Conditional evaluation happens in the asset route, after the method check and after the exact lookup, and applies only to a revalidated hit. A `404` for an unknown asset path, a `405` for a wrong method, and every error response are no-store; they never carry an `ETag` and never produce a `304`.

## What does not change

22. Scope is assets only. Every HTML response (inventory, list, view, edit, not-found, error), every JSON response (API, errors), and every record read stays `Cache-Control: no-store`. No record-derived byte becomes cacheable, and no response body or header becomes a function of a previous request.

23. No compression is introduced. There is no `Content-Encoding`, no `Accept-Encoding` negotiation, no pre-compressed body, and no `Vary: Accept-Encoding`; `Transfer-Encoding` stays rejected. This is unchanged from ADR-0005 and ADR-0027 decision 8 and is deliberately out of scope here, even though compression is the other large transfer win.

24. Range requests, keep-alive, pipelining, conditional writes, `Content-MD5`, `Server-Timing`, service workers, the Cache API, `localStorage`, IndexedDB, and any client-side asset cache remain absent. Caching is HTTP response caching only, and the binary remains the only delivery mechanism.

25. CSP, MIME types, `Content-Length`, the three security headers, exact-match routing, the `404`/`405` matrix, the asset manifest and digest test, the notice file, and the 12 MiB budget are unchanged. The CSS reference-resolution test keeps its rule — every `url(` in every served stylesheet resolves to an entry of the asset table — with the relative resolution now landing under the versioned directory.

## Test retargeting

26. The existing header assertions move from every response to HTML and JSON responses, and the asset assertions become class-specific:

   | Location | Current assertion | Retargeted assertion |
   |---|---|---|
   | `src/browser/http.rs::write_response` | writes `Cache-Control: no-store` for every response | writes the `Cache-Control` line, and the `ETag` line for the revalidated class, from the `Response` cache class, plus the `304` header set of decision 6 |
   | `tests/browser_cli.rs::t143_asset_table_serves_every_entry` | every entry of `ASSETS` returns `Cache-Control: no-store` on `GET` and on `HEAD` | vendor entries return `public, max-age=31536000, immutable` and no `ETag`; first-party entries return `no-cache` and a strong `ETag`; `GET` and `HEAD` headers still match. The unversioned vendor paths join the `404` matrix: `/assets/katex/katex.min.css`, `/assets/katex/fonts/KaTeX_Main-Regular.woff2`, `/assets/mermaid/mermaid.min.js` |
   | `tests/browser_cli.rs::head_matches_get_without_body` | compares `Cache-Control` between `GET` and `HEAD` across a mixed path list, with no literal value | keeps the comparison for every path; adds `ETag` to the compared header set for `/assets/app.css` and `/assets/app.js`; adds one vendor path with the immutable value; asserts the literal `no-store` for `/`, `/doc/REQ-001.001`, `/api/requirements`, and `/api/doc/REQ-001.001` |
   | `tests/backlog_discovery_smoke.rs::milestone_backlog_discovery_flow` | `/assets/app.css` returns `no-store` | `/assets/app.css` returns `no-cache` and an `ETag`; the literal `no-store` moves to the JSON response and the HTML response the same test already fetches |
   | `tests/search_browser_smoke.rs::milestone_search_and_browser_flow` | `/assets/app.css` returns `no-store` | the same retarget: `no-cache` plus `ETag` on the asset, `no-store` on the JSON and HTML response |
   | `tests/release_document_types_smoke.rs::t117_08_browser_matrix_over_http` | the asset loop (`/assets/app.css`, `/assets/app.js`) returns `no-store` | the loop returns `no-cache` plus `ETag`; one vendor path returns the immutable value; `no-store` is asserted on the HTML page and the JSON response the test already fetches |
   | `tests/technical_specifications_smoke.rs::milestone_technical_specifications_flow` | `/assets/app.css` and `/assets/app.js` return `no-store` | `no-cache` plus `ETag`; `no-store` moves to the HTML and JSON responses |
   | `tests/use_cases_test_cases_test_reports_smoke.rs::milestone_use_cases_test_cases_test_reports_flow` | `/assets/app.css` and `/assets/app.js` return `no-store` | `no-cache` plus `ETag`; `no-store` moves to the HTML and JSON responses |

27. New assertions are required, beyond the retargeted ones:

   - **Immutable vendor `200`:** for every vendor entry of `ASSETS` — KaTeX CSS, KaTeX script, all 20 KaTeX fonts, and Mermaid — `GET` and `HEAD` return `200`, the exact `Cache-Control: public, max-age=31536000, immutable`, no `ETag`, the documented MIME type, the exact `Content-Length`, and no CSP.
   - **Immutable ignores conditionals:** a `GET` or `HEAD` on a vendor URL with `If-None-Match: *` or with an exact matching tag returns `200`, never `304`; `GET` returns the full bytes.
   - **Unversioned vendor paths are gone:** `/assets/katex/katex.min.css`, `/assets/katex/katex.min.js`, one font path, and `/assets/mermaid/mermaid.min.js` return `404` for `GET`, `HEAD`, and `POST`.
   - **Version binding:** every vendored asset path matches `/assets/<package>/<version>/…`, and that `<version>` equals the manifest `version` column and the `src/browser/vendor/<package>/<version>/` directory of the embedded file. `src/browser/assets.rs::manifest_covers_every_asset_with_matching_bytes_and_digest` gains this check.
   - **App `304`:** `GET /assets/app.css` returns `200`, `no-cache`, and a strong `ETag` of the form `"<64 hex>"`; a conditional `GET` with that exact tag returns `304` with an empty body, the same `ETag`, `no-cache`, no `Content-Type`, no `Content-Length`, and no CSP; a conditional `HEAD` returns `304`; a stale tag returns `200` with the full body; `*` returns `304`; `W/"<tag>"` returns `200`; a comma list containing the exact tag returns `304`; two separate `If-None-Match` header lines are combined and match. The same matrix runs for `/assets/app.js` and `/assets/nav-state.js`.
   - **No-store stays:** `/`, `/doc/REQ-001.001`, `/api/requirements`, `/api/doc/REQ-001.001`, an HTML not-found response, and a JSON not-found response return `Cache-Control: no-store`, no `ETag`, and `200`/`404` with a body when sent `If-None-Match`.
   - **No compression:** no response of any class carries `Content-Encoding` or `Vary`.
   - **Page references:** a math page contains `/assets/katex/0.16.11/katex.min.css` and `/assets/katex/0.16.11/katex.min.js`; a diagram page contains `/assets/mermaid/11.17.2/mermaid.min.js`; no served page contains `/assets/mermaid/mermaid.min.js` or `/assets/katex/katex.min.css`; the stylesheet reference-resolution test resolves every `url(` under `/assets/katex/0.16.11/`.
   - **Method matrix:** `PUT`, `DELETE`, `OPTIONS`, and `POST` on a vendor URL and on a first-party URL still return `405` with `Allow: GET, HEAD`, before any conditional handling.

28. This milestone makes no source, test, or packaging change. The writer, the asset table, the page references, the retargeted assertions, the new assertions, and the packaging check are implementation work in the release milestone that follows; ADR-038 fixes the contract they must meet.

## Asset version and packaging

29. The asset version is the version directory name under `src/browser/vendor/<package>/`. Because an immutable URL may be cached for a year with no revalidation, the version segment is the only invalidation lever, and changing the bytes without changing the version is a defect:

   - Replacing a vendored file in place — a documented ADR-0027 decision 3 patch, a re-fetched tarball, or a hand edit — leaves the URL unchanged, so a browser that stored the old bytes keeps them until `max-age` expires. No server response can force a revalidation of that URL, because immutable responses carry no `ETag`.
   - The mitigation is a rule, not a mechanism: every vendored change produces a new asset version identifier (new directory, new manifest row with a new digest, updated notice, new URL), including a patch to an otherwise identical upstream file.
   - Three checks make the rule enforceable: the manifest digest test fails when a vendored byte changes without a manifest update; the version-binding assertion of decision 27 fails when the URL version, the manifest `version`, and the vendor directory disagree; and packaging refuses to build a `.deb` whose vendor version directories and manifest rows disagree (decision 30).
   - Residual risk is accepted and stated: within one published version there is no way to invalidate an immutable vendor URL, and an operator who bypasses the version rule ships stale bytes. Detection is at build time, through the tests and the packaging check, not at runtime.

30. Packaging computes the asset version from the packaged assets, not from the package version:

   - `scripts/build-deb.sh` enumerates the version directories of `src/browser/vendor/<package>/<version>/` — the same directories the `include_bytes!` paths name — and reads the `version` column of `src/browser/vendor/MANIFEST.md`.
   - It fails the package build when a package has no version directory, more than one, a manifest row with a different version, or a `copyright` section citing a different version. The package therefore cannot contain an asset whose served version disagrees with the version it packages.
   - It keeps its existing derivation of the Papyrus package version from `Cargo.toml` for `DEBIAN/control`; that value is unrelated to the asset version and does not appear in any asset URL.
   - No new file, build script, code generation, environment variable, or network access is introduced: `cargo build` stays the whole build (ADR-0027 decision 2), and the check is a packaging-time verification of data that is already committed.

## Downstream constraints

1. Exactly three cache classes exist, and the response kind decides the class: immutable vendor files, revalidated first-party files, and no-store for everything else. No route or handler chooses a `Cache-Control` value.
2. Vendor URLs are `/assets/<package>/<asset-version>/<file>`, mirroring `src/browser/vendor/<package>/<asset-version>/`; the unversioned vendor paths are `404`, with no redirect and no alias.
3. Immutable responses carry `public, max-age=31536000, immutable`, no `ETag`, no `Last-Modified`, no `Vary`, and never a `304`.
4. The first-party class is exactly `/assets/app.css`, `/assets/app.js`, and `/assets/nav-state.js`; each carries `no-cache` and a strong SHA-256 `ETag`, and a matching `If-None-Match` on `GET` or `HEAD` returns a bodyless `304`.
5. HTML, JSON, error, `404`, and `405` responses stay `no-store`, carry no validator, and never return a `304`.
6. No compression, no `Content-Encoding`, no `Accept-Encoding` negotiation, no `Vary`, and no range request is added.
7. Every vendored change changes the asset version identifier; a manifest row, the URL version, and the vendor directory name are one value.
8. CSP, MIME types, exact-match routing, the `404`/`405` matrix, the manifest digest test, the notices, and the 12 MiB budget are unchanged.
## Context
Every browser response today carries `Cache-Control: no-store`, emitted from one hard-coded line in `browser::http::write_response`. That includes the embedded asset table: `/assets/app.css`, `/assets/app.js`, `/assets/nav-state.js`, KaTeX 0.16.11 (23,335 B of CSS, 275,414 B of script, and 20 WOFF2 fonts totalling ≈259,792 B), and Mermaid 11.17.2 (3,572,661 B), each served byte-exactly from `src/browser/assets.rs`.

The bytes are identical for every request, every user, and every session, yet a diagram page re-transfers the 3.57 MB Mermaid bundle on every navigation, and a math page re-transfers the KaTeX stylesheet and script and every font it uses. The bootstrap is the largest cost of a first paint and of every later navigation, and it is unrelated to the record reads that NFR-007.001 and NFR-007.002 measure.

ADR-0027 decision 8 chose that deliberately and named the follow-up: `no-store` was retained, and "serving vendor assets with a long-lived cache is a separate decision and would need its own ADR because it changes ADR-0005's header contract". RE-010 decision 12 is that decision, and NFR-007.003 is the resulting requirement: versioned immutable vendor files, `no-cache` plus ETag and a 304 path for the app assets, `no-store` for HTML and JSON, assets only, no compression.

Forces:

- Repeat visits must transfer no vendor bytes and must revalidate the app assets cheaply, for a single-user loopback interface and for the 1,000-record corpus the performance work targets.
- No record response may become cacheable. Record content changes through the browser and the CLI, and a cached record page or JSON response would show a stale version with no invalidation signal; `no-store` on HTML and JSON is part of the read-after-write and privacy posture, not an incidental header.
- The build and the package stay as they are: one binary, `cargo build` as the whole build, no runtime asset directory, no Node toolchain, no new Rust crate (ADR-0001, ADR-0004, ADR-0027 decisions 2 and 6).
- Routing stays an exact string match over a static table with no path join, no directory expansion, and no percent-decoded filesystem access (ADR-0027 decision 1). A versioned URL must therefore be a literal table entry.
- No compression, no range requests, and no `Accept-Encoding` negotiation (ADR-0005, ADR-0027 decision 8). Compression remains the other large transfer win and remains out of scope.
- The browser floor and the offline behaviour do not change: no `Service-Worker`, no Cache API, no storage beyond the navigation-state key ADR-0028 already stores.
- A vendor cache must be invalidated by an actual asset change and not by an unrelated Papyrus release, and the invalidation lever must be visible in the source tree, the manifest, and the package, not hidden in a build timestamp.
- The existing suite asserts literal header values on asset responses in six test files; the retargeting must be deliberate and must not weaken the no-store guarantee for records.

Alternatives considered:

| Option | Outcome | Reason |
|---|---|---|
| Three cache classes, vendor URLs versioned by the packaged asset version, first-party assets revalidated by a SHA-256 ETag (this ADR) | Selected | Satisfies NFR-007.003 and RE-010 decision 12, keeps exact-match routing and the single binary, and invalidates vendor caches per asset rather than per Papyrus release. |
| Keep `no-store` everywhere (ADR-0027 decision 8) | Rejected | NFR-007.003 requires that repeat visits transfer no vendor bytes; the current cost is up to 3.9 MB per navigation independent of record reads. |
| Version vendor URLs by the Papyrus package version (`/assets/v/0.1.0/…`) | Rejected | Every Papyrus release would invalidate 3.9 MB of unchanged vendor bytes, and the asset version would become a second meaning of the package version with no asset-specific audit value. |
| Version vendor URLs by a query string (`?v=0.16.11`) | Rejected | The version would be invisible to the exact-match table, so it could not be checked against the manifest or the vendor directory, and the unversioned path would remain a second cache key for the same bytes. |
| Content-addressed vendor URLs (`/assets/<sha256>/…`) | Rejected | The hash would have to be produced by a build step, injected into every page render, and cross-checked against the notices; the packaged asset version already exists in the tree, the manifest, and the package, and is the value a reviewer can audit. |
| `no-store` plus an ETag for vendor assets | Rejected | Revalidation still costs one round trip per asset per navigation; an immutable version is available by construction and costs nothing at runtime. |
| `max-age` without `immutable` on vendor assets | Rejected | A normal reload revalidates, which defeats the point for the 3.57 MB bundle; the URL already carries the asset identity. |
| `max-age=0, must-revalidate` for the app assets instead of `no-cache` | Rejected | The NFR names `no-cache`; both force revalidation, but only `no-cache` states the intent without inventing a freshness lifetime. |
| Pre-compressed bodies or `Accept-Encoding` negotiation | Rejected here | Out of scope by NFR-007.003 and ADR-0005; it needs negotiation, a compressed-body path, and a `Vary` key. |
| A service worker or client-side cache for the assets | Rejected | A second cache with its own lifetime, update path, and failure modes, for what HTTP caching already provides; it would also add client state the project does not otherwise keep. |
## Consequences
**Positive:**

- A repeat visit transfers no vendor bytes. The 3.57 MB Mermaid bundle, the KaTeX script and stylesheet, and all 20 fonts are fetched once per asset version per browser, which removes the dominant bootstrap cost from every later navigation and from every diagram or math page.
- `app.css`, `app.js`, and `nav-state.js` are revalidated instead of re-downloaded: a repeat navigation costs one conditional request per file and a bodyless `304`, not ≈31 KB of bodies.
- Caching is scoped by construction: the class comes from the response kind, not from a route, so no record page, JSON document, or error response can become cacheable by mistake.
- The invalidation lever is visible and auditable. The asset version appears in the served URL, in the vendor directory, in the manifest row, and in the package check, so a review can answer "which bytes does this URL promise" from the tree alone.
- Exact-match routing, the static table, the single binary, and `cargo build` are untouched. The change is three header values and one conditional path, not a new delivery mechanism.
- Vendor caches survive Papyrus releases that do not touch the asset, so the 3.9 MB is not re-downloaded on every Papyrus upgrade.
- The ETag reuses the SHA-256 implementation the manifest test already needs, so the validator, the digest, and the notice inventory cannot drift into three different truths.

**Negative:**

- The hand-written HTTP server gains a conditional-request path, a cache-class concept on `Response`, a header-emission change in the writer, and an `ETag` computed at startup. That is more protocol surface than a single header line, and it is maintained in-project (ADR-0005).
- A stale app asset is now possible in principle where `no-store` made it impossible: a browser must revalidate, and a proxy or extension that ignores `no-cache` can serve an old stylesheet against a newer HTML page. The ETag makes the correct behaviour cheap, not mandatory on the client.
- The asset version becomes a release-critical identifier. A vendored change that forgets to bump it ships stale bytes to every browser that already cached the URL, and nothing at runtime can repair that.
- Six existing test suites assert `no-store` on assets. They must be retargeted rather than relaxed, which adds review work and a temporary window in which the specs (TECH-007, TECH-008, TECH-013, TECH-024, TECH-027, TECH-069) describe the old header until their owning tasks update them.
- More URLs exist in the table: every vendor file gains a version segment, so the table, the manifest, and the notice rows grow by one field each, and the unversioned paths must be removed rather than left as aliases.

**Risks and mitigations:**

- *A vendored byte change without a version change*: the manifest digest test fails on the changed bytes, the version-binding assertion fails when the URL, the manifest `version`, and the vendor directory disagree, and packaging refuses to build a `.deb` in that state. The residual case — an operator who updates both manifest and directory in place without a new version — is a process failure the release checklist owns, and it is recorded rather than mechanically prevented.
- *A record response accidentally cached*: no-store is the default class, only the asset table produces the other classes, and no handler can set `Cache-Control`; a test asserts `no-store` on HTML, JSON, error, and not-found responses.
- *A wrong `304` served for different bytes*: the ETag is the SHA-256 of the exact bytes, the comparison is strong and byte-exact, and an unknown, weak, or malformed tag never matches.
- *A `304` with a body or a length*: the `304` header set is fixed and asserted, including the absence of `Content-Type` and `Content-Length`.
- *KaTeX font resolution breaks under the version segment*: the relative `url(fonts/…)` references resolve inside the versioned directory, and the existing reference-resolution test asserts that every `url(` in every served stylesheet resolves to an asset-table entry.
- *Old pages reference unversioned vendor paths*: those paths are `404`, not aliases. A page is always served by the same binary that serves its assets, and a page cached from an older binary re-requests an old URL that the new binary does not serve; the clean failure is a missing stylesheet or diagram script, not mismatched bytes. The alternative, aliasing the old URL to new bytes under an immutable header, would be wrong by definition.
- *The suite keeps asserting `no-store` on assets after the change*: the retargeting table in decision 26 names every assertion, and the change is not complete until the new assertions of decision 27 pass.
- *Scope creep into compression*: decisions 23 and 6 forbid `Content-Encoding` and `Vary`, and the new no-compression assertion covers every class.
