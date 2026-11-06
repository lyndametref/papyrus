//! Milestone-level smoke test for TASK-62 (m-4: CLI packaging and verification).

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
    path.push(format!(
        "cli-milestone-smoke-{}-{}",
        name,
        std::process::id()
    ));
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

fn run_in(dir: &Path, args: &[&str]) -> std::process::Output {
    bin().current_dir(dir).args(args).output().unwrap()
}

fn create_record(project: &Path, args: &[&str]) -> String {
    let dir = dir_arg(project);
    let mut full = vec!["--dir", dir.as_str()];
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
    _cwd: PathBuf,
}

impl Server {
    fn start(project: &Path, cwd: &Path) -> Server {
        for _ in 0..10 {
            let port = free_port();
            let mut command = bin();
            command
                .current_dir(cwd)
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
            return Server {
                child,
                port,
                _cwd: cwd.to_path_buf(),
            };
        }
        panic!("browser did not start after retries");
    }

    fn get(&self, path: &str) -> HttpResponse {
        let raw = format!(
            "GET {} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n\r\n",
            path, self.port
        )
        .into_bytes();
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
fn milestone_cli_records_flow() {
    let out = bin().arg("--version").output().unwrap();
    assert!(out.status.success(), "version failed");
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "papyrus 0.1.0");

    let out = bin().arg("--help").output().unwrap();
    assert!(out.status.success(), "help failed");
    let help = String::from_utf8_lossy(&out.stdout);
    for cmd in [
        "glossary",
        "ui-spec",
        "ui-mockup",
        "rule",
        "audit",
        "tech-spec",
        "decision",
        "task",
        "milestone",
        "release",
        "search",
        "browser",
    ] {
        assert!(help.contains(cmd), "--help missing subcommand: {}", cmd);
    }

    let dir = test_root("records");
    init_project(&dir);
    let p = dir_arg(&dir);

    // glossary
    assert_eq!(
        create_record(
            &dir,
            &[
                "glossary",
                "create",
                "--term",
                PHRASE,
                "--definition",
                "def"
            ]
        ),
        "GLO-001"
    );
    assert!(run_ok(&["--dir", &p, "glossary", "list"]).contains("GLO-001"));
    let view = run_ok(&["--dir", &p, "glossary", "view", "GLO-001"]);
    assert!(view.contains("# GLO-001"));
    assert!(view.contains("## Definition"));
    assert_eq!(
        run_ok(&[
            "--dir",
            &p,
            "glossary",
            "edit",
            "GLO-001",
            "--term",
            "Updated Term"
        ]),
        ""
    );
    let view = run_ok(&["--dir", &p, "glossary", "view", "GLO-001"]);
    assert!(view.contains("# GLO-001 Updated Term"));
    assert!(view.contains("## Definition\ndef"));
    assert_eq!(
        run_ok(&["--dir", &p, "glossary", "deprecate", "GLO-001"]),
        ""
    );
    assert!(!run_ok(&["--dir", &p, "glossary", "list"]).contains("GLO-001"));
    assert!(run_ok(&["--dir", &p, "glossary", "view", "GLO-001"]).contains("GLO-001"));
    assert_eq!(run_ok(&["--dir", &p, "glossary", "restore", "GLO-001"]), "");
    assert!(run_ok(&["--dir", &p, "glossary", "list"]).contains("GLO-001"));

    // ui-spec
    assert_eq!(
        create_record(
            &dir,
            &[
                "ui-spec",
                "create",
                "UI Title",
                "--view",
                "v",
                "--description",
                PHRASE
            ]
        ),
        "UI-001"
    );
    assert!(run_ok(&["--dir", &p, "ui-spec", "list"]).contains("UI-001"));
    assert!(run_ok(&["--dir", &p, "ui-spec", "view", "UI-001"]).contains("# UI-001 UI Title"));
    assert_eq!(
        run_ok(&["--dir", &p, "ui-spec", "edit", "UI-001", "--view", "v2"]),
        ""
    );
    assert!(run_ok(&["--dir", &p, "ui-spec", "view", "UI-001"]).contains("## View\nv2"));
    assert_eq!(run_ok(&["--dir", &p, "ui-spec", "deprecate", "UI-001"]), "");
    assert!(!run_ok(&["--dir", &p, "ui-spec", "list"]).contains("UI-001"));
    assert_eq!(run_ok(&["--dir", &p, "ui-spec", "restore", "UI-001"]), "");
    assert!(run_ok(&["--dir", &p, "ui-spec", "list"]).contains("UI-001"));

    // ui-mockup
    assert_eq!(
        create_record(
            &dir,
            &[
                "ui-mockup",
                "create",
                "Mock Title",
                "--task",
                "TASK-1",
                "--spec",
                "UI-001"
            ]
        ),
        "MOCK-001"
    );
    assert!(run_ok(&["--dir", &p, "ui-mockup", "list"]).contains("MOCK-001"));
    assert!(run_ok(&["--dir", &p, "ui-mockup", "view", "MOCK-001"]).contains("MOCK-001"));
    assert_eq!(
        run_ok(&[
            "--dir",
            &p,
            "ui-mockup",
            "edit",
            "MOCK-001",
            "--title",
            "Mock Updated"
        ]),
        ""
    );
    assert!(run_ok(&["--dir", &p, "ui-mockup", "view", "MOCK-001"]).contains("Mock Updated"));
    assert_eq!(
        run_ok(&["--dir", &p, "ui-mockup", "deprecate", "MOCK-001"]),
        ""
    );
    assert!(!run_ok(&["--dir", &p, "ui-mockup", "list"]).contains("MOCK-001"));
    assert_eq!(
        run_ok(&["--dir", &p, "ui-mockup", "restore", "MOCK-001"]),
        ""
    );
    assert!(run_ok(&["--dir", &p, "ui-mockup", "list"]).contains("MOCK-001"));

    // rule
    assert_eq!(
        create_record(
            &dir,
            &[
                "rule",
                "create",
                "Rule Title",
                "--description",
                PHRASE,
                "--rationale",
                "rat",
                "--applicability",
                "app"
            ]
        ),
        "RUL-001"
    );
    assert!(run_ok(&["--dir", &p, "rule", "list"]).contains("RUL-001"));
    assert!(run_ok(&["--dir", &p, "rule", "view", "RUL-001"]).contains("# RUL-001 Rule Title"));
    assert_eq!(
        run_ok(&[
            "--dir",
            &p,
            "rule",
            "edit",
            "RUL-001",
            "--title",
            "Rule Updated"
        ]),
        ""
    );
    assert!(run_ok(&["--dir", &p, "rule", "view", "RUL-001"]).contains("# RUL-001 Rule Updated"));
    assert_eq!(run_ok(&["--dir", &p, "rule", "deprecate", "RUL-001"]), "");
    assert!(!run_ok(&["--dir", &p, "rule", "list"]).contains("RUL-001"));
    assert_eq!(run_ok(&["--dir", &p, "rule", "restore", "RUL-001"]), "");
    assert!(run_ok(&["--dir", &p, "rule", "list"]).contains("RUL-001"));

    // audit
    assert_eq!(
        create_record(
            &dir,
            &[
                "audit",
                "create",
                "--title",
                "Audit Title",
                "--content",
                PHRASE
            ]
        ),
        "AUD-001"
    );
    assert!(run_ok(&["--dir", &p, "audit", "list"]).contains("AUD-001"));
    assert!(run_ok(&["--dir", &p, "audit", "view", "AUD-001"]).contains("AUD-001"));
    assert_eq!(
        run_ok(&[
            "--dir",
            &p,
            "audit",
            "edit",
            "AUD-001",
            "--title",
            "Audit Updated"
        ]),
        ""
    );
    assert!(run_ok(&["--dir", &p, "audit", "view", "AUD-001"]).contains("Audit Updated"));
    assert_eq!(run_ok(&["--dir", &p, "audit", "deprecate", "AUD-001"]), "");
    assert!(!run_ok(&["--dir", &p, "audit", "list"]).contains("AUD-001"));
    assert_eq!(run_ok(&["--dir", &p, "audit", "restore", "AUD-001"]), "");
    assert!(run_ok(&["--dir", &p, "audit", "list"]).contains("AUD-001"));

    // tech-spec
    assert_eq!(
        create_record(
            &dir,
            &["tech-spec", "create", "Tech Title", "--description", PHRASE]
        ),
        "TECH-001"
    );
    assert!(run_ok(&["--dir", &p, "tech-spec", "list"]).contains("TECH-001"));
    assert!(
        run_ok(&["--dir", &p, "tech-spec", "view", "TECH-001"]).contains("# TECH-001 Tech Title")
    );
    assert_eq!(
        run_ok(&[
            "--dir",
            &p,
            "tech-spec",
            "edit",
            "TECH-001",
            "--title",
            "Tech Updated"
        ]),
        ""
    );
    assert!(
        run_ok(&["--dir", &p, "tech-spec", "view", "TECH-001"]).contains("# TECH-001 Tech Updated")
    );
    assert_eq!(
        run_ok(&["--dir", &p, "tech-spec", "deprecate", "TECH-001"]),
        ""
    );
    assert!(!run_ok(&["--dir", &p, "tech-spec", "list"]).contains("TECH-001"));
    assert_eq!(
        run_ok(&["--dir", &p, "tech-spec", "restore", "TECH-001"]),
        ""
    );
    assert!(run_ok(&["--dir", &p, "tech-spec", "list"]).contains("TECH-001"));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn installed_layout_has_no_runtime_asset_dependency() {
    let empty_cwd = test_root("empty-cwd");
    fs::create_dir_all(&empty_cwd).unwrap();

    let project = test_root("asset-project");
    init_project(&project);
    let p = dir_arg(&project);

    assert_eq!(
        run_ok(&[
            "--dir",
            &p,
            "decision",
            "create",
            "Decision One",
            "-d",
            PHRASE,
        ]),
        "ADR-001"
    );

    let out = run_in(&empty_cwd, &["--dir", &p, "search", PHRASE]);
    assert!(
        out.status.success(),
        "search failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(String::from_utf8_lossy(&out.stdout).contains("ADR-001 decision Decision One"));

    let server = Server::start(&project, &empty_cwd);
    let css = server.get("/assets/app.css");
    assert_eq!(css.status, 200);
    assert!(!css.body.is_empty());
    let js = server.get("/assets/app.js");
    assert_eq!(js.status, 200);
    assert!(!js.body.is_empty());

    let _ = fs::remove_dir_all(&empty_cwd);
    let _ = fs::remove_dir_all(&project);
}
