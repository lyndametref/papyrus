# Feature Specification — Browser Math and Diagram Rendering (TASK-145, TASK-146)

**Tickets:** TASK-145 (§1–§11 — math, KaTeX) delivered here; TASK-146 extends §2 (scope enumeration) and adds the diagram sections on top of the same pre-pass
**Milestone:** m-21
**Status:** Contract pinned 2026-11-16 for TASK-145 from REQ-06.017 and ADR-0025 decisions 2–10; revised 2026-11-16 with the ADR-0025 decisions 5–6 revision notes (`output` pinned to `htmlAndMathml`, `style-src` inline-style deviation accepted). TASK-146 remains gated by ADR-0026 acceptance
**Requirements:** REQ-06.017 (math and diagrams), REQ-06.015 (content width), NFR-01.003 (offline), NFR-02.002 (sanitize input), NFR-03.001 (single binary), NFR-05.001 (supported browsers), NFR-06.002 (GPLv3-or-later linking), REQ-06.004 (browser/CLI parity)
**ADRs:** ADR-0025 (Accepted — the contract for this page), ADR-0027 (vendored assets and notices), ADR-0005 (CSP baseline, no Markdown renderer), ADR-0028 (width classes; `.bleed` is a TASK-146 consumer only)
**UI:** no mockup. TASK-145 adds no control, page, or layout region: math is content inside existing prose containers, and the DOM shape is fixed by ADR-0025 decision 2. Visual verification is browser verification in TASK-149/TASK-150. Recorded in §11.
**Downstream:** TASK-146 (diagrams), TASK-148 (specification port into `docs/specifications/browser-requirements*.md` and packaging wording), TASK-149 (packaged assets, offline rendering, binary budget), TASK-150 (release verification)

## 1. Purpose and scope

REQ-06.017 requires the local browser to render math formulas in the curated prose fields that record views already display, fully offline, with all assets bundled, without arbitrary script execution, and with the source text as the fallback. ADR-0025 fixes the mechanism: a non-Markdown delimiter pre-pass on the server that emits placeholders, KaTeX rendering in the client, and a `font-src 'self'` CSP delta.

This page is the canonical detail for the pre-pass, the delimiter contract, the placeholder markup, the conditional asset emission, the client pass, and the CSP delta. It is deliberately separate from `browser-shell-shared-list-layout-and-page-width.md` (TASK-141/TASK-144 own the shell and width contract and are being edited concurrently); this page references the shell contract and records its own delta to it in §5.3. TASK-148 ports the normative clauses into the browser requirement specifications; this page must then be cited and superseded, not duplicated.

### Non-goals

- Diagram fences (`mermaid`, `plantuml`, and the other unsupported languages) — TASK-146, on the same pre-pass.
- A record-body view. No page renders a raw record body; see §2.3 for the one body-text path that exists.
- Server-side TeX, a Rust crate, a build step, or a network fetch of any kind.
- Markdown interpretation of any kind (headings, lists, tables, emphasis, links, raw HTML) — ADR-0005, ADR-0025 decision 2.
- Rendering in list cells, metadata rows, titles, IDs, labels, or form inputs.
- Changes to `style-src`, `img-src`, `script-src`, or any CSP directive other than the single `font-src 'self'` addition.
- Widening the `.text` 780 px measure, or introducing `.bleed` for math (REQ-06.015 full width is for diagrams/visualizations, TASK-146).

## 2. Rendered field set

### 2.1 Curated prose fields, by family

The pre-pass runs on the value of exactly these `Record` fields where `view_page` or `document_detail_page` renders them today. Every other escape call in `browser::render` keeps the current `escape()` behaviour.

| Family (`TypedHandler`) | `Record` field | Rendered section |
|---|---|---|
| Requirement | `story` (when `supports_story()`) | User story |
| Requirement | `description` | Description |
| Task | `description` | Description |
| Milestone | `description` | Description |
| Release | `description` | Description |
| Decision | `decision` | Decision |
| Decision | `context` | Context |
| Decision | `consequences` | Consequences |
| Glossary | `definition` | Definition |
| UiSpec | `view` | View |
| UiSpec | `description` | Description |
| Rule | `description` | Description |
| Rule | `rationale` | Rationale |
| Rule | `applicability` | Applicability |
| Audit | `description` (the `--content` body) | Body paragraph |
| TechSpec | `description` | Body paragraph |
| UseCase / TestCase / TestReport | `description` | Body paragraph |
| UseCase / TestCase / TestReport | `applicability` | Applicability |
| None (config-defined document type) | `description` | Description |
| `document_detail_page` (generic document) | `description` | Description |

