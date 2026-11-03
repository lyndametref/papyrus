//! Minimal blocking HTTP/1.1 request parsing and response writing.
//!
//! The server is deliberately small: one request per connection, fixed route
//! matching, explicit limits, and no framework or async runtime.

use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::time::{Duration, Instant};

pub const MAX_REQUEST_LINE: usize = 8192;
pub const MAX_TARGET: usize = 2048;
pub const MAX_HEADER_BLOCK: usize = 16384;
pub const MAX_HEADERS: usize = 64;
pub const MAX_BODY: usize = 1024 * 1024;

pub const CSP: &str = "default-src 'none'; style-src 'self'; script-src 'self'; img-src 'self'; form-action 'self'; base-uri 'none'; frame-ancestors 'none'";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Method {
    Get,
    Head,
    Post,
    Other,
}

impl Method {
    pub fn as_str(self) -> &'static str {
        match self {
            Method::Get => "GET",
            Method::Head => "HEAD",
            Method::Post => "POST",
            Method::Other => "OTHER",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Version {
    Http10,
    Http11,
}

#[derive(Clone, Debug)]
pub struct Target {
    pub path: String,
    pub query: Option<String>,
    pub raw: String,
}

#[derive(Clone, Debug)]
pub struct Request {
    pub method: Method,
    /// The method token exactly as sent, so the request log can print it.
    pub raw_method: String,
    pub target: Target,
    pub version: Version,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl Request {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }

    pub fn header_count(&self, name: &str) -> usize {
        self.headers
            .iter()
            .filter(|(key, _)| key.eq_ignore_ascii_case(name))
            .count()
    }
}

/// An error that maps directly to an HTTP status.
#[derive(Debug)]
pub struct HttpError {
    pub status: u16,
}

impl HttpError {
    pub fn new(status: u16) -> Self {
        Self { status }
    }
}

fn map_io(error: &io::Error) -> HttpError {
    match error.kind() {
        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut => HttpError::new(408),
        _ => HttpError::new(400),
    }
}

fn read_line<R: BufRead>(
    reader: &mut R,
    limit: usize,
    too_long_status: u16,
    deadline: Instant,
) -> Result<Option<Vec<u8>>, HttpError> {
    let mut line: Vec<u8> = Vec::new();
    loop {
        if Instant::now() >= deadline {
            return Err(HttpError::new(408));
        }
        let available = reader.fill_buf().map_err(|error| map_io(&error))?;
        if available.is_empty() {
            if line.is_empty() {
                return Ok(None);
            }
            return Err(HttpError::new(400));
        }
        if let Some(position) = available.iter().position(|byte| *byte == b'\n') {
            line.extend_from_slice(&available[..position]);
            reader.consume(position + 1);
            if line.len() > limit {
                return Err(HttpError::new(too_long_status));
            }
            if line.last() == Some(&b'\r') {
                line.pop();
            }
            return Ok(Some(line));
        }
        line.extend_from_slice(available);
        let consumed = available.len();
        reader.consume(consumed);
        if line.len() > limit {
            return Err(HttpError::new(too_long_status));
        }
    }
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn percent_decode(
    input: &str,
    plus_as_space: bool,
    reject_path_separators: bool,
) -> Result<String, HttpError> {
    let bytes = input.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0usize;
    while i < bytes.len() {
        match bytes[i] {
            b'%' => {
                if i + 2 >= bytes.len() {
                    return Err(HttpError::new(400));
                }
                let Some(high) = hex_value(bytes[i + 1]) else {
                    return Err(HttpError::new(400));
                };
                let Some(low) = hex_value(bytes[i + 2]) else {
                    return Err(HttpError::new(400));
                };
                let decoded = (high << 4) | low;
                // Path decoding rejects encoded NUL and separators. Form and
                // query decoding allow them so field validation can report a
                // 422 for `%00` instead of a malformed-request 400.
                if reject_path_separators && matches!(decoded, 0 | b'/' | b'\\') {
                    return Err(HttpError::new(400));
                }
                out.push(decoded);
                i += 3;
            }
            b'+' if plus_as_space => {
                out.push(b' ');
                i += 1;
            }
            byte => {
                out.push(byte);
                i += 1;
            }
        }
    }
    let decoded = String::from_utf8(out).map_err(|_| HttpError::new(400))?;
    if reject_path_separators && decoded.chars().any(char::is_control) {
        return Err(HttpError::new(400));
    }
    Ok(decoded)
}

fn validate_path(path: &str) -> Result<(), HttpError> {
    if !path.starts_with('/') {
        return Err(HttpError::new(400));
    }
    if path.chars().any(char::is_control) {
        return Err(HttpError::new(400));
    }
    if path == "/" {
        return Ok(());
    }
    let mut segments: Vec<&str> = path[1..].split('/').collect();
    if segments.last() == Some(&"") {
        segments.pop();
    }
    if segments.is_empty() {
        return Err(HttpError::new(400));
    }
    for segment in segments {
        if segment.is_empty() || segment == "." || segment == ".." {
            return Err(HttpError::new(400));
        }
    }
    Ok(())
}

fn parse_target(raw: &str) -> Result<Target, HttpError> {
    if raw.len() > MAX_TARGET {
        return Err(HttpError::new(414));
    }
    if !raw.starts_with('/') {
        return Err(HttpError::new(400));
    }
    let (path_raw, query) = match raw.split_once('?') {
        Some((path, query)) => (path, Some(query.to_string())),
        None => (raw, None),
    };
    let path = percent_decode(path_raw, false, true)?;
    validate_path(&path)?;
    Ok(Target {
        path,
        query,
        raw: raw.to_string(),
    })
}

/// Decode one form or query component, treating `+` as space.
pub fn decode_component(raw: &str) -> Result<String, HttpError> {
    percent_decode(raw, true, false)
}

/// Extract one query parameter, decoding `+` as space. Repeated parameters
/// follow last-value-wins. Unknown parameters are ignored.
pub fn query_param(query: &Option<String>, name: &str) -> Result<Option<String>, HttpError> {
    let Some(query) = query else {
        return Ok(None);
    };
    let mut found = None;
    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }
        let (key, value) = match pair.split_once('=') {
            Some((key, value)) => (key, value),
            None => (pair, ""),
        };
        let key = decode_component(key)?;
        if key == name {
            found = Some(decode_component(value)?);
        }
    }
    Ok(found)
}

/// Decode header bytes as ISO-8859-1, as required by the specification.
fn latin1(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| *byte as char).collect()
}

