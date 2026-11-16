---
id: TASK-143
title: Add the vendored browser asset table, digest manifest, and package notices
status: Done
assignee:
  - '@rust-dev'
created_date: '2026-11-15T16:13:00Z'
updated_date: '2026-11-15T19:02:00Z'
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
  - docs/specifications/rust-cli-and-debian-package-baseline.md
  - src/browser/assets.rs
  - src/browser/vendor/MANIFEST.md
modified_files:
  - src/browser/assets.rs
  - src/browser/mod.rs
  - src/browser/http.rs
  - src/browser/vendor/katex/0.16.11/katex.min.js
  - src/browser/vendor/katex/0.16.11/katex.min.css
  - src/browser/vendor/katex/0.16.11/fonts
  - src/browser/vendor/katex/0.16.11/LICENSE
  - src/browser/vendor/katex/0.16.11/UPSTREAM.md
  - src/browser/vendor/MANIFEST.md
  - packaging/deb/usr/share/doc/papyrus/copyright
  - tests/browser_cli.rs
priority: high
ordinal: 41560
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why.** ADR-0025 and ADR-0026 need vendored browser assets inside the single binary, and ADR-0027 is Accepted. The current asset model cannot serve them: `mod.rs` embeds exactly `app.css` and `app.js` and matches two hard-coded paths, `Response::text` cannot carry binary bytes, and the package notice file carries no third-party sections. The "no `url(`, no web font" assertions and specification wording were written for that two-asset model.

**What.** The static asset table, binary-safe responses, the vendored KaTeX layout, the digest manifest, and the package notices.

**Contract.** Normative: ADR-0027 decisions 1–10, ADR-0025 decision 4 for the KaTeX file set, ADR-0005 as amended. Tightened form: `docs/specifications/rust-cli-and-debian-package-baseline.md` → "Bundled browser assets and third-party notices (TASK-143)". That section is the source for this ticket's asset, vendor, digest, and notice clauses; the page-level wording revision belongs to TASK-148.

**Baseline verified at refinement time.**
- `src/browser/mod.rs`: `APP_CSS`/`APP_JS` at lines 32–33; the two hard-coded asset branches at lines 608 and 616; `head_only = request.method == Method::Head` at line 303, already passed to `http::write_response`.
- `browser::http::Response` already holds `body: Vec<u8>`, and `write_response` already omits the body for `HEAD` while keeping `Content-Length: <body.len()>`, so binary bodies need a constructor only.
- `route()` returns `Err(HttpError::new(405))` for a known path with a wrong method and `handle_request` renders it through `error_response`, which adds `Allow: GET, HEAD, POST`. That behaviour is asserted today and must not change.
- `parse_target` percent-decodes the path once and rejects `%2F`, `%5C`, `%00`, `.` and `..` segments before routing, so the table only ever sees a decoded, validated path.
- `packaging/deb/usr/share/doc/papyrus/copyright` is the only file under that directory, and `scripts/build-deb.sh` copies it with one `cp`.
- KaTeX 0.16.11 archive `https://registry.npmjs.org/katex/-/katex-0.16.11.tgz`, SHA-256 `8c5494c5f7d8e8bf73ff3fe64d4bd09a340e4878b7eec9bbd9ef8d8f3e24193f`. Its `dist/` holds `katex.min.js` (275,414 B), `katex.min.css` (23,335 B), and 60 `url(fonts/…)` values: 20 `@font-face` blocks × {woff2, woff, ttf}. No `.map`, no `@import`, no `sourceMappingURL`. The archive's root `LICENSE` is MIT (1,107 B) and the archive contains **no** OFL text.

## In scope

