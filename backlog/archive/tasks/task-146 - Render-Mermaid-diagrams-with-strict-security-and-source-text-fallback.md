---
id: TASK-146
title: Render Mermaid diagrams with strict security and source-text fallback
status: Done
assignee:
  - '@rust-dev'
created_date: '2026-11-15T16:13:00Z'
updated_date: '2026-11-16T12:22:00Z'
labels:
  - browser
  - rendering
  - security
milestone: m-21
dependencies:
  - TASK-143
  - TASK-145
references:
  - docs/requirements/business/REQ-06.017.md
  - docs/architecture/adr/ADR-0026-mermaid-only-offline-diagram-rendering.md
  - >-
    docs/architecture/adr/ADR-0027-vendored-embedded-browser-assets-and-packaging-notices.md
  - src/browser/render.rs
  - src/browser/http.rs
  - docs/specifications/browser-math-and-diagram-rendering.md
  - src/browser/math.rs
  - src/browser/assets.rs
  - src/browser/app.js
  - packaging/deb/usr/share/doc/papyrus/copyright
priority: high
ordinal: 41590
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Gate — cleared 2026-11-16.** ADR-0026 completed acceptance as a delegated architecture decision and every open question is resolved (version, type set, `mmd`, caps, CSP scope, iframe fallback). This ticket is execution-ready and must not re-open those decisions. Owner acknowledgement is tracked by the orchestrator, not here.

**Why.** REQ-06.017 requires a fenced `mermaid` block in the curated prose fields of a record view to render as a diagram, fully offline, with bundled assets, without arbitrary script execution, with the source kept as the fallback, and with full-width diagram content; a `plantuml` block is shown as source with a note and never rendered. TASK-145 delivered the shared pre-pass, the page-asset slot, and the per-element failure marker for math. This ticket adds diagrams to that same path.

**What.** A Mermaid-only diagram segment on the existing `src/browser/math.rs` pre-pass, a server-rendered figure whose `<pre>` is both fallback and client input, two hard input caps, a conditional vendored bundle link with a second CSP literal for diagram pages only, and one client block in `app.js` with strict security and per-figure isolation. PlantUML becomes a static source block with a note.

**Pinned contract (ADR-0026 as amended — implement exactly, do not re-litigate).**

- **Fence set.** Only the info string `mermaid` (case-sensitive, trailing whitespace trimmed, zero to three leading spaces, at least three backticks or tildes, same closer rule as the math fence). `mmd` is dropped and stays literal text. `plantuml` and `puml` become a static source figure with a visible note. `uml`, `dot`, `graphviz`, and every other info string stay literal text with their fence lines, never an error. An empty or whitespace-only `mermaid` or `plantuml`/`puml` fence stays literal text. An unclosed fence stays literal text.
- **Figure markup.** Exactly these three strings, no separators, no added attributes, source escaped once with newlines preserved:
  - renderable: `<figure class="diagram bleed" data-diagram="mermaid"><div class="diagram-scroll"><pre>SRC</pre></div></figure>`
  - over the caps: `<figure class="diagram bleed" data-diagram="mermaid" data-render-error="diagram"><div class="diagram-scroll"><pre>SRC</pre></div><figcaption class="render-note">Diagram not rendered: source exceeds the 100,000-byte or 1,000-line limit.</figcaption></figure>`
  - PlantUML: `<figure class="diagram diagram--unsupported" data-diagram="plantuml"><pre>SRC</pre><figcaption class="render-note">PlantUML diagrams are not supported in this release; showing source.</figcaption></figure>`
  A figure is a block segment: it flushes the open paragraph, is never nested inside `<p>`, and a surrounding paragraph is split around it. The PlantUML figure carries neither `bleed` nor `.diagram-scroll` (it is a source listing, not a visualization).
