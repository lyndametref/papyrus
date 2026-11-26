---
id: TECH-105
title: "Math and Diagram Rendering Assets"
created_date: '2026-11-25T20:53:43Z'
updated_date: '2026-11-25T20:53:43Z'
labels: []
references:
  - REQ-006.017
  - NFR-001.003
  - NFR-002.002
  - NFR-005.001
  - NFR-006.002
status: draft
---
# TECH-105 Math and Diagram Rendering Assets

## Description
Covers math and diagram rendering in the local web interface: vendored KaTeX and Mermaid assets and their served
paths, the server pre-pass, client contracts and caps, both CSP variants, and supply-chain obligations.

## Vendored assets and served paths

Two third-party browser assets are committed pre-built, byte-identical to upstream, and embedded in the binary
through the static asset table:

- KaTeX 0.16.11 — `katex.min.js` (275,414 B), `katex.min.css` (23,335 B), and 20 WOFF2 fonts (≈259,792 B total).
  Code is MIT; the fonts are SIL OFL-1.1.
- Mermaid 11.17.2 — `mermaid.min.js` (3,572,661 B), SHA-256
  `581ed7d74bd9048d0e3a91363927d72ef22942d7722546b27f7cc29e35390eb8`, MIT.

They live under `src/browser/vendor/katex/0.16.11/` and `src/browser/vendor/mermaid/11.17.2/`. Each version
directory holds the unmodified upstream files, a `LICENSE`, and an `UPSTREAM.md` with package, version, upstream
URL, and retrieval date. No Rust crate, Node toolchain, or network access is added; `Cargo.toml` is unchanged and
`cargo build` remains the whole build.

Vendor files are served from versioned paths that mirror the embedded paths, with `src/browser/vendor` replaced
by `/assets`:

- `/assets/katex/0.16.11/katex.min.css` ← `vendor/katex/0.16.11/katex.min.css`
- `/assets/katex/0.16.11/katex.min.js` ← `vendor/katex/0.16.11/katex.min.js`
- `/assets/katex/0.16.11/fonts/<name>.woff2` (20 files) ← the same path under `vendor/katex/0.16.11/fonts/`
- `/assets/mermaid/11.17.2/mermaid.min.js` ← `vendor/mermaid/11.17.2/mermaid.min.js`

Paths are literal entries of the static asset table, matched exactly: no prefix matching, path join, version
negotiation, redirect, or alias. The unversioned paths `/assets/katex/katex.min.css`,
`/assets/katex/katex.min.js`, `/assets/katex/fonts/<name>.woff2`, and `/assets/mermaid/mermaid.min.js` answer
`404` for every method. `katex.min.css` is unpatched and byte-identical: its relative `url(fonts/…)` references
resolve inside `/assets/katex/0.16.11/`, and only the WOFF2 form is served. Vendor responses are immutable
(`Cache-Control: public, max-age=31536000, immutable`, no `ETag`); the version segment is the version directory
name and the manifest version, never the Papyrus package version.

## Server pre-pass

The server performs a non-Markdown pre-pass over displayed prose fields. It interprets math and diagram delimiters
only; headings, lists, tables, emphasis, links, and raw HTML stay literal text. Recognized input forms:

- `$$...$$` — display math; `$...$` — inline math, where a backslash-escaped `\$` is a literal dollar sign and
  never opens math.
- A fenced block with info string `math`, `tex`, or `latex` — display math.
- A fenced block with info string exactly `mermaid` — a diagram. Matching is case-sensitive, trailing whitespace
  is trimmed, zero to three leading spaces are allowed, at least three backticks or tildes open the fence, and
  opener and closer use the same character.
- A fenced block with info string `plantuml` or `puml` — source text with a note.

`\(...\)` and `\[...\]` are not recognized and stay literal. `mmd`, `uml`, `dot`, `graphviz`, and every other
unknown info string stay literal text with their fence lines included. An empty or whitespace-only `mermaid`
fence is literal text, not a figure. Nothing about TeX or Mermaid syntax is parsed on the server; for diagrams the
server counts bytes and lines only.

## Placeholders

The pre-pass splits a field into an ordered list of literal-text and math or diagram segments:

- inline math: `<span class="math" data-tex="…">` with the escaped source as its inner content;
- display math: `<div class="math math--display" data-tex="…">` with the same inner content;
- a `mermaid` fence: `<figure class="diagram bleed" data-diagram="mermaid">` containing
  `<div class="diagram-scroll" role="region" aria-label="Diagram N" tabindex="0"><pre>escaped source</pre></div>`,
  where `N` is a 1-based figure number unique across the page;
- a `plantuml` or `puml` fence: `<figure class="diagram diagram--unsupported" data-diagram="plantuml">` with
  `<pre>escaped source</pre>` and a `figcaption` of class `render-note` reading
  `PlantUML diagrams are not supported in this release; showing source.` The block is server-rendered, links no
  script, and does not change the CSP class.

A field with no math and no diagram produces byte-identical output. Rendering applies to curated prose fields only:
for document records, every resolved field whose kind is `text` or `multiline` and whose value is present. IDs,
titles, labels, metadata rows, list cells, and form inputs stay literal.

## Fallback contract

The escaped source text is the element's initial content, so the page is correct with JavaScript disabled, when an
asset fails to load, and when a library rejects the input. A successful render replaces that content.

