---
id: ADR-0025
title: "Bundled KaTeX Client-Side Math Rendering"
created_date: '2026-11-15T00:00:00Z'
updated_date: '2026-11-15T15:58:31Z'
labels: []
references: []
---
# Bundled KaTeX Client-Side Math Rendering

## Status

Accepted (2026-11-15) — owner decision 2026-11-15: KaTeX is the accepted math renderer, with the `$$...$$` / `$...$` (`\$` escape) / fenced `math|tex|latex` delimiter set and curated-prose-only scope (no record-body view).

## Date

2026-11-15

## Context

REQ-06.017 requires the local web interface to render math formulas embedded in managed record content, fully offline, with all assets bundled, in the supported browsers (NFR-05.001), without arbitrary script execution (NFR-02.002), and with a rendering dependency that is open source and permits linking in a GPLv3-or-later application (NFR-06.002). Malformed input must degrade to visible source text instead of breaking the page.

Current shape that this decision must fit:

- ADR-0005 fixes a dependency-free HTTP server, server-side `{{name}}` templating, `include_str!` assets, a fixed route table, `Cache-Control: no-store`, and a CSP of `default-src 'none'; style-src 'self'; script-src 'self'; img-src 'self'; form-action 'self'; base-uri 'none'; frame-ancestors 'none'`. It explicitly rejected a Markdown renderer.
- `browser::render` emits every prose field through `escape()` into `<p class="text">` with `white-space: pre-wrap`. Verbs `src/browser/app.js` does nothing but the dirty-form guard and the narrow-viewport sidebar toggle.
- No record body is displayed today. The detail pages render curated sections (`description`, `rationale`, `context`, `consequences`, `content`, `story`, `applicability`, `definition`, `view`) and never the raw file.
- The binary is 4.2 MiB (`.deb` 1.28 MiB) with four Rust dependencies (`clap`, `serde`, `serde_json`, `yaml_serde`).

Forces: offline and single-binary (NFR-01.003, NFR-03.001, ADR-0002); no new Rust crate and no build toolchain beyond `cargo` (ADR-0001, ADR-0004); the CSP must not be weakened in `script-src`; the source text must remain the only stored value, so rendering cannot rewrite records; and the browser must stay usable without JavaScript (ADR-0005).

REQ-06.017 states that rendering happens **in the browser only** and that CLI output shows source text unchanged, which removes server-side preprocessing from the option space at requirement level; it is still recorded below so the exclusion is traceable.

## Decision

1. **Math is rendered client-side by KaTeX, vendored into the binary.** No TeX engine, service, or preprocessing step exists on the server. `papyrus` and every non-browser surface keep printing the stored source text; the browser prints the same source until a client script replaces the placeholder.

2. **The server does a non-Markdown pre-pass over each displayed prose field and emits placeholders.** This is the whole of the server-side work and it is not a Markdown renderer:

   - The public delimiter contract is exactly three forms: `$$...$$` (display), `$...$` (inline, a backslash-escaped `\$` is a literal dollar and never opens math), and a fenced block whose info string is `math`, `tex`, or `latex` (display). No other delimiter is recognized: `\(...\)` and `\[...\]` stay literal text.
   - Fenced blocks whose info string is not a recognized math/diagram language stay literal text (they are code).
   - The pass only splits a displayed field into an ordered list of segments: *literal text* (escaped exactly as today) and *math* (a `<span class="math" data-tex="…">` or `<div class="math math--display" data-tex="…">` container whose inner content is the escaped source).
   - Nothing else about Markdown is interpreted. Headings, lists, tables, emphasis, links, and raw HTML remain literal text, exactly as ADR-0005 decided. This pass is the only new interpretation of record content and it is limited to math/diagram delimiters.

3. **The placeholder carries the fallback, not an empty box.** Because the source text is the element's initial content, the page is correct with JavaScript disabled, when the asset fails to load, and when KaTeX rejects the input. A successful render replaces the element's content.

