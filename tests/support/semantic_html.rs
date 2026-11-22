#![allow(dead_code)] // Each integration test binary compiles this shared module.
//! Test-only semantic HTML comparator (MS-035 TASK-269, corrected by AUD-145 H1).
//!
//! The comparison is deliberately semantic rather than byte-oriented:
//!
//! * element existence, every attribute, and visible text are compared,
//! * attribute order, whitespace, entities, and insignificant markup are
//!   ignored,
//! * a difference in any element, attribute, text, link, or field fails unless
//!   the committed per-case node allowlist names that exact node part.
//!
//! The allowlist is static data in
//! `tests/fixtures/characterization/semantic-allowlist.json`. It is never
//! derived from the page under test, and it contains no page-container subtree
//! (`form.form`, `section.section`, `main.main`). Element-level entries on a
//! container are allowed because they remove only the container's own tokens.
//! Every entry names one node, a reason code from MS-035 TASK-269, and the
//! `EXPECTED-DELTAS.md` item that attributes it.

use std::sync::OnceLock;

use serde::Deserialize;

/// The committed static node allowlist.
const ALLOWLIST_JSON: &str = include_str!("../fixtures/characterization/semantic-allowlist.json");

/// One allowed difference: a case, one node on each side, and the reason code
/// plus the `EXPECTED-DELTAS.md` item it implements.
#[derive(Clone, Debug, Deserialize)]
pub struct AllowedDelta {
    pub case: String,
    pub kind: AllowKind,
    /// Structural node path in the captured baseline page.
    #[serde(default)]
    pub expected: Option<String>,
    /// Structural node path in the accepted page.
    #[serde(default)]
    pub actual: Option<String>,
    /// Attribute name for `kind: attr`.
    #[serde(default)]
    pub name: Option<String>,
    pub reason: String,
    pub delta: String,
}

/// The node part an allowance removes from the comparison.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AllowKind {
    Element,
    Attr,
    Text,
    Subtree,
}

/// The page family a case belongs to. The allowed reason/item combinations
/// depend on it: a title relocation is meaningful on view, edit, conflict, and
/// form pages, but a list page must not cite it to hide a seed label.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PageKind {
    List,
    View,
    Edit,
    Conflict,
    NotFound,
    Form,
    EmptyList,
}

fn page_kind(case: &str) -> Option<PageKind> {
    let tail = case.split_once('/')?.1;
    if tail.starts_with("not-found-") {
        return Some(PageKind::NotFound);
    }
    if tail.starts_with("empty-list-") {
        return Some(PageKind::EmptyList);
    }
    match tail.split('-').next()? {
        "list" => Some(PageKind::List),
        "view" => Some(PageKind::View),
        "edit" => Some(PageKind::Edit),
        "conflict" => Some(PageKind::Conflict),
        "form" => Some(PageKind::Form),
        _ => None,
    }
}

impl AllowedDelta {
    /// The only reason/item/case-kind combinations the expected-delta document
    /// authorises. The reason must also appear in that document, and the item
    /// must be a numbered entry there.
    fn is_allowed_combination(&self) -> bool {
        let Some(kind) = page_kind(&self.case) else {
            return false;
        };
        matches!(
            (self.reason.as_str(), self.delta.as_str(), kind),
            (
                "uniform-title-block",
                "item-9",
                PageKind::View | PageKind::Edit | PageKind::Conflict | PageKind::Form
            ) | (
                "seed-presentation",
                "item-16",
                PageKind::List
                    | PageKind::View
                    | PageKind::Edit
                    | PageKind::Conflict
                    | PageKind::NotFound
                    | PageKind::Form
                    | PageKind::EmptyList
            ) | ("seed-presentation", "item-18", PageKind::List)
                | (
                    "expected-delta-6",
                    "item-6",
                    PageKind::Conflict | PageKind::NotFound
                )
        )
    }

