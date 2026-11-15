---
id: TASK-145
title: Render math formulas with vendored KaTeX in curated prose fields
status: To Do
assignee:
  - '@rust-dev'
created_date: '2026-11-15T16:13:00Z'
labels:
  - browser
  - rendering
  - security
milestone: m-21
dependencies:
  - TASK-141
  - TASK-143
references:
  - docs/requirements/business/REQ-06.017.md
  - docs/architecture/adr/ADR-0025-bundled-katex-client-side-math-rendering.md
  - src/browser/render.rs
  - src/browser/http.rs
  - src/browser/app.js
priority: high
ordinal: 41580
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why.** REQ-06.017 requires browser-side math rendering that works fully offline, keeps CLI output as source text, and degrades to source instead of breaking the page. ADR-0025 is Accepted: vendored KaTeX, a non-Markdown delimiter pre-pass, and a placeholder whose initial content is the escaped source.

**What.** The delimiter pre-pass in `browser::render`, the KaTeX client pass, conditional asset emission, the `font-src` CSP delta, and the tests that pin the fallback and the no-Markdown boundary.

**In scope.**
- Pre-pass over the curated prose fields only (`description`, `story`, `rationale`, `applicability`, `definition`, `view`, `context`, `consequences`, `content`, and the generic document body text). IDs, titles, labels, list cells, metadata rows, and form inputs stay literal.
- Exactly three forms: `$$...$$` display, `$...$` inline, and a fenced block with info string `math`, `tex`, or `latex` display. `\$` is a literal dollar and never opens math; inline math must open and close on a non-space and may not contain a newline; `\(...\)` and `\[...\]` stay literal.
- Output segments: literal text escaped exactly as today, and `span.math[data-tex]` or `div.math.math--display[data-tex]` whose inner content is the escaped source, so the fallback exists without JavaScript.
- No other Markdown interpretation: headings, lists, tables, emphasis, links, and raw HTML stay literal (ADR-0005 and ADR-0025 decision 2).
- Link KaTeX CSS and script only on pages whose pre-pass found at least one math segment; classic deferred script from the TASK-143 asset table; no module loading, fetch, or dynamic import.
- Client pass with `throwOnError: false`, `trust: false`, `strict: 'warn'`, `output: 'html'`, `displayMode` per segment; a thrown error or missing global leaves the source and sets `data-render-error="math"`.
- CSP delta is exactly `font-src 'self'`; `script-src` stays `'self'`; the exact CSP string is asserted per response class.
- Tests: each delimiter form, the `\$` escape, the inline heuristics, fenced fallback, malformed and unparseable input, JavaScript-disabled source display, no math assets on pages without math, no Markdown interpretation, CLI and search output unchanged, and hostile input (`\\href`, `\\includegraphics`, raw HTML) producing no script and no remote reference.

**Out of scope.** Diagram fences (TASK-146); a record-body view; server-side TeX; any Rust crate; trusting user commands; rendering in list cells or metadata.

**Assumptions.** ADR-0025 decisions 2 to 10 are the contract. The `$...$` heuristic is documented in the README by TASK-148. Mermaid and `style-src` changes are not part of this task.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Display math, inline math, and fenced math blocks render in the curated prose fields of record views.
- [ ] #2 A backslash-escaped dollar sign is literal and never opens math, and inline math follows the non-space and no-newline rules.
- [ ] #3 The placeholder carries the escaped source as its initial content, so the page is correct with JavaScript disabled, a missing asset, or rejected input.
- [ ] #4 Math assets load only on pages that contain a math segment, and no other page gains bytes.
- [ ] #5 The pre-pass interprets no Markdown construct beyond the math delimiters; headings, lists, tables, emphasis, links, and raw HTML stay literal.
- [ ] #6 CSP adds exactly font-src self, script-src stays self, and the exact CSP string is asserted per response class.
- [ ] #7 A thrown render error or missing global leaves the source text and sets the math render-error marker.
- [ ] #8 CLI, search, and record files keep the source text unchanged.
- [ ] #9 Hostile input cannot produce a script node or a remote reference through KaTeX.
<!-- AC:END -->
