//! Milestone-level smoke test for TASK-16 (m-3: decision records).
//!
//! End-to-end over the built binary: decision lifecycle, search over decision
//! records, and the offline local browser over the decision category. Narrower
//! than the TASK-13/TASK-14/TASK-15 contract suites.

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
    path.push(format!("decision-records-{}-{}", name, std::process::id()));
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

fn decision_fixture(dir: &Path) {
    init_project(dir);
    assert_eq!(
        run_ok(&[
            "--dir",
            &dir_arg(dir),
            "decision",
            "create",
            "Decision One",
            "-d",
            "quick brown fox decision",
        ]),
        "ADR-001"
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
    fn start(project: &Path) -> Server {
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
    let status: u16 = head
        .split("\r\n")
        .next()
        .unwrap_or("")
        .split(' ')
        .nth(1)
        .and_then(|value| value.parse().ok())
        .expect("status code");
    HttpResponse { status, body }
}

#[test]
fn decision_lifecycle_cli() {
    let dir = test_root("lifecycle");
    decision_fixture(&dir);
    assert_eq!(
        run_ok(&["--dir", &dir_arg(&dir), "decision", "list"]).trim(),
        "ADR-001 Decision One"
    );
    let view = run_ok(&["--dir", &dir_arg(&dir), "decision", "view", "ADR-001"]);
    assert!(view.contains("## Status"));
    assert!(view.contains("proposed"));
    assert_eq!(
        run_ok(&[
            "--dir",
            &dir_arg(&dir),
            "decision",
            "edit",
            "ADR-001",
            "--status",
            "accepted",
        ]),
        ""
    );
    let view = run_ok(&["--dir", &dir_arg(&dir), "decision", "view", "ADR-001"]);
    assert!(view.contains("accepted"));
    assert_eq!(
        run_ok(&["--dir", &dir_arg(&dir), "decision", "archive", "ADR-001"]),
        ""
    );
    // archived viewable
    let view = run_ok(&["--dir", &dir_arg(&dir), "decision", "view", "ADR-001"]);
    assert!(view.contains("ADR-001"));
}

#[test]
fn decision_search_smoke() {
    let dir = test_root("search");
    decision_fixture(&dir);
    let out = search(&dir, &[PHRASE]);
    assert!(out.contains("ADR-001 decision Decision One"));
    let out = search(&dir, &[PHRASE, "--type", "decision"]);
    assert!(out.contains("ADR-001 decision Decision One"));
    // limit
    let out = search(&dir, &[PHRASE, "--limit", "1"]);
    assert_eq!(out.lines().count(), 1);
}

#[test]
fn decision_browser_smoke() {
    let dir = test_root("browser");
    decision_fixture(&dir);
    let server = Server::start(&dir);
    // list
    let res = server.get("/?type=decision");
    assert_eq!(res.status, 200);
    let body = res.text();
    assert!(body.contains("ADR-001"));
    assert!(body.contains("aria-current=\"page\""));
    // view
    let res = server.get("/doc/ADR-001");
    assert_eq!(res.status, 200);
    let body = res.text();
    assert!(body.contains("Decision One"));
    assert!(body.contains("proposed"));
    // JSON api
    let res = server.get("/api/decisions");
    assert_eq!(res.status, 200);
    assert!(res.text().contains("ADR-001"));
}
