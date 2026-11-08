//! Leading YAML front-matter detection, parsing, and patching.

use crate::error::AppError;
use std::str;

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
    let current = current_value_string(raw, key);
    if value.is_none() && !contains_key(raw, key) {
        return (raw.to_string(), false);
    }
    if let Some(v) = value
        && contains_key(raw, key)
        && current.as_deref() == Some(v)
    {
        return (raw.to_string(), false);
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

fn format_kv(key: &str, value: &str) -> String {
    let v = value.trim_end_matches('\n');
    if v.contains('\n') {
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
    fn no_op_patch_raw_returns_same_and_false() {
        let raw = "id: X\ntitle: T\nforeign: 42";
        let (out, changed) = patch_raw(raw, "title", Some("T"));
        assert_eq!(out, raw);
        assert!(!changed);
    }
}
