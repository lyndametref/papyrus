---
id: TASK-140
title: Render math formulas with vendored KaTeX in curated prose fields
status: Done
assignee:
  - @rust-dev
created_date: '2026-11-15T16:13:00Z'
updated_date: '2026-11-16T08:20:00Z'
labels:
  - browser
  - rendering
  - security
milestone: MS-022
dependencies:
  - TASK-136
  - TASK-138
  - TASK-139
references:
  - docs/requirements/business/REQ-06.017.md
  - docs/architecture/adr/ADR-0025-bundled-katex-client-side-math-rendering.md
  - >-
    docs/architecture/adr/ADR-0027-vendored-embedded-browser-assets-and-packaging-notices.md
  - >-
    docs/architecture/adr/ADR-0005-embedded-dependency-free-http-server-and-server-rendered-ui.md
  - >-
    docs/architecture/adr/ADR-0028-browser-shell-shared-list-layout-content-width-and-collapsible-navigation.md
  - docs/specifications/browser-math-and-diagram-rendering.md
  - docs/specifications/browser-shell-shared-list-layout-and-page-width.md
  - src/browser/render.rs
  - src/browser/http.rs
  - src/browser/shell.rs
  - src/browser/templates/shell.html
  - src/browser/app.js
  - src/browser/app.css
  - src/browser/assets.rs
  - src/browser/mod.rs
priority: high
ordinal: 41580
---


## Description
**Why.** REQ-06.017 requires browser-side math rendering that works fully offline, keeps CLI output as source text, and degrades to source instead of breaking the page. ADR-0025 is Accepted: vendored KaTeX 0.16.11 (shipped by TASK-143), a non-Markdown delimiter pre-pass in `browser::render`, and a placeholder whose initial content is the escaped source.

**Authority.** REQ-06.017; ADR-0025 decisions 2-10; the feature specification `docs/specifications/browser-math-and-diagram-rendering.md` §2-§9, which is the canonical contract for the field map, the delimiter edge rules, the markup, and the CSP. That page is deliberately separate from `docs/specifications/browser-shell-shared-list-layout-and-page-width.md`: TASK-144 owns that page's §3/§4/§7 and is editing the shell files, so this ticket references it and must not edit it.

**What.** The delimiter pre-pass in `browser::render`, the KaTeX client pass, conditional asset emission, the `font-src` CSP delta, and the tests that pin the fallback and the no-Markdown boundary.

**1. Rendered field set (curated prose only)**

The pre-pass runs on the value of exactly these `Record` fields, where `view_page` or `document_detail_page` renders them today:

- Requirement: `story` (when `supports_story()`), `description`
- Task: `description`
- Milestone: `description`
- Release: `description`
- Decision: `decision`, `context`, `consequences`
- Glossary: `definition`
- UiSpec: `view`, `description`
- Rule: `description`, `rationale`, `applicability`
- Audit: `description` (the `--content` body)
- TechSpec: `description` (body text)
- UseCase / TestCase / TestReport: `description`, `applicability`
- `None` handler (config-defined document type): `description`
- Generic document page: the `description` paragraph of `render::document_detail_page` (chosen by `Record::is_generic()`)

Everything else keeps today's `escape()` behaviour: IDs, titles, labels, statuses, assignees, milestone/release IDs, decision type, the supersession line, audit type; every list cell and board card; the `record_view_extra_rows` metadata rows; acceptance criteria, comments, dependencies, requirements, refs, specs and reference `<details>` lists; and all edit-page form controls (edit pages never run the pre-pass and never emit math assets).

ADR-0025 decision 8 lists `content`, but `Record` has no `content` field. Read it as the Decision family's decision text (`Record::decision`) plus the Audit `--content` body (`Record::description`); both are included, and the specification records the reconciliation.

"Generic document body text" means exactly one call site: the description paragraph of `render::document_detail_page`. No record-body view exists or is added.

**2. Delimiter contract**