- **Source extraction.** Content between the fences with one trailing `\r\n`/`\n` removed, then trimmed of leading and trailing whitespace. That trimmed text is the diagram source, the `<pre>` body, and the client input.
- **Caps.** Per fence, on the trimmed source: `MAX_DIAGRAM_BYTES = 100_000` (`source.len()`, UTF-8) and `MAX_DIAGRAM_LINES = 1_000` (`'\n'` count + 1 for a non-empty source). Over-limit means strictly greater than a cap; equality is allowed. The server counts bytes and lines only: it never parses Mermaid, never counts edges, never truncates, and never rejects the record. `maxEdges: 500` is client-side only. The caps are fixed constants with no configuration key.
- **`has_diagram`.** `math::Prose` gains `has_diagram`, true only for a renderable `mermaid` figure under both caps. A page ORs it across its curated prose fields. It alone selects the bundle link and the diagram CSP; an over-limit-only page and a PlantUML-only page get neither.
- **Vendored asset.** `src/browser/vendor/mermaid/11.17.2/mermaid.min.js` — 3,572,661 bytes, SHA-256 `581ed7d74bd9048d0e3a91363927d72ef22942d7722546b27f7cc29e35390eb8`, MIT — plus `LICENSE` and `UPSTREAM.md`, one sorted `MANIFEST.md` row (`/assets/mermaid/mermaid.min.js`, mermaid, 11.17.2, MIT, `https://registry.npmjs.org/mermaid/-/mermaid-11.17.2.tgz`, 3572661, digest), one `assets::ASSETS` entry (`text/javascript; charset=utf-8`), and one MIT section in `packaging/deb/usr/share/doc/papyrus/copyright` with its own short name (the existing `MIT` paragraph embeds the KaTeX copyright). No `.map`, no CSS, no second file, no patch.
- **Head emission.** `math::MERMAID_ASSETS = "<script src=\"/assets/mermaid/mermaid.min.js\" defer></script>"`, appended after the KaTeX fragment with one newline when both apply. Document order: KaTeX CSS, KaTeX script, Mermaid script, `app.js`. At most once per page; only on pages whose pre-pass found a renderable figure.
- **Two CSP literals.** Strict (every HTML response without a renderable diagram, including math-only, lists, board, edit and error pages): `default-src 'none'; style-src 'self'; script-src 'self'; img-src 'self'; font-src 'self'; form-action 'self'; base-uri 'none'; frame-ancestors 'none'`. Diagram (only an HTML response whose pre-pass set `has_diagram`): the same string with `style-src 'self' 'unsafe-inline'`. They differ in exactly one token. `script-src 'self'` never changes and never gains `'unsafe-inline'`, `'unsafe-eval'`, a nonce, or a hash. Asset responses carry no CSP header.
- **Client.** `securityLevel: 'strict'`, `htmlLabels: false`, `theme: 'neutral'`, `fontFamily: 'system-ui, sans-serif'`, `flowchart: { htmlLabels: false }`, `startOnLoad: false`, `maxTextSize: 100000`, `maxEdges: 500`; no other option. `mermaid.render(id, source)` with a deterministic, page-unique id sanitised to `[A-Za-z0-9_-]`. No `bindFunctions`, no click or link binding, no `fetch`, no dynamic `import()`, no `MutationObserver`, no second pass, no storage write. On success replace the `<pre>` with the returned SVG inside `.diagram-scroll`. On a throw or a rejected promise keep the `<pre>`, set `data-render-error="diagram"`, and append `Diagram not rendered: Mermaid rejected the source.` A missing `window.mermaid` marks every figure and appends `Diagram not rendered: the diagram script did not load.` without removing content. Each figure is isolated; a figure already carrying `data-render-error` is skipped; a figure never receives a second `figcaption.render-note`.
- **Promised types.** `flowchart` (incl. `graph`), `sequence`, `class`, `state`, `ER`, `gantt`, `pie`, `journey`, `gitGraph` — fixture-tested. Other parseable types are best-effort and not promised. Unsupported or unparseable types (including `flowchart-elk`, whose layout is not vendored) take the failure path.

**In scope — file-level changes.**

- `src/browser/math.rs`: extend the single scanner (no second scanner). Add `has_diagram` to `Prose`; extend the internal `Fence` classification with diagram and unsupported-source variants; add the two cap constants, the mermaid/plantuml figure builders, `MERMAID_ASSETS`, and unit tests. `HEAD_ASSETS` stays byte-identical.
- `src/browser/render.rs`: OR `has_diagram` at the same call sites that already OR `has_math`; build the page asset fragment from `has_math` and `has_diagram`; make `view_page` and `document_detail_page` report `has_diagram` to the router (a small return struct, e.g. `RecordPage { html, has_diagram }`). Only those two call sites pass the fragment.
- `src/browser/http.rs`: add `CSP_DIAGRAM`; give `Response` a `csp` field defaulting to `CSP` plus a diagram-page constructor; `write_response` prints `response.csp` for `text/html` only.
- `src/browser/mod.rs`: the record-view route picks the diagram constructor when the renderer reports `has_diagram`; every other route is unchanged.
- `src/browser/app.js`: one appended ES5 block per the client contract, after the math block, adding no global.
- `src/browser/app.css`: `.diagram`, `.diagram-scroll`, `.diagram--unsupported`, `.render-note` per spec §19; reuse `.bleed`; no rule for Mermaid-internal classes and no `max-width` on a rendered SVG.
- `src/browser/assets.rs`, `src/browser/vendor/MANIFEST.md`, `src/browser/vendor/mermaid/11.17.2/{mermaid.min.js,LICENSE,UPSTREAM.md}`, `packaging/deb/usr/share/doc/papyrus/copyright`.
- `docs/specifications/browser-math-and-diagram-rendering.md` §13–§21 (already updated with this refinement).
- Tests: `src/browser/math.rs` unit tests, `tests/browser_diagram_cli.rs` (new), plus any existing assertion that the two CSP literals or the asset-reference scans require.