ADR-0025 decision 8 enumerates the same set. Two readings are pinned here because the ADR's wording is narrower than its intent:

- `content` is read as (a) the Decision family's decision text, stored in `Record::decision` and rendered under the “Decision” heading, and (b) the Audit `--content` body, stored in `Record::description`. These are the only prose bodies the ADR's word can denote; both are included.
- “Generic document body text” is the `description` paragraph rendered by `render::document_detail_page` — the page chosen by `Record::is_generic()` (`ContentModel::Generic`, i.e. a config-defined or generic document type). It renders `<p>{…}</p>` without the `.text` class today; §4.3 makes it a `.text` paragraph so the shared prose path applies.

### 2.2 Explicitly excluded values

These keep `escape()` only, on every page:

- `id`, `title`, `labels`, `status`, `assignee`, `milestone_id`, `release_id`, `decision_type`, `superseded_by` and the supersession line, `audit_type`.
- List cells and list geometry: every value that reaches `browser::list` (ID, title, status, family slots, created, updated, tags, references) and the board cards.
- Metadata rows built by `record_view_extra_rows` (the `.meta` `<dl>` block), including free-text-looking front-matter values.
- `acceptance_criteria`, `comments`, `dependencies`, `requirements`, `refs`, `specs`, and the reference `<details>` lists — item lists, not prose.
- Form controls on edit pages: `input`, `textarea`, `select`, tag inputs. Edit pages render source text in a control and never run the pre-pass and never emit math assets.

### 2.3 What "generic document body text" means in code terms

Exactly one call site: `render::document_detail_page`'s description paragraph. There is no other page that renders a raw record body, and TASK-145 must not add one.

### 2.4 Markup-stability guarantee

For a field value that contains no math construct (§3), `math::prose` output is byte-identical to today's `<p class="text"{attrs}>{escape(value)}</p>`, including the empty-value case, with one deliberate exception: a `\$` sequence in literal text becomes `$` (§3.5). Existing assertions such as `<p class="text" style="white-space: pre-wrap">…</p>` therefore keep passing unchanged.

## 3. Delimiter contract

### 3.1 Scanning model

One left-to-right scan of the field value. At each position the scanner tries, in order: (1) a fenced block opener at a line start, (2) `$$`, (3) `$`. Everything not consumed as math is *literal text*. There is no second pass, no Markdown parsing, and no look-behind other than backslash parity on `$`.

A candidate that fails any rule below is **not** an error: its delimiter characters stay literal and scanning resumes one character later, so the rest of the field is still considered the way a reader would read it. No marker, no note, no log line is produced for a rejected candidate.

### 3.2 Display math `$$…$$`

- Opens at any `$$` that is not preceded by an odd number of consecutive backslashes, and closes at the next `$$` under the same parity rule.
- The content may span lines. Nothing inside is scanned for any other construct.
- The content is trimmed of leading and trailing whitespace before it becomes `data-tex`.
- No close before the end of the value → the `$$` is literal text.
- Content empty after trimming (`$$$$`, `$$ $$`) → literal text, delimiters included.

### 3.3 Inline math `$…$`

An inline candidate opens at a single `$` that is not preceded by an odd number of consecutive backslashes and is not the first half of `$$`. It is accepted only when all of the following hold:

1. a closing `$` (same parity rule) exists before the next line break and before the end of the value;
2. the content is non-empty;
3. the first character of the content is not whitespace (`char::is_whitespace`);
4. the last character of the content is not whitespace.

Otherwise the `$` is literal and scanning resumes at the next character. Inline content is never trimmed — rules 3 and 4 already forbid padding — and is passed to `data-tex` exactly.

### 3.4 Fenced blocks with `math`, `tex`, or `latex`