One left-to-right scan per field value. At each position try, in order: a fenced opener at a line start, then `$$`, then `$`. Everything not consumed is literal text. A candidate that fails any rule stays literal, with no error marker and no note, and scanning resumes one character later.

Three forms only:

- `$$...$$` display: content may span lines; trimmed of leading/trailing whitespace before it becomes `data-tex`.
- `$...$` inline: must close before the next line break; content non-empty; first and last content characters non-whitespace; never trimmed.
- Fenced block whose info string is exactly `math`, `tex`, or `latex` (case-sensitive): opener starts a line, 0-3 leading spaces, at least three identical backticks or tildes; closer is a later line of the same character at least as long, whitespace only after it; content trimmed. `Math`, `math,`, `math title`, and a 4-space-indented fence are not recognized.

Edge rules, each unit-tested:

- `\$` in literal text emits `$` and never opens math. A `$` is escaped when the run of consecutive backslashes immediately before it is odd, so `\\$` does not escape; when a `$` is escaped, exactly one backslash is removed from that run (`\$` -> `$`, `\\\$` -> `\\$`).
- Inside a math segment `\$` is TeX input and is not rewritten by the server.
- `\(...\)` and `\[...\]` stay literal. No other backslash sequence is interpreted.
- Unclosed `$$`, `$`, or fence: literal, delimiters included.
- Empty content after trimming (`$$$$`, `$$ $$`, an empty `math` fence): literal, delimiters and fence lines included.
- A fence with any other info string is opaque literal text: its `$`, `$$`, and `\$` are never scanned. An unclosed non-math fence is literal text scanned normally.
- `$5 and $10` stays literal: the candidate's closing `$` would follow a space, so the opener is rejected.
- Display math may start mid-line and always emits a block element.

**3. Output markup and fallback**

- Inline: `span.math[data-tex]` whose attribute and text content are the source.
- Display and fenced: `div.math.math--display[data-tex]` likewise.
- Both use the source passed through `render::escape`, so `&`, `<`, `>`, `"`, `'` are entities in the attribute and in the initial text.
- A field's HTML is: literal runs and inline segments inside `<p class="text"{attrs}>`, keeping each call site's existing attribute suffix (`""` or ` style="white-space: pre-wrap"`); a display segment ends the paragraph and emits the div, so no block element is nested inside a `<p>`; empty literal runs around display blocks are omitted.
- A field with no display segment is exactly one paragraph and is byte-identical to today, except that `\$` becomes `$`.
- The element's own source is the fallback: correct with JavaScript disabled, with a failed asset request, and when KaTeX rejects the input.

**4. Page integration and conditional assets**

- New `src/browser/math.rs` (declared `pub mod math;` in `src/browser/mod.rs`) owns `Prose { html, has_math }`, `prose(value, attrs) -> Prose`, and a `HEAD_ASSETS` constant holding the two KaTeX tags. It knows nothing about records or routes.
- `view_page` and `document_detail_page` accumulate `has_math` across their prose fields and pass `math::HEAD_ASSETS` or `""` to the page wrapper. `render::page` keeps its signature and forwards `""`; a sibling `page_with_assets` carries the fragment, so the other page builders are untouched.
- `shell::Page` gains `assets: &'a str`; `templates/shell.html` gains one `{{asset_links}}` slot on its own line after the `/assets/app.css` link and before the `/assets/nav-state.js` and `/assets/app.js` tags. The deferred `katex.min.js` must precede the deferred `app.js` in document order.
- KaTeX CSS and script appear exactly once on a page whose pre-pass found a math segment and on no other page class: list, board, browser inventory, lifecycle, configuration, skills, cleanup, error, edit, or a record view without math.

**5. Client pass**