**Out of scope.** PlantUML rendering, a JVM or `plantuml.jar`, a rendering service, any network call, a second client runtime, server-side Mermaid parsing or edge counting, a record-body view, diagrams outside curated prose fields, rendering on edit pages, list cells, metadata, titles, or acceptance-criteria lists, configurable caps, `mmd`, Mermaid 12.x, a sandboxed-iframe fallback, and any change to `script-src`.

**Assumptions.**

- TASK-145's `math.rs` scanner, `Prose`, `HEAD_ASSETS`, and `shell::Page.assets` slot are final and are extended, not replaced.
- The Mermaid 11.17.2 bundle is a classic IIFE that assigns `globalThis.mermaid` and `render()` returns a promise resolving `{ svg, bindFunctions }`; the client handles both a synchronous throw and a rejection.
- The figure markup carries no record id, so the client derives its render id from `window.location.pathname` plus the figure index; the path contains the record id for `/doc/<ID>`. Recorded as a resolved reading in spec §21.
- Mermaid's injected `<style>` inside the SVG is the only reason the diagram class needs `style-src 'unsafe-inline'`; the Firefox probe is recorded in ADR-0026, so no new probe is required by this ticket.
- `no-store` is unchanged, so a diagram page re-sends the 3.4 MiB bundle on each navigation; accepted in ADR-0026/ADR-0027.

**Dependencies.** TASK-143 (asset table, manifest, notices — completed) and TASK-145 (pre-pass and asset slot — implemented). TASK-149 measures packaging without the bundle; TASK-150 verifies the nine types, hostile fixtures, CSP classes, and final binary size. No new dependency is added to `Cargo.toml`.

**Test plan (all automated; browser-behaviour items run in TASK-150).**