- The opener must start a line (value start or immediately after `\n`), may be preceded by zero to three spaces, and is a run of at least three identical backticks or tildes followed by an info string.
- The info string is compared after trimming trailing whitespace, exactly and case-sensitively: `math`, `tex`, `latex`. `Math`, `math,` and `math title` are not recognized.
- The closer is a later line consisting of zero to three spaces, a run of the same fence character at least as long as the opener, and only whitespace to the end of that line.
- The content is the text between the opener line's newline and the closer line, with one trailing line break removed, then trimmed of leading and trailing whitespace. It becomes `data-tex`; it is display math.
- No opener match, or no closer, or empty/whitespace-only content → not a math segment. Unclosed opener → literal, scanning continues normally after the fence characters.
- A fenced block whose info string is anything else is **opaque literal text**: the fences, the content, and any `$`, `$$`, or `\$` inside are emitted as literal text and are never scanned. An unclosed non-math fence is literal text scanned normally.

### 3.5 Escapes and literal delimiters

- `\$` in literal text is emitted as `$` and never opens math. A `$` is escaped when the run of consecutive backslashes immediately before it is odd (so `\\$` has two and does not escape). When a `$` is escaped, exactly one backslash is removed from that run and the rest stay literal, so `\$` → `$` and `\\\$` → `\\$`.
- Inside a math segment, `\$` is part of the TeX source and is not rewritten by the server.
- `\(…\)` and `\[…\]` are not delimiters and stay literal text, unchanged and unescaped apart from HTML escaping.
- No other backslash sequence is interpreted. `\*`, `\_`, `\\` and the rest stay as written.

### 3.6 Edge-case table

| Value | Result |
|---|---|
| `$x$` | inline segment, `data-tex="x"` |
| `$ x$` | literal (opens on whitespace) |
| `$x $` | literal (closes on whitespace) |
| `$a\nb$` | literal (newline inside) |
| `$x` | literal (unclosed) |
| `$$x$$` | display segment, `data-tex="x"` |
| `$$\n a \n$$` | display segment, `data-tex="a"` |
| `$$x` | literal (unclosed) |
| `$$$$`, `$$ $$` | literal, delimiters included |
| `cost: \$5` | literal `cost: $5` |
| `$5 and $10` | literal, unchanged (the second `$` closes on a space, so the first candidate is rejected) |
| `a \\$x$ b` | literal `a \\` then inline `x` (two backslashes do not escape) |
| `\\\$y$` | literal `\\$y$` (three backslashes: one removed, the `$` escaped) |
| `\(x\)`, `\[x\]` | literal, unchanged |
| ```` ```math\nx^2\n``` ```` | display segment, `data-tex="x^2"` |
| `~~~tex\nx\n~~~` | display segment |
| ```` ```rust\n$x$\n``` ```` | literal block, `$x$` untouched |
| ```` ```math\n\n``` ```` | literal (empty content) |
| ```` ```math\nx^2 ```` (unclosed) | literal |

### 3.7 Accepted consequence

The single-`$` rule is a heuristic (ADR-0025 Consequences). The pinned rules above reject `$5 and $10` — the ADR's illustrative example — because the candidate closes on a space. The rules, not the illustration, are normative: a field is rendered as math only when it can be read unambiguously as `$…$`, and `\$` remains the documented escape hatch.

## 4. Output markup and fallback

### 4.1 Segment shapes

- Inline: `<span class="math" data-tex="SRC">SRC</span>`
- Display (from `$$` or a `math|tex|latex` fence): `<div class="math math--display" data-tex="SRC">SRC</div>`

`SRC` is the trimmed/exact TeX source of §3 passed through `render::escape` (so `&`, `<`, `>`, `"`, `'` are entities in both the attribute and the text node). The element's initial content is its own source, which is the fallback: the page is correct with JavaScript disabled, with a failed asset request, and when KaTeX rejects the input (ADR-0025 decision 3).

### 4.2 Paragraph blocks

A prose field renders as:

- one `<p class="text"{attrs}>…</p>` containing the literal runs and inline segments, when the value contains no display segment — byte-identical to today apart from the `\$` collapse (§2.4);
- otherwise a sequence of `<p class="text"{attrs}>…</p>` runs and `<div class="math math--display" data-tex="…">…</div>` blocks in source order, concatenated with no separator, where empty literal runs around or between display blocks are omitted.

A display block is never nested inside a `<p>` (block content inside a paragraph would be re-parented by the HTML parser and silently break both the paragraph and the fallback). `{attrs}` is the same attribute suffix each call site uses today (`""` or ` style="white-space: pre-wrap"`), so call sites keep their current markup.

