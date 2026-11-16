---
id: ADR-0026
title: "Mermaid-Only Offline Diagram Rendering, PlantUML Out of Scope"
created_date: '2026-11-15T00:00:00Z'
updated_date: '2026-11-16T08:15:58Z'
labels: []
references: []
---
# Mermaid-Only Offline Diagram Rendering, PlantUML Out of Scope

## Status

Accepted (2026-11-15 owner scope decision; mechanics completed 2026-11-16). The 2026-11-15 decision fixed Mermaid-only scope and the PlantUML exclusion but left four questions open and did not pin a Mermaid version; TASK-146 treated the ADR as `Proposed` and stayed blocked. This amendment resolves every open question, pins Mermaid 11.17.2, and records the executed decision. Owner approval was not available in the 2026-11-16 run: the completion was made under delegated architecture authority and is reported to the owner through the orchestrator.

## Date

2026-11-15; amended 2026-11-16.

## Context

REQ-06.017 requires the browser to render diagrams embedded in record content, offline, with bundled assets, in a single binary, under a dependency licence compatible with GPLv3-or-later, without arbitrary script execution, and with graceful degradation. It names "PlantUML- or Mermaid-style diagrams" as examples and deliberately leaves the library choice to implementation.

ADR-0025 settles math with a vendored KaTeX. Diagram rendering is a different problem: the two named ecosystems have incompatible delivery models.

- **Mermaid** is a JavaScript library (MIT). A pre-built classic-script bundle, `mermaid.min.js`, renders into the current document. Version 11.17.2's file is 3,572,661 bytes; version 12.3.0's is 5,537,838 bytes.
- **PlantUML** is a Java program (GPL-3.0). Rendering requires `plantuml.jar` and a JVM (a second runtime, a second process, and ~50 MB of JVM plus jar), or a running PlantUML HTTP service (a network dependency). The available JavaScript package, `plantuml-encoder`, only deflates and encodes source into a URL or image path; it does not render. PlantUML's `!include` directive reads local files, which is a second filesystem surface in a page renderer.

Forces: ADR-0002/ADR-0003 ship one `.deb` containing `/usr/bin/papyrus`; ADR-0005 forbids dependencies, external servers, and remote assets; NFR-01.003 forbids network use; NFR-03.001 fixes a Rust single binary; NFR-02.002 requires sanitisation; NFR-05.001 sets Firefox 100, Edge 130, and Safari 18 as the browser floor; REQ-06.015 lets diagram content use the full window width.

Version choice is part of the decision, not an implementation detail:

- **11.17.2 is the last 11.x release** (published 2026-08-25). It is a classic IIFE bundle that assigns `globalThis.mermaid`, has no `import`/`export`, no dynamic import, and no reachable `eval`/`new Function` (its only `Function(...)` constructor calls are the lodash global fallbacks, short-circuited by `self` in a browser). The advisory batches through 2026-08-06 — configuration prototype pollution, CSS injection into sibling elements, architecture-diagram prototype pollution, and radar/XY/gantt DoS — are all fixed by 11.16.1, so no open advisory affects 11.17.2.
- **12.3.0 is the current `latest`** (published 2026-11-03) but is a breaking release: it declares an ES2024 baseline, a Safari 17.4+ floor, and ELK as the bundled default layout with a new default theme. ES2024 syntax is not supported by Firefox 100, which NFR-05.001 still requires; adopting 12.x would require an NFR-05.001 change, not an implementation choice. It is also 1,965,177 bytes larger and changes how existing diagrams look.

Mermaid's renderer injects a `<style>` element into the SVG it produces, which collides with the current `style-src 'self'` policy. A Firefox 153esr probe against the exact CSP confirmed the consequence: with `style-src 'self'` the SVG renders as an unreadable black box; with `style-src 'self' 'unsafe-inline'` the neutral theme renders correctly. The relaxation is therefore required, not cosmetic.

Mermaid quality also matters: it is JS code executing in the same origin as the write API, so its sanitisation and input limits are part of the security model, not a detail.

The current release binary is 4,730,296 bytes (KaTeX and its fonts already embedded). The projected binary with Mermaid 11.17.2 is 8,302,957 bytes before the small Rust/client additions.

## Decision

