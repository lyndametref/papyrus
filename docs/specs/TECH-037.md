---
id: TECH-037
title: "Mermaid Diagram Rendering"
created_date: '2026-11-27T20:35:46Z'
updated_date: '2026-11-27T20:35:46Z'
labels: []
references:
  - REQ-006.017
  - NFR-001.003
  - NFR-002.002
  - NFR-005.001
  - NFR-006.002
status: verified
---
# TECH-037 Mermaid Diagram Rendering

## Description
Covers the vendored Mermaid asset, its served path, the client initialization and render contract, failure handling, input caps, and supply-chain obligations. Markdown conversion, the math/diagram pre-pass, placeholder markup, and conditional asset inclusion are separate specifications.

## Vendored asset

- Mermaid 11.17.2 is committed pre-built, byte-identical to upstream, and embedded in the binary through the static asset table.
- `mermaid.min.js` is 3,572,661 B, SHA-256 `581ed7d74bd9048d0e3a91363927d72ef22942d7722546b27f7cc29e35390eb8`, MIT.
- It lives under `apps/browser/web/vendor/mermaid/11.17.2/`, which holds the unmodified upstream file, a `LICENSE`, and an `UPSTREAM.md` with package, version, upstream URL, and retrieval date.
- No Rust crate, Node toolchain, or network access is added; the workspace manifests are unchanged and `cargo build` remains the whole build.
- The file is served from `/assets/mermaid/11.17.2/mermaid.min.js`, matching the embedded path exactly. The unversioned path answers `404` for every method.
- The path is a literal entry of the static asset table, matched exactly: no prefix matching, path join, version negotiation, redirect, or alias.
- Vendor responses are immutable (`Cache-Control: public, max-age=31536000, immutable`, no `ETag`); the version segment is the version directory name and the manifest version, never the Papyrus package version.

## Client contract

Initialization is fixed:

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

- For each `figure[data-diagram="mermaid"]:not([data-render-error])` the client reads `pre.textContent` and calls `mermaid.render(id, source)` with a deterministic, page-unique id derived from the record id and the figure index and sanitised to `[A-Za-z0-9_-]`.
- On success it replaces the `<pre>` with the returned SVG inside `.diagram-scroll`.
- `bindFunctions` is never called, so no click, link, or tooltip binding is installed.
- No `fetch`, dynamic `import()`, `MutationObserver`, second pass, or storage write is used.
- Supported diagram types: `flowchart` (including `graph`), `sequence`, `class`, `state`, `ER`, `gantt`, `pie`, `journey`, and `gitGraph`.
- No allowlist is enforced: another type the pinned bundle can parse renders on a best-effort basis and is not a promised capability, and a type it cannot parse, including `flowchart-elk`, falls back through the failure contract.

## Failure handling

- On a thrown error the `<pre>` stays, the figure gains `data-render-error="diagram"`, and one note line is appended: `Diagram not rendered: Mermaid rejected the source.`
- A missing `window.mermaid` gives every figure `data-render-error="diagram"` and the note `Diagram not rendered: the diagram script did not load.`, and removes nothing.
- Each figure renders inside its own `try`/`catch`; one failure aborts neither the page nor another figure.

## Input caps

- Server, per fence: 100,000 UTF-8 bytes and 1,000 lines. Counting stops at the cap. Over-limit input is not handed to Mermaid: the server emits the same figure shape with `data-render-error="diagram"`, keeps the full source in the `<pre>`, and adds `Diagram not rendered: source exceeds the 100,000-byte or 1,000-line limit.`
- Client: `maxTextSize: 100000` characters and `maxEdges: 500`. Above 500 edges the flowchart parser throws, `render()` rejects, and the per-figure failure contract applies. The server does not count edges.
- The caps are fixed constants; no configuration key, environment variable, or per-record override changes them.

## Supply chain

- `apps/browser/web/vendor/MANIFEST.md` lists the package, version, licence, upstream URL, byte size, and SHA-256 of `mermaid.min.js`.
- A test recomputes the digest of the embedded asset and fails on a mismatch.
- A licence that cannot be shown compatible with GPL-3.0-or-later distribution blocks shipment.
- `packaging/deb/usr/share/doc/papyrus/copyright` carries a Mermaid section with name, version, licence identifier, and licence text, in addition to the Papyrus GPL-3.0-or-later statement.

## Invariants

- Rendering is browser-only and works offline with no network request, external service, or server-side rendering.
- The stored record text is the only source and is never rewritten, re-saved, cached, pre-rendered, or written back.
- CLI output, `--json`, search, and record files keep the source text.
- No JVM, `plantuml.jar`, or rendering service exists in any code path.
- Supported browser: Firefox 115 and later, with no layout breakage on pages that render diagrams.