One block at the end of the existing `app.js` IIFE, in the file's ES5 style, with no new global: select `[data-tex]`; `displayMode` is true exactly when the element has `math--display`; call `katex.render(source, element, { throwOnError: false, displayMode, trust: false, strict: "warn", output: "html" })`. A thrown error sets `data-render-error="math"` on that element and the remaining elements still render; a missing `window.katex` sets the marker on every `[data-tex]` element and changes no content; with `throwOnError: false`, KaTeX's own error output counts as success and sets no marker. No `fetch`, no `import()`, no `MutationObserver`, no retry, no second pass.

**6. CSP**

`http::CSP` becomes exactly:

`default-src 'none'; style-src 'self'; script-src 'self'; img-src 'self'; font-src 'self'; form-action 'self'; base-uri 'none'; frame-ancestors 'none'`

`font-src 'self'` is the only addition; `script-src` and `style-src` stay `'self'`; no directive gains `unsafe-inline`. The existing `text/html` emission gate is unchanged. Tests assert the literal string (not the constant) for the board, a list page, a record view with math, a record view without math, an edit page, and an error page, and assert no CSP header on `/assets/app.css`, `/assets/katex/katex.min.css`, `/assets/katex/katex.min.js`, and one woff2 file.

**7. Security and the no-Markdown boundary**

- `trust: false` and no option that enables `\href`, `\url`, `\includegraphics`, `\htmlClass`, or `\htmlData`. TeX reaches the client only as an escaped attribute.
- Hostile fixtures (`\href{javascript:alert(1)}{x}`, `\url{https://example.test}`, `\includegraphics{https://example.test/x.png}`, `<script>alert(1)</script>`, `"><img src=x onerror=alert(1)>`) must leave no `<script`, no `javascript:`, and no `http://`, `https://`, or `//` reference introduced by record data; the payload appears only as escaped text inside `data-tex` and the element body.
- No Markdown interpretation beyond the math delimiters: headings, lists, tables, emphasis, links, inline code, block quotes, and raw HTML stay literal; a non-math fence stays literal including its fence lines.
- Rendering is a display concern only: no write path, patch, front matter, or search index knows about math.

**Out of scope.** Diagram fences (TASK-146); a record-body view; server-side TeX; any Rust crate; trusting user commands; rendering in list cells, metadata rows, titles, or form inputs; `style-src` or `img-src` changes; `.bleed` for math; widening `.text`.

**Assumptions.** ADR-0025 decisions 2-10 are the contract. TASK-143's KaTeX files, paths, and digests are final, and no new vendored file or asset-table entry is needed. The vendored `katex.min.js` is the upstream UMD classic script exposing `window.katex`. The `$...$` heuristic is documented in the README by TASK-148. Mermaid and `style-src` changes are not part of this task.

**Dependencies and sequencing.** TASK-141 and TASK-143 are completed. TASK-144 (same milestone) is rewriting `shell.rs`, `templates/shell.html`, `app.js`, `app.css`, and `assets.rs`; this ticket edits the same files, so it must land after TASK-144 merges - that is why it now depends on TASK-144. TASK-146 extends the same pre-pass.

**Open questions (none block implementation).** 1. `style-src 'self'` drops KaTeX's inline style attributes (`mspace` widths, the rejected-formula colour), so operator spacing can deviate slightly and a rejected formula shows uncoloured; accept for 1.0 or revisit together with TASK-146's Mermaid `style-src` decision? 2. `output: 'html'` (ADR-0025 decision 5) omits KaTeX's MathML, which is the assistive-technology path; changing it needs an ADR amendment, not a ticket change.

