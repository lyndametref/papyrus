//! Leading YAML front-matter detection, parsing, and patching.

use crate::error::AppError;
use std::str;

/// Typed front-matter value for unambiguous serialization.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FrontMatterValue {
    Null,
    Integer(i64),
    String(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct FrontMatter {
    pub raw: String,
    pub text: String,
    pub value: yaml_serde::Value,
    pub data: yaml_serde::Value,
}

/// Whether `content` begins with a leading YAML block.
pub fn has_front_matter(content: &str) -> bool {
    find_block(content).is_some()
}

pub fn detect(content: &str) -> bool {
    has_front_matter(content)
}

/// Locate the leading block. Returns (inner_start_byte, inner_end_byte, body_start_byte).
pub fn find_block(content: &str) -> Option<(usize, usize, usize)> {
    if !content.starts_with("---") {
        return None;
    }
    // opening delimiter must be exactly "---" on the first line
    let first_line_end = content.find('\n').unwrap_or(content.len());
    let first_line = &content[..first_line_end];
    let first_line = first_line.strip_suffix('\r').unwrap_or(first_line);
    if first_line != "---" {
        return None;
    }

    let mut cursor = first_line_end;
    if cursor < content.len() {
        cursor += 1; // skip newline
    }
    // find closing delimiter
    let mut closing_start = None;
    let mut pos = cursor;
    while pos < content.len() {
        let line_end = content[pos..]
            .find('\n')
            .map(|i| pos + i)
            .unwrap_or(content.len());
        let line = &content[pos..line_end];
        let line = line.strip_suffix('\r').unwrap_or(line);
        if line == "---" {
            closing_start = Some((pos, line_end));
            break;
        }
        if line_end >= content.len() {
            break;
        }
        pos = line_end + 1;
    }
    let (cs, ce) = closing_start?;
    // inner is between opening newline and closing delimiter
    let inner_start = cursor;
    let inner_end = cs;
    // body starts after closing delimiter line's newline
    let mut body_start = ce;
    if body_start < content.len() {
        body_start += 1; // skip newline
    }
    Some((inner_start, inner_end, body_start))
}

/// Parse the leading block if present.
pub fn parse(content: &str) -> Result<Option<FrontMatter>, AppError> {
    let Some((is, ie, _)) = find_block(content) else {
        return Ok(None);
    };
    let inner = &content[is..ie];
    // trim a single trailing newline for storage
    let raw = if let Some(stripped) = inner.strip_suffix('\n') {
        stripped.to_string()
    } else {
        inner.to_string()
    };
    let value: yaml_serde::Value = yaml_serde::from_str(&raw)
        .map_err(|e| AppError::Validation(format!("malformed front matter: {}", e)))?;
    Ok(Some(FrontMatter {
        text: raw.clone(),
        raw,
        value: value.clone(),
        data: value,
    }))
}

pub fn parse_block(content: &str) -> Result<Option<(FrontMatter, usize)>, AppError> {
    let Some((is, ie, bs)) = find_block(content) else {
        return Ok(None);
    };
    let inner = &content[is..ie];
    let raw = if let Some(stripped) = inner.strip_suffix('\n') {
        stripped.to_string()
    } else {
        inner.to_string()
    };
    let value: yaml_serde::Value = yaml_serde::from_str(&raw)
        .map_err(|e| AppError::Validation(format!("malformed front matter: {}", e)))?;
    Ok(Some((
        FrontMatter {
            text: raw.clone(),
            raw,
            value: value.clone(),
            data: value,
        },
        bs,
    )))
}

/// Split content into front matter (if any) and the remaining body text.
pub fn split(content: &str) -> Result<(Option<FrontMatter>, String), AppError> {
    if let Some((fm, bs)) = parse_block(content)? {
        Ok((Some(fm), content[bs..].to_string()))
    } else {
        Ok((None, content.to_string()))
    }
}

/// Patch a single key in the raw front-matter text. `value = None` clears.
/// Returns (new_raw, changed).
pub fn patch_raw(raw: &str, key: &str, value: Option<&str>) -> (String, bool) {
    // unchanged check
    if value.is_none() && !contains_key(raw, key) {
        return (raw.to_string(), false);
    }
    if let Some(v) = value
        && contains_key(raw, key)
    {
        // Scalar no-op.
        if current_value_string(raw, key).as_deref() == Some(v) {
            return (raw.to_string(), false);
        }
        // List no-op: compare the fully formatted key block.
        if current_key_block(raw, key).as_deref() == Some(format_kv(key, v).as_str()) {
            return (raw.to_string(), false);
        }
    }

    let lines: Vec<&str> = raw.split('\n').collect();
    // find top-level key span
    let mut start_idx: Option<usize> = None;
    for (i, line) in lines.iter().enumerate() {
        if line.starts_with(char::is_whitespace) || line.trim().is_empty() {
            continue;
        }
        if let Some((k, _)) = line.split_once(':')
            && k.trim() == key
        {
            start_idx = Some(i);
            break;
        }
    }

    if let Some(si) = start_idx {
        let mut ei = si + 1;
        while ei < lines.len() {
            if lines[ei].trim().is_empty() {
                break;
            }
            if !lines[ei].starts_with(char::is_whitespace) {
                break;
            }
            ei += 1;
        }
        let mut out: Vec<String> = lines[..si].iter().map(|s| s.to_string()).collect();
        if let Some(v) = value {
            for l in format_kv(key, v).lines() {
                out.push(l.to_string());
            }
        }
        out.extend(lines[ei..].iter().map(|s| s.to_string()));
        (out.join("\n"), true)
    } else {
        if let Some(v) = value {
            let mut out = raw.to_string();
            if !out.is_empty() && !out.ends_with('\n') {
                out.push('\n');
            }
            out.push_str(&format_kv(key, v));
            (out, true)
        } else {
            (raw.to_string(), false)
        }
    }
}

pub fn contains_key(raw: &str, key: &str) -> bool {
    for line in raw.lines() {
        if line.starts_with(char::is_whitespace) || line.trim().is_empty() {
            continue;
        }
        if let Some((k, _)) = line.split_once(':')
            && k.trim() == key
        {
            return true;
        }
    }
    false
}

pub fn current_key_block(raw: &str, key: &str) -> Option<String> {
    let lines: Vec<&str> = raw.split('\n').collect();
    let mut start_idx: Option<usize> = None;
    for (i, line) in lines.iter().enumerate() {
        if line.starts_with(char::is_whitespace) || line.trim().is_empty() {
            continue;
        }
        if let Some((k, _)) = line.split_once(':')
            && k.trim() == key
        {
            start_idx = Some(i);
            break;
        }
    }
    let si = start_idx?;
    let mut ei = si + 1;
    while ei < lines.len() {
        if lines[ei].trim().is_empty() {
            break;
        }
        if !lines[ei].starts_with(char::is_whitespace) {
            break;
        }
        ei += 1;
    }
    Some(lines[si..ei].join("\n"))
}

pub fn current_value_string(raw: &str, key: &str) -> Option<String> {
    if let Ok(yaml_serde::Value::Mapping(m)) = yaml_serde::from_str::<yaml_serde::Value>(raw)
        && let Some(v) = m.get(yaml_serde::Value::String(key.to_string()))
    {
        return Some(match v {
            yaml_serde::Value::String(s) => s.clone(),
            yaml_serde::Value::Bool(b) => b.to_string(),
            yaml_serde::Value::Number(n) => n.to_string(),
            yaml_serde::Value::Null => String::new(),
            other => yaml_serde::to_string(other)
                .unwrap_or_default()
                .trim()
                .to_string(),
        });
    }
    None
}

fn format_scalar(v: &str) -> String {
    let t = v.trim();
    if t.is_empty() {
        return "\"\"".to_string();
    }
    // timestamp quoting
    if is_timestamp(t) {
        return format!("'{}'", t);
    }
    // bool/null/number keywords
    let lower = t.to_lowercase();
    if lower == "true" || lower == "false" || lower == "null" || lower == "~" {
        return format!("\"{}\"", t);
    }
    if t.parse::<f64>().is_ok() {
        return format!("\"{}\"", t);
    }
    if t.contains(':') || t.contains('#') || t.contains(' ') || t.contains('\'') || t.contains('"')
    {
        // quote with double quotes, escape backslashes
        let escaped = t.replace('\\', "\\\\").replace('"', "\\\"");
        return format!("\"{}\"", escaped);
    }
    t.to_string()
}

fn is_timestamp(v: &str) -> bool {
    // YYYY-MM-DDTHH:MM:SSZ
    v.len() == 20
        && v.as_bytes()[4] == b'-'
        && v.as_bytes()[7] == b'-'
        && v.as_bytes()[10] == b'T'
        && v.as_bytes()[13] == b':'
        && v.as_bytes()[16] == b':'
        && v.as_bytes()[19] == b'Z'
}

fn is_list_key(key: &str) -> bool {
    matches!(
        key,
        "labels" | "references" | "documentation" | "dependencies" | "assignee"
    )
}

fn format_kv(key: &str, value: &str) -> String {
    let v = value.trim_end_matches('\n');
    if is_list_key(key) {
        // Empty list marker passed through as-is.
        if v.trim() == "[]" {
            return format!("{}: []", key);
        }
        let items: Vec<&str> = v
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect();
        if items.is_empty() {
            return format!("{}: []", key);
        }
        let mut s = format!("{}:", key);
        for item in items {
            s.push_str(&format!("\n  - {}", format_scalar(item)));
        }
        s
    } else if v.contains('\n') {
        let mut s = format!("{}:", key);
        for line in v.lines() {
            let item = line.trim();
            if !item.is_empty() {
                s.push_str(&format!("\n  - {}", item));
            }
        }
        s
    } else {
        format!("{}: {}", key, format_scalar(v))
    }
}

/// Format a typed front-matter key/value pair into raw YAML text.
pub fn format_kv_value(key: &str, value: &FrontMatterValue) -> String {
    match value {
        FrontMatterValue::Null => format!("{}: null", key),
        FrontMatterValue::Integer(n) => format!("{}: {}", key, n),
        FrontMatterValue::String(s) => format_kv(key, s),
    }
}

/// Patch a single key in the raw front-matter text using a typed value.
/// Returns (new_raw, changed).
pub fn patch_raw_value(raw: &str, key: &str, value: &FrontMatterValue) -> (String, bool) {
    let text = format_kv_value(key, value);
    // No-op if the existing block already matches.
    if let Some(block) = current_key_block(raw, key)
        && block == text
    {
        return (raw.to_string(), false);
    }

    let lines: Vec<&str> = raw.split('\n').collect();
    let mut start_idx: Option<usize> = None;
    for (i, line) in lines.iter().enumerate() {
        if line.starts_with(char::is_whitespace) || line.trim().is_empty() {
            continue;
        }
        if let Some((k, _)) = line.split_once(':')
            && k.trim() == key
        {
            start_idx = Some(i);
            break;
        }
    }

    if let Some(si) = start_idx {
        let mut ei = si + 1;
        while ei < lines.len() {
            if lines[ei].trim().is_empty() {
                break;
            }
            if !lines[ei].starts_with(char::is_whitespace) {
                break;
            }
            ei += 1;
        }
        let mut out: Vec<String> = lines[..si].iter().map(|s| s.to_string()).collect();
        out.push(text);
        out.extend(lines[ei..].iter().map(|s| s.to_string()));
        (out.join("\n"), true)
    } else {
        let mut out = raw.to_string();
        if !out.is_empty() && !out.ends_with('\n') {
            out.push('\n');
        }
        out.push_str(&text);
        (out, true)
    }
}

/// Format a single front-matter key/value pair (scalar or list) into raw YAML text.
pub fn format_kv_pub(key: &str, value: &str) -> String {
    format_kv(key, value)
}

/// Build a front-matter raw block from ordered typed entries via the shared patch path.
pub fn build_raw(entries: &[(&str, FrontMatterValue)]) -> String {
    let mut raw = String::new();
    for (key, value) in entries {
        let (next, _) = patch_raw_value(&raw, key, value);
        raw = next;
    }
    raw
}

/// Wrap a raw front-matter block and a body into a complete document.
pub fn wrap(front_matter_raw: &str, body: &str) -> String {
    let mut out = String::new();
    out.push_str("---\n");
    out.push_str(front_matter_raw);
    if !front_matter_raw.ends_with('\n') {
        out.push('\n');
    }
    out.push_str("---\n");
    if !body.is_empty() {
        out.push_str(body);
        if !body.ends_with('\n') {
            out.push('\n');
        }
    } else {
        out.push('\n');
    }
    out
}

/// Current UTC time as quoted ISO-8601 string payload (without quotes).
pub fn utc_now() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let (y, mo, d, h, mi, se) = civil_from_unix(secs);
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", y, mo, d, h, mi, se)
}

