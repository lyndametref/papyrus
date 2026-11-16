//! The non-Markdown math delimiter pre-pass (ADR-0025 decisions 2-5).
//!
//! `prose` turns one curated prose field into the HTML fragment its record
//! view renders and reports whether any math segment was found. The module is
//! a pure string function plus the head-asset constant: it knows nothing about
//! records, pages, or routes.
//!
//! Exactly three forms are recognised, in one left-to-right scan:
//!
//! - `$$...$$` display math (may span lines, content trimmed),
//! - `$...$` inline math (single line, no padding, content exact),
//! - a fenced block whose info string is exactly `math`, `tex`, or `latex`.
//!
//! Everything else, including a rejected candidate, stays literal text: no
//! marker, no note, and no Markdown interpretation. `\$` is the documented
//! escape for a literal dollar. The contract is
//! `docs/specifications/browser-math-and-diagram-rendering.md` §3.

use crate::browser::render::escape;

/// One rendered prose field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Prose {
    /// The rendered HTML fragment: paragraphs and display blocks in order.
    pub html: String,
    /// True when at least one inline or display math segment was found.
    pub has_math: bool,
}

/// The head fragment for a page that contains math; `""` otherwise.
pub const HEAD_ASSETS: &str = "<link rel=\"stylesheet\" href=\"/assets/katex/katex.min.css\">\n<script src=\"/assets/katex/katex.min.js\" defer></script>";

/// What a fence-looking line turned out to be.
enum Fence<'a> {
    /// Not a recognised opener, an unclosed opener, or empty math content.
    /// Scanning continues normally one character later.
    Ordinary,
    /// A recognised math fence with trimmed content and the index just past
    /// the closer line (its trailing newline is not consumed).
    Math { tex: &'a str, next: usize },
    /// A closed non-math fence: the whole block is opaque literal text that
    /// ends at `end` (just past the closer line).
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

    let content_start = opener_end + 1;
    let Some((closer_start, closer_end)) = find_fence_closer(value, fence, run, content_start)
    else {
        // Unclosed: literal text scanned normally, math or not.
        return Fence::Ordinary;
    };

    if !math {
        // Opaque literal block: `$`, `$$`, and `\$` inside are never scanned.
        return Fence::Opaque { end: closer_end };
    }

    let mut content = &value[content_start..closer_start];
    content = content
        .strip_suffix("\r\n")
        .or_else(|| content.strip_suffix('\n'))
        .unwrap_or(content);
    let tex = content.trim();
    if tex.is_empty() {
        // Empty content is not a math segment; the fence stays literal.
        return Fence::Ordinary;
    }
    Fence::Math {
        tex,
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
}