1. **Asset table and routing.** New `src/browser/assets.rs`:
   - `pub struct Asset { pub path: &'static str, pub mime: &'static str, pub bytes: &'static [u8] }`, `pub static ASSETS: &[Asset]`, `pub fn lookup(path: &str) -> Option<&'static Asset>`.
   - `lookup` is a linear scan for byte-exact, case-sensitive equality. No prefix matching, no directory index, no path join, no percent-decoding, no filesystem access — nothing in the module references `std::fs` or `Path`/`PathBuf`.
   - `path` is the exact request path including the leading `/`.
   - `src/browser/mod.rs` replaces both hard-coded branches with one router branch: any path starting with `/assets/` is resolved only by the table — hit with `GET`/`HEAD` → `200` with `asset.mime` and `asset.bytes`; hit with any other method → `Err(HttpError::new(405))`; miss → `Err(HttpError::new(404))`. The `/assets/` prefix is a gate only; `/assets/`, `/assets/app.css/`, and `/assets/nope` are misses. Every other path routes unchanged.
   - A `405` for an asset and a `404` for a miss reuse the existing error rendering, including the current `Allow` header. No asset-specific error body is added.

2. **Binary responses.** `Response::bytes(status: u16, content_type: &'static str, body: &'static [u8]) -> Self` in `src/browser/http.rs`. The MIME set is exactly `text/css; charset=utf-8` (stylesheets), `text/javascript; charset=utf-8` (scripts), `font/woff2` (WOFF2 fonts). `Response::text` stays for its remaining callers.

3. **Vendor layout.** `src/browser/vendor/katex/0.16.11/` contains `katex.min.js`, `katex.min.css`, `fonts/` with the 20 `.woff2` files, `LICENSE`, and `UPSTREAM.md`. Upstream's `dist/` is flattened onto the version root, which is what makes the stylesheet's relative `url(fonts/…)` resolve against `/assets/katex/katex.min.css` to `/assets/katex/fonts/…`. Flattening a directory does not alter bytes, so it is not a patch; the archive path (`package/dist/…`) is recorded in `UPSTREAM.md`. All vendored files are byte-identical to upstream and no `.patch` file exists.

   `UPSTREAM.md` fields: package, version, exact download URL, per-file archive path, archive SHA-256, retrieval date (UTC, `YYYY-MM-DD`), and a "no file modified" statement.

   `LICENSE`, `UPSTREAM.md`, and `MANIFEST.md` are not `ASSETS` entries and are never served.

4. **Exact asset-table entries after this task (23).** `/assets/app.css` and `/assets/app.js` keep their current paths, bytes, and MIME types and simply move onto the table. Added:

   | Path | Bytes |
   |---|---|
   | `/assets/katex/katex.min.css` | 23,335 |
   | `/assets/katex/katex.min.js` | 275,414 |
   | `/assets/katex/fonts/KaTeX_AMS-Regular.woff2` | 28,076 |
   | `/assets/katex/fonts/KaTeX_Caligraphic-Bold.woff2` | 6,912 |
   | `/assets/katex/fonts/KaTeX_Caligraphic-Regular.woff2` | 6,908 |
   | `/assets/katex/fonts/KaTeX_Fraktur-Bold.woff2` | 11,348 |
   | `/assets/katex/fonts/KaTeX_Fraktur-Regular.woff2` | 11,316 |
   | `/assets/katex/fonts/KaTeX_Main-Bold.woff2` | 25,324 |
   | `/assets/katex/fonts/KaTeX_Main-BoldItalic.woff2` | 16,780 |
   | `/assets/katex/fonts/KaTeX_Main-Italic.woff2` | 16,988 |
   | `/assets/katex/fonts/KaTeX_Main-Regular.woff2` | 26,272 |
   | `/assets/katex/fonts/KaTeX_Math-BoldItalic.woff2` | 16,400 |
   | `/assets/katex/fonts/KaTeX_Math-Italic.woff2` | 16,440 |
   | `/assets/katex/fonts/KaTeX_SansSerif-Bold.woff2` | 12,216 |
   | `/assets/katex/fonts/KaTeX_SansSerif-Italic.woff2` | 12,028 |
   | `/assets/katex/fonts/KaTeX_SansSerif-Regular.woff2` | 10,344 |
   | `/assets/katex/fonts/KaTeX_Script-Regular.woff2` | 9,644 |
   | `/assets/katex/fonts/KaTeX_Size1-Regular.woff2` | 5,468 |
   | `/assets/katex/fonts/KaTeX_Size2-Regular.woff2` | 5,208 |
   | `/assets/katex/fonts/KaTeX_Size3-Regular.woff2` | 3,624 |
   | `/assets/katex/fonts/KaTeX_Size4-Regular.woff2` | 4,928 |
   | `/assets/katex/fonts/KaTeX_Typewriter-Regular.woff2` | 13,568 |

   The 20 WOFF2 files total 259,792 B and match the sizes ADR-0025 records. If the download does not produce these exact byte counts, stop — a wrong or repacked archive was fetched.

   **Not committed** from the archive: the `.woff` and `.ttf` font forms, `katex.css`, `katex.js`, `katex.mjs`, `dist/README.md`, `dist/contrib/**`, `src/**`, `contrib/**`, `cli.js`, `package.json`, and `README.md`. The supported browsers always select the `woff2` source, so the `.woff`/`.ttf` sources inside each `@font-face` block are inert.

