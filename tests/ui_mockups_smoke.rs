//! Milestone-level smoke test for TASK-42 (m-7: UI mockups).
//!
//! End-to-end over the built binary: UI mockup management, search over UI
//! mockups, and the offline local browser over the UI mockups category. Narrower
//! than the TASK-38/TASK-39/TASK-40 contract suites.

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

fn test_root(name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(".tmp");
    path.push(format!("ui-mockups-{}-{}", name, std::process::id()));
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

fn run_ok_in(cwd: &Path, args: &[&str]) -> String {
    let out = bin().current_dir(cwd).args(args).output().unwrap();
    assert!(
        out.status.success(),
        "command failed: {:?}\nstderr: {}",
        args,
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn run_status(args: &[&str]) -> i32 {
    bin()
        .args(args)
        .output()
        .unwrap()
        .status
        .code()
        .expect("exit code")
}

fn dir_arg(dir: &Path) -> String {
    dir.to_str().unwrap().to_string()
}

fn init_project(dir: &Path) {
    run_ok(&["--dir", &dir_arg(dir), "init", "Proj", "--no-git"]);
}

fn create_mockup(dir: &Path, title: &str, task: &str, specs: &[&str]) -> String {
    let d = dir_arg(dir);
    let mut args: Vec<&str> = vec![
        "--dir",
        d.as_str(),
        "ui-mockup",
        "create",
        title,
        "--task",
        task,
    ];
    for spec in specs {
        args.push("--spec");
        args.push(*spec);
    }
    run_ok(&args)
}

fn search(dir: &Path, args: &[&str]) -> String {
    let d = dir_arg(dir);
    let mut full: Vec<&str> = vec!["--dir", d.as_str(), "search"];
    full.extend_from_slice(args);
    run_ok(&full)
}

fn search_in(cwd: &Path, dir: &Path, args: &[&str]) -> String {
    let d = dir_arg(dir);
    let mut full: Vec<&str> = vec!["--dir", d.as_str(), "search"];
    full.extend_from_slice(args);
    run_ok_in(cwd, &full)
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
        .filter_map(|line| line.split_once(':'))
        .map(|(name, value)| (name.to_string(), value.trim().to_string()))
        .collect();
    HttpResponse {
        status,
        headers,
        body,
    }
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

fn form_body(fields: &[(&str, &str)]) -> String {
    fields
        .iter()
        .map(|(key, value)| format!("{}={}", urlencode(key), urlencode(value)))
        .collect::<Vec<_>>()
        .join("&")
}

fn extract_version(html: &str) -> String {
    let marker = "name=\"version\" value=\"";
    let start = html.find(marker).expect("version field") + marker.len();
    let rest = &html[start..];
    let end = rest.find('"').expect("version terminator");
    rest[..end].to_string()
}

fn record_path(dir: &Path, id: &str) -> PathBuf {
    dir.join("docs/specifications/ui/mockups")
        .join(format!("{}.html", id))
}

fn deprecated_path(dir: &Path, id: &str) -> PathBuf {
    dir.join("docs/specifications/ui/mockups/deprecated")
        .join(format!("{}.html", id))
}

fn assert_no_external_refs(body: &str, page: &str) {
    for needle in ["http://", "https://", "@import", "url(", "<img", "//cdn."] {
        assert!(
            !body.to_lowercase().contains(needle),
            "{page} references an external origin via {needle}"
        );
    }
}

#[test]
fn milestone_ui_mockups_flow() {
    let dir = test_root("flow");
    init_project(&dir);

    assert_eq!(
        create_mockup(&dir, "quick brown fox mockup", "TASK-1", &["UI-001"]),
        "MOCK-001"
    );
    assert_eq!(
        create_mockup(&dir, "Beta mockup", "TASK-2", &["UI-002"]),
        "MOCK-002"
    );
    assert!(record_path(&dir, "MOCK-001").exists());
    assert!(record_path(&dir, "MOCK-002").exists());

    let d = dir_arg(&dir);
    let list = run_ok(&["--dir", &d, "ui-mockup", "list"]);
    assert!(list.contains("MOCK-001 quick brown fox mockup"));
    assert!(list.contains("MOCK-002 Beta mockup"));

    let view = run_ok(&["--dir", &d, "ui-mockup", "view", "MOCK-001"]);
    assert!(view.contains("<!-- mockup-id: MOCK-001 -->"));
    assert!(view.contains("<!-- status: pending -->"));
    assert!(view.contains("quick brown fox mockup"));

    assert_eq!(
        run_ok(&[
            "--dir",
            &d,
            "ui-mockup",
            "edit",
            "MOCK-001",
            "--title",
            "quick brown fox updated",
        ]),
        ""
    );
    let view = run_ok(&["--dir", &d, "ui-mockup", "view", "MOCK-001"]);
    assert!(view.contains("quick brown fox updated"));

    // Deprecation hides the record from list and search; restoration keeps the ID.
    assert_eq!(
        run_ok(&["--dir", &d, "ui-mockup", "deprecate", "MOCK-001"]),
        ""
    );
    assert!(!record_path(&dir, "MOCK-001").exists());
    assert!(deprecated_path(&dir, "MOCK-001").exists());
    assert!(!run_ok(&["--dir", &d, "ui-mockup", "list"]).contains("MOCK-001"));
    assert!(!search(&dir, &[PHRASE]).contains("MOCK-001"));

    assert_eq!(
        run_ok(&["--dir", &d, "ui-mockup", "restore", "MOCK-001"]),
        ""
    );
    assert!(record_path(&dir, "MOCK-001").exists());
    let view = run_ok(&["--dir", &d, "ui-mockup", "view", "MOCK-001"]);
    assert!(view.contains("<!-- mockup-id: MOCK-001 -->"));
    assert!(view.contains("<!-- status: pending -->"));

    // Search: contiguous phrase, --type doc, --limit, --json, deprecated hidden.
    let out = search(&dir, &[PHRASE]);
    assert!(out.contains("MOCK-001 doc quick brown fox updated"));
    assert!(!search(&dir, &["quick fox brown"]).contains("MOCK-001"));
    assert!(search(&dir, &[PHRASE, "--type", "doc"]).contains("MOCK-001 doc"));
    assert_eq!(search(&dir, &[PHRASE, "--limit", "1"]).lines().count(), 1);
    let json: serde_json::Value = serde_json::from_str(&search(&dir, &["--json", PHRASE])).unwrap();
    let hits = json.as_array().unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0]["id"], "MOCK-001");
    assert_eq!(hits[0]["type"], "doc");
    assert!(
        hits[0]["title"]
            .as_str()
            .unwrap()
            .contains("quick brown fox updated")
    );
    assert!(hits[0]["path"].as_str().unwrap().contains("MOCK-001.html"));
    let empty: serde_json::Value =
        serde_json::from_str(&search(&dir, &["--json", "zzz-no-such-term"])).unwrap();
    assert_eq!(empty.as_array().unwrap().len(), 0);

    // Supersession moves the predecessor to deprecated/ with a back-reference.
    assert_eq!(
        run_ok(&[
            "--dir",
            &d,
            "ui-mockup",
            "edit",
            "MOCK-002",
            "--superseded-by",
            "MOCK-001",
        ]),
        ""
    );
    assert!(!record_path(&dir, "MOCK-002").exists());
    assert!(deprecated_path(&dir, "MOCK-002").exists());
    let view = run_ok(&["--dir", &d, "ui-mockup", "view", "MOCK-002"]);
    assert!(view.contains("Superseded by: MOCK-001"));
    assert!(!run_ok(&["--dir", &d, "ui-mockup", "list"]).contains("MOCK-002"));
    assert_eq!(
        run_status(&[
            "--dir",
            &d,
            "ui-mockup",
            "edit",
            "MOCK-001",
            "--superseded-by",
            "MOCK-001",
        ]),
        4
    );
    assert_eq!(
        run_status(&[
            "--dir",
            &d,
            "ui-mockup",
            "edit",
            "MOCK-001",
            "--superseded-by",
            "MOCK-999",
        ]),
        3
    );
    assert_eq!(
        run_status(&["--dir", &d, "ui-mockup", "deprecate", "MOCK-002"]),
        4
    );
    assert_eq!(
        run_status(&["--dir", &d, "ui-mockup", "restore", "MOCK-002"]),
        4
    );

    // Browser: category, view, edit round-trip, API, assets, 320 px, offline content.
    let server = Server::start(&dir, None);
    let res = server.get("/?type=ui-mockup");
    assert_eq!(res.status, 200);
    let list = res.text();
    assert!(list.contains("MOCK-001"));
    assert!(!list.contains("MOCK-002"));
    assert!(list.contains("aria-current=\"page\""));
    let specs_link = list.find("href=\"/?type=ui-spec\"").unwrap();
    let mockups_link = list.find("href=\"/?type=ui-mockup\"").unwrap();
    assert!(specs_link < mockups_link);
    assert!(
        list.contains("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">")
    );
    assert!(list.contains("aria-expanded=\"true\""));
    assert!(list.contains("aria-controls=\"type-list\""));
    assert!(list.contains("footer-long"));
    assert!(list.contains("footer-short"));
    assert!(res.header("Content-Security-Policy").is_some());
    assert_no_external_refs(&list, "list");

    let res = server.get("/doc/MOCK-001");
    assert_eq!(res.status, 200);
    let view = res.text();
    assert!(view.contains("MOCK-001"));
    assert!(view.contains("quick brown fox updated"));
    assert!(view.contains("UI specifications"));
    assert!(view.contains("References"));
    assert!(view.contains("href=\"/doc/MOCK-001/edit\""));
    assert_no_external_refs(&view, "view");

    let res = server.get("/doc/MOCK-001/edit");
    assert_eq!(res.status, 200);
    let edit = res.text();
    assert!(edit.contains("Hidden field: version (optimistic-concurrency token)"));
    assert!(edit.contains("name=\"specs\""));
    assert!(edit.contains("name=\"refs\""));
    assert_no_external_refs(&edit, "edit");
    let version = extract_version(&edit);

    let body = form_body(&[
        ("version", version.as_str()),
        ("title", "quick brown fox browser edited"),
        ("specs", "UI-001"),
        ("specs", "UI-002"),
        ("refs", "docs/specifications/ui-mockups-management.md"),
    ]);
    let res = server.post_form("/doc/MOCK-001", &body);
    assert_eq!(res.status, 303);
    assert_eq!(res.header("Location"), Some("/doc/MOCK-001"));

    let view = server.get("/doc/MOCK-001").text();
    assert!(view.contains("quick brown fox browser edited"));
    assert!(view.contains("UI-002"));
    assert!(view.contains("docs/specifications/ui-mockups-management.md"));
    let cli = run_ok(&["--dir", &d, "ui-mockup", "view", "MOCK-001"]);
    assert!(cli.contains("quick brown fox browser edited"));
    assert!(cli.contains("UI-002"));

    let res = server.get("/api/ui-mockups");
    assert_eq!(res.status, 200);
    let api: serde_json::Value = serde_json::from_str(&res.text()).unwrap();
    let ids: Vec<&str> = api
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|item| item["id"].as_str())
        .collect();
    assert!(ids.contains(&"MOCK-001"));
    assert!(!ids.contains(&"MOCK-002"));

    let res = server.get("/api/doc/MOCK-001");
    assert_eq!(res.status, 200);
    let doc: serde_json::Value = serde_json::from_str(&res.text()).unwrap();
    assert_eq!(doc["id"], "MOCK-001");
    assert_eq!(doc["kind"], "ui-mockup");
    assert_eq!(doc["form"], "ui-mockup");
    assert!(
        doc["specs"]
            .as_array()
            .unwrap()
            .iter()
            .any(|value| value == "UI-002")
    );
    assert!(
        doc["refs"]
            .as_array()
            .unwrap()
            .iter()
            .any(|value| value == "docs/specifications/ui-mockups-management.md")
    );

    let res = server.get("/assets/app.css");
    assert_eq!(res.status, 200);
    assert!(res.header("Content-Type").unwrap().starts_with("text/css"));
    assert!(!res.body.is_empty());
    let css = res.text();
    assert!(css.contains("@media (max-width: 480px)"));
    assert!(css.contains("320"));
    assert_no_external_refs(&css, "stylesheet");

    let res = server.get("/assets/app.js");
    assert_eq!(res.status, 200);
    assert!(
        res.header("Content-Type")
            .unwrap()
            .starts_with("text/javascript")
    );
    assert!(!res.body.is_empty());
    assert_no_external_refs(&res.text(), "script");

    // Deprecated mockups are not listed or reachable.
    assert_eq!(server.get("/doc/MOCK-002").status, 404);
    assert_eq!(server.get("/doc/MOCK-002/edit").status, 404);
    assert_eq!(server.get("/api/doc/MOCK-002").status, 404);
    let body = form_body(&[("version", "1.2.3"), ("title", "Nope"), ("specs", "UI-001")]);
    assert_eq!(server.post_form("/doc/MOCK-002", &body).status, 404);

    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn installed_layout_has_no_runtime_asset_dependency() {
    let dir = test_root("assets");
    init_project(&dir);
    assert_eq!(
        create_mockup(&dir, "quick brown fox mockup", "TASK-1", &["UI-001"]),
        "MOCK-001"
    );

    let empty_cwd = test_root("emptycwd");
    let out = search_in(&empty_cwd, &dir, &[PHRASE]);
    assert!(out.contains("MOCK-001 doc quick brown fox mockup"));

    let server = Server::start(&dir, Some(&empty_cwd));
    let res = server.get("/?type=ui-mockup");
    assert_eq!(res.status, 200);
    assert!(res.text().contains("MOCK-001"));
    let res = server.get("/assets/app.css");
    assert_eq!(res.status, 200);
    assert!(!res.body.is_empty());
    let res = server.get("/assets/app.js");
    assert_eq!(res.status, 200);
    assert!(!res.body.is_empty());

    drop(server);
    let _ = fs::remove_dir_all(&dir);
    let _ = fs::remove_dir_all(&empty_cwd);
}