1. **Diagram syntax support is Mermaid only.** The accepted fence info string is exactly `mermaid` (case-sensitive, trailing whitespace trimmed, zero to three leading spaces, at least three backticks or tildes, same opener/closer rules as the math fence in `docs/specifications/browser-math-and-diagram-rendering.md` §3.4). The `mmd` alias is **dropped**: REQ-06.017 names only `mermaid`, the alias is an unrequested extension, and it would double the fence-recognition and test surface. An `mmd` fence stays literal text.

   - `plantuml` and `puml` fences are PlantUML fences and render as source with a visible note (decision 2). `uml`, `dot`, `graphviz`, and every other unknown fence remain literal text exactly as today, fence lines included, never an error, never silently dropped.
   - An empty or whitespace-only `mermaid` fence is literal text (fences included), not a figure and not an error.

2. **PlantUML is excluded from the product scope, not deferred silently.** A `plantuml` or `puml` fence renders as a static block:

   ```html
   <figure class="diagram diagram--unsupported" data-diagram="plantuml">
     <pre>escaped source</pre>
     <figcaption class="render-note">PlantUML diagrams are not supported in this release; showing source.</figcaption>
   </figure>
   ```

   The block is server-rendered; it never links the Mermaid bundle and never relaxes the CSP. `dot`, `graphviz`, and `uml` are not PlantUML and get no note.

   Reasons, in order of force: a JVM and `plantuml.jar` cannot be shipped inside a Rust binary (NFR-03.001, ADR-0002); a PlantUML service is a network dependency (NFR-01.003); `plantuml-encoder` does not render; and PlantUML's file-include semantics would add a filesystem surface to the renderer. REQ-06.017's wording is permissive ("for example … PlantUML- or Mermaid-style"), so this is a scope decision, not a requirement gap.

3. **Server-side work is the same delimiter pre-pass as ADR-0025, extended with one fence shape.** A `mermaid` fence becomes:

   ```html
   <figure class="diagram bleed" data-diagram="mermaid">
     <div class="diagram-scroll"><pre>escaped source</pre></div>
   </figure>
   ```

   - The content between the fences is taken with one trailing line break removed and trimmed of leading and trailing whitespace, then escaped with `render::escape`; newlines are preserved. The `<pre>` is both the no-JavaScript fallback and the source of truth for the client. Mermaid receives `pre.textContent`, which restores the escaped characters exactly.
   - A diagram fence is a block segment, like display math: it is never nested inside a `<p>`; a surrounding paragraph is split around it. A field with no diagram keeps byte-identical output.
   - `Prose` gains `has_diagram`, true only when at least one `mermaid` figure is under the input caps and therefore renderable. `has_diagram` controls both the bundle link and the CSP variant; a page with only an over-limit or PlantUML block gets neither.
   - Nothing about Mermaid syntax is parsed on the server. The server counts bytes and lines only; the edge cap is enforced by Mermaid (decision 7).

4. **Pinned asset: Mermaid 11.17.2, one file.**

   - Upstream: `https://registry.npmjs.org/mermaid/-/mermaid-11.17.2.tgz`, published 2026-08-25T11:52:39Z, MIT.
   - Vendored file: `src/browser/vendor/mermaid/11.17.2/mermaid.min.js`, 3,572,661 bytes, SHA-256 `581ed7d74bd9048d0e3a91363927d72ef22942d7722546b27f7cc29e35390eb8`.
   - Served path: `/assets/mermaid/mermaid.min.js`, `Content-Type: text/javascript; charset=utf-8`, through the ADR-0027 asset table with a `MANIFEST.md` row, a `LICENSE`, and an `UPSTREAM.md`. No `.map`, no CSS, no icon pack, no second file is vendored.
   - The file is a classic IIFE script, loaded as `<script src="/assets/mermaid/mermaid.min.js" defer></script>`. No module loading, no dynamic `import()`, no `fetch`. It assigns `globalThis.mermaid`.
   - Load order on a page: KaTeX stylesheet, KaTeX script, Mermaid script, then `app.js`, all deferred, so both globals exist before the client passes run.
   - The bundle is linked only on pages whose pre-pass found at least one renderable `mermaid` figure. `no-store` is unchanged (ADR-0027 decision 8), so a diagram page re-sends the 3.4 MiB bundle on each navigation; accepted for a loopback single-user interface.