4. **Assets and loading.**
   - `src/browser/vendor/katex/0.16.11/katex.min.js` (275,414 B), `katex.min.css` (23,335 B), and the 20 woff2 font files (≈259,792 B total) are committed pre-built and embedded with `include_bytes!` through the asset table of ADR-0027.
   - KaTeX CSS is served **byte-identical** from `/assets/katex/katex.min.css`; its 60 relative `url(fonts/…)` references resolve to `/assets/katex/fonts/…`, which is exactly where the woff2 files are served. No vendoring patch is applied, so the recorded digest matches the upstream file. Only the woff2 form is served (the CSS also lists woff and ttf fallbacks that the supported browsers never request). No request path is ever joined to a filesystem path: every asset path is an entry in the static table.
   - The stylesheet and script are linked **only** on pages whose pre-pass found at least one math segment; other pages gain no bytes.
   - The script is a classic script with `defer`. No module loading, no `fetch`, no dynamic `import()`.

5. **Rendering call.** One pass over `[data-tex]` elements using `katex.render(source, element, { throwOnError: false, displayMode, trust: false, strict: 'warn', output: 'html' })`. `throwOnError: false` renders the offending expression in `errorColor` rather than throwing; a thrown error or a missing global leaves the source text in place and marks the element `data-render-error="math"`.

6. **CSP delta: `font-src 'self'` only.** KaTeX emits class-bearing spans and needs its woff2 files; it does not need inline styles, inline scripts, or `eval`. The served CSP becomes:

   `default-src 'none'; style-src 'self'; script-src 'self'; img-src 'self'; font-src 'self'; form-action 'self'; base-uri 'none'; frame-ancestors 'none'`

   `script-src` is unchanged, so no script path other than the embedded assets can execute.

7. **Sanitisation is structural.** `trust: false` disables `\href`, `\url`, `\includegraphics`, `\htmlClass`, and `\htmlData`; KaTeX output is built from its own text nodes; the TeX source reaches it as a string attribute, never as HTML. The attribute is escaped by the existing escaping rules. A record cannot make KaTeX emit a script or a remote reference.

8. **Scope of rendering: curated prose fields only.** `description`, `story`, `rationale`, `applicability`, `definition`, `view`, `context`, `consequences`, `content`, and the generic document body text. IDs, titles, labels, list cells, metadata rows, and form inputs stay literal — they are identity, not prose, and KaTeX inside a list cell would break the shared list layout (ADR-0028). This ADR adds **no record-body view**: no page renders a raw record body, and a future body view is a separate feature that reuses this pre-pass rather than widening this scope silently.

9. **No new Rust crate, no new build step.** The only new artifacts are vendored browser assets; `Cargo.toml` is untouched. `cargo build` stays the entire build.

10. **Rendering is a display concern only.** No write path, patch, front matter, or search index knows about math. `papyrus search` and `papyrus <slug> view` keep matching and printing the source text.

## Consequences

**Positive:**

- REQ-06.017's offline, bundled, single-binary, and graceful-degradation criteria are met with one asset family and no runtime dependency.
- The CSP keeps `script-src 'self'`, `default-src 'none'`, and no `eval`; the only relaxation is `font-src 'self'`, which cannot execute anything.
- The fallback is the source text by construction, not by error handling, so a browser without JavaScript and a browser with a broken asset behave identically and acceptably.
- Restricting the pre-pass to `$`/`$$`/fenced math keeps ADR-0005's "no Markdown renderer" decision intact and keeps the served markup auditable.

**Negative:**

- The project takes on a vendored third-party browser asset with its own release cycle, checksums, and licence notices (ADR-0027).
- KaTeX needs its woff2 fonts, so the "no web font" wording used in several existing packaging specifications must be revised (owner: business analyst / tech writer).
- The `$…$` rule is a heuristic: a record containing a literal dollar amount (`$5 and $10`) will be read as inline math. Escaping with `\$` is the escape hatch, and the fallback shows the expression rather than hiding it.
- `strict: 'warn'` writes KaTeX warnings to the browser console on unusual input.
- The pre-pass touches every prose field in `browser::render` (4.7k lines), which is the largest regression surface of this change.

**Risks and mitigations:**

- *Dollar-sign false positives*: a record with prices renders oddly. Mitigation: require the `$…$` form to open on a non-space and close on a non-space, disallow a newline inside inline math, and document `\$`; block math (`$$`, fenced) is unambiguous and is the recommended form.
- *Asset drift or tampering*: the vendored file is committed with a SHA-256 and a licence file, and a test asserts the embedded bytes match the recorded digest.
- *CSP regression*: one test asserts the exact CSP string per response class; a second asserts KaTeX renders with that CSP in the supported browsers.
- *Font rendering differences across the three supported browsers*: NFR-05.001 verification must include a math page on Firefox 100+, Edge 130+, and Safari 18+.
- *Licence uncertainty on the font files*: KaTeX code is MIT and its woff2 fonts are SIL OFL-1.1; the OFL review must complete before acceptance. If it fails, the recorded fallback is MathJax 3 with SVG output (Apache-2.0, no web fonts, no `font-src`), which needs its own ADR because the asset weight and the CSP delta differ.

