#![allow(dead_code)] // Each integration test binary compiles this shared module.
//! Test-only semantic HTML comparator (MS-035 TASK-269).
//!
//! Compares two rendered document pages without an HTML parser dependency. The
//! comparison is deliberately semantic rather than byte-oriented:
//!
//! * tags and their sorted attributes are compared (selectors, ids, classes,
//!   `name`, `type`, `value`, `required`, `aria-describedby`, `aria-current`,
//!   `href`, roles and every other hook),
//! * visible text is compared with whitespace collapsed and entities decoded,
//! * a difference in any element, attribute, text, link, or field fails.
//!
//! Only the differences that `EXPECTED-DELTAS.md` attributes may be excluded,
//! through an explicit per-case selector allowlist. An allowlist entry that
//! does not actually hide a difference fails the comparison.

/// One allowed difference: a case, an element selector, and the reason code
/// plus the `EXPECTED-DELTAS.md` item it implements.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AllowedDelta {
    pub case: &'static str,
    pub selector: &'static str,
    pub reason: Reason,
    pub delta: &'static str,
}

/// The three attributed reason codes from MS-035 TASK-269.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reason {
    /// ADR-039 uniform document head/title and generic renderer structure.
    UniformTitleBlock,
    /// A label, heading, help, order, slot, alias, or reference-visibility
    /// value that follows the seed.
    SeedPresentation,
    /// The MS-032 `EXPECTED-DELTAS.md` item 6 conflict/not-found collapse.
    ExpectedDelta6,
}

impl Reason {
    pub fn code(self) -> &'static str {
        match self {
            Reason::UniformTitleBlock => "uniform-title-block",
            Reason::SeedPresentation => "seed-presentation",
            Reason::ExpectedDelta6 => "expected-delta-6",
        }
    }
}

/// The explicit allowlist. Only entries whose `case` equals the compared case
/// apply. Every entry must hide at least one real difference.
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

/// Compare one document page semantically. `Ok(())` means every non-allowed
/// token matched; `Err` names the first difference or an unused allowlist
/// entry.
pub fn compare_document_html(
    case: &str,
    expected: &str,
    actual: &str,
    allowlist: &Allowlist,
) -> Result<(), String> {
    let expected_tokens = tokenize(expected);
    let actual_tokens = tokenize(actual);
    let entries = allowlist.for_case(case);

    // Every allowlist entry must be necessary: removing it from the union must
    // make the comparison fail. An entry that hides nothing is a bug.
    for (index, entry) in entries.iter().enumerate() {
        let without: Vec<&str> = entries
            .iter()
            .enumerate()
            .filter(|(other, _)| *other != index)
            .map(|(_, entry)| entry.selector)
            .collect();
        let expected_without = filter(&expected_tokens, &without);
        let actual_without = filter(&actual_tokens, &without);
        if same_keys(&expected_without, &actual_without) {
            return Err(format!(
                "{case}: unused allowlist entry selector `{}` ({}, {})",
                entry.selector,
                entry.reason.code(),
                entry.delta
            ));
        }
    }

    let selectors: Vec<&str> = entries.iter().map(|entry| entry.selector).collect();
    let expected_visible = filter(&expected_tokens, &selectors);
    let actual_visible = filter(&actual_tokens, &selectors);
    if same_keys(&expected_visible, &actual_visible) {
        return Ok(());
    }
    Err(first_difference(case, &expected_visible, &actual_visible))
}

/// Whether `selector`'s subtree (element and descendants) differs between
/// the two pages. Used to build an explicit per-case allowlist that contains
/// only selectors that actually hide a difference.
pub fn selector_differs(expected: &str, actual: &str, selector: &str) -> bool {
    let expected_tokens = tokenize(expected);
    let actual_tokens = tokenize(actual);
    !same_keys(
        &scope(&expected_tokens, selector),
        &scope(&actual_tokens, selector),
    )
}