## Acceptance Criteria
- [x] Record views render `$$...$$` display math, `$...$` inline math, and `math`/`tex`/`latex` fenced math as `span.math[data-tex]` or `div.math.math--display[data-tex]` in the curated prose fields listed in the description, and in no other field, list cell, metadata row, title, label, or form control.
- [x] `\$` in literal text renders as a literal `$` and never opens math; an inline candidate is rejected when it has no closing `$` before the next line break, empty content, or a whitespace first or last content character, and every rejected candidate stays literal text with no error marker.
- [x] Each math element's initial content is its own escaped source, identical to its `data-tex`, so the served HTML alone is correct with JavaScript disabled, when the KaTeX asset fails to load, and when KaTeX rejects the formula.
- [x] The KaTeX stylesheet and script are linked exactly once, before `/assets/app.js` in document order, on a page whose prose contains a math segment, and never on a list, board, inventory, lifecycle, configuration, skills, cleanup, error, or edit page, or on a record view without math.
- [x] The pre-pass interprets no Markdown construct beyond the three math forms: headings, lists, tables, emphasis, links, inline code, block quotes, raw HTML, and non-math fenced blocks stay literal, HTML-escaped text.
- [x] The served CSP is exactly `default-src 'none'; style-src 'self'; script-src 'self'; img-src 'self'; font-src 'self'; form-action 'self'; base-uri 'none'; frame-ancestors 'none'` on every HTML response, the literal string is asserted for list, board, record view with math, record view without math, edit, and error responses, non-HTML asset responses carry no CSP header, and `unsafe-inline` appears in no directive.
- [x] A thrown `katex.render` error sets `data-render-error="math"` on that element only, a missing `window.katex` sets it on every `[data-tex]` element, and in both cases the source text stays in place and the rest of the page still works.
- [x] `papyrus <slug> view`, `papyrus search`, and the stored record files keep the math source text byte-for-byte, and search still matches a term that appears inside a formula.
- [x] Hostile TeX and HTML fixture input (`\href`, `\url`, `\includegraphics`, `<script>`, an event-handler payload) produces no `<script` element, no `javascript:` URL, and no remote reference introduced by record data in the served HTML, the payload appears only as escaped text, and no KaTeX option enables `trust`, `\href`, or `\includegraphics`.

## Definition of Done
- [x] #1 `cargo test` passes, including the new `src/browser/math.rs` unit tests and `tests/browser_math_cli.rs`.
- [x] #2 `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` pass.
- [x] #3 `Cargo.toml` is unchanged: no new Rust crate, no build step, no generated asset.
- [x] #4 No new vendored file and no new `assets.rs` entry; the KaTeX tags reference only the TASK-143 paths under `/assets/katex/`.
- [x] #5 `http::CSP` equals the exact string in the description, and at least one test asserts the literal value rather than the constant.
- [x] #6 `docs/specifications/browser-math-and-diagram-rendering.md` matches the implementation; `browser-shell-shared-list-layout-and-page-width.md` is unmodified.
- [x] #7 No Markdown interpretation was added: no heading, list, table, emphasis, link, code, or raw-HTML path exists in the pre-pass, and a test proves those constructs stay literal.
- [x] #8 The JavaScript-disabled fallback is proven from the served HTML for all three math forms, and `data-render-error="math"` is covered by a test or a documented app.js assertion.
- [x] #9 CLI output, search output, and stored file bytes for math records are unchanged, asserted by tests.
- [x] #10 Implementation notes and the final summary are recorded in this ticket, and the specification page is listed in the references.
- [x] #11 Markup stability holds: every existing prose assertion passes unchanged, and the only permitted byte changes on math-free pages are the CSP string, `\$` sequences, and the generic-document `<p>` to `<p class="text">` wrapper recorded in the specification §4.3.


## Implementation Notes
1. Add `src/browser/math.rs` and declare `pub mod math;` in `src/browser/mod.rs`: `Prose { html, has_math }`, `prose(value, attrs) -> Prose`, `HEAD_ASSETS` (`<link rel="stylesheet" href="/assets/katex/katex.min.css">` then `<script src="/assets/katex/katex.min.js" defer></script>`). Implement the §2 scanner (fence opener at line start, then `$$`, then `$`; backslash-parity escape handling; literal fallback on any rejected candidate) and the §3 block assembly. Unit tests for every rule in the ticket and in the specification §3.6.

