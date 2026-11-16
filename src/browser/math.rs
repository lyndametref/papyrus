//! The non-Markdown math delimiter pre-pass (ADR-0025 decisions 2-5).
//!
//! `prose` turns one curated prose field into the HTML fragment its record
//! view renders and reports whether any math or diagram segment was found. The
//! module is a pure string function plus the head-asset constants: it knows
//! nothing about records, pages, or routes.
//!
//! Exactly the following forms are recognised, in one left-to-right scan:
//!
//! - `$$...$$` display math (may span lines, content trimmed),
//! - `$...$` inline math (single line, no padding, content exact),
//! - a fenced block whose info string is exactly `math`, `tex`, or `latex`;
//! - a fenced block whose info string is exactly `mermaid`, emitted as a
//!   diagram figure (ADR-0026);
//! - a fenced block whose info string is exactly `plantuml` or `puml`,
//!   emitted as a static source figure with a note.
//!
//! Everything else, including a rejected candidate, stays literal text: no
//! marker, no note, and no Markdown interpretation. `\$` is the documented
//! escape for a literal dollar. The server never parses Mermaid; it counts
//! bytes and lines only. The contract is
//! `docs/specifications/browser-math-and-diagram-rendering.md` §3 and §13–§16.

use crate::browser::render::escape;

/// One rendered prose field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Prose {
    /// The rendered HTML fragment: paragraphs, display blocks, and figures in
    /// order.
    pub html: String,
    /// True when at least one inline or display math segment was found.
    pub has_math: bool,
    /// True when at least one renderable `mermaid` figure was found. An
    /// over-limit or PlantUML block does not set it.
    pub has_diagram: bool,
}

/// The head fragment for a page that contains math; `""` otherwise.
pub const HEAD_ASSETS: &str = "<link rel=\"stylesheet\" href=\"/assets/katex/katex.min.css\">\n<script src=\"/assets/katex/katex.min.js\" defer></script>";

/// The head fragment for a page that contains a renderable diagram.
pub const MERMAID_ASSETS: &str = "<script src=\"/assets/mermaid/mermaid.min.js\" defer></script>";

/// Maximum diagram source size in UTF-8 bytes.
pub const MAX_DIAGRAM_BYTES: usize = 100_000;

/// Maximum diagram source line count.
pub const MAX_DIAGRAM_LINES: usize = 1_000;

