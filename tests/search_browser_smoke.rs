//! Milestone-level smoke test for TASK-7 (m-1: search and browser for
//! requirements).
//!
//! These tests run the built binary end-to-end: search over requirement
//! content, then the offline local browser over the same project. They are
//! deliberately narrower than the TASK-6 contract suite in
//! `tests/browser_cli.rs`; request limits, error-status matrices, XSS,
//! symlink escape, and version conflicts remain authoritative there.

use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::Duration;

const PHRASE: &str = "quick brown fox";

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_papyrus"))
}

/// Scratch root under the project-local `.tmp/` directory (project
/// convention: no system temporary directories).
fn test_root(name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(".tmp");
    path.push(format!("search-browser-{}-{}", name, std::process::id()));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path
}

fn run_ok(args: &[&str]) -> String {
    let out = bin().args(args).output().unwrap();
    assert!(
        out.status.success(),
        "command failed: {:?}\nstderr: {}",
        args,
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn dir_arg(dir: &Path) -> String {
    dir.to_str().unwrap().to_string()
}

fn init_project(dir: &Path) {
    run_ok(&["--dir", &dir_arg(dir), "init", "Proj", "--no-git"]);
}

/// One business-requirement group with a member carrying [`PHRASE`], plus one
/// non-functional group and member.
fn fixture(dir: &Path) {
    init_project(dir);
    assert_eq!(
        run_ok(&[
            "--dir",
            &dir_arg(dir),
            "req",
            "create-group",
            "--title",
            "Payment Group"
        ]),
        "REQ-001"
    );
    assert_eq!(
        run_ok(&[
            "--dir",
            &dir_arg(dir),
            "req",
            "create",
            "Capture payment",
            "--group",
            "REQ-001",
            "--story",
            "As a cashier I capture a payment",
            "-d",
            "The quick brown fox jumps over the lazy dog.",
        ]),
        "REQ-001.001"
    );
    assert_eq!(
        run_ok(&[
            "--dir",
            &dir_arg(dir),
            "nfreq",
            "create-group",
            "--title",
            "Performance"
        ]),
        "NFR-001"
    );
    assert_eq!(
        run_ok(&[
            "--dir",
            &dir_arg(dir),
            "nfreq",
            "create",
            "Latency budget",
            "--group",
            "NFR-001",
            "-d",
            "Response time stays within budget.",
        ]),
        "NFR-001.001"
    );
}

fn search(dir: &Path, args: &[&str]) -> String {
    let dir = dir_arg(dir);
    let mut full = vec!["--dir", dir.as_str(), "search"];
    full.extend_from_slice(args);
    run_ok(&full)
}

fn free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    port
}

struct Server {
    child: Child,
    port: u16,
}

impl Server {
    /// Start `papyrus browser --no-open` on a free loopback port, optionally
    /// from a different working directory.
    fn start(project: &Path, cwd: Option<&Path>) -> Server {
        for _ in 0..10 {
            let port = free_port();
            let mut command = bin();
            command
                .arg("--dir")
                .arg(project)
                .args(["browser", "--no-open", "--port"])
                .arg(port.to_string())
                .stdout(Stdio::piped())
                .stderr(Stdio::null());
            if let Some(cwd) = cwd {
                command.current_dir(cwd);
            }
            let mut child = command.spawn().unwrap();
            let mut reader = BufReader::new(child.stdout.take().unwrap());
            let mut startup = String::new();
            let _ = reader.read_line(&mut startup);
            if startup.trim().is_empty() {
                let _ = child.wait();
                continue;
            }
            assert_eq!(
                startup.trim(),
                format!("Serving http://127.0.0.1:{}/", port)
            );
            return Server { child, port };
        }
        panic!("browser did not start after retries");
    }

    fn get(&self, path: &str) -> HttpResponse {
        self.request("GET", path, None)
    }

    fn post_form(&self, path: &str, body: &str) -> HttpResponse {
        self.request("POST", path, Some(body))
    }

    fn request(&self, method: &str, path: &str, body: Option<&str>) -> HttpResponse {
        let mut raw = format!(
            "{} {} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n",
            method, path, self.port
        )
        .into_bytes();
        if let Some(body) = body {
            raw.extend_from_slice(
                format!(
                    "Content-Type: application/x-www-form-urlencoded\r\nContent-Length: {}\r\n",
                    body.len()
                )
                .as_bytes(),
            );
            raw.extend_from_slice(b"\r\n");
            raw.extend_from_slice(body.as_bytes());
        } else {
            raw.extend_from_slice(b"\r\n");
        }
        let mut stream = TcpStream::connect(("127.0.0.1", self.port)).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(15)))
            .unwrap();
        stream.write_all(&raw).unwrap();
        let mut bytes = Vec::new();
        stream.read_to_end(&mut bytes).unwrap();
        parse_response(&bytes)
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

struct HttpResponse {
    status: u16,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

impl HttpResponse {
    fn text(&self) -> String {
        String::from_utf8_lossy(&self.body).to_string()
    }

    fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }
}

fn parse_response(bytes: &[u8]) -> HttpResponse {
    let split = bytes
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .expect("response header terminator");
    let head = String::from_utf8_lossy(&bytes[..split]).to_string();
    let body = bytes[split + 4..].to_vec();
    let mut lines = head.split("\r\n");
    let status: u16 = lines
        .next()
        .unwrap_or("")
        .split(' ')
        .nth(1)
        .and_then(|value| value.parse().ok())
        .expect("status code");
    let headers = lines
        .filter_map(|line| {
            line.split_once(':')
                .map(|(name, value)| (name.trim().to_string(), value.trim().to_string()))
        })
        .collect();
    HttpResponse {
        status,
        headers,
        body,
    }
}

fn form_body(fields: &[(&str, &str)]) -> String {
    fields
        .iter()
        .map(|(key, value)| format!("{}={}", urlencode(key), urlencode(value)))
        .collect::<Vec<_>>()
        .join("&")
}

fn urlencode(value: &str) -> String {
    let mut out = String::new();
    for byte in value.as_bytes() {
        match *byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*byte as char)
            }
            b' ' => out.push('+'),
            other => out.push_str(&format!("%{:02X}", other)),
        }
    }
    out
}