2. `src/browser/render.rs`: replace the prose call sites from description §1 with `math::prose(value, attrs)`; keep the `empty-text` placeholders and the `specs`/`refs` paragraphs unchanged. Accumulate `has_math` in `view_page` and in `document_detail_page`; add `page_with_assets(chrome, slug, title, sidebar, body, assets)` and keep `page(...)` forwarding `""`. In `document_detail_page`, change the description paragraph to `<p class="text">` so the shared path applies.

3. `src/browser/shell.rs` + `src/browser/templates/shell.html`: add `Page::assets: &'a str` and one `{{asset_links}}` slot on its own line after the `/assets/app.css` link and before the `/assets/nav-state.js` and `/assets/app.js` tags; fill it in `shell::render` from the raw placeholder list. Extend the shell unit tests for the empty default.

4. `src/browser/app.js`: append one ES5 block at the end of the existing IIFE implementing description §5. No new global, no new event listener, no retry.

5. `src/browser/app.css`: minimal rules for `.math`, `.math--display` (block box, `overflow-x: auto`, neutralise inherited `white-space: pre-wrap`, readable fallback). No `.bleed`, no change to `.text`.

6. `src/browser/http.rs`: set `CSP` to the exact string in description §6.

7. New `tests/browser_math_cli.rs` in the style of `tests/browser_shell_cli.rs`, plus a fixture project carrying math in a requirement description, a rule rationale, a decision context, a glossary definition, a task description, and a generic document body, one math-free record, one hostile record, and one Markdown-heavy record. Cover the test plan below.

8. Update the exact-CSP assertions in `tests/browser_shell_cli.rs` (and any other literal CSP assertion) to the new literal string.

9. Keep `docs/specifications/browser-math-and-diagram-rendering.md` §2-§9 accurate; record any deviation in the ticket notes. Do not edit `browser-shell-shared-list-layout-and-page-width.md`.

**Test plan**

Unit, `src/browser/math.rs`:
- inline: basic; opens on whitespace; closes on whitespace; contains a newline; unclosed; empty content.
- display: basic; multi-line; trimmed `data-tex`; unclosed; `$$$$` and `$$ $$`; mid-line start; block ordering against surrounding literal runs.
- fences: `math`, `tex`, `latex`; `~~~` opener; longer closer; 0-3 leading spaces; 4-space indent rejected; `Math`/`math,`/`math title` rejected; non-math fence opaque (`$x$`, `\$` untouched); unclosed fence literal; empty fence literal.
- escapes and literals: `\$` collapse; `\\$` parity; `\$` preserved inside a math segment; `\(...\)`, `\[...\]`, `\*`, `\_` literal.
- markup: attribute and text escaping of `<`, `>`, `&`, `"`, `'`; `has_math` false for math-free input; single-paragraph byte-identity for math-free input including the empty value; multiple `<p>` blocks around display segments and no `<p>` containing a `<div>`.
- heuristic: `$5 and $10` literal.

Integration, `tests/browser_math_cli.rs`:
- each family's prose field renders `class="math"` / `class="math math--display"` with `data-tex`; the same records' titles, labels, list rows, metadata rows, acceptance criteria, comments, and references contain no `data-tex`.
- a record view with math links `/assets/katex/katex.min.css` and `/assets/katex/katex.min.js` exactly once, and `katex.min.js` appears before `/assets/app.js` in the document.
- list page, board, inventory, lifecycle, configuration, skills, cleanup, error page, edit page, and math-free record view contain neither KaTeX URL.
- `/assets/katex/katex.min.css`, `/assets/katex/katex.min.js`, and one font return 200 with their expected content types.
- JavaScript-disabled fallback: the served HTML carries the source as both `data-tex` and element text for every delimiter form.
- CSP: literal string on board, list, math view, math-free view, edit, and error; no header on the four asset responses; no `unsafe-inline` and no inline `<script>` anywhere.
- CLI and search unchanged: `papyrus <slug> view` prints `$$...$$` and `\$` verbatim; `papyrus search` matches a term inside a formula; the stored file bytes are unchanged.
- no Markdown: a field with headings, a table, emphasis, a link, inline code, and raw HTML renders them as escaped literal text.
- hostile fixtures: no `<script`, no `javascript:`, no `http://`/`https://`/`//` from record data, payload only inside escaped `data-tex` and element text.