1. `math.rs` unit — classification: `mermaid`→renderable figure, `plantuml`/`puml`→unsupported figure with note, `mmd`/`uml`/`dot`/`graphviz`/`rust`/`Mermaid`/`mermaid,`/`mermaid title`→opaque literal, empty and unclosed fences→literal, four-space indent is not a fence.
2. `math.rs` unit — exact markup strings for the three shapes, escaping of `& < > " '` in the `<pre>`, newlines preserved, one escape only.
3. `math.rs` unit — block behaviour: figure flushes the paragraph, no `<p>` wrapping, source order across two figures, diagram-free field byte-identical to today.
4. `math.rs` unit — caps: exactly 100,000 bytes and exactly 1,000 lines render; one byte over and one line over take the over-limit figure with the limit note and `has_diagram == false`; a second renderable figure on the same page still sets `has_diagram`.
5. `math.rs` unit — `has_diagram` false for PlantUML, over-limit, literal fences, and math-only input; `HEAD_ASSETS` unchanged and `MERMAID_ASSETS` references only `/assets/mermaid/mermaid.min.js`.
6. `assets.rs` unit — existing manifest test passes with the new row (row count equals `ASSETS.len()`, byte size 3,572,661, digest `581ed7d7…`).
7. `tests/browser_diagram_cli.rs` — one fixture per promised type (flowchart, graph, sequence, class, state, ER, gantt, pie, journey, gitGraph) served with `data-diagram="mermaid"` and its source intact in `<pre>`.
8. `tests/browser_diagram_cli.rs` — `plantuml` and `puml` fixtures emit the unsupported figure with the fixed note, no `bleed`, no bundle link, and strict CSP; `uml`/`dot`/`graphviz`/`mmd` fixtures stay literal with fence lines and produce no figure.
9. `tests/browser_diagram_cli.rs` — unknown-type fixture (a type the pinned bundle cannot parse, e.g. `flowchart-elk`) is served as a figure with its source and the client asserts the failure path in TASK-150; a malformed source is never rejected by the server.
10. `tests/browser_diagram_cli.rs` — over-byte and over-line fixtures: full source present, `data-render-error="diagram"`, limit note, no bundle link, strict CSP; per-figure isolation: one bad and one good figure on one page both emitted, only the bad one marked.
11. `tests/browser_diagram_cli.rs` — client source assertions: every option of the pinned object, `securityLevel: "strict"`, `htmlLabels: false`, `maxEdges: 500`, `maxTextSize: 100000`, render-id sanitisation charset, skip rule for `data-render-error`, no `bindFunctions`, no `fetch(`, no `import(`, no `MutationObserver`, note texts.
12. `tests/browser_diagram_cli.rs` — missing `window.mermaid` branch exists and removes no content (source assertion) and is exercised in a browser in TASK-150.
13. `tests/browser_diagram_cli.rs` — hostile fixtures (`<img src=x onerror=…>`, `classDef` payload, `javascript:` link) served with no `<script` node, no `on…=` attribute, and no `javascript:` URL introduced by record data.
14. `tests/browser_diagram_cli.rs` — both CSP literals: diagram class asserted byte-exactly for the diagram record view; strict class asserted byte-exactly for a math-only view, a plain view, a list, the board, an edit page, and an error page; no CSP header on `/assets/mermaid/mermaid.min.js`; no `unsafe-inline` inside any `script-src` (parse the header string).
15. `tests/browser_diagram_cli.rs` — bundle link appears exactly once on a diagram page and nowhere on math-only, over-limit-only, PlantUML-only, list, board, edit, or error pages; script order KaTeX CSS → KaTeX JS → Mermaid JS → `app.js`; no inline `<script>` element.
16. `tests/browser_diagram_cli.rs` — CLI/search/stored bytes: `papyrus <slug> view`, `--json`, and `search` show the fence source unchanged; the record file's bytes are identical before and after a browser GET.
17. `tests/browser_diagram_cli.rs` — source scan: no JVM or PlantUML invocation anywhere in `src/` or `Cargo.toml` (no `plantuml.jar`, no `java` command, no `Command::new("java")`, no `plantuml` dependency), and `Cargo.toml` dependency set unchanged.
18. `tests/browser_cli.rs` — the existing served-asset reference scans pass with the new asset; `app.css` keeps no `url(` outside the KaTeX font allowlist.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The implementation conforms to ADR-0026 as amended with no re-litigation: only the info string `mermaid` is recognised (`mmd` stays literal), PlantUML is a source block with a note, the caps are 100,000 bytes and 1,000 lines server-side with `maxEdges: 500` client-side, the CSP is the strict literal except on diagram pages, and no server-side Mermaid parsing exists.
- [x] #2 A `mermaid` fence in a curated prose field renders as the figure of spec §14.1 including its focusable labelled scroll container (`role="region"`, a page-unique `aria-label`, `tabindex="0"`), a page whose pre-pass found a renderable figure links `/assets/mermaid/mermaid.min.js` exactly once after the KaTeX fragment and before `app.js`, and every other page class links it zero times.
- [x] #3 `plantuml` and `puml` fences render as the static source figure with the fixed note, no `bleed`, no bundle link and the strict CSP; `uml`, `dot`, `graphviz`, `mmd`, and every other info string stay literal text with their fence lines and never produce a figure or an error; empty and unclosed fences stay literal.
- [x] #4 The two caps are enforced in the pre-pass on the trimmed source with equality allowed: an over-limit fence renders the full source in `<pre>` with `data-render-error="diagram"` and the 100,000-byte/1,000-line note, the bundle is not linked, and the page keeps the strict CSP; the server counts bytes and lines only and never counts edges or parses Mermaid.
- [x] #5 `app.js` initialises Mermaid with exactly `startOnLoad: false`, `securityLevel: 'strict'`, `htmlLabels: false`, `theme: 'neutral'`, `fontFamily: 'system-ui, sans-serif'`, `flowchart: { htmlLabels: false }`, `maxTextSize: 100000`, `maxEdges: 500`, uses a deterministic `[A-Za-z0-9_-]` render id, never calls `bindFunctions`, and a client-source test asserts every one of those facts.
- [x] #6 A render failure (thrown error or rejected promise) keeps the `<pre>`, sets `data-render-error="diagram"`, appends exactly one `figcaption.render-note` line, and leaves every other figure and the page unaffected; a missing `window.mermaid` marks every figure with the script-note and removes no content.
- [x] #7 The diagram figure is a full-width `bleed` region whose `.diagram-scroll` container keeps `overflow-x: auto` as a safety net; the rendered SVG is scaled to fit by Mermaid's own inline `max-width`, so a very wide diagram fits the container instead of forcing a scroll, and the page gains no horizontal scrollbar at 320-1920 CSS px.
- [x] #8 Hostile fixtures (`<img src=x onerror=…>`, a `classDef` payload, a `javascript:` link) leave the served HTML with no `<script` node, no `on…=` event-handler attribute, and no `javascript:` URL introduced by record data; `unsafe-inline` never appears in any `script-src`; both CSP literals are asserted byte-exactly per response class with no CSP header on any asset response; and no rendered diagram exposes a navigable link, because anchors are neutralised after render.
- [x] #9 The vendored bundle is `vendor/mermaid/11.17.2/mermaid.min.js` at 3,572,661 bytes with the recorded SHA-256, accompanied by `LICENSE` and `UPSTREAM.md`, covered by a sorted `MANIFEST.md` row that the existing digest test verifies, and noticed by an MIT section in the package `copyright`; no JVM, `plantuml.jar`, PlantUML dependency, or rendering service exists in the codebase, and CLI, `--json`, search, and stored record bytes are unchanged.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Unit tests in `src/browser/math.rs` cover every classification row, the three exact figure strings, source escaping, block splitting, both cap boundaries (exactly at the cap and one over), and both page flags.
- [x] #2 New integration test file `tests/browser_diagram_cli.rs` covers the nine promised types, PlantUML/`puml`/`uml`/`dot`/`graphviz`/`mmd` literal behaviour, unknown-type and malformed input, over-byte and over-line input, per-figure isolation, missing `window.mermaid`, hostile fixtures, both CSP literals per response class, bundle-link placement, CLI/`--json`/search/stored-bytes parity, and the no-JVM/no-PlantUML source scan.
- [x] #3 The existing `assets.rs` manifest digest test passes with the Mermaid row and reports 3,572,661 bytes and the recorded SHA-256; no new digest test is written.
- [x] #4 `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, and the full `cargo test` suite pass.
- [x] #5 `Cargo.toml` is unchanged: no new crate, no build step, no network at build time, and no server-side Mermaid parsing or edge counting.
- [x] #6 `docs/specifications/browser-math-and-diagram-rendering.md` §13–§21 are consistent with the shipped markup, caps, CSP literals, client options, and failure notes; no ADR, requirement, business rule, `docs/specifications/ui/**` file, or shell spec is edited.
- [x] #7 The budget is recorded: projected 8,302,957 bytes (7.92 MiB) against the 12 MiB ceiling with 4,279,955 bytes of headroom, and the measured release binary size is noted for TASK-150; TASK-149 needs no threshold change.
<!-- DOD:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Vendor the asset (no build step). Create `src/browser/vendor/mermaid/11.17.2/`. Obtain `mermaid.min.js` once from `https://registry.npmjs.org/mermaid/-/mermaid-11.17.2.tgz`, then verify it is exactly 3,572,661 bytes with SHA-256 `581ed7d74bd9048d0e3a91363927d72ef22942d7722546b27f7cc29e35390eb8`. Copy the upstream `LICENSE` (MIT) verbatim and write `UPSTREAM.md` (package, version, registry URL, publication date 2026-08-25, retrieval date, licence). If the exact bytes or digest cannot be reproduced, stop and report a blocked vendor step; never substitute another version or patch the file (ADR-0027 decision 3).
2. Register the asset. Add one `Asset` entry to `src/browser/assets.rs` (`/assets/mermaid/mermaid.min.js`, `text/javascript; charset=utf-8`, `include_bytes!`). Add one sorted row to the `## Vendored assets` table in `src/browser/vendor/MANIFEST.md` with package, version, licence, upstream, 3572661, and the digest. Run the asset unit test and confirm the row count, byte count, and digest all match.
3. Package notice. Add a `Files:` stanza for `src/browser/vendor/mermaid/11.17.2/mermaid.min.js` to `packaging/deb/usr/share/doc/papyrus/copyright`, with the Mermaid copyright line and a standalone MIT licence paragraph under a distinct short name (do not reuse the KaTeX `MIT` paragraph, which embeds the Khan Academy copyright). The docs set stays one file.
4. Extend the pre-pass in `src/browser/math.rs` — one scanner, no second pass. Update the module docs. Add `MAX_DIAGRAM_BYTES = 100_000`, `MAX_DIAGRAM_LINES = 1_000`, and `MERMAID_ASSETS`. Extend the internal `Fence` enum with a renderable-diagram variant (source + next index), an over-limit-diagram variant (source + next), and an unsupported-source variant (`plantuml`/`puml`, source + next); classify the info string in `fence_at` after the existing `trim_end`. Build the three figure strings of spec §14 with `render::escape` applied once. Reuse the existing paragraph flush so a figure is a block segment. Add `has_diagram` to `Prose`, set only for a variant under both caps. Leave `HEAD_ASSETS`, the math branches, and every literal-fence path byte-identical.
5. Report the page flag in `src/browser/render.rs`. Introduce `pub struct RecordPage { pub html: String, pub has_diagram: bool }` and change `view_page` and `document_detail_page` to return it, OR-ing `has_diagram` at every call site that already ORs `has_math`. Build the asset fragment locally: `math::HEAD_ASSETS` when `has_math`, then `math::MERMAID_ASSETS` when `has_diagram`, joined by one `\n`, `""` when neither, and pass it to `page_with_assets`. Update the two in-file callers and any test that consumes the old `String` return.
6. Add the second CSP literal in `src/browser/http.rs`. Keep `CSP` byte-identical; add `pub const CSP_DIAGRAM` with `style-src 'self' 'unsafe-inline'` and every other directive unchanged. Give `Response` a `csp: &'static str` field (default `CSP` in every constructor) and a diagram-page constructor; make `write_response` print `response.csp` for `text/html` only, so assets keep no CSP header. Add a unit test that the two literals differ in exactly one token and that `unsafe-inline` is absent from both `script-src` values.
7. Wire the route in `src/browser/mod.rs`. At `view_page`, destructure the `RecordPage` from step 5 and return `Response::html_diagram(200, html)` when `has_diagram`, otherwise `Response::html(200, html)`. Touch no other route: board, lists, inventory, lifecycle, config, skills, cleanup, edits, redirects, and error pages stay on `Response::html`.
8. Add the client block in `src/browser/app.js` after the math block, in the existing ES5 style and inside the existing IIFE (no new global). Implement spec §16.4 verbatim: the exact `mermaid.initialize` object, the skip rule for `figure[data-diagram="mermaid"][data-render-error]`, source from `pre.textContent`, the sanitised page-path render id, per-figure `try`/`catch` plus promise rejection handling, SVG replacement inside `.diagram-scroll` only on success, no `bindFunctions`, the two failure note texts, the missing-`window.mermaid` branch, and the no-duplicate-note rule.
9. Style in `src/browser/app.css`: `.diagram { margin: 12px 0; }`, `.diagram-scroll { overflow-x: auto; }`, a `.render-note` rule (small, muted, block), and any `.diagram--unsupported` rule needed to keep the static block in the text measure. Reuse the existing `.bleed` rule; add no rule for Mermaid-internal classes, no `max-width` on a rendered SVG, and no `url(`. Confirm `.bleed` and `pre` already provide the containment the figure needs, and that the page gains no horizontal scrollbar at 320–1920 CSS px.
10. Tests. Extend `src/browser/math.rs` unit tests (classification, exact markup, escaping, block splitting, cap boundaries, flags, asset constants) and add `tests/browser_diagram_cli.rs` following `tests/browser_math_cli.rs` (temp fixture project copied from `tests/fixtures/characterization/lists/project`, the same `Server` helper, `set_body` fixtures, raw header assertions). Reuse the existing `assets.rs` manifest test rather than writing a new digest test. Update `tests/browser_math_cli.rs`/`tests/browser_cli.rs` only where the new asset or the CSP split changes an assertion.
11. Verify. `cargo fmt --all --check`; `cargo clippy --all-targets -- -D warnings`; `cargo test --test browser_diagram_cli`; `cargo test --test browser_math_cli --test browser_cli --test browser_shell_cli`; `cargo test browser::math browser::assets browser::http`; then the full `cargo test`. Record the release binary size (`cargo build --release`, `stat -c %s`) as evidence for the 12 MiB budget: projected 8,302,957 bytes (7.92 MiB), headroom 4,279,955 bytes, no TASK-149 threshold change.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
The math pre-pass now recognises a mermaid fence and emits <figure class=diagram bleed data-diagram=mermaid><div class=diagram-scroll role=region aria-label='Diagram N' tabindex=0><pre>escaped source</pre></div></figure>; N is continuous across the page's prose fields so the accessible name is unique. plantuml/puml become a static source figure with a note, mmd/uml/dot/graphviz and unknown info strings stay literal. Server caps are 100,000 bytes and 1,000 lines on the trimmed source (equality allowed), over-limit figures keep the full source with data-render-error=diagram and a note, and the server never parses Mermaid. Mermaid 11.17.2 is vendored (3,572,661 B, digest-manifested, MIT notice) and linked once, only on pages with a renderable figure, after KaTeX and before app.js. The client uses the pinned options with maxEdges 500, deterministic render ids, per-figure isolation, source-preserving failure including a missing bundle, and neutralises anchors after render so no diagram can navigate away. Diagram pages get the scoped style-src unsafe-inline literal while every other response keeps the strict one; script-src never relaxes. Review rounds fixed: navigable anchors, a wrong failure-path fixture, scale-to-fit wording, the region semantics, and page-unique names. 1046 tests pass; real Firefox checks cover link neutralisation, unknown type, isolation, missing bundle, focus and no page scrollbar at 320/1920 px. Residual for TASK-150: nine-type rendering, literal/PlantUML rendering in browsers, midpoint widths, and Edge 130/Safari 18/Firefox 100.
<!-- SECTION:NOTES:END -->