pub fn utc_from_unix(secs: i64) -> String {
    let (y, mo, d, h, mi, se) = civil_from_unix(secs);
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", y, mo, d, h, mi, se)
}

pub fn is_timestamp_pub(v: &str) -> bool {
    is_timestamp(v)
}

fn civil_from_unix(secs: i64) -> (i64, u32, u32, u32, u32, u32) {
    let days = secs.div_euclid(86400);
    let rem = secs.rem_euclid(86400);
    let h = (rem / 3600) as u32;
    let mi = ((rem % 3600) / 60) as u32;
    let se = (rem % 60) as u32;
    // Howard Hinnant algorithm
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d, h, mi, se)
}

/// Expose format_scalar for timestamp quoting.
pub fn format_scalar_pub(v: &str) -> String {
    format_scalar(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_and_parse() {
        let content = "---\nid: X\ntitle: T\n---\n\n# X T\n";
        assert!(has_front_matter(content));
        let fm = parse(content).unwrap().unwrap();
        assert_eq!(fm.raw, "id: X\ntitle: T");
    }

    #[test]
    fn malformed() {
        let content = "---\n: bad\n---\n";
        assert!(parse(content).is_err());
    }

    #[test]
    fn unknown_key_preserved_in_raw() {
        let content = "---\nid: X\nforeign: 42\ntitle: T\n---\n\n# X T\n";
        let fm = parse(content).unwrap().unwrap();
        assert!(fm.raw.contains("foreign: 42"));
        let (new_raw, changed) = patch_raw(&fm.raw, "title", Some("New"));
        assert!(changed);
        assert!(new_raw.contains("foreign: 42"));
        assert!(new_raw.contains("id: X"));
    }

    #[test]
    fn malformed_yaml_returns_validation_error() {
        let content = "---\n: bad\n---\n";
        let err = parse(content).unwrap_err();
        assert!(matches!(err, crate::error::AppError::Validation(_)));
    }

    #[test]
    fn has_front_matter_requires_byte_zero_delimiter() {
        let content = "intro\n---\nid: X\n---\n";
        assert!(!has_front_matter(content));
    }

    #[test]
    fn single_list_item_serializes_as_sequence() {
        let raw = "id: X";
        let (out, _) = patch_raw(raw, "labels", Some("foo"));
        assert!(out.contains("labels:\n  - foo"), "got: {}", out);
    }

    #[test]
    fn list_no_op_byte_identical() {
        let raw = "id: X\nlabels:\n  - foo";
        let (out, changed) = patch_raw(raw, "labels", Some("foo"));
        assert!(!changed);
        assert_eq!(out, raw);
    }

    #[test]
    fn no_op_patch_raw_returns_same_and_false() {
        let raw = "id: X\ntitle: T\nforeign: 42";
        let (out, changed) = patch_raw(raw, "title", Some("T"));
        assert_eq!(out, raw);
        assert!(!changed);
    }
}