## Notes
src/browser/math.rs implements one left-to-right pre-pass over curated prose fields: display double-dollar, inline single-dollar, and math/tex/latex fences, with odd-backslash dollar escaping, inline newline/non-space delimit rules, opaque non-math fences, and literal fallback for unbalanced or empty delimiters. Output is span.math[data-tex] and div.math.math--display[data-tex] with the escaped source as attribute and initial text, so the no-JS and failed-asset states show the source. KaTeX CSS and JS are emitted once, only on pages whose pre-pass found math, after app.css and before app.js; the client pass runs htmlAndMathml with trust false, strict warn, throwOnError false. CSP gains exactly font-src self and is asserted literally per response class, absent on assets. Review found and fixed one real fallback bug: kaTeX clears the element before parsing and rethrows non-parse errors, so the catch now restores the captured source before setting data-render-error=math (proved in a Node VM harness with a throwing stub: both placeholders kept their source and carried the marker). 1011 tests pass; clippy and fmt clean. Runtime rendering in real browsers remains a TASK-150 verification item.


## Comments
author: orchestrator
created: 2026-11-16T07:00:00Z

---

Open questions settled by ADR-0025 amendment of 2026-11-16 (owner acknowledgement pending, status unchanged): (1) KaTeX output is htmlAndMathml, not html. Reason: the vendored 0.16.11 bundle sets aria-hidden=true on .katex-html, so a successfully rendered formula under output html would expose nothing to assistive technology, which is worse than the JS-disabled state where the escaped source is read. MathML is built client-side, so there is no payload or asset cost. Implementation contract: katex.render(source, element, { throwOnError: false, displayMode, trust: false, strict: 'warn', output: 'htmlAndMathml' }) and the two sibling trees must not be added to, stripped, or reordered. (2) style-src stays 'self' with no nonce, hash, 'unsafe-hashes', or 'unsafe-inline'. KaTeX marks a few nodes with inline style attributes (mspace widths, rejected-formula colour); those are dropped by the browser, which is an accepted cosmetic deviation. Nonces cannot address style attributes, hashes are unbounded per formula, and 'unsafe-inline' would relax the whole page for a sub-em spacing delta. Math must render correctly with style-src 'self' alone; TASK-146's Mermaid relaxation must not be assumed by math pages. Follow docs/specifications/browser-math-and-diagram-rendering.md and the amended ADR-0025 rather than this ticket's earlier open-question text.

---

author: orchestrator
created: 2026-11-16T08:18:00Z

---

Ticket text alignment after the ADR-0025 amendment of 2026-11-16: the description's 'output: html' wording and open question 2 are superseded. The implementation, the amended ADR, and docs/specifications/browser-math-and-diagram-rendering.md pin output 'htmlAndMathml' and keep style-src 'self' strict, with the inline-style loss accepted as a cosmetic deviation. Follow the spec, not the older description text.

---

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Curated prose fields now render TeX through one pre-pass that emits source-preserving placeholders: display blocks, inline formulas and math fences become elements whose initial content is the escaped TeX, so a page without JavaScript or with a failed asset still shows the formula source. KaTeX renders those placeholders in the browser only when a page actually contains math, with the stylesheet and script emitted once and the library's own output split into an accessible MathML tree and a visual HTML tree. Malformed input keeps KaTeX's error output, a thrown render restores the source and marks the element, and one failing formula never stops the others. No Markdown is interpreted beyond math, hostile TeX cannot introduce scripts or remote references, the content security policy gains only font-src self, and CLI output plus stored bytes are unchanged.