- Math: a thrown error or a missing `window.katex` leaves the source in place and sets `data-render-error="math"`.
- Diagram: on a thrown error the `<pre>` stays, the figure gains `data-render-error="diagram"`, and one note line
  is appended: `Diagram not rendered: Mermaid rejected the source.`
- A missing `window.mermaid` gives every figure `data-render-error="diagram"` and the note
  `Diagram not rendered: the diagram script did not load.`, and removes nothing.
- Each figure renders inside its own `try`/`catch`; one failure aborts neither the page nor another figure.

## KaTeX client

One pass over `[data-tex]` elements calls:

```js
katex.render(source, element, {
  throwOnError: false, displayMode, trust: false, strict: 'warn', output: 'htmlAndMathml'
});
```

`output: 'htmlAndMathml'` is pinned so the MathML node exposes the formula, including a
`<annotation encoding="application/x-tex">` with the source, to assistive technology; the HTML node alone is
marked `aria-hidden="true"`. `throwOnError: false` renders a rejected expression in `errorColor` instead of
throwing. `trust: false` disables `\href`, `\url`, `\includegraphics`, `\htmlClass`, and `\htmlData`; the TeX
source reaches KaTeX only as an escaped attribute value, never as HTML. The style policy drops the inline `style`
attributes KaTeX emits, so spacing may deviate slightly and a rejected formula shows uncoloured.

## Mermaid client

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

For each `figure[data-diagram="mermaid"]:not([data-render-error])` the client reads `pre.textContent` and calls
`mermaid.render(id, source)` with a deterministic, page-unique id derived from the record id and the figure index
and sanitised to `[A-Za-z0-9_-]`. On success it replaces the `<pre>` with the returned SVG inside
`.diagram-scroll`. `bindFunctions` is never called, so no click, link, or tooltip binding is installed. No
`fetch`, dynamic `import()`, `MutationObserver`, second pass, or storage write is used. Supported diagram types:
`flowchart` (including `graph`), `sequence`, `class`, `state`, `ER`, `gantt`, `pie`, `journey`, and `gitGraph`.
No allowlist is enforced: another type the pinned bundle can parse renders on a best-effort basis and is not a
promised capability, and a type it cannot parse, including `flowchart-elk`, falls back through the failure
contract.

## Input caps

- Server, per fence: 100,000 UTF-8 bytes and 1,000 lines. Counting stops at the cap. Over-limit input is not
  handed to Mermaid: the server emits the same figure shape with `data-render-error="diagram"`, keeps the full
  source in the `<pre>`, and adds `Diagram not rendered: source exceeds the 100,000-byte or 1,000-line limit.`
- Client: `maxTextSize: 100000` characters and `maxEdges: 500`. Above 500 edges the flowchart parser throws,
  `render()` rejects, and the per-figure failure contract applies. The server does not count edges.

The caps are fixed constants; no configuration key, environment variable, or per-record override changes them.

## Asset loading and CSP

- The KaTeX stylesheet and script are linked only on pages whose pre-pass found at least one math segment. The
  Mermaid bundle is linked only on pages with at least one renderable `mermaid` figure. A page with neither gains
  no vendor bytes; a page with only an over-limit or PlantUML block gets neither. All scripts are classic and
  deferred, loaded in the order KaTeX stylesheet, KaTeX script, Mermaid script, `app.js`.

The server emits exactly two HTML policy strings, differing in one token:

```
default-src 'none'; style-src 'self'; script-src 'self'; img-src 'self'; font-src 'self'; form-action 'self'; base-uri 'none'; frame-ancestors 'none'
```

```
default-src 'none'; style-src 'self' 'unsafe-inline'; script-src 'self'; img-src 'self'; font-src 'self'; form-action 'self'; base-uri 'none'; frame-ancestors 'none'
```

The first applies to every HTML response without a renderable diagram, including math-only record views, edit
pages, lists, the board, and error pages. The second applies only to an HTML response whose pre-pass set a
renderable diagram. `script-src` is `'self'` in both strings and never contains `'unsafe-inline'`,
`'unsafe-eval'`, a nonce, or a hash. Mermaid injects a `<style>` element into the SVG it produces; that injection
is the sole reason for the diagram-page style relaxation.

## Supply chain and budget

`src/browser/vendor/MANIFEST.md` lists, per asset, package, version, licence, upstream URL, byte size, and
SHA-256. A test recomputes the digest of every embedded asset and fails on a mismatch. An asset whose licence
cannot be shown compatible with GPL-3.0-or-later distribution does not ship.
`packaging/deb/usr/share/doc/papyrus/copyright` carries one section per bundled asset with name, version, licence
identifier, and licence text, in addition to the Papyrus GPL-3.0-or-later statement; the package keeps one
documentation file. The release binary stays within the 12 MiB budget; assets are embedded unconditionally, so
only their transfer is conditional.

## Invariants

Rendering is browser-only and works offline with no network request, external service, or server-side rendering.
The stored record text is the only source and is never rewritten, re-saved, cached, pre-rendered, or written back.
CLI output, `--json`, search, and record files keep the source text. No TeX engine, JVM, `plantuml.jar`, or
rendering service exists in any code path. Supported browser: Firefox 115 and later, with no layout breakage on
pages that render math and diagrams.
