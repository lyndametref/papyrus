//! Milestone-level smoke test for TASK-59 (m-11: technical specifications).
//!
//! End-to-end over the built binary: technical-specification management, search
//! over technical specifications, and the offline local browser over the
//! Technical Specifications category. Narrower than the TASK-55/TASK-56/TASK-57
//! contract suites.

use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::time::Duration;

const PHRASE: &str = "quick brown fox";
const SECOND_PHRASE: &str = "hidden zebra fact";
const NOT_FOUND_HELP: &str = "Deprecated technical specifications are not listed and cannot be opened. The same page is used for an unknown address.";

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_papyrus"))
}

fn test_root(name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(".tmp");
    path.push(format!("tech-spec-smoke-{}-{}", name, std::process::id()));
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

fn run_status(dir: &Path, args: &[&str]) -> i32 {
    let d = dir_arg(dir);
    let mut full: Vec<&str> = vec!["--dir", d.as_str()];
    full.extend_from_slice(args);
    bin()
        .args(&full)
        .output()
        .unwrap()
        .status
        .code()
        .expect("process terminated by signal")
}

fn tech_spec_output(dir: &Path, args: &[&str]) -> Output {
    let d = dir_arg(dir);
    let mut full: Vec<&str> = vec!["--dir", d.as_str()];
    full.extend_from_slice(args);
    bin().args(&full).output().unwrap()
}

fn dir_arg(dir: &Path) -> String {
    dir.to_str().unwrap().to_string()
}

fn init_project(dir: &Path) {
    run_ok(&["--dir", &dir_arg(dir), "init", "Proj", "--no-git"]);
}

fn create_spec(dir: &Path, title: &str, description: &str) -> String {
    let out = tech_spec_output(
        dir,
        &["tech-spec", "create", title, "--description", description],
    );
    assert!(
        out.status.success(),
        "tech-spec create failed: {:?}\nstderr: {}",
        vec![title, description],
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
        self.request("GET", path, None, None)
    }

    fn post_form(&self, path: &str, body: &str) -> HttpResponse {
        self.request(
            "POST",
            path,
            Some("application/x-www-form-urlencoded"),
            Some(body),
        )
    }

    fn post_json(&self, path: &str, body: &str) -> HttpResponse {
        self.request("POST", path, Some("application/json"), Some(body))
    }

    fn request(
        &self,
        method: &str,
        path: &str,
        content_type: Option<&str>,
        body: Option<&str>,
    ) -> HttpResponse {
        let mut raw = format!(
            "{} {} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n",
            method, path, self.port
        )
        .into_bytes();
        if let Some(body) = body {
            raw.extend_from_slice(
                format!(
                    "Content-Type: {}\r\nContent-Length: {}\r\n",
                    content_type.unwrap_or("application/octet-stream"),
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
    dir.join("docs/technical").join(format!("{}.md", id))
}

fn deprecated_path(dir: &Path, id: &str) -> PathBuf {
    dir.join("docs/technical/deprecated")
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

fn assert_markup_hooks(page: &str, name: &str) {
    assert!(
        page.contains("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">"),
        "{name} lacks the viewport meta"
    );
    assert!(
        page.contains("aria-expanded="),
        "{name} lacks aria-expanded"
    );
    assert!(
        page.contains("aria-controls=\"type-list\""),
        "{name} lacks the sidebar toggle target"
    );
}

#[test]
fn milestone_technical_specifications_flow() {
    let dir = test_root("flow");
    init_project(&dir);

    // Lifecycle: create/list/view/edit/deprecate/restore with ID preservation.
    assert_eq!(
        create_spec(&dir, "quick brown fox architecture", "quick brown fox desc"),
        "TECH-001"
    );
    assert_eq!(
        create_spec(&dir, "Hidden zebra note", SECOND_PHRASE),
        "TECH-002"
    );
    assert!(record_path(&dir, "TECH-001").exists());
    assert!(record_path(&dir, "TECH-002").exists());
    let content = fs::read_to_string(record_path(&dir, "TECH-001")).unwrap();
    assert_eq!(
        content,
        "# TECH-001 quick brown fox architecture\n\n## Description\nquick brown fox desc\n"
    );

    let d = dir_arg(&dir);
    let list = run_ok(&["--dir", &d, "tech-spec", "list"]);
    assert!(list.contains("TECH-001 quick brown fox architecture"));
    assert!(list.contains("TECH-002 Hidden zebra note"));
    assert!(list.find("TECH-001").unwrap() < list.find("TECH-002").unwrap());

    let view = run_ok(&["--dir", &d, "tech-spec", "view", "TECH-001"]);
    assert!(view.contains("# TECH-001 quick brown fox architecture"));
    assert!(view.contains("## Description\nquick brown fox desc"));

    assert_eq!(
        run_ok(&[
            "--dir",
            &d,
            "tech-spec",
            "edit",
            "TECH-001",
            "--title",
            "quick brown fox updated",
        ]),
        ""
    );
    let view = run_ok(&["--dir", &d, "tech-spec", "view", "TECH-001"]);
    assert!(view.contains("# TECH-001 quick brown fox updated"));
    assert!(view.contains("## Description\nquick brown fox desc"));

    assert_eq!(
        run_ok(&[
            "--dir",
            &d,
            "tech-spec",
            "edit",
            "TECH-001",
            "--description",
            "quick brown fox desc v2",
        ]),
        ""
    );
    let view = run_ok(&["--dir", &d, "tech-spec", "view", "TECH-001"]);
    assert!(view.contains("# TECH-001 quick brown fox updated"));
    assert!(view.contains("## Description\nquick brown fox desc v2"));

    // Deprecation hides the specification but keeps it viewable by ID.
    assert_eq!(
        run_ok(&["--dir", &d, "tech-spec", "deprecate", "TECH-001"]),
        ""
    );
    assert!(!record_path(&dir, "TECH-001").exists());
    assert!(deprecated_path(&dir, "TECH-001").exists());
    assert!(!run_ok(&["--dir", &d, "tech-spec", "list"]).contains("TECH-001"));
    assert!(
        run_ok(&["--dir", &d, "tech-spec", "view", "TECH-001"]).contains("quick brown fox desc v2")
    );
    assert_eq!(
        run_status(&dir, &["tech-spec", "deprecate", "TECH-001"]),
        4,
        "already deprecated"
    );

    let deprecated_content = fs::read_to_string(deprecated_path(&dir, "TECH-001")).unwrap();
    assert_eq!(
        run_ok(&["--dir", &d, "tech-spec", "restore", "TECH-001"]),
        ""
    );
    assert!(record_path(&dir, "TECH-001").exists());
    assert!(!deprecated_path(&dir, "TECH-001").exists());
    assert_eq!(
        fs::read_to_string(record_path(&dir, "TECH-001")).unwrap(),
        deprecated_content
    );
    assert!(run_ok(&["--dir", &d, "tech-spec", "list"]).contains("TECH-001"));
    assert_eq!(
        run_status(&dir, &["tech-spec", "restore", "TECH-001"]),
        4,
        "already active"
    );

    // Exit-code contract exercised end to end.
    for args in [
        vec!["tech-spec", "view", "TECH-404"],
        vec!["tech-spec", "edit", "TECH-404", "--title", "t"],
        vec!["tech-spec", "deprecate", "TECH-404"],
        vec!["tech-spec", "restore", "TECH-404"],
    ] {
        assert_eq!(run_status(&dir, &args), 3, "args={args:?}");
    }
    for args in [
        vec!["tech-spec", "create", "T"],
        vec!["tech-spec", "create", "--description", "d"],
        vec!["tech-spec", "create", "T", "--description"],
    ] {
        assert_eq!(run_status(&dir, &args), 2, "args={args:?}");
    }
    for args in [
        vec!["tech-spec", "create", "", "--description", "d"],
        vec!["tech-spec", "create", "a/b", "--description", "d"],
    ] {
        assert_eq!(run_status(&dir, &args), 4, "args={args:?}");
    }

    // Search over technical specifications.
    let out = search(&dir, &[PHRASE]);
    assert!(out.contains("TECH-001 doc quick brown fox updated"));
    assert!(!search(&dir, &["quick fox brown"]).contains("TECH-001"));
    assert!(search(&dir, &[PHRASE, "--type", "doc"]).contains("TECH-001 doc"));
    assert!(!search(&dir, &[PHRASE, "--type", "task"]).contains("TECH-001"));
    for ty in ["milestone", "release", "decision"] {
        assert!(
            !search(&dir, &[PHRASE, "--type", ty]).contains("TECH-001"),
            "type={ty}"
        );
    }
    assert_eq!(search(&dir, &[PHRASE, "--limit", "1"]).lines().count(), 1);
    assert_eq!(
        run_status(&dir, &["search", PHRASE, "--type", "tech-spec"]),
        2,
        "undocumented --type value"
    );

    let json: serde_json::Value = serde_json::from_str(&search(&dir, &["--json", PHRASE])).unwrap();
    let hits = json.as_array().unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0]["id"], "TECH-001");
    assert_eq!(hits[0]["type"], "doc");
    assert!(
        hits[0]["title"]
            .as_str()
            .unwrap()
            .contains("quick brown fox updated")
    );
    assert!(hits[0]["path"].as_str().unwrap().ends_with("TECH-001.md"));
    let empty: serde_json::Value =
        serde_json::from_str(&search(&dir, &["--json", "zzz-no-such-term"])).unwrap();
    assert_eq!(empty.as_array().unwrap().len(), 0);

    // Deprecated technical specifications are hidden from search until restored.
    assert!(search(&dir, &[SECOND_PHRASE]).contains("TECH-002 doc Hidden zebra note"));
    assert_eq!(
        run_ok(&["--dir", &d, "tech-spec", "deprecate", "TECH-002"]),
        ""
    );
    assert!(!search(&dir, &[SECOND_PHRASE]).contains("TECH-002"));
    assert_eq!(
        run_ok(&["--dir", &d, "tech-spec", "restore", "TECH-002"]),
        ""
    );
    assert!(search(&dir, &[SECOND_PHRASE]).contains("TECH-002 doc Hidden zebra note"));

    // Field-preserving setup: a preamble and unknown sections must survive an
    // edit that only changes the heading title and the ## Description body.
    let original = "# TECH-001 quick brown fox updated\n\nPreamble kept.\n\n## Description\nquick brown fox desc v2\n\n## Editor Notes\nKeep this line\n\n## Unknown\nAnd this\n";
    fs::write(record_path(&dir, "TECH-001"), original).unwrap();

    // Browser over the Technical Specifications category. TECH-002 is active
    // after the restore above, so it must reappear before being deprecated for
    // the hidden/404 checks.
    let server = Server::start(&dir, None);
    let restored = server.get("/?type=tech-spec").text();
    assert!(restored.contains("2 technical specifications"));
    assert!(restored.contains("TECH-002"));
    assert_eq!(
        run_ok(&["--dir", &d, "tech-spec", "deprecate", "TECH-002"]),
        ""
    );

    let res = server.get("/?type=tech-spec");
    assert_eq!(res.status, 200);
    let list = res.text();
    assert!(list.contains("<h2 class=\"h2\">Technical Specifications</h2>"));
    assert!(list.contains("1 technical specification"));
    assert!(list.contains("TECH-001"));
    assert!(list.contains("quick brown fox updated"));
    assert!(list.contains("quick brown fox desc v2"));
    assert!(list.contains("href=\"/doc/TECH-001\""));
    assert!(!list.contains("TECH-002"));
    assert!(list.contains("href=\"/?type=tech-spec\" aria-current=\"page\""));
    let audit_at = list.find("href=\"/?type=audit-record\"").unwrap();
    let tech_at = list.find("href=\"/?type=tech-spec\"").unwrap();
    let nfr_at = list.find("href=\"/?type=nfr\"").unwrap();
    assert!(
        audit_at < tech_at,
        "Technical Specifications after Audit Records"
    );
    assert!(tech_at < nfr_at);
    assert_markup_hooks(&list, "list");
    assert!(res.header("Content-Security-Policy").is_some());
    assert_no_external_refs(&list, "list");

    let res = server.get("/doc/TECH-001");
    assert_eq!(res.status, 200);
    let view = res.text();
    assert!(view.contains("Record ID"));
    assert!(view.contains("TECH-001"));
    assert!(view.contains("Technical specification"));
    assert!(view.contains("Technical Specifications"));
    assert!(view.contains("<h1 class=\"h1\">quick brown fox updated</h1>"));
    assert!(view.contains("<p class=\"text\">quick brown fox desc v2</p>"));
    assert!(!view.contains("Description</h"));
    assert!(view.contains(
        "href=\"/?type=tech-spec\">Technical Specifications</a></li><li><span aria-current=\"page\">TECH-001</span>"
    ));
    assert_eq!(view.matches("href=\"/doc/TECH-001/edit\"").count(), 1);
    assert!(view.contains(">Edit</a>"));
    assert_markup_hooks(&view, "view");
    assert_no_external_refs(&view, "view");

    // An absent/empty description is viewable as (none).
    let empty_record = record_path(&dir, "TECH-003");
    fs::write(&empty_record, "# TECH-003 Empty description\n").unwrap();
    let empty_view = server.get("/doc/TECH-003");
    assert_eq!(empty_view.status, 200);
    assert!(empty_view.text().contains("(none)"));
    fs::remove_file(&empty_record).unwrap();

    let res = server.get("/doc/TECH-001/edit");
    assert_eq!(res.status, 200);
    let edit = res.text();
    assert!(edit.contains("Hidden field: version (optimistic-concurrency token)"));
    assert!(edit.contains("name=\"title\""));
    assert!(edit.contains("name=\"description\""));
    assert!(edit.contains("Save changes"));
    assert!(edit.contains("Cancel"));
    assert_markup_hooks(&edit, "edit");
    assert_no_external_refs(&edit, "edit");
    let version = extract_version(&edit);

    let body = form_body(&[
        ("version", version.as_str()),
        ("title", "quick brown fox browser edited"),
        ("description", "New body\nsecond line"),
    ]);
    let res = server.post_form("/doc/TECH-001", &body);
    assert_eq!(res.status, 303);
    assert_eq!(res.header("Location"), Some("/doc/TECH-001"));

    let view = server.get("/doc/TECH-001").text();
    assert!(view.contains("<h1 class=\"h1\">quick brown fox browser edited</h1>"));
    assert!(view.contains("New body\nsecond line"));
    let cli = run_ok(&["--dir", &d, "tech-spec", "view", "TECH-001"]);
    assert!(cli.contains("quick brown fox browser edited"));
    assert!(cli.contains("New body"));
    assert!(cli.contains("second line"));

    // Field-preserving write: only the heading title and ## Description changed.
    let expected = "# TECH-001 quick brown fox browser edited\n\nPreamble kept.\n\n## Description\nNew body\nsecond line\n\n## Editor Notes\nKeep this line\n\n## Unknown\nAnd this\n";
    assert_eq!(
        fs::read_to_string(record_path(&dir, "TECH-001")).unwrap(),
        expected
    );

    // JSON API.
    for alias in ["/api/tech-specs", "/api/tech-spec"] {
        let res = server.get(alias);
        assert_eq!(res.status, 200, "alias {alias}");
        assert!(
            res.header("Content-Type")
                .unwrap()
                .starts_with("application/json")
        );
        let api: serde_json::Value = serde_json::from_str(&res.text()).unwrap();
        let arr = api.as_array().unwrap();
        assert_eq!(arr.len(), 1, "alias {alias}");
        assert_eq!(arr[0]["id"], "TECH-001");
        assert_eq!(arr[0]["title"], "quick brown fox browser edited");
    }

    let res = server.get("/api/doc/TECH-001");
    assert_eq!(res.status, 200);
    let doc: serde_json::Value = serde_json::from_str(&res.text()).unwrap();
    assert_eq!(doc["id"], "TECH-001");
    assert_eq!(doc["kind"], "tech-spec");
    assert_eq!(doc["form"], "tech-spec");
    assert_eq!(doc["title"], "quick brown fox browser edited");
    assert_eq!(doc["description"], "New body\nsecond line");
    assert!(doc["version"].is_string());
    let path = doc["path"].as_str().unwrap();
    assert!(path.ends_with("TECH-001.md"));
    assert!(!path.starts_with('/'));

    // Embedded assets served from memory.
    let res = server.get("/assets/app.css");
    assert_eq!(res.status, 200);
    assert!(res.header("Content-Type").unwrap().starts_with("text/css"));
    assert_eq!(res.header("Cache-Control"), Some("no-store"));
    assert!(!res.body.is_empty());
    let css = res.text();
    assert!(css.contains("white-space: pre-wrap"));
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
    assert_eq!(res.header("Cache-Control"), Some("no-store"));
    assert!(!res.body.is_empty());
    assert_no_external_refs(&res.text(), "script");

    // Deprecated technical specifications are neither listed nor reachable.
    assert_eq!(server.get("/doc/TECH-002").status, 404);
    let not_found = server.get("/doc/TECH-002").text();
    assert!(not_found.contains("Record not found"));
    assert!(not_found.contains(NOT_FOUND_HELP));
    assert!(not_found.contains("href=\"/?type=tech-spec\""));
    assert_eq!(
        not_found
            .matches(">Back to technical specifications</a>")
            .count(),
        1
    );
    let unknown = server.get("/doc/TECH-404");
    assert_eq!(unknown.status, 404);
    assert_eq!(not_found, unknown.text());
    let deprecated_edit = server.get("/doc/TECH-002/edit");
    assert_eq!(deprecated_edit.status, 404);
    assert_eq!(not_found, deprecated_edit.text());
    let body = form_body(&[("version", "1.2.3"), ("title", "Ok"), ("description", "ok")]);
    let deprecated_post = server.post_form("/doc/TECH-002", &body);
    assert_eq!(deprecated_post.status, 404);
    assert_eq!(not_found, deprecated_post.text());

    let deprecated_json = server.get("/api/doc/TECH-002");
    assert_eq!(deprecated_json.status, 404);
    assert!(
        deprecated_json
            .header("Content-Type")
            .unwrap()
            .starts_with("application/json")
    );
    let json: serde_json::Value = serde_json::from_str(&deprecated_json.text()).unwrap();
    assert_eq!(json["error"]["code"], 404);
    let deprecated_json_post = server.post_json(
        "/api/doc/TECH-002",
        "{\"version\":\"1.2.3\",\"title\":\"Ok\"}",
    );
    assert_eq!(deprecated_json_post.status, 404);
    let json: serde_json::Value = serde_json::from_str(&deprecated_json_post.text()).unwrap();
    assert_eq!(json["error"]["code"], 404);

    drop(server);
    let _ = fs::remove_dir_all(&dir);

    // A missing technical-specification directory yields an empty result.
    let missing = test_root("missing-dir");
    init_project(&missing);
    fs::remove_dir_all(missing.join("docs/technical")).unwrap();
    assert_eq!(search(&missing, &[PHRASE]), "");
    assert!(!missing.join("docs/technical").exists());
    let _ = fs::remove_dir_all(&missing);

    // A disabled document type rejects create and list.
    let disabled = test_root("disabled");
    init_project(&disabled);
    run_ok(&[
        "--dir",
        &dir_arg(&disabled),
        "config",
        "set",
        "document_types.tech-spec.enabled",
        "false",
    ]);
    assert_eq!(
        run_status(
            &disabled,
            &["tech-spec", "create", "T", "--description", "d"]
        ),
        4
    );
    assert_eq!(run_status(&disabled, &["tech-spec", "list"]), 4);
    assert!(!record_path(&disabled, "TECH-001").exists());
    let _ = fs::remove_dir_all(&disabled);
}

#[test]
fn installed_layout_has_no_runtime_asset_dependency() {
    let dir = test_root("assets");
    init_project(&dir);
    assert_eq!(
        create_spec(&dir, "quick brown fox architecture", "quick brown fox desc"),
        "TECH-001"
    );

    let empty_cwd = test_root("emptycwd");
    let out = search_in(&empty_cwd, &dir, &[PHRASE]);
    assert!(out.contains("TECH-001 doc quick brown fox architecture"));

    let server = Server::start(&dir, Some(&empty_cwd));
    let res = server.get("/?type=tech-spec");
    assert_eq!(res.status, 200);
    assert!(res.text().contains("TECH-001"));
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