### 4.3 Call-site placeholders

The `empty-text` placeholders (`No description.`, `(none)`, `No labels.`) and the `records.specs`/`records.refs` paragraphs are not prose and stay as they are. `document_detail_page` changes `<p>{…}</p>` to `<p class="text">{…}</p>` so the shared rules apply; that page has no math-free markup guarantee.

## 5. Page integration

### 5.1 Pre-pass module

New `src/browser/math.rs` (declared `pub mod math;` in `src/browser/mod.rs`), unit-tested in place:

```rust
pub struct Prose { pub html: String, pub has_math: bool }

/// Run the delimiter pre-pass over one curated prose value.
pub fn prose(value: &str, attrs: &str) -> Prose;

/// The head fragment for pages that contain math; `""` otherwise.
pub const HEAD_ASSETS: &str = "<link rel=\"stylesheet\" href=\"/assets/katex/katex.min.css\">\n<script src=\"/assets/katex/katex.min.js\" defer></script>";
```

`has_math` is true when the field produced at least one inline or display segment. The module knows nothing about records, pages, or routes; it is a pure string function plus the asset constant, so TASK-146 extends it with diagram fences.

### 5.2 Render call sites

Each row of §2.1 replaces `escape(value)` with `math::prose(value, attrs)` and `format!`-style `<p class="text"{attrs}>{}</p>` with the returned `html`. `view_page` and `document_detail_page` accumulate `has_math` across every field they render, then pass `math::HEAD_ASSETS` or `""` to the page wrapper.

`render::page` keeps its signature and forwards `""`; a sibling `page_with_assets(..., assets: &str)` carries the fragment so the ~40 non-prose pages are untouched.

### 5.3 Shell delta

`shell::Page` gains `pub assets: &'a str` (internal fragment, rendered as a raw placeholder; never user data). `templates/shell.html` gains one slot, `{{asset_links}}`, on its own line after the `/assets/app.css` link and before the `/assets/nav-state.js` and `/assets/app.js` script tags. `shell::render` fills it from `page.assets`.

The placement is normative: both KaTeX tags are `defer`/`stylesheet`, and the deferred `katex.min.js` must precede the deferred `app.js` in document order so `window.katex` exists when the client pass runs. This is the only change this ticket makes to the shell contract described in `browser-shell-shared-list-layout-and-page-width.md` §3; TASK-148 ports it into that page.

### 5.4 Conditional emission

The KaTeX stylesheet and script appear only on responses whose pre-pass found at least one math segment. No other page gains a byte:

- never on list pages, the board, the browser inventory, lifecycle, configuration, skills, cleanup, or error pages;
- never on edit pages;
- never on a record view whose prose fields contain no math construct;
- a page with math carries both tags exactly once.

## 6. Client render pass

CSS class and DOM handling belong to `app.css`/`app.js`, which TASK-144 is editing; TASK-145 adds one self-contained block at the end of the existing `app.js` IIFE, in the file's existing ES5 style, with no new global.

- Select every `[data-tex]` element after parsing; do nothing when the set is empty.
- `displayMode` is true exactly when the element carries the `math--display` class.
- Call `window.katex.render(source, element, { throwOnError: false, displayMode: displayMode, trust: false, strict: "warn", output: "htmlAndMathml" })`. `output` is pinned to exactly that string (ADR-0025 decision 5, revised 2026-11-16); `"html"` is not permitted.
- The rendered element then holds two sibling trees: `.katex-mathml` (built by KaTeX, clipped to a 1 px box by the vendored `katex.min.css`, exposed to assistive technology, carrying `<annotation encoding="application/x-tex">` with the same source as `data-tex`) and `.katex-html` (the visual tree, which KaTeX marks `aria-hidden="true"`). With `output: "html"` only the second exists, so after the source fallback is replaced the formula would be announced as nothing. Nothing in this ticket may add, strip, or reorder those two trees, and no further option (`macros`, `globalGroup`, `errorColor`, `trust`) is added.
- A thrown error sets `data-render-error="math"` on that element and leaves its existing content in place; other elements continue.
- A missing `window.katex` (script blocked or failed) sets `data-render-error="math"` on every `[data-tex]` element and changes no content.
- No `fetch`, no dynamic `import()`, no `MutationObserver`, no second pass, no state written to storage or the DOM beyond `data-render-error`.
- With `throwOnError: false` a malformed formula is rendered by KaTeX as its own error output (`errorColor`, offending source preserved inside the element); that counts as a successful call and does not set `data-render-error`. Only a thrown error or a missing global sets the marker.

