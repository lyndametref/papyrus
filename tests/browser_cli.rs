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
    assert!(text.contains("papyrus browser [--host <host>] [--port <port>] [--no-open]"));

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

// T-09 (empty project and empty document type)
#[test]
fn t09_list_empty_project_and_empty_type() {
    // Empty project: both categories render the project-level empty state.
    let empty = test_root("empty-project");
    init_project(&empty);
    let server = Server::start(&empty, &["--no-open"], &[]);
    for path in ["/", "/?type=nfr"] {
        let html = get(&server, path).text();
        assert!(
            html.contains(
                "No requirement records were found in the configured document directories."
            ),
            "path={}",
            path
        );
        assert!(html.contains("Records are created from the command line"));
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
    assert!(html.contains("No non-functional requirements in this project."));
    assert!(html.contains("Business requirements are available in Document types."));
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
    let html = get(&server, "/").text();
    assert!(html.contains("No requirements in this group"));
    assert!(html.contains("No requirements"));
    assert!(html.contains("Other requirements"));
    assert!(html.contains(&format!("Group: {} (not found)", orphan_group)));
    assert!(html.contains(&format!("/doc/{}.001", orphan_group)));
    assert!(!html.contains(&format!("href=\"/doc/{}\"", orphan_group)));

    let group = get(&server, &format!("/doc/{}", empty_group)).text();
    assert!(group.contains("Requirements in this group"));
    assert!(group.contains("No requirements in this group."));
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
    let cli = run_ok(&[
        "--dir",
        dir.to_str().unwrap(),
        "nfreq",
        "view",
        "NFR-001.001",
    ]);
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

    let description_message =
        "Description cannot contain a line starting with ## . Remove that line or change it.";
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

    let story_message =
        "User story cannot contain a line starting with ## . Remove that line or change it.";
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
    let html = response.text();
    assert!(html.contains("This record changed on disk"));
    assert!(html.contains("record changed on disk; reload the form"));
    assert!(html.contains("Reload the form"));
    assert!(html.contains("View current record"));
    assert!(html.contains("Back to requirements"));
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

    let dir = fixture("mock-layout");
    let server = Server::start(&dir, &["--no-open"], &[]);
    let html = get(&server, "/").text();
    assert!(html.contains("footer-long"));
    assert!(html.contains("footer-short"));
    assert!(html.contains("Papyrus local browser &mdash; served from this project only."));
    drop(server);
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