5. **Client contract.**

   ```js
   mermaid.initialize({
     startOnLoad: false,
     securityLevel: 'strict',
     htmlLabels: false,
     theme: 'neutral',
     fontFamily: 'system-ui, sans-serif',
     flowchart: { htmlLabels: false },
     maxTextSize: 100000,
     maxEdges: 500
   });
   ```

   - For each `figure[data-diagram="mermaid"]:not([data-render-error])`, read the source from its `<pre>`, then call `mermaid.render(id, source)` with a deterministic, page-unique id derived from the record id and the figure index and sanitised to `[A-Za-z0-9_-]`.
   - On success, replace the `<pre>` with the returned SVG inside `.diagram-scroll`. Never call the returned `bindFunctions`: no click, link, or tooltip binding is installed.
   - **Supported diagram types for this release:** `flowchart` (including `graph`), `sequence`, `class`, `state`, `ER`, `gantt`, `pie`, `journey`, and `gitGraph`. These are the types covered by fixtures and by TASK-150. No allowlist is enforced: another Mermaid type that 11.17.2 can parse is rendered on a best-effort basis and is not a promised capability. A type the pinned bundle cannot detect or parse — including `flowchart-elk`, whose ELK layout package is not vendored — fails through the standard failure contract below.
   - **Failure contract:** on any thrown error, the `<pre>` stays in place, the figure gains `data-render-error="diagram"`, and one `<figcaption class="render-note">` line is appended: `Diagram not rendered: Mermaid rejected the source.` Each figure is rendered in its own `try`/`catch`; one failure does not abort the page or any other figure. If `window.mermaid` is missing (asset blocked or failed), every figure gets `data-render-error="diagram"` and the note `Diagram not rendered: the diagram script did not load.` and no content is removed.
   - No `fetch`, dynamic `import()`, `MutationObserver`, second pass, storage write, or DOM state beyond `data-render-error` is added. No click binding, no `markdownAutoWrap` change, no `layout: 'elk'`.

6. **CSP: `style-src 'self' 'unsafe-inline'` is granted only on diagram pages; `script-src 'self'` is unchanged everywhere.**

   The server emits exactly two HTML CSP strings:

   - Strict (every HTML response without a renderable diagram, including math-only record views, edit pages, lists, the board, and error pages):

     ```
     default-src 'none'; style-src 'self'; script-src 'self'; img-src 'self'; font-src 'self'; form-action 'self'; base-uri 'none'; frame-ancestors 'none'
     ```

   - Diagram (only an HTML response whose pre-pass set `has_diagram`):

     ```
     default-src 'none'; style-src 'self' 'unsafe-inline'; script-src 'self'; img-src 'self'; font-src 'self'; form-action 'self'; base-uri 'none'; frame-ancestors 'none'
     ```

   Asset responses keep no CSP header. The two strings differ in exactly one token; `script-src` never contains `'unsafe-inline'`, `'unsafe-eval'`, a nonce, or a hash.

   Reasoning:

   - `script-src 'self'` remains, so no inline script, `eval`, `Function`, or remote script can execute. Mermaid's only `Function("return this")` is unreachable in a browser (the lodash `self` branch wins), and the Firefox probe confirmed the bundle initialises and renders under this policy.
   - `default-src 'none'` remains, so the rendered SVG cannot fetch anything. `img-src 'self'` and `font-src 'self'` are unchanged.
   - Mermaid's injected `<style>` element is inside the SVG and is blocked without `'unsafe-inline'`; the probe showed the result is an unreadable black diagram, not a merely unstyled one. A nonce cannot help because Mermaid sets no nonce, and hashes cannot enumerate dynamic theme CSS. Patching the vendored file is forbidden by ADR-0027 decision 3.
   - The relaxation is confined to the response class that already links a 3.4 MiB third-party renderer. Math-only and diagram-free pages keep the strict policy.
   - Residual risk is CSS-only: a hostile record that gets CSS through Mermaid could restyle or occlude the diagram page. It cannot execute script, cannot read files, and cannot reach the network: `default-src 'none'` blocks remote `@import`/`url()`, and `img-src`/`font-src` allow only same-origin asset paths. No supported browser executes CSS as script.
   - Tests assert the literal strict string for a math-only record view, a no-math record view, a list, an edit page, and an error page; the literal diagram string for a record view with a Mermaid figure; no CSP on assets; and the absence of `unsafe-inline` in every `script-src`.

