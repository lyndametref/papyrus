//! Leading YAML front-matter detection, parsing, and patching.

use crate::error::AppError;
use std::str;

/// Typed front-matter value for unambiguous serialization.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FrontMatterValue {
    Null,
    Integer(i64),
    /// An explicit YAML sequence, including the empty sequence.
    Sequence(Vec<String>),
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
    let data = yaml_serde::from_str::<yaml_serde::Value>(raw).ok()?;
    value_string(&data, key)
}

/// Render one already-parsed front-matter value as a plain string.
///
/// Callers that already hold a [`FrontMatter`] use this to avoid parsing the
/// same YAML a second time just to read one key.
pub fn value_string(data: &yaml_serde::Value, key: &str) -> Option<String> {
    let yaml_serde::Value::Mapping(mapping) = data else {
        return None;
    };
    let value = mapping.get(yaml_serde::Value::String(key.to_string()))?;
    Some(match value {
        yaml_serde::Value::String(s) => s.clone(),
        yaml_serde::Value::Bool(b) => b.to_string(),
        yaml_serde::Value::Number(n) => n.to_string(),
        yaml_serde::Value::Null => String::new(),
        other => yaml_serde::to_string(other)
            .unwrap_or_default()
            .trim()
            .to_string(),
    })
}

fn double_quote(t: &str) -> String {
    let mut out = String::from("\"");
    for ch in t.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '\"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => {
                let code = c as u32;
                if code <= 0xFFFF {
                    out.push_str(&format!("\\u{:04X}", code));
                } else {
                    out.push_str(&format!("\\U{:08X}", code));
                }
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Whether a plain (unquoted) rendering parses back to exactly `t` as a
/// scalar string. This is the final guard against indicator prefixes, flow
/// collection punctuation, and resolver keywords that would otherwise change
/// the scalar's type or value.
fn plain_round_trips(t: &str) -> bool {
    let Ok(yaml_serde::Value::Mapping(mapping)) =
        yaml_serde::from_str::<yaml_serde::Value>(&format!("__key__: {t}\n"))
    else {
        return false;
    };
    matches!(
        mapping.get(yaml_serde::Value::String("__key__".to_string())),
        Some(yaml_serde::Value::String(value)) if value == t
    )
}

fn starts_with_yaml_indicator(t: &str) -> bool {
    matches!(
        t.chars().next(),
        Some(
            '-' | '?'
                | ':'
                | ','
                | '['
                | ']'
                | '{'
                | '}'
                | '#'
                | '&'
                | '*'
                | '!'
                | '|'
                | '>'
                | '\''
                | '"'
                | '%'
                | '@'
                | '`'
                | '~'
        )
    )
}

fn format_scalar(v: &str) -> String {
    let t = v.trim();
    if t.is_empty() {
        return "\"\"".to_string();
    }
    // @-prefixed scalars must be quoted (YAML forbids unquoted leading @).
    if t.starts_with('@') {
        return format!("'{}'", t.replace('\'', "''"));
    }
    // timestamp quoting keeps the historical single-quoted form
    if is_timestamp(t) {
        return format!("'{}'", t);
    }
    // bool/null/number keywords would change type if left plain
    let lower = t.to_lowercase();
    if lower == "true" || lower == "false" || lower == "null" || lower == "~" {
        return double_quote(t);
    }
    if t.parse::<f64>().is_ok() {
        return double_quote(t);
    }
    // Document markers are only valid as quoted scalars.
    if t == "---" || t == "..." {
        return double_quote(t);
    }
    // Quote every indicator-leading or flow-collection-bearing value, every
    // value containing a line break or control character, and anything whose
    // plain rendering would not parse back to the same string.
    let structurally_unsafe = starts_with_yaml_indicator(t)
        || t.chars().any(|c| c.is_control())
        || t.contains('\n')
        || t.contains('\r')
        || t.contains(':')
        || t.contains('#')
        || t.contains(' ')
        || t.contains('\'')
        || t.contains('"')
        || t.contains('[')
        || t.contains(']')
        || t.contains('{')
        || t.contains('}')
        || t.contains(',')
        || !plain_round_trips(t);
    if structurally_unsafe {
        return double_quote(t);
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
    } else {
        // A scalar key is never coerced into a sequence: embedded line breaks
        // are escaped inside one quoted scalar.
        format!("{}: {}", key, format_scalar(v))
    }
}

/// Format a typed front-matter key/value pair into raw YAML text.
pub fn format_kv_value(key: &str, value: &FrontMatterValue) -> String {
    match value {
        FrontMatterValue::Null => format!("{}: null", key),
        FrontMatterValue::Integer(n) => format!("{}: {}", key, n),
        FrontMatterValue::Sequence(items) => format_sequence(key, items),
        FrontMatterValue::String(s) => format_kv(key, s),
    }
}

/// Re-parse one rendered key and confirm it has exactly the intended type and
/// value. A non-round-tripping render is a validation error, so an unsafe
/// scalar is never written.
pub fn verify_value(raw: &str, key: &str, value: &FrontMatterValue) -> Result<(), AppError> {
    let parsed: yaml_serde::Value = yaml_serde::from_str(raw).map_err(|error| {
        AppError::Validation(format!(
            "rendered front matter for '{}' is not valid YAML: {}",
            key, error
        ))
    })?;
    let yaml_serde::Value::Mapping(mapping) = parsed else {
        return Err(AppError::Validation(format!(
            "rendered front matter for '{}' is not a mapping",
            key
        )));
    };
    let actual = mapping.get(yaml_serde::Value::String(key.to_string()));
    let round_trips = match (value, actual) {
        (FrontMatterValue::Null, Some(yaml_serde::Value::Null)) => true,
        (FrontMatterValue::Integer(expected), Some(yaml_serde::Value::Number(number))) => {
            number.as_i64() == Some(*expected)
        }
        (FrontMatterValue::Sequence(expected), Some(yaml_serde::Value::Sequence(actual))) => {
            actual.len() == expected.len()
                && expected.iter().zip(actual).all(|(expected, actual)| {
                    matches!(actual, yaml_serde::Value::String(actual) if actual == expected)
                })
        }
        (FrontMatterValue::String(expected), Some(yaml_serde::Value::String(actual))) => {
            actual == expected
        }
        _ => false,
    };
    if round_trips {
        Ok(())
    } else {
        Err(AppError::Validation(format!(
            "rendered front matter value for '{}' does not round-trip",
            key
        )))
    }
}

/// Format an explicit YAML sequence, including a single item and the empty
/// sequence.
fn format_sequence(key: &str, items: &[String]) -> String {
    if items.is_empty() {
        return format!("{}: []", key);
    }
    let mut out = format!("{}:", key);
    for item in items {
        out.push_str(&format!("\n  - {}", format_scalar(item)));
    }
    out
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

/// Parse an ISO-8601 UTC timestamp with a `Z` or `±HH:MM` offset into UTC
/// seconds since the Unix epoch.
///
/// This is the inverse of [`utc_from_unix`]: the accepted shape is
/// `YYYY-MM-DDTHH:MM:SS` followed by `Z` or `±HH:MM`. Values outside the
/// calendar or clock ranges resolve to `None`.
pub fn timestamp_seconds(value: &str) -> Option<i64> {
    let raw = value.trim();
    if !raw.is_ascii() {
        return None;
    }
    let bytes = raw.as_bytes();
    let offset = match raw.len() {
        20 if bytes[19] == b'Z' => 0,
        25 => {
            let sign: i64 = match bytes[19] {
                b'+' => 1,
                b'-' => -1,
                _ => return None,
            };
            if bytes[22] != b':' {
                return None;
            }
            let hours = parse_digits(&bytes[20..22])?;
            let minutes = parse_digits(&bytes[23..25])?;
            if hours > 23 || minutes > 59 {
                return None;
            }
            sign * (i64::from(hours) * 3600 + i64::from(minutes) * 60)
        }
        _ => return None,
    };
    if bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes[10] != b'T'
        || bytes[13] != b':'
        || bytes[16] != b':'
    {
        return None;
    }
    let year = parse_digits(&bytes[0..4])? as i64;
    let month = parse_digits(&bytes[5..7])?;
    let day = parse_digits(&bytes[8..10])?;
    if !(1..=12).contains(&month) || day < 1 || day > days_in_month(year, month) {
        return None;
    }
    let hour = parse_digits(&bytes[11..13])?;
    let minute = parse_digits(&bytes[14..16])?;
    let second = parse_digits(&bytes[17..19])?;
    if hour > 23 || minute > 59 || second > 59 {
        return None;
    }
    let seconds = days_from_civil(year, month, day) * 86_400
        + i64::from(hour) * 3_600
        + i64::from(minute) * 60
        + i64::from(second);
    Some(seconds - offset)
}

fn parse_digits(bytes: &[u8]) -> Option<u32> {
    let mut value = 0u32;
    for &byte in bytes {
        if !byte.is_ascii_digit() {
            return None;
        }
        value = value.checked_mul(10)?.checked_add(u32::from(byte - b'0'))?;
    }
    Some(value)
}

fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn days_in_month(year: i64, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
    // Howard Hinnant's days_from_civil, the inverse of `civil_from_unix`.
    let year = if month <= 2 { year - 1 } else { year };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let month_prime = if month > 2 {
        i64::from(month) - 3
    } else {
        i64::from(month) + 9
    };
    let day_of_year = (153 * month_prime + 2) / 5 + i64::from(day) - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
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
    fn timestamp_seconds_round_trips_utc() {
        assert_eq!(timestamp_seconds("1970-01-01T00:00:00Z"), Some(0));
        for value in [
            "2026-01-01T00:00:00Z",
            "2026-11-15T17:30:00Z",
            "2000-02-29T23:59:59Z",
            "1969-12-31T23:59:59Z",
        ] {
            let seconds = timestamp_seconds(value).expect(value);
            assert_eq!(utc_from_unix(seconds), value);
        }
    }

    #[test]
    fn timestamp_seconds_compares_offsets_as_instants() {
        let utc = timestamp_seconds("2026-01-01T10:00:00Z").unwrap();
        let ahead = timestamp_seconds("2026-01-01T12:00:00+02:00").unwrap();
        let behind = timestamp_seconds("2026-01-01T05:00:00-05:00").unwrap();
        assert_eq!(utc, ahead);
        assert_eq!(utc, behind);
    }

    #[test]
    fn timestamp_seconds_rejects_invalid_values() {
        for value in [
            "",
            "not a date",
            "2026-13-01T00:00:00Z",
            "2026-02-30T00:00:00Z",
            "2026-01-01T25:00:00Z",
            "2026-01-01T00:61:00Z",
            "2026-01-01T00:00:60Z",
            "2026-01-01T00:00:00",
            "2026-01-01T00:00:00+2:00",
            "2026-01-01T00:00:00X",
        ] {
            assert_eq!(timestamp_seconds(value), None, "{:?}", value);
        }
    }

    fn parse_scalar(rendered: &str, key: &str) -> yaml_serde::Value {
        let parsed: yaml_serde::Value = yaml_serde::from_str(rendered).unwrap();
        let yaml_serde::Value::Mapping(mapping) = parsed else {
            panic!("not a mapping: {rendered}");
        };
        mapping
            .get(yaml_serde::Value::String(key.to_string()))
            .cloned()
            .unwrap_or(yaml_serde::Value::Null)
    }

    #[test]
    fn yaml_indicator_scalars_round_trip_as_strings() {
        for value in [
            "*bold",
            "[a]",
            "&x",
            "!x",
            "|x",
            "?x",
            "-x",
            "{}",
            "---",
            "...",
            "a[b]c",
            "a,b",
            "true",
            "false",
            "null",
            "123",
            "1.5",
            "@handle",
            "2026-01-01T00:00:00Z",
        ] {
            let rendered = format_kv_value("title", &FrontMatterValue::String(value.to_string()));
            assert_eq!(
                parse_scalar(&rendered, "title"),
                yaml_serde::Value::String(value.to_string()),
                "rendered {rendered:?}"
            );
        }
    }

    #[test]
    fn scalar_never_coerces_to_a_sequence() {
        let rendered = format_kv_value(
            "note",
            &FrontMatterValue::String("first line\nsecond line".to_string()),
        );
        assert!(!rendered.contains("\n  - "), "{rendered}");
        assert_eq!(
            parse_scalar(&rendered, "note"),
            yaml_serde::Value::String("first line\nsecond line".to_string())
        );
        let legacy = format_kv("note", "first line\nsecond line");
        assert!(!legacy.contains("\n  - "), "{legacy}");
        assert_eq!(
            parse_scalar(&legacy, "note"),
            yaml_serde::Value::String("first line\nsecond line".to_string())
        );
    }

    #[test]
    fn sequences_keep_bracketed_and_empty_items() {
        let rendered = format_kv_value(
            "labels",
            &FrontMatterValue::Sequence(vec!["[]".to_string(), "core".to_string()]),
        );
        assert_eq!(
            parse_scalar(&rendered, "labels"),
            yaml_serde::Value::Sequence(vec![
                yaml_serde::Value::String("[]".to_string()),
                yaml_serde::Value::String("core".to_string()),
            ])
        );
        assert_eq!(
            format_kv_value("labels", &FrontMatterValue::Sequence(Vec::new())),
            "labels: []"
        );
    }

    #[test]
    fn verify_value_rejects_a_non_round_tripping_render() {
        let error = verify_value(
            "title: [broken]",
            "title",
            &FrontMatterValue::String("[broken]".to_string()),
        )
        .expect_err("a sequence render for a string must fail");
        assert!(matches!(error, crate::error::AppError::Validation(_)));
    }

    #[test]
    fn no_op_patch_raw_returns_same_and_false() {
        let raw = "id: X\ntitle: T\nforeign: 42";
        let (out, changed) = patch_raw(raw, "title", Some("T"));
        assert_eq!(out, raw);
        assert!(!changed);
    }
}
