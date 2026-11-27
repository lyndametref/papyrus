---
id: TECH-098
title: "Browser Rendering"
created_date: '2026-11-25T20:42:11Z'
updated_date: '2026-11-27T18:38:05Z'
labels: []
references:
  - REQ-006.017
  - NFR-001.003
  - NFR-002.002
  - NFR-005.001
status: verified
---
# TECH-098 Browser Rendering

## Description
Covers the browser rendering pipeline: Markdown-to-HTML conversion of displayed prose fields, the math and diagram pre-pass, placeholder markup, source fallback, conditional vendor asset inclusion, server-side template rendering, and the rendering constraints. The KaTeX and Mermaid asset contracts are separate specifications; routes are separate.

## Markdown to HTML

- Every displayed prose field is converted from Markdown to HTML at render time.
- The converter is CommonMark-compatible and renders headings, paragraphs, emphasis, strong text, lists, tables, code blocks, inline code, blockquotes, and links.
- Raw HTML in the source is escaped, not interpreted.
- The math and diagram pre-pass runs before conversion and splits the field; only literal-text segments are converted, so placeholder markup is never parsed as Markdown.
- No client-side Markdown conversion exists.

## Math and diagram pre-pass

- The server performs a non-Markdown pre-pass over displayed prose fields. It interprets math and diagram delimiters only; headings, lists, tables, emphasis, links, and raw HTML stay literal text.
- `$$...$$` is display math; `$...$` is inline math, where a backslash-escaped `\$` is a literal dollar sign and never opens math.
- A fenced block with info string `math`, `tex`, or `latex` is display math.
- A fenced block with info string exactly `mermaid` is a diagram. Matching is case-sensitive, trailing whitespace is trimmed, zero to three leading spaces are allowed, at least three backticks or tildes open the fence, and opener and closer use the same character.
- A fenced block with info string `plantuml` or `puml` is source text with a note.
- `\(...\)` and `\[...\]` are not recognized and stay literal. `mmd`, `uml`, `dot`, `graphviz`, and every other unknown info string stay literal text with their fence lines included.
- An empty or whitespace-only `mermaid` fence is literal text, not a figure.
- Nothing about TeX or Mermaid syntax is parsed on the server; for diagrams the server counts bytes and lines only.

## Placeholders

The pre-pass splits a field into an ordered list of literal-text and math or diagram segments:

- Inline math: `<span class="math" data-tex="…">` with the escaped source as its inner content.
- Display math: `<div class="math math--display" data-tex="…">` with the same inner content.
- A `mermaid` fence: `<figure class="diagram bleed" data-diagram="mermaid">` containing `<div class="diagram-scroll" role="region" aria-label="Diagram N" tabindex="0"><pre>escaped source</pre></div>`, where `N` is a 1-based figure number unique across the page.
- A `plantuml` or `puml` fence: `<figure class="diagram diagram--unsupported" data-diagram="plantuml">` with `<pre>escaped source</pre>` and a `figcaption` of class `render-note` reading `PlantUML diagrams are not supported in this release; showing source.` The block is server-rendered, links no script, and does not change the CSP class.

A field with no math and no diagram produces byte-identical output. Rendering applies to curated prose fields only: for document records, every resolved field whose kind is `text` or `multiline` and whose value is present. IDs, titles, labels, metadata rows, list cells, and form inputs stay literal.

## Source fallback

- The escaped source text is the element's initial content, so the page is correct with JavaScript disabled, when an asset fails to load, and when a library rejects the input.
- A successful render replaces that content.
- KaTeX and Mermaid failure handling is specified by their asset specifications.

## Asset inclusion

- The KaTeX stylesheet and script are linked only on pages whose pre-pass found at least one math segment.
- The Mermaid bundle is linked only on pages with at least one renderable `mermaid` figure.
- A page with neither gains no vendor bytes; a page with only an over-limit or PlantUML block gets neither.
- All scripts are classic and deferred, loaded in the order KaTeX stylesheet, KaTeX script, Mermaid script, `app.js`.

## Server-side rendering

- HTML is rendered server-side from templates embedded with `include_str!` from `apps/browser/src/templates/*.html`.
- Placeholder syntax is fixed: `{{name}}`. Substitution is a single pass over the template; substituted values are never re-scanned, so a value containing `{{...}}` is emitted literally, and an unknown placeholder is left untouched.
- There is no template engine and no expression evaluation.
- Substituted values are escaped at render time. Internal fragments — sidebar, breadcrumb, section, metadata, form — are pre-escaped raw fragments; no other raw insertion exists.
- JSON responses are produced from the same record access as the HTML routes.
- Markup and assets target Firefox 115 and later. Every page works without JavaScript; the dirty-form guard and the sidebar toggle are optional.

## Rendered fields and surfaces

- Rendering applies to prose fields only and introduces no separate record-body view (REQ-006.017).
- Rendering is browser-only: command-line output and every other surface show the source text unchanged (REQ-006.017).

## Rendering constraints

- Rendering works fully offline: all rendering assets are bundled, and no network request, external service, or server-side rendering is required (REQ-006.017).
- Rendered output conforms to the supported browsers and must not result in arbitrary script execution (REQ-006.017).
- Unrenderable input — an unclosed delimiter, an unparseable formula, or an invalid diagram — is shown as source text with a note instead of breaking the page (REQ-006.017).
- No TeX engine, JVM, `plantuml.jar`, or rendering service exists in any code path.