## Comments

<!-- COMMENTS:BEGIN -->
author: orchestrator
created: 2026-11-16T08:17:00Z

---

Blocker cleared: ADR-0026 completed acceptance 2026-11-16 as a delegated architecture decision (owner acknowledgement pending). Resolved: Mermaid pinned to 11.17.2 (dist/mermaid.min.js, 3,572,661 B, sha256 581ed7d7...); supported types flowchart/graph, sequence, class, state, ER, gantt, pie, journey, gitGraph with other parseable types best-effort and unsupported types taking the failure path; the mmd alias is dropped (REQ-06.017 names only mermaid); style-src 'self' 'unsafe-inline' is granted only on HTML responses that link the diagram bundle, all other HTML including math-only pages keeps style-src 'self' and script-src stays 'self' everywhere; caps are 100,000 UTF-8 bytes and 1,000 lines in the server pre-pass plus maxEdges 500 client-side, with over-limit input shown as source plus a limit note; the sandboxed-iframe fallback is rejected (opaque-origin cannot load /assets/mermaid.min.js without CORS, allow-same-origin defeats isolation); failure contract is source preserved in pre, data-render-error=diagram, one figcaption render-note, per-figure try/catch isolation. Budget: projected 7.92 MiB against the 12 MiB ceiling, so TASK-149 needs no adjustment.