fn parse_method(raw: &str) -> Method {
    match raw {
        "GET" => Method::Get,
        "HEAD" => Method::Head,
        "POST" => Method::Post,
        _ => Method::Other,
    }
}

/// Read exactly one request from the connection.
pub fn read_request(stream: &mut TcpStream) -> Result<Request, HttpError> {
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut reader = BufReader::new(&mut *stream);
    let Some(line) = read_line(&mut reader, MAX_REQUEST_LINE, 414, deadline)? else {
        return Err(HttpError::new(400));
    };
    let request_line = String::from_utf8_lossy(&line).to_string();
    let mut parts = request_line.split(' ');
    let (Some(method), Some(target), Some(version), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return Err(HttpError::new(400));
    };
    let raw_method = method.to_string();
    let method = parse_method(&raw_method);
    let version = match version {
        "HTTP/1.1" => Version::Http11,
        "HTTP/1.0" => Version::Http10,
        _ => return Err(HttpError::new(505)),
    };
    let target = parse_target(target)?;

    let mut headers: Vec<(String, String)> = Vec::new();
    let mut header_bytes = 0usize;
    loop {
        let Some(line) = read_line(&mut reader, MAX_HEADER_BLOCK, 431, deadline)? else {
            return Err(HttpError::new(400));
        };
        if line.is_empty() {
            break;
        }
        header_bytes += line.len() + 2;
        if header_bytes > MAX_HEADER_BLOCK {
            return Err(HttpError::new(431));
        }
        if headers.len() >= MAX_HEADERS {
            return Err(HttpError::new(431));
        }
        if line
            .first()
            .is_some_and(|byte| *byte == b' ' || *byte == b'\t')
        {
            return Err(HttpError::new(400));
        }
        let Some(colon) = line.iter().position(|byte| *byte == b':') else {
            return Err(HttpError::new(400));
        };
        let name = latin1(&line[..colon]).trim().to_string();
        let value = latin1(&line[colon + 1..]).trim().to_string();
        if name.is_empty() || name.chars().any(|c| c.is_whitespace() || c.is_control()) {
            return Err(HttpError::new(400));
        }
        headers.push((name, value));
    }

    let transfer_encoding = headers
        .iter()
        .any(|(name, _)| name.eq_ignore_ascii_case("transfer-encoding"));
    let content_lengths: Vec<&str> = headers
        .iter()
        .filter(|(name, _)| name.eq_ignore_ascii_case("content-length"))
        .map(|(_, value)| value.as_str())
        .collect();

    if transfer_encoding {
        if !content_lengths.is_empty() {
            return Err(HttpError::new(400));
        }
        return Err(HttpError::new(501));
    }

    let mut body = Vec::new();
    if method == Method::Post {
        if content_lengths.is_empty() {
            return Err(HttpError::new(411));
        }
        if content_lengths.len() > 1 {
            return Err(HttpError::new(400));
        }
        let length: usize = content_lengths[0]
            .parse()
            .map_err(|_| HttpError::new(400))?;
        if length > MAX_BODY {
            return Err(HttpError::new(413));
        }
        body.resize(length, 0);
        reader
            .read_exact(&mut body)
            .map_err(|error| map_io(&error))?;
        if Instant::now() >= deadline {
            return Err(HttpError::new(408));
        }
    }

    Ok(Request {
        method,
        raw_method,
        target,
        version,
        headers,
        body,
    })
}