    pub fn is_documented(&self, expected_deltas: &str) -> bool {
        let item = match self.delta.as_str() {
            "item-6" => "6",
            "item-9" => "9",
            "item-16" => "16",
            "item-18" => "18",
            _ => return false,
        };
        self.is_allowed_combination()
            && expected_deltas.contains(&self.reason)
            && expected_deltas.contains(&format!("\n{item}. "))
    }
}

/// The explicit allowlist. Only entries whose `case` equals the compared case
/// apply. Every entry must name a token that exists on a declared side.
#[derive(Debug, Default)]
pub struct Allowlist {
    entries: Vec<AllowedDelta>,
}

impl Allowlist {
    pub fn new(entries: Vec<AllowedDelta>) -> Allowlist {
        Allowlist { entries }
    }

    pub fn empty() -> Allowlist {
        Allowlist::default()
    }

    /// The committed static table, parsed once per test process.
    pub fn static_table() -> &'static Allowlist {
        static TABLE: OnceLock<Allowlist> = OnceLock::new();
        TABLE.get_or_init(|| {
            let entries: Vec<AllowedDelta> = serde_json::from_str(ALLOWLIST_JSON)
                .expect("parse tests/fixtures/characterization/semantic-allowlist.json");
            Allowlist::new(entries)
        })
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn entries(&self) -> &[AllowedDelta] {
        &self.entries
    }