fn extract_version(html: &str) -> String {
    let marker = "name=\"version\" value=\"";
    let start = html.find(marker).expect("version field") + marker.len();
    let rest = &html[start..];
    let end = rest.find('"').expect("version terminator");
    rest[..end].to_string()
}

fn assert_no_external_reference(label: &str, text: &str) {
    for forbidden in ["http://", "https://", "//cdn", "@import"] {
        assert!(
            !text.contains(forbidden),
            "{label} references an external origin via {forbidden:?}"
        );
    }
}

// T-01 / T-09 / T-10 / T-11 / T-12 / T-15: consolidated milestone flow.
#[test]
fn milestone_search_and_browser_flow() {
    let root = test_root("flow");
    let project = root.join("project");
    fs::create_dir_all(&project).unwrap();
    fixture(&project);

    // Search: contiguous quoted phrase, ascending ID order.
    assert_eq!(
        search(&project, &[PHRASE]),
        "REQ-001.001 doc Capture payment"
    );
    // The same words split apart are not a phrase match.
    assert_eq!(search(&project, &["quick fox"]), "");

    // Type filter: requirement records only; unrelated types are empty.
    let docs = search(&project, &["payment", "--type", "doc"]);
    assert!(docs.contains("REQ-001 doc Payment Group"), "{docs}");
    assert!(docs.contains("REQ-001.001 doc Capture payment"), "{docs}");
    assert_eq!(search(&project, &["payment", "--type", "task"]), "");

    // Limit caps the ordered result set.
    let limited = search(&project, &["payment", "--limit", "1"]);
    assert_eq!(limited.lines().count(), 1, "{limited}");

    // JSON shape and empty-array case.
    let json = search(&project, &["payment", "--json"]);
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    let hits = value.as_array().expect("JSON array");
    assert_eq!(hits.len(), 2, "{json}");
    for hit in hits {
        assert!(hit.get("id").is_some(), "{json}");
        assert_eq!(hit["type"], "doc", "{json}");
        assert!(hit.get("title").is_some(), "{json}");
        assert!(hit.get("path").is_some(), "{json}");
    }
    assert_eq!(search(&project, &["no-such-text-anywhere", "--json"]), "[]");

    // Deprecated records are hidden and reappear after restore.
    run_ok(&[
        "--dir",
        &dir_arg(&project),
        "req",
        "deprecate",
        "REQ-001.001",
    ]);
    assert_eq!(search(&project, &[PHRASE]), "");
    run_ok(&["--dir", &dir_arg(&project), "req", "restore", "REQ-001.001"]);
    assert_eq!(
        search(&project, &[PHRASE]),
        "REQ-001.001 doc Capture payment"
    );

    // Browser over the same content.
    let server = Server::start(&project, None);

    // Both categories render one at a time, selected entry marked current.
    let business = server.get("/");
    assert_eq!(business.status, 200);
    let business_html = business.text();
    assert!(business_html.contains("REQ-001.001"), "{business_html}");
    assert!(business_html.contains("aria-current=\"page\""));
    let non_functional = server.get("/?type=nfr");
    assert_eq!(non_functional.status, 200);
    let nfr_html = non_functional.text();
    assert!(nfr_html.contains("NFR-001.001"), "{nfr_html}");
    assert!(!nfr_html.contains("REQ-001.001"), "{nfr_html}");

    // Record view.
    let view = server.get("/doc/REQ-001.001");
    assert_eq!(view.status, 200);
    let view_html = view.text();
    assert!(view_html.contains("Capture payment"));
    assert!(view_html.contains("quick brown fox"));

    // Edit round-trip through the UI, persisted for the CLI.
    let edit = server.get("/doc/REQ-001.001/edit");
    assert_eq!(edit.status, 200);
    let version = extract_version(&edit.text());
    let response = server.post_form(
        "/doc/REQ-001.001",
        &form_body(&[
            ("version", &version),
            ("title", "Capture payment revised"),
            ("story", "Revised story"),
            ("description", "Revised description"),
        ]),
    );
    assert_eq!(response.status, 303);
    assert_eq!(response.header("Location"), Some("/doc/REQ-001.001"));
    let updated = server.get("/doc/REQ-001.001").text();
    assert!(updated.contains("Capture payment revised"));
    let cli = run_ok(&["--dir", &dir_arg(&project), "req", "view", "REQ-001.001"]);
    assert!(cli.contains("Capture payment revised"), "{cli}");

    // JSON API.
    let index = server.get("/api/requirements?type=req");
    assert_eq!(index.status, 200);
    let index_value: serde_json::Value = serde_json::from_str(&index.text()).unwrap();
    assert_eq!(index_value["groups"][0]["id"], "REQ-001");
    assert_eq!(index_value["requirements"][0]["id"], "REQ-001.001");
    let record = server.get("/api/doc/REQ-001.001");
    assert_eq!(record.status, 200);
    let record_value: serde_json::Value = serde_json::from_str(&record.text()).unwrap();
    assert_eq!(record_value["id"], "REQ-001.001");
    assert_eq!(record_value["form"], "requirement");

    // Embedded assets.
    let css = server.get("/assets/app.css");
    assert_eq!(css.status, 200);
    assert_eq!(css.header("Content-Type"), Some("text/css; charset=utf-8"));
    assert_eq!(css.header("Cache-Control"), Some("no-store"));
    assert!(!css.body.is_empty());
    let js = server.get("/assets/app.js");
    assert_eq!(js.status, 200);
    assert_eq!(
        js.header("Content-Type"),
        Some("text/javascript; charset=utf-8")
    );
    assert!(!js.body.is_empty());

    // 320 px markup hooks and no external references.
    for html in [&business_html, &nfr_html, &view_html] {
        assert!(
            html.contains(
                "<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">"
            )
        );
        assert!(html.contains("sidebar-toggle"));
        assert!(html.contains("aria-expanded="));
        assert!(html.contains("aria-controls=\"type-list\""));
        assert!(html.contains("footer-long"));
        assert!(html.contains("footer-short"));
        assert_no_external_reference("served HTML", html);
    }
    assert!(css.text().contains("@media (max-width: 480px)"));
    assert_no_external_reference("served CSS", &css.text());
    assert_no_external_reference("served JS", &js.text());
    assert!(business.header("Content-Security-Policy").is_some());

    drop(server);
    let _ = fs::remove_dir_all(&root);
}