7. **Input caps are fixed constants and part of the contract.**

   - Server pre-pass, per fence: **100,000 UTF-8 bytes** and **1,000 lines**. Counting stops at the cap. Over-limit input is not handed to Mermaid: the server emits the same figure shape, carries `data-render-error="diagram"`, keeps the full source in the `<pre>`, and adds `Diagram not rendered: source exceeds the 100,000-byte or 1,000-line limit.` The client skips any figure with `data-render-error`.
   - Client: `maxTextSize: 100000` (characters) and `maxEdges: 500`. Mermaid's flowchart parser throws `Edge limit exceeded` above 500 edges; `render()` rejects; the per-figure catch applies the failure contract. The server does **not** count edges, because counting them would require parsing Mermaid syntax on the server, which decision 3 and ADR-0025 forbid.
   - The caps are not configurable in this release: no configuration key, no environment variable, no per-record override. A record is user input (NFR-02.002), and a pathological diagram must not hang the browser thread or exhaust memory; changing a constant requires a new ADR.

8. **The sandboxed-iframe fallback is rejected.** An opaque-origin `sandbox="allow-scripts"` document cannot load `/assets/mermaid.min.js`: the request is cross-origin from an opaque origin and the asset server sends no CORS headers. Adding `allow-same-origin` removes the isolation that motivated the iframe. The only remaining variant inlines the 3.4 MiB bundle into the child document, which needs `script-src 'unsafe-inline'` inside the frame — a larger relaxation than the scoped `style-src` grant — plus height messaging, print, full-width, focus, and accessibility machinery. The scoped style grant contains the risk with less complexity, so the fallback is not adopted. A future proposal to revisit it needs its own ADR.

9. **No new Rust crate, no JVM, no service, no network.** Mermaid is a browser asset under ADR-0027; `Cargo.toml` is unchanged.

10. **Diagram output obeys the width contract.** A rendered diagram is a full-width-eligible region (ADR-0028): the `<figure>` carries the `bleed` class and a `.diagram-scroll` container with `overflow-x: auto`, so a wide diagram scrolls inside the figure instead of forcing page-wide horizontal scroll. The PlantUML source block is not a rendered visualization and stays within the normal text measure.

11. **Rendering is browser-only.** CLI output, `--json`, search, and the record files keep the source text. Nothing about a diagram is stored, cached, pre-rendered, or written back.

12. **Binary budget: the 12 MiB ceiling in ADR-0027 decision 6 is unchanged.** The projected binary is 4,730,296 + 3,572,661 = 8,302,957 bytes (7.92 MiB) before the small Rust and `app.js` additions, leaving 4,279,955 bytes (4.08 MiB) of headroom. The compressed `.deb` estimate rises by roughly 0.75–0.98 MB; no package-size budget exists.

## Threat model

A record author controls the diagram source, and Mermaid parses it client-side and builds SVG/DOM in the same origin as the unauthenticated loopback write API, so a Mermaid defect is a same-origin defect; the assets are the record corpus and the write API, not a remote account. Containment is layered and type-independent: `securityLevel: 'strict'` keeps Mermaid's DOMPurify sanitisation and disables click handling, `htmlLabels: false` removes the HTML-label injection path, the client never calls `bindFunctions` so no click or link binding is installed, the byte/line caps and `maxEdges` bound CPU and memory, `script-src 'self'` prevents inline, `eval`, or remote script, `default-src 'none'` prevents network fetches, `img-src`/`font-src 'self'` limit subresources to embedded assets, and the only style relaxation (`style-src 'self' 'unsafe-inline'`) is scoped to pages that already load the diagram bundle, so a CSS-only injection can restyle or occlude a diagram page but cannot execute script or exfiltrate data. Residual risk is a Mermaid CVE or a CSS-injection variant; the pinned version, the manifest digest, the advisory review, and the hostile-fixture tests are the controls.

## Consequences

**Positive:**