## 7. CSP

### 7.1 Exact value

`http::CSP` becomes exactly:

```
default-src 'none'; style-src 'self'; script-src 'self'; img-src 'self'; font-src 'self'; form-action 'self'; base-uri 'none'; frame-ancestors 'none'
```

`font-src 'self'` is the only addition (ADR-0025 decision 6). `script-src` stays `'self'`; `style-src` stays `'self'`; no directive gains `'unsafe-inline'`, a nonce, a hash, or `'unsafe-hashes'`. This is the complete delta: the CSP string is identical on a math page and on a math-free page.

### 7.2 Per-response-class assertions

The header is emitted only when the content type starts with `text/html`; that gate is unchanged. Tests assert the literal string above — not the constant — for: the board (`/?type=task`), a list page, a record view with math, a record view without math, an edit page, and an error page. They assert the absence of `Content-Security-Policy` on `/assets/app.css`, `/assets/katex/katex.min.css`, `/assets/katex/katex.min.js`, and one font file, and the absence of `unsafe-inline` in the served HTML script elements.

### 7.3 Inline style attributes are dropped by design (settled 2026-11-16)

KaTeX sets inline `style` attributes on some generated nodes: `mspace` and rule geometry (width, height, vertical-align, border), `\color` output, and the `errorColor` of a rejected formula. With `style-src 'self'` and no `'unsafe-inline'`, the browser drops every one of them. This is the accepted contract (ADR-0025 decision 6, revised 2026-11-16), not a defect to fix:

- The visual effect is bounded: spacing around operators and rules can deviate by a fraction of an em, and a rejected formula shows uncoloured instead of `errorColor`. Text, structure, line breaking, horizontal containment, and the source fallback are unaffected.
- No execution path opens: an attribute the browser drops cannot restyle the page, run script, or fetch anything.
- A nonce does not address it: CSP nonces match `<style>` and `<link>` elements, never a `style` attribute, and KaTeX emits no style element for math.
- Hashes do not address it: the values are content-dependent (`mspace` widths derive from the formula), so the allowlist is unbounded, and `style-src-attr`/`'unsafe-hashes'` are unavailable on all of NFR-05.001's browsers (Firefox 100 lacks them).
- `'unsafe-inline'` in `style-src` is not taken: it relaxes every page for a cosmetic deviation. Mermaid's need for it (ADR-0026 decision 5) concerns an injected `<style>` element and is decided separately.

TASK-146 does not inherit this rule: it follows ADR-0026 decision 5, and this ticket's client pass must render correctly with `style-src 'self'` alone. TASK-149/TASK-150 record the observed deviation on a math page in all three supported browsers as expected behaviour, not as a failure.

## 8. Security and the non-Markdown boundary

- `trust: false` disables `\href`, `\url`, `\includegraphics`, `\htmlClass`, and `\htmlData`; no KaTeX option in this ticket may enable them. `output: 'htmlAndMathml'` and `strict: 'warn'` are the only other options. The MathML `<annotation encoding="application/x-tex">` carries the same escaped source as `data-tex`; it adds no new injection path, because KaTeX builds it from its own text nodes.
- The TeX source reaches the client only as an escaped attribute; KaTeX builds its own text nodes. A record cannot make KaTeX emit a `<script>`, an event-handler attribute, or a remote URL.
- Hostile fixtures must show, in the served HTML: no `<script` introduced by record data, no `javascript:` URL, no `http://`, `https://`, or `//` reference introduced by record data, and the payload present only as escaped text inside `data-tex` and the element body. Covered: `\href{javascript:alert(1)}{x}`, `\url{https://example.test}`, `\includegraphics{https://example.test/x.png}`, `<script>alert(1)</script>`, and `"><img src=x onerror=alert(1)>` inside math and as literal text.
- No Markdown interpretation: `#`, `##`, `-`, `1.`, `|`, `*`, `**`, `[text](url)`, `` `code` ``, `>` and raw HTML stay literal characters (escaped). A non-math fence stays literal including its fence lines.
- Command-line surfaces are untouched: `papyrus <slug> view`, `papyrus search`, and the stored files keep the source text byte-for-byte. Tests assert the source (`$$…$$`, `\$`) in CLI output and that search matches a term inside a formula.

