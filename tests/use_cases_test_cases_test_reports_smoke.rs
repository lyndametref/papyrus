//! Milestone-level smoke test for TASK-82 (m-15: use cases and test documentation).
//!
//! End-to-end over the built binary: use-case, test-case, and test-report
//! management, search over all three types, and the offline local browser over
//! the three categories. Narrower than the TASK-78/TASK-80/TASK-81 contract
//! suites.

use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::time::Duration;

const PHRASE: &str = "quick brown fox";
const SECOND_PHRASE: &str = "hidden zebra fact";
const NOT_FOUND_UC: &str = "Deprecated use cases are not listed and cannot be opened. The same page is used for an unknown address.";
const NOT_FOUND_TC: &str = "Deprecated test cases are not listed and cannot be opened. The same page is used for an unknown address.";
const NOT_FOUND_TR: &str = "Deprecated test reports are not listed and cannot be opened. The same page is used for an unknown address.";

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_papyrus"))
}

fn test_root(name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(".tmp");
    path.push(format!("uc-tc-tr-smoke-{}-{}", name, std::process::id()));
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

fn type_output(dir: &Path, args: &[&str]) -> Output {
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

fn create_uc(dir: &Path, title: &str, description: &str) -> String {
    let out = type_output(
        dir,
        &["use-case", "create", title, "--description", description],
    );
    assert!(
        out.status.success(),
        "use-case create failed: {}\nstderr: {}",
        title,
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn create_tc(dir: &Path, title: &str, description: &str) -> String {
    let out = type_output(
        dir,
        &["test-case", "create", title, "--description", description],
    );
    assert!(
        out.status.success(),
        "test-case create failed: {}\nstderr: {}",
        title,
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn create_tr(dir: &Path, title: &str, description: &str) -> String {
    let out = type_output(
        dir,
        &["test-report", "create", title, "--description", description],
    );
    assert!(
        out.status.success(),
        "test-report create failed: {}\nstderr: {}",
        title,
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

fn uc_path(dir: &Path, id: &str) -> PathBuf {
    dir.join("docs/use-cases").join(format!("{}.md", id))
}
fn uc_deprecated_path(dir: &Path, id: &str) -> PathBuf {
    dir.join("docs/use-cases/deprecated")
        .join(format!("{}.md", id))
}
fn tc_path(dir: &Path, id: &str) -> PathBuf {
    dir.join("docs/test-cases").join(format!("{}.md", id))
}
fn tr_path(dir: &Path, id: &str) -> PathBuf {
    dir.join("docs/test-reports").join(format!("{}.md", id))
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
fn milestone_use_cases_test_cases_test_reports_flow() {
    let dir = test_root("flow");
    init_project(&dir);

    assert_eq!(
        create_uc(&dir, "quick brown fox use case", "quick brown fox desc"),
        "UC-001"
    );
    assert_eq!(
        create_tc(&dir, "Hidden zebra test case", SECOND_PHRASE),
        "TC-001"
    );
    assert_eq!(
        create_tr(&dir, "third report test report", "third report desc"),
        "TR-001"
    );
    assert!(uc_path(&dir, "UC-001").exists());
    assert!(tc_path(&dir, "TC-001").exists());
    assert!(tr_path(&dir, "TR-001").exists());

    let d = dir_arg(&dir);
    let uc_list = run_ok(&["--dir", &d, "use-case", "list"]);
    assert!(uc_list.contains("UC-001 quick brown fox use case"));
    let tc_list = run_ok(&["--dir", &d, "test-case", "list"]);
    assert!(tc_list.contains("TC-001 Hidden zebra test case"));
    let tr_list = run_ok(&["--dir", &d, "test-report", "list"]);
    assert!(tr_list.contains("TR-001 third report test report"));

    assert_eq!(
        run_ok(&[
            "--dir",
            &d,
            "use-case",
            "edit",
            "UC-001",
            "--title",
            "quick brown fox updated"
        ]),
        ""
    );
    let view = run_ok(&["--dir", &d, "use-case", "view", "UC-001"]);
    assert!(view.contains("# UC-001 quick brown fox updated"));
    assert!(view.contains("## Description\nquick brown fox desc"));

    assert_eq!(
        run_ok(&[
            "--dir",
            &d,
            "use-case",
            "edit",
            "UC-001",
            "--description",
            "quick brown fox desc v2"
        ]),
        ""
    );
    let view = run_ok(&["--dir", &d, "use-case", "view", "UC-001"]);
    assert!(view.contains("# UC-001 quick brown fox updated"));
    assert!(view.contains("## Description\nquick brown fox desc v2"));

    let uc_content = fs::read_to_string(uc_path(&dir, "UC-001")).unwrap();
    assert_eq!(
        run_ok(&["--dir", &d, "use-case", "deprecate", "UC-001"]),
        ""
    );
    assert!(!uc_path(&dir, "UC-001").exists());
    assert!(uc_deprecated_path(&dir, "UC-001").exists());
    assert!(!run_ok(&["--dir", &d, "use-case", "list"]).contains("UC-001"));
    assert_eq!(run_status(&dir, &["use-case", "deprecate", "UC-001"]), 4);
    assert_eq!(run_ok(&["--dir", &d, "use-case", "restore", "UC-001"]), "");
    assert!(uc_path(&dir, "UC-001").exists());
    assert!(!uc_deprecated_path(&dir, "UC-001").exists());
    assert_eq!(
        fs::read_to_string(uc_path(&dir, "UC-001")).unwrap(),
        uc_content
    );
    assert!(run_ok(&["--dir", &d, "use-case", "list"]).contains("UC-001"));
    assert_eq!(run_status(&dir, &["use-case", "restore", "UC-001"]), 4);

    for args in [
        vec!["use-case", "view", "UC-404"],
        vec!["test-case", "view", "TC-404"],
        vec!["test-report", "view", "TR-404"],
    ] {
        assert_eq!(run_status(&dir, &args), 3, "args={args:?}");
    }
    assert_eq!(run_status(&dir, &["use-case", "create", "T"]), 2);
    assert_eq!(
        run_status(&dir, &["use-case", "create", "", "--description", "d"]),
        4
    );

    assert_eq!(
        create_uc(&dir, "Hidden zebra use case", SECOND_PHRASE),
        "UC-002"
    );
    assert_eq!(
        create_tc(&dir, "Hidden zebra test case", SECOND_PHRASE),
        "TC-002"
    );
    assert_eq!(
        create_tr(&dir, "Hidden zebra test report", SECOND_PHRASE),
        "TR-002"
    );

    let out = search(&dir, &[PHRASE]);
    assert!(out.contains("UC-001 doc quick brown fox updated"));
    assert!(!search(&dir, &["quick fox brown"]).contains("UC-001"));

    assert_eq!(
        create_uc(&dir, "marker use case", "sharedmarker desc"),
        "UC-003"
    );
    assert_eq!(
        create_tc(&dir, "marker test case", "sharedmarker desc"),
        "TC-003"
    );
    assert_eq!(
        create_tr(&dir, "marker test report", "sharedmarker desc"),
        "TR-003"
    );

    let out = search(&dir, &["sharedmarker"]);
    assert!(out.contains("UC-003 doc"));
    assert!(out.contains("TC-003 doc"));
    assert!(out.contains("TR-003 doc"));
    assert!(search(&dir, &["sharedmarker", "--type", "doc"]).contains("UC-003 doc"));
    assert!(search(&dir, &["sharedmarker", "--type", "doc"]).contains("TC-003 doc"));
    assert!(search(&dir, &["sharedmarker", "--type", "doc"]).contains("TR-003 doc"));
    for ty in ["task", "milestone", "release", "decision"] {
        let r = search(&dir, &["sharedmarker", "--type", ty]);
        assert!(!r.contains("UC-003"), "type={ty} uc");
        assert!(!r.contains("TC-003"), "type={ty} tc");
        assert!(!r.contains("TR-003"), "type={ty} tr");
    }
    assert_eq!(
        run_status(&dir, &["search", "sharedmarker", "--type", "use-case"]),
        2
    );
    assert_eq!(
        run_status(&dir, &["search", "sharedmarker", "--type", "test-case"]),
        2
    );
    assert_eq!(
        run_status(&dir, &["search", "sharedmarker", "--type", "test-report"]),
        2
    );

    assert_eq!(
        search(&dir, &["sharedmarker", "--limit", "1"])
            .lines()
            .count(),
        1
    );

    let json: serde_json::Value =
        serde_json::from_str(&search(&dir, &["--json", "sharedmarker"])).unwrap();
    let hits = json.as_array().unwrap();
    assert!(hits.len() >= 3);
    assert!(hits.iter().any(|h| h["id"] == "UC-003"));
    assert!(hits.iter().any(|h| h["id"] == "TC-003"));
    assert!(hits.iter().any(|h| h["id"] == "TR-003"));
    for h in hits {
        assert_eq!(h["type"], "doc");
        assert!(h.get("id").unwrap().is_string());
        assert!(h.get("title").unwrap().is_string());
        assert!(h.get("path").unwrap().is_string());
    }
    let empty: serde_json::Value =
        serde_json::from_str(&search(&dir, &["--json", "zzz-no-such-term"])).unwrap();
    assert_eq!(empty.as_array().unwrap().len(), 0);

    assert!(search(&dir, &[SECOND_PHRASE]).contains("UC-002 doc"));
    assert!(search(&dir, &[SECOND_PHRASE]).contains("TC-002 doc"));
    assert!(search(&dir, &[SECOND_PHRASE]).contains("TR-002 doc"));
    for (cmd, id) in [
        ("use-case", "UC-002"),
        ("test-case", "TC-002"),
        ("test-report", "TR-002"),
    ] {
        assert_eq!(run_ok(&["--dir", &d, cmd, "deprecate", id]), "");
    }
    let s = search(&dir, &[SECOND_PHRASE]);
    assert!(!s.contains("UC-002"));
    assert!(!s.contains("TC-002"));
    assert!(!s.contains("TR-002"));
    for (cmd, id) in [
        ("use-case", "UC-002"),
        ("test-case", "TC-002"),
        ("test-report", "TR-002"),
    ] {
        assert_eq!(run_ok(&["--dir", &d, cmd, "restore", id]), "");
    }
    let s = search(&dir, &[SECOND_PHRASE]);
    assert!(s.contains("UC-002 doc"));
    assert!(s.contains("TC-002 doc"));
    assert!(s.contains("TR-002 doc"));

    for (cmd, id) in [
        ("use-case", "UC-002"),
        ("test-case", "TC-002"),
        ("test-report", "TR-002"),
    ] {
        assert_eq!(run_ok(&["--dir", &d, cmd, "deprecate", id]), "");
    }

    let original = "# UC-001 quick brown fox updated\n\nPreamble kept.\n\n## Description\nquick brown fox desc v2\n\n## Editor Notes\nKeep this line\n\n## Unknown\nAnd this\n";
    fs::write(uc_path(&dir, "UC-001"), original).unwrap();

    let server = Server::start(&dir, None);

    let res = server.get("/?type=use-case");
    assert_eq!(res.status, 200);
    let list = res.text();
    assert!(list.contains("<h2 class=\"h2\">Use Cases</h2>"));
    assert!(list.contains("UC-001"));
    assert!(list.contains("quick brown fox updated"));
    assert!(list.contains("quick brown fox desc v2"));
    let audit_at = list.find("href=\"/?type=audit-record\"").unwrap();
    let tech_at = list.find("href=\"/?type=tech-spec\"").unwrap();
    let uc_at = list.find("href=\"/?type=use-case\"").unwrap();
    let tc_at = list.find("href=\"/?type=test-case\"").unwrap();
    let tr_at = list.find("href=\"/?type=test-report\"").unwrap();
    let nfr_at = list.find("href=\"/?type=nfr\"").unwrap();
    assert!(audit_at < tech_at);
    assert!(tech_at < uc_at);
    assert!(uc_at < tc_at);
    assert!(tc_at < tr_at);
    assert!(tr_at < nfr_at);
    assert!(list.contains("href=\"/?type=use-case\" aria-current=\"page\""));
    assert_markup_hooks(&list, "use-case list");
    assert_no_external_refs(&list, "use-case list");

    for (ty, label) in [("test-case", "Test Cases"), ("test-report", "Test Reports")] {
        let res = server.get(&format!("/?type={}", ty));
        assert_eq!(res.status, 200);
        let body = res.text();
        assert!(body.contains(&format!("<h2 class=\"h2\">{}</h2>", label)));
        assert!(body.contains("aria-current=\"page\""));
        assert_markup_hooks(&body, ty);
        assert_no_external_refs(&body, ty);
    }

    let res = server.get("/doc/UC-001");
    assert_eq!(res.status, 200);
    let view = res.text();
    assert!(view.contains("UC-001"));
    assert!(view.contains("Use case"));
    assert!(view.contains("Use Cases"));
    assert!(view.contains("<h1 class=\"h1\">quick brown fox updated</h1>"));
    assert!(
        view.contains(
            "<p class=\"text\" style=\"white-space: pre-wrap\">quick brown fox desc v2</p>"
        )
    );
    assert!(!view.contains("Description</h"));
    assert_eq!(view.matches("href=\"/doc/UC-001/edit\"").count(), 1);
    assert!(view.contains(">Edit</a>"));
    assert_markup_hooks(&view, "uc view");
    assert_no_external_refs(&view, "uc view");

    for alias in ["/api/use-cases", "/api/use-case"] {
        let res = server.get(alias);
        assert_eq!(res.status, 200, "alias {alias}");
        assert!(
            res.header("Content-Type")
                .unwrap()
                .starts_with("application/json")
        );
        let api: serde_json::Value = serde_json::from_str(&res.text()).unwrap();
        let arr = api.as_array().unwrap();
        assert!(arr.iter().any(|r| r["id"] == "UC-001"));
    }
    for alias in ["/api/test-cases", "/api/test-case"] {
        let res = server.get(alias);
        assert_eq!(res.status, 200, "alias {alias}");
    }
    for alias in ["/api/test-reports", "/api/test-report"] {
        let res = server.get(alias);
        assert_eq!(res.status, 200, "alias {alias}");
    }

    let res = server.get("/api/doc/UC-001");
    assert_eq!(res.status, 200);
    let doc: serde_json::Value = serde_json::from_str(&res.text()).unwrap();
    assert_eq!(doc["id"], "UC-001");
    assert_eq!(doc["kind"], "use-case");
    assert_eq!(doc["form"], "use-case");
    assert_eq!(doc["title"], "quick brown fox updated");
    assert_eq!(doc["description"], "quick brown fox desc v2");
    assert!(doc["version"].is_string());
    let path = doc["path"].as_str().unwrap();
    assert!(path.ends_with("UC-001.md"));
    assert!(!path.starts_with('/'));

    let res = server.get("/doc/UC-001/edit");
    assert_eq!(res.status, 200);
    let edit = res.text();
    assert!(edit.contains("Hidden field: version (optimistic-concurrency token)"));
    assert!(edit.contains("name=\"title\""));
    assert!(edit.contains("name=\"description\""));
    assert!(edit.contains("Save changes"));
    assert!(edit.contains("Cancel"));
    assert_markup_hooks(&edit, "uc edit");
    assert_no_external_refs(&edit, "uc edit");
    let version = extract_version(&edit);

    let body = form_body(&[
        ("version", version.as_str()),
        ("title", "quick brown fox browser edited"),
        ("description", "New body\nsecond line"),
    ]);
    let res = server.post_form("/doc/UC-001", &body);
    assert_eq!(res.status, 303);
    assert_eq!(res.header("Location"), Some("/doc/UC-001"));

    let view = server.get("/doc/UC-001").text();
    assert!(view.contains("<h1 class=\"h1\">quick brown fox browser edited</h1>"));
    assert!(view.contains("New body\nsecond line"));
    let cli = run_ok(&["--dir", &d, "use-case", "view", "UC-001"]);
    assert!(cli.contains("quick brown fox browser edited"));
    assert!(cli.contains("New body"));
    assert!(cli.contains("second line"));

    let after = fs::read_to_string(uc_path(&dir, "UC-001")).unwrap();
    assert!(after.starts_with("---\n"));
    assert!(after.contains("# UC-001 quick brown fox browser edited\n"));
    assert!(after.contains("Preamble kept."));
    assert!(after.contains("## Description\nNew body\nsecond line"));
    assert!(after.contains("## Editor Notes\nKeep this line"));
    assert!(after.contains("## Unknown\nAnd this"));

    let res = server.get("/assets/app.css");
    assert_eq!(res.status, 200);
    assert!(res.header("Content-Type").unwrap().starts_with("text/css"));
    assert_eq!(res.header("Cache-Control"), Some("no-store"));
    assert!(!res.body.is_empty());
    assert_no_external_refs(&res.text(), "stylesheet");
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

    for (id, help, recovery_text) in [
        ("UC-002", NOT_FOUND_UC, "Back to Use Cases"),
        ("TC-002", NOT_FOUND_TC, "Back to Test Cases"),
        ("TR-002", NOT_FOUND_TR, "Back to Test Reports"),
    ] {
        let recovery_href = match id {
            "UC-002" => "/?type=use-case",
            "TC-002" => "/?type=test-case",
            "TR-002" => "/?type=test-report",
            _ => unreachable!(),
        };
        let unknown_id = match id {
            "UC-002" => "UC-404",
            "TC-002" => "TC-404",
            "TR-002" => "TR-404",
            _ => unreachable!(),
        };
        let url = format!("/doc/{}", id);
        let resp = server.get(&url);
        assert_eq!(resp.status, 404);
        let not_found = resp.text();
        assert!(not_found.contains("Record not found"));
        assert!(not_found.contains(help));
        assert!(not_found.contains(&format!("href=\"{}\"", recovery_href)));
        assert!(not_found.contains(&format!(">{}</a>", recovery_text)));
        let unknown = server.get(&format!("/doc/{}", unknown_id));
        assert_eq!(unknown.status, 404);
        assert_eq!(not_found, unknown.text());
        let edit_res = server.get(&format!("/doc/{}/edit", id));
        assert_eq!(edit_res.status, 404);
        assert_eq!(not_found, edit_res.text());
        let body = form_body(&[("version", "1.2.3"), ("title", "Ok"), ("description", "ok")]);
        let post_res = server.post_form(&format!("/doc/{}", id), &body);
        assert_eq!(post_res.status, 404);
        assert_eq!(not_found, post_res.text());
        let api_res = server.get(&format!("/api/doc/{}", id));
        assert_eq!(api_res.status, 404);
        let json: serde_json::Value = serde_json::from_str(&api_res.text()).unwrap();
        assert_eq!(json["error"]["code"], 404);
        let api_post = server.post_json(
            &format!("/api/doc/{}", id),
            "{\"version\":\"1.2.3\",\"title\":\"Ok\"}",
        );
        assert_eq!(api_post.status, 404);
        let json: serde_json::Value = serde_json::from_str(&api_post.text()).unwrap();
        assert_eq!(json["error"]["code"], 404);
    }

    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn installed_layout_has_no_runtime_asset_dependency() {
    let dir = test_root("assets");
    init_project(&dir);
    assert_eq!(
        create_uc(&dir, "quick brown fox architecture", "quick brown fox desc"),
        "UC-001"
    );

    let empty_cwd = test_root("emptycwd");
    let out = search_in(&empty_cwd, &dir, &[PHRASE]);
    assert!(out.contains("UC-001 doc quick brown fox architecture"));

    let server = Server::start(&dir, Some(&empty_cwd));
    let res = server.get("/?type=use-case");
    assert_eq!(res.status, 200);
    assert!(res.text().contains("UC-001"));
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