// T-05 / T-15: the installed layout has no runtime asset dependency.
#[test]
fn installed_layout_has_no_runtime_asset_dependency() {
    let root = test_root("no-assets");
    let project = root.join("project");
    let cwd = root.join("empty-cwd");
    fs::create_dir_all(&project).unwrap();
    fs::create_dir_all(&cwd).unwrap();
    fixture(&project);

    // The project contains no asset files of its own.
    let mut stack = vec![project.clone()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                let name = path.file_name().unwrap().to_string_lossy().to_string();
                assert!(
                    !(name.ends_with(".css") || name.ends_with(".js") || name.ends_with(".html")),
                    "unexpected asset file in project: {}",
                    path.display()
                );
            }
        }
    }

    // Search works from a working directory with no source tree.
    let out = bin()
        .current_dir(&cwd)
        .args(["--dir", &dir_arg(&project), "search", PHRASE])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(
        String::from_utf8_lossy(&out.stdout).trim(),
        "REQ-001.001 doc Capture payment"
    );

    // The browser serves non-empty assets compiled into the binary.
    let server = Server::start(&project, Some(&cwd));
    let page = server.get("/");
    assert_eq!(page.status, 200);
    assert!(page.text().contains("REQ-001.001"));
    let css = server.get("/assets/app.css");
    assert_eq!(css.status, 200);
    assert!(css.text().contains("body"), "CSS body missing");
    let js = server.get("/assets/app.js");
    assert_eq!(js.status, 200);
    assert!(js.text().contains("sidebar"), "JS sidebar hook missing");

    drop(server);
    let _ = fs::remove_dir_all(&root);
}