/// The tokens that belong to the selected element (or its descendants).
fn scope(tokens: &[Token], selector: &str) -> Vec<Token> {
    tokens
        .iter()
        .filter(|token| {
            token
                .element
                .is_some_and(|element| selector_matches(selector, &tokens[element]))
                || token
                    .ancestors
                    .iter()
                    .any(|ancestor| selector_matches(selector, &tokens[*ancestor]))
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

fn describe(token: &Token, _tokens: &[Token]) -> String {
    token.key.clone()
}

fn first_difference(case: &str, expected: &[Token], actual: &[Token]) -> String {
    let limit = expected.len().max(actual.len());
    for index in 0..limit {
        match (expected.get(index), actual.get(index)) {
            (Some(left), Some(right)) if left.key == right.key => {}
            (Some(left), Some(right)) => {
                return format!(
                    "{case}: semantic HTML differs at token {index}:\n  expected: {}\n  actual:   {}",
                    describe(left, expected),
                    describe(right, actual)
                );
            }
            (Some(left), None) => {
                return format!(
                    "{case}: semantic HTML has an extra expected token {index}: {}",
                    left.key
                );
            }
            (None, Some(right)) => {
                return format!(
                    "{case}: semantic HTML has an extra actual token {index}: {}",
                    right.key
                );
            }
            (None, None) => break,
        }
    }
    format!("{case}: semantic HTML differs in length")
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Token {
    key: String,
    /// The start-token index of this token's element (`None` for text).
    element: Option<usize>,
    /// Start-token indices of the open ancestors, outermost first.
    ancestors: Vec<usize>,
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

fn tokenize(html: &str) -> Vec<Token> {
    let bytes = html.as_bytes();
    let mut index = 0usize;
    let mut tokens: Vec<Token> = Vec::new();
    let mut stack: Vec<(usize, String)> = Vec::new();
    while index < bytes.len() {
        if bytes[index] != b'<' {
            let next = html[index..]
                .find('<')
                .map(|offset| index + offset)
                .unwrap_or(bytes.len());
            let raw = &html[index..next];
            if let Some(text) = canonical_text(raw) {
                tokens.push(Token {
                    key: format!("text:{text}"),
                    element: None,
                    ancestors: stack.iter().map(|(index, _)| *index).collect(),
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
            let position = stack.iter().rposition(|(_, open)| open == &name);
            let element = position.map(|position| stack[position].0);
            if let Some(position) = position {
                stack.truncate(position);
            }
            tokens.push(Token {
                key: format!("end:{name}"),
                element,
                ancestors: stack.iter().map(|(index, _)| *index).collect(),
            });
            continue;
        }
        let (tag, next, self_closing) = parse_start_tag(html, index);
        index = next;
        let name = tag.name.clone();
        let token_index = tokens.len();
        tokens.push(Token {
            key: start_key(&tag),
            element: Some(token_index),
            ancestors: stack.iter().map(|(index, _)| *index).collect(),
        });
        if !VOID_ELEMENTS.contains(&name.as_str()) && !self_closing {
            stack.push((token_index, name));
        } else if !VOID_ELEMENTS.contains(&name.as_str()) && self_closing {
            tokens.push(Token {
                key: format!("end:{name}"),
                element: Some(token_index),
                ancestors: stack.iter().map(|(index, _)| *index).collect(),
            });
        }
    }
    tokens
}

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

fn start_key(tag: &StartTag) -> String {
    let mut attrs = tag.attrs.clone();
    attrs.sort();
    let mut out = format!("start:{}", tag.name);
    for (name, value) in attrs {
        if value.is_empty() {
            out.push_str(&format!(" {name}"));
        } else {
            out.push_str(&format!(" {name}={value:?}"));
        }
    }
    out
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

/// Whether the two pages are semantic-equal when every selector in
/// `selectors` is hidden. Used to prune redundant allowlist entries.
pub fn equal_after_filter(expected: &str, actual: &str, selectors: &[&str]) -> bool {
    let expected_tokens = tokenize(expected);
    let actual_tokens = tokenize(actual);
    same_keys(
        &filter(&expected_tokens, selectors),
        &filter(&actual_tokens, selectors),
    )
}

fn filter(tokens: &[Token], selectors: &[&str]) -> Vec<Token> {
    if selectors.is_empty() {
        return tokens.to_vec();
    }
    tokens
        .iter()
        .filter(|token| {
            !selectors.iter().any(|selector| {
                token
                    .element
                    .is_some_and(|element| selector_matches(selector, &tokens[element]))
                    || token
                        .ancestors
                        .iter()
                        .any(|ancestor| selector_matches(selector, &tokens[*ancestor]))
            })
        })
        .cloned()
        .collect()
}

/// Match a small, explicit selector grammar against a start token:
/// `tag`, `.class`, `#id`, and `tag.class` combinations.
fn selector_matches(selector: &str, token: &Token) -> bool {
    let Some(rest) = token.key.strip_prefix("start:") else {
        return false;
    };
    let tag = rest.split_whitespace().next().unwrap_or("");
    let attrs = parse_key_attrs(rest);
    let mut expected_tag = String::new();
    let mut classes: Vec<&str> = Vec::new();
    let mut id: Option<&str> = None;
    let mut remainder = selector;
    let mut consumed_tag = false;
    while !remainder.is_empty() {
        if let Some(after) = remainder.strip_prefix('.') {
            let end = after.find(['.', '#']).unwrap_or(after.len());
            classes.push(&after[..end]);
            remainder = &after[end..];
        } else if let Some(after) = remainder.strip_prefix('#') {
            let end = after.find(['.', '#']).unwrap_or(after.len());
            id = Some(&after[..end]);
            remainder = &after[end..];
        } else {
            let end = remainder.find(['.', '#']).unwrap_or(remainder.len());
            expected_tag = remainder[..end].to_string();
            remainder = &remainder[end..];
            consumed_tag = true;
        }
    }
    if !consumed_tag {
        expected_tag = "*".to_string();
    }
    if expected_tag != "*" && expected_tag != tag {
        return false;
    }
    if let Some(id) = id
        && attrs.get("id").map(String::as_str) != Some(id)
    {
        return false;
    }
    if !classes.is_empty() {
        let found = attrs.get("class").map(String::as_str).unwrap_or("");
        let found: Vec<&str> = found.split_whitespace().collect();
        if !classes.iter().all(|class| found.contains(class)) {
            return false;
        }
    }
    true
}

fn parse_key_attrs(rest: &str) -> std::collections::BTreeMap<String, String> {
    let mut attrs = std::collections::BTreeMap::new();
    let mut lexer = rest.split_whitespace();
    let _tag = lexer.next();
    for piece in lexer {
        if let Some((name, value)) = piece.split_once('=') {
            let value = value.trim_matches('"').to_string();
            attrs.insert(name.to_string(), value);
        } else {
            attrs.insert(piece.to_string(), String::new());
        }
    }
    attrs
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
    fn an_allowlisted_selector_hides_only_its_subtree_and_must_be_used() {
        let allow = Allowlist::new(vec![AllowedDelta {
            case: "synthetic",
            selector: "aside.sidebar",
            reason: Reason::SeedPresentation,
            delta: "item-16",
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