- One diagram ecosystem, one asset, one sanitisation story, and one set of limits; the security review has a single target.
- The single-binary, offline, and packaging constraints hold without a second runtime.
- The source-text fallback means a broken diagram never costs the reader the diagram source.
- The CSP relaxation is scoped to diagram pages; math-only and diagram-free pages keep the strict policy, which is stronger than a site-wide grant.
- Excluding PlantUML is an explicit, documented scope decision rather than an accidental capability gap.
- Pinning the last 11.x keeps Firefox 100 support and avoids the 12.x ES2024 floor and default-layout change.

**Negative:**

- Authors who write PlantUML must convert to Mermaid or accept source-only display. This is a visible product limitation relative to the customer's phrasing.
- The vendored bundle is 3.4 MiB — by far the largest single asset in the project. A diagram page transfers it on every navigation because `no-store` is retained.
- `style-src 'unsafe-inline'` weakens the strongest CSP directive pair on diagram pages. The mitigation is that `script-src` is untouched and the relaxation never reaches a page without a diagram, but a reviewer must accept the change consciously.
- Mermaid's own vulnerabilities become Papyrus vulnerabilities: a Mermaid CVE is a Papyrus CVE until the asset is updated. ADR-0027's pin, manifest digest, and update procedure are the control; 11.17.2 is clear of the known 2026 advisories.
- Only nine diagram types are promised. Other Mermaid types are best-effort; a type needing an unvendored layout (for example `flowchart-elk`) falls back to source.
- Diagram rendering is a client-side CPU cost on a page load; a large diagram is visibly slow on first paint.

**Risks and mitigations:**

- *XSS through Mermaid output*: `securityLevel: 'strict'`, `htmlLabels: false`, no click bindings, no `bindFunctions`, input caps, and a CSP that forbids remote content. A regression test renders hostile fixtures (`<img src=x onerror=…>`, a `classDef` payload, a `javascript:` link) and asserts no script node, no event-handler attribute, and no `javascript:` URL survives. The Firefox probe showed an `<img onerror>` label rendered as literal text under `securityLevel: 'strict'` and `htmlLabels: false`.
- *CSS injection on a diagram page*: accepted residual risk, bounded to restyling or occlusion. `default-src 'none'` blocks remote fetches, and no script path opens.
- *Bundle drift and CVEs*: pinned version, recorded SHA-256, licence file, and a documented review step on each Mermaid security advisory.
- *Diagram hangs the page*: byte and line caps plus Mermaid's `maxEdges`, per-figure `try`/`catch`; the rest of the page stays interactive.
- *CSP accepted too broadly*: two literal strings, asserted per response class; `unsafe-inline` appears only in the diagram `style-src`, never in `script-src`.
- *Full-width diagram forces page-level horizontal scroll*: every diagram is wrapped in an `overflow-x: auto` container and a test asserts `document.scrollWidth == clientWidth` on a wide-diagram page.
- *Mermaid 12.x drift*: the manifest pins 11.17.2; an update to 12.x is an NFR-05.001 browser-floor decision and needs a new ADR.

## Alternatives Considered

| Option | Outcome | Reason |
|---|---|---|
| Mermaid 11.17.2 vendored, client-side, scoped `style-src 'unsafe-inline'` (this ADR) | Selected | Only option that renders the requested diagram class offline, in one binary, without a second runtime, while keeping Firefox 100 support. |
| Mermaid 12.3.0 vendored | Rejected | 5,537,838 B (+1,965,177 B), ES2024 baseline and Safari 17.4+ floor incompatible with NFR-05.001's Firefox 100, and a breaking default-layout/theme change. |
| Mermaid rendered inside a sandboxed opaque-origin iframe with its own CSP | Rejected | An opaque origin cannot load the same-origin bundle without CORS, `allow-same-origin` defeats the isolation, and inlining the bundle requires `script-src 'unsafe-inline'` in the frame plus height/print/full-width complexity. |
| PlantUML via `plantuml.jar` + JVM | Rejected | Second runtime and process; breaks single-binary packaging (NFR-03.001, ADR-0002). |
| PlantUML via a local or remote PlantUML server | Rejected | Network or a background service; violates NFR-01.003 and ADR-0005's no-external-server rule. |
| `plantuml-encoder` in the browser + an image endpoint | Rejected | The encoder does not render; it produces a URL for a renderer that does not exist offline. |
| Graphviz / `viz.js` (WASM) | Rejected | 2–3 MB WASM plus `script-src 'wasm-unsafe-eval'` and a wasm loader; its syntax is not PlantUML or Mermaid, so it satisfies neither named example. |
| Server-side rendering to SVG at record save time | Rejected | REQ-06.017 requires browser-side rendering and source preservation; it would rewrite records or store derived artifacts, both forbidden by ADR-0013. |
| Mermaid syntax subset re-implemented in Rust → inline SVG | Rejected | A diagram layout engine is a large, permanent in-project liability; it would also contradict "no server-side rendering". |
| Diagrams deferred to a later release (math only) | Rejected | REQ-06.017 has an explicit acceptance criterion for diagrams; deferral is a requirement change, not an implementation choice. |