/// What a fence-looking line turned out to be.
enum Fence<'a> {
    /// Not a recognised opener, an unclosed opener, or empty math/diagram
    /// content. Scanning continues normally one character later.
    Ordinary,
    /// A recognised math fence with trimmed content and the index just past
    /// the closer line (its trailing newline is not consumed).
    Math { tex: &'a str, next: usize },
    /// A closed `mermaid` fence with trimmed source. `renderable` is false
    /// when the source exceeds either input cap.
    Diagram {
        source: &'a str,
        next: usize,
        renderable: bool,
    },
    /// A closed `plantuml` or `puml` fence with trimmed source.
    UnsupportedSource { source: &'a str, next: usize },
    /// A closed non-math, non-diagram fence: the whole block is opaque literal
    /// text that ends at `end` (just past the closer line).
    Opaque { end: usize },
}

/// Run the delimiter pre-pass over one curated prose value.
///
/// `attrs` is the call site's existing paragraph attribute suffix (`""` or
/// ` style="white-space: pre-wrap"`). A value with no display segment is one
/// paragraph, byte-identical to `<p class="text"{attrs}>{escape(value)}</p>`
/// apart from the `\$` collapse.
pub fn prose(value: &str, attrs: &str) -> Prose {
    let bytes = value.as_bytes();
    let mut out = String::with_capacity(value.len() + 32);
    let mut run = String::with_capacity(value.len());
    let mut run_has_content = false;
    let mut has_math = false;
    let mut has_diagram = false;
    let mut diagram_index = 0usize;
    let mut index = 0;

    while index < value.len() {
        if index == 0 || bytes[index - 1] == b'\n' {
            match fence_at(value, index) {
                Fence::Math { tex, next } => {
                    flush_paragraph(&mut out, attrs, &mut run, &mut run_has_content);
                    out.push_str(&display_html(tex));
                    has_math = true;
                    index = next;
                    continue;
                }
                Fence::Diagram {
                    source,
                    next,
                    renderable,
                } => {
                    flush_paragraph(&mut out, attrs, &mut run, &mut run_has_content);
                    diagram_index += 1;
                    out.push_str(&diagram_html(source, renderable, diagram_index));
                    has_diagram |= renderable;
                    index = next;
                    continue;
                }
                Fence::UnsupportedSource { source, next } => {
                    flush_paragraph(&mut out, attrs, &mut run, &mut run_has_content);
                    out.push_str(&unsupported_source_html(source));
                    index = next;
                    continue;
                }
                Fence::Opaque { end } => {
                    run.push_str(&escape(&value[index..end]));
                    run_has_content = true;
                    index = end;
                    continue;
                }
                Fence::Ordinary => {}
            }
        }

        let character = value[index..]
            .chars()
            .next()
            .expect("index is a character boundary");

        if character == '\\' {
            let run_len = backslash_run(bytes, index);
            let after = index + run_len;
            if after < bytes.len() && bytes[after] == b'$' {
                if !run_len.is_multiple_of(2) {
                    // Odd parity escapes the dollar; exactly one backslash is
                    // removed from the run.
                    run.push_str(&"\\".repeat(run_len - 1));
                    run.push('$');
                    run_has_content = true;
                    index = after + 1;
                } else {
                    // Even parity leaves the dollar as a delimiter candidate.
                    run.push_str(&"\\".repeat(run_len));
                    run_has_content = true;
                    index = after;
                }
                continue;
            }
            run.push_str(&value[index..after]);
            run_has_content = true;
            index = after;
            continue;
        }

        if character == '$' {
            if index + 1 < bytes.len() && bytes[index + 1] == b'$' {
                if let Some((tex, next)) = display_at(value, index) {
                    flush_paragraph(&mut out, attrs, &mut run, &mut run_has_content);
                    out.push_str(&display_html(&tex));
                    has_math = true;
                    index = next;
                    continue;
                }
            } else if let Some((tex, next)) = inline_at(value, index) {
                run.push_str(&inline_html(&tex));
                run_has_content = true;
                has_math = true;
                index = next;
                continue;
            }
            // A rejected candidate stays literal and scanning resumes one
            // character later.
            run.push('$');
            run_has_content = true;
            index += 1;
            continue;
        }

        match character {
            '&' => run.push_str("&amp;"),
            '<' => run.push_str("&lt;"),
            '>' => run.push_str("&gt;"),
            '"' => run.push_str("&quot;"),
            '\'' => run.push_str("&#39;"),
            other => run.push(other),
        }
        run_has_content = true;
        index += character.len_utf8();
    }

    if run_has_content {
        flush_paragraph(&mut out, attrs, &mut run, &mut run_has_content);
    } else if out.is_empty() {
        // A field with no display segment is exactly one paragraph, including
        // the empty value.
        out.push_str("<p class=\"text\"");
        out.push_str(attrs);
        out.push_str("></p>");
    }

    Prose {
        html: out,
        has_math,
        has_diagram,
    }
}

fn flush_paragraph(out: &mut String, attrs: &str, run: &mut String, has_content: &mut bool) {
    if !*has_content {
        return;
    }
    out.push_str("<p class=\"text\"");
    out.push_str(attrs);
    out.push('>');
    out.push_str(run);
    out.push_str("</p>");
    run.clear();
    *has_content = false;
}

fn inline_html(tex: &str) -> String {
    let source = escape(tex);
    format!("<span class=\"math\" data-tex=\"{source}\">{source}</span>")
}

fn display_html(tex: &str) -> String {
    let source = escape(tex);
    format!("<div class=\"math math--display\" data-tex=\"{source}\">{source}</div>")
}

/// A `mermaid` figure. The `<pre>` is the fallback and the client input.
///
/// `index` is the figure's 1-based position within its field. The scroll
/// container is a focusable, labelled region, matching every other scroll
/// region in the shell; the name is deterministic and server-rendered.
fn diagram_html(source: &str, renderable: bool, index: usize) -> String {
    let source = escape(source);
    let name = escape(&format!("Diagram {index}"));
    if renderable {
        return format!(
            "<figure class=\"diagram bleed\" data-diagram=\"mermaid\"><div class=\"diagram-scroll\" role=\"region\" aria-label=\"{name}\" tabindex=\"0\"><pre>{source}</pre></div></figure>"
        );
    }
    format!(
        "<figure class=\"diagram bleed\" data-diagram=\"mermaid\" data-render-error=\"diagram\"><div class=\"diagram-scroll\" role=\"region\" aria-label=\"{name}\" tabindex=\"0\"><pre>{source}</pre></div><figcaption class=\"render-note\">Diagram not rendered: source exceeds the 100,000-byte or 1,000-line limit.</figcaption></figure>"
    )
}

/// A `plantuml`/`puml` source figure. It is not a visualization: no `bleed`
/// and no `.diagram-scroll`.
fn unsupported_source_html(source: &str) -> String {
    let source = escape(source);
    format!(
        "<figure class=\"diagram diagram--unsupported\" data-diagram=\"plantuml\"><pre>{source}</pre><figcaption class=\"render-note\">PlantUML diagrams are not supported in this release; showing source.</figcaption></figure>"
    )
}

/// True when the trimmed source exceeds the byte or line cap. Equality is
/// allowed, and `source` is never empty here.
fn diagram_over_limit(source: &str) -> bool {
    let lines = source.bytes().filter(|byte| *byte == b'\n').count() + 1;
    source.len() > MAX_DIAGRAM_BYTES || lines > MAX_DIAGRAM_LINES
}

fn backslash_run(bytes: &[u8], start: usize) -> usize {
    let mut end = start;
    while end < bytes.len() && bytes[end] == b'\\' {
        end += 1;
    }
    end - start
}

/// Find the next closing delimiter at or after `from` whose preceding
/// backslash run has even parity. Inline closing stops at a line break.
fn find_close(value: &str, from: usize, marker: &str, inline: bool) -> Option<usize> {
    let bytes = value.as_bytes();
    let marker = marker.as_bytes();
    let mut index = from;
    let mut backslashes = 0usize;
    while index < bytes.len() {
        if inline && bytes[index] == b'\n' {
            return None;
        }
        if bytes[index..].starts_with(marker) && backslashes.is_multiple_of(2) {
            return Some(index);
        }
        if bytes[index] == b'\\' {
            backslashes += 1;
        } else {
            backslashes = 0;
        }
        index += 1;
    }
    None
}

/// Try a `$$...$$` display segment at `start`. Returns the trimmed TeX and the
/// index just past the closing delimiter.
fn display_at(value: &str, start: usize) -> Option<(String, usize)> {
    let close = find_close(value, start + 2, "$$", false)?;
    let tex = value[start + 2..close].trim();
    if tex.is_empty() {
        return None;
    }
    Some((tex.to_string(), close + 2))
}

/// Try a `$...$` inline segment at `start`. Returns the exact TeX and the
/// index just past the closing delimiter.
fn inline_at(value: &str, start: usize) -> Option<(String, usize)> {
    let close = find_close(value, start + 1, "$", true)?;
    let tex = &value[start + 1..close];
    let first = tex.chars().next()?;
    let last = tex.chars().next_back()?;
    if first.is_whitespace() || last.is_whitespace() {
        return None;
    }
    Some((tex.to_string(), close + 1))
}

fn line_end(bytes: &[u8], from: usize) -> usize {
    bytes[from..]
        .iter()
        .position(|byte| *byte == b'\n')
        .map(|offset| from + offset)
        .unwrap_or(bytes.len())
}

/// Find a closer line for a fence opener.
///
/// A closer is a line with zero to three leading spaces, a run of the same
/// fence character at least as long as the opener, and only trailing
/// whitespace. Returns `(line_start, line_end_of_closer)`.
fn find_fence_closer(
    value: &str,
    fence: u8,
    opener_len: usize,
    content_start: usize,
) -> Option<(usize, usize)> {
    let bytes = value.as_bytes();
    let mut line_start = content_start;
    loop {
        if line_start > bytes.len() {
            return None;
        }
        let end = line_end(bytes, line_start);
        let mut cursor = line_start;
        let mut spaces = 0;
        while spaces < 4 && cursor < end && bytes[cursor] == b' ' {
            cursor += 1;
            spaces += 1;
        }
        if spaces <= 3 && cursor < end && bytes[cursor] == fence {
            let mut run = 0;
            let mut after = cursor;
            while after < end && bytes[after] == fence {
                run += 1;
                after += 1;
            }
            if run >= opener_len && value[after..end].trim().is_empty() {
                return Some((line_start, end));
            }
        }
        if end >= bytes.len() {
            return None;
        }
        line_start = end + 1;
    }
}

/// Inspect the line start at `start` for a fenced math or opaque block.
fn fence_at(value: &str, start: usize) -> Fence<'_> {
    let bytes = value.as_bytes();
    let mut cursor = start;
    let mut spaces = 0;
    while spaces < 4 && cursor < bytes.len() && bytes[cursor] == b' ' {
        cursor += 1;
        spaces += 1;
    }
    if spaces > 3 || cursor >= bytes.len() {
        return Fence::Ordinary;
    }
    let fence = bytes[cursor];
    if fence != b'`' && fence != b'~' {
        return Fence::Ordinary;
    }
    let mut run = 0;
    let mut after = cursor;
    while after < bytes.len() && bytes[after] == fence {
        run += 1;
        after += 1;
    }
    if run < 3 {
        return Fence::Ordinary;
    }
    let opener_end = line_end(bytes, after);
    if opener_end >= bytes.len() {
        // The opener line has no newline, so it cannot have a closer.
        return Fence::Ordinary;
    }
    let info = value[after..opener_end].trim_end();
    let math = matches!(info, "math" | "tex" | "latex");
    let diagram = info == "mermaid";
    let unsupported_source = matches!(info, "plantuml" | "puml");

    let content_start = opener_end + 1;
    let Some((closer_start, closer_end)) = find_fence_closer(value, fence, run, content_start)
    else {
        // Unclosed: literal text scanned normally, math or not.
        return Fence::Ordinary;
    };

    if !math && !diagram && !unsupported_source {
        // Opaque literal block: `$`, `$$`, and `\$` inside are never scanned.
        return Fence::Opaque { end: closer_end };
    }

    let mut content = &value[content_start..closer_start];
    content = content
        .strip_suffix("\r\n")
        .or_else(|| content.strip_suffix('\n'))
        .unwrap_or(content);
    let source = content.trim();
    if source.is_empty() {
        // Empty content is not a math or diagram segment; the fence stays
        // literal.
        return Fence::Ordinary;
    }
    if diagram {
        return Fence::Diagram {
            source,
            next: closer_end,
            renderable: !diagram_over_limit(source),
        };
    }
    if unsupported_source {
        return Fence::UnsupportedSource {
            source,
            next: closer_end,
        };
    }
    Fence::Math {
        tex: source,
        next: closer_end,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PRE: &str = " style=\"white-space: pre-wrap\"";

    fn paragraph(inner: &str) -> String {
        format!("<p class=\"text\">{inner}</p>")
    }

    fn inline(tex: &str) -> String {
        format!("<span class=\"math\" data-tex=\"{tex}\">{tex}</span>")
    }

    fn display(tex: &str) -> String {
        format!("<div class=\"math math--display\" data-tex=\"{tex}\">{tex}</div>")
    }

    #[test]
    fn inline_basic() {
        let rendered = prose("$x$", "");
        assert_eq!(rendered.html, paragraph(&inline("x")));
        assert!(rendered.has_math);
    }

    #[test]
    fn inline_opens_on_whitespace() {
        let rendered = prose("$ x$", "");
        assert_eq!(rendered.html, paragraph("$ x$"));
        assert!(!rendered.has_math);
    }

    #[test]
    fn inline_closes_on_whitespace() {
        let rendered = prose("$x $", "");
        assert_eq!(rendered.html, paragraph("$x $"));
        assert!(!rendered.has_math);
    }

    #[test]
    fn inline_rejects_a_newline() {
        let rendered = prose("$a\nb$", "");
        assert_eq!(rendered.html, paragraph("$a\nb$"));
        assert!(!rendered.has_math);
    }

    #[test]
    fn inline_rejects_no_closing_dollar() {
        let rendered = prose("$x", "");
        assert_eq!(rendered.html, paragraph("$x"));
        assert!(!rendered.has_math);
    }

    #[test]
    fn inline_rejects_empty_content() {
        // `$$` is a display candidate, and `$ $` fails the whitespace rules.
        for value in ["$$", "$ $"] {
            let rendered = prose(value, "");
            assert_eq!(rendered.html, paragraph(value), "{value}");
            assert!(!rendered.has_math, "{value}");
        }
    }

    #[test]
    fn display_basic() {
        let rendered = prose("$$x$$", "");
        assert_eq!(rendered.html, display("x"));
        assert!(rendered.has_math);
    }

    #[test]
    fn display_spans_lines_and_trims() {
        let rendered = prose("$$\n a \n$$", "");
        assert_eq!(rendered.html, display("a"));
        assert!(rendered.has_math);
    }

    #[test]
    fn display_rejects_unclosed_and_empty() {
        for value in ["$$x", "$$$$", "$$ $$"] {
            let rendered = prose(value, "");
            assert_eq!(rendered.html, paragraph(value), "{value}");
            assert!(!rendered.has_math, "{value}");
        }
    }

    #[test]
    fn display_can_start_mid_line_and_breaks_the_paragraph() {
        let rendered = prose("before $$x$$ after", "");
        assert_eq!(
            rendered.html,
            format!(
                "{}{}{}",
                paragraph("before "),
                display("x"),
                paragraph(" after")
            )
        );
        assert!(rendered.has_math);
    }

    #[test]
    fn display_blocks_keep_source_order_and_omit_empty_runs() {
        let rendered = prose("$$a$$$$b$$", "");
        assert_eq!(rendered.html, format!("{}{}", display("a"), display("b")));
        assert!(rendered.has_math);
        assert!(!rendered.html.contains("<p class=\"text\"></p>"));
        assert!(!rendered.html.contains("<p class=\"text\"><div"));
    }

    #[test]
    fn fenced_math_forms_are_display() {
        for value in [
            "```math\nx^2\n```",
            "```tex\nx^2\n```",
            "```latex\nx^2\n```",
            "~~~tex\nx^2\n~~~",
        ] {
            let rendered = prose(value, "");
            assert_eq!(rendered.html, display("x^2"), "{value}");
            assert!(rendered.has_math, "{value}");
        }
    }

    #[test]
    fn fenced_closer_may_be_longer_and_indented() {
        let rendered = prose("  ```math\nx\n  `````", "");
        assert_eq!(rendered.html, display("x"));
    }

    #[test]
    fn four_space_indent_is_not_a_fence() {
        let value = "    ```math\nx\n    ```";
        let rendered = prose(value, "");
        assert_eq!(rendered.html, paragraph("    ```math\nx\n    ```"));
        assert!(!rendered.has_math);
    }

    #[test]
    fn only_exact_math_info_strings_are_recognised() {
        for info in ["Math", "math,", "math title"] {
            let value = format!("```{info}\nx\n```");
            let rendered = prose(&value, "");
            assert!(!rendered.has_math, "{info}");
            assert_eq!(rendered.html, paragraph(&escape(&value)), "{info}");
        }
    }

    #[test]
    fn non_math_fence_is_opaque_literal_text() {
        let value = "```rust\n$x$ and $$y$$ and \\$z\n```";
        let rendered = prose(value, "");
        assert!(!rendered.has_math);
        assert_eq!(rendered.html, paragraph(&escape(value)));
    }

    #[test]
    fn unclosed_non_math_fence_is_scanned_normally() {
        let rendered = prose("```rust\n$x$", "");
        assert!(rendered.has_math);
        assert_eq!(
            rendered.html,
            paragraph(&format!("```rust\n{}", inline("x")))
        );
    }

    #[test]
    fn unclosed_math_fence_is_literal() {
        let value = "```math\nx^2";
        let rendered = prose(value, "");
        assert_eq!(rendered.html, paragraph("```math\nx^2"));
        assert!(!rendered.has_math);
    }

    #[test]
    fn empty_math_fence_is_literal() {
        let value = "```math\n\n```";
        let rendered = prose(value, "");
        assert_eq!(rendered.html, paragraph(value));
        assert!(!rendered.has_math);
    }

    #[test]
    fn dollar_escape_collapses_one_backslash() {
        assert_eq!(prose("cost: \\$5", "").html, paragraph("cost: $5"));
        assert!(!prose("cost: \\$5", "").has_math);
    }

    #[test]
    fn backslash_parity_controls_the_escape() {
        // Two backslashes do not escape the dollar.
        let rendered = prose("a \\\\$x$ b", "");
        assert_eq!(
            rendered.html,
            paragraph(&format!("a \\\\{} b", inline("x")))
        );
        assert!(rendered.has_math);

        // Three backslashes escape it, removing exactly one.
        let rendered = prose("\\\\\\$y$", "");
        assert_eq!(rendered.html, paragraph("\\\\$y$"));
        assert!(!rendered.has_math);
    }

    #[test]
    fn dollar_escape_inside_math_is_tex_input() {
        let rendered = prose("$\\$x$", "");
        assert!(rendered.has_math);
        assert_eq!(rendered.html, paragraph(&inline("\\$x")));
    }

    #[test]
    fn other_backslash_sequences_stay_literal() {
        for value in ["\\(x\\)", "\\[x\\]", "\\*", "\\_", "\\\\"] {
            let rendered = prose(value, "");
            assert_eq!(rendered.html, paragraph(&escape(value)), "{value}");
            assert!(!rendered.has_math, "{value}");
        }
    }

    #[test]
    fn attribute_and_text_escape_the_source() {
        let rendered = prose("$<x&\"'>$", "");
        let escaped = "&lt;x&amp;&quot;&#39;&gt;";
        assert_eq!(rendered.html, paragraph(&inline(escaped)));
        assert!(rendered.has_math);
    }

    #[test]
    fn math_free_input_is_byte_identical_to_one_paragraph() {
        for value in ["", "Plain prose", "Line one\nLine two", "a & b < c"] {
            let rendered = prose(value, PRE);
            assert_eq!(
                rendered.html,
                format!("<p class=\"text\"{PRE}>{}</p>", escape(value)),
                "{value:?}"
            );
            assert!(!rendered.has_math, "{value:?}");
        }
    }

    #[test]
    fn multiple_paragraphs_surround_display_without_nesting() {
        let rendered = prose("one $$a$$ two $$b$$ three", PRE);
        assert_eq!(
            rendered.html,
            format!(
                "<p class=\"text\"{PRE}>one </p>{}<p class=\"text\"{PRE}> two </p>{}<p class=\"text\"{PRE}> three</p>",
                display("a"),
                display("b")
            )
        );
        assert!(!rendered.html.contains("<p class=\"text\"><div"));
    }

    #[test]
    fn heuristics_reject_price_like_text() {
        let rendered = prose("$5 and $10", "");
        assert_eq!(rendered.html, paragraph("$5 and $10"));
        assert!(!rendered.has_math);
    }

    #[test]
    fn a_rejected_candidate_resumes_scanning() {
        // The first `$` opens on a space and is rejected, but the second one
        // still opens a valid segment.
        let rendered = prose("$ 1 $x$", "");
        assert_eq!(rendered.html, paragraph(&format!("$ 1 {}", inline("x"))));
        assert!(rendered.has_math);
    }

    #[test]
    fn markdown_constructs_stay_literal() {
        let value = "# H\n- item\n1. item\n| a | b |\n**bold** [x](https://e) `c` > q\n<script>alert(1)</script>";
        let rendered = prose(value, "");
        assert_eq!(rendered.html, paragraph(&escape(value)));
        assert!(!rendered.has_math);
    }

    #[test]
    fn head_assets_reference_only_vendored_paths() {
        assert!(HEAD_ASSETS.contains("href=\"/assets/katex/katex.min.css\""));
        assert!(HEAD_ASSETS.contains("src=\"/assets/katex/katex.min.js\" defer"));
        assert!(HEAD_ASSETS.find("katex.min.css") < HEAD_ASSETS.find("katex.min.js"));
    }

    const MERMAID_FIGURE: &str = "<figure class=\"diagram bleed\" data-diagram=\"mermaid\"><div class=\"diagram-scroll\" role=\"region\" aria-label=\"Diagram {INDEX}\" tabindex=\"0\"><pre>{SOURCE}</pre></div></figure>";
    const MERMAID_OVER_LIMIT_FIGURE: &str = "<figure class=\"diagram bleed\" data-diagram=\"mermaid\" data-render-error=\"diagram\"><div class=\"diagram-scroll\" role=\"region\" aria-label=\"Diagram {INDEX}\" tabindex=\"0\"><pre>{SOURCE}</pre></div><figcaption class=\"render-note\">Diagram not rendered: source exceeds the 100,000-byte or 1,000-line limit.</figcaption></figure>";
    const PLANTUML_FIGURE: &str = "<figure class=\"diagram diagram--unsupported\" data-diagram=\"plantuml\"><pre>{SOURCE}</pre><figcaption class=\"render-note\">PlantUML diagrams are not supported in this release; showing source.</figcaption></figure>";

    fn figure(template: &str, index: usize, source: &str) -> String {
        template
            .replace("{SOURCE}", &escape(source))
            .replace("{INDEX}", &index.to_string())
    }

    fn diagram_fence(source: &str) -> String {
        format!("```mermaid\n{source}\n```")
    }

    #[test]
    fn mermaid_fence_is_a_renderable_figure() {
        let source = "flowchart TD\n  A --> B";
        for fence in ["```", "~~~"] {
            let value = format!("{fence}mermaid\n{source}\n{fence}");
            let rendered = prose(&value, "");
            assert_eq!(rendered.html, figure(MERMAID_FIGURE, 1, source), "{fence}");
            assert!(rendered.has_diagram, "{fence}");
            assert!(!rendered.has_math, "{fence}");
        }
        let rendered = prose("  ```mermaid\ngraph TD\n  A --> B\n  `````", "");
        assert_eq!(
            rendered.html,
            figure(MERMAID_FIGURE, 1, "graph TD\n  A --> B")
        );
        assert!(rendered.has_diagram);
    }

    #[test]
    fn plantuml_and_puml_fences_are_source_figures() {
        for info in ["plantuml", "puml"] {
            let rendered = prose(&format!("```{info}\nA -> B\n```"), "");
            assert_eq!(
                rendered.html,
                figure(PLANTUML_FIGURE, 0, "A -> B"),
                "{info}"
            );
            assert!(!rendered.has_diagram, "{info}");
            assert!(!rendered.has_math, "{info}");
        }
    }

    #[test]
    fn other_fence_info_strings_stay_opaque() {
        for info in [
            "mmd",
            "uml",
            "dot",
            "graphviz",
            "rust",
            "Mermaid",
            "mermaid,",
            "mermaid title",
        ] {
            let value = format!("```{info}\nflowchart TD\nA --> B\n```");
            let rendered = prose(&value, "");
            assert_eq!(rendered.html, paragraph(&escape(&value)), "{info}");
            assert!(!rendered.has_diagram, "{info}");
            assert!(!rendered.has_math, "{info}");
        }
    }

    #[test]
    fn empty_unclosed_and_indented_diagram_fences_stay_literal() {
        for value in [
            "```mermaid\n\n```",
            "```mermaid\n   \n```",
            "```mermaid\nA --> B",
        ] {
            let rendered = prose(value, "");
            assert_eq!(rendered.html, paragraph(&escape(value)), "{value:?}");
            assert!(!rendered.has_diagram, "{value:?}");
        }
        for value in ["```plantuml\n\n```", "```puml\n   \n```", "```puml\nA -> B"] {
            let rendered = prose(value, "");
            assert_eq!(rendered.html, paragraph(&escape(value)), "{value:?}");
            assert!(!rendered.has_diagram, "{value:?}");
        }
        let value = "    ```mermaid\nA --> B\n    ```";
        let rendered = prose(value, "");
        assert_eq!(rendered.html, paragraph(&escape(value)));
        assert!(!rendered.has_diagram);
    }

    #[test]
    fn figure_source_is_escaped_once_and_keeps_newlines() {
        let source = "graph TD\n  A[<x & \"q\" 'z'>] --> B";
        let rendered = prose(&diagram_fence(source), "");
        assert_eq!(rendered.html, figure(MERMAID_FIGURE, 1, source));
        assert!(rendered.html.contains(
            "<pre>graph TD\n  A[&lt;x &amp; &quot;q&quot; &#39;z&#39;&gt;] --&gt; B</pre>"
        ));
        assert!(!rendered.html.contains("&amp;amp;"));

        let once = prose(&diagram_fence("&amp;"), "");
        assert!(once.html.contains("<pre>&amp;amp;</pre>"));
    }

    #[test]
    fn a_figure_flushes_the_paragraph_and_never_nests() {
        let source = "graph TD\n  A --> B";
        let rendered = prose(&format!("before\n\n{}\n\nafter", diagram_fence(source)), "");
        assert_eq!(
            rendered.html,
            format!(
                "{}{}{}",
                paragraph("before\n\n"),
                figure(MERMAID_FIGURE, 1, source),
                paragraph("\n\nafter")
            )
        );
        assert!(!rendered.html.contains("<p class=\"text\"><figure"));
        assert!(!rendered.html.contains("<p class=\"text\"><pre"));
        assert!(rendered.has_diagram);
    }

    #[test]
    fn figures_and_display_math_keep_source_order() {
        let rendered = prose(
            "$$m$$\n```mermaid\ngraph TD\nA --> B\n```\n```plantuml\nA -> B\n```",
            "",
        );
        let math_at = rendered
            .html
            .find("data-tex=\"m\"")
            .expect("display math present");
        let mermaid_at = rendered
            .html
            .find("data-diagram=\"mermaid\"")
            .expect("mermaid figure present");
        let plantuml_at = rendered
            .html
            .find("data-diagram=\"plantuml\"")
            .expect("plantuml figure present");
        assert!(math_at < mermaid_at, "math precedes the mermaid figure");
        assert!(
            mermaid_at < plantuml_at,
            "mermaid figure precedes the plantuml figure"
        );
        assert!(rendered.has_math);
        assert!(rendered.has_diagram);
    }

    #[test]
    fn diagram_free_inputs_keep_the_page_flags_false() {
        for value in [
            "",
            "Plain prose",
            "$x$",
            "```mmd\ngraph TD\nA --> B\n```",
            "~~~uml\ngraph TD\nA --> B\n~~~",
        ] {
            let rendered = prose(value, PRE);
            assert!(!rendered.has_diagram, "{value:?}");
        }
    }

    #[test]
    fn a_plantuml_only_field_never_sets_has_diagram() {
        let rendered = prose("```plantuml\nA -> B\n```", "");
        assert!(!rendered.has_diagram);
        assert!(rendered.html.contains("diagram--unsupported"));
    }

    #[test]
    fn caps_allow_equality_and_reject_one_over() {
        let at_bytes = "a".repeat(MAX_DIAGRAM_BYTES);
        let rendered = prose(&diagram_fence(&at_bytes), "");
        assert!(rendered.has_diagram, "exactly MAX_DIAGRAM_BYTES");
        assert_eq!(rendered.html, figure(MERMAID_FIGURE, 1, &at_bytes));

        let over_bytes = "a".repeat(MAX_DIAGRAM_BYTES + 1);
        let rendered = prose(&diagram_fence(&over_bytes), "");
        assert!(!rendered.has_diagram, "one byte over");
        assert_eq!(
            rendered.html,
            figure(MERMAID_OVER_LIMIT_FIGURE, 1, &over_bytes)
        );
        assert!(rendered.html.contains("data-render-error=\"diagram\""));
    }

    #[test]
    fn line_cap_allows_equality_and_rejects_one_line_over() {
        let at_lines = "l\n".repeat(MAX_DIAGRAM_LINES - 1) + "l";
        assert_eq!(
            at_lines.bytes().filter(|byte| *byte == b'\n').count() + 1,
            MAX_DIAGRAM_LINES
        );
        let rendered = prose(&diagram_fence(&at_lines), "");
        assert!(rendered.has_diagram, "exactly MAX_DIAGRAM_LINES");
        assert_eq!(rendered.html, figure(MERMAID_FIGURE, 1, &at_lines));

        let over_lines = "l\n".repeat(MAX_DIAGRAM_LINES) + "l";
        assert_eq!(
            over_lines.bytes().filter(|byte| *byte == b'\n').count() + 1,
            MAX_DIAGRAM_LINES + 1
        );
        let rendered = prose(&diagram_fence(&over_lines), "");
        assert!(!rendered.has_diagram, "one line over");
        assert_eq!(
            rendered.html,
            figure(MERMAID_OVER_LIMIT_FIGURE, 1, &over_lines)
        );
    }

    #[test]
    fn a_renderable_figure_sets_the_page_flag_even_beside_an_over_limit_one() {
        let over = "a".repeat(MAX_DIAGRAM_BYTES + 1);
        let rendered = prose(
            &format!(
                "{}\n\n{}",
                diagram_fence(&over),
                diagram_fence("graph TD\nA --> B")
            ),
            "",
        );
        assert!(rendered.has_diagram);
        assert_eq!(
            rendered
                .html
                .matches("data-render-error=\"diagram\"")
                .count(),
            1
        );
        assert_eq!(rendered.html.matches("class=\"diagram bleed\"").count(), 2);
    }

    #[test]
    fn mermaid_assets_reference_only_the_vendored_bundle() {
        assert_eq!(
            MERMAID_ASSETS,
            "<script src=\"/assets/mermaid/mermaid.min.js\" defer></script>"
        );
        assert_eq!(MERMAID_ASSETS.matches("/assets/").count(), 1);
        assert!(MERMAID_ASSETS.contains("defer"));
        assert_eq!(
            HEAD_ASSETS,
            "<link rel=\"stylesheet\" href=\"/assets/katex/katex.min.css\">\n<script src=\"/assets/katex/katex.min.js\" defer></script>"
        );
    }
}
