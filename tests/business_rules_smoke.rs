//! Milestone-level smoke test for TASK-46 (m-8: business rules).
//!
//! End-to-end over the built binary: business-rule management, search over
//! business rules, and the offline local browser over the business-rule
//! category. Narrower than the TASK-43/TASK-44/TASK-25 contract suites.

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
    path.push(format!("business-rules-{}-{}", name, std::process::id()));
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

fn dir_arg(dir: &Path) -> String {
    dir.to_str().unwrap().to_string()
}

fn init_project(dir: &Path) {
    run_ok(&["--dir", &dir_arg(dir), "init", "Proj", "--no-git"]);
}

fn create_rule(
    dir: &Path,
    title: &str,
    description: &str,
    rationale: &str,
    applicability: &str,
    requirements: &[&str],
) -> String {
    let d = dir_arg(dir);
    let mut args: Vec<String> = vec![
        "--dir".to_string(),
        d.clone(),
        "rule".to_string(),
        "create".to_string(),
        title.to_string(),
        "--description".to_string(),
        description.to_string(),
        "--rationale".to_string(),
        rationale.to_string(),
        "--applicability".to_string(),
        applicability.to_string(),
    ];
    for req in requirements {
        args.push("--requirement".to_string());
        args.push((*req).to_string());
    }
    let out = bin().args(&args).output().unwrap();
    assert!(
        out.status.success(),
        "rule create failed: {:?}\nstderr: {}",
        args,
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
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
    dir.join("docs/business-rules").join(format!("{}.md", id))
}

fn deprecated_path(dir: &Path, id: &str) -> PathBuf {
    dir.join("docs/business-rules/deprecated")
        .join(format!("{}.md", id))
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
fn milestone_business_rules_flow() {
    let dir = test_root("flow");
    init_project(&dir);

    assert_eq!(
        create_rule(
            &dir,
            "quick brown fox rule",
            "quick brown fox desc",
            "rationale text",
            "applies here",
            &[]
        ),
        "RUL-001"
    );
    assert_eq!(
        create_rule(
            &dir,
            "Beta rule",
            "Beta desc",
            "rationale text",
            "applies here",
            &[]
        ),
        "RUL-002"
    );
    assert!(record_path(&dir, "RUL-001").exists());
    assert!(record_path(&dir, "RUL-002").exists());

    let d = dir_arg(&dir);
    let list = run_ok(&["--dir", &d, "rule", "list"]);
    assert!(list.contains("RUL-001 quick brown fox rule"));
    assert!(list.contains("RUL-002 Beta rule"));

    let view = run_ok(&["--dir", &d, "rule", "view", "RUL-001"]);
    assert!(view.contains("# RUL-001 quick brown fox rule"));
    assert!(view.contains("## Description\nquick brown fox desc"));
    assert!(view.contains("## Rationale\nrationale text"));
    assert!(view.contains("## Applicability\napplies here"));

    assert_eq!(
        run_ok(&[
            "--dir",
            &d,
            "rule",
            "edit",
            "RUL-001",
            "--title",
            "quick brown fox updated",
        ]),
        ""
    );
    let view = run_ok(&["--dir", &d, "rule", "view", "RUL-001"]);
    assert!(view.contains("# RUL-001 quick brown fox updated"));
    assert!(view.contains("## Description\nquick brown fox desc"));

    // Requirements-tag interaction.
    assert_eq!(
        run_ok(&[
            "--dir",
            &d,
            "rule",
            "edit",
            "RUL-001",
            "--requirement",
            "REQ-001",
        ]),
        ""
    );
    let view = run_ok(&["--dir", &d, "rule", "view", "RUL-001"]);
    assert!(view.contains("- REQ-001"));
    assert_eq!(
        run_ok(&[
            "--dir",
            &d,
            "rule",
            "edit",
            "RUL-001",
            "--no-requirement",
            "REQ-001",
        ]),
        ""
    );
    let view = run_ok(&["--dir", &d, "rule", "view", "RUL-001"]);
    assert!(!view.contains("- REQ-001"));
    // remove all
    assert_eq!(
        run_ok(&[
            "--dir",
            &d,
            "rule",
            "create",
            "R2",
            "--description",
            "d",
            "--rationale",
            "r",
            "--applicability",
            "a",
            "--requirement",
            "REQ-010",
        ]),
        "RUL-003"
    );
    assert_eq!(
        run_ok(&["--dir", &d, "rule", "edit", "RUL-003", "--no-requirement"]),
        ""
    );
    let view = run_ok(&["--dir", &d, "rule", "view", "RUL-003"]);
    assert!(!view.contains("## Requirements"));

    // Deprecation hides the record from list and search; restoration keeps the ID.
    assert_eq!(run_ok(&["--dir", &d, "rule", "deprecate", "RUL-001"]), "");
    assert!(!record_path(&dir, "RUL-001").exists());
    assert!(deprecated_path(&dir, "RUL-001").exists());
    assert!(!run_ok(&["--dir", &d, "rule", "list"]).contains("RUL-001"));
    assert!(!search(&dir, &[PHRASE]).contains("RUL-001"));

    assert_eq!(run_ok(&["--dir", &d, "rule", "restore", "RUL-001"]), "");
    assert!(record_path(&dir, "RUL-001").exists());
    let view = run_ok(&["--dir", &d, "rule", "view", "RUL-001"]);
    assert!(view.contains("# RUL-001 quick brown fox updated"));

    // Search: contiguous phrase, --type doc, --limit, --json, deprecated hidden.
    let out = search(&dir, &[PHRASE]);
    assert!(out.contains("RUL-001 doc quick brown fox updated"));
    assert!(!search(&dir, &["quick fox brown"]).contains("RUL-001"));
    assert!(search(&dir, &[PHRASE, "--type", "doc"]).contains("RUL-001 doc"));
    assert_eq!(search(&dir, &[PHRASE, "--limit", "1"]).lines().count(), 1);
    let json: serde_json::Value = serde_json::from_str(&search(&dir, &["--json", PHRASE])).unwrap();
    let hits = json.as_array().unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0]["id"], "RUL-001");
    assert_eq!(hits[0]["type"], "doc");
    assert!(
        hits[0]["title"]
            .as_str()
            .unwrap()
            .contains("quick brown fox updated")
    );
    assert!(hits[0]["path"].as_str().unwrap().contains("RUL-001.md"));
    let empty: serde_json::Value =
        serde_json::from_str(&search(&dir, &["--json", "zzz-no-such-term"])).unwrap();
    assert_eq!(empty.as_array().unwrap().len(), 0);

    // Deprecated hidden until restored (RUL-001 already restored; deprecate RUL-002 not phrase).
    // Browser uses RUL-002 deprecated.
    assert_eq!(run_ok(&["--dir", &d, "rule", "deprecate", "RUL-002"]), "");

    // Field-preserving setup: add unknown section to RUL-001.
    let path = record_path(&dir, "RUL-001");
    let original = fs::read_to_string(&path).unwrap();
    let with_unknown = original.replace(
        "## Requirements",
        "## Custom Section\nKeep this line\n\n## Requirements",
    );
    // If no Requirements section, append.
    let with_unknown = if with_unknown == original {
        format!("{}\n## Custom Section\nKeep this line\n", original)
    } else {
        with_unknown
    };
    fs::write(&path, &with_unknown).unwrap();
    let before = fs::read_to_string(&path).unwrap();

    // Browser.
    let server = Server::start(&dir, None);
    let res = server.get("/?type=business-rule");
    assert_eq!(res.status, 200);
    let list = res.text();
    assert!(list.contains("RUL-001"));
    assert!(!list.contains("RUL-002"));
    assert!(list.contains("aria-current=\"page\""));
    let reqs_at = list.find("href=\"/?type=req\"").unwrap();
    let rules_at = list.find("href=\"/?type=business-rule\"").unwrap();
    assert!(reqs_at < rules_at, "sidebar order");
    assert!(
        list.contains("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">")
    );
    assert!(list.contains("aria-expanded=\"true\""));
    assert!(list.contains("aria-controls=\"type-list\""));
    assert!(res.header("Content-Security-Policy").is_some());
    assert_no_external_refs(&list, "list");

    let res = server.get("/doc/RUL-001");
    assert_eq!(res.status, 200);
    let view = res.text();
    assert!(view.contains("RUL-001"));
    assert!(view.contains("quick brown fox updated"));
    assert!(view.contains("Business Rules"));
    assert!(view.contains("href=\"/doc/RUL-001/edit\""));
    assert_no_external_refs(&view, "view");

    let res = server.get("/doc/RUL-001/edit");
    assert_eq!(res.status, 200);
    let edit = res.text();
    assert!(edit.contains("Hidden field: version (optimistic-concurrency token)"));
    assert!(edit.contains("name=\"requirements\""));
    assert_no_external_refs(&edit, "edit");
    let version = extract_version(&edit);

    let body = form_body(&[
        ("version", version.as_str()),
        ("title", "quick brown fox browser edited"),
        ("description", "New desc"),
        ("rationale", "New rationale"),
        ("applicability", "New applicability"),
        ("requirements", ""),
    ]);
    let res = server.post_form("/doc/RUL-001", &body);
    assert_eq!(res.status, 303);
    assert_eq!(res.header("Location"), Some("/doc/RUL-001"));

    let view = server.get("/doc/RUL-001").text();
    assert!(view.contains("quick brown fox browser edited"));
    let cli = run_ok(&["--dir", &d, "rule", "view", "RUL-001"]);
    assert!(cli.contains("quick brown fox browser edited"));
    assert!(cli.contains("New desc"));

    // Field-preserving: unknown section retained.
    let after = fs::read_to_string(&path).unwrap();
    assert!(after.contains("## Custom Section\nKeep this line"));
    assert!(after.contains("quick brown fox browser edited"));
    for needle in ["## Custom Section", "Keep this line"] {
        if before.contains(needle) {
            assert!(after.contains(needle));
        }
    }

    let res = server.get("/api/business-rules");
    assert_eq!(res.status, 200);
    let api: serde_json::Value = serde_json::from_str(&res.text()).unwrap();
    let ids: Vec<&str> = api
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|item| item["id"].as_str())
        .collect();
    assert!(ids.contains(&"RUL-001"));
    assert!(!ids.contains(&"RUL-002"));

    let res = server.get("/api/business-rule");
    assert_eq!(res.status, 200);

    let res = server.get("/api/doc/RUL-001");
    assert_eq!(res.status, 200);
    let doc: serde_json::Value = serde_json::from_str(&res.text()).unwrap();
    assert_eq!(doc["id"], "RUL-001");
    assert_eq!(doc["kind"], "business-rule");
    assert_eq!(doc["form"], "business-rule");
    assert_eq!(doc["title"], "quick brown fox browser edited");

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

    // Deprecated rule not listed or reachable.
    assert_eq!(server.get("/doc/RUL-002").status, 404);
    assert_eq!(server.get("/doc/RUL-002/edit").status, 404);
    assert_eq!(server.get("/api/doc/RUL-002").status, 404);
    let body = form_body(&[("version", "1.2.3"), ("title", "Nope")]);
    assert_eq!(server.post_form("/doc/RUL-002", &body).status, 404);

    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn installed_layout_has_no_runtime_asset_dependency() {
    let dir = test_root("assets");
    init_project(&dir);
    assert_eq!(
        create_rule(
            &dir,
            "quick brown fox rule",
            "quick brown fox desc",
            "rationale text",
            "applies here",
            &[]
        ),
        "RUL-001"
    );

    let empty_cwd = test_root("emptycwd");
    let out = search_in(&empty_cwd, &dir, &[PHRASE]);
    assert!(out.contains("RUL-001 doc quick brown fox rule"));

    let server = Server::start(&dir, Some(&empty_cwd));
    let res = server.get("/?type=business-rule");
    assert_eq!(res.status, 200);
    assert!(res.text().contains("RUL-001"));
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