## Open Questions

All resolved. No open question remains for this ADR.

1. ~~Confirm PlantUML exclusion~~ — resolved by owner decision 2026-11-15: Mermaid-only in this release; a `plantuml`/`puml` fence displays as source with a note (decision 2).
2. ~~Accept `style-src 'self' 'unsafe-inline'`, or require the sandboxed-iframe fallback?~~ — resolved 2026-11-16: accept, scoped to HTML responses that link the diagram bundle; the sandboxed-iframe fallback is rejected (decisions 6 and 8).
3. ~~Which Mermaid diagram types are in the supported set?~~ — resolved 2026-11-16: flowchart, sequence, class, state, ER, gantt, pie, journey, gitGraph are promised and tested; other types are best-effort, and unparseable types fall back through the failure contract (decision 5).
4. ~~Should the input caps be configurable?~~ — resolved 2026-11-16: fixed constants, no configuration key (decision 7).
5. ~~Which Mermaid version?~~ — resolved 2026-11-16: 11.17.2, the last 11.x, for Firefox 100 compatibility, size, and advisory status (decision 4).
6. ~~Keep the `mmd` alias?~~ — resolved 2026-11-16: no; only `mermaid` is accepted (decision 1).
7. ~~Adopt the sandboxed-iframe fallback?~~ — resolved 2026-11-16: no (decision 8).

## Revision notes (2026-11-16, status completed)

| Decision | Before | After | Reason |
|---|---|---|---|
| Status | `Accepted` claimed on 2026-11-15 while four questions were open | Completed 2026-11-16; every open question resolved | TASK-146 could not start against an incomplete acceptance. |
| 1 — fence set | `mermaid` and `mmd` | `mermaid` only | REQ-06.017 names only `mermaid`; the alias is an unrequested extension. |
| 2 — PlantUML | `.puml`/`plantuml` note, `dot`/`graphviz` ambiguous | `plantuml`/`puml` source block with note; `uml`/`dot`/`graphviz` literal text | Aligns with the requirement's single named `plantuml` fence and removes ambiguity. |
| 4 — version and asset | "Mermaid 11.x", 3.4 MiB | 11.17.2, one file, 3,572,661 B, SHA-256 recorded | 11.x was not a pin; 12.x breaks Firefox 100 and adds 1.97 MB. |
| 5 — client | `mermaid.render()` only; type set unspecified | nine promised types, no allowlist, explicit failure contract, no `bindFunctions` | Makes the promise testable and the fallback deterministic. |
| 6 — CSP | site-wide `style-src 'unsafe-inline'` implied | only HTML responses with a renderable diagram | Keeps math-only and diagram-free pages on the strict policy. |
| 7 — caps | 100 KB / 1,000 lines / 500 edges enforced by "the pre-pass" | 100 KB / 1,000 lines server-side; 500 edges client-side via `maxEdges` | The server must not parse Mermaid syntax (decision 3, ADR-0025). |
| 8 — iframe | recorded fallback if `unsafe-inline` refused | rejected | Opaque-origin/CORS and inline-bundle costs outweigh the scoped style grant. |

## Downstream constraints

