---
id: TECH-105
title: "KaTeX Math Rendering"
created_date: '2026-11-25T20:53:43Z'
updated_date: '2026-11-27T18:38:06Z'
labels: []
references:
  - REQ-006.017
  - NFR-001.003
  - NFR-002.002
  - NFR-005.001
  - NFR-006.002
status: verified
---
# TECH-105 KaTeX Math Rendering

## Description
Covers the vendored KaTeX asset, its served paths, the client rendering contract, failure handling, and supply-chain obligations. Markdown conversion, the math/diagram pre-pass, placeholder markup, and conditional asset inclusion are separate specifications.

## Vendored asset

- KaTeX 0.16.11 is committed pre-built, byte-identical to upstream, and embedded in the binary through the static asset table.
- The files are `katex.min.js` (275,414 B), `katex.min.css` (23,335 B), and 20 WOFF2 fonts (≈259,792 B total). The code is MIT; the fonts are SIL OFL-1.1.
- They live under `apps/browser/web/vendor/katex/0.16.11/`, which holds the unmodified upstream files, a `LICENSE`, and an `UPSTREAM.md` with package, version, upstream URL, and retrieval date.
- No Rust crate, Node toolchain, or network access is added; the workspace manifests are unchanged and `cargo build` remains the whole build.
- The files are served from `/assets/katex/0.16.11/katex.min.css`, `/assets/katex/0.16.11/katex.min.js`, and `/assets/katex/0.16.11/fonts/<name>.woff2`, matching the embedded paths exactly.
- Paths are literal entries of the static asset table, matched exactly: no prefix matching, path join, version negotiation, redirect, or alias. The unversioned paths answer `404` for every method.
- `katex.min.css` is unpatched and byte-identical: its relative `url(fonts/…)` references resolve inside `/assets/katex/0.16.11/`, and only the WOFF2 form is served.
- Vendor responses are immutable (`Cache-Control: public, max-age=31536000, immutable`, no `ETag`); the version segment is the version directory name and the manifest version, never the Papyrus package version.

## Client contract

- One pass over `[data-tex]` elements calls `katex.render(source, element, { throwOnError: false, displayMode, trust: false, strict: 'warn', output: 'htmlAndMathml' })`.
- `output: 'htmlAndMathml'` is pinned so the MathML node exposes the formula, including an `<annotation encoding="application/x-tex">` with the source, to assistive technology; the HTML node alone is marked `aria-hidden="true"`.
- `throwOnError: false` renders a rejected expression in `errorColor` instead of throwing.
- `trust: false` disables `\href`, `\url`, `\includegraphics`, `\htmlClass`, and `\htmlData`; the TeX source reaches KaTeX only as an escaped attribute value, never as HTML.
- The style policy drops the inline `style` attributes KaTeX emits, so spacing may deviate slightly and a rejected formula shows uncoloured.

## Failure handling

- A thrown error or a missing `window.katex` leaves the escaped source in place and sets `data-render-error="math"`.
- One failure does not abort the page or another formula.
- The source stays readable when JavaScript is disabled or the asset fails to load.

## Supply chain

- `apps/browser/web/vendor/MANIFEST.md` lists the package, version, licence, upstream URL, byte size, and SHA-256 of `katex.min.js` and its fonts.
- A test recomputes the digest of every embedded KaTeX asset and fails on a mismatch.
- A licence that cannot be shown compatible with GPL-3.0-or-later distribution blocks shipment.
- `packaging/deb/usr/share/doc/papyrus/copyright` carries a KaTeX section with name, version, licence identifier, and licence text, in addition to the Papyrus GPL-3.0-or-later statement.

## Invariants

- Rendering is browser-only and works offline with no network request, external service, or server-side rendering.
- The stored record text is the only source and is never rewritten, re-saved, cached, pre-rendered, or written back.
- CLI output, `--json`, search, and record files keep the source text.
- No TeX engine or rendering service exists in any code path.
- Supported browser: Firefox 115 and later, with no layout breakage on pages that render math.