---

author: @task-refiner
created: 2026-11-16T08:24:00Z

---

Refinement 2026-11-16: ticket rewritten as execution-ready against ADR-0026 as amended. Blocker label removed; gate AC replaced by a conformance AC. Spec `docs/specifications/browser-math-and-diagram-rendering.md` gains Part II (§13–§21): fence set, exact figure strings, caps, vendored tree/manifest/notice, conditional head emission, client contract incl. render-id derivation, the two CSP literals and their response classes, the nine promised types, the four-row failure table, styling, and diagram tests. Render-id reading: the pinned figure markup carries no record id, so the client derives the id from `window.location.pathname` (which contains `/doc/<ID>`) plus the figure index, sanitised to `[A-Za-z0-9_-]` — recorded in spec §21. Caps measured on the trimmed source with equality allowed. PlantUML licence notice uses its own short name because the existing `MIT` paragraph embeds the KaTeX copyright. Notes are appended at most once per figure. Open questions: none. Remaining risk: the vendored file must be obtained at exactly 3,572,661 bytes / the recorded digest; if not, the vendor step blocks rather than substituting a version.

---

author: orchestrator
created: 2026-11-16T09:14:00Z

---

AC7 amended after real-browser measurement: Mermaid 11.17.2 emits every SVG with width=100% and an inline max-width, so a very wide diagram scales to fit its container instead of scrolling. Measured on a 25-node flowchart LR (viewBox 6941 px wide): rendered at 284 px, .diagram-scroll clientWidth == scrollWidth, and the page had no horizontal scrollbar at 320-1920 px. The accepted contract is therefore scale-to-fit with no page-level horizontal scrollbar; .diagram-scroll keeps overflow-x auto only as a safety net, and requiring natural-size SVG or changing useMaxWidth is out of scope (ADR-0026 pins the client options). Also recorded from the same review: click/link directives produced a navigable anchor despite securityLevel strict, which contradicts ADR-0026's no-click-binding intent; the implementation now neutralises anchors after render so no diagram can navigate away, and flowchart-elk is not a failure-path example because Mermaid 11.17.2 renders it (the fixture uses a genuinely unknown type).

