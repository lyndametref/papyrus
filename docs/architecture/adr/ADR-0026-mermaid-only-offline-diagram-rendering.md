# Mermaid-Only Offline Diagram Rendering, PlantUML Out of Scope

## Status

Proposed (2026-11-15) — awaiting owner validation. Do not implement before acceptance.

## Date

2026-11-15

## Context

REQ-06.017 requires the browser to render diagrams embedded in record content, offline, with bundled assets, in a single binary, under a dependency licence compatible with GPLv3-or-later, without arbitrary script execution, and with graceful degradation. It names "PlantUML- or Mermaid-style diagrams" as examples and deliberately leaves the library choice to implementation.

ADR-0025 settles math with a vendored KaTeX. Diagram rendering is a different problem: the two named ecosystems have incompatible delivery models.

- **Mermaid** is a JavaScript library (MIT). A pre-built UMD bundle, `mermaid.min.js`, is 3,572,661 bytes, and it renders into the current document.
- **PlantUML** is a Java program (GPL-3.0). Rendering requires `plantuml.jar` and a JVM (a second runtime, a second process, and ~50 MB of JVM plus jar), or a running PlantUML HTTP service (a network dependency). The available JavaScript package, `plantuml-encoder`, only deflates and encodes source into a URL or image path; it does not render. PlantUML's `!include` directive reads local files, which is a second filesystem surface in a page renderer.

Forces: ADR-0002/ADR-0003 ship one `.deb` containing `/usr/bin/papyrus`; ADR-0005 forbids dependencies, external servers, and remote assets; NFR-01.003 forbids network use; NFR-03.001 fixes a Rust single binary; NFR-02.002 requires sanitisation; REQ-06.015 lets diagram content use the full window width. Mermaid's renderer injects a `<style>` element into the SVG it produces, which collides with the current `style-src 'self'` policy.

Mermaid quality also matters: it is JS code executing in the same origin as the write API, so its sanitisation and input limits are part of the security model, not a detail.

## Decision

1. **Diagram syntax support is Mermaid only, rendered client-side by a vendored bundle.** Accepted fence info strings: `mermaid` and its alias `mmd`. Every other fence, including `plantuml`, `puml`, `uml`, `dot`, and `graphviz`, is displayed as literal code text — never as an error, never silently dropped.

2. **PlantUML is excluded from the product scope, not deferred silently.** A `.puml`/`plantuml` fence renders as source with a visible note that PlantUML is unsupported in this release. The exclusion is recorded as a product decision (see Open Questions 1) because REQ-06.017's wording is permissive ("for example … PlantUML- or Mermaid-style") rather than mandatory.

   Reasons, in order of force: a JVM and `plantuml.jar` cannot be shipped inside a Rust binary (NFR-03.001, ADR-0002); a PlantUML service is a network dependency (NFR-01.003); `plantuml-encoder` does not render; and PlantUML's file-include semantics would add a filesystem surface to the renderer.

3. **Server-side work is the same delimiter pre-pass as ADR-0025.** A fenced block with info string `mermaid`/`mmd` becomes `<figure class="diagram" data-diagram="mermaid"><pre>escaped source</pre></figure>`. The `<pre>` is both the fallback and the source of truth for the client. Nothing about Mermaid syntax is parsed on the server.

4. **Client contract.**
   - `mermaid.initialize({ startOnLoad: false, securityLevel: 'strict', htmlLabels: false, theme: 'neutral', fontFamily: 'system-ui, sans-serif', flowchart: { htmlLabels: false }, maxTextSize: 100000, maxEdges: 500 })`.
   - For each `figure[data-diagram="mermaid"]`, call `mermaid.render()` with a deterministic id derived from the record id and the figure index, then replace the `<pre>` with the returned SVG.
   - On any error, the `<pre>` stays and the figure gains `data-render-error="diagram"`, plus a one-line visible message beneath it. A failure in one figure does not abort the page or the other figures; each figure is rendered in its own `try`/`catch`.
   - The bundle is linked only on pages whose pre-pass found at least one Mermaid fence.
   - `securityLevel: 'strict'` keeps Mermaid's internal DOMPurify sanitisation and disables click handlers and HTML labels; `htmlLabels: false` removes the largest HTML-injection path. Both settings are mandatory and are asserted by a test on the served asset/config call.