1. **TASK-146 (implementation).** Use `mermaid` only; no `mmd`. Emit the exact figure shapes of decisions 1–3; keep the `<pre>` until a successful render; enforce the 100,000-byte and 1,000-line caps server-side and `maxEdges: 500` client-side; do not parse Mermaid on the server; do not call `bindFunctions`; link the bundle only on `has_diagram` pages; use the two literal CSP strings of decision 6 and apply the diagram string only to `has_diagram` responses.
2. **TASK-148 (specifications).** Record the diagram fence set (`mermaid` only; `plantuml`/`puml` note; other fences literal), the supported type list, the two CSP strings and their response classes, the caps, the failure contract, and the 11.17.2 pin. Do not describe a site-wide `style-src` relaxation or an `mmd` alias.
3. **TASK-149 (packaging and budget).** The 12 MiB ceiling is unchanged; no threshold adjustment. TASK-149 runs before TASK-146 and can only measure the KaTeX-only binary, so it records that measurement and the 4.0 MiB projected headroom; the final post-Mermaid binary size is measured by TASK-150. If a single final measurement is preferred, TASK-146 must be added as a dependency (planner decision, not this ADR).
4. **TASK-150 (verification).** Verify the nine supported types, the `plantuml` note, the `mmd`/literal fence behaviour, an unknown-type fallback, per-figure isolation, the hostile fixtures, both CSP strings per response class, the absence of `unsafe-inline` in `script-src`, the bundle-only-on-diagram-pages rule, the final binary size, and the offline behaviour in Firefox 100+, Edge 130+, and Safari 18+.
5. **Asset and notice.** The Mermaid file, `LICENSE`, `UPSTREAM.md`, `MANIFEST.md` row, digest test, and the MIT section of the package `copyright` are mandatory before the asset ships (ADR-0027).
6. **No JVM, no `plantuml.jar`, no rendering service, no network call** exists in any code path; a grep test asserts no `plantuml`/`java` invocation.
7. **`script-src` stays `'self'`**; `'unsafe-inline'` may appear only in the diagram response's `style-src`. A change to that grant, to `securityLevel`, to `htmlLabels`, or to a cap requires a new ADR.
8. **Mermaid 12.x is out of scope** until NFR-05.001's Firefox floor is revisited; an asset update to 12.x needs a new ADR, not a manifest edit.
9. **Every rendered figure keeps its source `<pre>` until a successful replacement**, and a failure restores/keeps the source and marks the figure.
10. **Diagram source is never parsed, rewritten, or persisted by the server.**
11. **The 12 MiB binary budget** (ADR-0027 decision 6) remains the release gate; the projected 7.92 MiB leaves 4.08 MiB of headroom.

## References

- `docs/requirements/business/REQ-06.017.md` — diagrams in the browser.
- `docs/requirements/business/REQ-06.015.md` — full-width diagram content.
- `docs/requirements/non-functional/NFR-01.003.md`, `NFR-02.002.md`, `NFR-03.001.md`, `NFR-05.001.md`, `NFR-06.002.md`.
- `docs/specifications/browser-math-and-diagram-rendering.md` — the shared pre-pass, fallback contract, and CSP baseline.
- `docs/architecture/adr/ADR-0002-distribution-format-amd64-deb-packages.md`, `ADR-0003-supported-debian-release-target-forky-only.md` — one binary per package.
- `docs/architecture/adr/ADR-0005-embedded-dependency-free-http-server-and-server-rendered-ui.md` — dependency-free server, CSP baseline.
- `docs/architecture/adr/ADR-0025-bundled-katex-client-side-math-rendering.md` — the shared delimiter pre-pass and the fallback contract.
- `docs/architecture/adr/ADR-0027-vendored-embedded-browser-assets-and-packaging-notices.md` — asset embedding, checksums, licence notices, 12 MiB budget.
- `docs/architecture/adr/ADR-0028-browser-shell-shared-list-layout-content-width-and-collapsible-navigation.md` — width classes.
- Mermaid 11.17.2: `https://registry.npmjs.org/mermaid/-/mermaid-11.17.2.tgz`; advisory batches fixed by 11.16.1 (GHSA-c4c3-pg64-4m4v, GHSA-6x64-9x62-f2gx, GHSA-3rrr-jr9j-h3q3, GHSA-2v8p-3f2j-5mp7, GHSA-rhh3-jpg6-66xh, GHSA-6m6c-36f7-fhxh).
- Implementation: `src/browser/render.rs`, `src/browser/math.rs`, `src/browser/vendor/mermaid/11.17.2/mermaid.min.js`, `src/browser/assets.rs`, `src/browser/http.rs`, `src/browser/app.js`, `src/browser/app.css`.