## 9. Styling requirements

Behavioral, not prescriptive about selectors: a display block is a block-level box that scrolls horizontally inside its container instead of widening the page (the page must not gain a horizontal scrollbar at 320–1920 CSS px); a rendered formula does not inherit `white-space: pre-wrap` in a way that distorts spacing; fallback source text stays readable (newlines preserved for display math). `app.css` additions stay minimal and are made after TASK-144's CSS work lands. No `.bleed`, no change to `.text`'s 780 px measure. The visually hidden `.katex-mathml` box comes from the vendored `katex.min.css`; `app.css` adds no rule for it and must not make it visible.

## 10. Tests

Unit tests in `src/browser/math.rs` cover every row of §3.6 plus: attribute and text escaping, `\$` not rewritten inside a math segment, `has_math` false for math-free input, the single-paragraph guarantee and its byte-identity, block ordering, and the Markdown-literal set of §8.

Integration tests (`tests/browser_math_cli.rs`, in the style of `tests/browser_shell_cli.rs`, against a fixture project with math in each key family) cover: rendered markup per family and per form; exclusion from titles, labels, list cells, metadata rows, acceptance criteria, comments, and edit forms; assets present exactly once on math pages and absent on every other page class; script order (`katex.min.js` before `app.js`); CSP literal per response class and absent on assets; CLI and search output unchanged; hostile fixtures.

## 11. Resolved questions, assumptions, open questions

Resolved (2026-11-15 owner decisions, recorded in ADR-0025): delimiter set, curated-prose scope, no record-body view, no rendering in lists or metadata.

Resolved here by this page (implementation-level readings of the ADR, all testable):

1. `content` means the Decision-family decision text and the Audit content body (§2.1).
2. Rejected candidates are literal with no marker; there is no error note for math in this ticket (§3.1).
3. Unbalanced or empty delimiters are literal, never placeholders (§3.2–§3.4).
4. Display blocks break the `<p>`; inline stays inside it (§4.2).
5. The shell gains one head slot, `{{asset_links}}`, before the deferred app script (§5.3).
6. Math-free prose markup is byte-identical except the `\$` collapse (§2.4) and the generic-document `<p class="text">` wrapper (§4.3).

Assumptions: TASK-143's KaTeX files and asset paths are final; TASK-144's shell work lands first (§5.3 dependency); no new asset-table entry is needed; the KaTeX build is the upstream UMD classic script exposing `window.katex`.

Settled on 2026-11-16 by ADR-0025 decisions 5–6 revision notes (previously open questions 1 and 2):

7. `output` is `"htmlAndMathml"` (§6). The MathML tree is the assistive-technology path; without it a rendered formula would be entirely `aria-hidden`.
8. `style-src` stays `'self'`; KaTeX's inline `style` attributes are dropped and the resulting deviation is accepted (§7.3). TASK-146 follows ADR-0026 decision 5, not this rule.

Open questions: none for TASK-145.

Recorded assumption: `docs/requirements/non-functional/` contains no accessibility requirement. The MathML path is chosen to avoid a screen-reader regression, not to satisfy a stated requirement; if an accessibility requirement is added, §6 and §7.3 must be reassessed.

## 12. Traceability

| Source | Clause |
|---|---|
| REQ-06.017 AC 1–3, 7, 10–12, 14–17 | §2, §3, §4, §5.4, §7 |
| ADR-0025 decisions 1–10 | §1, §2.1, §3, §4, §5, §6, §7, §8 |
| ADR-0005 (CSP baseline, no Markdown) | §7.1, §8 |
| ADR-0027 (vendored assets) | §5.1, §5.4 |
| ADR-0028 (shell, width) | §5.3, §9 |
| NFR-01.003, NFR-03.001 | §5.4, §8 |
| NFR-02.002 | §8 |
| NFR-05.001 | §9, TASK-149/TASK-150 |
| NFR-06.002 | TASK-143 notices, unchanged here |