5. **CSP: `style-src 'self' 'unsafe-inline'` is accepted; `script-src` stays `'self'`.** Mermaid injects a `<style>` element inside the SVG it produces (mermaid-js/mermaid#856, open since 2020; not configurable), so a strict `style-src 'self'` would render unstyled diagrams. The relaxation is confined to styles:

   - `script-src 'self'` remains, so no inline script, no `eval`, no `Function`, no remote script can run. Mermaid's own code is a same-origin asset loaded from `/assets/`.
   - `default-src 'none'` remains, so the rendered SVG cannot fetch anything.
   - The residual risk is CSS-only: a hostile record that manages to inject markup through Mermaid could at worst restyle the page or leak layout information through CSS selectors. It cannot execute script, read files, or reach the network.
   - If the security review refuses `'unsafe-inline'` for styles, the recorded fallback is option B below (sandboxed opaque-origin iframe with its own CSP); that fallback needs its own ADR because it changes sizing, printing, and full-width behaviour.

6. **Input limits are part of the contract.** The pre-pass enforces, per fence: ≤ 100,000 bytes, ≤ 1,000 lines, ≤ 500 edges as counted by Mermaid (`maxEdges`). Oversized or over-limit input is not handed to Mermaid; it is displayed as source with a note. Rationale: a record is user input (NFR-02.002), and a pathological diagram must not hang the browser thread or exhaust memory.

7. **No new Rust crate, no JVM, no service, no network.** Mermaid is a browser asset under ADR-0027; `Cargo.toml` is unchanged.

8. **Diagram output obeys the width contract.** A rendered diagram is a full-width-eligible region (ADR-0028): the `<figure>` carries the `bleed` class and an `overflow-x: auto` container so a wide diagram scrolls inside the figure instead of forcing page-wide horizontal scroll.

9. **Rendering is browser-only.** CLI output, `--json`, search, and the record files keep the source text. Nothing about a diagram is stored, cached, pre-rendered, or written back.

## Consequences

**Positive:**

- One diagram ecosystem, one asset, one sanitisation story, and one set of limits; the security review has a single target.
- The single-binary, offline, and packaging constraints hold without a second runtime.
- The source-text fallback means a broken diagram never costs the reader the diagram source.
- Excluding PlantUML is an explicit, documented scope decision rather than an accidental capability gap.

**Negative:**

- Authors who write PlantUML must convert to Mermaid or accept source-only display. This is a visible product limitation relative to the customer's phrasing.
- The vendored bundle is 3.4 MiB — by far the largest single asset in the project. Pages with diagrams carry it; pages without diagrams carry nothing.
- `style-src 'unsafe-inline'` weakens the strongest CSP directive pair the project had. The mitigation is that `script-src` is untouched, but a reviewer must accept the change consciously.
- Mermaid's own vulnerabilities become Papyrus vulnerabilities: a Mermaid CVE (for example CVE-2026-41149, HTML injection through `classDef`) is a Papyrus CVE until the asset is updated. ADR-0027's update procedure and checksum are the control.
- Diagram rendering is a client-side CPU cost on a page load; a large diagram is visibly slow on first paint.

**Risks and mitigations:**

- *XSS through Mermaid output*: `securityLevel: 'strict'`, `htmlLabels: false`, no click bindings, input size caps, and a CSP that forbids remote content. A regression test renders a hostile fixture (`<img src=x onerror=…>`, a `classDef` payload, a `javascript:` link) and asserts no script node and no attribute handler survives.
- *Bundle drift and CVEs*: pinned version, recorded SHA-256, licence file, and a documented review step on each Mermaid security advisory.
- *Diagram hangs the page*: size and edge caps plus per-figure `try`/`catch`; the rest of the page stays interactive.
- *CSP accepted too broadly*: the exact CSP string is asserted per response class, and `'unsafe-inline'` never appears in `script-src`.
- *Full-width diagram forces page-level horizontal scroll*: every diagram is wrapped in an `overflow-x: auto` container and a test asserts `document.scrollWidth == clientWidth` on a wide-diagram page.

## Alternatives Considered

| Option | Outcome | Reason |
|---|---|---|
| Mermaid vendored, client-side, `style-src 'unsafe-inline'` (this ADR) | Selected | Only option that renders the requested diagram class offline, in one binary, without a second runtime. |
| Mermaid rendered inside a sandboxed opaque-origin iframe with its own CSP | Rejected for now | Keeps the main page's `style-src 'self'` intact, but adds a second document, height messaging, print/full-width complications, and a `frame-src` CSP change. Recorded as the fallback if the security review rejects inline styles. |
| PlantUML via `plantuml.jar` + JVM | Rejected | Second runtime and process; breaks single-binary packaging (NFR-03.001, ADR-0002). |
| PlantUML via a local or remote PlantUML server | Rejected | Network or a background service; violates NFR-01.003 and ADR-0005's no-external-server rule. |
| `plantuml-encoder` in the browser + an image endpoint | Rejected | The encoder does not render; it produces a URL for a renderer that does not exist offline. |
| Graphviz / `viz.js` (WASM) | Rejected | 2–3 MB WASM plus `script-src 'wasm-unsafe-eval'` and a wasm loader; its syntax is not PlantUML or Mermaid, so it satisfies neither named example. |
| Server-side rendering to SVG at record save time | Rejected | REQ-06.017 requires browser-side rendering and source preservation; it would rewrite records or store derived artifacts, both forbidden by ADR-0013. |
| Mermaid syntax subset re-implemented in Rust → inline SVG | Rejected | A diagram layout engine is a large, permanent in-project liability; it would also contradict "no server-side rendering". |
| Diagrams deferred to a later release (math only) | Rejected | REQ-06.017 has an explicit acceptance criterion for diagrams; deferral is a requirement change, not an implementation choice. |

## Open Questions

1. **Confirm PlantUML exclusion** — the customer named PlantUML. Recommended: accept Mermaid-only in this release and record PlantUML as unsupported (source shown). Alternative: keep PlantUML as a future feature gated on a JVM-free Rust implementation.
2. **Accept `style-src 'self' 'unsafe-inline'`**, or require the sandboxed-iframe fallback? Recommended: accept it, on the grounds that `script-src 'self'` is untouched and `default-src 'none'` blocks all fetches.
3. **Which Mermaid diagram types are in the supported set?** Recommended: the ones Mermaid renders without external resources — flowchart, sequence, class, state, ER, gantt, pie, journey, gitGraph. Mermaid's `mindmap`/`timeline`/`sankey` should be checked against the pinned version before being promised.
4. **Should the input caps (100 KB / 1,000 lines / 500 edges) be configurable?** Recommended: fixed constants for this release; no configuration key.

## Downstream constraints

1. No JVM, no `plantuml.jar`, no rendering service, no network call exists in any code path; a grep test asserts no `plantuml`/`java` invocation.
2. `mermaid` is the only diagram fence language accepted; unknown fence languages stay literal text and never produce an error state.
3. `securityLevel: 'strict'`, `htmlLabels: false`, and the size/edge caps are mandatory; a change to any of them requires a new ADR.
4. `script-src` remains `'self'`; `'unsafe-inline'` may appear only in `style-src`, and only with this ADR in force.
5. Every rendered figure keeps its source `<pre>` in the document until a successful replacement, and restores the source on error.
6. Diagram assets load only on pages that contain a diagram; the total asset budget is governed by ADR-0027.
7. Diagram source is never parsed, rewritten, or persisted by the server.

## References

- `docs/requirements/business/REQ-06.017.md` — diagrams in the browser.
- `docs/requirements/business/REQ-06.015.md` — full-width diagram content.
- `docs/requirements/non-functional/NFR-01.003.md`, `NFR-02.002.md`, `NFR-03.001.md`, `NFR-05.001.md`, `NFR-06.002.md`.
- `docs/architecture/adr/ADR-0002-distribution-format-amd64-deb-packages.md`, `ADR-0003-supported-debian-release-target-forky-only.md` — one binary per package.
- `docs/architecture/adr/ADR-0005-embedded-dependency-free-http-server-and-server-rendered-ui.md` — dependency-free server, CSP baseline.
- `docs/architecture/adr/ADR-0025-bundled-katex-client-side-math-rendering.md` — the shared delimiter pre-pass and the fallback contract.
- `docs/architecture/adr/ADR-0027-vendored-embedded-browser-assets-and-packaging-notices.md` — asset embedding, checksums, licence notices.
- `docs/architecture/adr/ADR-0028-browser-shell-shared-list-layout-content-width-and-collapsible-navigation.md` — width classes.
- Implementation: `src/browser/render.rs`, `src/browser/vendor/mermaid/11/<bundle>`, `src/browser/assets.rs` (new), `src/browser/http.rs`.