5. **Digest manifest.** `src/browser/vendor/MANIFEST.md` holds two Markdown tables with fixed column order, one row per `ASSETS` entry, rows sorted by `path`, `bytes` as decimal integers, `sha256` as 64 lowercase hex characters:
   - vendored: `path | package | version | licence | upstream | bytes | sha256`
   - first-party: `path | bytes | sha256`

   The unit test in `src/browser/assets.rs` parses both tables from `include_str!("vendor/MANIFEST.md")`, asserts the manifest path set equals the `ASSETS` path set exactly (no duplicate, no missing, no extra row), recomputes SHA-256 over every `Asset.bytes`, and compares digest and byte length. It carries a `#[cfg(test)]` FIPS 180-4 SHA-256 in the same module — no digest crate — self-checked against the standard vectors for the empty input and for `abc`.

6. **Package notices.** `packaging/deb/usr/share/doc/papyrus/copyright` keeps its Debian machine-readable format and its `Files: *` GPL-3.0-or-later block, and gains after it:
   - KaTeX code (`katex.min.js`, `katex.min.css`): `License: MIT`, upstream copyright line, full MIT text verbatim from the vendored `LICENSE`.
   - KaTeX fonts (`fonts/*.woff2`): `License: OFL-1.1`, the attribution recorded by the licence review, full SIL OFL 1.1 text verbatim.

   First-party assets stay under `Files: *` and get no extra section. The docs set stays exactly one file and `scripts/build-deb.sh` is not modified. The OFL section cannot cite the archive: 0.16.11 ships only MIT, so the OFL text and the font attribution come from the licence review (Open question 1).