## Alternatives Considered

| Option | Outcome | Reason |
|---|---|---|
| KaTeX, vendored, client-side, `font-src 'self'` (this ADR) | Selected | Smallest bundle (≈560 KB), no `eval`, no inline style, permissive licences, synchronous DOM rendering that keeps the source-text fallback trivial. |
| MathJax 3, vendored, SVG output | Rejected for now | Apache-2.0 and no web fonts (no CSP delta), but ≈1.4–2 MB, slower startup, and its SVG pipeline adds DOM/style behaviour that needs its own review; kept as the licence fallback. |
| Server-side TeX preprocessing in Rust | Rejected | Excluded by REQ-06.017 ("rendering happens in the browser only"), and no TeX engine exists among the four current crates. |
| Server-side preprocessing in a helper process (TeX Live, `pandoc`) | Rejected | Breaks single-binary packaging (ADR-0002), offline guarantee, and adds an external runtime. |
| MathJax or KaTeX from a CDN | Rejected | Violates NFR-01.003 and ADR-0005's no-remote-asset rule. |
| Render math into the stored record at write time (SVG or MathML in the file) | Rejected | Rewrites record content, breaks source-of-truth and the preservation rule (ADR-0013), and duplicates the rendering contract in the CLI. |
| Full Markdown rendering (headings, lists, emphasis, math) | Rejected | Contradicts ADR-0005 and expands the sanitisation surface far beyond this requirement. |

## Resolved questions (owner, 2026-11-15)

1. **Delimiter set** — decided: `$$...$$` (display), `$...$` (inline, `\$` escapes a literal dollar), and a fenced block with info string `math`, `tex`, or `latex` (display). `\(...\)` and `\[...\]` are not recognized.
2. **Record body** — decided: rendering applies to the curated prose fields of decision 8 only; no full record-body view is added by this ADR.
3. **Lists and metadata rows** — decided: no math rendering there; structure and identity stay literal (decision 8).

## Downstream constraints

1. No Rust crate is added; math rendering has no server component beyond the delimiter pre-pass.
2. The pre-pass may not interpret any Markdown construct other than math/diagram delimiters; a diff that adds bold, links, headings, or lists to it is out of contract.
3. KaTeX receives only escaped attribute values; `trust` stays `false`; no KaTeX option may enable `\href`/`\includegraphics`.
4. `script-src` stays `'self'`; the CSP may add `font-src 'self'` and nothing else on account of this ADR.
5. Math assets are served only on pages that contain math; the binary must not grow by more than the ADR-0027 budget.
6. The source text is never modified, never re-saved, and never sent to the CLI in rendered form.
7. Existing packaging-specification wording that forbids web fonts (`docs/specifications/*.md`) must be revised; that update is owned by the business analyst / tech writer, not by this ADR.

## References

- `docs/requirements/business/REQ-06.017.md` — render math and diagrams in the browser.
- `docs/requirements/business/REQ-06.015.md` — content-width rules that apply to rendered output.
- `docs/requirements/non-functional/NFR-01.003.md`, `NFR-02.002.md`, `NFR-03.001.md`, `NFR-05.001.md`, `NFR-06.002.md`.
- `docs/architecture/adr/ADR-0005-embedded-dependency-free-http-server-and-server-rendered-ui.md` — transport, templates, CSP baseline, no Markdown renderer.
- `docs/architecture/adr/ADR-0027-vendored-embedded-browser-assets-and-packaging-notices.md` — how vendored assets are embedded, served, and licensed.
- `docs/architecture/adr/ADR-0028-browser-shell-shared-list-layout-content-width-and-collapsible-navigation.md` — width classes applied to rendered output.
- Implementation: `src/browser/render.rs` (pre-pass and placeholders), `src/browser/vendor/katex/0.16.11/`, `src/browser/assets.rs` (new), `src/browser/http.rs` (CSP, binary bodies), `src/browser/app.css`.