    fn for_case(&self, case: &str) -> Vec<&AllowedDelta> {
        self.entries
            .iter()
            .filter(|entry| entry.case == case)
            .collect()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Expected,
    Actual,
}

/// Compare one document page semantically. `Ok(())` means every token matched
/// or was named by a documented node allowance; `Err` names the first
/// difference or an unused allowance.
pub fn compare_document_html(
    case: &str,
    expected: &str,
    actual: &str,
    allowlist: &Allowlist,
) -> Result<(), String> {
    let expected_tokens = tokenize(expected);
    let actual_tokens = tokenize(actual);
    let entries = allowlist.for_case(case);

    let expected_visible = filter_side(&expected_tokens, &entries, Side::Expected);
    let actual_visible = filter_side(&actual_tokens, &entries, Side::Actual);

    for entry in &entries {
        let expected_matched = matched_side(&expected_tokens, entry, Side::Expected);
        let actual_matched = matched_side(&actual_tokens, entry, Side::Actual);
        if expected_matched.is_empty() && actual_matched.is_empty() {
            return Err(format!(
                "{case}: unused allowlist entry {} at {} ({}, {})",
                describe_kind(entry),
                entry
                    .expected
                    .as_deref()
                    .or(entry.actual.as_deref())
                    .unwrap_or("<no path>"),
                entry.reason,
                entry.delta
            ));
        }
        if same_keys(&expected_matched, &actual_matched) {
            return Err(format!(
                "{case}: unused allowlist entry {} at {} hides no difference ({}, {})",
                describe_kind(entry),
                entry
                    .expected
                    .as_deref()
                    .or(entry.actual.as_deref())
                    .unwrap_or("<no path>"),
                entry.reason,
                entry.delta
            ));
        }
    }

    compare_relocated_title(case, &expected_tokens, &actual_tokens)?;

    same_keys(&expected_visible, &actual_visible)
        .then_some(())
        .ok_or_else(|| first_difference(case, &expected_visible, &actual_visible))
}

/// Compare the page heading with the document title independently of the node
/// allowlist. A subtree allowance may hide the relocated `h1` from the generic
/// token walk, but the accepted heading must still be the title text: either
/// the whole `<title>` suffix (list, edit, conflict, not-found) or its trailing
/// record-title segment (view). When the document title itself did not change,
/// the baseline heading and the accepted heading must also match. Conflict
/// pages are the documented exception (item 6): their heading moves from the
/// conflict notice to the edit title, so they are pinned to their own title.
fn compare_relocated_title(
    case: &str,
    expected_tokens: &[Token],
    actual_tokens: &[Token],
) -> Result<(), String> {
    let Some(actual_heading) = element_text(actual_tokens, "h1") else {
        return Ok(());
    };
    let Some(actual_title) = title_suffix(actual_tokens) else {
        return Ok(());
    };
    if !title_matches_heading(&actual_title, &actual_heading) {
        return Err(format!(
            "{case}: page title text {actual_heading:?} does not match the document title {actual_title:?}"
        ));
    }
    if let (Some(expected_heading), Some(expected_title)) = (
        element_text(expected_tokens, "h1"),
        title_suffix(expected_tokens),
    ) && title_matches_heading(&expected_title, &expected_heading)
        && expected_title == actual_title
        && expected_heading != actual_heading
    {
        return Err(format!(
            "{case}: relocated page title changed from {expected_heading:?} to {actual_heading:?}"
        ));
    }
    Ok(())
}

fn title_matches_heading(title: &str, heading: &str) -> bool {
    !heading.is_empty() && (title == heading || title.ends_with(&format!(" {heading}")))
}

fn element_text(tokens: &[Token], tag: &str) -> Option<String> {
    let start = tokens
        .iter()
        .find(|token| token.kind == TokenKind::Start && token.key == format!("start:{tag}"))?;
    let text_path = format!("{}#text0", start.path);
    tokens
        .iter()
        .find(|token| token.path == text_path && token.kind == TokenKind::Text)
        .and_then(|token| token.key.strip_prefix("text:"))
        .map(str::to_string)
}

fn title_suffix(tokens: &[Token]) -> Option<String> {
    let title = element_text(tokens, "title")?;
    title
        .split_once(" \u{2014} ")
        .map(|(_, suffix)| suffix.to_string())
}

/// Parse a page into path-addressed semantic tokens.
pub fn tokenize(html: &str) -> Vec<Token> {
    let bytes = html.as_bytes();
    let mut index = 0usize;
    let mut tokens: Vec<Token> = Vec::new();
    let mut stack: Vec<OpenElement> = vec![OpenElement::root()];
    while index < bytes.len() {
        if bytes[index] != b'<' {
            let next = html[index..]
                .find('<')
                .map(|offset| index + offset)
                .unwrap_or(bytes.len());
            let raw = &html[index..next];
            if let Some(text) = canonical_text(raw) {
                let parent = stack.last_mut().expect("parser root");
                let path = text_path(&parent.path, parent.texts);
                parent.texts += 1;
                tokens.push(Token {
                    key: format!("text:{text}"),
                    path,
                    kind: TokenKind::Text,
                });
            }
            index = next;
            continue;
        }
        if html[index..].starts_with("<!--") {
            index = html[index + 4..]
                .find("-->")
                .map(|offset| index + 4 + offset + 3)
                .unwrap_or(bytes.len());
            continue;
        }
        if html[index..].starts_with("<!") || html[index..].starts_with("<?") {
            index = html[index..]
                .find('>')
                .map(|offset| index + offset + 1)
                .unwrap_or(bytes.len());
            continue;
        }
        if bytes.get(index + 1) == Some(&b'/') {
            let end = html[index..]
                .find('>')
                .map(|offset| index + offset)
                .unwrap_or(bytes.len());
            let name = html[index + 2..end]
                .split_whitespace()
                .next()
                .unwrap_or("")
                .to_ascii_lowercase();
            index = (end + 1).min(bytes.len());
            if name.is_empty() {
                continue;
            }
            let position = stack.iter().rposition(|open| open.tag == name);
            let Some(position) = position else {
                continue;
            };
            let path = stack[position].path.clone();
            tokens.push(Token {
                key: format!("end:{name}"),
                path,
                kind: TokenKind::End,
            });
            stack.truncate(position);
            continue;
        }
        let (tag, next, self_closing) = parse_start_tag(html, index);
        index = next;
        let name = tag.name.clone();
        let (path, parent) = {
            let parent = stack.last_mut().expect("parser root");
            let path = element_path(&parent.path, parent.children);
            parent.children += 1;
            (path, parent.path.clone())
        };
        let _ = parent;
        tokens.push(Token {
            key: format!("start:{name}"),
            path: path.clone(),
            kind: TokenKind::Start,
        });
        let mut attrs = tag.attrs.clone();
        attrs.sort();
        for (attr, value) in attrs {
            tokens.push(Token {
                key: format!("attr:{attr}={value:?}"),
                path: path.clone(),
                kind: TokenKind::Attr(attr),
            });
        }
        if !VOID_ELEMENTS.contains(&name.as_str()) && !self_closing {
            stack.push(OpenElement {
                path,
                tag: name,
                children: 0,
                texts: 0,
            });
        } else if !VOID_ELEMENTS.contains(&name.as_str()) && self_closing {
            tokens.push(Token {
                key: format!("end:{name}"),
                path,
                kind: TokenKind::End,
            });
        }
    }
    tokens
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token {
    pub key: String,
    pub path: String,
    pub kind: TokenKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TokenKind {
    Start,
    End,
    Attr(String),
    Text,
}

struct OpenElement {
    path: String,
    tag: String,
    children: usize,
    texts: usize,
}

impl OpenElement {
    fn root() -> OpenElement {
        OpenElement {
            path: String::new(),
            tag: String::new(),
            children: 0,
            texts: 0,
        }
    }
}

fn element_path(parent: &str, index: usize) -> String {
    if parent.is_empty() {
        index.to_string()
    } else {
        format!("{parent}.{index}")
    }
}

fn text_path(parent: &str, index: usize) -> String {
    if parent.is_empty() {
        format!("#text{index}")
    } else {
        format!("{parent}#text{index}")
    }
}

fn describe_kind(entry: &AllowedDelta) -> String {
    match entry.kind {
        AllowKind::Element => "element".to_string(),
        AllowKind::Attr => format!("attribute {}", entry.name.as_deref().unwrap_or("<none>")),
        AllowKind::Text => "text".to_string(),
        AllowKind::Subtree => "subtree".to_string(),
    }
}

fn side_path(entry: &AllowedDelta, side: Side) -> Option<&str> {
    match side {
        Side::Expected => entry.expected.as_deref(),
        Side::Actual => entry.actual.as_deref(),
    }
}

fn filter_side(tokens: &[Token], entries: &[&AllowedDelta], side: Side) -> Vec<Token> {
    tokens
        .iter()
        .filter(|token| {
            !entries.iter().any(|entry| {
                side_path(entry, side).is_some_and(|path| entry_matches(token, entry, path))
            })
        })
        .cloned()
        .collect()
}

fn matched_side(tokens: &[Token], entry: &AllowedDelta, side: Side) -> Vec<Token> {
    let Some(path) = side_path(entry, side) else {
        return Vec::new();
    };
    tokens
        .iter()
        .filter(|token| entry_matches(token, entry, path))
        .cloned()
        .collect()
}

fn entry_matches(token: &Token, entry: &AllowedDelta, path: &str) -> bool {
    match entry.kind {
        AllowKind::Subtree => {
            token.path == path
                || token.path.starts_with(&format!("{path}."))
                || token.path.starts_with(&format!("{path}#text"))
        }
        AllowKind::Element => {
            token.path == path && matches!(token.kind, TokenKind::Start | TokenKind::End)
        }
        AllowKind::Attr => {
            token.path == path
                && entry.name.as_deref().is_some_and(
                    |name| matches!(&token.kind, TokenKind::Attr(found) if found == name),
                )
        }
        AllowKind::Text => token.path == path && token.kind == TokenKind::Text,
    }
}

/// The element tag at a structural path, if the path names a start token.
pub fn element_tag_at_path(html: &str, path: &str) -> Option<String> {
    tokenize(html).into_iter().find_map(|token| {
        if token.path == path && token.kind == TokenKind::Start {
            token.key.strip_prefix("start:").map(str::to_string)
        } else {
            None
        }
    })
}

fn subtree_tokens(tokens: &[Token], path: &str) -> Vec<Token> {
    tokens
        .iter()
        .filter(|token| {
            token.path == path
                || token.path.starts_with(&format!("{path}."))
                || token.path.starts_with(&format!("{path}#text"))
        })
        .cloned()
        .collect()
}

fn same_keys(left: &[Token], right: &[Token]) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .all(|(left, right)| left.key == right.key)
}

fn first_difference(case: &str, expected: &[Token], actual: &[Token]) -> String {
    let limit = expected.len().max(actual.len());
    for index in 0..limit {
        match (expected.get(index), actual.get(index)) {
            (Some(left), Some(right)) if left.key == right.key => {}
            (Some(left), Some(right)) => {
                return format!(
                    "{case}: semantic HTML differs at token {index} ({} vs {}):\n  expected: {}\n  actual:   {}",
                    left.path, right.path, left.key, right.key
                );
            }
            (Some(left), None) => {
                return format!(
                    "{case}: semantic HTML has an extra expected token {index} at {}: {}",
                    left.path, left.key
                );
            }
            (None, Some(right)) => {
                return format!(
                    "{case}: semantic HTML has an extra actual token {index} at {}: {}",
                    right.path, right.key
                );
            }
            (None, None) => break,
        }
    }
    format!("{case}: semantic HTML differs in length")
}

#[derive(Clone, Debug)]
struct StartTag {
    name: String,
    attrs: Vec<(String, String)>,
}

const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

fn parse_start_tag(html: &str, start: usize) -> (StartTag, usize, bool) {
    let bytes = html.as_bytes();
    let mut index = start + 1;
    let name_start = index;
    while index < bytes.len() && !is_tag_delimiter(bytes[index]) {
        index += 1;
    }
    let name = html[name_start..index].to_ascii_lowercase();
    let mut attrs = Vec::new();
    let mut self_closing = false;
    loop {
        while index < bytes.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }
        if index >= bytes.len() {
            break;
        }
        match bytes[index] {
            b'>' => {
                index += 1;
                break;
            }
            b'/' => {
                self_closing = true;
                index += 1;
                continue;
            }
            _ => {}
        }
        let attr_start = index;
        while index < bytes.len()
            && !bytes[index].is_ascii_whitespace()
            && bytes[index] != b'='
            && bytes[index] != b'>'
            && bytes[index] != b'/'
        {
            index += 1;
        }
        let attr = html[attr_start..index].to_ascii_lowercase();
        if attr.is_empty() {
            index += 1;
            continue;
        }
        while index < bytes.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }
        let mut value = String::new();
        if bytes.get(index) == Some(&b'=') {
            index += 1;
            while index < bytes.len() && bytes[index].is_ascii_whitespace() {
                index += 1;
            }
            match bytes.get(index).copied() {
                Some(quote @ (b'"' | b'\'')) => {
                    index += 1;
                    let value_start = index;
                    while index < bytes.len() && bytes[index] != quote {
                        index += 1;
                    }
                    value = decode_entities(&html[value_start..index]);
                    index = (index + 1).min(bytes.len());
                }
                Some(_) => {
                    let value_start = index;
                    while index < bytes.len()
                        && !bytes[index].is_ascii_whitespace()
                        && bytes[index] != b'>'
                    {
                        index += 1;
                    }
                    value = decode_entities(&html[value_start..index]);
                }
                None => {}
            }
        }
        attrs.push((attr, value));
    }
    (StartTag { name, attrs }, index, self_closing)
}