7. **Tests.** New, in `tests/browser_cli.rs` (next to the existing asset tests):
   - Table routing: `GET`/`HEAD` on all 23 paths → `200` with the exact `Content-Type`, `Content-Length`, `Cache-Control: no-store`, and a body equal to the vendored file (fonts: equal length and non-empty); `POST`/`PUT`/`DELETE`/`OPTIONS` on a table path → `405`; `GET`/`HEAD`/`POST` on `/assets/nope`, `/assets/`, and `/assets/app.css/` → `404`; `/assets/katex/katex.min.js?x=1` → `200`.
   - Table-wide offline scan (the assertions the six existing scans do not make):
     - every `url(...)` value in every served stylesheet is a relative `fonts/<name>.<ext>` reference — no scheme, no leading `/`, no `data:`, no `..`, exactly one path segment after `fonts/`;
     - every such `<name>.<ext>` has `<ext>` in {`woff2`, `woff`, `ttf`} and a `/assets/katex/fonts/<name>.woff2` entry in `ASSETS`;
     - no served stylesheet contains `@import`;
     - no `ASSETS` path ends with `.map`, and no served text asset contains `sourceMappingURL`;
     - no served page contains `http://`, `https://`, or `//cdn`, and every `<link rel="stylesheet" href=…>` and `<script src=…>` value starts with `/assets/`;
     - no asset path contains `..`.
   - A source-level guard that `src/browser/assets.rs` contains no `std::fs`, `File::open`, `Path`, or `PathBuf`.

   Existing tests that must **not** be weakened or deleted: `tests/browser_cli.rs::t24_assets_are_offline_and_embedded`, `t40l_offline_asset_scan`, `t25l_offline_asset_scan`, `t26k_offline_asset_scan`, `t57k_offline_asset_scan`, `t81l_offline_asset_scan`. They only read `/assets/app.css` and `/assets/app.js`, whose content and MIME types do not change, so all six keep passing unchanged. Do **not** extend their `!contains("url(")` / `!contains("http://")` loops to the KaTeX assets: `katex.min.js` legitimately contains the XML namespace identifiers `http://www.w3.org/1998/Math/MathML` and `http://www.w3.org/2000/svg`, which are names, never fetched. ADR-0027 decision 10 asks these tests to "gain a sibling assertion"; that assertion is the table-wide scan above, written once instead of six times. Also unchanged and expected to pass: `unknown_path_is_404_for_every_method`, `head_matches_get_without_body`, `t28_accessibility_and_compatibility_basics`, `t25m_no_new_dependencies`, `t26l_no_new_dependencies`, and the asset table in `tests/release_document_types_smoke.rs`.

## Out of scope

- The math/diagram pre-pass, conditional asset emission, and CSP changes (TASK-145, TASK-146).
- The release binary-size assertion, packaged verification, and installed-binary checks (TASK-149).
- Revising browser-UI and packaging-verification pages that still say "no web font" or "no `url(`", the CSP-per-response-class wording, and spec/ADR index regeneration (TASK-148).
- Any Rust crate, npm build, asset fetched at build time, asset loader, or runtime asset directory.
- Any `ETag`, compression, range request, long-lived cache, or `Accept-Encoding` negotiation.
- Adding Mermaid paths to `ASSETS`, the manifest, or the notices.

## Assumptions

- Assumption: ADR-0027 decisions 1–10 and ADR-0025 decision 4 are the contract; where this ticket is more specific it only pins what those decisions already imply.
- Assumption: the KaTeX licence review reaches the outcome ADR-0027 already records — code MIT, fonts SIL OFL-1.1, both compatible with GPL-3.0-or-later. If it does not, ADR-0025's recorded fallback (MathJax with SVG output) applies and this task is re-planned before merge.
- Assumption: only the WOFF2 form is vendored and served, per ADR-0025; the `.woff`/`.ttf` sources in the stylesheet stay byte-identical and are never requested by the supported browsers. This is why the table-wide `url(` rule is "resolves via its `.woff2` sibling" rather than "every literal `url(` has its own entry".
- Assumption: the 12 MiB binary budget is asserted by TASK-149, not here; this task adds ≈558 KB of assets.
- Assumption: `cargo build` stays the whole build and the checkout builds offline once the vendor files are committed.

## Open questions