---

author: orchestrator
created: 2026-11-16T09:42:00Z

---

AC2 amended for the delivered figure markup: the .diagram-scroll container inside a mermaid figure now carries role=region, a page-unique aria-label and tabindex=0, because the shell contract requires every scrollable block inside a .bleed to be keyboard reachable and named (browser-shell-shared-list-layout-and-page-width.md section 4.4). Spec section 14.1/14.2/14.4/20 updated to match. The PlantUML figure is unchanged and has no scroll container. Real-browser verification confirmed link neutralisation (anchors carry no href/xlink:href/target and clicking does not navigate), the unknown-type failure path, per-figure isolation, the missing-bundle branch, scale-to-fit with no page scrollbar at 320 and 1920 px, and keyboard focus reaching the region with a visible ring.

---

author: @rust-dev
created: 2026-11-16T11:12:00Z

---

DEF-1 (reported by TASK-150, severity High, no fix made): Firefox 100.0 cannot render any Mermaid diagram, so NFR-05.001's Firefox 100 floor is not met.

Expected: on `/doc/REQ-001.001` in Firefox 100.0 at 1440 CSS px (DPR 1), all 10 promised Mermaid figures become `<svg>` inside `.diagram-scroll`, with no `data-render-error`.
Observed: 0/10 SVG; all 10 figures keep `<pre>`, carry `data-render-error="diagram"`, and show `Diagram not rendered: Mermaid rejected the source.` `mermaid.render('probe','flowchart TD\n A --> B')` rejects with `TypeError: Illegal constructor` at `http://127.0.0.1:6431/assets/mermaid/mermaid.min.js:3582:1657` (stack also frames 3584:664 and 3586:1427). Firefox 102.0esr renders 10/10 with the same page and bundle.