fn is_tag_delimiter(byte: u8) -> bool {
    byte.is_ascii_whitespace() || byte == b'>' || byte == b'/'
}

fn canonical_text(raw: &str) -> Option<String> {
    let decoded = decode_entities(raw);
    let collapsed = decoded.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.is_empty() {
        None
    } else {
        Some(collapsed)
    }
}

fn decode_entities(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut rest = raw;
    while let Some(position) = rest.find('&') {
        out.push_str(&rest[..position]);
        let after = &rest[position..];
        let Some(end) = after.find(';') else {
            out.push_str(after);
            return out;
        };
        let entity = &after[1..end];
        if let Some(decoded) = decode_entity(entity) {
            out.push_str(&decoded);
            rest = &after[end + 1..];
        } else {
            out.push('&');
            rest = &after[1..];
        }
    }
    out.push_str(rest);
    out
}

fn decode_entity(entity: &str) -> Option<String> {
    let named = match entity {
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" => Some('"'),
        "apos" => Some('\''),
        "nbsp" => Some('\u{a0}'),
        "middot" => Some('\u{b7}'),
        "mdash" => Some('\u{2014}'),
        "ndash" => Some('\u{2013}'),
        "times" => Some('\u{00d7}'),
        "copy" => Some('\u{00a9}'),
        "hellip" => Some('\u{2026}'),
        _ => None,
    };
    if let Some(character) = named {
        return Some(character.to_string());
    }
    let numeric = if let Some(hex) = entity
        .strip_prefix("#x")
        .or_else(|| entity.strip_prefix("#X"))
    {
        u32::from_str_radix(hex, 16).ok()
    } else {
        entity
            .strip_prefix('#')
            .and_then(|decimal| decimal.parse::<u32>().ok())
    };
    numeric
        .and_then(char::from_u32)
        .map(|character| character.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page(body: &str) -> String {
        format!("<!doctype html><html><head><title>T</title></head><body>{body}</body></html>")
    }

    fn compare(expected: &str, actual: &str) -> Result<(), String> {
        compare_document_html("synthetic", expected, actual, &Allowlist::empty())
    }

    #[test]
    fn attribute_order_whitespace_entities_and_doctype_are_ignored() {
        let expected = page("<a class=\"link\" href=\"/doc/REQ-001\">A &amp; B</a>");
        let actual = page("\n  <a href=\"/doc/REQ-001\" class=\"link\">A &#38; B</a>\n");
        assert_eq!(compare(&expected, &actual), Ok(()));
    }

    #[test]
    fn a_one_character_visible_text_change_fails() {
        let error = compare(&page("<h1>Stable</h1>"), &page("<h1>Stabke</h1>")).unwrap_err();
        assert!(error.contains("Stab"));
    }

    #[test]
    fn a_selector_or_class_change_fails() {
        let error = compare(
            &page("<div class=\"record-section\">x</div>"),
            &page("<div class=\"meta\">x</div>"),
        )
        .unwrap_err();
        assert!(error.contains("class"));
    }

    #[test]
    fn an_attribute_change_fails() {
        let error = compare(
            &page("<input name=\"a\" type=\"text\">"),
            &page("<input name=\"a\" type=\"password\">"),
        )
        .unwrap_err();
        assert!(error.contains("type"));
    }

    #[test]
    fn an_added_removed_or_reordered_form_field_fails() {
        let baseline =
            page("<form><input name=\"a\" type=\"text\"><input name=\"b\" type=\"text\"></form>");
        let added = page(
            "<form><input name=\"a\" type=\"text\"><input name=\"b\" type=\"text\"><input name=\"c\" type=\"text\"></form>",
        );
        let removed = page("<form><input name=\"a\" type=\"text\"></form>");
        let reordered =
            page("<form><input name=\"b\" type=\"text\"><input name=\"a\" type=\"text\"></form>");
        assert!(compare(&baseline, &added).is_err());
        assert!(compare(&baseline, &removed).is_err());
        assert!(compare(&baseline, &reordered).is_err());
    }

    #[test]
    fn a_changed_link_target_fails() {
        let error = compare(
            &page("<a href=\"/doc/REQ-001\">View</a>"),
            &page("<a href=\"/doc/REQ-002\">View</a>"),
        )
        .unwrap_err();
        assert!(error.contains("/doc/REQ-002"));
    }

    #[test]
    fn a_relocated_page_title_is_compared_even_when_a_subtree_allowance_hides_it() {
        let expected = page("<main><span></span><h1>Stable</h1></main>");
        let actual = page("<main><h1>Stable</h1></main>");
        let allow = Allowlist::new(vec![
            AllowedDelta {
                case: "synthetic".to_string(),
                kind: AllowKind::Element,
                expected: Some(element_path_with_key(&expected, "start:span")),
                actual: None,
                name: None,
                reason: "seed-presentation".to_string(),
                delta: "item-16".to_string(),
            },
            AllowedDelta {
                case: "synthetic".to_string(),
                kind: AllowKind::Subtree,
                expected: Some(element_path_with_key(&expected, "start:h1")),
                actual: None,
                name: None,
                reason: "seed-presentation".to_string(),
                delta: "item-16".to_string(),
            },
            AllowedDelta {
                case: "synthetic".to_string(),
                kind: AllowKind::Subtree,
                expected: None,
                actual: Some(element_path_with_key(&actual, "start:h1")),
                name: None,
                reason: "seed-presentation".to_string(),
                delta: "item-16".to_string(),
            },
        ]);
        // The page helper's title is "T", so give both pages a matching title.
        let expected =
            expected.replace("<title>T</title>", "<title>Papyrus \u{2014} Stable</title>");
        let actual = actual.replace("<title>T</title>", "<title>Papyrus \u{2014} Stable</title>");
        assert_eq!(
            compare_document_html("synthetic", &expected, &actual, &allow),
            Ok(())
        );
        let mutated = actual.replace(">Stable<", ">Changed<");
        let error = compare_document_html("synthetic", &expected, &mutated, &allow).unwrap_err();
        assert!(error.contains("page title text"), "{error}");
    }

    fn element_path_with_key(html: &str, key: &str) -> String {
        tokenize(html)
            .into_iter()
            .find(|token| token.key == key && token.kind == TokenKind::Start)
            .map(|token| token.path)
            .unwrap_or_else(|| panic!("missing {key}"))
    }

    #[test]
    fn a_list_page_seed_label_allowance_must_not_cite_item_9() {
        let deltas = include_str!("../fixtures/characterization/corpus/EXPECTED-DELTAS.md");
        let list_label = |reason: &str, delta: &str| AllowedDelta {
            case: "http/list-req".to_string(),
            kind: AllowKind::Attr,
            expected: Some("0.1.0.1.1.3".to_string()),
            actual: Some("0.1.0.1.1.3".to_string()),
            name: Some("aria-label".to_string()),
            reason: reason.to_string(),
            delta: delta.to_string(),
        };
        assert!(
            !list_label("seed-presentation", "item-9").is_documented(deltas),
            "a list-page seed label must not cite the edit-form title delta"
        );
        assert!(
            !list_label("uniform-title-block", "item-9").is_documented(deltas),
            "a list page has no title relocation allowance"
        );
        assert!(
            list_label("seed-presentation", "item-16").is_documented(deltas),
            "a seed-label change is documented by item 16"
        );
    }

    #[test]
    fn a_node_allowance_hides_only_that_node_part_and_must_be_used() {
        let allow = Allowlist::new(vec![AllowedDelta {
            case: "synthetic".to_string(),
            kind: AllowKind::Text,
            expected: Some("0.1.0#text0".to_string()),
            actual: Some("0.1.0#text0".to_string()),
            name: None,
            reason: "seed-presentation".to_string(),
            delta: "item-16".to_string(),
        }]);
        assert_eq!(
            compare_document_html(
                "synthetic",
                &page("<aside class=\"sidebar\">Old</aside><h1>Same</h1>"),
                &page("<aside class=\"sidebar\">New</aside><h1>Same</h1>"),
                &allow
            ),
            Ok(())
        );
        let unused = compare_document_html(
            "synthetic",
            &page("<aside class=\"sidebar\">Same</aside><h1>Same</h1>"),
            &page("<aside class=\"sidebar\">Same</aside><h1>Same</h1>"),
            &allow,
        )
        .unwrap_err();
        assert!(unused.contains("unused allowlist entry"));
    }
}
