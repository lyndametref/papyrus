//! Integration tests for `papyrus browser` (specification checks T-01 to T-30).

use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_papyrus"))
}

fn test_root(name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(".tmp");
    path.push(format!("browser-{}-{}", name, std::process::id()));
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

fn init_project(dir: &Path) {
    run_ok(&["--dir", dir.to_str().unwrap(), "init", "Proj", "--no-git"]);
}

fn create_req_group(dir: &Path, title: &str) -> String {
    run_ok(&[
        "--dir",
        dir.to_str().unwrap(),
        "req",
        "create-group",
        "--title",
        title,
    ])
}

fn create_req(dir: &Path, title: &str, group: &str) -> String {
    let story = format!("As a user I want {}", title);
    let description = format!("Description of {}", title);
    run_ok(&[
        "--dir",
        dir.to_str().unwrap(),
        "req",
        "create",
        title,
        "--group",
        group,
        "--story",
        &story,
        "-d",
        &description,
    ])
}

fn create_nfr_group(dir: &Path, title: &str) -> String {
    run_ok(&[
        "--dir",
        dir.to_str().unwrap(),
        "nfreq",
        "create-group",
        "--title",
        title,
    ])
}

fn create_nfr(dir: &Path, title: &str, group: &str) -> String {
    let description = format!("Description of {}", title);
    run_ok(&[
        "--dir",
        dir.to_str().unwrap(),
        "nfreq",
        "create",
        title,
        "--group",
        group,
        "-d",
        &description,
    ])
}

fn fixture(name: &str) -> PathBuf {
    let dir = test_root(name);
    init_project(&dir);
    create_req_group(&dir, "Payments Group");
    create_req(&dir, "Capture payment", "REQ-001");
    create_req_group(&dir, "Search Group");
    create_req(&dir, "Search records", "REQ-002");
    create_nfr_group(&dir, "Persistence");
    create_nfr(&dir, "Restrict access", "NFR-001");
    dir
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
    stderr_path: PathBuf,
}

impl Server {
    fn start(project: &Path, extra: &[&str], envs: &[(&str, &str)]) -> Server {
        for attempt in 0..10 {
            let port = free_port();
            let stderr_path = project.join(format!(".stderr-{}-{}", port, attempt));
            let stderr = fs::File::create(&stderr_path).unwrap();
            let mut command = bin();
            command
                .arg("--dir")
                .arg(project)
                .arg("browser")
                .arg("--port")
                .arg(port.to_string())
                .args(extra)
                .envs(envs.iter().cloned())
                .stdout(Stdio::piped())
                .stderr(Stdio::from(stderr));
            let mut child = command.spawn().unwrap();
            let stdout = child.stdout.take().unwrap();
            let mut reader = BufReader::new(stdout);
            let mut startup = String::new();
            let _ = reader.read_line(&mut startup);
            if startup.trim().is_empty() {
                let _ = child.wait();
                continue;
            }
            let _ = startup;
            return Server {
                child,
                port,
                stderr_path,
            };
        }
        panic!("server did not start after retries");
    }

    fn stderr_text(&self) -> String {
        fs::read_to_string(&self.stderr_path).unwrap_or_default()
    }

    fn wait_for_stderr(&self, needle: &str) {
        let deadline = Instant::now() + Duration::from_secs(3);
        while Instant::now() < deadline {
            if self.stderr_text().contains(needle) {
                return;
            }
            std::thread::sleep(Duration::from_millis(25));
        }
        panic!(
            "stderr did not contain {:?}: {}",
            needle,
            self.stderr_text()
        );
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
    let status_line = lines.next().unwrap_or("");
    let status: u16 = status_line
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

fn send_raw(port: u16, raw: &[u8]) -> HttpResponse {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(15)))
        .unwrap();
    stream.write_all(raw).unwrap();
    let mut bytes = Vec::new();
    stream.read_to_end(&mut bytes).unwrap();
    parse_response(&bytes)
}

fn request(
    port: u16,
    method: &str,
    path: &str,
    extra_headers: &[(&str, &str)],
    body: Option<&[u8]>,
) -> HttpResponse {
    let mut raw = format!(
        "{} {} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n",
        method, path, port
    )
    .into_bytes();
    for (name, value) in extra_headers {
        raw.extend_from_slice(format!("{}: {}\r\n", name, value).as_bytes());
    }
    if let Some(body) = body {
        raw.extend_from_slice(format!("Content-Length: {}\r\n", body.len()).as_bytes());
    }
    raw.extend_from_slice(b"\r\n");
    if let Some(body) = body {
        raw.extend_from_slice(body);
    }
    send_raw(port, &raw)
}

fn get(server: &Server, path: &str) -> HttpResponse {
    request(server.port, "GET", path, &[], None)
}

fn post_form(server: &Server, path: &str, body: &str) -> HttpResponse {
    request(
        server.port,
        "POST",
        path,
        &[("Content-Type", "application/x-www-form-urlencoded")],
        Some(body.as_bytes()),
    )
}

fn post_json(server: &Server, path: &str, body: &str) -> HttpResponse {
    request(
        server.port,
        "POST",
        path,
        &[("Content-Type", "application/json")],
        Some(body.as_bytes()),
    )
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

fn record_path(dir: &Path, kind: &str, id: &str) -> PathBuf {
    let sub = if kind == "req" {
        "docs/requirements/business"
    } else {
        "docs/requirements/non-functional"
    };
    dir.join(sub).join(format!("{}.md", id))
}

// T-01
#[test]
fn t01_command_shape() {
    let out = bin().args(["browser", "--help"]).output().unwrap();
    assert!(out.status.success());
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("papyrus browser [--host <host>] [--port <port>] [--no-open]"));

    let out = bin().args(["browser", "extra"]).output().unwrap();
    assert_eq!(out.status.code(), Some(2));
}

// T-02
#[test]
fn t02_defaults() {
    let listener = TcpListener::bind("127.0.0.1:6420");
    if listener.is_err() {
        return; // default port occupied on this machine; covered by unit tests
    }
    drop(listener);
    let dir = test_root("defaults");
    init_project(&dir);
    let mut child = bin()
        .arg("--dir")
        .arg(&dir)
        .args(["browser", "--no-open", "--json"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    let _ = child.kill();
    let _ = child.wait();
    let value: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
    assert_eq!(value["status"], "serving");
    assert_eq!(value["host"], "127.0.0.1");
    assert_eq!(value["port"], 6420);
    assert_eq!(value["open"], false);
    assert_eq!(value["url"], "http://127.0.0.1:6420/");
    let _ = fs::remove_dir_all(&dir);
}

// T-03
fn startup_line(project: &Path, extra: &[&str], envs: &[(&str, &str)]) -> String {
    let mut command = bin();
    command
        .arg("--dir")
        .arg(project)
        .arg("browser")
        .args(extra)
        .envs(envs.iter().cloned())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    let mut child = command.spawn().unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    let _ = child.kill();
    let _ = child.wait();
    line.trim().to_string()
}

#[test]
fn t03_precedence_per_setting() {
    let dir = test_root("precedence");
    init_project(&dir);
    let config_port = free_port();
    run_ok(&[
        "--dir",
        dir.to_str().unwrap(),
        "config",
        "set",
        "browser_port",
        &config_port.to_string(),
    ]);
    let env_port = free_port();

    let line = startup_line(&dir, &["--json", "--no-open"], &[]);
    let value: serde_json::Value = serde_json::from_str(&line).unwrap();
    assert_eq!(value["port"], config_port, "configuration tier");

    let env_port_text = env_port.to_string();
    let line = startup_line(
        &dir,
        &["--json", "--no-open"],
        &[("PAPYRUS_BROWSER_PORT", &env_port_text)],
    );
    let value: serde_json::Value = serde_json::from_str(&line).unwrap();
    assert_eq!(
        value["port"], env_port,
        "environment overrides configuration"
    );

    let cli_port = free_port();
    let cli_port_text = cli_port.to_string();
    let line = startup_line(
        &dir,
        &["--json", "--no-open", "--port", &cli_port_text],
        &[("PAPYRUS_BROWSER_PORT", &env_port_text)],
    );
    let value: serde_json::Value = serde_json::from_str(&line).unwrap();
    assert_eq!(
        value["port"], cli_port,
        "command line overrides environment"
    );

    let line = startup_line(
        &dir,
        &["--json", "--no-open"],
        &[("PAPYRUS_BROWSER_OPEN", "false")],
    );
    let value: serde_json::Value = serde_json::from_str(&line).unwrap();
    assert_eq!(value["open"], false);
    let _ = fs::remove_dir_all(&dir);
}

// T-04
#[cfg(unix)]
#[test]
fn t04_open_and_no_open() {
    use std::os::unix::fs::PermissionsExt;
    let dir = test_root("open");
    init_project(&dir);
    let bin_dir = dir.join("fakebin");
    fs::create_dir_all(&bin_dir).unwrap();
    let marker = dir.join("opened.txt");
    let script = bin_dir.join("xdg-open");
    fs::write(
        &script,
        format!("#!/bin/sh\nprintf '%s' \"$1\" > '{}'\n", marker.display()),
    )
    .unwrap();
    let mut permissions = fs::metadata(&script).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script, permissions).unwrap();
    let path = format!(
        "{}:{}",
        bin_dir.display(),
        std::env::var("PATH").unwrap_or_default()
    );

    let server = Server::start(&dir, &[], &[("PATH", &path)]);
    let deadline = Instant::now() + Duration::from_secs(3);
    while !marker.exists() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(25));
    }
    assert!(marker.exists(), "opener should run by default");
    drop(server);
    let _ = fs::remove_file(&marker);

    let server = Server::start(&dir, &["--no-open"], &[("PATH", &path)]);
    std::thread::sleep(Duration::from_millis(300));
    assert!(!marker.exists(), "--no-open must suppress the opener");
    drop(server);

    run_ok(&[
        "--dir",
        dir.to_str().unwrap(),
        "config",
        "set",
        "browser_open",
        "false",
    ]);
    let server = Server::start(&dir, &[], &[("PATH", &path)]);
    std::thread::sleep(Duration::from_millis(300));
    assert!(
        !marker.exists(),
        "browser_open: false must suppress the opener"
    );
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-05
#[test]
fn t05_missing_opener_is_a_warning() {
    let dir = test_root("noopener");
    init_project(&dir);
    let server = Server::start(&dir, &[], &[("PATH", "")]);
    server.wait_for_stderr("could not open the default browser:");
    let response = get(&server, "/");
    assert_eq!(response.status, 200);
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-06
#[test]
fn t06_non_loopback_warning() {
    let dir = test_root("nonloopback");
    init_project(&dir);
    let server = Server::start(&dir, &["--host", "0.0.0.0", "--no-open"], &[]);
    server.wait_for_stderr("warning: binding to non-loopback address 0.0.0.0");
    let response = get(&server, "/");
    assert_eq!(response.status, 200);
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-07
#[test]
fn t07_port_in_use_exits_one() {
    let dir = test_root("portconflict");
    init_project(&dir);
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let out = bin()
        .arg("--dir")
        .arg(&dir)
        .args(["browser", "--port", &port.to_string(), "--no-open"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("cannot bind"));
    drop(listener);
    let _ = fs::remove_dir_all(&dir);
}

// T-08
#[test]
fn t08_invalid_port_and_host_exit_codes() {
    let dir = test_root("invalidopts");
    init_project(&dir);
    for port in ["0", "70000", "abc", "-1"] {
        let out = bin()
            .arg("--dir")
            .arg(&dir)
            .args(["browser", "--port", port, "--no-open"])
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(2), "port={}", port);
    }
    for host in ["bad/host", "", "bad host", "bad@host"] {
        let out = bin()
            .arg("--dir")
            .arg(&dir)
            .args(["browser", "--host", host, "--no-open"])
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(4), "host={}", host);
    }
    let _ = fs::remove_dir_all(&dir);
}

// T-09
#[test]
fn t09_list_renders_one_category() {
    let dir = fixture("list");
    let server = Server::start(&dir, &["--no-open"], &[]);

    let business = get(&server, "/");
    assert_eq!(business.status, 200);
    let html = business.text();
    assert!(html.contains("Business requirements"));
    assert!(html.contains("REQ-001"));
    assert!(html.contains("REQ-001.001"));
    assert!(html.contains("REQ-002.001"));
    assert!(!html.contains("NFR-001"));
    assert!(html.contains("aria-current=\"page\""));
    let first = html.find("REQ-001").unwrap();
    let second = html.find("REQ-002").unwrap();
    assert!(first < second);

    let nonfunctional = get(&server, "/?type=nfr");
    let html = nonfunctional.text();
    assert!(html.contains("NFR-001"));
    assert!(html.contains("NFR-001.001"));
    assert!(!html.contains("REQ-001"));
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-10
#[test]
fn t10_category_parameter() {
    let dir = fixture("category");
    let server = Server::start(&dir, &["--no-open"], &[]);
    for path in ["/", "/?type=req", "/?type=nfr"] {
        assert_eq!(get(&server, path).status, 200, "path={}", path);
    }
    for path in ["/?type=all", "/?type=x"] {
        assert_eq!(get(&server, path).status, 400, "path={}", path);
        assert_eq!(
            get(&server, &format!("/api/requirements{}", &path[1..])).status,
            400
        );
    }
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-11
#[test]
fn t11_deprecated_records_are_hidden() {
    let dir = fixture("deprecated");
    run_ok(&[
        "--dir",
        dir.to_str().unwrap(),
        "req",
        "deprecate",
        "REQ-001.001",
    ]);
    let server = Server::start(&dir, &["--no-open"], &[]);
    let list = get(&server, "/").text();
    assert!(!list.contains("REQ-001.001"));
    assert_eq!(get(&server, "/doc/REQ-001.001").status, 404);
    assert_eq!(get(&server, "/doc/REQ-001.001/edit").status, 404);
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-12
#[test]
fn t12_record_views() {
    let dir = fixture("views");
    let server = Server::start(&dir, &["--no-open"], &[]);

    let req = get(&server, "/doc/REQ-001.001").text();
    assert!(req.contains("Record ID"));
    assert!(req.contains("Requirement"));
    assert!(req.contains("Business requirement"));
    assert!(req.contains("REQ-001 Payments Group"));
    assert!(req.contains("User story"));
    assert!(req.contains("Description"));
    assert!(!req.contains("Back to requirements"));

    let group = get(&server, "/doc/REQ-001").text();
    assert!(group.contains("Group"));
    assert!(group.contains("Requirements in this group"));
    assert!(group.contains("REQ-001.001"));

    let nfr = get(&server, "/doc/NFR-001.001").text();
    assert!(nfr.contains("Non-functional requirement"));
    assert!(!nfr.contains("User story"));

    let nfr_group = get(&server, "/doc/NFR-001").text();
    assert!(nfr_group.contains("NFR-001.001"));
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-13
#[test]
fn t13_edit_round_trip() {
    let dir = fixture("roundtrip");
    let server = Server::start(&dir, &["--no-open"], &[]);
    let edit = get(&server, "/doc/REQ-001.001/edit");
    assert_eq!(edit.status, 200);
    let version = extract_version(&edit.text());
    let body = form_body(&[
        ("version", &version),
        ("title", "Capture payment revised"),
        ("story", "Revised story"),
        ("description", "Revised description"),
    ]);
    let response = post_form(&server, "/doc/REQ-001.001", &body);
    assert_eq!(response.status, 303);
    assert_eq!(response.header("Location"), Some("/doc/REQ-001.001"));
    let view = get(&server, "/doc/REQ-001.001").text();
    assert!(view.contains("Capture payment revised"));
    assert!(view.contains("Revised story"));
    assert!(view.contains("Revised description"));
    let cli = run_ok(&["--dir", dir.to_str().unwrap(), "req", "view", "REQ-001.001"]);
    assert!(cli.contains("Capture payment revised"));
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-14
#[test]
fn t14_unknown_sections_are_preserved() {
    let dir = fixture("sections");
    let path = record_path(&dir, "req", "REQ-001.001");
    let original = fs::read_to_string(&path).unwrap();
    let with_sections = format!(
        "{}\n## Criticality\nMust have\n\n## Traceability\n- REQ-001\n\n## Open Question\nWhy?\n",
        original.trim_end()
    );
    fs::write(&path, &with_sections).unwrap();
    let server = Server::start(&dir, &["--no-open"], &[]);
    let edit = get(&server, "/doc/REQ-001.001/edit").text();
    let version = extract_version(&edit);
    let body = form_body(&[
        ("version", &version),
        ("title", "New title"),
        ("story", "New story"),
        ("description", "New description"),
    ]);
    assert_eq!(post_form(&server, "/doc/REQ-001.001", &body).status, 303);
    let saved = fs::read_to_string(&path).unwrap();
    assert!(saved.contains("## Criticality\nMust have\n"));
    assert!(saved.contains("## Traceability\n- REQ-001\n"));
    assert!(saved.contains("## Open Question\nWhy?\n"));
    assert!(saved.contains("# REQ-001.001 New title"));
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-15
#[test]
fn t15_validation_returns_422_without_writing() {
    let dir = fixture("validation");
    let path = record_path(&dir, "req", "REQ-001.001");
    let before = fs::read(&path).unwrap();
    let server = Server::start(&dir, &["--no-open"], &[]);
    let version = extract_version(&get(&server, "/doc/REQ-001.001/edit").text());

    let cases = [
        ("", "Title is required"),
        (&"x".repeat(201), "Title"),
        ("bad\u{0007}title", "control"),
    ];
    for (title, needle) in cases {
        let body = form_body(&[("version", &version), ("title", title)]);
        let response = post_form(&server, "/doc/REQ-001.001", &body);
        assert_eq!(response.status, 422, "title={:?}", title);
        assert!(response.text().contains(needle));
        assert_eq!(fs::read(&path).unwrap(), before);
    }
    let body = form_body(&[
        ("version", &version),
        ("title", "Valid"),
        ("description", "Line\n## Not allowed\nMore"),
    ]);
    let response = post_form(&server, "/doc/REQ-001.001", &body);
    assert_eq!(response.status, 422);
    assert!(response.text().contains("## "));
    assert_eq!(fs::read(&path).unwrap(), before);

    let body = form_body(&[
        ("version", &version),
        ("title", ""),
        ("description", "Line\n## Not allowed\nMore"),
    ]);
    let response = post_form(&server, "/doc/REQ-001.001", &body);
    assert_eq!(response.status, 422);
    let html = response.text();
    assert!(html.contains("Title is required"));
    assert!(html.contains("## "));
    assert_eq!(fs::read(&path).unwrap(), before);
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-16
#[test]
fn t16_immutable_fields_are_ignored() {
    let dir = fixture("immutable");
    let path = record_path(&dir, "req", "REQ-001.001");
    let server = Server::start(&dir, &["--no-open"], &[]);
    let version = extract_version(&get(&server, "/doc/REQ-001.001/edit").text());
    let body = form_body(&[
        ("version", &version),
        ("title", "Renamed"),
        ("id", "REQ-999.999"),
        ("group", "REQ-999"),
        ("form", "group"),
    ]);
    assert_eq!(post_form(&server, "/doc/REQ-001.001", &body).status, 303);
    let saved = fs::read_to_string(&path).unwrap();
    assert!(saved.starts_with("# REQ-001.001 Renamed"));
    assert!(saved.contains("## Group\nREQ-001 Payments Group"));
    assert!(!path.with_file_name("REQ-999.999.md").exists());
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-17
#[test]
fn t17_stale_version_is_conflict() {
    let dir = fixture("conflict");
    let path = record_path(&dir, "req", "REQ-001.001");
    let server = Server::start(&dir, &["--no-open"], &[]);
    let version = extract_version(&get(&server, "/doc/REQ-001.001/edit").text());
    fs::write(
        &path,
        format!(
            "{}\n## Out of band\nchanged\n",
            fs::read_to_string(&path).unwrap()
        ),
    )
    .unwrap();
    let body = form_body(&[("version", &version), ("title", "Should not save")]);
    let response = post_form(&server, "/doc/REQ-001.001", &body);
    assert_eq!(response.status, 409);
    assert!(response.text().contains("This record changed on disk"));
    let saved = fs::read_to_string(&path).unwrap();
    assert!(!saved.contains("Should not save"));
    assert!(saved.contains("## Out of band"));
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-18
#[test]
fn t18_writes_are_atomic_and_leave_no_temporary_file() {
    let dir = fixture("atomic");
    let path = record_path(&dir, "req", "REQ-001.001");
    let server = Server::start(&dir, &["--no-open"], &[]);
    let version = extract_version(&get(&server, "/doc/REQ-001.001/edit").text());
    let before: Vec<String> = fs::read_dir(path.parent().unwrap())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    let body = form_body(&[("version", &version), ("title", "Atomic")]);
    assert_eq!(post_form(&server, "/doc/REQ-001.001", &body).status, 303);
    let after: Vec<String> = fs::read_dir(path.parent().unwrap())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    assert_eq!(before, after);
    assert!(!after.iter().any(|name| name.contains(".tmp")));
    // A rejected save leaves no temporary file either.
    let body = form_body(&[("version", "1.2.3"), ("title", "Rejected")]);
    assert_eq!(post_form(&server, "/doc/REQ-001.001", &body).status, 409);
    let after: Vec<String> = fs::read_dir(path.parent().unwrap())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    assert_eq!(before, after);
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-19
#[cfg(unix)]
#[test]
fn t19_symlink_escape_is_404() {
    use std::os::unix::fs::symlink;
    let dir = fixture("symlink");
    let outside = test_root("symlink-outside");
    let secret = outside.join("secret.md");
    fs::write(&secret, "# REQ-001.001 SECRET CONTENT\n").unwrap();
    let path = record_path(&dir, "req", "REQ-001.001");
    fs::remove_file(&path).unwrap();
    symlink(&secret, &path).unwrap();
    let server = Server::start(&dir, &["--no-open"], &[]);
    let response = get(&server, "/doc/REQ-001.001");
    assert_eq!(response.status, 404);
    assert!(!response.text().contains("SECRET CONTENT"));
    let list = get(&server, "/").text();
    assert!(!list.contains("REQ-001.001"));
    let version = "1.2.3";
    let body = form_body(&[("version", version), ("title", "Attack")]);
    assert_eq!(post_form(&server, "/doc/REQ-001.001", &body).status, 404);
    assert!(
        fs::read_to_string(&secret)
            .unwrap()
            .contains("SECRET CONTENT")
    );
    drop(server);
    let _ = fs::remove_dir_all(&dir);
    let _ = fs::remove_dir_all(&outside);
}

// T-20
#[test]
fn t20_html_escaping_prevents_script_injection() {
    let dir = test_root("xss");
    init_project(&dir);
    create_req_group(&dir, "Group");
    let payload = "\"><script>alert(1)</script>";
    let path = record_path(&dir, "req", "REQ-001.001");
    fs::write(
        &path,
        format!(
            "# REQ-001.001 {}\n\n## Group\nREQ-001 Group\n\n## Description\n{}\n",
            payload, payload
        ),
    )
    .unwrap();
    let server = Server::start(&dir, &["--no-open"], &[]);
    for path in ["/", "/doc/REQ-001.001", "/doc/REQ-001.001/edit"] {
        let html = get(&server, path).text();
        assert!(!html.contains("<script>alert(1)</script>"), "path={}", path);
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-21
#[test]
fn t21_json_api() {
    let dir = fixture("json");
    let server = Server::start(&dir, &["--no-open"], &[]);
    let list: serde_json::Value =
        serde_json::from_str(&get(&server, "/api/requirements").text()).unwrap();
    assert_eq!(list["groups"][0]["kind"], "req");
    assert_eq!(list["groups"][0]["form"], "group");
    assert!(list["requirements"][0]["group"].is_string());

    let doc: serde_json::Value =
        serde_json::from_str(&get(&server, "/api/doc/REQ-001.001").text()).unwrap();
    assert_eq!(doc["id"], "REQ-001.001");
    assert_eq!(doc["kind"], "req");
    assert_eq!(doc["form"], "requirement");
    assert_eq!(doc["group"]["id"], "REQ-001");
    assert!(doc["path"].as_str().unwrap().starts_with("docs/"));
    let version = doc["version"].as_str().unwrap().to_string();

    let body = format!(
        "{{\"version\":\"{}\",\"title\":\"API title\",\"description\":\"API description\"}}",
        version
    );
    let updated: serde_json::Value =
        serde_json::from_str(&post_json(&server, "/api/doc/REQ-001.001", &body).text()).unwrap();
    assert_eq!(updated["title"], "API title");
    assert_eq!(updated["description"], "API description");
    assert!(updated["story"].is_null() || updated["story"].is_string());

    assert_eq!(
        post_json(&server, "/api/doc/REQ-001.001", "{not json").status,
        400
    );
    let version = updated["version"].as_str().unwrap().to_string();
    assert_eq!(
        post_json(
            &server,
            "/api/doc/REQ-001.001",
            &format!("{{\"version\":\"{}\",\"title\":5}}", version)
        )
        .status,
        400
    );
    assert_eq!(
        post_json(
            &server,
            "/api/doc/REQ-001.001",
            &format!("{{\"version\":\"{}\",\"title\":\"x\"}}", "1.2.3")
        )
        .status,
        409
    );
    assert_eq!(
        request(
            server.port,
            "POST",
            "/api/doc/REQ-001.001",
            &[("Content-Type", "text/plain")],
            Some(b"x"),
        )
        .status,
        415
    );
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-22
#[test]
fn t22_request_limits_and_status_codes() {
    let dir = fixture("limits");
    let server = Server::start(&dir, &["--no-open"], &[]);
    let port = server.port;

    assert_eq!(
        request(port, "PUT", "/doc/REQ-001.001", &[], None).status,
        405
    );
    assert_eq!(request(port, "POST", "/", &[], Some(b"")).status, 405);
    assert_eq!(get(&server, "/nope").status, 404);
    assert_eq!(get(&server, "/doc/%2F").status, 400);
    assert_eq!(
        send_raw(
            port,
            b"GET / HTTP/1.1\r\nHost: 127.0.0.1\r\nHost: 127.0.0.1\r\n\r\n"
        )
        .status,
        400
    );
    assert_eq!(
        send_raw(
            port,
            format!("GET / HTTP/1.1\r\nHost: 127.0.0.1:{}\r\n\r\n", port).as_bytes()
        )
        .status,
        200
    );
    assert_eq!(
        send_raw(port, b"GET / HTTP/2.0\r\nHost: 127.0.0.1\r\n\r\n").status,
        505
    );
    assert_eq!(
        request(
            port,
            "POST",
            "/doc/REQ-001.001",
            &[("Content-Type", "text/plain")],
            Some(b"x")
        )
        .status,
        415
    );
    assert_eq!(
        send_raw(
            port,
            format!(
                "POST /doc/REQ-001.001 HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nTransfer-Encoding: chunked\r\n\r\n",
                port
            )
            .as_bytes()
        )
        .status,
        501
    );
    assert_eq!(
        send_raw(
            port,
            format!(
                "POST /doc/REQ-001.001 HTTP/1.1\r\nHost: 127.0.0.1:{}\r\n\r\n",
                port
            )
            .as_bytes()
        )
        .status,
        411
    );
    assert_eq!(
        send_raw(
            port,
            format!(
                "POST /doc/REQ-001.001 HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Length: 1048577\r\n\r\n",
                port
            )
            .as_bytes()
        )
        .status,
        413
    );
    let long_target = format!("/{}", "a".repeat(3000));
    assert_eq!(
        send_raw(
            port,
            format!(
                "GET {} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\n\r\n",
                long_target, port
            )
            .as_bytes()
        )
        .status,
        414
    );
    let mut many_headers = format!("GET / HTTP/1.1\r\nHost: 127.0.0.1:{}\r\n", port);
    for index in 0..65 {
        many_headers.push_str(&format!("X-Test-{}: value\r\n", index));
    }
    many_headers.push_str("\r\n");
    assert_eq!(send_raw(port, many_headers.as_bytes()).status, 431);
    let body = form_body(&[("version", "1.2.3"), ("title", "x")]);
    assert_eq!(
        request(
            port,
            "POST",
            "/doc/REQ-001.001",
            &[
                ("Content-Type", "application/x-www-form-urlencoded"),
                ("Origin", "http://evil.test"),
            ],
            Some(body.as_bytes()),
        )
        .status,
        403
    );

    // Read timeout: a partial request is answered with 408 after five seconds.
    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();
    stream.write_all(b"GET / HT").unwrap();
    std::thread::sleep(Duration::from_secs(6));
    let mut bytes = Vec::new();
    stream.read_to_end(&mut bytes).unwrap();
    assert_eq!(parse_response(&bytes).status, 408);

    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-23
#[test]
fn t23_one_request_per_connection() {
    let dir = fixture("oneconn");
    let server = Server::start(&dir, &["--no-open"], &[]);
    let mut stream = TcpStream::connect(("127.0.0.1", server.port)).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();
    let first = format!("GET / HTTP/1.1\r\nHost: 127.0.0.1:{}\r\n\r\n", server.port);
    stream.write_all(first.as_bytes()).unwrap();
    stream.write_all(first.as_bytes()).ok();
    let mut bytes = Vec::new();
    stream.read_to_end(&mut bytes).unwrap();
    let text = String::from_utf8_lossy(&bytes);
    assert_eq!(text.matches("HTTP/1.1 200 OK").count(), 1);
    assert!(text.contains("Connection: close"));
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-24 and T-27
#[test]
fn t24_assets_are_offline_and_embedded() {
    let dir = fixture("assets");
    let server = Server::start(&dir, &["--no-open"], &[]);
    let css = get(&server, "/assets/app.css");
    assert_eq!(css.status, 200);
    assert_eq!(css.header("Content-Type"), Some("text/css; charset=utf-8"));
    let js = get(&server, "/assets/app.js");
    assert_eq!(js.status, 200);
    assert_eq!(
        js.header("Content-Type"),
        Some("text/javascript; charset=utf-8")
    );
    assert_eq!(css.text(), include_str!("../src/browser/app.css"));
    assert_eq!(js.text(), include_str!("../src/browser/app.js"));

    for path in ["/", "/doc/REQ-001.001", "/doc/REQ-001.001/edit"] {
        let html = get(&server, path).text();
        assert!(!html.contains("http://"));
        assert!(!html.contains("https://"));
        assert!(!html.contains("//cdn"));
        assert!(!html.contains("@import"));
    }
    for asset in [css.text(), js.text()] {
        assert!(!asset.contains("@import"));
        assert!(!asset.contains("http://"));
        assert!(!asset.contains("https://"));
        assert!(!asset.contains("url("));
    }
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-25
#[test]
fn t25_non_utf8_record_is_skipped_and_not_rewritten() {
    let dir = fixture("nonutf8");
    let path = record_path(&dir, "req", "REQ-009");
    fs::write(&path, [0xff, 0xfe, 0x00, 0x01]).unwrap();
    let before = fs::read(&path).unwrap();
    let server = Server::start(&dir, &["--no-open"], &[]);
    let list = get(&server, "/").text();
    assert!(!list.contains("REQ-009"));
    assert_eq!(get(&server, "/doc/REQ-009").status, 500);
    assert_eq!(fs::read(&path).unwrap(), before);
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-26
#[test]
fn t26_config_keys() {
    let dir = test_root("config");
    init_project(&dir);
    let list = run_ok(&["--dir", dir.to_str().unwrap(), "config", "list"]);
    assert!(list.contains("browser_host: 127.0.0.1"));
    assert!(list.contains("browser_port: 6420"));
    assert!(list.contains("browser_open: true"));
    assert_eq!(
        run_ok(&[
            "--dir",
            dir.to_str().unwrap(),
            "config",
            "get",
            "browser_host"
        ]),
        "127.0.0.1"
    );
    run_ok(&[
        "--dir",
        dir.to_str().unwrap(),
        "config",
        "set",
        "browser_port",
        "7001",
    ]);
    assert_eq!(
        run_ok(&[
            "--dir",
            dir.to_str().unwrap(),
            "config",
            "get",
            "browser_port"
        ]),
        "7001"
    );
    let out = bin()
        .arg("--dir")
        .arg(&dir)
        .args(["config", "set", "browser_port", "0"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));
    let out = bin()
        .arg("--dir")
        .arg(&dir)
        .args(["config", "set", "browser_open", "maybe"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));
    let out = bin()
        .arg("--dir")
        .arg(&dir)
        .args(["config", "set", "browser_host", "bad/host"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));
    let json = run_ok(&["--dir", dir.to_str().unwrap(), "--json", "config", "list"]);
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["browser_host"], "127.0.0.1");
    assert_eq!(value["browser_port"], 7001);
    assert_eq!(value["browser_open"], true);
    run_ok(&[
        "--dir",
        dir.to_str().unwrap(),
        "config",
        "unset",
        "browser_port",
    ]);
    assert_eq!(
        run_ok(&[
            "--dir",
            dir.to_str().unwrap(),
            "config",
            "get",
            "browser_port"
        ]),
        "6420"
    );
    let _ = fs::remove_dir_all(&dir);
}

// T-28
#[test]
fn t28_accessibility_and_compatibility_basics() {
    let dir = fixture("a11y");
    let server = Server::start(&dir, &["--no-open"], &[]);
    let list = get(&server, "/").text();
    assert_eq!(list.matches("<h1").count(), 1);
    for landmark in ["<header", "<nav", "<main", "<footer"] {
        assert!(list.contains(landmark), "missing {}", landmark);
    }
    assert!(list.contains("aria-current=\"page\""));
    assert!(list.contains("aria-controls=\"type-list\""));
    assert!(list.contains("aria-expanded="));
    assert!(list.contains("sidebar-toggle"));

    let edit = get(&server, "/doc/REQ-001.001/edit").text();
    assert!(edit.contains("<label for=\"story\">"));
    assert!(edit.contains("<label for=\"description\">"));
    assert!(edit.contains("required"));

    let css = include_str!("../src/browser/app.css");
    let js = include_str!("../src/browser/app.js");
    for forbidden in [
        ":has(",
        "color-mix(",
        "@layer",
        "container-type",
        "subgrid",
        "dialog",
        "popover",
        "inert",
    ] {
        assert!(
            !css.contains(forbidden),
            "forbidden CSS feature {}",
            forbidden
        );
    }
    for forbidden in ["fetch(", "import ", "?.", "??", "=>"] {
        assert!(
            !js.contains(forbidden),
            "forbidden JS feature {}",
            forbidden
        );
    }
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-30
#[test]
fn t30_no_writes_outside_project() {
    let dir = fixture("boundary");
    let outside = test_root("boundary-outside");
    let marker = outside.join("marker.txt");
    fs::write(&marker, "untouched").unwrap();
    let server = Server::start(&dir, &["--no-open"], &[]);
    let version = extract_version(&get(&server, "/doc/REQ-001.001/edit").text());
    let body = form_body(&[
        ("version", &version),
        ("title", "Boundary"),
        ("description", "Only inside"),
    ]);
    assert_eq!(post_form(&server, "/doc/REQ-001.001", &body).status, 303);
    assert_eq!(fs::read_to_string(&marker).unwrap(), "untouched");
    assert!(
        fs::read_to_string(record_path(&dir, "req", "REQ-001.001"))
            .unwrap()
            .contains("Boundary")
    );
    drop(server);
    let _ = fs::remove_dir_all(&dir);
    let _ = fs::remove_dir_all(&outside);
}
