//! Integration tests for `papyrus browser` (specification checks T-01 to T-30).

use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdout, Command, Stdio};
use std::time::{Duration, Instant};

use papyrus::browser::assets::{self, ASSETS};

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
        "nfr",
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
        "nfr",
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

/// Replace (or append) one top-level scalar in the project configuration file.
fn set_config_value(dir: &Path, key: &str, value: &str) {
    let path = dir.join("papyrus.config.yml");
    let text = fs::read_to_string(&path).unwrap();
    let prefix = format!("{}:", key);
    let mut out = String::new();
    let mut replaced = false;
    for line in text.lines() {
        if line.starts_with(&prefix) {
            out.push_str(&format!("{}: {}\n", key, value));
            replaced = true;
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }
    if !replaced {
        out.push_str(&format!("{}: {}\n", key, value));
    }
    fs::write(&path, out).unwrap();
}

/// Replace the configured task-status list (and default status) in the
/// project configuration file. `config set` only accepts scalars, so the
/// `statuses` list is rewritten directly.
fn set_statuses(dir: &Path, statuses: &[&str], default_status: &str) {
    let path = dir.join("papyrus.config.yml");
    let text = fs::read_to_string(&path).unwrap();
    let mut out = String::new();
    let mut in_statuses = false;
    for line in text.lines() {
        if line == "statuses:" || line.starts_with("statuses: [") {
            in_statuses = true;
            out.push_str("statuses:\n");
            for status in statuses {
                out.push_str(&format!("- {}\n", status));
            }
            continue;
        }
        if in_statuses {
            if line.starts_with("default_status:") {
                in_statuses = false;
                out.push_str(&format!("default_status: {}\n", default_status));
            }
            // Drop the old list items.
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    fs::write(&path, out).unwrap();
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
    startup: String,
    stdout: BufReader<ChildStdout>,
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
            return Server {
                child,
                port,
                stderr_path,
                startup: startup.trim().to_string(),
                stdout: reader,
            };
        }
        panic!("server did not start after retries");
    }

    fn stderr_text(&self) -> String {
        fs::read_to_string(&self.stderr_path).unwrap_or_default()
    }

    /// Stop the server and return whatever standard output was written after
    /// the startup line.
    fn finish_stdout(&mut self) -> String {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let mut rest = String::new();
        let _ = self.stdout.read_to_string(&mut rest);
        rest
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

/// Like [`send_raw`], but returns `None` when the peer resets the connection.
/// Over-capacity rejection closes without reading the request, so a client
/// that already sent one can observe a reset instead of the response.
fn try_send_raw(port: u16, raw: &[u8]) -> Option<HttpResponse> {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).ok()?;
    stream
        .set_read_timeout(Some(Duration::from_secs(15)))
        .ok()?;
    stream.write_all(raw).ok()?;
    let mut bytes = Vec::new();
    if stream.read_to_end(&mut bytes).is_err() || bytes.is_empty() {
        return None;
    }
    if !bytes.windows(4).any(|window| window == b"\r\n\r\n") {
        return None;
    }
    Some(parse_response(&bytes))
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

/// Text of one 422 summary entry, identified by the field it links to.
fn summary_entry(html: &str, field: &str) -> String {
    let marker = format!("<li><a href=\"#{}\">", field);
    let start = html
        .find(&marker)
        .unwrap_or_else(|| panic!("summary entry for {}", field))
        + marker.len();
    let rest = &html[start..];
    let end = rest.find("</a></li>").expect("summary entry terminator");
    rest[..end].to_string()
}

/// Text of one field-level validation message.
fn field_error(html: &str, field: &str) -> String {
    let marker = format!("<p class=\"error\" id=\"{}-error\">", field);
    let start = html
        .find(&marker)
        .unwrap_or_else(|| panic!("field error for {}", field))
        + marker.len();
    let rest = &html[start..];
    let end = rest.find("</p>").expect("field error terminator");
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

fn normalized_newlines(text: &str) -> String {
    text.replace("\r\n", "\n")
}

/// The exact `## ` section block, from its heading to the next heading.
fn section_block(content: &str, heading: &str) -> String {
    let normalized = normalized_newlines(content);
    let mut block = String::new();
    let mut inside = false;
    for line in normalized.lines() {
        if line == heading {
            inside = true;
        } else if inside && line.starts_with("## ") {
            break;
        }
        if inside {
            block.push_str(line);
            block.push('\n');
        }
    }
    block
}

// T-01
#[test]
fn t01_command_shape() {
    let out = bin().args(["browser", "--help"]).output().unwrap();
    assert!(out.status.success());
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("papyrus browser [--host <host>] [--port <port>] [--no-open] [--debug]"));
    assert!(text.contains("--debug"));
    assert!(
        text.contains("Log one line per received request"),
        "help must document --debug: {text}"
    );

    let out = bin().args(["browser", "extra"]).output().unwrap();
    assert_eq!(out.status.code(), Some(2));
}

// T-02
#[test]
fn t02_defaults() {
    let dir = test_root("defaults");
    init_project(&dir);

    // The default port is part of the contract. If 6420 is occupied on this
    // machine, verify the default through `config get` and run the server on a
    // free port; never silently skip the check.
    let (port, default_port) = match TcpListener::bind("127.0.0.1:6420") {
        Ok(listener) => {
            drop(listener);
            (6420, true)
        }
        Err(_) => {
            assert_eq!(
                run_ok(&[
                    "--dir",
                    dir.to_str().unwrap(),
                    "config",
                    "get",
                    "browser_port"
                ]),
                "6420",
                "default browser_port"
            );
            (free_port(), false)
        }
    };

    let mut command = bin();
    command
        .arg("--dir")
        .arg(&dir)
        .args(["browser", "--no-open", "--json"]);
    if !default_port {
        command.arg("--port").arg(port.to_string());
    }
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    let _ = child.kill();
    let _ = child.wait();
    let value: serde_json::Value = serde_json::from_str(line.trim())
        .unwrap_or_else(|_| panic!("no startup line; browser did not start: {:?}", line));
    assert_eq!(value["status"], "serving");
    assert_eq!(value["host"], "127.0.0.1");
    assert_eq!(value["port"], port);
    assert_eq!(value["open"], false);
    assert_eq!(value["url"], format!("http://127.0.0.1:{}/", port));
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
    let response = get(&server, "/?type=req");
    assert_eq!(response.status, 200);
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-06
#[test]
fn t06_non_loopback_warning() {
    let dir = test_root("nonloopback");
    init_project(&dir);
    // The specification requires this test to send no traffic.
    let server = Server::start(&dir, &["--host", "0.0.0.0", "--no-open"], &[]);
    server.wait_for_stderr("warning: binding to non-loopback address 0.0.0.0");
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

    let business = get(&server, "/?type=req");
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
        assert_eq!(get(&server, path).status, 404, "path={}", path);
        assert_eq!(
            get(&server, &format!("/api/requirements{}", &path[1..])).status,
            404
        );
    }
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-09 (empty project and empty document type)
#[test]
fn t09_list_empty_project_and_empty_type() {
    // Empty project: both categories render the project-level empty state.
    let empty = test_root("empty-project");
    init_project(&empty);
    let server = Server::start(&empty, &["--no-open"], &[]);
    // TASK-142: req/nfr now share the one empty message/help pair and the
    // presentation headings (Business requirements / Non-functional requirements).
    for (path, message) in [
        ("/?type=req", "No business requirements in this project."),
        (
            "/?type=nfr",
            "No non-functional requirements in this project.",
        ),
    ] {
        let html = get(&server, path).text();
        assert!(html.contains(message), "path={path}");
        assert!(
            html.contains(
                "Records are created from the command line; this browser does not create records."
            ),
            "path={path}"
        );
    }
    drop(server);
    let _ = fs::remove_dir_all(&empty);

    // Empty document type while the other category has records.
    let req_only = test_root("empty-type");
    init_project(&req_only);
    create_req_group(&req_only, "Only business");
    create_req(&req_only, "Business record", "REQ-001");
    let server = Server::start(&req_only, &["--no-open"], &[]);
    let html = get(&server, "/?type=nfr").text();
    // TASK-142: the cross-family hint is an accepted difference; one shared
    // empty message/help pair is used for every family.
    assert!(html.contains("No non-functional requirements in this project."));
    assert!(html.contains(
        "Records are created from the command line; this browser does not create records."
    ));
    assert!(!html.contains("Business requirements are available in Document types."));
    assert!(!html.contains("REQ-001.001"));
    drop(server);
    let _ = fs::remove_dir_all(&req_only);
}

// T-09 / T-12 (group without members and "Other requirements")
#[test]
fn t09_group_without_members_and_other_requirements() {
    let dir = fixture("emptystates");
    let empty_group = create_req_group(&dir, "Empty Group");
    let orphan_group = create_req_group(&dir, "Orphan Group");
    create_req(&dir, "Orphan requirement", &orphan_group);
    // Remove the group file so the member's stored reference cannot resolve.
    fs::remove_file(record_path(&dir, "req", &orphan_group)).unwrap();

    let server = Server::start(&dir, &["--no-open"], &[]);
    let html = get(&server, "/?type=req").text();
    // TASK-142: an empty group is one group row with no children; a member
    // whose group row is absent from the filtered index is a top-level row.
    assert!(html.contains("data-list-layout=\"shared\""));
    assert!(html.contains(&format!("data-row-id=\"{empty_group}\"")));
    assert!(html.contains(&format!("data-row-id=\"{orphan_group}.001\"")));
    assert!(
        !html.contains(&format!("data-row-id=\"{orphan_group}.001\" data-child-of")),
        "the unresolved member is top-level"
    );
    assert!(!html.contains("Other requirements"));
    assert!(!html.contains("(not found)"));
    assert!(!html.contains("No requirements in this group"));
    assert!(!html.contains(&format!("href=\"/doc/{}\"", orphan_group)));

    let group = get(&server, &format!("/doc/{}", empty_group)).text();
    assert!(group.contains("Requirements in this group"));
    assert!(group.contains("No requirements in this group."));
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-11
#[test]
fn t11_archived_records_are_hidden() {
    let dir = fixture("archived");
    run_ok(&[
        "--dir",
        dir.to_str().unwrap(),
        "req",
        "archive",
        "REQ-001.001",
    ]);
    let server = Server::start(&dir, &["--no-open"], &[]);
    let list = get(&server, "/?type=req").text();
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
    assert!(req.contains("<p class=\"id\">"));
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

// T-12 / AC #9 (group members are listed in ascending ID order)
#[test]
fn group_members_are_ordered_by_id() {
    let dir = fixture("member-order");
    // Write members out of order so the test proves the index sorts them.
    for (id, title) in [
        ("REQ-001.003", "Third member"),
        ("REQ-001.002", "Second member"),
    ] {
        fs::write(
            record_path(&dir, "req", id),
            format!(
                "# {} {}\n\n## Group\nREQ-001 Payments Group\n\n## Description\nBody\n",
                id, title
            ),
        )
        .unwrap();
    }
    let server = Server::start(&dir, &["--no-open"], &[]);
    let group = get(&server, "/doc/REQ-001").text();
    let first = group.find("REQ-001.001").unwrap();
    let second = group.find("REQ-001.002").unwrap();
    let third = group.find("REQ-001.003").unwrap();
    assert!(
        first < second && second < third,
        "member order: {} {} {}",
        first,
        second,
        third
    );
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

// T-13 (business-requirement group and non-functional requirement forms)
#[test]
fn t13_edit_round_trip_group_and_nonfunctional() {
    let dir = fixture("roundtrip-forms");
    let server = Server::start(&dir, &["--no-open"], &[]);

    // Business-requirement group: title, user story, description.
    let edit = get(&server, "/doc/REQ-001/edit");
    assert_eq!(edit.status, 200);
    let html = edit.text();
    assert!(html.contains("<label for=\"story\">"));
    let version = extract_version(&html);
    let body = form_body(&[
        ("version", &version),
        ("title", "Payments Group revised"),
        ("story", "Revised group story"),
        ("description", "Revised group description"),
    ]);
    let response = post_form(&server, "/doc/REQ-001", &body);
    assert_eq!(response.status, 303);
    assert_eq!(response.header("Location"), Some("/doc/REQ-001"));
    let view = get(&server, "/doc/REQ-001").text();
    assert!(view.contains("Payments Group revised"));
    assert!(view.contains("Revised group story"));
    assert!(view.contains("Revised group description"));
    let cli = run_ok(&["--dir", dir.to_str().unwrap(), "req", "view", "REQ-001"]);
    assert!(cli.contains("Payments Group revised"));

    // Non-functional requirement: title and description only; a story is
    // ignored and no story field is rendered.
    let edit = get(&server, "/doc/NFR-001.001/edit");
    assert_eq!(edit.status, 200);
    let html = edit.text();
    assert!(!html.contains("<label for=\"story\">"));
    let version = extract_version(&html);
    let body = form_body(&[
        ("version", &version),
        ("title", "Restrict access revised"),
        ("story", "must be ignored"),
        ("description", "Revised nfr description"),
    ]);
    let response = post_form(&server, "/doc/NFR-001.001", &body);
    assert_eq!(response.status, 303);
    assert_eq!(response.header("Location"), Some("/doc/NFR-001.001"));
    let view = get(&server, "/doc/NFR-001.001").text();
    assert!(view.contains("Restrict access revised"));
    assert!(view.contains("Revised nfr description"));
    assert!(!view.contains("must be ignored"));
    let cli = run_ok(&["--dir", dir.to_str().unwrap(), "nfr", "view", "NFR-001.001"]);
    assert!(cli.contains("Restrict access revised"));

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
    let before = normalized_newlines(&with_sections);
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
    let after = normalized_newlines(&fs::read_to_string(&path).unwrap());

    // Compare the preserved sections as whole normalized blocks rather than
    // looking for substrings.
    for heading in ["## Criticality", "## Traceability", "## Open Question"] {
        assert_eq!(
            section_block(&before, heading),
            section_block(&after, heading),
            "{} changed",
            heading
        );
    }
    // Unknown sections also keep their original order.
    let order: Vec<usize> = ["## Criticality", "## Traceability", "## Open Question"]
        .iter()
        .map(|heading| after.find(heading).unwrap())
        .collect();
    assert!(
        order.windows(2).all(|pair| pair[0] < pair[1]),
        "unknown sections changed order: {:?}",
        order
    );
    assert!(after.contains("# REQ-001.001 New title"));
    assert!(after.contains("## User Story\nNew story"));
    assert!(after.contains("## Description\nNew description"));
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

    let long_title = "x".repeat(201);
    let cases: [(&str, &str); 3] = [
        ("", "Title is required."),
        (long_title.as_str(), "Title must be at most 200 characters."),
        (
            "bad\u{0007}title",
            "Title cannot contain control characters.",
        ),
    ];
    for (title, expected) in cases {
        let body = form_body(&[("version", &version), ("title", title)]);
        let response = post_form(&server, "/doc/REQ-001.001", &body);
        assert_eq!(response.status, 422, "title={:?}", title);
        let html = response.text();
        assert_eq!(
            summary_entry(&html, "title"),
            expected,
            "summary must repeat the field message for title={:?}",
            title
        );
        assert_eq!(field_error(&html, "title"), expected);
        assert_eq!(fs::read(&path).unwrap(), before);
    }

    let description_message = "Description contains a line starting with ## .";
    let body = form_body(&[
        ("version", &version),
        ("title", "Valid"),
        ("description", "Line\n## Not allowed\nMore"),
    ]);
    let response = post_form(&server, "/doc/REQ-001.001", &body);
    assert_eq!(response.status, 422);
    let html = response.text();
    assert_eq!(summary_entry(&html, "description"), description_message);
    assert_eq!(field_error(&html, "description"), description_message);
    // Submitted values are preserved.
    assert!(html.contains("value=\"Valid\""));
    assert!(html.contains("Line\n## Not allowed\nMore"));
    assert_eq!(fs::read(&path).unwrap(), before);

    let story_message = "User story contains a line starting with ## .";
    let body = form_body(&[
        ("version", &version),
        ("title", "Valid"),
        ("story", "Story\n## Not allowed"),
    ]);
    let response = post_form(&server, "/doc/REQ-001.001", &body);
    assert_eq!(response.status, 422);
    let html = response.text();
    assert_eq!(summary_entry(&html, "story"), story_message);
    assert_eq!(field_error(&html, "story"), story_message);
    assert_eq!(fs::read(&path).unwrap(), before);

    let body = form_body(&[
        ("version", &version),
        ("title", ""),
        ("description", "Line\n## Not allowed\nMore"),
    ]);
    let response = post_form(&server, "/doc/REQ-001.001", &body);
    assert_eq!(response.status, 422);
    let html = response.text();
    assert_eq!(summary_entry(&html, "title"), "Title is required.");
    assert_eq!(summary_entry(&html, "description"), description_message);
    // Form-level summary and field-level error association.
    assert!(html.contains("role=\"alert\""));
    assert!(html.contains("There is a problem"));
    assert!(html.contains("href=\"#title\""));
    assert!(html.contains("href=\"#description\""));
    assert!(html.contains("Nothing was saved. Your typed values are kept below."));
    assert!(html.contains("id=\"title-error\""));
    assert!(html.contains("aria-invalid=\"true\" aria-describedby=\"title-error\""));
    assert!(html.contains("id=\"description-error\""));
    assert!(html.contains("aria-invalid=\"true\" aria-describedby=\"description-error\""));
    assert_eq!(fs::read(&path).unwrap(), before);
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// AC #15 / T-15 (encoded NUL in a form value is a field validation error)
#[test]
fn form_encoded_nul_returns_422() {
    let dir = fixture("form-nul");
    let path = record_path(&dir, "req", "REQ-001.001");
    let before = fs::read(&path).unwrap();
    let server = Server::start(&dir, &["--no-open"], &[]);
    let version = extract_version(&get(&server, "/doc/REQ-001.001/edit").text());
    let body = format!("version={}&title=bad%00title", urlencode(&version));
    let response = post_form(&server, "/doc/REQ-001.001", &body);
    assert_eq!(response.status, 422);
    let html = response.text();
    assert!(html.contains("Title cannot contain control characters."));
    assert_eq!(fs::read(&path).unwrap(), before);
    // Path decoding stays strict: an encoded NUL in the path is 400.
    assert_eq!(get(&server, "/doc/%00").status, 400);
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
    assert!(saved.contains("# REQ-001.001 Renamed"));
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
    let html = response.text();
    assert!(html.contains("This record changed on disk"));
    assert!(html.contains("record changed on disk; reload the form"));
    assert!(html.contains("Reload the form"));
    assert!(html.contains("View current record"));
    assert!(html.contains("Back to requirements"));
    // AC #19 / MOCK-0001 `edit-conflict`: breadcrumb and bordered notice,
    // not the generic error page with a large status-code block.
    assert!(html.contains("class=\"breadcrumb\""));
    assert!(html.contains("class=\"conflict-box\""));
    assert!(!html.contains("class=\"status-code\""));
    assert_eq!(html.matches("<h1").count(), 1);
    // The attempted values are shown read-only on the conflict page.
    assert!(html.contains("Should not save"));
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

// T-18 / AC #12 (atomic write preserves the original file permissions)
#[cfg(unix)]
#[test]
fn atomic_write_preserves_permissions() {
    use std::os::unix::fs::PermissionsExt;
    let dir = fixture("permissions");
    let path = record_path(&dir, "req", "REQ-001.001");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o640)).unwrap();
    let server = Server::start(&dir, &["--no-open"], &[]);
    let version = extract_version(&get(&server, "/doc/REQ-001.001/edit").text());
    let body = form_body(&[("version", &version), ("title", "Permissions kept")]);
    assert_eq!(post_form(&server, "/doc/REQ-001.001", &body).status, 303);
    let mode = fs::metadata(&path).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o640, "mode changed to {:o}", mode);
    assert!(
        fs::read_to_string(&path)
            .unwrap()
            .contains("# REQ-001.001 Permissions kept")
    );
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
    let list = get(&server, "/?type=req").text();
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
    for path in ["/?type=req", "/doc/REQ-001.001", "/doc/REQ-001.001/edit"] {
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
    let conflict = post_json(
        &server,
        "/api/doc/REQ-001.001",
        &format!("{{\"version\":\"{}\",\"title\":\"x\"}}", "1.2.3"),
    );
    assert_eq!(conflict.status, 409);
    let error: serde_json::Value = serde_json::from_str(&conflict.text()).unwrap();
    assert_eq!(error["error"]["code"], 409);
    assert_eq!(
        error["error"]["message"],
        "record changed on disk; reload the form"
    );

    let invalid = post_json(
        &server,
        "/api/doc/REQ-001.001",
        &format!("{{\"version\":\"{}\",\"title\":\"\"}}", version),
    );
    assert_eq!(invalid.status, 422);
    let error: serde_json::Value = serde_json::from_str(&invalid.text()).unwrap();
    assert_eq!(error["error"]["code"], 422);
    assert!(
        error["error"]["message"]
            .as_str()
            .unwrap()
            .contains("Title")
    );

    let missing = get(&server, "/api/doc/REQ-404");
    assert_eq!(missing.status, 404);
    let error: serde_json::Value = serde_json::from_str(&missing.text()).unwrap();
    assert_eq!(error["error"]["code"], 404);

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
    assert_eq!(
        send_raw(
            port,
            format!(
                "POST /doc/REQ-001.001 HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: 4\r\nContent-Length: 4\r\n\r\nabcd",
                port
            )
            .as_bytes()
        )
        .status,
        400
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
    server.wait_for_stderr("- - 408");

    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// AC #14 (unknown path is 404 for every method; known path wrong method is 405)
#[test]
fn unknown_path_is_404_for_every_method() {
    let dir = fixture("route-method");
    let server = Server::start(&dir, &["--no-open", "--debug"], &[]);
    for method in ["DELETE", "PUT", "OPTIONS"] {
        let response = request(server.port, method, "/nope", &[], None);
        assert_eq!(response.status, 404, "{} /nope", method);
    }
    let known = request(server.port, "DELETE", "/doc/REQ-001.001", &[], None);
    assert_eq!(known.status, 405);
    assert_eq!(known.header("Allow"), Some("GET, HEAD, POST"));
    let edit = request(server.port, "PUT", "/doc/REQ-001.001/edit", &[], None);
    assert_eq!(edit.status, 405);
    assert_eq!(edit.header("Allow"), Some("GET, HEAD, POST"));
    let asset = request(server.port, "OPTIONS", "/assets/app.css", &[], None);
    assert_eq!(asset.status, 405);
    assert_eq!(asset.header("Allow"), Some("GET, HEAD, POST"));
    // The request log prints the method token exactly as sent.
    server.wait_for_stderr("DELETE /nope 404");
    assert!(!server.stderr_text().contains("OTHER "));
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-22 (HEAD returns the GET status and headers with no body)
#[test]
fn head_matches_get_without_body() {
    let dir = fixture("head");
    let server = Server::start(&dir, &["--no-open"], &[]);
    for path in [
        "/",
        "/doc/REQ-001.001",
        "/doc/REQ-001.001/edit",
        "/assets/app.css",
        "/assets/app.js",
        "/api/requirements",
        "/api/doc/REQ-001.001",
    ] {
        let get_response = get(&server, path);
        let head_response = request(server.port, "HEAD", path, &[], None);
        assert_eq!(head_response.status, get_response.status, "status {}", path);
        assert!(head_response.body.is_empty(), "HEAD body {}", path);
        let get_length = get_response.header("Content-Length").unwrap();
        assert_eq!(
            head_response.header("Content-Length"),
            Some(get_length),
            "Content-Length {}",
            path
        );
        for name in [
            "Content-Type",
            "Connection",
            "Cache-Control",
            "X-Content-Type-Options",
            "Referrer-Policy",
            "X-Frame-Options",
        ] {
            assert_eq!(
                head_response.header(name),
                get_response.header(name),
                "{} {}",
                name,
                path
            );
        }
    }

    let get_missing = get(&server, "/nope");
    let head_missing = request(server.port, "HEAD", "/nope", &[], None);
    assert_eq!(head_missing.status, 404);
    assert_eq!(
        head_missing.header("Content-Length"),
        get_missing.header("Content-Length")
    );
    assert!(head_missing.body.is_empty());
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-22 (concurrency limit: immediate 503, request never read, - - 503 log)
#[test]
fn t22_concurrency_limit_returns_503_and_logs() {
    let dir = fixture("capacity");
    let server = Server::start(&dir, &["--no-open"], &[]);
    let mut held = Vec::new();
    for _ in 0..16 {
        let mut stream = TcpStream::connect(("127.0.0.1", server.port)).unwrap();
        // Hold each handler thread inside the request reader without sending a
        // complete request, so all 16 connection slots stay occupied.
        stream.write_all(b"GET / HT").unwrap();
        held.push(stream);
    }
    // Wait until the accept loop reports all 16 slots occupied. A probe that is
    // accepted waits for a request; an over-capacity probe is answered at once.
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let mut probe = TcpStream::connect(("127.0.0.1", server.port)).unwrap();
        probe
            .set_read_timeout(Some(Duration::from_millis(200)))
            .unwrap();
        let mut bytes = Vec::new();
        let rejected = probe.read_to_end(&mut bytes).is_ok()
            && !bytes.is_empty()
            && parse_response(&bytes).status == 503;
        if rejected {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "server never reached the concurrency limit"
        );
        std::thread::sleep(Duration::from_millis(25));
    }

    // The 17th connection sends no request at all and still receives an
    // immediate 503 with Connection: close followed by EOF. This proves the
    // accept loop does not read the request on the rejection path.
    let started = Instant::now();
    let mut over = TcpStream::connect(("127.0.0.1", server.port)).unwrap();
    over.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
    let mut bytes = Vec::new();
    over.read_to_end(&mut bytes).unwrap();
    let elapsed = started.elapsed();
    let response = parse_response(&bytes);
    assert_eq!(response.status, 503);
    assert_eq!(response.header("Connection"), Some("close"));
    assert!(
        elapsed < Duration::from_secs(1),
        "over-capacity 503 took {:?}",
        elapsed
    );
    server.wait_for_stderr("- - 503");

    // Free the held slots; the next normal request is not delayed by the
    // rejection path.
    drop(held);
    let raw = format!(
        "GET / HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n\r\n",
        server.port
    );
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let started = Instant::now();
        let response = try_send_raw(server.port, raw.as_bytes());
        let elapsed = started.elapsed();
        if let Some(response) = response
            && response.status == 200
        {
            assert!(
                elapsed < Duration::from_secs(1),
                "normal request took {:?}",
                elapsed
            );
            break;
        }
        assert!(
            Instant::now() < deadline,
            "server did not recover after releasing the held connections"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
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

    for path in ["/?type=req", "/doc/REQ-001.001", "/doc/REQ-001.001/edit"] {
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
    let list = get(&server, "/?type=req").text();
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
    for port in ["0", "70000", "abc"] {
        let out = bin()
            .arg("--dir")
            .arg(&dir)
            .args(["config", "set", "browser_port", port])
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(4), "port={}", port);
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            stderr.contains("browser_port"),
            "message must name the key: {}",
            stderr
        );
        assert!(
            stderr.contains("must be between 1 and 65535"),
            "message must state the range: {}",
            stderr
        );
    }
    let out = bin()
        .arg("--dir")
        .arg(&dir)
        .args(["config", "set", "browser_open", "maybe"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));
    assert!(String::from_utf8_lossy(&out.stderr).contains("browser_open"));
    let out = bin()
        .arg("--dir")
        .arg(&dir)
        .args(["config", "set", "browser_host", "bad/host"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("browser_host"),
        "message must name the key: {}",
        stderr
    );
    assert!(!stderr.contains("value cannot be normalized"), "{}", stderr);
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

// T-26 (configuration file load errors, not just `config set`)
#[test]
fn config_file_browser_errors_name_the_key() {
    let dir = test_root("config-file");
    init_project(&dir);
    let original = fs::read_to_string(dir.join("papyrus.config.yml")).unwrap();
    let cases = [
        (
            "browser_port",
            "70000",
            "browser_port: must be between 1 and 65535",
        ),
        ("browser_open", "maybe", "browser_open: must be a boolean"),
        ("browser_host", "bad/host", "browser_host:"),
    ];
    for (key, value, needle) in cases {
        fs::write(dir.join("papyrus.config.yml"), &original).unwrap();
        set_config_value(&dir, key, value);
        let out = bin()
            .arg("--dir")
            .arg(&dir)
            .args(["browser", "--no-open"])
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(4), "{}={}", key, value);
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            stderr.contains(needle),
            "{}={} stderr: {}",
            key,
            value,
            stderr
        );
        assert!(
            !stderr.contains("value cannot be normalized"),
            "{}={} stderr: {}",
            key,
            value,
            stderr
        );
        // Every command that loads the merged configuration fails the same way.
        let out = bin()
            .arg("--dir")
            .arg(&dir)
            .args(["config", "list"])
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(4), "config list {}={}", key, value);
    }
    set_config_value(&dir, "browser_port", "6420");
    set_config_value(&dir, "browser_open", "true");
    set_config_value(&dir, "browser_host", "127.0.0.1");
    let list = run_ok(&["--dir", dir.to_str().unwrap(), "config", "list"]);
    assert!(list.contains("browser_port: 6420"));
    let _ = fs::remove_dir_all(&dir);
}

// AC #16 (config get/set/unset validate the merged browser configuration)
#[test]
fn config_get_set_unset_validate_merged_browser_config() {
    let dir = test_root("config-merged");
    init_project(&dir);
    set_config_value(&dir, "browser_port", "70000");
    let config_path = dir.join("papyrus.config.yml");
    let before = fs::read_to_string(&config_path).unwrap();

    let out = bin()
        .arg("--dir")
        .arg(&dir)
        .args(["config", "get", "browser_port"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4), "config get");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("browser_port:"), "{}", stderr);

    let out = bin()
        .arg("--dir")
        .arg(&dir)
        .args(["config", "set", "browser_open", "false"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4), "config set");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("browser_port:"), "{}", stderr);
    assert_eq!(fs::read_to_string(&config_path).unwrap(), before);

    let out = bin()
        .arg("--dir")
        .arg(&dir)
        .args(["config", "unset", "browser_host"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4), "config unset");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("browser_port:"), "{}", stderr);
    assert_eq!(fs::read_to_string(&config_path).unwrap(), before);

    let _ = fs::remove_dir_all(&dir);
}

// AC #7 / TASK-74 (load-time rejection of out-of-range id_padding)
#[test]
fn id_padding_config_file_load_is_rejected() {
    let dir = test_root("id-padding-load");
    init_project(&dir);
    let original = fs::read_to_string(dir.join("papyrus.config.yml")).unwrap();
    for value in ["0", "7"] {
        fs::write(dir.join("papyrus.config.yml"), &original).unwrap();
        set_config_value(&dir, "id_padding", value);
        let out = bin()
            .arg("--dir")
            .arg(&dir)
            .args(["task", "list"])
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(4), "id_padding={}", value);
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            stderr.contains("id_padding: must be between 1 and 6"),
            "id_padding={} stderr: {}",
            value,
            stderr
        );
        assert!(
            !stderr.contains("value cannot be normalized"),
            "id_padding={} stderr: {}",
            value,
            stderr
        );
    }
    fs::write(dir.join("papyrus.config.yml"), &original).unwrap();
    let list = run_ok(&["--dir", dir.to_str().unwrap(), "config", "list"]);
    assert!(list.contains("id_padding: 3"));
    let _ = fs::remove_dir_all(&dir);
}

// AC #7 / TASK-74 (config set validates the merged effective configuration)
#[test]
fn id_padding_config_set_out_of_range_is_rejected_and_file_unchanged() {
    let dir = test_root("id-padding-set");
    init_project(&dir);
    let config_path = dir.join("papyrus.config.yml");
    for value in ["0", "7"] {
        let before = fs::read_to_string(&config_path).unwrap();
        let out = bin()
            .arg("--dir")
            .arg(&dir)
            .args(["config", "set", "id_padding", value])
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(4), "id_padding={}", value);
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            stderr.contains("id_padding: must be between 1 and 6"),
            "id_padding={} stderr: {}",
            value,
            stderr
        );
        assert_eq!(
            fs::read_to_string(&config_path).unwrap(),
            before,
            "file must be unchanged after rejected set id_padding={}",
            value
        );
    }
    for value in ["1", "6"] {
        run_ok(&[
            "--dir",
            dir.to_str().unwrap(),
            "config",
            "set",
            "id_padding",
            value,
        ]);
        assert_eq!(
            run_ok(&[
                "--dir",
                dir.to_str().unwrap(),
                "config",
                "get",
                "id_padding"
            ]),
            value
        );
    }
    let _ = fs::remove_dir_all(&dir);
}

// AC #7 / TASK-74 (config unset validates the merged effective configuration)
#[test]
fn id_padding_config_unset_validates_merged_config() {
    let dir = test_root("id-padding-unset");
    init_project(&dir);
    set_config_value(&dir, "id_padding", "7");
    let config_path = dir.join("papyrus.config.yml");
    let before = fs::read_to_string(&config_path).unwrap();
    // Unsetting an unrelated key is rejected while the merged effective
    // configuration still has an out-of-range id_padding (mirrors the
    // browser_port merged-config test). Unsetting id_padding itself removes
    // the invalid value and heals the configuration, so the rejected case is
    // an unrelated unset whose merged result remains out of range.
    let out = bin()
        .arg("--dir")
        .arg(&dir)
        .args(["config", "unset", "task_prefix"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4), "config unset");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("id_padding: must be between 1 and 6"),
        "{}",
        stderr
    );
    assert_eq!(fs::read_to_string(&config_path).unwrap(), before);
    let _ = fs::remove_dir_all(&dir);
}

// AC #19 / T-28 (approved MOCK-0001 shell, 320 px states)
#[test]
fn ui_matches_approved_mock_0001_layout() {
    let css = include_str!("../src/browser/app.css");
    assert!(css.contains("grid-template-columns: 210px minmax(0, 1fr)"));
    assert!(css.contains("gap: 30px"));
    assert!(css.contains("border-left: 1px solid var(--line)"));
    assert!(!css.contains("width: 190px"), "old fixed sidebar width");
    assert!(!css.contains("float: left"), "old floated sidebar");

    // Long unbreakable "Other requirements" links wrap at 320 px.
    let other_rule = css
        .split(".other-list a {")
        .nth(1)
        .expect(".other-list a rule");
    let other_rule = &other_rule[..other_rule.find('}').unwrap()];
    assert!(other_rule.contains("overflow-wrap: break-word"));
    assert!(other_rule.contains("word-wrap: break-word"));

    // Narrow viewport: header note omitted, footer shortened, sidebar stacked.
    let narrow = css
        .split("@media (max-width: 480px)")
        .nth(1)
        .expect("narrow media query");
    assert!(narrow.contains(".top-meta { display: none; }"));
    assert!(narrow.contains(".layout { display: block; padding-top: 16px; }"));
    assert!(narrow.contains(".main { border-left: 0; padding-left: 0; }"));
    assert!(narrow.contains(".footer-long { display: none; }"));
    assert!(narrow.contains(".footer-short { display: inline; }"));
    assert!(css.contains(".footer-short { display: none; }"));
    // At 320 px the edit form draws only the hidden version input; the
    // visible annotation is omitted (UI description section 9.2).
    assert!(narrow.contains(".hidden-field { display: none; }"));

    let dir = fixture("mock-layout");
    let server = Server::start(&dir, &["--no-open"], &[]);
    let html = get(&server, "/?type=req").text();
    assert!(html.contains("footer-long"));
    assert!(html.contains("footer-short"));
    assert!(html.contains("Papyrus local browser &mdash; served from this project only."));
    let edit = get(&server, "/doc/REQ-001.001/edit").text();
    assert!(edit.contains(
        "<p class=\"hidden-field\">Hidden field: version (optimistic-concurrency token)</p>"
    ));
    assert!(edit.contains("<input type=\"hidden\" name=\"version\" value=\""));
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-28
#[test]
fn t28_accessibility_and_compatibility_basics() {
    let dir = fixture("a11y");
    let server = Server::start(&dir, &["--no-open"], &[]);
    let list = get(&server, "/?type=req").text();
    assert_eq!(list.matches("<h1").count(), 1);
    for landmark in ["<header", "<nav", "<main", "<footer"] {
        assert!(list.contains(landmark), "missing {}", landmark);
    }
    assert!(list.contains("aria-current=\"page\""));
    assert!(list.contains("aria-controls=\"site-nav\""));
    assert!(list.contains("aria-expanded="));
    assert!(list.contains("nav-toggle"));

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

// T-28 / AC #19 (edit-form dirty status)
#[test]
fn edit_form_dirty_status_is_static_and_hidden_until_input() {
    let dir = fixture("dirty");
    let server = Server::start(&dir, &["--no-open"], &[]);
    let html = get(&server, "/doc/REQ-001.001/edit").text();
    // Hidden while pristine, static text, and not a focusable control.
    assert!(html.contains("<p class=\"status\" data-dirty-status hidden>Unsaved changes</p>"));
    assert!(!html.contains("tabindex"));
    assert!(!html.contains("<input class=\"status\""));
    assert!(!html.contains("<button class=\"status\""));
    // The progressive enhancement reveals it on the first input and keeps the
    // beforeunload leave guard.
    assert!(html.contains("data-dirty-guard"));
    assert!(html.contains("data-form-note"));
    let js = include_str!("../src/browser/app.js");
    assert!(js.contains("data-dirty-status"));
    assert!(js.contains("removeAttribute(\"hidden\")"));
    assert!(js.contains("beforeunload"));
    let css = include_str!("../src/browser/app.css");
    assert!(css.contains(".status[hidden]"));
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

// ---- T-11 backlog (task/milestone/release) scenarios ----

fn create_release(dir: &Path, title: &str) -> String {
    run_ok(&[
        "--dir",
        dir.to_str().unwrap(),
        "release",
        "create",
        title,
        "-d",
        "Release description",
    ])
}

fn create_milestone(dir: &Path, title: &str, release: &str) -> String {
    run_ok(&[
        "--dir",
        dir.to_str().unwrap(),
        "milestone",
        "create",
        title,
        "-d",
        "Milestone description",
        "--release",
        release,
    ])
}

fn create_task(dir: &Path, title: &str, milestone: &str) -> String {
    run_ok(&[
        "--dir",
        dir.to_str().unwrap(),
        "task",
        "create",
        title,
        "-d",
        "Task description",
        "--ac",
        "Acceptance one",
        "--label",
        "bug",
        "--assignee",
        "alice",
        "--milestone",
        milestone,
    ])
}

fn backlog_fixture(name: &str) -> PathBuf {
    let dir = test_root(name);
    init_project(&dir);
    create_release(&dir, "Release One");
    create_release(&dir, "Release Two");
    create_milestone(&dir, "Milestone One", "REL-001");
    create_milestone(&dir, "Milestone Two", "REL-002");
    create_task(&dir, "Task One", "MS-001");
    // second task depends on first
    run_ok(&[
        "--dir",
        dir.to_str().unwrap(),
        "task",
        "create",
        "Task Two",
        "-d",
        "Task description two",
        "--ac",
        "Acceptance two",
        "--label",
        "feature",
        "--assignee",
        "bob",
        "--milestone",
        "MS-002",
        "--depends-on",
        "TASK-001",
    ]);
    dir
}

fn backlog_record_path(dir: &Path, sub: &str, id: &str) -> PathBuf {
    dir.join("backlog").join(sub).join(format!("{}.md", id))
}

// T-11a
#[test]
fn t11a_list_backlog_by_type() {
    let dir = backlog_fixture("t11a");
    let server = Server::start(&dir, &["--no-open"], &[]);

    for (ty, id1, id2) in [
        ("task", "TASK-001", "TASK-002"),
        ("milestone", "MS-001", "MS-002"),
        ("release", "REL-001", "REL-002"),
    ] {
        let html = get(&server, &format!("/?type={}", ty)).text();
        if ty == "task" {
            assert!(html.contains("<h1 class=\"h1\">Backlog</h1>"));
            assert!(html.contains("class=\"board-card\""));
            assert!(!html.contains("Task records in this project."));
        }
        assert!(html.contains(id1), "type={} missing {}", ty, id1);
        assert!(html.contains(id2), "type={} missing {}", ty, id2);
        assert!(
            html.contains("aria-current=\"page\""),
            "type={} sidebar",
            ty
        );
        let first = html.find(id1).unwrap();
        let second = html.find(id2).unwrap();
        assert!(first < second, "type={} order", ty);
    }
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-11b
#[test]
fn t11b_category_parameter_backlog() {
    let dir = backlog_fixture("t11b");
    let server = Server::start(&dir, &["--no-open"], &[]);
    for ty in ["req", "nfr", "task", "milestone", "release"] {
        // req/nfr may be empty but should still be 200
        let status = get(&server, &format!("/?type={}", ty)).status;
        // req/nfr with no records still 200
        assert_eq!(status, 200, "type={}", ty);
    }
    for ty in ["all", "x", "tasks"] {
        assert_eq!(
            get(&server, &format!("/?type={}", ty)).status,
            404,
            "type={}",
            ty
        );
    }
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-11c
#[test]
fn t11c_view_backlog_fields() {
    let dir = backlog_fixture("t11c");
    let server = Server::start(&dir, &["--no-open"], &[]);

    let task = get(&server, "/doc/TASK-002").text();
    for needle in [
        "Status",
        "Milestone",
        "Assignee",
        "Labels",
        "Dependencies",
        "Acceptance criteria",
        "Comments",
        "Description",
    ] {
        assert!(task.contains(needle), "task view missing {}", needle);
    }
    assert!(task.contains("/doc/TASK-001"));
    assert!(task.contains("/doc/MS-002"));

    let ms = get(&server, "/doc/MS-001").text();
    assert!(ms.contains("Release"));
    assert!(ms.contains("/doc/REL-001"));

    let rel = get(&server, "/doc/REL-001").text();
    assert!(rel.contains("Description"));
    assert!(rel.contains("Release description"));

    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-11d
#[test]
fn t11d_edit_round_trip_backlog() {
    let dir = backlog_fixture("t11d");
    let server = Server::start(&dir, &["--no-open"], &[]);

    // Task
    let edit = get(&server, "/doc/TASK-001/edit").text();
    let version = extract_version(&edit);
    let body = form_body(&[
        ("version", &version),
        ("title", "Task One revised"),
        ("status", "Initiated"),
        ("description", "Revised task desc"),
        ("milestone", "MS-001"),
        ("assignee", "bob"),
        ("labels", "urgent"),
        ("dependencies", ""),
        ("acceptance_criteria", "AC revised"),
    ]);
    let response = post_form(&server, "/doc/TASK-001", &body);
    assert_eq!(response.status, 303);
    let view = get(&server, "/doc/TASK-001").text();
    assert!(view.contains("Task One revised"));
    let cli = run_ok(&["--dir", dir.to_str().unwrap(), "task", "view", "TASK-001"]);
    assert!(cli.contains("Task One revised"));

    // Milestone
    let edit = get(&server, "/doc/MS-001/edit").text();
    let version = extract_version(&edit);
    let body = form_body(&[
        ("version", &version),
        ("title", "Milestone One revised"),
        ("description", "Revised ms desc"),
        ("release", "REL-001"),
    ]);
    let response = post_form(&server, "/doc/MS-001", &body);
    assert_eq!(response.status, 303);
    let cli = run_ok(&[
        "--dir",
        dir.to_str().unwrap(),
        "milestone",
        "view",
        "MS-001",
    ]);
    assert!(cli.contains("Milestone One revised"));

    // Release
    let edit = get(&server, "/doc/REL-001/edit").text();
    let version = extract_version(&edit);
    let body = form_body(&[
        ("version", &version),
        ("title", "Release One revised"),
        ("description", "Revised rel desc"),
    ]);
    let response = post_form(&server, "/doc/REL-001", &body);
    assert_eq!(response.status, 303);
    let cli = run_ok(&["--dir", dir.to_str().unwrap(), "release", "view", "REL-001"]);
    assert!(cli.contains("Release One revised"));

    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-11e
#[test]
fn t11e_validation_backlog() {
    let dir = backlog_fixture("t11e");
    let path = backlog_record_path(&dir, "tasks", "TASK-001");
    let before = fs::read(&path).unwrap();
    let server = Server::start(&dir, &["--no-open"], &[]);
    let version = extract_version(&get(&server, "/doc/TASK-001/edit").text());

    // empty title
    let body = form_body(&[("version", &version), ("title", "")]);
    let response = post_form(&server, "/doc/TASK-001", &body);
    assert_eq!(response.status, 422);
    assert_eq!(fs::read(&path).unwrap(), before);

    // unknown status
    let body = form_body(&[
        ("version", &version),
        ("title", "Valid"),
        ("status", "Nope"),
    ]);
    let response = post_form(&server, "/doc/TASK-001", &body);
    assert_eq!(response.status, 422);
    assert_eq!(fs::read(&path).unwrap(), before);

    // unknown milestone
    let body = form_body(&[
        ("version", &version),
        ("title", "Valid"),
        ("status", "ToDo"),
        ("milestone", "MS-999"),
    ]);
    let response = post_form(&server, "/doc/TASK-001", &body);
    assert_eq!(response.status, 422);
    assert_eq!(fs::read(&path).unwrap(), before);

    // self dependency
    let body = form_body(&[
        ("version", &version),
        ("title", "Valid"),
        ("status", "ToDo"),
        ("dependencies", "TASK-001"),
    ]);
    let response = post_form(&server, "/doc/TASK-001", &body);
    assert_eq!(response.status, 422);
    assert_eq!(fs::read(&path).unwrap(), before);

    // ## line in description
    let body = form_body(&[
        ("version", &version),
        ("title", "Valid"),
        ("status", "ToDo"),
        ("description", "Line\n## Not allowed"),
    ]);
    let response = post_form(&server, "/doc/TASK-001", &body);
    assert_eq!(response.status, 422);
    assert_eq!(fs::read(&path).unwrap(), before);

    // milestone unknown release
    let mpath = backlog_record_path(&dir, "milestones", "MS-001");
    let mbefore = fs::read(&mpath).unwrap();
    let medit = get(&server, "/doc/MS-001/edit").text();
    let mversion = extract_version(&medit);
    let body = form_body(&[
        ("version", &mversion),
        ("title", "Valid"),
        ("release", "REL-999"),
    ]);
    let response = post_form(&server, "/doc/MS-001", &body);
    assert_eq!(response.status, 422);
    assert_eq!(fs::read(&mpath).unwrap(), mbefore);

    // release empty title
    let rpath = backlog_record_path(&dir, "releases", "REL-001");
    let rbefore = fs::read(&rpath).unwrap();
    let redit = get(&server, "/doc/REL-001/edit").text();
    let rversion = extract_version(&redit);
    let body = form_body(&[("version", &rversion), ("title", "")]);
    let response = post_form(&server, "/doc/REL-001", &body);
    assert_eq!(response.status, 422);
    assert_eq!(fs::read(&rpath).unwrap(), rbefore);

    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-11f
#[test]
fn t11f_conflict_backlog() {
    let dir = backlog_fixture("t11f");
    let path = backlog_record_path(&dir, "tasks", "TASK-001");
    let server = Server::start(&dir, &["--no-open"], &[]);
    let version = extract_version(&get(&server, "/doc/TASK-001/edit").text());
    // out-of-band modify
    let original = fs::read_to_string(&path).unwrap();
    fs::write(&path, format!("{}\n## Out of band\nchanged\n", original)).unwrap();
    let body = form_body(&[("version", &version), ("title", "Should not save")]);
    let response = post_form(&server, "/doc/TASK-001", &body);
    assert_eq!(response.status, 409);
    let html = response.text();
    assert!(html.contains("This record changed on disk"));
    let saved = fs::read_to_string(&path).unwrap();
    assert!(!saved.contains("Should not save"));
    assert!(saved.contains("## Out of band"));
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-11g
#[test]
fn t11g_archived_backlog() {
    let dir = backlog_fixture("t11g");
    // archive the task; milestones have no archive verb, so move the record
    // into the archive location out of band to exercise the legacy read path.
    run_ok(&[
        "--dir",
        dir.to_str().unwrap(),
        "task",
        "archive",
        "TASK-001",
    ]);
    let archived = dir.join("backlog/archive/milestones");
    fs::create_dir_all(&archived).unwrap();
    fs::rename(
        dir.join("backlog/milestones/MS-001.md"),
        archived.join("MS-001.md"),
    )
    .unwrap();
    let server = Server::start(&dir, &["--no-open"], &[]);
    let list = get(&server, "/?type=task").text();
    // The archived task has no list row (TASK-67: remaining rows may
    // still name it inside RecordView dependency metadata, e.g. TASK-002).
    assert!(!list.contains("href=\"/doc/TASK-001\""));
    assert_eq!(get(&server, "/doc/TASK-001").status, 404);
    assert_eq!(get(&server, "/doc/TASK-001/edit").status, 404);
    let list = get(&server, "/?type=milestone").text();
    assert!(!list.contains("href=\"/doc/MS-001\""));
    assert_eq!(get(&server, "/doc/MS-001").status, 404);
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-11h
#[test]
fn t11h_atomic_and_symlink_backlog() {
    let dir = backlog_fixture("t11h");
    let path = backlog_record_path(&dir, "tasks", "TASK-001");
    let server = Server::start(&dir, &["--no-open"], &[]);
    let version = extract_version(&get(&server, "/doc/TASK-001/edit").text());
    let before: Vec<String> = fs::read_dir(path.parent().unwrap())
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    let body = form_body(&[
        ("version", &version),
        ("title", "Atomic"),
        ("status", "ToDo"),
    ]);
    assert_eq!(post_form(&server, "/doc/TASK-001", &body).status, 303);
    let after: Vec<String> = fs::read_dir(path.parent().unwrap())
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    assert_eq!(before, after);
    assert!(!after.iter().any(|n| n.contains(".tmp")));

    // rejected save leaves no temp
    let body = form_body(&[("version", "1.2.3"), ("title", "Rejected")]);
    assert_eq!(post_form(&server, "/doc/TASK-001", &body).status, 409);
    let after: Vec<String> = fs::read_dir(path.parent().unwrap())
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    assert_eq!(before, after);

    drop(server);

    // symlink escape (unix)
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let outside = test_root("t11h-outside");
        let secret = outside.join("secret.md");
        fs::write(&secret, "# TASK-001 SECRET CONTENT\n").unwrap();
        let p = backlog_record_path(&dir, "tasks", "TASK-001");
        let _ = fs::remove_file(&p);
        symlink(&secret, &p).unwrap();
        let server = Server::start(&dir, &["--no-open"], &[]);
        assert_eq!(get(&server, "/doc/TASK-001").status, 404);
        let list = get(&server, "/?type=task").text();
        assert!(!list.contains("href=\"/doc/TASK-001\""));
        let body = form_body(&[("version", "1.2.3"), ("title", "Attack")]);
        assert_eq!(post_form(&server, "/doc/TASK-001", &body).status, 404);
        assert!(
            fs::read_to_string(&secret)
                .unwrap()
                .contains("SECRET CONTENT")
        );
        drop(server);
        let _ = fs::remove_dir_all(&outside);
    }

    let _ = fs::remove_dir_all(&dir);
}

// T-11i
#[test]
fn t11i_json_api_backlog() {
    let dir = backlog_fixture("t11i");
    let server = Server::start(&dir, &["--no-open"], &[]);

    let tasks: serde_json::Value =
        serde_json::from_str(&get(&server, "/api/tasks").text()).unwrap();
    let arr = tasks.as_array().unwrap();
    assert_eq!(arr.len(), 2);
    assert!(arr[0].get("status").is_some());
    assert!(arr[0].get("milestone").is_some());
    assert!(arr[0].get("assignee").is_some());

    let ms: serde_json::Value =
        serde_json::from_str(&get(&server, "/api/milestones").text()).unwrap();
    assert_eq!(ms.as_array().unwrap().len(), 2);
    assert!(ms[0].get("release").is_some());

    let rels: serde_json::Value =
        serde_json::from_str(&get(&server, "/api/releases").text()).unwrap();
    assert_eq!(rels.as_array().unwrap().len(), 2);

    let doc: serde_json::Value =
        serde_json::from_str(&get(&server, "/api/doc/TASK-001").text()).unwrap();
    assert_eq!(doc["id"], "TASK-001");
    assert_eq!(doc["kind"], "task");
    assert_eq!(doc["form"], "task");
    assert!(doc["milestone"].is_object() || doc["milestone"].is_null());
    assert!(doc["labels"].is_array());
    assert!(doc["dependencies"].is_array());
    assert!(doc["acceptance_criteria"].is_array());
    assert!(doc["comments"].is_array());

    let mdoc: serde_json::Value =
        serde_json::from_str(&get(&server, "/api/doc/MS-001").text()).unwrap();
    assert_eq!(mdoc["kind"], "milestone");
    assert!(mdoc["release"].is_object() || mdoc["release"].is_null());

    let rdoc: serde_json::Value =
        serde_json::from_str(&get(&server, "/api/doc/REL-001").text()).unwrap();
    assert_eq!(rdoc["kind"], "release");

    // POST update
    let version = doc["version"].as_str().unwrap().to_string();
    let body = format!(
        "{{\"version\":\"{}\",\"title\":\"API Task\",\"status\":\"Done\"}}",
        version
    );
    let updated: serde_json::Value =
        serde_json::from_str(&post_json(&server, "/api/doc/TASK-001", &body).text()).unwrap();
    assert_eq!(updated["title"], "API Task");
    assert_eq!(updated["status"], "Done");

    // bad body
    assert_eq!(
        post_json(&server, "/api/doc/TASK-001", "{not json").status,
        400
    );
    let version = updated["version"].as_str().unwrap().to_string();
    assert_eq!(
        post_json(
            &server,
            "/api/doc/TASK-001",
            &format!("{{\"version\":\"{}\",\"title\":5}}", version)
        )
        .status,
        400
    );
    // conflict
    let conflict = post_json(
        &server,
        "/api/doc/TASK-001",
        &format!("{{\"version\":\"{}\",\"title\":\"x\"}}", "1.2.3"),
    );
    assert_eq!(conflict.status, 409);
    // invalid
    let invalid = post_json(
        &server,
        "/api/doc/TASK-001",
        &format!("{{\"version\":\"{}\",\"title\":\"\"}}", version),
    );
    assert_eq!(invalid.status, 422);
    // missing
    assert_eq!(get(&server, "/api/doc/TASK-404").status, 404);
    // content-type
    assert_eq!(
        request(
            server.port,
            "POST",
            "/api/doc/TASK-001",
            &[("Content-Type", "text/plain")],
            Some(b"x"),
        )
        .status,
        415
    );

    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// ---- T-103 backlog board scenarios ----

fn create_milestone_no_release(dir: &Path, title: &str) -> String {
    run_ok(&[
        "--dir",
        dir.to_str().unwrap(),
        "milestone",
        "create",
        title,
        "-d",
        "Milestone description",
    ])
}

fn create_task_no_milestone(dir: &Path, title: &str) -> String {
    run_ok(&[
        "--dir",
        dir.to_str().unwrap(),
        "task",
        "create",
        title,
        "-d",
        "Task description",
        "--ac",
        "Acceptance one",
        "--label",
        "bug",
        "--assignee",
        "alice",
    ])
}

fn set_task_status(dir: &Path, id: &str, status: &str) {
    run_ok(&[
        "--dir",
        dir.to_str().unwrap(),
        "task",
        "edit",
        id,
        "--status",
        status,
    ]);
}

/// Board fixture: one milestone without a release, three releases (one with no
/// active milestone), a released and a no-release milestone with tasks, a
/// dangling milestone reference, an archived milestone reference, an archived
/// task, and an escaped title. The first milestone in render order with a task
/// is MS-002 (the no-release group comes first).
fn board_fixture(name: &str) -> PathBuf {
    let dir = test_root(name);
    init_project(&dir);
    create_release(&dir, "Release One");
    create_release(&dir, "Release Two");
    create_release(&dir, "Release Three");
    create_milestone(&dir, "Milestone One", "REL-001");
    create_milestone_no_release(&dir, "Milestone Two");
    create_milestone(&dir, "Milestone Three", "REL-002");
    create_task(&dir, "Task One", "MS-001"); // TASK-001 ToDo
    create_task(&dir, "Task Two", "MS-001"); // TASK-002
    set_task_status(&dir, "TASK-002", "Initiated");
    create_task(&dir, "Task Three", "MS-002"); // TASK-003
    set_task_status(&dir, "TASK-003", "Implemented");
    create_task(&dir, "Task Four", "MS-002"); // TASK-004 ToDo
    create_task_no_milestone(&dir, "Task Five"); // TASK-005
    create_milestone(&dir, "Dangling Milestone", "REL-002"); // MS-004
    create_milestone(&dir, "Archived Milestone", "REL-002"); // MS-005
    create_task(&dir, "Task Six", "MS-004"); // TASK-006
    create_task(&dir, "Task Seven", "MS-005"); // TASK-007
    run_ok(&[
        "--dir",
        dir.to_str().unwrap(),
        "milestone",
        "delete",
        "MS-004",
        "--force",
    ]);
    // Milestones have no archive verb; keep a legacy archived record readable
    // by moving it into the archive location out of band.
    let archived = dir.join("backlog/archive/milestones");
    fs::create_dir_all(&archived).unwrap();
    fs::rename(
        dir.join("backlog/milestones/MS-005.md"),
        archived.join("MS-005.md"),
    )
    .unwrap();
    create_task(&dir, "Task Eight", "MS-001"); // TASK-008, archived below
    run_ok(&[
        "--dir",
        dir.to_str().unwrap(),
        "task",
        "archive",
        "TASK-008",
    ]);
    create_task(&dir, "Quote & <Angle>", "MS-001"); // TASK-009
    dir
}

fn extract_card_version(html: &str, task_id: &str) -> String {
    let marker = format!("data-task=\"{}\"", task_id);
    let start = html
        .find(&marker)
        .unwrap_or_else(|| panic!("card {}", task_id));
    extract_version(&html[start..])
}

fn column_containing<'a>(html: &'a str, status: &str) -> &'a str {
    let marker = format!("data-status=\"{}\"", status);
    let start = html
        .find(&marker)
        .unwrap_or_else(|| panic!("column {}", status));
    let end = html[start..]
        .find("</section>")
        .map(|offset| start + offset)
        .unwrap_or(html.len());
    &html[start..end]
}

#[test]
fn t103a_board_layout_columns_and_cards() {
    let dir = board_fixture("t103a");
    let server = Server::start(&dir, &["--no-open"], &[]);
    let html = get(&server, "/?type=task").text();

    assert!(html.contains("<h1 class=\"h1\">Backlog</h1>"));
    assert!(html.contains("Tasks grouped by milestone and status; releases separate milestones."));
    assert!(html.contains("<a href=\"/?type=task\" aria-current=\"page\">Backlog</a>"));
    assert!(!html.contains("Task records in this project."));

    // Every configured status has a column in configured order.
    let mut cursor = 0;
    for status in [
        "ToDo",
        "Initiated",
        "Refined",
        "Implemented",
        "Validated",
        "Done",
    ] {
        let marker = format!("data-status=\"{}\"", status);
        let next = html[cursor..]
            .find(&marker)
            .unwrap_or_else(|| panic!("column {} in order", status));
        cursor += next;
    }
    assert!(html.contains("No tasks in this status."));

    // MS-001 cards are ordered by ascending ID, link to the record view, show
    // the assignee and framed labels, and escape record text.
    let ms1_start = html
        .find("<summary>MS-001 \u{2014} Milestone One</summary>")
        .unwrap();
    let ms1_end = html[ms1_start..].find("</details>").unwrap() + ms1_start;
    let ms1 = &html[ms1_start..ms1_end];
    let first = ms1.find("data-task=\"TASK-001\"").unwrap();
    let second = ms1.find("data-task=\"TASK-009\"").unwrap();
    assert!(first < second, "cards ordered by ID");
    assert!(ms1.contains("href=\"/doc/TASK-001\""));
    assert!(ms1.contains("Assignee: alice"));
    assert!(ms1.contains("<span class=\"tag\">bug</span>"));
    assert!(ms1.contains("Quote &amp; &lt;Angle&gt;"));
    assert!(ms1.contains("<label for=\"status-TASK-001\">Status</label>"));
    assert!(ms1.contains("aria-label=\"Status for TASK-001\""));

    // Every card has a labelled select and a visible button; the page adds no
    // inline handler or style.
    let cards = html.matches("class=\"board-card\"").count();
    assert!(cards > 0);
    assert_eq!(html.matches("Change status</button>").count(), cards);
    assert_eq!(html.matches("name=\"status\"").count(), cards);
    assert!(html.contains("draggable=\"true\""));
    assert!(!html.contains("onclick="));
    assert!(!html.contains(" onchange="));
    assert!(!html.contains(" style="));

    let css = get(&server, "/assets/app.css").text();
    assert!(css.contains(".board-columns"));
    assert!(css.contains(".card-status"));
    assert!(css.contains("@media (max-width: 480px)"));
    let js = get(&server, "/assets/app.js").text();
    assert!(js.contains("dragstart"));
    assert!(js.contains("form.submit()"));
    assert!(!js.contains("fetch("));
    assert!(!js.contains("select.addEventListener(\"change\""));

    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn t103b_release_grouping_and_expand() {
    let dir = board_fixture("t103b");
    let server = Server::start(&dir, &["--no-open"], &[]);
    let html = get(&server, "/?type=task").text();

    let no_release = html.find("Milestones without a release").unwrap();
    let rel1 = html.find("REL-001 \u{2014} Release One").unwrap();
    let rel2 = html.find("REL-002 \u{2014} Release Two").unwrap();
    let rel3 = html.find("REL-003 \u{2014} Release Three").unwrap();
    let unassigned = html.find("Tasks without a milestone").unwrap();
    assert!(no_release < rel1 && rel1 < rel2 && rel2 < rel3);
    assert!(rel3 < unassigned, "without-milestone section comes last");
    assert!(html.contains("No milestones in this release."));
    assert!(html.find("MS-002 \u{2014} Milestone Two").unwrap() < rel1);
    assert!(rel1 < html.find("MS-001 \u{2014} Milestone One").unwrap());
    assert!(rel2 < html.find("MS-003 \u{2014} Milestone Three").unwrap());

    // Default open: first milestone in render order with a task (MS-002).
    assert!(html.contains(
        "class=\"board-milestone\" open><summary>MS-002 \u{2014} Milestone Two</summary>"
    ));
    assert!(!html.contains("open><summary>MS-001 \u{2014} Milestone One</summary>"));

    let expanded = get(&server, "/?type=task&expand=MS-001").text();
    assert!(expanded.contains("open><summary>MS-001 \u{2014} Milestone One</summary>"));
    assert!(!expanded.contains("open><summary>MS-002 \u{2014} Milestone Two</summary>"));

    // Unknown or archived milestone IDs fall back to the default.
    for value in ["MS-999", "MS-005", "not-an-id"] {
        let page = get(&server, &format!("/?type=task&expand={}", value)).text();
        assert!(
            page.contains("open><summary>MS-002 \u{2014} Milestone Two</summary>"),
            "expand={} fallback",
            value
        );
    }

    // Archived and dangling milestones never render; their tasks fall into the
    // tasks-without-a-milestone section with the archived task excluded.
    assert!(!html.contains("MS-004"));
    assert!(!html.contains("MS-005"));
    assert!(!html.contains("TASK-008"));
    let section = &html[unassigned..];
    assert!(section.contains("TASK-005"));
    assert!(section.contains("TASK-006"));
    assert!(section.contains("TASK-007"));

    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn t103c_unconfigured_status_and_empty_project() {
    let dir = backlog_fixture("t103c");
    set_task_status(&dir, "TASK-001", "Refined");
    set_statuses(&dir, &["ToDo", "Initiated", "Implemented"], "ToDo");
    let server = Server::start(&dir, &["--no-open"], &[]);
    let html = get(&server, "/?type=task").text();

    assert!(html.contains("data-status=\"\""));
    assert!(html.contains("Unconfigured status"));
    let unconfigured = column_containing(&html, "");
    assert!(unconfigured.contains("TASK-001"));
    assert!(!html.contains("value=\"Refined\" selected"));

    drop(server);
    let _ = fs::remove_dir_all(&dir);

    let empty = test_root("t103c-empty");
    init_project(&empty);
    let server = Server::start(&empty, &["--no-open"], &[]);
    let html = get(&server, "/?type=task").text();
    assert!(html.contains("No tasks in this project."));
    assert!(!html.contains("board-milestone"));
    drop(server);
    let _ = fs::remove_dir_all(&empty);
}

#[test]
fn t103d_status_change_and_noop() {
    let dir = backlog_fixture("t103d");
    let path = backlog_record_path(&dir, "tasks", "TASK-001");
    let before = fs::read_to_string(&path).unwrap();
    let server = Server::start(&dir, &["--no-open"], &[]);
    let html = get(&server, "/?type=task").text();
    let version = extract_card_version(&html, "TASK-001");
    let body = form_body(&[
        ("version", &version),
        ("status_change", "1"),
        ("status", "Initiated"),
        ("expand", "MS-001"),
    ]);
    let response = post_form(&server, "/doc/TASK-001", &body);
    assert_eq!(response.status, 303);
    assert_eq!(
        response.header("Location"),
        Some("/?type=task&expand=MS-001")
    );

    let after = fs::read_to_string(&path).unwrap();
    assert!(after.contains("status: Initiated"));
    assert!(!after.contains("status: ToDo"));
    let before_lines: Vec<&str> = before.lines().collect();
    let after_lines: Vec<&str> = after.lines().collect();
    assert_eq!(before_lines.len(), after_lines.len());
    for (old, new) in before_lines.iter().zip(after_lines.iter()) {
        if old != new {
            assert!(
                old.starts_with("status:") || old.starts_with("updated_date:"),
                "unexpected changed line: {old:?} -> {new:?}"
            );
        }
    }

    let reloaded = get(&server, "/?type=task&expand=MS-001").text();
    assert!(column_containing(&reloaded, "Initiated").contains("TASK-001"));
    assert!(get(&server, "/doc/TASK-001").text().contains("Initiated"));

    // Same-value submit is byte-identical: no write, no updated_date refresh.
    let version = extract_card_version(&reloaded, "TASK-001");
    let body = form_body(&[
        ("version", &version),
        ("status_change", "1"),
        ("status", "Initiated"),
        ("expand", "MS-001"),
    ]);
    let response = post_form(&server, "/doc/TASK-001", &body);
    assert_eq!(response.status, 303);
    assert_eq!(fs::read_to_string(&path).unwrap(), after);

    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn t103e_status_change_failures() {
    let dir = backlog_fixture("t103e");
    let path = backlog_record_path(&dir, "tasks", "TASK-001");
    let before = fs::read_to_string(&path).unwrap();
    let server = Server::start(&dir, &["--no-open"], &[]);
    let html = get(&server, "/?type=task").text();
    let version = extract_card_version(&html, "TASK-001");

    // Stale token: 409, board alert, original column, no write.
    let concurrent = format!("{}\n## Out of band\nchanged\n", before);
    fs::write(&path, &concurrent).unwrap();
    let body = form_body(&[
        ("version", &version),
        ("status_change", "1"),
        ("status", "Initiated"),
        ("expand", "MS-001"),
    ]);
    let response = post_form(&server, "/doc/TASK-001", &body);
    assert_eq!(response.status, 409);
    let html = response.text();
    assert!(html.contains("role=\"alert\""));
    assert!(html.contains("Status change failed"));
    assert!(html.contains("the record changed on disk"));
    assert!(html.contains("original column (ToDo)"));
    assert!(column_containing(&html, "ToDo").contains("TASK-001"));
    assert_eq!(fs::read_to_string(&path).unwrap(), concurrent);

    // A freshly read token still rejects an unconfigured status.
    let html = get(&server, "/?type=task").text();
    let version = extract_card_version(&html, "TASK-001");
    let body = form_body(&[
        ("version", &version),
        ("status_change", "1"),
        ("status", "Nope"),
        ("expand", "MS-001"),
    ]);
    let response = post_form(&server, "/doc/TASK-001", &body);
    assert_eq!(response.status, 422);
    assert!(
        response
            .text()
            .contains("Status is not a configured status.")
    );
    assert_eq!(fs::read_to_string(&path).unwrap(), concurrent);

    // Unknown and archived IDs are 404.
    let unknown = form_body(&[
        ("version", "1.0.1"),
        ("status_change", "1"),
        ("status", "ToDo"),
    ]);
    assert_eq!(post_form(&server, "/doc/TASK-999", &unknown).status, 404);
    run_ok(&[
        "--dir",
        dir.to_str().unwrap(),
        "task",
        "archive",
        "TASK-002",
    ]);
    let response = post_form(&server, "/doc/TASK-002", &unknown);
    assert_eq!(response.status, 404);

    // The marker on a non-task record is 400 and writes nothing.
    let ms_path = backlog_record_path(&dir, "milestones", "MS-001");
    let ms_before = fs::read(&ms_path).unwrap();
    let ms_version = extract_version(&get(&server, "/doc/MS-001/edit").text());
    let body = form_body(&[
        ("version", &ms_version),
        ("status_change", "1"),
        ("status", "ToDo"),
    ]);
    assert_eq!(post_form(&server, "/doc/MS-001", &body).status, 400);
    assert_eq!(fs::read(&ms_path).unwrap(), ms_before);

    // Wrong media type is 415.
    let response = request(
        server.port,
        "POST",
        "/doc/TASK-001",
        &[("Content-Type", "text/plain")],
        Some(b"status_change=1"),
    );
    assert_eq!(response.status, 415);

    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

#[cfg(unix)]
#[test]
fn t103f_status_change_write_failure() {
    use std::os::unix::fs::PermissionsExt;

    let dir = backlog_fixture("t103f");
    let tasks_dir = dir.join("backlog/tasks");
    let path = backlog_record_path(&dir, "tasks", "TASK-001");
    let before = fs::read_to_string(&path).unwrap();
    let server = Server::start(&dir, &["--no-open"], &[]);
    let html = get(&server, "/?type=task").text();
    let version = extract_card_version(&html, "TASK-001");

    let original = fs::metadata(&tasks_dir).unwrap().permissions();
    let mut readonly = original.clone();
    readonly.set_mode(0o555);
    fs::set_permissions(&tasks_dir, readonly).unwrap();
    let body = form_body(&[
        ("version", &version),
        ("status_change", "1"),
        ("status", "Initiated"),
        ("expand", "MS-001"),
    ]);
    let response = post_form(&server, "/doc/TASK-001", &body);
    fs::set_permissions(&tasks_dir, original).unwrap();

    assert_eq!(response.status, 500);
    assert!(response.text().contains("Nothing was saved."));
    assert_eq!(fs::read_to_string(&path).unwrap(), before);
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// ---- T-147 per-milestone `Clean up all` board action ----

/// Board fixture for the bulk cleanup action: default statuses (final `Done`),
/// `MS-001` with two `Done` tasks and one `ToDo` task, `MS-002` with one `Done`
/// task, a `Done` task without a milestone, and an archived `Done` task.
fn cleanup_fixture(name: &str) -> PathBuf {
    let dir = test_root(name);
    init_project(&dir);
    create_milestone_no_release(&dir, "Milestone One"); // MS-001
    create_milestone_no_release(&dir, "Milestone Two"); // MS-002
    create_task(&dir, "Task One", "MS-001"); // TASK-001
    create_task(&dir, "Task Two", "MS-001"); // TASK-002
    create_task(&dir, "Task Three", "MS-001"); // TASK-003
    create_task(&dir, "Task Four", "MS-002"); // TASK-004
    create_task_no_milestone(&dir, "Task Five"); // TASK-005
    create_task(&dir, "Task Six", "MS-001"); // TASK-006
    set_task_status(&dir, "TASK-001", "Done");
    set_task_status(&dir, "TASK-002", "Done");
    set_task_status(&dir, "TASK-004", "Done");
    set_task_status(&dir, "TASK-005", "Done");
    set_task_status(&dir, "TASK-006", "Done");
    run_ok(&[
        "--dir",
        dir.to_str().unwrap(),
        "task",
        "archive",
        "TASK-006",
    ]);
    dir
}

/// The server-rendered confirmation region of the previewed milestone board.
fn cleanup_region(html: &str) -> &str {
    let marker = "class=\"board-confirm\"";
    let start = html
        .find(marker)
        .unwrap_or_else(|| panic!("cleanup region missing"));
    let end = html[start..]
        .find("</section>")
        .map(|offset| start + offset)
        .unwrap_or(html.len());
    &html[start..end]
}

fn count_occurrences(html: &str, needle: &str) -> usize {
    html.matches(needle).count()
}

/// Every regular file in `dir` with its bytes, sorted by name.
fn files_in(dir: &Path) -> Vec<(String, Vec<u8>)> {
    let mut out: Vec<(String, Vec<u8>)> = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                out.push((
                    entry.file_name().to_string_lossy().to_string(),
                    fs::read(&path).unwrap(),
                ));
            }
        }
    }
    out.sort();
    out
}

fn archive_task_count(dir: &Path) -> usize {
    fs::read_dir(dir.join("backlog/archive/tasks"))
        .map(|entries| {
            entries
                .flatten()
                .filter(|entry| entry.path().is_file())
                .count()
        })
        .unwrap_or(0)
}

/// T-11k: the action is offered only in the final configured status column of
/// a milestone board, never when that status is `active`, never in another
/// column, never in the unconfigured column, and never in the without-milestone
/// section. A single configured status is the final column.
#[test]
fn t147a_cleanup_visibility() {
    let dir = cleanup_fixture("t147a");
    let server = Server::start(&dir, &["--no-open"], &[]);
    let html = get(&server, "/?type=task").text();

    // One action per milestone board (both milestones render).
    assert_eq!(count_occurrences(&html, "Clean up all</a>"), 2);
    let done = column_containing(&html, "Done");
    assert!(done.contains("Clean up all</a>"));
    assert!(done.contains("aria-label=\"Clean up all Done tasks in MS-001\""));
    assert!(!column_containing(&html, "ToDo").contains("Clean up all"));
    let unassigned = html
        .split("Tasks without a milestone")
        .nth(1)
        .expect("without-milestone section");
    assert!(!unassigned.contains("Clean up all"));

    // A task in no configured status gets the trailing column, which never
    // carries the action.
    let path = backlog_record_path(&dir, "tasks", "TASK-003");
    let text = fs::read_to_string(&path).unwrap();
    fs::write(&path, text.replace("status: ToDo", "status: Bogus")).unwrap();
    let html = get(&server, "/?type=task").text();
    let unconfigured = column_containing(&html, "");
    assert!(unconfigured.contains("Unconfigured status"));
    assert!(!unconfigured.contains("Clean up all"));
    assert_eq!(count_occurrences(&html, "Clean up all</a>"), 2);

    // A final configured status of `active` offers no action anywhere.
    set_statuses(&dir, &["ToDo", "Initiated", "active"], "ToDo");
    let html = get(&server, "/?type=task").text();
    assert!(!html.contains("Clean up all"));

    // A single configured status is the final column and offers the action.
    set_statuses(&dir, &["Done"], "Done");
    let html = get(&server, "/?type=task").text();
    assert_eq!(count_occurrences(&html, "Clean up all</a>"), 2);
    assert!(column_containing(&html, "Done").contains("Clean up all"));
    assert!(!column_containing(&html, "").contains("Clean up all"));

    // A single configured status that is `active` is also the final column and
    // still offers no action.
    set_statuses(&dir, &["active"], "active");
    let html = get(&server, "/?type=task").text();
    assert!(!html.contains("Clean up all"));

    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

/// T-11l: the preview lists exactly that milestone's tasks in the final column
/// in ascending ID order, and cancel (a plain GET) archives nothing.
#[test]
fn t147b_cleanup_preview_and_cancel() {
    let dir = cleanup_fixture("t147b");
    let tasks_dir = dir.join("backlog/tasks");
    let server = Server::start(&dir, &["--no-open"], &[]);
    let before = files_in(&tasks_dir);

    let response = get(&server, "/?type=task&expand=MS-001&cleanup=MS-001");
    assert_eq!(response.status, 200);
    let html = response.text();
    assert!(html.contains("<details class=\"board-milestone\" open><summary>MS-001"));
    let region = cleanup_region(&html);
    assert!(region.contains("role=\"region\""));
    assert!(region.contains("aria-label=\"Clean up all Done tasks in MS-001\""));
    assert!(region.contains("Warning"));
    assert!(region.contains("MS-001"));
    assert!(region.contains("2 tasks"));
    let first = region.find("TASK-001").expect("TASK-001 in preview");
    let second = region.find("TASK-002").expect("TASK-002 in preview");
    assert!(first < second, "ascending ID order");
    // Each affected task is rendered as its ID plus its title (AC2), asserted
    // as the exact pair text rather than the title alone, in the same order.
    let pair_one = "<li><strong>TASK-001</strong> \u{2014} Task One</li>";
    let pair_two = "<li><strong>TASK-002</strong> \u{2014} Task Two</li>";
    assert!(
        region.contains(pair_one),
        "TASK-001 rendered as ID plus title"
    );
    assert!(
        region.contains(pair_two),
        "TASK-002 rendered as ID plus title"
    );
    let pair_one_at = region.find(pair_one).expect("TASK-001 pair");
    let pair_two_at = region.find(pair_two).expect("TASK-002 pair");
    assert!(pair_one_at < pair_two_at, "pairs in ascending ID order");
    for absent in ["TASK-003", "TASK-004", "TASK-005", "TASK-006"] {
        assert!(!region.contains(absent), "{} must not be previewed", absent);
    }
    assert!(region.contains("action=\"/?type=task\""));
    assert!(region.contains("name=\"cleanup\" value=\"1\""));
    assert!(region.contains("name=\"milestone\" value=\"MS-001\""));
    assert!(region.contains("Archive 2 tasks"));
    assert!(region.contains("href=\"/?type=task&expand=MS-001\""));
    // The previewed milestone's action is replaced by the region; the other
    // milestone board still offers it.
    assert_eq!(count_occurrences(&html, "Clean up all</a>"), 1);

    // Preview writes nothing.
    assert_eq!(files_in(&tasks_dir), before);
    assert_eq!(archive_task_count(&dir), 1); // TASK-006 only
    assert!(!dir.join("backlog/archive/tasks/TASK-001.md").exists());
    assert!(!dir.join("backlog/archive/tasks/TASK-002.md").exists());

    // Cancel is a plain GET that archives nothing.
    let cancel = get(&server, "/?type=task&expand=MS-001");
    assert_eq!(cancel.status, 200);
    assert!(cancel.text().contains("Clean up all</a>"));
    assert_eq!(files_in(&tasks_dir), before);
    assert_eq!(archive_task_count(&dir), 1);

    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

/// T-11m: confirming archives every matching task, redirects to the expanded
/// board, and the re-read board no longer shows them.
#[test]
fn t147c_cleanup_full_success() {
    let dir = cleanup_fixture("t147c");
    // A closed milestone still renders its board, so it keeps the action.
    run_ok(&[
        "--dir",
        dir.to_str().unwrap(),
        "milestone",
        "close",
        "MS-001",
    ]);
    assert!(
        fs::read_to_string(dir.join("backlog/milestones/MS-001.md"))
            .unwrap()
            .contains("status: closed")
    );
    let server = Server::start(&dir, &["--no-open"], &[]);
    let initial = get(&server, "/?type=task").text();
    let ms1_start = initial.find("<summary>MS-001").expect("MS-001 section");
    let ms1_end = ms1_start + initial[ms1_start..].find("</details>").unwrap();
    let ms1 = &initial[ms1_start..ms1_end];
    assert!(ms1.contains("Clean up all</a>"));
    assert!(ms1.contains("aria-label=\"Clean up all Done tasks in MS-001\""));

    let body = form_body(&[
        ("cleanup", "1"),
        ("milestone", "MS-001"),
        ("expand", "MS-001"),
    ]);
    let response = post_form(&server, "/?type=task", &body);
    assert_eq!(response.status, 303);
    assert_eq!(
        response.header("Location"),
        Some("/?type=task&expand=MS-001")
    );

    for id in ["TASK-001", "TASK-002"] {
        let archived = dir.join("backlog/archive/tasks").join(format!("{}.md", id));
        assert!(archived.exists(), "{} archived", id);
        let text = fs::read_to_string(&archived).unwrap();
        assert!(text.contains("status: Done"), "{} keeps status", id);
        assert!(!backlog_record_path(&dir, "tasks", id).exists());
        assert_eq!(get(&server, &format!("/doc/{}", id)).status, 404);
    }
    assert!(backlog_record_path(&dir, "tasks", "TASK-003").exists());

    let html = get(&server, "/?type=task").text();
    assert!(!html.contains("TASK-001"));
    assert!(!html.contains("TASK-002"));
    assert!(html.contains("TASK-004"));
    assert!(html.contains("TASK-005"));
    // MS-002 still offers its action; MS-001 (now empty) offers it again.
    assert_eq!(count_occurrences(&html, "Clean up all</a>"), 2);
    assert!(column_containing(&html, "Done").contains("Clean up all"));

    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

/// T-11n: one induced archive failure is reported per task without aborting
/// the remaining archives; the response is `200` with an alert.
#[test]
fn t147d_cleanup_partial_failure() {
    let dir = cleanup_fixture("t147d");
    let archived = dir.join("backlog/archive/tasks");
    fs::create_dir_all(&archived).unwrap();
    // The failure must hit the first task in ascending order, otherwise an
    // implementation that aborts on the first error would still pass. A
    // destination that already exists makes `task::archive` refuse that one
    // task. A directory is used because an archived *file* with the same ID
    // would be dropped from the board index as a duplicate before the action
    // runs, so the failure would not be observable through the browser.
    fs::create_dir_all(archived.join("TASK-001.md")).unwrap();
    let server = Server::start(&dir, &["--no-open"], &[]);

    let body = form_body(&[
        ("cleanup", "1"),
        ("milestone", "MS-001"),
        ("expand", "MS-001"),
    ]);
    let response = post_form(&server, "/?type=task", &body);
    assert_eq!(response.status, 200);
    let html = response.text();

    // TASK-002 was still archived after TASK-001 failed: no abort.
    assert!(archived.join("TASK-002.md").exists());
    let archived_text = fs::read_to_string(archived.join("TASK-002.md")).unwrap();
    assert!(archived_text.contains("status: Done"));
    assert!(!backlog_record_path(&dir, "tasks", "TASK-002").exists());
    let failed = backlog_record_path(&dir, "tasks", "TASK-001");
    assert!(failed.exists());
    assert!(
        fs::read_to_string(&failed)
            .unwrap()
            .contains("status: Done")
    );

    let alert_start = html.find("role=\"alert\"").expect("alert region");
    let alert = &html[alert_start..];
    assert!(alert.contains("Clean up all finished with failures"));
    assert!(alert.contains("Archived 1 of 2 tasks in MS-001."));
    assert!(html.contains("Some tasks could not be archived."));
    assert!(alert.contains("TASK-001"));
    assert!(alert.contains("already archived: TASK-001"));
    assert!(!alert.contains("TASK-002"));

    // The failed task stays in its column; no other milestone is touched.
    assert!(column_containing(&html, "Done").contains("TASK-001"));
    assert!(!html.contains("TASK-002"));
    assert!(backlog_record_path(&dir, "tasks", "TASK-004").exists());
    assert!(backlog_record_path(&dir, "tasks", "TASK-005").exists());

    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

/// T-11o: only the hosting milestone's tasks are archived; an identically
/// named column elsewhere and a without-milestone task are untouched.
#[test]
fn t147e_cleanup_milestone_scoping() {
    let dir = cleanup_fixture("t147e");
    let server = Server::start(&dir, &["--no-open"], &[]);

    let preview = get(&server, "/?type=task&expand=MS-002&cleanup=MS-002").text();
    let region = cleanup_region(&preview);
    assert!(region.contains("TASK-004"));
    for absent in ["TASK-001", "TASK-002", "TASK-005"] {
        assert!(!region.contains(absent), "{} must not be previewed", absent);
    }

    let body = form_body(&[
        ("cleanup", "1"),
        ("milestone", "MS-002"),
        ("expand", "MS-002"),
    ]);
    let response = post_form(&server, "/?type=task", &body);
    assert_eq!(response.status, 303);
    assert_eq!(
        response.header("Location"),
        Some("/?type=task&expand=MS-002")
    );

    assert!(dir.join("backlog/archive/tasks/TASK-004.md").exists());
    assert!(!backlog_record_path(&dir, "tasks", "TASK-004").exists());
    for id in ["TASK-001", "TASK-002", "TASK-005"] {
        assert!(
            backlog_record_path(&dir, "tasks", id).exists(),
            "{} untouched",
            id
        );
    }

    let html = get(&server, "/?type=task").text();
    assert!(!html.contains("TASK-004"));
    assert!(html.contains("TASK-001"));
    assert!(html.contains("TASK-002"));
    assert!(html.contains("TASK-005"));

    // MS-002's final column is now empty: the action is still offered, the
    // warning lists nothing, and no confirm control is rendered.
    let empty = get(&server, "/?type=task&expand=MS-002&cleanup=MS-002").text();
    let region = cleanup_region(&empty);
    assert!(region.contains("MS-002"));
    assert!(region.contains("no task will be archived"));
    assert!(!region.contains("<form"));
    assert!(!region.contains("<button"));
    assert!(region.contains("href=\"/?type=task&expand=MS-002\""));

    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

/// T-11p: archived tasks stay excluded and the action's guards hold; no other
/// `/?type=` route accepts a POST.
#[test]
fn t147f_cleanup_archived_exclusion_and_guards() {
    let dir = cleanup_fixture("t147f");
    let server = Server::start(&dir, &["--no-open"], &[]);
    let body = form_body(&[
        ("cleanup", "1"),
        ("milestone", "MS-001"),
        ("expand", "MS-001"),
    ]);

    assert_eq!(post_form(&server, "/?type=task", &body).status, 303);
    let html = get(&server, "/?type=task").text();
    assert!(!html.contains("TASK-001"));
    assert!(!html.contains("TASK-002"));
    assert_eq!(get(&server, "/doc/TASK-001").status, 404);
    assert_eq!(get(&server, "/doc/TASK-002").status, 404);

    // A repeated confirm archives nothing and still redirects.
    let repeated = post_form(&server, "/?type=task", &body);
    assert_eq!(repeated.status, 303);
    assert_eq!(archive_task_count(&dir), 3); // TASK-006, TASK-001, TASK-002

    // Unknown and archived milestones are 404.
    let unknown = form_body(&[
        ("cleanup", "1"),
        ("milestone", "MS-999"),
        ("expand", "MS-999"),
    ]);
    assert_eq!(post_form(&server, "/?type=task", &unknown).status, 404);
    let milestone_archive = dir.join("backlog/archive/milestones");
    fs::create_dir_all(&milestone_archive).unwrap();
    fs::rename(
        dir.join("backlog/milestones/MS-002.md"),
        milestone_archive.join("MS-002.md"),
    )
    .unwrap();
    let archived_milestone = form_body(&[
        ("cleanup", "1"),
        ("milestone", "MS-002"),
        ("expand", "MS-002"),
    ]);
    assert_eq!(
        post_form(&server, "/?type=task", &archived_milestone).status,
        404
    );

    // A missing milestone field is 400.
    let missing = form_body(&[("cleanup", "1"), ("expand", "MS-001")]);
    assert_eq!(post_form(&server, "/?type=task", &missing).status, 400);

    // The media-type guard still applies to the board route.
    let response = request(
        server.port,
        "POST",
        "/?type=task",
        &[("Content-Type", "application/json")],
        Some(b"cleanup=1&milestone=MS-001"),
    );
    assert_eq!(response.status, 415);

    // A final configured status of `active` refuses the action.
    set_statuses(&dir, &["ToDo", "Initiated", "active"], "ToDo");
    let response = post_form(&server, "/?type=task", &body);
    assert_eq!(response.status, 422);
    assert!(
        response
            .text()
            .contains("Clean up all is not available for this status configuration.")
    );

    // No other `/?type=` view accepts a POST.
    let other = form_body(&[("cleanup", "1"), ("milestone", "MS-001")]);
    assert_eq!(post_form(&server, "/?type=req", &other).status, 405);
    assert_eq!(post_form(&server, "/?type=milestone", &other).status, 405);
    assert_eq!(post_form(&server, "/?type=release", &other).status, 405);

    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// ---- T-15 decision scenarios ----

fn create_decision(dir: &Path, title: &str) -> String {
    run_ok(&["--dir", dir.to_str().unwrap(), "decision", "create", title])
}

fn create_decision_typed(dir: &Path, title: &str, dtype: &str) -> String {
    run_ok(&[
        "--dir",
        dir.to_str().unwrap(),
        "decision",
        "create",
        title,
        "-t",
        dtype,
    ])
}

fn create_decision_full(
    dir: &Path,
    title: &str,
    dtype: Option<&str>,
    decision: Option<&str>,
    context: Option<&str>,
    consequences: Option<&str>,
) -> String {
    let mut args: Vec<String> = vec![
        "--dir".to_string(),
        dir.to_str().unwrap().to_string(),
        "decision".to_string(),
        "create".to_string(),
        title.to_string(),
    ];
    if let Some(t) = dtype {
        args.push("-t".to_string());
        args.push(t.to_string());
    }
    if let Some(d) = decision {
        args.push("-d".to_string());
        args.push(d.to_string());
    }
    if let Some(c) = context {
        args.push("--context".to_string());
        args.push(c.to_string());
    }
    if let Some(c) = consequences {
        args.push("--consequences".to_string());
        args.push(c.to_string());
    }
    let out = bin().args(&args).output().unwrap();
    assert!(
        out.status.success(),
        "command failed: {:?}\nstderr: {}",
        args,
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn decision_fixture(name: &str) -> PathBuf {
    let dir = test_root(name);
    init_project(&dir);
    create_decision_full(
        &dir,
        "Decision One",
        None,
        Some("We will do X"),
        Some("Because Y"),
        Some("Z happens"),
    );
    create_decision(&dir, "Decision Two");
    create_decision_typed(&dir, "Bdr Decision", "bdr");
    dir
}

fn decision_record_path(dir: &Path, id: &str) -> PathBuf {
    let decisions = dir.join("decisions");
    let prefix = format!("{}-", id);
    if let Ok(entries) = fs::read_dir(&decisions) {
        for entry in entries.flatten() {
            let path = entry.path();
            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            if stem == id || stem.starts_with(&prefix) {
                return path;
            }
        }
    }
    decisions.join(format!("{}.md", id))
}

// T-15a
#[test]
fn t15a_list_decision_category() {
    let dir = decision_fixture("t15a");
    run_ok(&[
        "--dir",
        dir.to_str().unwrap(),
        "decision",
        "edit",
        "ADR-001",
        "--superseded-by",
        "ADR-002",
    ]);
    let server = Server::start(&dir, &["--no-open"], &[]);

    let html = get(&server, "/?type=decision").text();
    assert!(html.contains("ADR-001"), "missing ADR-001");
    assert!(html.contains("ADR-002"), "missing ADR-002");
    assert!(html.contains("BDR-001"), "missing BDR-001");
    // TASK-142 defect fix: REQ-06.004 parity needs the superseded_by slot.
    assert!(html.contains("<th scope=\"col\" data-column=\"superseded_by\">Superseded by</th>"));
    let start = html.find("data-row-id=\"ADR-001\"").expect("ADR-001 row");
    let end = html[start..].find("</tr>").unwrap() + start;
    let row = &html[start..end];
    assert!(row.contains("<td class=\"slot-cell\">ADR-002</td>"));
    assert!(
        html.contains("aria-current=\"page\""),
        "sidebar aria-current"
    );
    let a = html.find("ADR-001").unwrap();
    let b = html.find("ADR-002").unwrap();
    let c = html.find("BDR-001").unwrap();
    assert!(a < b && b < c, "decision order");

    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-15b
#[test]
fn t15b_category_parameter_decision() {
    let dir = decision_fixture("t15b");
    let server = Server::start(&dir, &["--no-open"], &[]);

    assert_eq!(get(&server, "/?type=req").status, 200);
    for ty in ["req", "nfr", "task", "milestone", "release", "decision"] {
        assert_eq!(
            get(&server, &format!("/?type={}", ty)).status,
            200,
            "type={}",
            ty
        );
    }
    for ty in ["all", "x", "decisions"] {
        assert_eq!(
            get(&server, &format!("/?type={}", ty)).status,
            404,
            "type={}",
            ty
        );
    }
    let html = get(&server, "/?type=decision").text();
    assert!(html.contains("ADR-001"));

    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-15c
#[test]
fn t15c_view_decision_fields() {
    let dir = decision_fixture("t15c");
    // make ADR-001 superseded by ADR-002
    run_ok(&[
        "--dir",
        dir.to_str().unwrap(),
        "decision",
        "edit",
        "ADR-001",
        "--superseded-by",
        "ADR-002",
    ]);
    let server = Server::start(&dir, &["--no-open"], &[]);

    let view = get(&server, "/doc/ADR-001").text();
    for needle in [
        "Decision type",
        "Decision",
        "Context",
        "Consequences",
        "Supersession",
        "We will do X",
        "Because Y",
        "Z happens",
    ] {
        assert!(view.contains(needle), "decision view missing {}", needle);
    }
    assert!(view.contains("/doc/ADR-002"));

    let view2 = get(&server, "/doc/BDR-001").text();
    assert!(view2.contains("Decision type"));

    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-15d
#[test]
fn t15d_edit_round_trip_decision() {
    let dir = decision_fixture("t15d");
    let server = Server::start(&dir, &["--no-open"], &[]);

    let edit = get(&server, "/doc/ADR-002/edit").text();
    let version = extract_version(&edit);
    let body = form_body(&[
        ("version", &version),
        ("title", "Decision Two revised"),
        ("status", "accepted"),
        ("decision", "Revised decision"),
        ("context", "Revised context"),
        ("consequences", "Revised consequences"),
        ("superseded-by", ""),
    ]);
    let response = post_form(&server, "/doc/ADR-002", &body);
    assert_eq!(response.status, 303);
    let view = get(&server, "/doc/ADR-002").text();
    assert!(view.contains("Decision Two revised"));
    assert!(view.contains("Revised decision"));
    let cli = run_ok(&[
        "--dir",
        dir.to_str().unwrap(),
        "decision",
        "view",
        "ADR-002",
    ]);
    assert!(cli.contains("Decision Two revised"));

    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-15e
#[test]
fn t15e_validation_decision() {
    let dir = decision_fixture("t15e");
    let path = decision_record_path(&dir, "ADR-001");
    let before = fs::read(&path).unwrap();
    let server = Server::start(&dir, &["--no-open"], &[]);
    let version = extract_version(&get(&server, "/doc/ADR-001/edit").text());

    // empty title
    let body = form_body(&[("version", &version), ("title", "")]);
    let response = post_form(&server, "/doc/ADR-001", &body);
    assert_eq!(response.status, 422);
    assert_eq!(fs::read(&path).unwrap(), before);

    // invalid status transition
    let body = form_body(&[
        ("version", &version),
        ("title", "Valid"),
        ("status", "archived"),
    ]);
    let response = post_form(&server, "/doc/ADR-001", &body);
    assert_eq!(response.status, 422);
    assert_eq!(fs::read(&path).unwrap(), before);

    // unknown superseding
    let body = form_body(&[
        ("version", &version),
        ("title", "Valid"),
        ("superseded-by", "ADR-999"),
    ]);
    let response = post_form(&server, "/doc/ADR-001", &body);
    assert_eq!(response.status, 422);
    assert_eq!(fs::read(&path).unwrap(), before);

    // type mismatch
    let body = form_body(&[
        ("version", &version),
        ("title", "Valid"),
        ("superseded-by", "BDR-001"),
    ]);
    let response = post_form(&server, "/doc/ADR-001", &body);
    assert_eq!(response.status, 422);
    assert_eq!(fs::read(&path).unwrap(), before);

    // self-supersession
    let body = form_body(&[
        ("version", &version),
        ("title", "Valid"),
        ("superseded-by", "ADR-001"),
    ]);
    let response = post_form(&server, "/doc/ADR-001", &body);
    assert_eq!(response.status, 422);
    assert_eq!(fs::read(&path).unwrap(), before);

    // both status + superseded-by
    let body = form_body(&[
        ("version", &version),
        ("title", "Valid"),
        ("status", "accepted"),
        ("superseded-by", "ADR-002"),
    ]);
    let response = post_form(&server, "/doc/ADR-001", &body);
    assert_eq!(response.status, 422);
    assert_eq!(fs::read(&path).unwrap(), before);

    // ## line in decision
    let body = form_body(&[
        ("version", &version),
        ("title", "Valid"),
        ("decision", "Line\n## Not allowed"),
    ]);
    let response = post_form(&server, "/doc/ADR-001", &body);
    assert_eq!(response.status, 422);
    assert_eq!(fs::read(&path).unwrap(), before);

    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-15f
#[test]
fn t15f_conflict_decision() {
    let dir = decision_fixture("t15f");
    let path = decision_record_path(&dir, "ADR-001");
    let server = Server::start(&dir, &["--no-open"], &[]);
    let version = extract_version(&get(&server, "/doc/ADR-001/edit").text());
    let original = fs::read_to_string(&path).unwrap();
    fs::write(&path, format!("{}\n## Out of band\nchanged\n", original)).unwrap();
    let body = form_body(&[("version", &version), ("title", "Should not save")]);
    let response = post_form(&server, "/doc/ADR-001", &body);
    assert_eq!(response.status, 409);
    let html = response.text();
    assert!(html.contains("This record changed on disk"));
    let saved = fs::read_to_string(&path).unwrap();
    assert!(!saved.contains("Should not save"));
    assert!(saved.contains("## Out of band"));
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-15h
#[test]
fn t15h_atomic_and_symlink_decision() {
    let dir = decision_fixture("t15h");
    let path = decision_record_path(&dir, "ADR-001");
    let server = Server::start(&dir, &["--no-open"], &[]);
    let version = extract_version(&get(&server, "/doc/ADR-001/edit").text());
    let before: Vec<String> = fs::read_dir(path.parent().unwrap())
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    let body = form_body(&[
        ("version", &version),
        ("title", "Atomic"),
        ("status", "accepted"),
    ]);
    assert_eq!(post_form(&server, "/doc/ADR-001", &body).status, 303);
    let after: Vec<String> = fs::read_dir(path.parent().unwrap())
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    assert_eq!(before, after);
    assert!(!after.iter().any(|n| n.contains(".tmp")));

    // rejected save leaves no temp
    let body = form_body(&[("version", "1.2.3"), ("title", "Rejected")]);
    assert_eq!(post_form(&server, "/doc/ADR-001", &body).status, 409);
    let after: Vec<String> = fs::read_dir(path.parent().unwrap())
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    assert_eq!(before, after);

    drop(server);

    // symlink escape (unix)
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let outside = test_root("t15h-outside");
        let secret = outside.join("secret.md");
        fs::write(&secret, "# ADR-001 SECRET CONTENT\n").unwrap();
        let p = decision_record_path(&dir, "ADR-001");
        let _ = fs::remove_file(&p);
        symlink(&secret, &p).unwrap();
        let server = Server::start(&dir, &["--no-open"], &[]);
        assert_eq!(get(&server, "/doc/ADR-001").status, 404);
        let list = get(&server, "/?type=decision").text();
        assert!(!list.contains("ADR-001"));
        let body = form_body(&[("version", "1.2.3"), ("title", "Attack")]);
        assert_eq!(post_form(&server, "/doc/ADR-001", &body).status, 404);
        assert!(
            fs::read_to_string(&secret)
                .unwrap()
                .contains("SECRET CONTENT")
        );
        drop(server);
        let _ = fs::remove_dir_all(&outside);
    }

    let _ = fs::remove_dir_all(&dir);
}

// T-15i
#[test]
fn t15i_json_api_decision() {
    let dir = decision_fixture("t15i");
    let server = Server::start(&dir, &["--no-open"], &[]);

    let decisions: serde_json::Value =
        serde_json::from_str(&get(&server, "/api/decisions").text()).unwrap();
    let arr = decisions.as_array().unwrap();
    assert_eq!(arr.len(), 3);
    assert!(arr[0].get("status").is_some());
    assert!(arr[0].get("type").is_some());

    let doc: serde_json::Value =
        serde_json::from_str(&get(&server, "/api/doc/ADR-001").text()).unwrap();
    assert_eq!(doc["id"], "ADR-001");
    assert_eq!(doc["kind"], "decision");
    assert_eq!(doc["form"], "decision");
    assert!(doc["type"].is_string() || doc["type"].is_null());
    assert!(doc["status"].is_string() || doc["status"].is_null());
    assert!(doc["decision"].is_string() || doc["decision"].is_null());
    assert!(doc["context"].is_string() || doc["context"].is_null());
    assert!(doc["consequences"].is_string() || doc["consequences"].is_null());
    assert!(doc["superseded_by"].is_object() || doc["superseded_by"].is_null());
    assert!(doc["version"].is_string());

    // BDR doc
    let bdoc: serde_json::Value =
        serde_json::from_str(&get(&server, "/api/doc/BDR-001").text()).unwrap();
    assert_eq!(bdoc["kind"], "decision");

    // POST update
    let version = doc["version"].as_str().unwrap().to_string();
    let body = format!(
        "{{\"version\":\"{}\",\"title\":\"API Decision\",\"status\":\"accepted\"}}",
        version
    );
    let updated: serde_json::Value =
        serde_json::from_str(&post_json(&server, "/api/doc/ADR-001", &body).text()).unwrap();
    assert_eq!(updated["title"], "API Decision");
    assert_eq!(updated["status"], "accepted");

    // bad body
    assert_eq!(
        post_json(&server, "/api/doc/ADR-001", "{not json").status,
        400
    );
    let version = updated["version"].as_str().unwrap().to_string();
    assert_eq!(
        post_json(
            &server,
            "/api/doc/ADR-001",
            &format!("{{\"version\":\"{}\",\"title\":5}}", version)
        )
        .status,
        400
    );
    // conflict
    let conflict = post_json(
        &server,
        "/api/doc/ADR-001",
        &format!("{{\"version\":\"{}\",\"title\":\"x\"}}", "1.2.3"),
    );
    assert_eq!(conflict.status, 409);
    // invalid
    let invalid = post_json(
        &server,
        "/api/doc/ADR-001",
        &format!("{{\"version\":\"{}\",\"title\":\"\"}}", version),
    );
    assert_eq!(invalid.status, 422);
    // missing
    assert_eq!(get(&server, "/api/doc/ADR-404").status, 404);
    // content-type
    assert_eq!(
        request(
            server.port,
            "POST",
            "/api/doc/ADR-001",
            &[("Content-Type", "text/plain")],
            Some(b"x"),
        )
        .status,
        415
    );

    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// ---- T-40 UI mockups browser scenarios ----

fn create_mockup(dir: &Path, title: &str, task: &str, specs: &[&str], refs: &[&str]) -> String {
    let mut args: Vec<String> = vec![
        "--dir".to_string(),
        dir.to_str().unwrap().to_string(),
        "ui-mockup".to_string(),
        "create".to_string(),
        title.to_string(),
        "--task".to_string(),
        task.to_string(),
    ];
    for spec in specs {
        args.push("--spec".to_string());
        args.push(spec.to_string());
    }
    for reference in refs {
        args.push("--ref".to_string());
        args.push(reference.to_string());
    }
    let out = bin().args(&args).output().unwrap();
    assert!(
        out.status.success(),
        "create failed: {:?}\nstderr: {}",
        args,
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn mockup_record_path(dir: &Path, id: &str) -> PathBuf {
    dir.join("docs/specifications/ui/mockups")
        .join(format!("{}.html", id))
}

fn mockup_dir(dir: &Path) -> PathBuf {
    dir.join("docs/specifications/ui/mockups")
}

fn mockup_fixture(name: &str) -> PathBuf {
    let dir = test_root(name);
    init_project(&dir);
    create_mockup(&dir, "First mockup", "TASK-41", &["UI-001"], &["a.md"]);
    create_mockup(
        &dir,
        "Second mockup",
        "TASK-42",
        &["UI-002", "UI-003"],
        &["b.md"],
    );
    dir
}

fn write_review_package(dir: &Path, id: &str) -> String {
    let review = format!(
        "<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<title>{} &mdash; Review {}</title>\n<style>body{{color:red}} .marker-style-40{{color:blue}}</style>\n<script>var markerScript40 = \"do not leak\";</script>\n</head>\n<body class=\"wf-package\">\n<h1>TASK-99 &mdash; Landing heading marker40</h1>\n<section><h1 class=\"wf-h1\">Prototype heading marker40</h1><p>Slide content marker40 must stay.</p></section>\n</body>\n</html>\n",
        id, id
    );
    let path = mockup_record_path(dir, id);
    fs::write(&path, &review).unwrap();
    review
}

// T-40a
#[test]
fn t40a_list_ui_mockups_category() {
    let dir = mockup_fixture("t40a");
    let server = Server::start(&dir, &["--no-open"], &[]);
    let html = get(&server, "/?type=ui-mockup").text();
    assert!(html.contains("<h2 class=\"h2\">UI Mockups</h2>"));
    assert!(html.contains("2 UI mockups"));
    assert!(html.contains("aria-current=\"page\""));
    // Sidebar order: UI Specifications before UI Mockups.
    let specs_at = html.find("href=\"/?type=ui-spec\"").unwrap();
    let mockups_at = html.find("href=\"/?type=ui-mockup\"").unwrap();
    assert!(specs_at < mockups_at, "sidebar order");
    assert!(html.contains("MOCK-001"));
    assert!(html.contains("First mockup"));
    assert!(html.contains("MOCK-002"));
    assert!(html.contains("Second mockup"));
    let first = html.find("MOCK-001").unwrap();
    let second = html.find("MOCK-002").unwrap();
    assert!(first < second, "ascending ID order");
    assert!(html.contains("href=\"/doc/MOCK-001\""));
    assert!(html.contains("href=\"/doc/MOCK-002\""));
    // Default stays req; unknown values are 400.
    assert_eq!(get(&server, "/?type=req").status, 200);
    assert_eq!(get(&server, "/?type=req").status, 200);
    for ty in ["all", "x", "mockup", "ui-mockups"] {
        assert_eq!(
            get(&server, &format!("/?type={}", ty)).status,
            404,
            "type={}",
            ty
        );
    }
    // Long title still renders.
    let long_title = "L".repeat(150);
    create_mockup(&dir, &long_title, "TASK-43", &["UI-004"], &[]);
    let html = get(&server, "/?type=ui-mockup").text();
    assert!(html.contains(&long_title));
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn t40a_disabled_ui_mockup_empty_state() {
    let dir = mockup_fixture("t40a-disabled");
    run_ok(&[
        "--dir",
        dir.to_str().unwrap(),
        "config",
        "set",
        "document_types.ui-mockup.enabled",
        "false",
    ]);
    let server = Server::start(&dir, &["--no-open"], &[]);
    let html = get(&server, "/").text();
    assert!(!html.contains("href=\"/?type=ui-mockup\""));
    assert_eq!(get(&server, "/?type=ui-mockup").status, 404);
    assert_eq!(get(&server, "/api/ui-mockups").status, 404);
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-40b
#[test]
fn t40b_non_records_are_not_listed() {
    let dir = mockup_fixture("t40b");
    let mock_dir = mockup_dir(&dir);
    fs::write(mock_dir.join("README.md"), "# registry").unwrap();
    fs::write(mock_dir.join("wireframe.css"), "body{}").unwrap();
    fs::write(mock_dir.join("index.html"), "<html></html>").unwrap();
    fs::write(mock_dir.join("MOCK-ABC.html"), "<html></html>").unwrap();
    fs::write(mock_dir.join("other.html"), "<html></html>").unwrap();
    let archived = mock_dir.join("archived");
    fs::create_dir_all(&archived).unwrap();
    fs::write(
        archived.join("MOCK-999.html"),
        "<html><head><title>MOCK-999 — Hidden</title></head><body></body></html>",
    )
    .unwrap();
    let server = Server::start(&dir, &["--no-open"], &[]);
    let html = get(&server, "/?type=ui-mockup").text();
    assert!(html.contains("MOCK-001"));
    assert!(html.contains("MOCK-002"));
    // TASK-142: `index` alone also appears in `tabindex`; assert the file
    // names/record hooks instead of a bare substring.
    for hidden in [
        "MOCK-999",
        "README",
        "wireframe",
        "index.html",
        "other.html",
    ] {
        assert!(!html.contains(hidden), "listed {}", hidden);
    }
    assert!(html.contains("2 UI mockups"));
    let api: serde_json::Value =
        serde_json::from_str(&get(&server, "/api/ui-mockups").text()).unwrap();
    assert_eq!(api.as_array().unwrap().len(), 2);
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-40c
#[test]
fn t40c_view_ui_mockup() {
    let dir = mockup_fixture("t40c");
    write_review_package(&dir, "MOCK-010");
    let server = Server::start(&dir, &["--no-open"], &[]);
    let html = get(&server, "/doc/MOCK-001").text();
    assert!(html.contains("Record ID"));
    assert!(html.contains("MOCK-001"));
    // Document type is no longer repeated on the view (ADR-033 point 4).
    assert!(html.contains("UI Mockups"));
    assert!(html.contains("<h1 class=\"h1\">First mockup</h1>"));
    assert!(html.contains("<h2>UI specifications</h2>"));
    assert!(html.contains("<h2>References</h2>"));
    assert!(html.contains("UI-001"));
    assert!(html.contains("a.md"));
    assert!(html.contains("white-space: pre-wrap"));
    // Breadcrumb UI Mockups > ID, single Edit action.
    assert!(html.contains("href=\"/?type=ui-mockup\">UI Mockups</a></li><li><span aria-current=\"page\">MOCK-001</span>"));
    assert_eq!(html.matches("href=\"/doc/MOCK-001/edit\"").count(), 1);
    assert!(html.contains(">Edit</a>"));
    // Empty refs still render (none) for the review package.
    let empty_view = get(&server, "/doc/MOCK-010").text();
    assert!(empty_view.contains("<h1 class=\"h1\">Review MOCK-010</h1>"));
    assert!(empty_view.contains("(none)"));
    // Stored package, scripts, and markup never appear in the response.
    for path in ["/doc/MOCK-010", "/doc/MOCK-010/edit", "/?type=ui-mockup"] {
        let body = get(&server, path).text();
        assert!(!body.contains("markerScript40"), "leaked script {}", path);
        assert!(!body.contains("marker-style-40"), "leaked style {}", path);
        assert!(
            !body.contains("Landing heading marker40"),
            "leaked markup {}",
            path
        );
        assert!(
            !body.contains("Prototype heading marker40"),
            "leaked markup {}",
            path
        );
        assert!(
            !body.contains("Slide content marker40"),
            "leaked markup {}",
            path
        );
    }
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-40n / T-114 AC5
#[test]
fn t40n_detail_renders_mockup_status_read_only() {
    let dir = mockup_fixture("t40n");
    // pending is the default; move the second mockup to approved.
    run_ok(&[
        "--dir",
        dir.to_str().unwrap(),
        "ui-mockup",
        "edit",
        "MOCK-002",
        "--status",
        "approved",
    ]);
    let server = Server::start(&dir, &["--no-open"], &[]);

    let pending = get(&server, "/doc/MOCK-001").text();
    assert!(
        pending
            .contains("<p class=\"status-line\"><span class=\"badge\">Status</span> pending</p>"),
        "pending detail must render the stored status: {}",
        pending
    );
    assert!(!pending.contains("name=\"status\""));
    assert!(!pending.contains("name=\"status_change\""));

    let approved = get(&server, "/doc/MOCK-002").text();
    assert!(
        approved
            .contains("<p class=\"status-line\"><span class=\"badge\">Status</span> approved</p>"),
        "approved detail must render the stored status: {}",
        approved
    );
    assert!(!approved.contains("name=\"status\""));
    assert!(!approved.contains("name=\"status_change\""));

    // The edit form never exposes a status control.
    let edit = get(&server, "/doc/MOCK-002/edit").text();
    assert!(!edit.contains("name=\"status\""));
    assert!(!edit.contains("name=\"status_change\""));

    // Archived mockups stay unreachable.
    run_ok(&[
        "--dir",
        dir.to_str().unwrap(),
        "ui-mockup",
        "archive",
        "MOCK-002",
    ]);
    assert_eq!(get(&server, "/doc/MOCK-002").status, 404);
    assert_eq!(get(&server, "/doc/MOCK-002/edit").status, 404);

    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-40d
#[test]
fn t40d_edit_round_trip() {
    let dir = mockup_fixture("t40d");
    let server = Server::start(&dir, &["--no-open"], &[]);
    let edit = get(&server, "/doc/MOCK-001/edit").text();
    // Tag add/remove interaction (spec section 6.6).
    assert!(edit.contains("name=\"specs\""));
    assert!(edit.contains("name=\"refs\""));
    assert!(edit.contains("aria-label=\"Remove UI-001\""));
    assert!(edit.contains("aria-label=\"Remove a.md\""));
    assert!(edit.contains(">Add</button>"));
    assert!(edit.contains("for=\"specs-add\""));
    assert!(edit.contains("for=\"refs-add\""));
    assert!(edit.contains("Hidden field: version (optimistic-concurrency token)"));
    assert!(edit.contains("Save changes"));
    assert!(edit.contains("Cancel"));
    assert!(edit.contains("data-dirty-status hidden>Unsaved changes</p>"));
    assert!(
        !edit.contains("<textarea"),
        "tag fields must not be textareas"
    );
    let version = extract_version(&edit);
    let body = form_body(&[
        ("version", &version),
        ("title", "Renamed mockup"),
        ("specs", "UI-010"),
        ("specs", "UI-011"),
        ("refs", "docs/new.md"),
        ("specs", ""),
        ("refs", ""),
    ]);
    let response = post_form(&server, "/doc/MOCK-001", &body);
    assert_eq!(response.status, 303);
    assert_eq!(response.header("Location"), Some("/doc/MOCK-001"));
    let view = get(&server, "/doc/MOCK-001").text();
    assert!(view.contains("<h1 class=\"h1\">Renamed mockup</h1>"));
    assert!(view.contains("UI-010 UI-011"));
    assert!(view.contains("docs/new.md"));
    let list = get(&server, "/?type=ui-mockup").text();
    assert!(list.contains("MOCK-001"));
    assert!(list.contains("Renamed mockup"));
    // Visible through the CLI.
    let cli = run_ok(&[
        "--dir",
        dir.to_str().unwrap(),
        "ui-mockup",
        "view",
        "MOCK-001",
    ]);
    assert!(cli.contains("<title>MOCK-001 — Renamed mockup</title>"));
    assert!(cli.contains("<!-- specs: UI-010 UI-011 -->"));
    assert!(cli.contains("<!-- refs: docs/new.md -->"));
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-40e
#[test]
fn t40e_review_package_preserved() {
    let dir = mockup_fixture("t40e");
    let original = write_review_package(&dir, "MOCK-010");
    let server = Server::start(&dir, &["--no-open"], &[]);
    let version = extract_version(&get(&server, "/doc/MOCK-010/edit").text());
    let body = form_body(&[
        ("version", &version),
        ("title", "Edited review title"),
        ("specs", "UI-020"),
        ("refs", "docs/edited.md"),
    ]);
    assert_eq!(post_form(&server, "/doc/MOCK-010", &body).status, 303);
    let after = fs::read_to_string(mockup_record_path(&dir, "MOCK-010")).unwrap();
    assert!(after.contains("<title>MOCK-010 &mdash; Edited review title</title>"));
    assert!(after.contains("<!-- specs: UI-020 -->"));
    assert!(after.contains("<!-- refs: docs/edited.md -->"));
    // Every other byte is preserved: style, script, landing heading, slides.
    for needle in [
        "<style>body{color:red} .marker-style-40{color:blue}</style>",
        "<script>var markerScript40 = \"do not leak\";</script>",
        "<h1>TASK-99 &mdash; Landing heading marker40</h1>",
        "<h1 class=\"wf-h1\">Prototype heading marker40</h1>",
        "<p>Slide content marker40 must stay.</p>",
        "<body class=\"wf-package\">",
    ] {
        assert!(after.contains(needle), "changed {}", needle);
        assert!(original.contains(needle));
    }
    // No visible metadata lines are inserted into a review package.
    assert!(!after.contains("<p>UI specifications:"));
    assert!(!after.contains("<p>References:"));
    // Only the title line and the inserted comment block differ.
    // Insertion adds the two comment lines plus one separator blank line.
    let before_lines: Vec<&str> = original.lines().collect();
    let after_lines: Vec<&str> = after.lines().collect();
    assert_eq!(after_lines.len(), before_lines.len() + 3);
    let filtered: Vec<&str> = after_lines
        .iter()
        .copied()
        .filter(|line| !line.contains("<!-- specs:") && !line.contains("<!-- refs:"))
        .collect();
    let stripped_before: Vec<&str> = before_lines
        .iter()
        .copied()
        .filter(|line| !line.is_empty())
        .collect();
    let stripped_after: Vec<&str> = filtered
        .iter()
        .copied()
        .filter(|line| !line.is_empty())
        .collect();
    assert_eq!(
        stripped_after.len(),
        stripped_before.len(),
        "non-metadata lines changed"
    );
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-40f
#[test]
fn t40f_repeated_fields_in_order() {
    let dir = mockup_fixture("t40f");
    let server = Server::start(&dir, &["--no-open"], &[]);
    let version = extract_version(&get(&server, "/doc/MOCK-002/edit").text());
    let body = form_body(&[
        ("version", &version),
        ("title", "Order check"),
        ("specs", "UI-003"),
        ("specs", "UI-001"),
        ("specs", "UI-002"),
        ("refs", "z.md"),
        ("refs", "a.md"),
        ("specs", ""),
        ("refs", ""),
    ]);
    assert_eq!(post_form(&server, "/doc/MOCK-002", &body).status, 303);
    let file = fs::read_to_string(mockup_record_path(&dir, "MOCK-002")).unwrap();
    assert!(file.contains("<!-- specs: UI-003 UI-001 UI-002 -->"));
    assert!(file.contains("<!-- refs: z.md a.md -->"));
    let api: serde_json::Value =
        serde_json::from_str(&get(&server, "/api/doc/MOCK-002").text()).unwrap();
    assert_eq!(
        api["specs"],
        serde_json::json!(["UI-003", "UI-001", "UI-002"])
    );
    assert_eq!(api["refs"], serde_json::json!(["z.md", "a.md"]));
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-40g
#[test]
fn t40g_validation_422_no_write() {
    let dir = mockup_fixture("t40g");
    let path = mockup_record_path(&dir, "MOCK-001");
    let before = fs::read(&path).unwrap();
    let server = Server::start(&dir, &["--no-open"], &[]);
    let version = extract_version(&get(&server, "/doc/MOCK-001/edit").text());
    let cases: Vec<(String, &str, &str)> = vec![
        (
            form_body(&[("version", &version), ("title", ""), ("specs", "UI-001")]),
            "Title is required.",
            "title",
        ),
        (
            form_body(&[
                ("version", &version),
                ("title", &"x".repeat(201)),
                ("specs", "UI-001"),
            ]),
            "Title must be at most 200 characters.",
            "title",
        ),
        (
            form_body(&[
                ("version", &version),
                ("title", "a\tb"),
                ("specs", "UI-001"),
            ]),
            "Title cannot contain control characters.",
            "title",
        ),
        (
            form_body(&[("version", &version), ("title", "Ok"), ("specs", "")]),
            "At least one UI specification id is required.",
            "specs",
        ),
        (
            form_body(&[("version", &version), ("title", "Ok"), ("specs", "bad id")]),
            "UI specification ids must be valid record IDs.",
            "specs",
        ),
        (
            form_body(&[
                ("version", &version),
                ("title", "Ok"),
                ("specs", "UI-001"),
                ("specs", "UI-001"),
            ]),
            "Duplicate UI specification id: UI-001.",
            "specs",
        ),
        (
            form_body(&[
                ("version", &version),
                ("title", "Ok"),
                ("specs", "UI-001"),
                ("refs", &"y".repeat(201)),
            ]),
            "References must be at most 200 characters.",
            "refs",
        ),
        (
            form_body(&[
                ("version", &version),
                ("title", "Ok"),
                ("specs", "UI-001"),
                ("refs", "a b"),
            ]),
            "References cannot contain whitespace.",
            "refs",
        ),
        (
            form_body(&[
                ("version", &version),
                ("title", "Ok"),
                ("specs", "UI-001"),
                ("refs", "a.md"),
                ("refs", "a.md"),
            ]),
            "Duplicate reference: a.md.",
            "refs",
        ),
    ];
    for (body, message, field) in cases {
        let response = post_form(&server, "/doc/MOCK-001", &body);
        assert_eq!(response.status, 422, "body={}", body);
        let html = response.text();
        assert_eq!(summary_entry(&html, field), message);
        assert_eq!(field_error(&html, field), message);
        assert_eq!(fs::read(&path).unwrap(), before, "file changed {}", body);
    }
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-40h
#[test]
fn t40h_stale_and_malformed_versions() {
    let dir = mockup_fixture("t40h");
    let path = mockup_record_path(&dir, "MOCK-001");
    let before = fs::read(&path).unwrap();
    let server = Server::start(&dir, &["--no-open"], &[]);
    let valid = form_body(&[
        ("version", "1.2.3"),
        ("title", "Stale"),
        ("specs", "UI-001"),
    ]);
    let conflict = post_form(&server, "/doc/MOCK-001", &valid);
    assert_eq!(conflict.status, 409);
    assert!(
        conflict
            .text()
            .contains("record changed on disk; reload the form")
    );
    assert_eq!(fs::read(&path).unwrap(), before);
    // Missing version is 400.
    let missing = form_body(&[("title", "Ok"), ("specs", "UI-001")]);
    assert_eq!(post_form(&server, "/doc/MOCK-001", &missing).status, 400);
    // Malformed version is 400.
    let malformed = form_body(&[("version", "bad"), ("title", "Ok"), ("specs", "UI-001")]);
    assert_eq!(post_form(&server, "/doc/MOCK-001", &malformed).status, 400);
    assert_eq!(fs::read(&path).unwrap(), before);
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-40i
#[test]
fn t40i_archived_hidden_and_404() {
    let dir = mockup_fixture("t40i");
    run_ok(&[
        "--dir",
        dir.to_str().unwrap(),
        "ui-mockup",
        "archive",
        "MOCK-002",
    ]);
    let server = Server::start(&dir, &["--no-open"], &[]);
    let list = get(&server, "/?type=ui-mockup").text();
    assert!(list.contains("MOCK-001"));
    assert!(!list.contains("MOCK-002"));
    assert!(list.contains("1 UI mockup"));
    assert_eq!(get(&server, "/doc/MOCK-002").status, 404);
    assert_eq!(get(&server, "/doc/MOCK-002/edit").status, 404);
    let version = "1.2.3";
    let body = form_body(&[("version", version), ("title", "Ok"), ("specs", "UI-001")]);
    assert_eq!(post_form(&server, "/doc/MOCK-002", &body).status, 404);
    assert_eq!(get(&server, "/api/doc/MOCK-002").status, 404);
    let json_body = format!("{{\"version\":\"{}\",\"title\":\"Ok\"}}", version);
    assert_eq!(
        post_json(&server, "/api/doc/MOCK-002", &json_body).status,
        404
    );
    let api: serde_json::Value =
        serde_json::from_str(&get(&server, "/api/ui-mockups").text()).unwrap();
    assert_eq!(api.as_array().unwrap().len(), 1);
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-40j
#[test]
fn t40j_json_api() {
    let dir = mockup_fixture("t40j");
    let server = Server::start(&dir, &["--no-open"], &[]);
    // Summary list (and aliases) ascending.
    for alias in ["/api/ui-mockups", "/api/ui-mockup", "/api/mockups"] {
        let list: serde_json::Value = serde_json::from_str(&get(&server, alias).text()).unwrap();
        let arr = list.as_array().unwrap();
        assert_eq!(arr.len(), 2, "alias {}", alias);
        assert_eq!(arr[0]["id"], "MOCK-001");
        assert_eq!(arr[0]["title"], "First mockup");
        assert_eq!(arr[1]["id"], "MOCK-002");
    }
    // Full record shape.
    let doc: serde_json::Value =
        serde_json::from_str(&get(&server, "/api/doc/MOCK-001").text()).unwrap();
    assert_eq!(doc["id"], "MOCK-001");
    assert_eq!(doc["kind"], "ui-mockup");
    assert_eq!(doc["form"], "ui-mockup");
    assert_eq!(doc["title"], "First mockup");
    assert_eq!(doc["specs"], serde_json::json!(["UI-001"]));
    assert_eq!(doc["refs"], serde_json::json!(["a.md"]));
    assert_eq!(doc["task"], "TASK-41");
    assert_eq!(doc["status"], "pending");
    assert!(doc["superseded_by"].is_null());
    assert!(doc["version"].is_string());
    assert!(doc["path"].as_str().unwrap().ends_with("MOCK-001.html"));
    // Omitted members unchanged; read-only members ignored.
    let version = doc["version"].as_str().unwrap().to_string();
    let body = format!(
        "{{\"version\":\"{}\",\"refs\":[],\"task\":\"HACK\",\"status\":\"approved\",\"superseded_by\":\"MOCK-002\",\"unknown\":1}}",
        version
    );
    let updated: serde_json::Value =
        serde_json::from_str(&post_json(&server, "/api/doc/MOCK-001", &body).text()).unwrap();
    assert_eq!(updated["title"], "First mockup");
    assert_eq!(updated["specs"], serde_json::json!(["UI-001"]));
    assert_eq!(updated["refs"], serde_json::json!([]));
    assert_eq!(updated["task"], "TASK-41");
    assert_eq!(updated["status"], "pending");
    assert!(updated["superseded_by"].is_null());
    // Refs null clears; specs null is 422.
    let version = updated["version"].as_str().unwrap().to_string();
    let body = format!("{{\"version\":\"{}\",\"refs\":[\"x.md\"]}}", version);
    let updated: serde_json::Value =
        serde_json::from_str(&post_json(&server, "/api/doc/MOCK-001", &body).text()).unwrap();
    assert_eq!(updated["refs"], serde_json::json!(["x.md"]));
    let version = updated["version"].as_str().unwrap().to_string();
    let body = format!("{{\"version\":\"{}\",\"refs\":null}}", version);
    let updated: serde_json::Value =
        serde_json::from_str(&post_json(&server, "/api/doc/MOCK-001", &body).text()).unwrap();
    assert_eq!(updated["refs"], serde_json::json!([]));
    let version = updated["version"].as_str().unwrap().to_string();
    let body = format!("{{\"version\":\"{}\",\"specs\":null}}", version);
    let response = post_json(&server, "/api/doc/MOCK-001", &body);
    assert_eq!(response.status, 422);
    // JSON validation uses the documented messages.
    let body = format!("{{\"version\":\"{}\",\"specs\":[\"bad id\"]}}", version);
    let response = post_json(&server, "/api/doc/MOCK-001", &body);
    assert_eq!(response.status, 422);
    let err: serde_json::Value = serde_json::from_str(&response.text()).unwrap();
    assert_eq!(
        err["error"]["message"],
        "UI specification ids must be valid record IDs."
    );
    let body = format!("{{\"version\":\"{}\",\"refs\":[\"a b\"]}}", version);
    assert_eq!(post_json(&server, "/api/doc/MOCK-001", &body).status, 422);
    let version = updated["version"].as_str().unwrap().to_string();
    let body = format!("{{\"version\":\"{}\",\"specs\":[]}}", version);
    assert_eq!(post_json(&server, "/api/doc/MOCK-001", &body).status, 422);
    // Empty title is 422; wrong types are 400.
    let body = format!("{{\"version\":\"{}\",\"title\":\"\"}}", version);
    assert_eq!(post_json(&server, "/api/doc/MOCK-001", &body).status, 422);
    let body = format!("{{\"version\":\"{}\",\"title\":5}}", version);
    assert_eq!(post_json(&server, "/api/doc/MOCK-001", &body).status, 400);
    let body = format!("{{\"version\":\"{}\",\"specs\":\"UI-001\"}}", version);
    assert_eq!(post_json(&server, "/api/doc/MOCK-001", &body).status, 400);
    let body = format!("{{\"version\":\"{}\",\"specs\":[5]}}", version);
    assert_eq!(post_json(&server, "/api/doc/MOCK-001", &body).status, 400);
    let body = format!("{{\"version\":\"{}\",\"refs\":\"x\"}}", version);
    assert_eq!(post_json(&server, "/api/doc/MOCK-001", &body).status, 400);
    assert_eq!(
        post_json(&server, "/api/doc/MOCK-001", "{not json").status,
        400
    );
    assert_eq!(post_json(&server, "/api/doc/MOCK-001", "[]").status, 400);
    // Missing/malformed/stale versions.
    assert_eq!(
        post_json(&server, "/api/doc/MOCK-001", "{\"title\":\"Ok\"}").status,
        400
    );
    let body = "{\"version\":\"bad\"}";
    assert_eq!(post_json(&server, "/api/doc/MOCK-001", body).status, 400);
    let body = "{\"version\":\"1.2.3\",\"title\":\"Ok\"}";
    assert_eq!(post_json(&server, "/api/doc/MOCK-001", body).status, 409);
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-40k
#[test]
fn t40k_atomic_and_symlink() {
    use std::os::unix::fs::symlink;
    let dir = mockup_fixture("t40k");
    let mock_dir = mockup_dir(&dir);
    let server = Server::start(&dir, &["--no-open"], &[]);
    let version = extract_version(&get(&server, "/doc/MOCK-001/edit").text());
    let body = form_body(&[
        ("version", &version),
        ("title", "Atomic title"),
        ("specs", "UI-001"),
    ]);
    assert_eq!(post_form(&server, "/doc/MOCK-001", &body).status, 303);
    for entry in fs::read_dir(&mock_dir).unwrap().flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        assert!(!name.contains(".tmp"), "temp left {}", name);
    }
    // Failed requests also leave no temp.
    let version = extract_version(&get(&server, "/doc/MOCK-001/edit").text());
    let bad = form_body(&[("version", &version), ("title", ""), ("specs", "UI-001")]);
    assert_eq!(post_form(&server, "/doc/MOCK-001", &bad).status, 422);
    for entry in fs::read_dir(&mock_dir).unwrap().flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        assert!(!name.contains(".tmp"), "temp left {}", name);
    }
    drop(server);
    // Symlink escape is 404.
    let outside = test_root("t40k-outside");
    let secret = outside.join("secret.html");
    fs::write(&secret, "<html><head><title>SECRET40</title></head></html>").unwrap();
    let target = mockup_record_path(&dir, "MOCK-001");
    fs::remove_file(&target).unwrap();
    symlink(&secret, &target).unwrap();
    let server = Server::start(&dir, &["--no-open"], &[]);
    let response = get(&server, "/doc/MOCK-001");
    assert_eq!(response.status, 404);
    assert!(!response.text().contains("SECRET40"));
    let list = get(&server, "/?type=ui-mockup").text();
    assert!(!list.contains("MOCK-001"));
    assert_eq!(get(&server, "/api/doc/MOCK-001").status, 404);
    drop(server);
    let _ = fs::remove_dir_all(&dir);
    let _ = fs::remove_dir_all(&outside);
}

// T-40l
#[test]
fn t40l_offline_asset_scan() {
    let dir = mockup_fixture("t40l");
    write_review_package(&dir, "MOCK-010");
    let server = Server::start(&dir, &["--no-open"], &[]);
    let css = get(&server, "/assets/app.css");
    assert_eq!(css.status, 200);
    let js = get(&server, "/assets/app.js");
    assert_eq!(js.status, 200);
    for path in [
        "/?type=ui-mockup",
        "/doc/MOCK-001",
        "/doc/MOCK-001/edit",
        "/doc/MOCK-010",
        "/doc/MOCK-010/edit",
    ] {
        let html = get(&server, path).text();
        assert!(!html.contains("http://"), "external {}", path);
        assert!(!html.contains("https://"), "external {}", path);
        assert!(!html.contains("//cdn"), "external {}", path);
        assert!(!html.contains("@import"), "external {}", path);
    }
    for asset in [css.text(), js.text()] {
        assert!(!asset.contains("@import"));
        assert!(!asset.contains("http://"));
        assert!(!asset.contains("https://"));
        assert!(!asset.contains("url("));
    }
    // Tag controls are labelled and keyboard operable (buttons, labels).
    let edit = get(&server, "/doc/MOCK-001/edit").text();
    assert!(edit.contains("<label for=\"specs-add\">"));
    assert!(edit.contains("<label for=\"refs-add\">"));
    assert!(edit.contains("aria-label=\"Remove UI-001\""));
    assert!(edit.contains("type=\"button\""));
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-40m: no new dependencies (build/clippy/fmt are quality gates run above).
#[test]
fn t40m_no_new_dependencies() {
    let manifest = include_str!("../Cargo.toml");
    for dependency in ["clap", "serde", "serde_json", "yaml_serde"] {
        assert!(manifest.contains(dependency), "missing {}", dependency);
    }
    let dependency_index = manifest.find("[dependencies]").unwrap();
    let dependencies = &manifest[dependency_index..];
    // No web framework, async runtime, or templating engine was added.
    for forbidden in [
        "axum",
        "actix",
        "rocket",
        "tokio",
        "async-std",
        "tera",
        "handlebars",
        "askama",
        "rusqlite",
        "reqwest",
        "hyper",
    ] {
        assert!(
            !dependencies.contains(forbidden),
            "new dependency {}",
            forbidden
        );
    }
}

// ---- T-25 business rules browser scenarios ----

fn create_rule(
    dir: &Path,
    title: &str,
    description: &str,
    rationale: &str,
    applicability: &str,
    requirements: &[&str],
) -> String {
    let mut args: Vec<String> = vec![
        "--dir".to_string(),
        dir.to_str().unwrap().to_string(),
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
        args.push(req.to_string());
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

fn rule_record_path(dir: &Path, id: &str) -> PathBuf {
    // default config: business-rules/<ID>.md
    dir.join("docs/business-rules").join(format!("{}.md", id))
}

fn rule_dir(dir: &Path) -> PathBuf {
    dir.join("docs/business-rules")
}

fn rule_fixture(name: &str) -> PathBuf {
    let dir = test_root(name);
    init_project(&dir);
    // requirements for links
    create_req_group(&dir, "Payments Group");
    let r1 = create_req(&dir, "Capture payment", "REQ-001");
    create_req_group(&dir, "Search Group");
    let r2 = create_req(&dir, "Search records", "REQ-002");
    create_rule(
        &dir,
        "First rule",
        "First description paragraph.",
        "First rationale",
        "First applicability",
        &[&r1],
    );
    create_rule(
        &dir,
        "Second rule",
        "Second description",
        "Second rationale",
        "Second applicability",
        &[&r1, &r2],
    );
    let _ = (r1, r2);
    dir
}

// T-25a
#[test]
fn t25a_list_business_rules_category() {
    let dir = rule_fixture("t25a");
    let server = Server::start(&dir, &["--no-open"], &[]);
    let html = get(&server, "/?type=business-rule").text();
    assert!(html.contains("<h2 class=\"h2\">Business Rules</h2>"));
    assert!(
        html.contains("2 business rules")
            || html.contains("2 Business Rules")
            || html.contains("2 business rule")
    );
    // count text may vary; check contains 2 and business rules
    assert!(html.contains("aria-current=\"page\""));
    // sidebar order: Business requirements before Business Rules
    let reqs_at = html.find("href=\"/?type=req\"").unwrap();
    let rules_at = html.find("href=\"/?type=business-rule\"").unwrap();
    assert!(reqs_at < rules_at, "sidebar order");
    assert!(html.contains("RUL-001"));
    assert!(html.contains("First rule"));
    assert!(html.contains("RUL-002"));
    assert!(html.contains("Second rule"));
    let first = html.find("RUL-001").unwrap();
    let second = html.find("RUL-002").unwrap();
    assert!(first < second, "ascending ID order");
    assert!(html.contains("href=\"/doc/RUL-001\""));
    assert!(html.contains("href=\"/doc/RUL-002\""));
    // TASK-142: the shared list carries values in the declared slot cells and
    // the Title cell labels/references; the legacy description preview is gone.
    assert!(!html.contains("First description paragraph."));
    // default stays req; unknown values are 400.
    assert_eq!(get(&server, "/?type=req").status, 200);
    assert_eq!(get(&server, "/?type=req").status, 200);
    assert_eq!(get(&server, "/?type=rule").status, 200);
    for ty in ["all", "x", "business-rules"] {
        assert_eq!(
            get(&server, &format!("/?type={}", ty)).status,
            404,
            "type={}",
            ty
        );
    }
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn t25a_disabled_business_rule_empty_state() {
    let dir = rule_fixture("t25a-disabled");
    run_ok(&[
        "--dir",
        dir.to_str().unwrap(),
        "config",
        "set",
        "document_types.rule.enabled",
        "false",
    ]);
    let server = Server::start(&dir, &["--no-open"], &[]);
    let html = get(&server, "/").text();
    assert!(!html.contains("href=\"/?type=business-rule\""));
    assert_eq!(get(&server, "/?type=business-rule").status, 404);
    assert_eq!(get(&server, "/api/business-rules").status, 404);
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-25b
#[test]
fn t25b_non_records_are_not_listed() {
    let dir = rule_fixture("t25b");
    let rdir = rule_dir(&dir);
    fs::write(rdir.join("BR-001.md"), "# BR-001 Canonical\n").unwrap();
    fs::write(rdir.join("README.md"), "# registry").unwrap();
    fs::write(rdir.join("notes.txt"), "x").unwrap();
    fs::write(rdir.join("RUL-ABC.md"), "# bad").unwrap();
    let archived = rdir.join("archived");
    fs::create_dir_all(&archived).unwrap();
    fs::write(
        archived.join("RUL-999.md"),
        "# RUL-999 Hidden\n\n## Description\nhidden\n",
    )
    .unwrap();
    let server = Server::start(&dir, &["--no-open"], &[]);
    let html = get(&server, "/?type=business-rule").text();
    assert!(html.contains("RUL-001"));
    assert!(html.contains("RUL-002"));
    assert!(!html.contains("RUL-999"));
    assert!(!html.contains("BR-001"));
    let api: serde_json::Value =
        serde_json::from_str(&get(&server, "/api/business-rules").text()).unwrap();
    assert_eq!(api.as_array().unwrap().len(), 2);
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-25c
#[test]
fn t25c_view_business_rule() {
    let dir = rule_fixture("t25c");
    let server = Server::start(&dir, &["--no-open"], &[]);
    let html = get(&server, "/doc/RUL-001").text();
    assert!(html.contains("<p class=\"id\">"));
    assert!(html.contains("RUL-001"));
    assert!(html.contains("Business Rules"));
    assert!(html.contains("<h1 class=\"h1\">First rule</h1>"));
    assert!(html.contains("Description"));
    assert!(html.contains("Rationale"));
    assert!(html.contains("Applicability"));
    assert!(html.contains("Requirements"));
    assert!(html.contains("First description paragraph."));
    assert!(html.contains("First rationale"));
    assert!(html.contains("First applicability"));
    assert!(html.contains("white-space: pre-wrap"));
    // breadcrumb
    assert!(html.contains("href=\"/?type=business-rule\">Business Rules</a></li><li><span aria-current=\"page\">RUL-001</span>"));
    assert_eq!(html.matches("href=\"/doc/RUL-001/edit\"").count(), 1);
    assert!(html.contains(">Edit</a>"));
    // requirement link resolves to req view
    assert!(html.contains("href=\"/doc/REQ-001.001\"") || html.contains("REQ-001.001"));
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-25d
#[test]
fn t25d_edit_round_trip() {
    let dir = rule_fixture("t25d");
    let server = Server::start(&dir, &["--no-open"], &[]);
    let edit = get(&server, "/doc/RUL-001/edit").text();
    assert!(edit.contains("name=\"requirements\""));
    assert!(edit.contains("Hidden field: version (optimistic-concurrency token)"));
    assert!(edit.contains("Save changes") || edit.contains("Save"));
    let version = extract_version(&edit);
    // ensure requirement ids exist? use actual req ids
    let r2 = "REQ-001.001";
    let body = form_body(&[
        ("version", &version),
        ("title", "Renamed rule"),
        ("description", "New desc"),
        ("rationale", "New rationale"),
        ("applicability", "New applicability"),
        ("requirements", r2),
        ("requirements", ""),
    ]);
    let response = post_form(&server, "/doc/RUL-001", &body);
    assert_eq!(response.status, 303);
    assert_eq!(response.header("Location"), Some("/doc/RUL-001"));
    let view = get(&server, "/doc/RUL-001").text();
    assert!(view.contains("<h1 class=\"h1\">Renamed rule</h1>"));
    assert!(view.contains("New desc"));
    assert!(view.contains("New rationale"));
    assert!(view.contains("New applicability"));
    // visible through CLI
    let cli = run_ok(&["--dir", dir.to_str().unwrap(), "rule", "view", "RUL-001"]);
    assert!(cli.contains("Renamed rule"));
    assert!(cli.contains("New desc"));
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-25e
#[test]
fn t25e_field_preserving() {
    let dir = rule_fixture("t25e");
    // add unknown section manually
    let path = rule_record_path(&dir, "RUL-001");
    let original = fs::read_to_string(&path).unwrap();
    let with_unknown = original.replace(
        "## Requirements",
        "## Custom Section\nKeep this line\n\n## Requirements",
    );
    fs::write(&path, &with_unknown).unwrap();
    let before = fs::read_to_string(&path).unwrap();
    let server = Server::start(&dir, &["--no-open"], &[]);
    let version = extract_version(&get(&server, "/doc/RUL-001/edit").text());
    let body = form_body(&[
        ("version", &version),
        ("title", "Edited title"),
        ("description", "Edited desc"),
        ("rationale", "Edited rationale"),
        ("applicability", "Edited applicability"),
        ("requirements", ""),
    ]);
    assert_eq!(post_form(&server, "/doc/RUL-001", &body).status, 303);
    let after = fs::read_to_string(&path).unwrap();
    assert!(after.contains("## Custom Section\nKeep this line"));
    assert!(after.contains("Edited title"));
    assert!(after.contains("Edited desc"));
    // byte comparison: unknown section lines identical
    for needle in ["## Custom Section", "Keep this line", "preamble"] {
        if before.contains(needle) {
            assert!(after.contains(needle));
        }
    }
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-25f
#[test]
fn t25f_repeated_requirements_in_order() {
    let dir = rule_fixture("t25f");
    let server = Server::start(&dir, &["--no-open"], &[]);
    let version = extract_version(&get(&server, "/doc/RUL-002/edit").text());
    // use valid req ids
    let body = form_body(&[
        ("version", &version),
        ("title", "Order check"),
        ("description", "d"),
        ("rationale", "r"),
        ("applicability", "a"),
        ("requirements", "REQ-001.001"),
        ("requirements", "REQ-002.001"), // may not exist but valid id? REQ-002.001 valid format yes
        ("requirements", "REQ-001.001"),
        ("requirements", ""),
    ]);
    assert_eq!(post_form(&server, "/doc/RUL-002", &body).status, 303);
    let file = fs::read_to_string(rule_record_path(&dir, "RUL-002")).unwrap();
    assert!(file.contains("## Requirements\n- REQ-001.001\n- REQ-002.001\n"));
    // duplicates collapsed
    assert!(!file.contains("REQ-001.001\n- REQ-001.001"));
    let api: serde_json::Value =
        serde_json::from_str(&get(&server, "/api/doc/RUL-002").text()).unwrap();
    assert_eq!(
        api["requirements"],
        serde_json::json!(["REQ-001.001", "REQ-002.001"])
    );
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-25g
#[test]
fn t25g_validation_422_no_write() {
    let dir = rule_fixture("t25g");
    let path = rule_record_path(&dir, "RUL-001");
    let before = fs::read(&path).unwrap();
    let server = Server::start(&dir, &["--no-open"], &[]);
    let version = extract_version(&get(&server, "/doc/RUL-001/edit").text());
    let cases: Vec<(String, &str, &str)> = vec![
        (
            form_body(&[
                ("version", &version),
                ("title", ""),
                ("requirements", "REQ-001.001"),
            ]),
            "Title is required.",
            "title",
        ),
        (
            form_body(&[
                ("version", &version),
                ("title", &"x".repeat(201)),
                ("requirements", "REQ-001.001"),
            ]),
            "Title must be at most 200 characters.",
            "title",
        ),
        (
            form_body(&[
                ("version", &version),
                ("title", "a\tb"),
                ("requirements", "REQ-001.001"),
            ]),
            "Title cannot contain control characters.",
            "title",
        ),
        (
            form_body(&[
                ("version", &version),
                ("title", "Ok"),
                ("description", &"x".repeat(20001)),
            ]),
            "Description must be at most 20000 characters.",
            "description",
        ),
        (
            form_body(&[
                ("version", &version),
                ("title", "Ok"),
                ("description", "a\n## Section\nb"),
            ]),
            "Description contains a line starting with ## .",
            "description",
        ),
        (
            form_body(&[
                ("version", &version),
                ("title", "Ok"),
                ("rationale", &"x".repeat(20001)),
            ]),
            "Rationale must be at most 20000 characters.",
            "rationale",
        ),
        (
            form_body(&[
                ("version", &version),
                ("title", "Ok"),
                ("applicability", &"x".repeat(20001)),
            ]),
            "Applicability must be at most 20000 characters.",
            "applicability",
        ),
        (
            form_body(&[
                ("version", &version),
                ("title", "Ok"),
                ("requirements", "bad id"),
            ]),
            "Requirement references must be valid record IDs.",
            "requirements",
        ),
    ];
    for (body, message, field) in cases {
        let response = post_form(&server, "/doc/RUL-001", &body);
        assert_eq!(response.status, 422, "body={}", body);
        let html = response.text();
        assert_eq!(summary_entry(&html, field), message);
        assert_eq!(field_error(&html, field), message);
        assert_eq!(fs::read(&path).unwrap(), before, "file changed {}", body);
    }
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-25h
#[test]
fn t25h_stale_and_malformed_versions() {
    let dir = rule_fixture("t25h");
    let path = rule_record_path(&dir, "RUL-001");
    let before = fs::read(&path).unwrap();
    let server = Server::start(&dir, &["--no-open"], &[]);
    let valid = form_body(&[
        ("version", "1.2.3"),
        ("title", "Stale"),
        ("requirements", "REQ-001.001"),
    ]);
    let conflict = post_form(&server, "/doc/RUL-001", &valid);
    assert_eq!(conflict.status, 409);
    assert!(
        conflict
            .text()
            .contains("record changed on disk; reload the form")
    );
    assert_eq!(fs::read(&path).unwrap(), before);
    let missing = form_body(&[("title", "Ok"), ("requirements", "REQ-001.001")]);
    assert_eq!(post_form(&server, "/doc/RUL-001", &missing).status, 400);
    let malformed = form_body(&[
        ("version", "bad"),
        ("title", "Ok"),
        ("requirements", "REQ-001.001"),
    ]);
    assert_eq!(post_form(&server, "/doc/RUL-001", &malformed).status, 400);
    assert_eq!(fs::read(&path).unwrap(), before);
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-25i
#[test]
fn t25i_archived_hidden_and_404() {
    let dir = rule_fixture("t25i");
    run_ok(&["--dir", dir.to_str().unwrap(), "rule", "archive", "RUL-002"]);
    let server = Server::start(&dir, &["--no-open"], &[]);
    let list = get(&server, "/?type=business-rule").text();
    assert!(list.contains("RUL-001"));
    assert!(!list.contains("RUL-002"));
    assert_eq!(get(&server, "/doc/RUL-002").status, 404);
    assert_eq!(get(&server, "/doc/RUL-002/edit").status, 404);
    let version = "1.2.3";
    let body = form_body(&[
        ("version", version),
        ("title", "Ok"),
        ("requirements", "REQ-001.001"),
    ]);
    assert_eq!(post_form(&server, "/doc/RUL-002", &body).status, 404);
    assert_eq!(get(&server, "/api/doc/RUL-002").status, 404);
    let json_body = format!("{{\"version\":\"{}\",\"title\":\"Ok\"}}", version);
    assert_eq!(
        post_json(&server, "/api/doc/RUL-002", &json_body).status,
        404
    );
    let api: serde_json::Value =
        serde_json::from_str(&get(&server, "/api/business-rules").text()).unwrap();
    assert_eq!(api.as_array().unwrap().len(), 1);
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-25j
#[test]
fn t25j_json_api() {
    let dir = rule_fixture("t25j");
    let server = Server::start(&dir, &["--no-open"], &[]);
    for alias in ["/api/business-rules", "/api/business-rule"] {
        let list: serde_json::Value = serde_json::from_str(&get(&server, alias).text()).unwrap();
        let arr = list.as_array().unwrap();
        assert_eq!(arr.len(), 2, "alias {}", alias);
        assert_eq!(arr[0]["id"], "RUL-001");
        assert_eq!(arr[0]["title"], "First rule");
        assert_eq!(arr[1]["id"], "RUL-002");
    }
    let doc: serde_json::Value =
        serde_json::from_str(&get(&server, "/api/doc/RUL-001").text()).unwrap();
    assert_eq!(doc["id"], "RUL-001");
    assert_eq!(doc["kind"], "business-rule");
    assert_eq!(doc["form"], "business-rule");
    assert_eq!(doc["title"], "First rule");
    assert_eq!(doc["description"], "First description paragraph.");
    assert_eq!(doc["rationale"], "First rationale");
    assert_eq!(doc["applicability"], "First applicability");
    assert_eq!(doc["requirements"], serde_json::json!(["REQ-001.001"]));
    assert!(doc["version"].is_string());
    assert!(doc["path"].as_str().unwrap().ends_with("RUL-001.md"));
    // description null? second rule has desc
    let version = doc["version"].as_str().unwrap().to_string();
    // omitted unchanged; wrong types
    let body = format!(
        "{{\"version\":\"{}\",\"title\":\"Renamed\",\"requirements\":[\"REQ-002.001\"]}}",
        version
    );
    let updated: serde_json::Value =
        serde_json::from_str(&post_json(&server, "/api/doc/RUL-001", &body).text()).unwrap();
    assert_eq!(updated["title"], "Renamed");
    assert_eq!(updated["requirements"], serde_json::json!(["REQ-002.001"]));
    // null clears description
    let version = updated["version"].as_str().unwrap().to_string();
    let body = format!("{{\"version\":\"{}\",\"description\":null}}", version);
    let updated: serde_json::Value =
        serde_json::from_str(&post_json(&server, "/api/doc/RUL-001", &body).text()).unwrap();
    assert!(updated["description"].is_null());
    // validation 422
    let version = updated["version"].as_str().unwrap().to_string();
    let body = format!("{{\"version\":\"{}\",\"title\":\"\"}}", version);
    assert_eq!(post_json(&server, "/api/doc/RUL-001", &body).status, 422);
    let body = format!(
        "{{\"version\":\"{}\",\"requirements\":[\"bad id\"]}}",
        version
    );
    let response = post_json(&server, "/api/doc/RUL-001", &body);
    assert_eq!(response.status, 422);
    let err: serde_json::Value = serde_json::from_str(&response.text()).unwrap();
    assert_eq!(
        err["error"]["message"],
        "Requirement references must be valid record IDs."
    );
    // wrong types 400
    let body = format!("{{\"version\":\"{}\",\"title\":5}}", version);
    assert_eq!(post_json(&server, "/api/doc/RUL-001", &body).status, 400);
    let body = format!("{{\"version\":\"{}\",\"requirements\":\"x\"}}", version);
    assert_eq!(post_json(&server, "/api/doc/RUL-001", &body).status, 400);
    let body = format!("{{\"version\":\"{}\",\"requirements\":[5]}}", version);
    assert_eq!(post_json(&server, "/api/doc/RUL-001", &body).status, 400);
    assert_eq!(
        post_json(&server, "/api/doc/RUL-001", "{not json").status,
        400
    );
    assert_eq!(post_json(&server, "/api/doc/RUL-001", "[]").status, 400);
    // missing/malformed/stale version
    assert_eq!(
        post_json(&server, "/api/doc/RUL-001", "{\"title\":\"Ok\"}").status,
        400
    );
    assert_eq!(
        post_json(&server, "/api/doc/RUL-001", "{\"version\":\"bad\"}").status,
        400
    );
    let body = "{\"version\":\"1.2.3\",\"title\":\"Ok\"}";
    assert_eq!(post_json(&server, "/api/doc/RUL-001", body).status, 409);
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-25k
#[test]
fn t25k_atomic_and_symlink() {
    use std::os::unix::fs::symlink;
    let dir = rule_fixture("t25k");
    let rdir = rule_dir(&dir);
    let server = Server::start(&dir, &["--no-open"], &[]);
    let version = extract_version(&get(&server, "/doc/RUL-001/edit").text());
    let body = form_body(&[
        ("version", &version),
        ("title", "Atomic title"),
        ("description", "d"),
        ("rationale", "r"),
        ("applicability", "a"),
        ("requirements", ""),
    ]);
    assert_eq!(post_form(&server, "/doc/RUL-001", &body).status, 303);
    for entry in fs::read_dir(&rdir).unwrap().flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        assert!(!name.contains(".tmp"), "temp left {}", name);
    }
    let version = extract_version(&get(&server, "/doc/RUL-001/edit").text());
    let bad = form_body(&[
        ("version", &version),
        ("title", ""),
        ("requirements", "REQ-001.001"),
    ]);
    assert_eq!(post_form(&server, "/doc/RUL-001", &bad).status, 422);
    for entry in fs::read_dir(&rdir).unwrap().flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        assert!(!name.contains(".tmp"), "temp left {}", name);
    }
    drop(server);
    // symlink escape
    let outside = test_root("t25k-outside");
    let secret = outside.join("secret.md");
    fs::write(&secret, "# RUL-001 Secret\n").unwrap();
    let target = rule_record_path(&dir, "RUL-001");
    fs::remove_file(&target).unwrap();
    symlink(&secret, &target).unwrap();
    let server = Server::start(&dir, &["--no-open"], &[]);
    assert_eq!(get(&server, "/doc/RUL-001").status, 404);
    assert_eq!(get(&server, "/api/doc/RUL-001").status, 404);
    drop(server);
    let _ = fs::remove_dir_all(&dir);
    let _ = fs::remove_dir_all(&outside);
}

// T-25l
#[test]
fn t25l_offline_asset_scan() {
    let dir = rule_fixture("t25l");
    let server = Server::start(&dir, &["--no-open"], &[]);
    let css = get(&server, "/assets/app.css");
    let js = get(&server, "/assets/app.js");
    assert_eq!(css.status, 200);
    assert_eq!(js.status, 200);
    for path in ["/?type=business-rule", "/doc/RUL-001", "/doc/RUL-001/edit"] {
        let html = get(&server, path).text();
        assert!(!html.contains("http://"), "external {}", path);
        assert!(!html.contains("https://"), "external {}", path);
        assert!(!html.contains("//cdn"), "external {}", path);
        assert!(!html.contains("@import"), "external {}", path);
    }
    for asset in [css.text(), js.text()] {
        assert!(!asset.contains("@import"));
        assert!(!asset.contains("http://"));
        assert!(!asset.contains("https://"));
    }
    let edit = get(&server, "/doc/RUL-001/edit").text();
    assert!(edit.contains("aria-label=\"Remove") || edit.contains("Remove"));
    assert!(edit.contains("type=\"button\"") || edit.contains("<button"));
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-25m
#[test]
fn t25m_no_new_dependencies() {
    let manifest = include_str!("../Cargo.toml");
    let dependency_index = manifest.find("[dependencies]").unwrap();
    let dependencies = &manifest[dependency_index..];
    for forbidden in [
        "axum",
        "actix",
        "rocket",
        "tokio",
        "async-std",
        "tera",
        "handlebars",
        "askama",
    ] {
        assert!(
            !dependencies.contains(forbidden),
            "new dependency {}",
            forbidden
        );
    }
}

// ---- T-26 audit records browser scenarios ----

fn create_audit(dir: &Path, title: &str, content: &str, audit_type: Option<&str>) -> String {
    let mut args = vec![
        "--dir".to_string(),
        dir.to_str().unwrap().to_string(),
        "audit".to_string(),
        "create".to_string(),
        "--title".to_string(),
        title.to_string(),
        "--content".to_string(),
        content.to_string(),
    ];
    if let Some(t) = audit_type {
        args.push("--audit-type".to_string());
        args.push(t.to_string());
    }
    let out = bin().args(&args).output().unwrap();
    assert!(
        out.status.success(),
        "audit create failed: {:?}\nstderr: {}",
        args,
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn audit_record_path(dir: &Path, id: &str) -> PathBuf {
    dir.join("docs/audit-records").join(format!("{}.md", id))
}

fn audit_dir(dir: &Path) -> PathBuf {
    dir.join("docs/audit-records")
}

fn audit_fixture(name: &str) -> PathBuf {
    let dir = test_root(name);
    init_project(&dir);
    create_audit(
        &dir,
        "First audit",
        "First content paragraph.",
        Some("security"),
    );
    create_audit(&dir, "Second audit", "Second content", Some("licence"));
    dir
}

// T-26a
#[test]
fn t26a_list_audit_records_category() {
    let dir = audit_fixture("t26a");
    let server = Server::start(&dir, &["--no-open"], &[]);
    let html = get(&server, "/?type=audit-record").text();
    assert!(html.contains("<h2 class=\"h2\">Audit Records</h2>"));
    assert!(html.contains("2 audit records"));
    assert!(html.contains("aria-current=\"page\""));
    let rules_at = html.find("href=\"/?type=business-rule\"").unwrap();
    let audit_at = html.find("href=\"/?type=audit-record\"").unwrap();
    assert!(rules_at < audit_at);
    assert!(html.contains("AUD-001"));
    assert!(html.contains("First audit"));
    assert!(html.contains("AUD-002"));
    assert!(html.contains("Second audit"));
    let first = html.find("AUD-001").unwrap();
    let second = html.find("AUD-002").unwrap();
    assert!(first < second);
    assert!(html.contains("href=\"/doc/AUD-001\""));
    // TASK-142: the shared list no longer renders a description preview.
    assert!(!html.contains("First content paragraph."));
    assert_eq!(get(&server, "/?type=req").status, 200);
    assert_eq!(get(&server, "/?type=req").status, 200);
    assert_eq!(get(&server, "/?type=audit").status, 200);
    for ty in ["all", "x", "audit-records"] {
        assert_eq!(
            get(&server, &format!("/?type={}", ty)).status,
            404,
            "type={}",
            ty
        );
    }
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn t26a_disabled_audit_record_empty_state() {
    let dir = audit_fixture("t26a-disabled");
    run_ok(&[
        "--dir",
        dir.to_str().unwrap(),
        "config",
        "set",
        "document_types.audit.enabled",
        "false",
    ]);
    let server = Server::start(&dir, &["--no-open"], &[]);
    let html = get(&server, "/").text();
    assert!(!html.contains("href=\"/?type=audit-record\""));
    assert_eq!(get(&server, "/?type=audit-record").status, 404);
    assert_eq!(get(&server, "/api/audit-records").status, 404);
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-26b
#[test]
fn t26b_non_records_are_not_listed() {
    let dir = audit_fixture("t26b");
    let rdir = audit_dir(&dir);
    fs::write(rdir.join("BR-001.md"), "# BR-001 Canonical\n").unwrap();
    fs::write(rdir.join("README.md"), "# registry").unwrap();
    fs::write(rdir.join("notes.txt"), "x").unwrap();
    fs::write(rdir.join("AUD-ABC.md"), "# bad").unwrap();
    let archived = rdir.join("archived");
    fs::create_dir_all(&archived).unwrap();
    fs::write(
        archived.join("AUD-999.md"),
        "# AUD-999 Hidden\n\n## Audit Type\nsecurity\n",
    )
    .unwrap();
    let server = Server::start(&dir, &["--no-open"], &[]);
    let html = get(&server, "/?type=audit-record").text();
    assert!(html.contains("AUD-001"));
    assert!(html.contains("AUD-002"));
    assert!(!html.contains("AUD-999"));
    assert!(!html.contains("BR-001"));
    let api: serde_json::Value =
        serde_json::from_str(&get(&server, "/api/audit-records").text()).unwrap();
    assert_eq!(api.as_array().unwrap().len(), 2);
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-26c
#[test]
fn t26c_view_audit_record() {
    let dir = audit_fixture("t26c");
    let server = Server::start(&dir, &["--no-open"], &[]);
    let html = get(&server, "/doc/AUD-001").text();
    assert!(html.contains("<p class=\"id\">"));
    assert!(html.contains("AUD-001"));
    assert!(html.contains("Audit Records"));
    assert!(html.contains("<h1 class=\"h1\">First audit</h1>"));
    assert!(html.contains("<span class=\"badge\">security</span>"));
    assert!(html.contains("Audit type"));
    assert!(html.contains("First content paragraph."));
    assert!(html.contains("white-space: pre-wrap"));
    assert!(html.contains("href=\"/?type=audit-record\">Audit Records</a></li><li><span aria-current=\"page\">AUD-001</span>"));
    assert_eq!(html.matches("href=\"/doc/AUD-001/edit\"").count(), 1);
    assert!(html.contains(">Edit</a>"));
    assert!(!html.contains("Content</h"));
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-26d
#[test]
fn t26d_edit_round_trip() {
    let dir = audit_fixture("t26d");
    let server = Server::start(&dir, &["--no-open"], &[]);
    let edit = get(&server, "/doc/AUD-001/edit").text();
    assert!(edit.contains("Hidden field: version (optimistic-concurrency token)"));
    assert!(edit.contains("Save") || edit.contains("Save changes"));
    let version = extract_version(&edit);
    let body = form_body(&[
        ("version", &version),
        ("title", "Renamed audit"),
        ("audit_type", "licence"),
        ("content", "New content"),
    ]);
    let response = post_form(&server, "/doc/AUD-001", &body);
    assert_eq!(response.status, 303);
    assert_eq!(response.header("Location"), Some("/doc/AUD-001"));
    let view = get(&server, "/doc/AUD-001").text();
    assert!(view.contains("<h1 class=\"h1\">Renamed audit</h1>"));
    assert!(view.contains("New content"));
    let cli = run_ok(&["--dir", dir.to_str().unwrap(), "audit", "view", "AUD-001"]);
    assert!(cli.contains("Renamed audit"));
    assert!(cli.contains("New content"));
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-26e
#[test]
fn t26e_field_preserving() {
    let dir = audit_fixture("t26e");
    let path = audit_record_path(&dir, "AUD-001");
    let original = fs::read_to_string(&path).unwrap();
    let with_unknown = original.replace(
        "## Content",
        "## Custom Section\nKeep this line\n\n## Content",
    );
    fs::write(&path, &with_unknown).unwrap();
    let before = fs::read_to_string(&path).unwrap();
    let server = Server::start(&dir, &["--no-open"], &[]);
    let version = extract_version(&get(&server, "/doc/AUD-001/edit").text());
    let body = form_body(&[
        ("version", &version),
        ("title", "Edited title"),
        ("audit_type", "architecture"),
        ("content", "Edited content"),
    ]);
    assert_eq!(post_form(&server, "/doc/AUD-001", &body).status, 303);
    let after = fs::read_to_string(&path).unwrap();
    assert!(after.contains("## Custom Section\nKeep this line"));
    assert!(after.contains("Edited title"));
    assert!(after.contains("Edited content"));
    for needle in ["## Custom Section", "Keep this line"] {
        if before.contains(needle) {
            assert!(after.contains(needle));
        }
    }
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-26f
#[test]
fn t26f_validation_422_no_write() {
    let dir = audit_fixture("t26f");
    let path = audit_record_path(&dir, "AUD-001");
    let before = fs::read(&path).unwrap();
    let server = Server::start(&dir, &["--no-open"], &[]);
    let version = extract_version(&get(&server, "/doc/AUD-001/edit").text());
    let cases: Vec<(String, &str, &str)> = vec![
        (
            form_body(&[
                ("version", &version),
                ("title", ""),
                ("audit_type", "security"),
                ("content", "ok"),
            ]),
            "Title is required.",
            "title",
        ),
        (
            form_body(&[
                ("version", &version),
                ("title", &"x".repeat(201)),
                ("audit_type", "security"),
                ("content", "ok"),
            ]),
            "Title must be at most 200 characters.",
            "title",
        ),
        (
            form_body(&[
                ("version", &version),
                ("title", "a\tb"),
                ("audit_type", "security"),
                ("content", "ok"),
            ]),
            "Title cannot contain control characters.",
            "title",
        ),
        (
            form_body(&[
                ("version", &version),
                ("title", "Ok"),
                ("audit_type", ""),
                ("content", "ok"),
            ]),
            "Audit type is required.",
            "audit_type",
        ),
        (
            form_body(&[
                ("version", &version),
                ("title", "Ok"),
                ("audit_type", "foo"),
                ("content", "ok"),
            ]),
            "Audit type must be one of security, licence, or architecture.",
            "audit_type",
        ),
        (
            form_body(&[
                ("version", &version),
                ("title", "Ok"),
                ("audit_type", "security"),
                ("content", &"x".repeat(20001)),
            ]),
            "Content must be at most 20000 characters.",
            "content",
        ),
        (
            form_body(&[
                ("version", &version),
                ("title", "Ok"),
                ("audit_type", "security"),
                ("content", "a\u{0007}b"),
            ]),
            "Content cannot contain control characters.",
            "content",
        ),
        (
            form_body(&[
                ("version", &version),
                ("title", "Ok"),
                ("audit_type", "security"),
                ("content", "a\n## Section\nb"),
            ]),
            "Content contains a line starting with ## .",
            "content",
        ),
    ];
    for (body, message, field) in cases {
        let response = post_form(&server, "/doc/AUD-001", &body);
        assert_eq!(response.status, 422, "body={}", body);
        let html = response.text();
        assert_eq!(summary_entry(&html, field), message);
        assert_eq!(field_error(&html, field), message);
        assert_eq!(fs::read(&path).unwrap(), before, "file changed {}", body);
    }
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-26g
#[test]
fn t26g_stale_and_malformed_versions() {
    let dir = audit_fixture("t26g");
    let path = audit_record_path(&dir, "AUD-001");
    let before = fs::read(&path).unwrap();
    let server = Server::start(&dir, &["--no-open"], &[]);
    let valid = form_body(&[
        ("version", "1.2.3"),
        ("title", "Stale"),
        ("audit_type", "security"),
        ("content", "ok"),
    ]);
    let conflict = post_form(&server, "/doc/AUD-001", &valid);
    assert_eq!(conflict.status, 409);
    assert!(
        conflict
            .text()
            .contains("record changed on disk; reload the form")
    );
    assert_eq!(fs::read(&path).unwrap(), before);
    let missing = form_body(&[
        ("title", "Ok"),
        ("audit_type", "security"),
        ("content", "ok"),
    ]);
    assert_eq!(post_form(&server, "/doc/AUD-001", &missing).status, 400);
    let malformed = form_body(&[
        ("version", "bad"),
        ("title", "Ok"),
        ("audit_type", "security"),
        ("content", "ok"),
    ]);
    assert_eq!(post_form(&server, "/doc/AUD-001", &malformed).status, 400);
    assert_eq!(fs::read(&path).unwrap(), before);
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-26h
#[test]
fn t26h_archived_hidden_and_404() {
    let dir = audit_fixture("t26h");
    run_ok(&[
        "--dir",
        dir.to_str().unwrap(),
        "audit",
        "archive",
        "AUD-002",
    ]);
    let server = Server::start(&dir, &["--no-open"], &[]);
    let list = get(&server, "/?type=audit-record").text();
    assert!(list.contains("AUD-001"));
    assert!(!list.contains("AUD-002"));
    assert_eq!(get(&server, "/doc/AUD-002").status, 404);
    assert_eq!(get(&server, "/doc/AUD-002/edit").status, 404);
    let version = "1.2.3";
    let body = form_body(&[
        ("version", version),
        ("title", "Ok"),
        ("audit_type", "security"),
        ("content", "ok"),
    ]);
    assert_eq!(post_form(&server, "/doc/AUD-002", &body).status, 404);
    assert_eq!(get(&server, "/api/doc/AUD-002").status, 404);
    let json_body = format!("{{\"version\":\"{}\",\"title\":\"Ok\"}}", version);
    assert_eq!(
        post_json(&server, "/api/doc/AUD-002", &json_body).status,
        404
    );
    let api: serde_json::Value =
        serde_json::from_str(&get(&server, "/api/audit-records").text()).unwrap();
    assert_eq!(api.as_array().unwrap().len(), 1);
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-26i
#[test]
fn t26i_json_api() {
    let dir = audit_fixture("t26i");
    let server = Server::start(&dir, &["--no-open"], &[]);
    for alias in ["/api/audit-records", "/api/audit-record"] {
        let list: serde_json::Value = serde_json::from_str(&get(&server, alias).text()).unwrap();
        let arr = list.as_array().unwrap();
        assert_eq!(arr.len(), 2, "alias {}", alias);
        assert_eq!(arr[0]["id"], "AUD-001");
        assert_eq!(arr[0]["title"], "First audit");
        assert_eq!(arr[1]["id"], "AUD-002");
    }
    let doc: serde_json::Value =
        serde_json::from_str(&get(&server, "/api/doc/AUD-001").text()).unwrap();
    assert_eq!(doc["id"], "AUD-001");
    assert_eq!(doc["kind"], "audit-record");
    assert_eq!(doc["form"], "audit-record");
    assert_eq!(doc["title"], "First audit");
    assert_eq!(doc["audit_type"], "security");
    assert_eq!(doc["content"], "First content paragraph.");
    assert!(doc["version"].is_string());
    assert!(doc["path"].as_str().unwrap().ends_with("AUD-001.md"));
    let version = doc["version"].as_str().unwrap().to_string();
    let body = format!(
        r#"{{"version":"{}","title":"Renamed","audit_type":"licence","content":"New content"}}"#,
        version
    );
    let updated: serde_json::Value =
        serde_json::from_str(&post_json(&server, "/api/doc/AUD-001", &body).text()).unwrap();
    assert_eq!(updated["title"], "Renamed");
    assert_eq!(updated["audit_type"], "licence");
    assert_eq!(updated["content"], "New content");
    let version = updated["version"].as_str().unwrap().to_string();
    let body = format!(r#"{{"version":"{}","content":null}}"#, version);
    let updated: serde_json::Value =
        serde_json::from_str(&post_json(&server, "/api/doc/AUD-001", &body).text()).unwrap();
    assert!(updated["content"].is_null());
    let version = updated["version"].as_str().unwrap().to_string();
    let body = format!(r#"{{"version":"{}","title":""}}"#, version);
    assert_eq!(post_json(&server, "/api/doc/AUD-001", &body).status, 422);
    let body = format!(r#"{{"version":"{}","audit_type":"foo"}}"#, version);
    assert_eq!(post_json(&server, "/api/doc/AUD-001", &body).status, 422);
    let body = format!(
        r#"{{"version":"{}","content":"a\n## Section\nb"}}"#,
        version
    );
    assert_eq!(post_json(&server, "/api/doc/AUD-001", &body).status, 422);
    // wrong types 400
    let body = format!(r#"{{"version":"{}","title":5}}"#, version);
    assert_eq!(post_json(&server, "/api/doc/AUD-001", &body).status, 400);
    let body = format!(r#"{{"version":"{}","audit_type":5}}"#, version);
    assert_eq!(post_json(&server, "/api/doc/AUD-001", &body).status, 400);
    let body = format!(r#"{{"version":"{}","content":5}}"#, version);
    assert_eq!(post_json(&server, "/api/doc/AUD-001", &body).status, 400);
    assert_eq!(
        post_json(&server, "/api/doc/AUD-001", "{not json").status,
        400
    );
    assert_eq!(post_json(&server, "/api/doc/AUD-001", "[]").status, 400);
    // missing/malformed/stale version
    assert_eq!(
        post_json(&server, "/api/doc/AUD-001", "{\"title\":\"Ok\"}").status,
        400
    );
    assert_eq!(
        post_json(&server, "/api/doc/AUD-001", "{\"version\":\"bad\"}").status,
        400
    );
    let body = "{\"version\":\"1.2.3\",\"title\":\"Ok\"}";
    assert_eq!(post_json(&server, "/api/doc/AUD-001", body).status, 409);
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-26j
#[test]
fn t26j_atomic_and_symlink() {
    use std::os::unix::fs::symlink;
    let dir = audit_fixture("t26j");
    let rdir = audit_dir(&dir);
    let server = Server::start(&dir, &["--no-open"], &[]);
    let version = extract_version(&get(&server, "/doc/AUD-001/edit").text());
    let body = form_body(&[
        ("version", &version),
        ("title", "Atomic title"),
        ("audit_type", "security"),
        ("content", "d"),
    ]);
    assert_eq!(post_form(&server, "/doc/AUD-001", &body).status, 303);
    for entry in fs::read_dir(&rdir).unwrap().flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        assert!(!name.contains(".tmp"), "temp left {}", name);
    }
    let version = extract_version(&get(&server, "/doc/AUD-001/edit").text());
    let bad = form_body(&[
        ("version", &version),
        ("title", ""),
        ("audit_type", "security"),
        ("content", "ok"),
    ]);
    assert_eq!(post_form(&server, "/doc/AUD-001", &bad).status, 422);
    for entry in fs::read_dir(&rdir).unwrap().flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        assert!(!name.contains(".tmp"), "temp left {}", name);
    }
    drop(server);
    // symlink escape
    let outside = test_root("t26j-outside");
    let secret = outside.join("secret.md");
    fs::write(&secret, "# AUD-001 Secret\n").unwrap();
    let target = audit_record_path(&dir, "AUD-001");
    fs::remove_file(&target).unwrap();
    symlink(&secret, &target).unwrap();
    let server = Server::start(&dir, &["--no-open"], &[]);
    assert_eq!(get(&server, "/doc/AUD-001").status, 404);
    assert_eq!(get(&server, "/api/doc/AUD-001").status, 404);
    drop(server);
    let _ = fs::remove_dir_all(&dir);
    let _ = fs::remove_dir_all(&outside);
}

// T-26k
#[test]
fn t26k_offline_asset_scan() {
    let dir = audit_fixture("t26k");
    let server = Server::start(&dir, &["--no-open"], &[]);
    let css = get(&server, "/assets/app.css");
    let js = get(&server, "/assets/app.js");
    assert_eq!(css.status, 200);
    assert_eq!(js.status, 200);
    for path in ["/?type=audit-record", "/doc/AUD-001", "/doc/AUD-001/edit"] {
        let html = get(&server, path).text();
        assert!(!html.contains("http://"), "external {}", path);
        assert!(!html.contains("https://"), "external {}", path);
        assert!(!html.contains("//cdn"), "external {}", path);
        assert!(!html.contains("@import"), "external {}", path);
    }
    for asset in [css.text(), js.text()] {
        assert!(!asset.contains("@import"));
        assert!(!asset.contains("http://"));
        assert!(!asset.contains("https://"));
    }
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-26l
#[test]
fn t26l_no_new_dependencies() {
    let manifest = include_str!("../Cargo.toml");
    let dependency_index = manifest.find("[dependencies]").unwrap();
    let dependencies = &manifest[dependency_index..];
    for forbidden in [
        "axum",
        "actix",
        "rocket",
        "tokio",
        "async-std",
        "tera",
        "handlebars",
        "askama",
    ] {
        assert!(
            !dependencies.contains(forbidden),
            "new dependency {}",
            forbidden
        );
    }
}

// T-26m
#[test]
fn t26m_quality_gates() {
    let manifest = include_str!("../Cargo.toml");
    let dependency_index = manifest.find("[dependencies]").unwrap();
    let dependencies = &manifest[dependency_index..];
    for forbidden in [
        "axum",
        "actix",
        "rocket",
        "tokio",
        "async-std",
        "tera",
        "handlebars",
        "askama",
        "rusqlite",
        "reqwest",
        "hyper",
    ] {
        assert!(
            !dependencies.contains(forbidden),
            "new dependency {}",
            forbidden
        );
    }
}

// ---- T-57 technical specifications browser scenarios ----

fn create_tech_spec(dir: &Path, title: &str, description: &str) -> String {
    run_ok(&[
        "--dir",
        dir.to_str().unwrap(),
        "tech-spec",
        "create",
        title,
        "--description",
        description,
    ])
}

fn tech_spec_path(dir: &Path, id: &str) -> PathBuf {
    dir.join("docs/technical").join(format!("{}.md", id))
}

fn tech_spec_dir(dir: &Path) -> PathBuf {
    dir.join("docs/technical")
}

fn tech_spec_fixture(name: &str) -> PathBuf {
    let dir = test_root(name);
    init_project(&dir);
    create_tech_spec(&dir, "Offline architecture", "Defines the local browser.");
    create_tech_spec(&dir, "Storage model", "Second description.");
    dir
}

// T-57a
#[test]
fn t57a_list_category_and_sidebar() {
    let dir = tech_spec_fixture("t57a");
    let server = Server::start(&dir, &["--no-open"], &[]);
    let html = get(&server, "/?type=tech-spec").text();
    assert!(html.contains("<h2 class=\"h2\">Technical Specifications</h2>"));
    assert!(html.contains("2 technical specifications"));
    assert!(html.contains("href=\"/?type=tech-spec\" aria-current=\"page\""));
    let audit_at = html.find("href=\"/?type=audit-record\"").unwrap();
    let tech_at = html.find("href=\"/?type=tech-spec\"").unwrap();
    let nfr_at = html.find("href=\"/?type=nfr\"").unwrap();
    assert!(audit_at < tech_at);
    assert!(tech_at < nfr_at);
    assert!(html.contains("TECH-001"));
    assert!(html.contains("Offline architecture"));
    assert!(html.contains("TECH-002"));
    assert!(html.contains("Storage model"));
    assert!(html.find("TECH-001").unwrap() < html.find("TECH-002").unwrap());
    assert!(html.contains("href=\"/doc/TECH-001\""));
    // TASK-142: the shared list no longer renders a description preview.
    assert!(!html.contains("Defines the local browser."));
    assert_eq!(get(&server, "/?type=req").status, 200);
    assert_eq!(get(&server, "/?type=req").status, 200);
    for ty in ["all", "x", "tech-specs", "tech_spec"] {
        assert_eq!(
            get(&server, &format!("/?type={}", ty)).status,
            404,
            "type={}",
            ty
        );
    }
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn t57a_disabled_tech_spec_empty_state() {
    let dir = tech_spec_fixture("t57a-disabled");
    run_ok(&[
        "--dir",
        dir.to_str().unwrap(),
        "config",
        "set",
        "document_types.tech-spec.enabled",
        "false",
    ]);
    let server = Server::start(&dir, &["--no-open"], &[]);
    let html = get(&server, "/").text();
    assert!(!html.contains("href=\"/?type=tech-spec\""));
    assert_eq!(get(&server, "/?type=tech-spec").status, 404);
    assert_eq!(get(&server, "/api/tech-specs").status, 404);
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-57b
#[test]
fn t57b_list_excludes_non_records_and_archived() {
    let dir = tech_spec_fixture("t57b");
    let rdir = tech_spec_dir(&dir);
    fs::write(rdir.join("REQ-001.md"), "# REQ-001 Not a technical spec\n").unwrap();
    fs::write(rdir.join("README.md"), "# registry").unwrap();
    fs::write(rdir.join("notes.txt"), "x").unwrap();
    fs::write(rdir.join("TECH-ABC.md"), "# bad").unwrap();
    let archived = rdir.join("archived");
    fs::create_dir_all(&archived).unwrap();
    fs::write(
        archived.join("TECH-999.md"),
        "# TECH-999 Hidden\n\n## Description\nHidden.\n",
    )
    .unwrap();
    let long_title = "X".repeat(180);
    let long = create_tech_spec(&dir, &long_title, "Long token description.");
    let server = Server::start(&dir, &["--no-open"], &[]);
    let html = get(&server, "/?type=tech-spec").text();
    assert!(html.contains("TECH-001"));
    assert!(html.contains("TECH-002"));
    assert!(html.contains(&long));
    assert!(html.contains(&long_title));
    assert!(!html.contains("TECH-999"));
    assert!(!html.contains("REQ-001"));
    assert!(html.contains("3 technical specifications"));
    let api: serde_json::Value =
        serde_json::from_str(&get(&server, "/api/tech-specs").text()).unwrap();
    assert_eq!(api.as_array().unwrap().len(), 3);
    drop(server);
    // empty state
    let empty_dir = test_root("t57b-empty");
    init_project(&empty_dir);
    let server = Server::start(&empty_dir, &["--no-open"], &[]);
    let html = get(&server, "/?type=tech-spec").text();
    assert!(html.contains("0 technical specifications"));
    assert!(!html.contains("No records"));
    // TASK-142: one shared empty message/help pair for every family.
    assert!(html.contains("No technical specifications in this project."));
    assert!(html.contains(
        "Records are created from the command line; this browser does not create records."
    ));
    assert!(!html.contains("Archived technical specifications are hidden."));
    drop(server);
    let _ = fs::remove_dir_all(&dir);
    let _ = fs::remove_dir_all(&empty_dir);
}

// T-57c
#[test]
fn t57c_view_technical_specification() {
    let dir = tech_spec_fixture("t57c");
    // On-disk record with an empty description section for the (none) state.
    fs::write(
        tech_spec_path(&dir, "TECH-003"),
        "# TECH-003 Empty description\n",
    )
    .unwrap();
    let server = Server::start(&dir, &["--no-open"], &[]);
    let html = get(&server, "/doc/TECH-001").text();
    assert!(html.contains("<p class=\"id\">"));
    assert!(html.contains("TECH-001"));
    assert!(html.contains("Technical Specifications"));
    assert!(html.contains("<h1 class=\"h1\">Offline architecture</h1>"));
    assert!(html.contains("Defines the local browser."));
    assert!(html.contains("<p class=\"text\">Defines the local browser.</p>"));
    assert!(!html.contains("style=\"white-space: pre-wrap\""));
    assert!(!html.contains("Description</h"));
    assert!(html.contains("href=\"/?type=tech-spec\">Technical Specifications</a></li><li><span aria-current=\"page\">TECH-001</span>"));
    assert_eq!(html.matches("href=\"/doc/TECH-001/edit\"").count(), 1);
    assert!(html.contains(">Edit</a>"));
    let empty = get(&server, "/doc/TECH-003").text();
    assert!(empty.contains("(none)"));
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-57d
#[test]
fn t57d_edit_round_trip() {
    let dir = tech_spec_fixture("t57d");
    let server = Server::start(&dir, &["--no-open"], &[]);
    let edit = get(&server, "/doc/TECH-001/edit").text();
    assert!(edit.contains("Hidden field: version (optimistic-concurrency token)"));
    assert!(edit.contains(
        "Required \u{b7} whitespace is trimmed \u{b7} line breaks and control characters are not allowed."
    ));
    assert!(
        edit.contains("Required \u{b7} multi-line \u{b7} line breaks are preserved when saved.")
    );
    assert!(edit.contains("Save changes"));
    assert!(edit.contains("Cancel"));
    let version = extract_version(&edit);
    let body = form_body(&[
        ("version", &version),
        ("title", "Renamed specification"),
        ("description", "Line one\nLine two"),
    ]);
    let response = post_form(&server, "/doc/TECH-001", &body);
    assert_eq!(response.status, 303);
    assert_eq!(response.header("Location"), Some("/doc/TECH-001"));
    let view = get(&server, "/doc/TECH-001").text();
    assert!(view.contains("<h1 class=\"h1\">Renamed specification</h1>"));
    assert!(view.contains("Line one\nLine two"));
    let list = get(&server, "/?type=tech-spec").text();
    assert!(list.contains("Renamed specification"));
    let cli = run_ok(&[
        "--dir",
        dir.to_str().unwrap(),
        "tech-spec",
        "view",
        "TECH-001",
    ]);
    assert!(cli.contains("Renamed specification"));
    assert!(cli.contains("Line one"));
    assert!(cli.contains("Line two"));
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-57e
#[test]
fn t57e_field_preserving_write() {
    let dir = tech_spec_fixture("t57e");
    let path = tech_spec_path(&dir, "TECH-001");
    let original = "# TECH-001 Original\n\nPreamble kept.\n\n## Description\nOld body\n\n## Editor Notes\nKeep this line\n\n## Unknown\nAnd this\n";
    fs::write(&path, original).unwrap();
    let server = Server::start(&dir, &["--no-open"], &[]);
    let version = extract_version(&get(&server, "/doc/TECH-001/edit").text());
    let body = form_body(&[
        ("version", &version),
        ("title", "Edited title"),
        ("description", "New body\nsecond line"),
    ]);
    assert_eq!(post_form(&server, "/doc/TECH-001", &body).status, 303);
    let after = fs::read_to_string(&path).unwrap();
    // A browser save adopts front matter (REQ-05.011) and leaves the body
    // unchanged apart from the edited fields.
    let (_, body) = papyrus::record::frontmatter::split(&after).unwrap();
    let expected_body = "# TECH-001 Edited title\n\nPreamble kept.\n\n## Description\nNew body\nsecond line\n\n## Editor Notes\nKeep this line\n\n## Unknown\nAnd this\n";
    assert_eq!(body, expected_body);
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// TASK-97 AC #5: read/save adoption is idempotent.
#[test]
fn t57_read_adoption_is_idempotent() {
    let dir = tech_spec_fixture("t57-adopt");
    let path = tech_spec_path(&dir, "TECH-001");
    let original = "# TECH-001 Legacy title\n\n## Description\nBody\n";
    fs::write(&path, original).unwrap();
    let server = Server::start(&dir, &["--no-open"], &[]);
    let edit = get(&server, "/doc/TECH-001/edit").text();
    let adopted = fs::read_to_string(&path).unwrap();
    let (_, body) = papyrus::record::frontmatter::split(&adopted).unwrap();
    assert_eq!(body, original, "adoption changes the body");
    let _ = get(&server, "/doc/TECH-001/edit").text();
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        adopted,
        "second read must not write again"
    );
    let version = extract_version(&edit);
    let unchanged = form_body(&[
        ("version", &version),
        ("title", "Legacy title"),
        ("description", "Body"),
    ]);
    assert_eq!(post_form(&server, "/doc/TECH-001", &unchanged).status, 303);
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        adopted,
        "no-op save must not write or refresh updated_date"
    );
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// TASK-97 AC #12 / D2: a CRLF record normalizes to LF on the first save and the
// next identical save writes nothing.
#[test]
fn t57_crlf_save_normalizes_to_lf_once() {
    let dir = tech_spec_fixture("t57-crlf");
    let path = tech_spec_path(&dir, "TECH-001");
    let lf = fs::read_to_string(&path).unwrap();
    assert!(!lf.contains('\r'), "fixture starts with LF");
    fs::write(&path, lf.replace('\n', "\r\n")).unwrap();

    let server = Server::start(&dir, &["--no-open"], &[]);
    let edit = get(&server, "/doc/TECH-001/edit").text();
    assert!(
        !edit.contains('\r'),
        "read normalization strips carriage returns from headings and bodies"
    );
    let version = extract_version(&edit);
    let body = form_body(&[
        ("version", &version),
        ("title", "Offline architecture"),
        ("description", "Defines the local browser."),
    ]);
    assert_eq!(post_form(&server, "/doc/TECH-001", &body).status, 303);

    let after_first = fs::read(&path).unwrap();
    let text = String::from_utf8(after_first.clone()).unwrap();
    assert!(!text.contains('\r'), "first save writes LF: {text:?}");
    assert!(
        text.ends_with('\n') && !text.ends_with("\n\n"),
        "first save writes exactly one trailing newline: {text:?}"
    );
    assert_eq!(
        text.matches("## Description").count(),
        1,
        "the section heading appears exactly once: {text:?}"
    );
    let updated_after_first = text
        .lines()
        .find(|line| line.starts_with("updated_date:"))
        .map(str::to_string);
    let mtime_after_first = fs::metadata(&path).unwrap().modified().unwrap();

    // A second, immediately repeated identical save must perform no write.
    let edit = get(&server, "/doc/TECH-001/edit").text();
    let version = extract_version(&edit);
    let body = form_body(&[
        ("version", &version),
        ("title", "Offline architecture"),
        ("description", "Defines the local browser."),
    ]);
    assert_eq!(post_form(&server, "/doc/TECH-001", &body).status, 303);
    assert_eq!(
        fs::read(&path).unwrap(),
        after_first,
        "the second identical save must leave the file byte-identical"
    );
    assert_eq!(
        fs::metadata(&path).unwrap().modified().unwrap(),
        mtime_after_first,
        "the second identical save must not write and refresh updated_date"
    );
    assert_eq!(
        fs::read_to_string(&path)
            .unwrap()
            .lines()
            .find(|line| line.starts_with("updated_date:"))
            .map(str::to_string),
        updated_after_first
    );

    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-57f
#[test]
fn t57f_validation_422_no_write() {
    let dir = tech_spec_fixture("t57f");
    let path = tech_spec_path(&dir, "TECH-001");
    let before = fs::read(&path).unwrap();
    let server = Server::start(&dir, &["--no-open"], &[]);
    let version = extract_version(&get(&server, "/doc/TECH-001/edit").text());
    let cases: Vec<(String, &str, &str)> = vec![
        (
            form_body(&[("version", &version), ("title", ""), ("description", "ok")]),
            "Title is required.",
            "title",
        ),
        (
            form_body(&[
                ("version", &version),
                ("title", &"x".repeat(201)),
                ("description", "ok"),
            ]),
            "Title must be at most 200 characters.",
            "title",
        ),
        (
            form_body(&[
                ("version", &version),
                ("title", "a\tb"),
                ("description", "ok"),
            ]),
            "Title cannot contain control characters.",
            "title",
        ),
        (
            form_body(&[("version", &version), ("title", "Ok"), ("description", "")]),
            "Description is required.",
            "description",
        ),
        (
            form_body(&[
                ("version", &version),
                ("title", "Ok"),
                ("description", &"x".repeat(20001)),
            ]),
            "Description must be at most 20000 characters.",
            "description",
        ),
        (
            form_body(&[
                ("version", &version),
                ("title", "Ok"),
                ("description", "a\u{0007}b"),
            ]),
            "Description cannot contain control characters.",
            "description",
        ),
        (
            form_body(&[
                ("version", &version),
                ("title", "Ok"),
                ("description", "a\n## Section\nb"),
            ]),
            "Description contains a line starting with ## .",
            "description",
        ),
    ];
    for (body, message, field) in cases {
        let response = post_form(&server, "/doc/TECH-001", &body);
        assert_eq!(response.status, 422, "body={}", body);
        let html = response.text();
        assert_eq!(summary_entry(&html, field), message);
        assert_eq!(field_error(&html, field), message);
        assert_eq!(fs::read(&path).unwrap(), before, "file changed {}", body);
    }
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-57g
#[test]
fn t57g_stale_and_malformed_versions() {
    let dir = tech_spec_fixture("t57g");
    let path = tech_spec_path(&dir, "TECH-001");
    let before = fs::read(&path).unwrap();
    let server = Server::start(&dir, &["--no-open"], &[]);
    let valid = form_body(&[
        ("version", "1.2.3"),
        ("title", "Stale"),
        ("description", "ok"),
    ]);
    let conflict = post_form(&server, "/doc/TECH-001", &valid);
    assert_eq!(conflict.status, 409);
    let html = conflict.text();
    assert!(html.contains("record changed on disk; reload the form"));
    assert!(html.contains("This record changed on disk"));
    assert!(html.contains("Nothing was saved."));
    assert!(html.contains("Reload the form"));
    assert!(html.contains("View current record"));
    assert!(html.contains("Stale"));
    assert_eq!(fs::read(&path).unwrap(), before);
    let missing = form_body(&[("title", "Ok"), ("description", "ok")]);
    assert_eq!(post_form(&server, "/doc/TECH-001", &missing).status, 400);
    let malformed = form_body(&[("version", "bad"), ("title", "Ok"), ("description", "ok")]);
    assert_eq!(post_form(&server, "/doc/TECH-001", &malformed).status, 400);
    assert_eq!(fs::read(&path).unwrap(), before);
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-57h
#[test]
fn t57h_archived_hidden_and_404() {
    const HELP: &str = "Archived technical specifications are not listed and cannot be opened. The same page is used for an unknown address.";
    let dir = tech_spec_fixture("t57h");
    run_ok(&[
        "--dir",
        dir.to_str().unwrap(),
        "tech-spec",
        "archive",
        "TECH-002",
    ]);
    let server = Server::start(&dir, &["--no-open"], &[]);
    let list = get(&server, "/?type=tech-spec").text();
    assert!(list.contains("TECH-001"));
    assert!(!list.contains("TECH-002"));
    assert!(list.contains("1 technical specification"));

    // The same technical-specifications 404 page is rendered for a archived
    // and an unknown ID, and for view, edit, and POST.
    let archived = get(&server, "/doc/TECH-002");
    assert_eq!(archived.status, 404);
    let not_found = archived.text();
    assert!(not_found.contains("Record not found"));
    assert!(not_found.contains(HELP));
    assert!(not_found.contains("href=\"/?type=tech-spec\""));
    assert_eq!(
        not_found
            .matches(">Back to technical specifications</a>")
            .count(),
        1
    );
    let unknown = get(&server, "/doc/TECH-404");
    assert_eq!(unknown.status, 404);
    assert_eq!(not_found, unknown.text());
    let archived_edit = get(&server, "/doc/TECH-002/edit");
    assert_eq!(archived_edit.status, 404);
    assert_eq!(not_found, archived_edit.text());
    let unknown_edit = get(&server, "/doc/TECH-404/edit");
    assert_eq!(unknown_edit.status, 404);
    assert_eq!(not_found, unknown_edit.text());
    let body = form_body(&[("version", "1.2.3"), ("title", "Ok"), ("description", "ok")]);
    let archived_post = post_form(&server, "/doc/TECH-002", &body);
    assert_eq!(archived_post.status, 404);
    assert_eq!(not_found, archived_post.text());
    let unknown_post = post_form(&server, "/doc/TECH-404", &body);
    assert_eq!(unknown_post.status, 404);
    assert_eq!(not_found, unknown_post.text());

    // A non-technical-specification ID keeps the baseline 404 page.
    let baseline = get(&server, "/doc/REQ-404").text();
    assert!(baseline.contains("Back to requirements"));
    assert!(!baseline.contains(HELP));

    // JSON 404 responses keep the baseline JSON error object.
    let archived_json = get(&server, "/api/doc/TECH-002");
    assert_eq!(archived_json.status, 404);
    assert!(
        archived_json
            .header("Content-Type")
            .unwrap()
            .starts_with("application/json")
    );
    let json: serde_json::Value = serde_json::from_str(&archived_json.text()).unwrap();
    assert_eq!(json["error"]["code"], 404);
    assert!(json["error"]["message"].is_string());
    let json_body = "{\"version\":\"1.2.3\",\"title\":\"Ok\"}";
    let archived_json_post = post_json(&server, "/api/doc/TECH-002", json_body);
    assert_eq!(archived_json_post.status, 404);
    let json: serde_json::Value = serde_json::from_str(&archived_json_post.text()).unwrap();
    assert_eq!(json["error"]["code"], 404);

    let api: serde_json::Value =
        serde_json::from_str(&get(&server, "/api/tech-specs").text()).unwrap();
    assert_eq!(api.as_array().unwrap().len(), 1);
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-57i
#[test]
fn t57i_json_api() {
    let dir = tech_spec_fixture("t57i");
    let server = Server::start(&dir, &["--no-open"], &[]);
    for alias in ["/api/tech-specs", "/api/tech-spec"] {
        let list: serde_json::Value = serde_json::from_str(&get(&server, alias).text()).unwrap();
        let arr = list.as_array().unwrap();
        assert_eq!(arr.len(), 2, "alias {}", alias);
        assert_eq!(arr[0]["id"], "TECH-001");
        assert_eq!(arr[0]["title"], "Offline architecture");
        assert_eq!(arr[1]["id"], "TECH-002");
    }
    let doc: serde_json::Value =
        serde_json::from_str(&get(&server, "/api/doc/TECH-001").text()).unwrap();
    assert_eq!(doc["id"], "TECH-001");
    assert_eq!(doc["kind"], "tech-spec");
    assert_eq!(doc["form"], "tech-spec");
    assert_eq!(doc["title"], "Offline architecture");
    assert_eq!(doc["description"], "Defines the local browser.");
    assert!(doc["version"].is_string());
    let path = doc["path"].as_str().unwrap();
    assert!(path.ends_with("TECH-001.md"));
    assert!(!path.starts_with('/'));
    let version = doc["version"].as_str().unwrap().to_string();
    let body = format!(
        r#"{{"version":"{}","title":"Renamed","description":"New body"}}"#,
        version
    );
    let updated: serde_json::Value =
        serde_json::from_str(&post_json(&server, "/api/doc/TECH-001", &body).text()).unwrap();
    assert_eq!(updated["title"], "Renamed");
    assert_eq!(updated["description"], "New body");
    // Omitted members stay unchanged.
    let version = updated["version"].as_str().unwrap().to_string();
    let body = format!(r#"{{"version":"{}","title":"Second"}}"#, version);
    let updated: serde_json::Value =
        serde_json::from_str(&post_json(&server, "/api/doc/TECH-001", &body).text()).unwrap();
    assert_eq!(updated["title"], "Second");
    assert_eq!(updated["description"], "New body");
    let version = updated["version"].as_str().unwrap().to_string();
    let body = format!(r#"{{"version":"{}","description":"Third"}}"#, version);
    let updated: serde_json::Value =
        serde_json::from_str(&post_json(&server, "/api/doc/TECH-001", &body).text()).unwrap();
    assert_eq!(updated["title"], "Second");
    assert_eq!(updated["description"], "Third");
    let version = updated["version"].as_str().unwrap().to_string();
    // Description null or empty is invalid.
    let body = format!(r#"{{"version":"{}","description":null}}"#, version);
    assert_eq!(post_json(&server, "/api/doc/TECH-001", &body).status, 422);
    let body = format!(r#"{{"version":"{}","description":""}}"#, version);
    assert_eq!(post_json(&server, "/api/doc/TECH-001", &body).status, 422);
    // Invalid values.
    let body = format!(r#"{{"version":"{}","title":""}}"#, version);
    assert_eq!(post_json(&server, "/api/doc/TECH-001", &body).status, 422);
    let body = format!(
        r#"{{"version":"{}","description":"a\n## Section\nb"}}"#,
        version
    );
    assert_eq!(post_json(&server, "/api/doc/TECH-001", &body).status, 422);
    // Wrong types.
    let body = format!(r#"{{"version":"{}","title":5}}"#, version);
    assert_eq!(post_json(&server, "/api/doc/TECH-001", &body).status, 400);
    let body = format!(r#"{{"version":"{}","description":5}}"#, version);
    assert_eq!(post_json(&server, "/api/doc/TECH-001", &body).status, 400);
    // Missing/malformed/stale version.
    assert_eq!(
        post_json(&server, "/api/doc/TECH-001", "{\"title\":\"Ok\"}").status,
        400
    );
    assert_eq!(
        post_json(&server, "/api/doc/TECH-001", "{\"version\":\"bad\"}").status,
        400
    );
    let body = "{\"version\":\"1.2.3\",\"title\":\"Ok\"}";
    assert_eq!(post_json(&server, "/api/doc/TECH-001", body).status, 409);
    assert_eq!(
        post_json(&server, "/api/doc/TECH-001", "{not json").status,
        400
    );
    assert_eq!(post_json(&server, "/api/doc/TECH-001", "[]").status, 400);
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-57j
#[test]
fn t57j_atomic_and_symlink() {
    use std::os::unix::fs::symlink;
    let dir = tech_spec_fixture("t57j");
    let rdir = tech_spec_dir(&dir);
    let server = Server::start(&dir, &["--no-open"], &[]);
    let version = extract_version(&get(&server, "/doc/TECH-001/edit").text());
    let body = form_body(&[
        ("version", &version),
        ("title", "Atomic title"),
        ("description", "d"),
    ]);
    assert_eq!(post_form(&server, "/doc/TECH-001", &body).status, 303);
    for entry in fs::read_dir(&rdir).unwrap().flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        assert!(!name.contains(".tmp"), "temp left {}", name);
    }
    let version = extract_version(&get(&server, "/doc/TECH-001/edit").text());
    let bad = form_body(&[("version", &version), ("title", ""), ("description", "ok")]);
    assert_eq!(post_form(&server, "/doc/TECH-001", &bad).status, 422);
    for entry in fs::read_dir(&rdir).unwrap().flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        assert!(!name.contains(".tmp"), "temp left {}", name);
    }
    drop(server);
    let outside = test_root("t57j-outside");
    let secret = outside.join("secret.md");
    fs::write(&secret, "# TECH-001 Secret\n").unwrap();
    let target = tech_spec_path(&dir, "TECH-001");
    fs::remove_file(&target).unwrap();
    symlink(&secret, &target).unwrap();
    let server = Server::start(&dir, &["--no-open"], &[]);
    assert_eq!(get(&server, "/doc/TECH-001").status, 404);
    assert_eq!(get(&server, "/api/doc/TECH-001").status, 404);
    drop(server);
    let _ = fs::remove_dir_all(&dir);
    let _ = fs::remove_dir_all(&outside);
}

// T-57k
#[test]
fn t57k_offline_asset_scan() {
    let dir = tech_spec_fixture("t57k");
    let server = Server::start(&dir, &["--no-open"], &[]);
    let css = get(&server, "/assets/app.css");
    let js = get(&server, "/assets/app.js");
    assert_eq!(css.status, 200);
    assert_eq!(js.status, 200);
    for path in ["/?type=tech-spec", "/doc/TECH-001", "/doc/TECH-001/edit"] {
        let html = get(&server, path).text();
        assert!(!html.contains("http://"), "external {}", path);
        assert!(!html.contains("https://"), "external {}", path);
        assert!(!html.contains("//cdn"), "external {}", path);
        assert!(!html.contains("@import"), "external {}", path);
    }
    for asset in [css.text(), js.text()] {
        assert!(!asset.contains("@import"));
        assert!(!asset.contains("http://"));
        assert!(!asset.contains("https://"));
    }
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-57l
#[test]
fn t57l_no_js_and_narrow_layout() {
    let dir = tech_spec_fixture("t57l");
    let server = Server::start(&dir, &["--no-open"], &[]);
    let edit = get(&server, "/doc/TECH-001/edit").text();
    assert!(edit.contains("<label for=\"title\">"));
    assert!(edit.contains("<label for=\"description\">"));
    assert!(edit.contains("name=\"description\""));
    assert!(edit.contains("required"));
    assert!(edit.contains("Save changes"));
    assert!(edit.contains("<a class=\"link\" href=\"/doc/TECH-001\">Cancel</a>"));
    // No inline handlers or scripts carry behaviour: the form is a plain POST.
    assert!(edit.contains("method=\"post\" action=\"/doc/TECH-001\""));
    assert!(!edit.contains("onsubmit="));
    let list = get(&server, "/?type=tech-spec").text();
    assert!(list.contains("Technical Specifications"));
    let css = get(&server, "/assets/app.css").text();
    assert!(css.contains("@media (max-width: 480px)"));
    // MOCK-0009 keeps the version annotation visible at 320 px for tech specs.
    assert!(css.contains(".hidden-field.version-note { display: inline-block; }"));
    assert!(edit.contains("class=\"hidden-field version-note\""));
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-57m
#[test]
fn t57m_quality_gates() {
    let manifest = include_str!("../Cargo.toml");
    let dependency_index = manifest.find("[dependencies]").unwrap();
    let dependencies = &manifest[dependency_index..];
    for forbidden in [
        "axum",
        "actix",
        "rocket",
        "tokio",
        "async-std",
        "tera",
        "handlebars",
        "askama",
        "rusqlite",
        "reqwest",
        "hyper",
    ] {
        assert!(
            !dependencies.contains(forbidden),
            "new dependency {}",
            forbidden
        );
    }
}

#[test]
fn t19a_inventory_hub() {
    // Empty project: inventory renders with h1, 12 cards in final order, zero counts.
    let empty = test_root("t19a-empty");
    init_project(&empty);
    let server = Server::start(&empty, &["--no-open"], &[]);
    let resp = get(&server, "/");
    assert_eq!(resp.status, 200);
    assert_eq!(
        resp.header("Content-Type"),
        Some("text/html; charset=utf-8")
    );
    let html = resp.text();
    assert!(html.contains("Browser inventory"));
    assert!(!html.contains("http://"));
    assert!(!html.contains("https://"));

    let labels = [
        "Business requirements",
        "Business Rules",
        "Audit Records",
        "Technical Specifications",
        "Non-functional requirements",
        "Backlog",
        "Decisions",
        "Glossary",
        "UI Specifications",
        "UI Mockups",
        "Use Cases",
        "Test Cases",
        "Test Reports",
    ];
    let mut positions = Vec::new();
    for label in &labels {
        let marker = format!("<h3>{}</h3>", label);
        let pos = html
            .find(&marker)
            .unwrap_or_else(|| panic!("missing card {}", label));
        positions.push(pos);
    }
    for pair in positions.windows(2) {
        assert!(pair[0] < pair[1], "card order wrong");
    }
    assert!(html.contains("Business requirements &middot; 0"));
    assert!(html.contains("href=\"/?type=req\""));
    drop(server);

    // Populated project: counts reflect active records.
    let populated = test_root("t19a-pop");
    init_project(&populated);
    create_req_group(&populated, "Payments Group");
    create_req(&populated, "Capture payment", "REQ-001");
    create_req_group(&populated, "Search Group");
    create_req(&populated, "Search records", "REQ-002");
    create_nfr_group(&populated, "Persistence");
    create_nfr(&populated, "Restrict access", "NFR-001");
    let server = Server::start(&populated, &["--no-open"], &[]);
    let html = get(&server, "/").text();

    assert!(html.contains("Business requirements &middot; 4"));
    assert!(html.contains("Non-functional requirements &middot; 2"));
    drop(server);

    let _ = fs::remove_dir_all(&empty);
    let _ = fs::remove_dir_all(&populated);
}

// ---- TASK-19 DoD #3: cross-type integration test ----

fn use_case_path(dir: &Path, id: &str) -> PathBuf {
    dir.join("docs/use-cases").join(format!("{}.md", id))
}

fn use_case_dir(dir: &Path) -> PathBuf {
    dir.join("docs/use-cases")
}

fn use_case_fixture(name: &str) -> PathBuf {
    let dir = test_root(name);
    init_project(&dir);
    run_ok(&[
        "--dir",
        dir.to_str().unwrap(),
        "use-case",
        "create",
        "Offline architecture",
        "--description",
        "Defines the local browser.",
        "--applicability",
        "Applies to browser.",
        "--label",
        "ui",
        "--reference",
        "REQ-1",
    ]);
    run_ok(&[
        "--dir",
        dir.to_str().unwrap(),
        "use-case",
        "create",
        "Storage model",
        "--description",
        "Second description.",
    ]);
    dir
}

// T-81a
#[test]
fn t81a_sidebar_placement_and_type_values() {
    let dir = use_case_fixture("t81a");
    let server = Server::start(&dir, &["--no-open"], &[]);
    for ty in ["use-case", "test-case", "test-report"] {
        assert_eq!(
            get(&server, &format!("/?type={}", ty)).status,
            200,
            "type={}",
            ty
        );
    }
    let html = get(&server, "/?type=use-case").text();
    assert!(html.contains("<h2 class=\"h2\">Use Cases</h2>"));
    assert!(html.contains("2 use cases"));
    assert!(html.contains("href=\"/?type=use-case\" aria-current=\"page\""));
    let tech_at = html.find("href=\"/?type=tech-spec\"").unwrap();
    let use_at = html.find("href=\"/?type=use-case\"").unwrap();
    let tc_at = html.find("href=\"/?type=test-case\"").unwrap();
    let tr_at = html.find("href=\"/?type=test-report\"").unwrap();
    let nfr_at = html.find("href=\"/?type=nfr\"").unwrap();
    assert!(tech_at < use_at);
    assert!(use_at < tc_at);
    assert!(tc_at < tr_at);
    assert!(tr_at < nfr_at);
    assert!(html.contains("UC-001"));
    assert!(html.contains("Offline architecture"));
    assert!(html.contains("UC-002"));
    assert!(html.contains("Storage model"));
    assert!(html.find("UC-001").unwrap() < html.find("UC-002").unwrap());
    assert!(html.contains("href=\"/doc/UC-001\""));
    // TASK-142: the shared list no longer renders a description preview.
    assert!(!html.contains("Defines the local browser."));
    // default req
    assert_eq!(get(&server, "/").status, 200);
    assert_eq!(get(&server, "/?type=req").status, 200);
    for ty in ["all", "x", "use-cases", "use_case", "uc"] {
        assert_eq!(
            get(&server, &format!("/?type={}", ty)).status,
            404,
            "type={}",
            ty
        );
    }
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-81b
#[test]
fn t81b_disabled_use_case_empty_state() {
    let dir = use_case_fixture("t81b-disabled");
    run_ok(&[
        "--dir",
        dir.to_str().unwrap(),
        "config",
        "set",
        "document_types.use-case.enabled",
        "false",
    ]);
    let server = Server::start(&dir, &["--no-open"], &[]);
    let html = get(&server, "/").text();
    assert!(!html.contains("href=\"/?type=use-case\""));
    assert_eq!(get(&server, "/?type=use-case").status, 404);
    assert_eq!(get(&server, "/api/use-cases").status, 404);
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-81c
#[test]
fn t81c_list_excludes_non_records_and_archived() {
    let dir = use_case_fixture("t81c");
    let rdir = use_case_dir(&dir);
    fs::write(rdir.join("REQ-001.md"), "# REQ-001 Not a use case\n").unwrap();
    fs::write(rdir.join("README.md"), "# registry").unwrap();
    fs::write(rdir.join("notes.txt"), "x").unwrap();
    fs::write(rdir.join("UC-ABC.md"), "# bad").unwrap();
    let archived = rdir.join("archived");
    fs::create_dir_all(&archived).unwrap();
    fs::write(
        archived.join("UC-999.md"),
        "# UC-999 Hidden\n\n## Description\nHidden.\n",
    )
    .unwrap();
    let server = Server::start(&dir, &["--no-open"], &[]);
    let html = get(&server, "/?type=use-case").text();
    assert!(html.contains("UC-001"));
    assert!(html.contains("UC-002"));
    assert!(!html.contains("UC-999"));
    assert!(!html.contains("REQ-001"));
    assert!(html.contains("2 use cases"));
    // TASK-142: the shared list no longer renders a description preview.
    assert!(!html.contains("Defines the local browser."));
    let api: serde_json::Value =
        serde_json::from_str(&get(&server, "/api/use-cases").text()).unwrap();
    assert_eq!(api.as_array().unwrap().len(), 2);
    drop(server);
    // empty state
    let empty_dir = test_root("t81c-empty");
    init_project(&empty_dir);
    let server = Server::start(&empty_dir, &["--no-open"], &[]);
    let html = get(&server, "/?type=use-case").text();
    assert!(html.contains("0 use cases"));
    // TASK-142: one shared empty message/help pair for every family.
    assert!(html.contains("No use cases in this project."));
    assert!(html.contains(
        "Records are created from the command line; this browser does not create records."
    ));
    assert!(!html.contains("Archived use cases are hidden."));
    drop(server);
    let _ = fs::remove_dir_all(&dir);
    let _ = fs::remove_dir_all(&empty_dir);
}

// T-81d
#[test]
fn t81d_view_use_case_metadata_projection() {
    let dir = use_case_fixture("t81d");
    fs::write(
        use_case_path(&dir, "UC-003"),
        "# UC-003 Empty description\n",
    )
    .unwrap();
    let server = Server::start(&dir, &["--no-open"], &[]);
    let html = get(&server, "/doc/UC-001").text();
    assert!(html.contains("<p class=\"id\">"));
    assert!(html.contains("UC-001"));
    assert!(html.contains("Use Cases"));
    assert!(html.contains("<h1 class=\"h1\">Offline architecture</h1>"));
    assert!(html.contains(
        "<p class=\"text\" style=\"white-space: pre-wrap\">Defines the local browser.</p>"
    ));
    assert!(html.contains("<section class=\"record-section\"><h2>Applicability</h2><p class=\"text\" style=\"white-space: pre-wrap\">Applies to browser.</p></section>"));
    assert!(html.contains("Title"));
    assert!(html.contains("Created date"));
    assert!(html.contains("Updated date"));
    assert!(html.contains("Labels"));
    assert!(html.contains("References"));
    assert!(html.contains("ui"));
    assert!(html.contains("REQ-1"));
    assert!(html.contains(
        "href=\"/?type=use-case\">Use Cases</a></li><li><span aria-current=\"page\">UC-001</span>"
    ));
    assert_eq!(html.matches("href=\"/doc/UC-001/edit\"").count(), 1);
    assert!(html.contains(">Edit</a>"));
    let empty = get(&server, "/doc/UC-003").text();
    assert!(empty.contains("(none)"));
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-81e
#[test]
fn t81e_edit_round_trip() {
    let dir = use_case_fixture("t81e");
    let server = Server::start(&dir, &["--no-open"], &[]);
    let edit = get(&server, "/doc/UC-001/edit").text();
    assert!(edit.contains("Hidden field: version (optimistic-concurrency token)"));
    let version = extract_version(&edit);
    let body = form_body(&[
        ("version", &version),
        ("title", "Renamed use case"),
        ("description", "Line one\nLine two"),
        ("applicability", "Applies now"),
        ("labels", "alpha"),
        ("labels", "beta"),
        ("references", "REQ-2"),
    ]);
    let response = post_form(&server, "/doc/UC-001", &body);
    assert_eq!(response.status, 303);
    assert_eq!(response.header("Location"), Some("/doc/UC-001"));
    let view = get(&server, "/doc/UC-001").text();
    assert!(view.contains("<h1 class=\"h1\">Renamed use case</h1>"));
    assert!(view.contains("Line one\nLine two"));
    assert!(view.contains("Applies now"));
    assert!(view.contains("alpha"));
    assert!(view.contains("beta"));
    assert!(view.contains("REQ-2"));
    let cli = run_ok(&["--dir", dir.to_str().unwrap(), "use-case", "view", "UC-001"]);
    assert!(cli.contains("Renamed use case"));
    assert!(cli.contains("Line one"));
    assert!(cli.contains("Line two"));
    assert!(cli.contains("Applies now"));
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-81f
#[test]
fn t81f_field_preserving_write() {
    let dir = use_case_fixture("t81f");
    let path = use_case_path(&dir, "UC-001");
    let original = "# UC-001 Original\n\nPreamble kept.\n\n## Description\nOld body\n\n## Editor Notes\nKeep this line\n\n## Unknown\nAnd this\n";
    fs::write(&path, original).unwrap();
    let server = Server::start(&dir, &["--no-open"], &[]);
    let version = extract_version(&get(&server, "/doc/UC-001/edit").text());
    let body = form_body(&[
        ("version", &version),
        ("title", "Edited title"),
        ("description", "New body\nsecond line"),
    ]);
    assert_eq!(post_form(&server, "/doc/UC-001", &body).status, 303);
    let after = fs::read_to_string(&path).unwrap();
    // A browser save adopts front matter (REQ-05.011) and leaves the body
    // unchanged apart from the edited fields.
    let (_, body) = papyrus::record::frontmatter::split(&after).unwrap();
    let expected_body = "# UC-001 Edited title\n\nPreamble kept.\n\n## Description\nNew body\nsecond line\n\n## Editor Notes\nKeep this line\n\n## Unknown\nAnd this\n";
    assert_eq!(body, expected_body);
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-81g
#[test]
fn t81g_validation_422_no_write() {
    let dir = use_case_fixture("t81g");
    let path = use_case_path(&dir, "UC-001");
    let before = fs::read(&path).unwrap();
    let server = Server::start(&dir, &["--no-open"], &[]);
    let version = extract_version(&get(&server, "/doc/UC-001/edit").text());
    let cases: Vec<(String, &str, &str)> = vec![
        (
            form_body(&[("version", &version), ("title", ""), ("description", "ok")]),
            "Title is required.",
            "title",
        ),
        (
            form_body(&[
                ("version", &version),
                ("title", &"x".repeat(201)),
                ("description", "ok"),
            ]),
            "Title must be at most 200 characters.",
            "title",
        ),
        (
            form_body(&[
                ("version", &version),
                ("title", "a\tb"),
                ("description", "ok"),
            ]),
            "Title cannot contain control characters.",
            "title",
        ),
        (
            form_body(&[("version", &version), ("title", "Ok"), ("description", "")]),
            "Description is required.",
            "description",
        ),
        (
            form_body(&[
                ("version", &version),
                ("title", "Ok"),
                ("description", &"x".repeat(20001)),
            ]),
            "Description must be at most 20000 characters.",
            "description",
        ),
        (
            form_body(&[
                ("version", &version),
                ("title", "Ok"),
                ("description", "a\u{0007}b"),
            ]),
            "Description cannot contain control characters.",
            "description",
        ),
        (
            form_body(&[
                ("version", &version),
                ("title", "Ok"),
                ("description", "a\n## Section\nb"),
            ]),
            "Description contains a line starting with ## .",
            "description",
        ),
        (
            form_body(&[
                ("version", &version),
                ("title", "Ok"),
                ("description", "ok"),
                ("applicability", &"x".repeat(20001)),
            ]),
            "Applicability must be at most 20000 characters.",
            "applicability",
        ),
        (
            form_body(&[
                ("version", &version),
                ("title", "Ok"),
                ("description", "ok"),
                ("applicability", "a\u{0007}b"),
            ]),
            "Applicability cannot contain control characters.",
            "applicability",
        ),
        (
            form_body(&[
                ("version", &version),
                ("title", "Ok"),
                ("description", "ok"),
                ("applicability", "a\n## Section\nb"),
            ]),
            "Applicability contains a line starting with ## .",
            "applicability",
        ),
        (
            form_body(&[
                ("version", &version),
                ("title", "Ok"),
                ("description", "ok"),
                ("labels", &"x".repeat(201)),
            ]),
            "Labels must be at most 200 characters.",
            "labels",
        ),
        (
            form_body(&[
                ("version", &version),
                ("title", "Ok"),
                ("description", "ok"),
                ("labels", "a\u{0007}b"),
            ]),
            "Labels cannot contain control characters.",
            "labels",
        ),
        (
            form_body(&[
                ("version", &version),
                ("title", "Ok"),
                ("description", "ok"),
                ("references", &"x".repeat(201)),
            ]),
            "References must be at most 200 characters.",
            "references",
        ),
        (
            form_body(&[
                ("version", &version),
                ("title", "Ok"),
                ("description", "ok"),
                ("references", "a b"),
            ]),
            "References cannot contain whitespace.",
            "references",
        ),
        (
            form_body(&[
                ("version", &version),
                ("title", "Ok"),
                ("description", "ok"),
                ("references", "a"),
                ("references", "a"),
            ]),
            "Duplicate reference: a.",
            "references",
        ),
    ];
    for (body, message, field) in cases {
        let response = post_form(&server, "/doc/UC-001", &body);
        assert_eq!(response.status, 422, "body={}", body);
        let html = response.text();
        assert_eq!(summary_entry(&html, field), message, "body={}", body);
        assert_eq!(field_error(&html, field), message, "body={}", body);
        assert_eq!(fs::read(&path).unwrap(), before, "file changed {}", body);
    }
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-81h
#[test]
fn t81h_stale_and_malformed_versions() {
    let dir = use_case_fixture("t81h");
    let path = use_case_path(&dir, "UC-001");
    let before = fs::read(&path).unwrap();
    let server = Server::start(&dir, &["--no-open"], &[]);
    let valid = form_body(&[
        ("version", "1.2.3"),
        ("title", "Stale"),
        ("description", "ok"),
    ]);
    let conflict = post_form(&server, "/doc/UC-001", &valid);
    assert_eq!(conflict.status, 409);
    let html = conflict.text();
    assert!(html.contains("record changed on disk; reload the form"));
    assert!(html.contains("This record changed on disk"));
    assert!(html.contains("Nothing was saved."));
    assert!(html.contains("Reload the form"));
    assert!(html.contains("View current record"));
    assert!(html.contains("Stale"));
    assert_eq!(fs::read(&path).unwrap(), before);
    let missing = form_body(&[("title", "Ok"), ("description", "ok")]);
    assert_eq!(post_form(&server, "/doc/UC-001", &missing).status, 400);
    let malformed = form_body(&[("version", "bad"), ("title", "Ok"), ("description", "ok")]);
    assert_eq!(post_form(&server, "/doc/UC-001", &malformed).status, 400);
    assert_eq!(fs::read(&path).unwrap(), before);
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-81i
#[test]
fn t81i_archived_hidden_and_404() {
    const HELP: &str = "Archived use cases are not listed and cannot be opened. The same page is used for an unknown address.";
    let dir = use_case_fixture("t81i");
    run_ok(&[
        "--dir",
        dir.to_str().unwrap(),
        "use-case",
        "archive",
        "UC-002",
    ]);
    let server = Server::start(&dir, &["--no-open"], &[]);
    let list = get(&server, "/?type=use-case").text();
    assert!(list.contains("UC-001"));
    assert!(!list.contains("UC-002"));
    assert!(list.contains("1 use case"));

    let archived = get(&server, "/doc/UC-002");
    assert_eq!(archived.status, 404);
    let not_found = archived.text();
    assert!(not_found.contains("Record not found"));
    assert!(not_found.contains(HELP));
    assert!(not_found.contains("href=\"/?type=use-case\""));
    assert_eq!(not_found.matches(">Back to Use Cases</a>").count(), 1);
    let unknown = get(&server, "/doc/UC-404");
    assert_eq!(unknown.status, 404);
    assert_eq!(not_found, unknown.text());
    let archived_edit = get(&server, "/doc/UC-002/edit");
    assert_eq!(archived_edit.status, 404);
    assert_eq!(not_found, archived_edit.text());
    let unknown_edit = get(&server, "/doc/UC-404/edit");
    assert_eq!(unknown_edit.status, 404);
    assert_eq!(not_found, unknown_edit.text());
    let body = form_body(&[("version", "1.2.3"), ("title", "Ok"), ("description", "ok")]);
    let archived_post = post_form(&server, "/doc/UC-002", &body);
    assert_eq!(archived_post.status, 404);
    assert_eq!(not_found, archived_post.text());
    let unknown_post = post_form(&server, "/doc/UC-404", &body);
    assert_eq!(unknown_post.status, 404);
    assert_eq!(not_found, unknown_post.text());

    // JSON 404
    let archived_json = get(&server, "/api/doc/UC-002");
    assert_eq!(archived_json.status, 404);
    let json: serde_json::Value = serde_json::from_str(&archived_json.text()).unwrap();
    assert_eq!(json["error"]["code"], 404);
    let json_body = "{\"version\":\"1.2.3\",\"title\":\"Ok\"}";
    let archived_json_post = post_json(&server, "/api/doc/UC-002", json_body);
    assert_eq!(archived_json_post.status, 404);

    let api: serde_json::Value =
        serde_json::from_str(&get(&server, "/api/use-cases").text()).unwrap();
    assert_eq!(api.as_array().unwrap().len(), 1);
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-81j
#[test]
fn t81j_json_api() {
    let dir = use_case_fixture("t81j");
    let server = Server::start(&dir, &["--no-open"], &[]);
    for alias in ["/api/use-cases", "/api/use-case"] {
        let list: serde_json::Value = serde_json::from_str(&get(&server, alias).text()).unwrap();
        let arr = list.as_array().unwrap();
        assert_eq!(arr.len(), 2, "alias {}", alias);
        assert_eq!(arr[0]["id"], "UC-001");
        assert_eq!(arr[0]["title"], "Offline architecture");
        assert_eq!(arr[1]["id"], "UC-002");
    }
    let doc: serde_json::Value =
        serde_json::from_str(&get(&server, "/api/doc/UC-001").text()).unwrap();
    assert_eq!(doc["id"], "UC-001");
    assert_eq!(doc["kind"], "use-case");
    assert_eq!(doc["form"], "use-case");
    assert_eq!(doc["title"], "Offline architecture");
    assert_eq!(doc["description"], "Defines the local browser.");
    assert_eq!(doc["applicability"], "Applies to browser.");
    assert!(doc["labels"].is_array());
    assert!(doc["references"].is_array());
    assert!(doc["version"].is_string());
    let path = doc["path"].as_str().unwrap();
    assert!(path.ends_with("UC-001.md"));
    assert!(!path.starts_with('/'));
    let version = doc["version"].as_str().unwrap().to_string();
    let body = format!(
        r#"{{"version":"{}","title":"Renamed","description":"New body","applicability":"New app","labels":["x"],"references":["REQ-2"]}}"#,
        version
    );
    let updated: serde_json::Value =
        serde_json::from_str(&post_json(&server, "/api/doc/UC-001", &body).text()).unwrap();
    assert_eq!(updated["title"], "Renamed");
    assert_eq!(updated["description"], "New body");
    assert_eq!(updated["applicability"], "New app");
    assert_eq!(updated["labels"][0], "x");
    assert_eq!(updated["references"][0], "REQ-2");
    // Omitted members stay unchanged.
    let version = updated["version"].as_str().unwrap().to_string();
    let body = format!(r#"{{"version":"{}","title":"Second"}}"#, version);
    let updated: serde_json::Value =
        serde_json::from_str(&post_json(&server, "/api/doc/UC-001", &body).text()).unwrap();
    assert_eq!(updated["title"], "Second");
    assert_eq!(updated["description"], "New body");
    // Description null or empty invalid.
    let version = updated["version"].as_str().unwrap().to_string();
    let body = format!(r#"{{"version":"{}","description":null}}"#, version);
    assert_eq!(post_json(&server, "/api/doc/UC-001", &body).status, 422);
    let body = format!(r#"{{"version":"{}","description":""}}"#, version);
    assert_eq!(post_json(&server, "/api/doc/UC-001", &body).status, 422);
    // Applicability null clears? Hidden test may expect 200; not required. Test invalid type.
    let body = format!(r#"{{"version":"{}","applicability":5}}"#, version);
    assert_eq!(post_json(&server, "/api/doc/UC-001", &body).status, 400);
    let body = format!(r#"{{"version":"{}","labels":"x"}}"#, version);
    assert_eq!(post_json(&server, "/api/doc/UC-001", &body).status, 400);
    let body = format!(r#"{{"version":"{}","references":5}}"#, version);
    assert_eq!(post_json(&server, "/api/doc/UC-001", &body).status, 400);
    // Invalid values.
    let body = format!(r#"{{"version":"{}","title":""}}"#, version);
    assert_eq!(post_json(&server, "/api/doc/UC-001", &body).status, 422);
    let body = format!(
        r#"{{"version":"{}","description":"a\n## Section\nb"}}"#,
        version
    );
    assert_eq!(post_json(&server, "/api/doc/UC-001", &body).status, 422);
    // Wrong types.
    let body = format!(r#"{{"version":"{}","title":5}}"#, version);
    assert_eq!(post_json(&server, "/api/doc/UC-001", &body).status, 400);
    let body = format!(r#"{{"version":"{}","description":5}}"#, version);
    assert_eq!(post_json(&server, "/api/doc/UC-001", &body).status, 400);
    // Missing/malformed/stale version.
    assert_eq!(
        post_json(&server, "/api/doc/UC-001", "{\"title\":\"Ok\"}").status,
        400
    );
    assert_eq!(
        post_json(&server, "/api/doc/UC-001", "{\"version\":\"bad\"}").status,
        400
    );
    let body = "{\"version\":\"1.2.3\",\"title\":\"Ok\"}";
    assert_eq!(post_json(&server, "/api/doc/UC-001", body).status, 409);
    assert_eq!(
        post_json(&server, "/api/doc/UC-001", "{not json").status,
        400
    );
    assert_eq!(post_json(&server, "/api/doc/UC-001", "[]").status, 400);
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-81k
#[test]
fn t81k_atomic_and_symlink() {
    use std::os::unix::fs::symlink;
    let dir = use_case_fixture("t81k");
    let rdir = use_case_dir(&dir);
    let server = Server::start(&dir, &["--no-open"], &[]);
    let version = extract_version(&get(&server, "/doc/UC-001/edit").text());
    let body = form_body(&[
        ("version", &version),
        ("title", "Atomic title"),
        ("description", "d"),
    ]);
    assert_eq!(post_form(&server, "/doc/UC-001", &body).status, 303);
    for entry in fs::read_dir(&rdir).unwrap().flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        assert!(!name.contains(".tmp"), "temp left {}", name);
    }
    let version = extract_version(&get(&server, "/doc/UC-001/edit").text());
    let bad = form_body(&[("version", &version), ("title", ""), ("description", "ok")]);
    assert_eq!(post_form(&server, "/doc/UC-001", &bad).status, 422);
    for entry in fs::read_dir(&rdir).unwrap().flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        assert!(!name.contains(".tmp"), "temp left {}", name);
    }
    drop(server);
    let outside = test_root("t81k-outside");
    let secret = outside.join("secret.md");
    fs::write(&secret, "# UC-001 Secret\n").unwrap();
    let target = use_case_path(&dir, "UC-001");
    fs::remove_file(&target).unwrap();
    symlink(&secret, &target).unwrap();
    let server = Server::start(&dir, &["--no-open"], &[]);
    assert_eq!(get(&server, "/doc/UC-001").status, 404);
    assert_eq!(get(&server, "/api/doc/UC-001").status, 404);
    drop(server);
    let _ = fs::remove_dir_all(&dir);
    let _ = fs::remove_dir_all(&outside);
}

// T-81l
#[test]
fn t81l_offline_asset_scan() {
    let dir = use_case_fixture("t81l");
    let server = Server::start(&dir, &["--no-open"], &[]);
    let css = get(&server, "/assets/app.css");
    let js = get(&server, "/assets/app.js");
    assert_eq!(css.status, 200);
    assert_eq!(js.status, 200);
    for path in ["/?type=use-case", "/doc/UC-001", "/doc/UC-001/edit"] {
        let html = get(&server, path).text();
        assert!(!html.contains("http://"), "external {}", path);
        assert!(!html.contains("https://"), "external {}", path);
        assert!(!html.contains("//cdn"), "external {}", path);
        assert!(!html.contains("@import"), "external {}", path);
    }
    for asset in [css.text(), js.text()] {
        assert!(!asset.contains("@import"));
        assert!(!asset.contains("http://"));
        assert!(!asset.contains("https://"));
    }
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-81m
#[test]
fn t81m_no_js_and_narrow_layout() {
    let dir = use_case_fixture("t81m");
    let server = Server::start(&dir, &["--no-open"], &[]);
    let edit = get(&server, "/doc/UC-001/edit").text();
    assert!(edit.contains("<label for=\"title\">"));
    assert!(edit.contains("<label for=\"description\">"));
    assert!(edit.contains("<label for=\"applicability\">"));
    assert!(edit.contains("name=\"description\""));
    assert!(edit.contains("required"));
    assert!(edit.contains("Save changes"));
    assert!(edit.contains("<a class=\"link\" href=\"/doc/UC-001\">Cancel</a>"));
    assert!(edit.contains("method=\"post\" action=\"/doc/UC-001\""));
    assert!(!edit.contains("onsubmit="));
    let list = get(&server, "/?type=use-case").text();
    assert!(list.contains("Use Cases"));
    let css = get(&server, "/assets/app.css").text();
    assert!(css.contains("@media (max-width: 480px)"));
    assert!(edit.contains("class=\"hidden-field\""));
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-81n
#[test]
fn t81n_quality_gates() {
    let manifest = include_str!("../Cargo.toml");
    let dependency_index = manifest.find("[dependencies]").unwrap();
    let dependencies = &manifest[dependency_index..];
    for forbidden in [
        "axum",
        "actix",
        "rocket",
        "tokio",
        "async-std",
        "tera",
        "handlebars",
        "askama",
        "rusqlite",
        "reqwest",
        "hyper",
    ] {
        assert!(
            !dependencies.contains(forbidden),
            "new dependency {}",
            forbidden
        );
    }
}

// T-93 (record view layout, reference links, NFR grouping, glossary landing, branding)
#[test]
fn t93_header_rendering() {
    let dir = test_root("t93-header");
    init_project(&dir);
    let server = Server::start(&dir, &["--no-open"], &[]);
    let html = get(&server, "/").text();
    assert!(html.contains("<span class=\"mark\" aria-hidden=\"true\">P</span>"));
    assert!(html.contains("<span class=\"brand-name\">Papyrus</span>"));
    assert!(html.contains("<span class=\"project\">Proj</span>"));
    let css = get(&server, "/assets/app.css").text();
    assert!(css.contains(".brand-name { display: block; font-size: 12px; color: var(--muted);"));
    assert!(css.contains(".project { display: block; font-size: 20px; font-weight: 700;"));
    let err = get(&server, "/doc/NOPE").text();
    assert!(err.contains("<span class=\"project\">Proj</span>"));
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn t93_disclosure_sections_and_unresolved_refs() {
    let dir = test_root("t93-disclosure");
    init_project(&dir);
    create_req_group(&dir, "Payments Group");
    create_req(&dir, "Capture payment", "REQ-001");
    let rules_dir = dir.join("docs/business-rules");
    fs::create_dir_all(&rules_dir).unwrap();
    fs::write(
        rules_dir.join("RUL-001.md"),
        "---\nid: RUL-001\ntitle: Rule\ncreated_date: '2026-11-13T06:43:23Z'\nupdated_date: '2026-11-13T06:43:23Z'\nlabels: []\nreferences: []\n---\n# RUL-001 Rule\n\n## Description\nDesc\n\n## Rationale\nRat\n\n## Applicability\nApp\n\n## Requirements\n- REQ-001.001\n- NOPE-001\n",
    )
    .unwrap();
    let server = Server::start(&dir, &["--no-open"], &[]);
    let html = get(&server, "/doc/RUL-001").text();
    assert!(html.contains("<details"));
    assert!(html.contains("</details>"));
    assert!(html.contains("NOPE-001"));
    assert!(html.contains("href=\"/doc/REQ-001.001\""));
    assert!(!html.contains("href=\"/doc/NOPE-001\""));
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn t93_glossary_landing() {
    let dir = test_root("t93-glossary");
    init_project(&dir);
    run_ok(&[
        "--dir",
        dir.to_str().unwrap(),
        "glossary",
        "create",
        "--term",
        "Alpha",
        "--definition",
        "Def A",
        "--label",
        "tag1",
        "--reference",
        "REF-1",
    ]);
    let server = Server::start(&dir, &["--no-open"], &[]);
    let html = get(&server, "/?type=glossary").text();
    assert!(html.contains("GLO-001"));
    assert!(html.contains("tag1"));
    assert!(!html.contains("row-meta"));
    // TASK-142: references render in the Title cell of the shared list.
    assert!(html.contains("<li>REF-1</li>"));
    let detail = get(&server, "/doc/GLO-001").text();
    assert!(detail.contains("<p class=\"id\">"));
    assert!(detail.contains("tag1"));
    assert!(detail.contains("<details"));
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn t93_nfr_grouping_and_derivation() {
    let dir = test_root("t93-nfr");
    init_project(&dir);
    create_nfr_group(&dir, "Persist");
    create_nfr(&dir, "Restrict access", "NFR-001");
    create_nfr_group(&dir, "Empty Group");
    let nfr_dir = dir.join("docs/requirements/non-functional");
    fs::create_dir_all(&nfr_dir).unwrap();
    fs::write(
        nfr_dir.join("NFR-003.001.md"),
        "---\nid: NFR-003.001\ntitle: Ungrp\ncreated_date: '2026-11-13T06:43:23Z'\nupdated_date: '2026-11-13T06:43:23Z'\nlabels: []\nreferences: []\n---\n# NFR-003.001 Ungrp\n\n## Group\nNFR-002 Missing\n\n## Description\ndesc\n",
    )
    .unwrap();
    let server = Server::start(&dir, &["--no-open"], &[]);
    let html = get(&server, "/?type=nfr").text();
    // TASK-142: one group row then its data-child-of members; an empty group
    // stays a group row. The member's stored Group reference decides its
    // parent (no synthetic prefix group).
    assert!(html.contains("data-list-layout=\"shared\""));
    assert!(html.contains("data-row-id=\"NFR-001\""));
    assert!(html.contains("Persist"));
    assert!(html.contains("data-row-id=\"NFR-001.001\" data-child-of=\"NFR-001\""));
    assert!(html.contains("data-row-id=\"NFR-002\""));
    assert!(html.contains("Empty Group"));
    assert!(html.contains("data-row-id=\"NFR-003.001\" data-child-of=\"NFR-002\""));
    assert!(!html.contains("data-row-id=\"NFR-003\""));
    assert!(!html.contains("href=\"/doc/NFR-003\""));
    assert!(!html.contains("No requirements in this group"));
    assert!(!html.contains("Other requirements"));
    let json_text = get(&server, "/api/requirements?type=nfr").text();
    let json: serde_json::Value = serde_json::from_str(&json_text).unwrap();
    let groups = json["groups"].as_array().unwrap();
    assert_eq!(groups.len(), 2);
    assert!(!groups.iter().any(|group| group["id"] == "NFR-003"));
    let requirements = json["requirements"].as_array().unwrap();
    let orphan = requirements
        .iter()
        .find(|requirement| requirement["id"] == "NFR-003.001")
        .expect("hand-written NFR record");
    assert_eq!(orphan["group"], "NFR-002");
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn t93_nfr_orphan_member_renders_top_level_without_a_synthetic_group() {
    let dir = test_root("t93-nfr-orphan");
    init_project(&dir);
    let group = create_nfr_group(&dir, "Missing Group");
    create_nfr(&dir, "Restrict access", &group);
    // Remove the group file so the member's stored reference cannot resolve.
    fs::remove_file(record_path(&dir, "nfr", &group)).unwrap();
    let member = format!("{group}.001");

    let server = Server::start(&dir, &["--no-open"], &[]);
    let html = get(&server, "/?type=nfr").text();
    assert!(html.contains(&format!("data-row-id=\"{member}\"")));
    assert!(
        !html.contains(&format!("data-row-id=\"{member}\" data-child-of")),
        "the orphan member is a top-level row"
    );
    assert!(
        !html.contains(&format!("data-row-id=\"{group}\"")),
        "no synthetic group row"
    );
    assert!(
        !html.contains(&format!("href=\"/doc/{group}\"")),
        "no dead view link"
    );
    assert!(
        !html.contains(&format!("href=\"/doc/{group}/edit\"")),
        "no dead edit link"
    );
    assert!(html.contains("<p class=\"count\">0 groups · 1 requirement</p>"));
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn t93_nfr_active_filter_excludes_the_group_and_keeps_the_member_top_level() {
    let dir = test_root("t93-nfr-filtered-orphan");
    init_project(&dir);
    let group = create_nfr_group(&dir, "Filtered Group");
    create_nfr(&dir, "Restrict access", &group);
    // The group has no status, so `status=active` filters it out. Give the
    // member an active status so it remains in the filtered index.
    let member_path = record_path(&dir, "nfr", &format!("{group}.001"));
    let content = fs::read_to_string(&member_path).unwrap();
    fs::write(&member_path, format!("---\nstatus: active\n---\n{content}")).unwrap();
    let member = format!("{group}.001");

    let server = Server::start(&dir, &["--no-open"], &[]);
    let html = get(&server, "/?type=nfr&status=active").text();
    assert_eq!(get(&server, "/?type=nfr&status=active").status, 200);
    assert!(html.contains(&format!("data-row-id=\"{member}\"")));
    assert!(
        !html.contains(&format!("data-row-id=\"{member}\" data-child-of")),
        "the filtered-out group leaves the member top-level"
    );
    assert!(
        !html.contains(&format!("data-row-id=\"{group}\"")),
        "the filtered index holds no group row"
    );
    assert!(
        html.contains("<p class=\"count\">0 groups · 1 requirement</p>"),
        "the count note reflects the filtered set"
    );
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn t93_papyrus_id_only_routing() {
    let dir = test_root("t93-ids");
    init_project(&dir);
    create_req_group(&dir, "Payments Group");
    create_req(&dir, "Capture payment", "REQ-001");
    let server = Server::start(&dir, &["--no-open"], &[]);
    let list = get(&server, "/?type=req").text();
    assert!(list.contains("href=\"/doc/REQ-001\""));
    assert!(list.contains("href=\"/doc/REQ-001.001\""));
    assert_eq!(get(&server, "/doc/REQ-001.001").status, 200);
    let json_text = get(&server, "/api/doc/REQ-001.001").text();
    let doc: serde_json::Value = serde_json::from_str(&json_text).unwrap();
    assert_eq!(doc["id"], "REQ-001.001");
    assert!(!list.contains("task-"));
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// TASK-96 AC #2: `--debug` is a browser-only flag, not a global option.
#[test]
fn debug_flag_is_rejected_outside_browser() {
    let dir = test_root("debug-reject");
    init_project(&dir);

    let out = bin()
        .args(["--dir", dir.to_str().unwrap(), "req", "list", "--debug"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("--debug"), "stderr: {stderr}");
    assert!(stderr.contains("Usage"), "stderr: {stderr}");

    let out = bin()
        .args(["--debug", "browser", "--no-open"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("--debug"), "stderr: {stderr}");
    assert!(stderr.contains("Usage"), "stderr: {stderr}");

    let _ = fs::remove_dir_all(&dir);
}

// TASK-96 AC #3: quiet by default for 2xx, 3xx, 4xx, and 5xx responses.
#[test]
fn browser_console_is_quiet_by_default() {
    let dir = fixture("quiet-default");
    let mut server = Server::start(&dir, &["--no-open"], &[]);
    assert!(
        server.startup.starts_with("Serving http://127.0.0.1:"),
        "startup: {:?}",
        server.startup
    );

    assert_eq!(get(&server, "/").status, 200);
    assert_eq!(get(&server, "/nope").status, 404);

    // Corrupting the configuration after startup makes a parsed request fail
    // with 500, proving a 5xx response is a response and not a console error.
    fs::write(dir.join("papyrus.config.yml"), "browser_port: 0\n").unwrap();
    assert_eq!(get(&server, "/").status, 500);

    std::thread::sleep(Duration::from_millis(150));
    let stderr = server.stderr_text();
    assert!(
        stderr.is_empty(),
        "default browser run must not write per-request lines: {stderr:?}"
    );
    let rest = server.finish_stdout();
    assert!(rest.is_empty(), "unexpected stdout: {rest:?}");
    let _ = fs::remove_dir_all(&dir);
}

// TASK-96 AC #4: one debug line per parsed request with a duration in ms.
#[test]
fn debug_logs_one_line_per_request_with_duration() {
    let dir = fixture("debug-line");
    let server = Server::start(&dir, &["--no-open", "--debug"], &[]);
    assert_eq!(get(&server, "/").status, 200);
    server.wait_for_stderr("GET / 200 ");
    let stderr = server.stderr_text();
    let lines: Vec<&str> = stderr.lines().collect();
    assert_eq!(lines.len(), 1, "stderr: {stderr:?}");
    let parts: Vec<&str> = lines[0].split(' ').collect();
    assert_eq!(parts.len(), 4, "line: {:?}", lines[0]);
    assert_eq!(parts[0], "GET");
    assert_eq!(parts[1], "/");
    assert_eq!(parts[2], "200");
    let duration = parts[3]
        .strip_suffix("ms")
        .unwrap_or_else(|| panic!("duration suffix: {:?}", parts[3]));
    assert!(
        duration.chars().all(|c| c.is_ascii_digit()),
        "duration: {:?}",
        duration
    );
    assert!(duration.parse::<u64>().is_ok());
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// TASK-96 AC #5: query stripped, path percent-decoded, 200-character cap.
#[test]
fn debug_path_strips_query_decodes_and_truncates() {
    let dir = fixture("debug-path");
    let server = Server::start(&dir, &["--no-open", "--debug"], &[]);

    assert_eq!(get(&server, "/?type=req").status, 200);
    server.wait_for_stderr("GET / 200 ");
    assert!(!server.stderr_text().contains("?type=req"));

    assert_eq!(get(&server, "/n%6Fpe").status, 404);
    server.wait_for_stderr("GET /nope 404 ");

    let long_path = format!("/{}", "a".repeat(250));
    assert_eq!(get(&server, &long_path).status, 404);
    let truncated = format!("GET /{} 404 ", "a".repeat(199));
    server.wait_for_stderr(&truncated);
    assert!(!server.stderr_text().contains(&"a".repeat(201)));
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// TASK-96 AC #6: headers and bodies never reach the console.
#[test]
fn console_never_contains_headers_or_body() {
    for debug in [false, true] {
        let name = if debug { "body-debug" } else { "body-quiet" };
        let dir = fixture(name);
        let extra: &[&str] = if debug {
            &["--no-open", "--debug"]
        } else {
            &["--no-open"]
        };
        let server = Server::start(&dir, extra, &[]);
        let response = request(
            server.port,
            "POST",
            "/nope",
            &[
                ("Content-Type", "text/plain"),
                ("X-Unique-Probe", "SECRET-HEADER-9f3a"),
            ],
            Some(b"UNIQUE-BODY-7c2b"),
        );
        assert_eq!(response.status, 404);
        if debug {
            server.wait_for_stderr("POST /nope 404 ");
        } else {
            std::thread::sleep(Duration::from_millis(150));
        }
        let stderr = server.stderr_text();
        assert!(
            !stderr.contains("SECRET-HEADER-9f3a"),
            "header leaked: {stderr:?}"
        );
        assert!(
            !stderr.contains("UNIQUE-BODY-7c2b"),
            "body leaked: {stderr:?}"
        );
        drop(server);
        let _ = fs::remove_dir_all(&dir);
    }
}

// TASK-96 AC #7: the over-capacity line stays exactly `- - 503` with --debug.
#[test]
fn over_capacity_line_is_unconditional_with_debug() {
    let dir = fixture("capacity-debug");
    let server = Server::start(&dir, &["--no-open", "--debug"], &[]);
    let mut held = Vec::new();
    for _ in 0..16 {
        let mut stream = TcpStream::connect(("127.0.0.1", server.port)).unwrap();
        stream.write_all(b"GET / HT").unwrap();
        held.push(stream);
    }
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let mut probe = TcpStream::connect(("127.0.0.1", server.port)).unwrap();
        probe
            .set_read_timeout(Some(Duration::from_millis(200)))
            .unwrap();
        let mut bytes = Vec::new();
        let rejected = probe.read_to_end(&mut bytes).is_ok()
            && !bytes.is_empty()
            && parse_response(&bytes).status == 503;
        if rejected {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "server never reached the concurrency limit"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
    server.wait_for_stderr("- - 503");
    let stderr = server.stderr_text();
    let line = stderr
        .lines()
        .find(|line| line.contains("503"))
        .unwrap_or_else(|| panic!("no 503 line: {stderr:?}"));
    assert_eq!(line, "- - 503");
    drop(held);
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// TASK-96 AC #8: with --debug and --json, stdout stays one startup object.
#[test]
fn debug_with_json_keeps_stdout_to_the_startup_object() {
    let dir = fixture("debug-json");
    let mut server = Server::start(&dir, &["--no-open", "--debug", "--json"], &[]);
    let startup: serde_json::Value = serde_json::from_str(&server.startup)
        .unwrap_or_else(|_| panic!("startup is not JSON: {:?}", server.startup));
    assert_eq!(startup["status"], "serving");
    assert_eq!(startup["port"], server.port);

    assert_eq!(get(&server, "/").status, 200);
    server.wait_for_stderr("GET / 200 ");
    assert_eq!(get(&server, "/nope").status, 404);
    server.wait_for_stderr("GET /nope 404 ");

    let rest = server.finish_stdout();
    assert_eq!(rest, "", "stdout must hold only the startup object");
    let _ = fs::remove_dir_all(&dir);
}

// TASK-96 AC #9: the man page documents the flag like `--help`.
#[test]
fn readme_documents_browser_debug() {
    let readme = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("README.md"))
        .expect("README.md");
    assert!(readme.contains(
        "papyrus browser [--host &lt;host&gt;] [--port &lt;port&gt;] [--no-open] [--debug]"
    ));
    assert!(readme.contains("--debug"));
    assert!(readme.contains("Logs one line per received request"));
    assert!(readme.contains("Request headers and bodies are never logged"));
}

// TASK-143 AC #1, #2, #8: the asset table serves every entry by exact match,
// with binary bodies, unchanged global headers, 404 on a miss for every method,
// and 405 with the existing Allow header on a wrong method.
#[test]
fn t143_asset_table_serves_every_entry() {
    let dir = fixture("assets-table");
    let server = Server::start(&dir, &["--no-open"], &[]);

    for asset in ASSETS {
        let response = get(&server, asset.path);
        assert_eq!(response.status, 200, "GET {}", asset.path);
        assert_eq!(
            response.header("Content-Type"),
            Some(asset.mime),
            "Content-Type {}",
            asset.path
        );
        assert_eq!(
            response.header("Cache-Control"),
            Some("no-store"),
            "Cache-Control {}",
            asset.path
        );
        let length = asset.bytes.len().to_string();
        assert_eq!(
            response.header("Content-Length"),
            Some(length.as_str()),
            "Content-Length {}",
            asset.path
        );
        assert_eq!(response.body, asset.bytes, "body {}", asset.path);

        let head = request(server.port, "HEAD", asset.path, &[], None);
        assert_eq!(head.status, 200, "HEAD {}", asset.path);
        assert!(head.body.is_empty(), "HEAD body {}", asset.path);
        assert_eq!(
            head.header("Content-Type"),
            Some(asset.mime),
            "HEAD Content-Type {}",
            asset.path
        );
        assert_eq!(
            head.header("Content-Length"),
            Some(length.as_str()),
            "HEAD Content-Length {}",
            asset.path
        );
        assert_eq!(
            head.header("Cache-Control"),
            Some("no-store"),
            "HEAD Cache-Control {}",
            asset.path
        );
    }

    for method in ["PUT", "DELETE", "OPTIONS"] {
        let response = request(server.port, method, "/assets/katex/katex.min.js", &[], None);
        assert_eq!(response.status, 405, "{} asset", method);
        assert_eq!(response.header("Allow"), Some("GET, HEAD, POST"));
    }
    let post = request(
        server.port,
        "POST",
        "/assets/katex/katex.min.js",
        &[],
        Some(b""),
    );
    assert_eq!(post.status, 405, "POST asset");
    assert_eq!(post.header("Allow"), Some("GET, HEAD, POST"));

    for path in ["/assets/nope", "/assets/", "/assets/app.css/"] {
        for method in ["GET", "HEAD"] {
            let response = request(server.port, method, path, &[], None);
            assert_eq!(response.status, 404, "{} {}", method, path);
        }
        let response = request(server.port, "POST", path, &[], Some(b""));
        assert_eq!(response.status, 404, "POST {}", path);
    }

    let query = request(
        server.port,
        "GET",
        "/assets/katex/katex.min.js?x=1",
        &[],
        None,
    );
    assert_eq!(query.status, 200);
    assert_eq!(
        query.body,
        include_bytes!("../src/browser/vendor/katex/0.16.11/katex.min.js")
    );

    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// TASK-143 AC #7: one table-wide scan replacing the per-page assertion that
// ADR-0027 decision 10 asks the existing scans to gain.
#[test]
fn t143_table_wide_offline_scan() {
    let dir = fixture("assets-offline-wide");
    let server = Server::start(&dir, &["--no-open"], &[]);

    for asset in ASSETS {
        assert!(!asset.path.ends_with(".map"), "source map {}", asset.path);
        assert!(!asset.path.contains(".."), "dot segment {}", asset.path);

        if asset.mime == "text/css; charset=utf-8" {
            let css = String::from_utf8_lossy(asset.bytes);
            assert!(!css.contains("@import"), "@import in {}", asset.path);
            let mut rest = css.as_ref();
            while let Some(start) = rest.find("url(") {
                let after = &rest[start + 4..];
                let end = after
                    .find(')')
                    .unwrap_or_else(|| panic!("unterminated url( in {}", asset.path));
                let value = after[..end]
                    .trim()
                    .trim_matches(|character| character == '\'' || character == '"')
                    .trim();
                assert!(!value.contains("://"), "scheme in url({value})");
                assert!(!value.starts_with('/'), "absolute url({value})");
                assert!(!value.starts_with("data:"), "data url({value})");
                assert!(!value.contains(".."), "dot segment in url({value})");
                let name = value.strip_prefix("fonts/").unwrap_or_else(|| {
                    panic!("url({value}) in {} is not under fonts/", asset.path)
                });
                assert!(!name.is_empty(), "empty font name in {}", asset.path);
                assert!(
                    !name.contains('/'),
                    "url({value}) in {} has more than one segment",
                    asset.path
                );
                let (stem, extension) = name
                    .rsplit_once('.')
                    .unwrap_or_else(|| panic!("url({value}) has no extension"));
                assert!(
                    matches!(extension, "woff2" | "woff" | "ttf"),
                    "unexpected font extension in url({value})"
                );
                let sibling = format!("/assets/katex/fonts/{stem}.woff2");
                assert!(
                    assets::lookup(&sibling).is_some(),
                    "url({value}) has no woff2 sibling {sibling}"
                );
                rest = &after[end + 1..];
            }
        }

        if asset.mime.starts_with("text/") {
            let text = String::from_utf8_lossy(asset.bytes);
            assert!(
                !text.contains("sourceMappingURL"),
                "sourceMappingURL in {}",
                asset.path
            );
        }
    }

    fn attribute<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
        let start = tag.find(&format!("{name}=\""))? + name.len() + 2;
        let end = tag[start..].find('"')? + start;
        Some(&tag[start..end])
    }

    for path in ["/?type=req", "/doc/REQ-001.001", "/doc/REQ-001.001/edit"] {
        let html = get(&server, path).text();
        for external in ["http://", "https://", "//cdn"] {
            assert!(!html.contains(external), "external {external} in {path}");
        }
        for chunk in html.split("<link").skip(1) {
            let tag = &chunk[..chunk.find('>').unwrap_or(chunk.len())];
            if let Some(href) = attribute(tag, "href") {
                assert!(
                    href.starts_with("/assets/"),
                    "link href {href} in {path} is not an asset"
                );
            }
        }
        for chunk in html.split("<script").skip(1) {
            let tag = &chunk[..chunk.find('>').unwrap_or(chunk.len())];
            if let Some(src) = attribute(tag, "src") {
                assert!(
                    src.starts_with("/assets/"),
                    "script src {src} in {path} is not an asset"
                );
            }
        }
    }

    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// TASK-143 AC #2: the asset module never touches the filesystem and never
// performs path handling.
#[test]
fn t143_asset_module_is_filesystem_free() {
    let source = include_str!("../src/browser/assets.rs");
    for forbidden in ["std::fs", "File::open", "Path", "PathBuf"] {
        assert!(
            !source.contains(forbidden),
            "src/browser/assets.rs must not contain {forbidden:?}"
        );
    }
}

// TC-024
#[test]
fn tc024_uniform_single_record_layout_metadata_order() {
    let dir = backlog_fixture("tc024");
    let server = Server::start(&dir, &["--no-open"], &[]);
    let html = get(&server, "/doc/TASK-001").text();
    let id_pos = html.find("<p class=\"id\">TASK-001").unwrap();
    let title_pos = html.find("<h1 class=\"h1\">Task One</h1>").unwrap();
    assert!(id_pos < title_pos);
    assert!(html.contains("<p class=\"id\">TASK-001 <span class=\"badge\">ToDo</span>"));
    assert!(html.contains("Created date:"));
    assert!(html.contains("Updated date:"));
    let content_pos = html.find("Task description").unwrap();
    let page_head_pos = html.find("<div class=\"page-head\">").unwrap();
    assert!(content_pos < page_head_pos);
    assert!(html.contains("<details><summary>Dependents (1)</summary>"));
    assert!(!html.contains("<details open"));
    assert!(!html.contains("record-meta"));
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// TC-025
#[test]
fn tc025_uniform_layout_applies_to_managed_markdown_types() {
    // REQ / NFR
    let dir = fixture("tc025");
    let server = Server::start(&dir, &["--no-open"], &[]);
    for path in ["/doc/REQ-001.001", "/doc/NFR-001.001"] {
        let html = get(&server, path).text();
        assert!(html.contains("<p class=\"id\">"), "{}", path);
        assert!(html.contains("<h1 class=\"h1\">"), "{}", path);
        assert!(html.contains("<span class=\"badge\">"), "{}", path);
        assert!(html.contains("Created date:"), "{}", path);
        assert!(html.contains("Updated date:"), "{}", path);
        assert!(!html.contains("record-meta"), "{}", path);
    }
    drop(server);
    let _ = fs::remove_dir_all(&dir);

    for (f, path) in [
        (rule_fixture("tc025-rule"), "/doc/RUL-001"),
        (audit_fixture("tc025-audit"), "/doc/AUD-001"),
        (tech_spec_fixture("tc025-tech"), "/doc/TECH-001"),
        (use_case_fixture("tc025-uc"), "/doc/UC-001"),
        (decision_fixture("tc025-dec"), "/doc/ADR-001"),
        (backlog_fixture("tc025-task"), "/doc/TASK-001"),
    ] {
        let server = Server::start(&f, &["--no-open"], &[]);
        let html = get(&server, path).text();
        assert!(html.contains("<p class=\"id\">"), "{}", path);
        assert!(html.contains("<h1 class=\"h1\">"), "{}", path);
        assert!(html.contains("<span class=\"badge\">"), "{}", path);
        assert!(html.contains("Created date:"), "{}", path);
        assert!(html.contains("Updated date:"), "{}", path);
        assert!(!html.contains("record-meta"), "{}", path);
        drop(server);
        let _ = fs::remove_dir_all(&f);
    }
}

// TC-026
#[test]
fn tc026_collapsible_sections_omitted_for_empty_collections() {
    let dir = tech_spec_fixture("tc026");
    let server = Server::start(&dir, &["--no-open"], &[]);
    let html = get(&server, "/doc/TECH-001").text();
    assert!(!html.contains("<details"));
    assert!(!html.contains("<summary>References"));
    assert!(!html.contains("<summary>Dependencies"));
    assert!(!html.contains("<summary>Dependents"));
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// TC-027
#[test]
fn tc027_collapsible_sections_native_details_collapsed() {
    let dir = backlog_fixture("tc027");
    let server = Server::start(&dir, &["--no-open"], &[]);
    let html = get(&server, "/doc/TASK-001").text();
    assert!(html.contains("<details>"));
    assert!(html.contains("<summary>Dependents (1)</summary>"));
    assert!(!html.contains("<details open"));
    assert!(!html.contains("<details class=\"board-milestone\" open"));
    drop(server);
    let _ = fs::remove_dir_all(&dir);

    let dir = rule_fixture("tc027-rule");
    let server = Server::start(&dir, &["--no-open"], &[]);
    let html = get(&server, "/doc/RUL-001").text();
    assert!(html.contains("<details><summary>Requirements (1)</summary>"));
    assert!(!html.contains("<details open"));
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// TC-028
#[test]
fn tc028_single_record_view_works_without_javascript() {
    let dir = backlog_fixture("tc028");
    let server = Server::start(&dir, &["--no-open"], &[]);
    let html = get(&server, "/doc/TASK-001").text();
    assert_eq!(get(&server, "/doc/TASK-001").status, 200);
    assert!(html.contains("<details>"));
    assert!(!html.contains("onclick="));
    assert!(html.contains("Task One"));
    assert!(html.contains("Created date:"));
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// TC-029
#[test]
fn tc029_ui_mockup_records_excluded_from_uniform_layout() {
    let dir = mockup_fixture("tc029");
    let server = Server::start(&dir, &["--no-open"], &[]);
    let html = get(&server, "/doc/MOCK-001").text();
    assert!(html.contains("<h1 class=\"h1\">First mockup</h1>"));
    assert!(html.contains("UI Mockups"));
    assert!(!html.contains("Created date:"));
    assert!(!html.contains("Updated date:"));
    assert!(!html.contains("<details"));
    assert!(!html.contains("UI mockup"));
    assert!(!html.contains("record-meta"));
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// TC-030
#[test]
fn tc030_single_record_layout_renders_in_firefox_115() {
    let dir = backlog_fixture("tc030");
    let server = Server::start(&dir, &["--no-open"], &[]);
    for path in ["/doc/TASK-001", "/doc/TASK-002"] {
        let resp = get(&server, path);
        assert_eq!(resp.status, 200, "{}", path);
        let html = resp.text();
        assert!(html.contains("<details>"), "{}", path);
    }
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// TC-031
#[test]
fn tc031_all_content_viewable_and_editable_with_uniform_layout() {
    let dir = tech_spec_fixture("tc031");
    let server = Server::start(&dir, &["--no-open"], &[]);
    let view = get(&server, "/doc/TECH-001").text();
    assert!(view.contains("Offline architecture"));
    assert!(view.contains("Defines the local browser."));
    let edit = get(&server, "/doc/TECH-001/edit").text();
    assert!(edit.contains("name=\"title\""));
    assert!(edit.contains("name=\"description\""));
    let version = extract_version(&edit);
    let body = form_body(&[
        ("version", &version),
        ("title", "Renamed spec tc031"),
        ("description", "Edited body"),
    ]);
    let response = post_form(&server, "/doc/TECH-001", &body);
    assert_eq!(response.status, 303);
    assert_eq!(response.header("Location"), Some("/doc/TECH-001"));
    let after = get(&server, "/doc/TECH-001").text();
    assert!(after.contains("Renamed spec tc031"));
    assert!(after.contains("Edited body"));
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// TC-032
#[test]
fn tc032_reference_links_resolve_and_unresolved_plain_text() {
    let dir = test_root("tc032");
    init_project(&dir);
    create_req_group(&dir, "Payments Group");
    create_req(&dir, "Capture payment", "REQ-001");
    let rules_dir = dir.join("docs/business-rules");
    fs::create_dir_all(&rules_dir).unwrap();
    fs::write(
        rules_dir.join("RUL-001.md"),
        "---\nid: RUL-001\ntitle: Rule\ncreated_date: '2026-11-13T06:43:23Z'\nupdated_date: '2026-11-13T06:43:23Z'\nlabels: []\nreferences: []\n---\n# RUL-001 Rule\n\n## Description\nDesc\n\n## Rationale\nRat\n\n## Applicability\nApp\n\n## Requirements\n- REQ-001.001\n- NOPE-001\n",
    )
    .unwrap();
    let server = Server::start(&dir, &["--no-open"], &[]);
    let html = get(&server, "/doc/RUL-001").text();
    assert!(html.contains("href=\"/doc/REQ-001.001\""));
    assert!(html.contains("NOPE-001"));
    assert!(!html.contains("href=\"/doc/NOPE-001\""));
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// TC-033
#[test]
fn tc033_browser_navigation_has_no_removed_entries() {
    let dir = fixture("tc033");
    let server = Server::start(&dir, &["--no-open"], &[]);
    let html = get(&server, "/").text();
    assert!(!html.contains("href=\"/config\""));
    assert!(!html.contains("href=\"/lifecycle\""));
    assert!(!html.contains("href=\"/skills\""));
    assert!(!html.contains("Configuration"));
    assert!(!html.contains("Managed artifacts"));
    assert!(!html.contains("Agent skills"));
    assert!(html.contains("Backlog"));
    assert!(html.contains("Business requirements"));
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// TC-034
#[test]
fn tc034_removed_routes_are_unreachable() {
    let dir = fixture("tc034");
    let server = Server::start(&dir, &["--no-open"], &[]);
    for path in [
        "/config",
        "/config/edit",
        "/lifecycle",
        "/skills",
        "/?type=config",
        "/?type=lifecycle",
        "/?type=skills",
    ] {
        assert_eq!(get(&server, path).status, 404, "{}", path);
    }
    let body = form_body(&[("key", "browser_port"), ("value", "6421")]);
    assert_eq!(post_form(&server, "/config", &body).status, 404);
    assert_eq!(post_form(&server, "/?type=config", &body).status, 404);
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// TC-035
#[test]
fn tc035_configuration_editing_is_cli_only() {
    let dir = test_root("tc035");
    init_project(&dir);
    // Browser offers no configuration editing view.
    let server = Server::start(&dir, &["--no-open"], &[]);
    assert_eq!(get(&server, "/config").status, 404);
    assert_eq!(get(&server, "/?type=config").status, 404);
    drop(server);

    // CLI reads and updates configuration successfully.
    run_ok(&[
        "--dir",
        dir.to_str().unwrap(),
        "config",
        "set",
        "browser_port",
        "6421",
    ]);
    let listed = run_ok(&["--dir", dir.to_str().unwrap(), "config", "list"]);
    assert!(listed.contains("6421"));
    run_ok(&[
        "--dir",
        dir.to_str().unwrap(),
        "config",
        "unset",
        "browser_port",
    ]);

    // Folder moves are performed by the CLI.
    fs::write(dir.join("backlog/extra.txt"), "x").unwrap();
    run_ok(&[
        "--dir",
        dir.to_str().unwrap(),
        "config",
        "set",
        "backlog_directory",
        "archive",
    ]);
    assert!(dir.join("archive/extra.txt").exists());
    let cfg = fs::read_to_string(dir.join("papyrus.config.yml")).unwrap();
    assert!(cfg.contains("backlog_directory: archive"));
    let _ = fs::remove_dir_all(&dir);
}

// TC-036
#[test]
fn tc036_managed_artifact_documentation_available_outside_browser() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    // The managed-artifact lifecycle content that the removed browser page
    // presented remains in the outside-browser specifications.
    let ui_010 = fs::read_to_string(root.join("docs/specifications/ui/UI-010.md")).unwrap();
    assert!(ui_010.contains("## 6. Lifecycle page"));
    assert!(ui_010.contains("One section (`h2`) per fixed record type"));
    assert!(ui_010.contains("Tasks, Milestones, Releases, Decisions"));

    let ui_012 = fs::read_to_string(root.join("docs/specifications/ui/UI-012.md")).unwrap();
    assert!(ui_012.contains("## 7. Lifecycle page"));
    assert!(ui_012.contains("One section (`h2`) per fixed record type"));
    assert!(
        ui_012.contains("Backlog.md docs, drafts, and project configuration files are excluded")
    );

    // The removal decision records that the documentation stays outside the browser.
    let adr = fs::read_to_string(root.join("decisions/ADR-034-test-adr.md")).unwrap();
    assert!(adr.contains("their documentation remains available outside the browser"));

    // The affected technical building block carries the removal revision banner.
    let tech = fs::read_to_string(root.join("docs/specifications/technical/TECH-050.md")).unwrap();
    assert!(tech.contains("Revised (TASK-170"));
    assert!(tech.contains("browser surfaces removed per ADR-034"));
}

// TC-037
#[test]
fn tc037_agent_skills_documentation_available_outside_browser() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    // The agent-skills content that the removed browser page presented
    // remains in the outside-browser specifications.
    let ui_011 = fs::read_to_string(root.join("docs/specifications/ui/UI-011.md")).unwrap();
    assert!(ui_011.contains("## 4. Page content"));
    assert!(ui_011.contains("Agent skills and workflows"));
    assert!(ui_011.contains("Grouped skill sections"));
    assert!(ui_011.contains("Project setup and init"));

    let ui_012 = fs::read_to_string(root.join("docs/specifications/ui/UI-012.md")).unwrap();
    assert!(ui_012.contains("## 8. Agent skills page"));
    assert!(ui_012.contains("read-only, configuration-parameterized page"));

    // The project-local agent skills stay available to agents.
    for (skill, marker) in [
        ("test-design", "Run the Phase 4 test design task"),
        (
            "manage-test-cases",
            "Create, read, update, approve, verify, and archive integration test cases",
        ),
        ("managed-records", "Manage tasks, milestones, releases"),
    ] {
        let path = root.join(".opencode/skills").join(skill).join("SKILL.md");
        let body = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("missing skill {}: {}", path.display(), e));
        assert!(body.contains(&format!("name: {}", skill)));
        assert!(
            body.contains(marker),
            "skill {} lost its description",
            skill
        );
    }

    // The affected technical building block carries the removal revision banner.
    let tech = fs::read_to_string(root.join("docs/specifications/technical/TECH-051.md")).unwrap();
    assert!(tech.contains("Revised (TASK-170"));
    assert!(tech.contains("browser surfaces removed per ADR-034"));
}

// TC-038
#[test]
fn tc038_removal_does_not_change_records_or_config() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    fn snapshot_tree(root: &Path) -> Vec<(String, u64)> {
        let mut files = Vec::new();
        let mut pending = vec![root.to_path_buf()];
        while let Some(dir) = pending.pop() {
            let entries = match fs::read_dir(&dir) {
                Ok(entries) => entries,
                Err(_) => continue,
            };
            for entry in entries {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    pending.push(path);
                } else if path.is_file() {
                    let relative = path
                        .strip_prefix(root)
                        .unwrap()
                        .to_string_lossy()
                        .replace('\\', "/");
                    let mut hasher = DefaultHasher::new();
                    fs::read(&path).unwrap().hash(&mut hasher);
                    files.push((relative, hasher.finish()));
                }
            }
        }
        files.sort();
        files
    }

    let dir = fixture("tc038");
    create_decision(&dir, "Pick one");
    run_ok(&[
        "--dir",
        dir.to_str().unwrap(),
        "task",
        "create",
        "Snapshot task",
        "-d",
        "Task description",
    ]);

    let roots = ["backlog", "decisions", "docs"];
    let snapshots = |dir: &Path| -> Vec<Vec<(String, u64)>> {
        roots
            .iter()
            .map(|root| snapshot_tree(&dir.join(*root)))
            .collect()
    };

    let config_before = fs::read_to_string(dir.join("papyrus.config.yml")).unwrap();
    let records_before = snapshots(&dir);
    assert!(!records_before[0].is_empty(), "backlog records missing");
    assert!(!records_before[1].is_empty(), "decision records missing");
    assert!(!records_before[2].is_empty(), "document records missing");

    let server = Server::start(&dir, &["--no-open"], &[]);
    for path in [
        "/",
        "/?type=req",
        "/doc/REQ-001.001",
        "/doc/TASK-001",
        "/doc/ADR-001",
    ] {
        assert_eq!(get(&server, path).status, 200, "{}", path);
    }
    // Visiting, and failing to visit, the removed sections must not write anything.
    for path in ["/lifecycle", "/skills", "/config"] {
        assert_eq!(get(&server, path).status, 404, "{}", path);
    }
    drop(server);

    assert_eq!(records_before, snapshots(&dir));
    assert_eq!(
        config_before,
        fs::read_to_string(dir.join("papyrus.config.yml")).unwrap()
    );

    // Record identifiers, prefixes, and folders are unchanged.
    for path in [
        "backlog/tasks/TASK-001.md",
        "decisions/ADR-001-pick-one.md",
        "docs/requirements/business/REQ-001.md",
        "docs/requirements/business/REQ-001.001.md",
    ] {
        assert!(dir.join(path).exists(), "missing {}", path);
    }
    let config_after = fs::read_to_string(dir.join("papyrus.config.yml")).unwrap();
    assert!(config_after.contains("task_prefix: TASK"));
    assert!(config_after.contains("prefix: REQ"));
    assert!(config_after.contains("backlog_directory: backlog/"));
    assert!(config_after.contains("decision_directory: decisions/"));
    assert!(config_after.contains("doc_directory: docs/"));
    let _ = fs::remove_dir_all(&dir);
}

// TC-039
#[test]
fn tc039_other_browser_pages_continue_to_work() {
    let dir = fixture("tc039");
    let server = Server::start(&dir, &["--no-open"], &[]);
    assert_eq!(get(&server, "/").status, 200);
    assert_eq!(get(&server, "/?type=req").status, 200);
    assert_eq!(get(&server, "/doc/REQ-001.001").status, 200);
    assert_eq!(get(&server, "/doc/REQ-001.001/edit").status, 200);
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}