Fixture/route: TASK-150 S3 corpus, `/doc/REQ-001.001`; same result on `/doc/NOTE-012` and `/doc/NOTE-015`.
Browser: Firefox 100.0 (`firefox-bin` SHA-256 `7cabcabf4ae5092dd2ec802bacc8b9730b334835ea91718896a50f215ab932ee`), geckodriver 0.36.0, headless, viewport 1440x900, `papyrus.nav` cleared.
Command: `geckodriver --port <p>`; POST `/session` with `moz:firefoxOptions.binary` = Firefox 100 tarball; POST `/url` `http://127.0.0.1:6431/doc/REQ-001.001`; wait 5 s; evaluate the DOM; POST `/execute/async` calling `window.mermaid.render(...)`.
Reproduction: 1) build the TASK-150 S3 corpus and serve the installed binary; 2) launch the Firefox 100 tarball under geckodriver; 3) open `/doc/REQ-001.001` at 1440 px; 4) inspect `figure[data-diagram]` and the render promise rejection.
Evidence: `./.tmp/task150/evidence/def_ff100_diagram.json` during the run (scratch removed; excerpts above and in TASK-150 notes).
Impact: the browser floor in NFR-05.001 (Firefox 100+) is not satisfied for diagram rendering. Possible causes are a Mermaid 11.17.2 API requiring a newer Gecko (the rejection is consistent with a missing/illegal platform constructor) or a floor/requirement mismatch; the owning task decides whether to raise the floor or change the bundle/fallback.

---

author: orchestrator
created: 2026-11-16T11:39:00Z

---

Reopened for DEF-1 found by TASK-150: Mermaid 11.17.2 calls new CSSStyleSheet(), which Firefox 100 lacks, so 0/10 diagrams rendered there. ADR-0026 gained decision 13 (2026-11-16, delegated architecture authority): keep the vendored bundle byte-identical and install a feature-detected constructible-CSSStyleSheet substitute in app.js inside the diagram branch, before mermaid.initialize. Measured: Firefox 100 with the substitute renders 10/10 with a byte-identical 62,476-byte theme stylesheet to Firefox 102 native; Firefox 101/102/153 keep the native constructor and render 10/10. No manifest, digest, notice, CSP or cap change; app.js grows 1,238 B. Verification must re-run the diagram checks on Firefox 100 before this task closes again.

---

author: orchestrator
created: 2026-11-16T12:21:00Z

---

DEF-1 fix review corrections applied. Shipped figures: app.js 12,953 to 13,757 B (+804 B); release binary 8,324,128 B with 4,258,784 B headroom under the 12,582,912 B ceiling (the earlier 1,238 B / 8,324,534 B figures in the DEF-1 reopen comment were pre-implementation projections and are superseded). ADR-0026 decision 13 now records the measured cost, the corpus-dependent CSS rule count (62,476 B/643 rules on the full nine-type corpus; 62,431 B/623 top-level rules on the minimal fixture, with identical cross-browser per-figure stylesheet SHA-256 and SVG node counts on Firefox 100/102/153), and the known limitation that the substitute covers only insertRule/cssRules and that replace/replaceSync/adoptedStyleSheets are absent on Firefox 100. Independent review confirmed: shim installed only on diagram pages, only window.CSSStyleSheet replaced, native prototype preserved, Firefox 100 renders 10/10 with the shim while 102 and 153 keep the native constructor and render 10/10, no inline script, CSP and vendored bytes unchanged, 1048 tests pass, binary within budget.

---
<!-- COMMENTS:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Mermaid diagrams now render offline in the browser from the same pre-pass that handles math: a mermaid fence becomes a figure whose escaped source is visible until the vendored bundle renders it, and the bundle is served only on pages that actually contain a diagram. The client runs with strict security settings, a deterministic render id per figure and per-figure isolation, keeps the source and shows one note when a diagram cannot be rendered, and strips links from rendered diagrams so a diagram can never navigate away. PlantUML fences show their source with a note, unknown fences stay literal, oversized diagrams are refused with the source kept, and the content security policy relaxes only the style source on diagram pages while script execution stays same-origin everywhere.
<!-- SECTION:FINAL_SUMMARY:END -->