/// A response ready to be written.
#[derive(Debug)]
pub struct Response {
    pub status: u16,
    pub content_type: &'static str,
    pub body: Vec<u8>,
    pub extra_headers: Vec<(String, String)>,
}

impl Response {
    pub fn html(status: u16, body: String) -> Self {
        Self {
            status,
            content_type: "text/html; charset=utf-8",
            body: body.into_bytes(),
            extra_headers: Vec::new(),
        }
    }

    pub fn json(status: u16, value: serde_json::Value) -> Self {
        let body = serde_json::to_vec(&value).unwrap_or_else(|_| b"{}".to_vec());
        Self {
            status,
            content_type: "application/json; charset=utf-8",
            body,
            extra_headers: Vec::new(),
        }
    }

    pub fn text(status: u16, content_type: &'static str, body: &str) -> Self {
        Self {
            status,
            content_type,
            body: body.as_bytes().to_vec(),
            extra_headers: Vec::new(),
        }
    }

    pub fn redirect(location: &str) -> Self {
        let mut response = Self::text(303, "text/plain; charset=utf-8", "");
        response
            .extra_headers
            .push(("Location".to_string(), location.to_string()));
        response
    }

    pub fn with_header(mut self, name: &str, value: &str) -> Self {
        self.extra_headers
            .push((name.to_string(), value.to_string()));
        self
    }
}

pub fn reason(status: u16) -> &'static str {
    match status {
        200 => "OK",
        303 => "See Other",
        400 => "Bad Request",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        408 => "Request Timeout",
        409 => "Conflict",
        411 => "Length Required",
        413 => "Content Too Large",
        414 => "URI Too Long",
        415 => "Unsupported Media Type",
        422 => "Unprocessable Content",
        431 => "Request Header Fields Too Large",
        500 => "Internal Server Error",
        501 => "Not Implemented",
        503 => "Service Unavailable",
        505 => "HTTP Version Not Supported",
        _ => "Internal Server Error",
    }
}

/// Write one response and close the connection.
pub fn write_response(
    stream: &mut TcpStream,
    response: &Response,
    head_only: bool,
) -> io::Result<()> {
    let mut head = String::new();
    head.push_str(&format!(
        "HTTP/1.1 {} {}\r\n",
        response.status,
        reason(response.status)
    ));
    head.push_str("Server: papyrus\r\n");
    head.push_str(&format!("Content-Type: {}\r\n", response.content_type));
    head.push_str(&format!("Content-Length: {}\r\n", response.body.len()));
    head.push_str("Connection: close\r\n");
    head.push_str("Cache-Control: no-store\r\n");
    head.push_str("X-Content-Type-Options: nosniff\r\n");
    head.push_str("Referrer-Policy: no-referrer\r\n");
    head.push_str("X-Frame-Options: DENY\r\n");
    if response.content_type.starts_with("text/html") {
        head.push_str(&format!("Content-Security-Policy: {}\r\n", CSP));
    }
    for (name, value) in &response.extra_headers {
        head.push_str(&format!("{}: {}\r\n", name, value));
    }
    head.push_str("\r\n");
    stream.write_all(head.as_bytes())?;
    if !head_only {
        stream.write_all(&response.body)?;
    }
    stream.flush()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_percent_once() {
        assert_eq!(percent_decode("a%20b", false, false).unwrap(), "a b");
        assert_eq!(percent_decode("a+b", true, false).unwrap(), "a b");
        assert_eq!(percent_decode("%2520", false, false).unwrap(), "%20");
    }

    #[test]
    fn rejects_encoded_separators_and_nul() {
        assert!(percent_decode("%2F", false, true).is_err());
        assert!(percent_decode("%5C", false, true).is_err());
        assert!(percent_decode("%00", false, true).is_err());
        assert!(percent_decode("%zz", false, true).is_err());
    }

    #[test]
    fn target_validation() {
        assert!(parse_target("/").is_ok());
        assert!(parse_target("/doc/REQ-001.001").is_ok());
        assert!(parse_target("/doc/REQ-001.001/").is_ok());
        assert!(parse_target("/a/../b").is_err());
        assert!(parse_target("/a//b").is_err());
        assert!(parse_target("http://example.test/").is_err());
        assert!(parse_target("*").is_err());
        assert!(parse_target("/a%2Fb").is_err());
        assert!(parse_target(&format!("/{}", "a".repeat(MAX_TARGET))).is_err());
    }

    #[test]
    fn query_parameters() {
        let query = Some("type=nfr".to_string());
        assert_eq!(
            query_param(&query, "type").unwrap(),
            Some("nfr".to_string())
        );
        let query = Some("type=req&type=nfr".to_string());
        assert_eq!(
            query_param(&query, "type").unwrap(),
            Some("nfr".to_string())
        );
        assert_eq!(query_param(&None, "type").unwrap(), None);
    }
}