1. **Font attribution and OFL text source.** KaTeX 0.16.11 ships no OFL text and no per-font copyright header, so the `Copyright:` line for `fonts/*.woff2` and the exact OFL-1.1 text must come from the licence review, not from the archive. Blocking for the notice section (AC #6) before merge, not for starting: the vendor files and the table can land first. Owner: the licence review recorded under ADR-0027 decision 5. Until it resolves, the fonts section ships with `License: OFL-1.1` and full text; no holder line is invented.
2. **Manifest coverage of first-party assets.** ADR-0027 decision 4 says the manifest lists "per asset" package/version/licence/upstream, which first-party `app.css`/`app.js` have no upstream package for. This ticket adds a second `path | bytes | sha256` table so every embedded byte is digest-covered; confirm that reading is preferred over excluding first-party files from the manifest.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 A static asset table with exact-match lookup serves every embedded asset: a hit with GET or HEAD returns 200 with that asset's MIME type and exact bytes, HEAD sends Content-Length and no body, a miss returns 404 for every method, and a hit with any other method returns 405.
- [x] #2 Binary responses are supported with exactly text/css; charset=utf-8, text/javascript; charset=utf-8, and font/woff2, and nothing on the asset path reads the filesystem or joins a request path to a filesystem path.
- [x] #3 KaTeX 0.16.11 katex.min.js, katex.min.css, and exactly the 20 .woff2 font files are committed byte-identical under src/browser/vendor/katex/0.16.11/ with LICENSE (MIT) and UPSTREAM.md recording package, version, download URL, in-archive paths, archive SHA-256, retrieval date, and that no file was modified; no .woff, .ttf, or .map file is committed.
- [x] #4 src/browser/vendor/MANIFEST.md records path, byte size, and SHA-256 for every ASSETS entry plus package, version, licence, and upstream URL for vendored entries, and a unit test recomputes every digest and fails on a digest mismatch, a byte-count mismatch, a missing row, or an extra row.
- [x] #5 /assets/app.css and /assets/app.js are served from the table with unchanged paths, bytes, and MIME types, and cargo build remains the whole build: Cargo.toml gains no dependency, no npm or asset build step is added, and the checkout still builds offline.
- [x] #6 The package copyright file keeps its Papyrus GPL-3.0-or-later block and gains one section per bundled asset family with its licence identifier and full licence text, the docs set stays exactly one file, and scripts/build-deb.sh is unchanged.
- [x] #7 The existing offline asset scans are kept rather than deleted, and a new table-wide test asserts that every url() in every served stylesheet is a relative fonts/<name>.<ext> reference whose <name>.woff2 sibling is an asset-table entry, that no served stylesheet contains @import, that no served page references a stylesheet or script outside /assets/, and that no .map file is embedded and no served text asset contains sourceMappingURL.
- [x] #8 Cache-Control: no-store is retained for every response including assets, and no ETag, compression, range request, or asset-specific cache header is added.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 `src/browser/assets.rs` exists with `Asset`, `ASSETS` (23 entries), and `lookup`; `mod.rs` has no `APP_CSS`/`APP_JS` and no hard-coded asset path.
- [x] #2 All 23 asset paths return 200 with the expected `Content-Type` and body; HEAD matches GET headers with no body; a miss is 404 for every method; a wrong method is 405 with the existing `Allow` header.
- [x] #3 `Response::bytes` is implemented and unit-tested with a non-UTF-8 body.
- [x] #4 KaTeX 0.16.11 is vendored byte-identically under `src/browser/vendor/katex/0.16.11/` with `LICENSE` and `UPSTREAM.md`; no `.woff`/`.ttf`, no `.map`, no `katex.js`/`katex.css`/`katex.mjs`, no archive leftovers.
- [x] #5 `MANIFEST.md` covers every `ASSETS` entry and the digest test passes; mutating one byte or one byte count of a vendored file makes it fail (verified once by hand, then reverted).
- [x] #6 The new table-wide offline assertions pass, and the six existing offline scans plus `unknown_path_is_404_for_every_method`, `head_matches_get_without_body`, and `t28_accessibility_and_compatibility_basics` pass unchanged.
- [x] #7 The package copyright file carries the `Files: *` block plus one section per bundled asset family with a licence identifier and full text; `packaging/deb/usr/share/doc/papyrus/` still holds exactly one file and `scripts/build-deb.sh` is byte-identical to its pre-task content.
- [x] #8 `Cargo.toml` is unchanged, no crate or npm step is added, and `cargo build --offline` succeeds from the checkout.
- [x] #9 `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` all pass.
- [x] #10 No file under `docs/requirements/`, `docs/business-rules/`, `docs/architecture/adr/`, or `scripts/` was modified by this task.
<!-- DOD:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. **Fetch and verify the archive.** Download `https://registry.npmjs.org/katex/-/katex-0.16.11.tgz` into `./.tmp/` (never into the repo tree), confirm SHA-256 `8c5494c5…4f3e24193f`, and confirm `katex.min.js` = 275,414 B, `katex.min.css` = 23,335 B, and the 20 `.woff2` files total 259,792 B. Stop if any byte count differs.
2. **Vendor the files.** Copy `dist/katex.min.js`, `dist/katex.min.css`, and `dist/fonts/*.woff2` into `src/browser/vendor/katex/0.16.11/` (flattened, `fonts/` subdirectory), plus the archive's `LICENSE`. Do not copy the `.woff`/`.ttf` forms, `katex.js`/`katex.css`/`katex.mjs`, `dist/contrib/**`, `src/**`, `README.md`, or `package.json`. Verify each copied file with `cmp` against the extracted archive.
3. **Write `src/browser/vendor/katex/0.16.11/UPSTREAM.md`** with the fields from the ticket: package, version, download URL, per-file archive paths, archive SHA-256, retrieval date, and the modification statement.
4. **Write `src/browser/vendor/MANIFEST.md`** with the two tables, rows sorted by `path`, decimal `bytes`, lowercase 64-hex `sha256`, generated from the committed files (`sha256sum`) — never transcribed from the archive.
5. **Add `Response::bytes`** in `src/browser/http.rs` and unit-test it alongside the existing `Response` tests (status, content type, byte-exact body including a 0x00/0xFF payload).
6. **Create `src/browser/assets.rs`** with `Asset`, `ASSETS` (23 entries), and `lookup`; add `pub mod assets;` to `src/browser/mod.rs`. Bytes via `include_bytes!`, except `app.css`/`app.js` which may keep `include_str!(...).as_bytes()`.
7. **Rewire the router.** In `route()`, replace the two `if path == "/assets/…"` blocks with the single `/assets/`-gated table branch from the ticket; delete `APP_CSS`/`APP_JS` from `mod.rs`.
8. **Add the digest test** in `assets.rs::tests`: a `#[cfg(test)]` SHA-256 with the two standard-vector self-checks, the manifest parser, and the set/bijection/digest/length assertions.
9. **Write the package notices** in `packaging/deb/usr/share/doc/papyrus/copyright`: keep `Files: *` and `License: GPL-3.0-or-later`, append the two KaTeX sections with full MIT and full OFL-1.1 text. Leave `scripts/build-deb.sh` untouched.
10. **Add the new integration tests** in `tests/browser_cli.rs`: table routing, table-wide offline scan, and the `assets.rs` filesystem-free source guard.
11. **Confirm the six existing offline scans still pass unchanged**, then run the gates: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, and one offline build check (`cargo build --offline`).
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
src/browser/assets.rs serves 24 embedded assets from one exact-match table (2 first-party + KaTeX 0.16.11 katex.min.js, katex.min.css, 20 woff2); Response::bytes added; vendor tree carries upstream bytes, MIT LICENSE, fonts/OFL.txt (official SIL OFL 1.1, retrieved 2026-11-15) and UPSTREAM.md; MANIFEST.md digests are recomputed by a test; the deb copyright gains MIT and OFL-1.1 sections. Quality review: pass, with byte-identity verified by cmp against the real npm tarball, live HTTP GET/HEAD/404/405 checks, digest-failure mutation proofs, cargo build --offline (debug and release), 920 tests pass. Release binary 4,730,296 B (about 4.51 MiB of the 12 MiB budget).
<!-- SECTION:NOTES:END -->

## Comments

<!-- COMMENTS:BEGIN -->
author: @task-refiner
created: 2026-11-15T18:40:00Z

---

Refinement 2026-11-15 (TASK-143): contract pinned; no requirements or business rules changed.

Specification change (one file, one added section):
- `docs/specifications/rust-cli-and-debian-package-baseline.md` gained "Bundled browser assets and third-party notices (TASK-143)". It holds the asset table API and routing semantics, the MIME set, `Response::bytes`, the served path list with byte sizes, the vendor layout and `UPSTREAM.md` fields, the `MANIFEST.md` format plus the digest test, the notice structure, and the traceability row. It states that it narrows the earlier asset and licence sentences in that same page and that it does not revise UI wording.
- `docs/specifications/browser-shell-shared-list-layout-and-page-width.md` was deliberately not touched (@architect is editing it for TASK-141).

Handoff to TASK-148 (`@business-analyst`) — this task changed no page-level wording, so all of the following still needs revision:
1. `docs/specifications/browser-requirements.md`: the route table rows for `/assets/app.css` and `/assets/app.js` (lines ~204-205), §6.1 ("CSS and JS are … embedded with `include_str!`"), §6.5 ("no web fonts, icon fonts … no `url(...)`"), and the T-27 row in the test table (line ~517).
2. `docs/packaging.md`: §"Packaging" expected contents (lines ~95-99) and §3.6 (lines ~261-267) still say the package has no asset/template/doc file beyond the copyright file and describe only `/assets/app.css` + `/assets/app.js`.
3. Packaging verification pages that assert a two-asset, notice-free package: `backlog-discovery-…`, `cli-…`, `complete-content-management-…`, `config-defined-document-types-…`, `decision-records-…`, `glossary-terms-…`, `requirements-management-…`, `search-browser-…`, `technical-specifications-…`, and `use-cases-and-test-documentation-milestone-packaging-and-verification.md`.
4. `docs/specifications/browser-managed-artifacts-and-lifecycles.md` §6 item 3 ("loads no external … font").
5. `docs/specifications/browser-agent-skills-and-workflows.md` and `browser-config-defined-document-types.md` where they describe the served asset set.

Sentence TASK-148 may need in the shell spec (not written here): "Bundled stylesheets, scripts, and fonts are served from the single embedded asset table (`/assets/…`) under `Cache-Control: no-store`; no asset is read from disk or requested from another origin."

Deferred question for TASK-148/owner: the ticket's Open question 2 asks whether `MANIFEST.md` should also digest-cover the first-party `app.css`/`app.js` (this ticket says yes, via a second table) — ADR-0027 decision 4 only names package/version/licence/upstream fields.

---

author: orchestrator
created: 2026-11-15T19:02:00Z

---

Quality review: pass on all 8 AC and 10 DoD. Recorded corrections: (1) the ticket's '23 entries' is an arithmetic error in the ticket text; the real served set is 24 (2 first-party + 2 KaTeX code/licence files counted separately + 20 woff2) and code, MANIFEST.md, and the spec agree at 24. (2) fonts/OFL.txt is a justified addition: KaTeX 0.16.11 ships no font licence, so the official SIL OFL 1.1 text was retrieved from openfontlicense.org on 2026-11-15 and its provenance is recorded in UPSTREAM.md. (3) The normative spec section lives in docs/specifications/rust-cli-and-debian-package-baseline.md and is committed with the follow-up docs commit.

---
<!-- COMMENTS:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Browser assets now ship through one compile-time table instead of two hard-coded paths: every entry is served by exact match with its own MIME type, HEAD sends Content-Length without a body, misses are 404 for every method, and wrong methods are 405 with an Allow header, with no filesystem, join, prefix, or percent-decoding logic in the asset module. KaTeX 0.16.11 is vendored unmodified with its MIT licence and the SIL OFL 1.1 font licence, recorded in UPSTREAM.md and in a digest manifest whose SHA-256 values a test recomputes. The package copyright file gains one licence section per bundled asset, the existing offline url()/@import/remote-origin scans keep working, and a new table-wide audit covers every served stylesheet.
<!-- SECTION:FINAL_SUMMARY:END -->
