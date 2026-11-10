//! Milestone-level smoke test for TASK-20 (m-12: complete content management).

use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
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
    path.push(format!("content-mgmt-{}-{}", name, std::process::id()));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path
}

fn dir_arg(dir: &Path) -> String {
    dir.to_str().unwrap().to_string()
}

fn init_project(dir: &Path) {
    let out = bin()
        .args(["--dir", &dir_arg(dir), "init", "Proj", "--no-git"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "init failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
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

fn search(dir: &Path, args: &[&str]) -> String {
    let d = dir_arg(dir);
    let mut full: Vec<&str> = vec!["--dir", d.as_str(), "search"];
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
        let mut stream = std::net::TcpStream::connect(("127.0.0.1", self.port)).unwrap();
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
    body: Vec<u8>,
}

impl HttpResponse {
    fn text(&self) -> String {
        String::from_utf8_lossy(&self.body).to_string()
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
    HttpResponse { status, body }
}

fn assert_no_external_refs(body: &str, page: &str) {
    for needle in ["http://", "https://", "@import", "url(", "<img"] {
        assert!(
            !body.to_lowercase().contains(needle),
            "{page} references an external origin via {needle}"
        );
    }
}

#[test]
fn milestone_content_management_flow() {
    let dir = test_root("flow");
    init_project(&dir);
    let p = dir_arg(&dir);

    assert_eq!(
        run_ok(&["--dir", &p, "req", "create-group", "--title", "Group"]),
        "REQ-001"
    );
    assert_eq!(
        run_ok(&[
            "--dir",
            &p,
            "req",
            "create",
            "Req Title",
            "--group",
            "REQ-001",
            "-d",
            PHRASE
        ]),
        "REQ-001.001"
    );

    assert_eq!(
        run_ok(&["--dir", &p, "nfreq", "create-group", "--title", "NGroup"]),
        "NFR-001"
    );
    assert_eq!(
        run_ok(&[
            "--dir",
            &p,
            "nfreq",
            "create",
            "Nfr Title",
            "--group",
            "NFR-001",
            "-d",
            PHRASE
        ]),
        "NFR-001.001"
    );

    assert_eq!(
        run_ok(&["--dir", &p, "task", "create", "Task Title", "-d", PHRASE]),
        "TASK-001"
    );
    assert_eq!(
        run_ok(&[
            "--dir",
            &p,
            "milestone",
            "create",
            "Milestone Title",
            "-d",
            PHRASE
        ]),
        "MS-001"
    );
    assert_eq!(
        run_ok(&[
            "--dir",
            &p,
            "release",
            "create",
            "Release Title",
            "-d",
            PHRASE
        ]),
        "REL-001"
    );
    assert_eq!(
        run_ok(&[
            "--dir",
            &p,
            "decision",
            "create",
            "Decision Title",
            "-d",
            PHRASE
        ]),
        "ADR-001"
    );
    assert_eq!(
        run_ok(&[
            "--dir",
            &p,
            "glossary",
            "create",
            "--term",
            "Glossary Term",
            "--definition",
            PHRASE
        ]),
        "GLO-001"
    );
    assert_eq!(
        run_ok(&[
            "--dir",
            &p,
            "ui-spec",
            "create",
            "UI Title",
            "--view",
            "v",
            "--description",
            PHRASE
        ]),
        "UI-001"
    );
    assert_eq!(
        run_ok(&[
            "--dir",
            &p,
            "ui-mockup",
            "create",
            "Mock Title",
            "--task",
            "TASK-001",
            "--spec",
            "UI-001"
        ]),
        "MOCK-001"
    );
    assert_eq!(
        run_ok(&[
            "--dir",
            &p,
            "rule",
            "create",
            "Rule Title",
            "--description",
            PHRASE,
            "--rationale",
            "rat",
            "--applicability",
            "app"
        ]),
        "RUL-001"
    );
    assert_eq!(
        run_ok(&[
            "--dir",
            &p,
            "audit",
            "create",
            "--title",
            "Audit Title",
            "--content",
            PHRASE
        ]),
        "AUD-001"
    );
    assert_eq!(
        run_ok(&[
            "--dir",
            &p,
            "tech-spec",
            "create",
            "Tech Title",
            "--description",
            PHRASE
        ]),
        "TECH-001"
    );

    let out = search(&dir, &[PHRASE]);
    assert!(out.contains("REQ-001.001"));
    assert!(out.contains("TASK-001"));
    assert!(out.contains("ADR-001"));
    assert!(!search(&dir, &["quick fox brown"]).contains("REQ-001.001"));

    let json: serde_json::Value = serde_json::from_str(&search(&dir, &["--json", PHRASE])).unwrap();
    assert!(json.as_array().unwrap().len() >= 3);

    let doc_out = search(&dir, &[PHRASE, "--type", "doc"]);
    assert!(doc_out.contains("REQ-001.001"));

    let task_out = search(&dir, &[PHRASE, "--type", "task"]);
    assert!(task_out.contains("TASK-001"));

    let limited = search(&dir, &[PHRASE, "--limit", "1"]);
    assert_eq!(limited.lines().count(), 1);

    let empty: serde_json::Value =
        serde_json::from_str(&search(&dir, &["--json", "zzz-no-such-term"])).unwrap();
    assert_eq!(empty.as_array().unwrap().len(), 0);

    let server = Server::start(&dir, None);
    let res = server.get("/");
    assert_eq!(res.status, 200);
    let body = res.text();
    assert!(body.contains("Browser inventory"));
    assert_no_external_refs(&body, "inventory");

    let res = server.get("/?type=req");
    assert_eq!(res.status, 200);
    assert!(res.text().contains("REQ-001.001"));

    let res = server.get("/doc/REQ-001.001");
    assert_eq!(res.status, 200);
    assert!(res.text().contains("REQ-001.001"));

    let css = server.get("/assets/app.css");
    assert_eq!(css.status, 200);
    assert!(!css.body.is_empty());
    let js = server.get("/assets/app.js");
    assert_eq!(js.status, 200);
    assert!(!js.body.is_empty());

    assert!(
        body.contains("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">")
    );
    assert!(body.contains("aria-expanded=\"true\""));
    assert!(body.contains("aria-controls=\"type-list\""));

    let lc = server.get("/lifecycle");
    assert_eq!(lc.status, 200);
    assert!(lc.text().contains("Managed artifacts"));
    let sk = server.get("/skills");
    assert_eq!(sk.status, 200);
    assert!(sk.text().contains("Agent skills"));

    assert_eq!(server.get("/doc/ZZZ-999").status, 404);

    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn content_management_cross_type_resolution() {
    let dir = test_root("cross");
    init_project(&dir);
    let p = dir_arg(&dir);

    assert_eq!(
        run_ok(&["--dir", &p, "req", "create-group", "--title", "Group"]),
        "REQ-001"
    );
    assert_eq!(
        run_ok(&[
            "--dir",
            &p,
            "req",
            "create",
            "Req Title",
            "--group",
            "REQ-001",
            "-d",
            PHRASE
        ]),
        "REQ-001.001"
    );

    assert_eq!(
        run_ok(&["--dir", &p, "req", "deprecate", "REQ-001.001"]),
        ""
    );

    let server = Server::start(&dir, None);
    assert_eq!(server.get("/doc/REQ-001.001").status, 404);
    assert_eq!(server.get("/doc/NOPE-999").status, 404);
    drop(server);

    assert_eq!(run_ok(&["--dir", &p, "req", "restore", "REQ-001.001"]), "");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn content_management_metadata_and_pages() {
    let dir = test_root("meta");
    init_project(&dir);
    let p = dir_arg(&dir);

    assert_eq!(
        run_ok(&[
            "--dir",
            &p,
            "tech-spec",
            "create",
            "Tech Title",
            "--description",
            PHRASE
        ]),
        "TECH-001"
    );

    let server = Server::start(&dir, None);
    let res = server.get("/doc/TECH-001");
    assert_eq!(res.status, 200);
    let body = res.text();
    assert!(body.contains("TECH-001"));
    assert!(body.contains("Tech Title"));

    let lc = server.get("/lifecycle");
    assert_eq!(lc.status, 200);
    assert!(lc.text().contains("Managed artifacts"));

    let sk = server.get("/skills");
    assert_eq!(sk.status, 200);
    assert!(sk.text().contains("Agent skills"));

    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn installed_layout_has_no_runtime_asset_dependency() {
    let dir = test_root("assets");
    init_project(&dir);
    let p = dir_arg(&dir);

    assert_eq!(
        run_ok(&[
            "--dir",
            &p,
            "decision",
            "create",
            "Decision One",
            "-d",
            PHRASE
        ]),
        "ADR-001"
    );

    let empty_cwd = test_root("emptycwd");
    let out = run_ok_in(&empty_cwd, &["--dir", &p, "search", PHRASE]);
    assert!(out.contains("ADR-001"));

    let server = Server::start(&dir, Some(&empty_cwd));
    let css = server.get("/assets/app.css");
    assert_eq!(css.status, 200);
    assert!(!css.body.is_empty());
    let js = server.get("/assets/app.js");
    assert_eq!(js.status, 200);
    assert!(!js.body.is_empty());
    drop(server);

    let _ = fs::remove_dir_all(&dir);
    let _ = fs::remove_dir_all(&empty_cwd);
}
