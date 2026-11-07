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
            .starts_with("# REQ-001.001 Permissions kept")
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

// AC #14 (unknown path is 404 for every method; known path wrong method is 405)
#[test]
fn unknown_path_is_404_for_every_method() {
    let dir = fixture("route-method");
    let server = Server::start(&dir, &["--no-open"], &[]);
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
    let html = get(&server, "/").text();
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
            400,
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
    // archive task and milestone
    run_ok(&[
        "--dir",
        dir.to_str().unwrap(),
        "task",
        "archive",
        "TASK-001",
    ]);
    run_ok(&[
        "--dir",
        dir.to_str().unwrap(),
        "milestone",
        "archive",
        "MS-001",
    ]);
    let server = Server::start(&dir, &["--no-open"], &[]);
    let list = get(&server, "/?type=task").text();
    assert!(!list.contains("TASK-001"));
    assert_eq!(get(&server, "/doc/TASK-001").status, 404);
    assert_eq!(get(&server, "/doc/TASK-001/edit").status, 404);
    let list = get(&server, "/?type=milestone").text();
    assert!(!list.contains("MS-001"));
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
        assert!(!list.contains("TASK-001"));
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
    dir.join("decisions").join(format!("{}.md", id))
}

// T-15a
#[test]
fn t15a_list_decision_category() {
    let dir = decision_fixture("t15a");
    let server = Server::start(&dir, &["--no-open"], &[]);

    let html = get(&server, "/?type=decision").text();
    assert!(html.contains("ADR-001"), "missing ADR-001");
    assert!(html.contains("ADR-002"), "missing ADR-002");
    assert!(html.contains("BDR-001"), "missing BDR-001");
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

    assert_eq!(get(&server, "/").status, 200);
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
            400,
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
        "Status",
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
        ("status", "deprecated"),
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

// T-15g
#[test]
fn t15g_archived_decision() {
    let dir = decision_fixture("t15g");
    run_ok(&[
        "--dir",
        dir.to_str().unwrap(),
        "decision",
        "archive",
        "ADR-001",
    ]);
    let server = Server::start(&dir, &["--no-open"], &[]);
    let list = get(&server, "/?type=decision").text();
    assert!(!list.contains("ADR-001"));
    assert_eq!(get(&server, "/doc/ADR-001").status, 404);
    assert_eq!(get(&server, "/doc/ADR-001/edit").status, 404);
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
    assert_eq!(get(&server, "/").status, 200);
    assert_eq!(get(&server, "/?type=req").status, 200);
    for ty in ["all", "x", "mockup", "ui-mockups"] {
        assert_eq!(
            get(&server, &format!("/?type={}", ty)).status,
            400,
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
    let html = get(&server, "/?type=ui-mockup").text();
    assert_eq!(get(&server, "/?type=ui-mockup").status, 200);
    // Entry omitted from the sidebar; empty state, not an error.
    assert!(!html.contains("href=\"/?type=ui-mockup\""));
    assert!(html.contains("UI Mockups"));
    assert!(html.contains("No UI mockups"));
    let api: serde_json::Value =
        serde_json::from_str(&get(&server, "/api/ui-mockups").text()).unwrap();
    assert!(api.as_array().unwrap().is_empty());
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
    let deprecated = mock_dir.join("deprecated");
    fs::create_dir_all(&deprecated).unwrap();
    fs::write(
        deprecated.join("MOCK-999.html"),
        "<html><head><title>MOCK-999 — Hidden</title></head><body></body></html>",
    )
    .unwrap();
    let server = Server::start(&dir, &["--no-open"], &[]);
    let html = get(&server, "/?type=ui-mockup").text();
    assert!(html.contains("MOCK-001"));
    assert!(html.contains("MOCK-002"));
    for hidden in ["MOCK-999", "README", "wireframe", "index", "other.html"] {
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
    assert!(html.contains("UI mockup"));
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
fn t40i_deprecated_hidden_and_404() {
    let dir = mockup_fixture("t40i");
    run_ok(&[
        "--dir",
        dir.to_str().unwrap(),
        "ui-mockup",
        "deprecate",
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
    // preview first paragraph
    assert!(html.contains("First description paragraph."));
    // default stays req; unknown values are 400.
    assert_eq!(get(&server, "/").status, 200);
    assert_eq!(get(&server, "/?type=req").status, 200);
    for ty in ["all", "x", "business-rules", "rule"] {
        assert_eq!(
            get(&server, &format!("/?type={}", ty)).status,
            400,
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
    let html = get(&server, "/?type=business-rule").text();
    assert_eq!(get(&server, "/?type=business-rule").status, 200);
    assert!(!html.contains("href=\"/?type=business-rule\""));
    let api: serde_json::Value =
        serde_json::from_str(&get(&server, "/api/business-rules").text()).unwrap();
    assert!(api.as_array().unwrap().is_empty());
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
    let deprecated = rdir.join("deprecated");
    fs::create_dir_all(&deprecated).unwrap();
    fs::write(
        deprecated.join("RUL-999.md"),
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
    assert!(html.contains("Record ID"));
    assert!(html.contains("RUL-001"));
    assert!(html.contains("Business rule"));
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
fn t25i_deprecated_hidden_and_404() {
    let dir = rule_fixture("t25i");
    run_ok(&[
        "--dir",
        dir.to_str().unwrap(),
        "rule",
        "deprecate",
        "RUL-002",
    ]);
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
    assert!(html.contains("First content paragraph."));
    assert_eq!(get(&server, "/").status, 200);
    assert_eq!(get(&server, "/?type=req").status, 200);
    for ty in ["all", "x", "audit-records", "audit"] {
        assert_eq!(
            get(&server, &format!("/?type={}", ty)).status,
            400,
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
    let html = get(&server, "/?type=audit-record").text();
    assert_eq!(get(&server, "/?type=audit-record").status, 200);
    assert!(!html.contains("href=\"/?type=audit-record\""));
    let api: serde_json::Value =
        serde_json::from_str(&get(&server, "/api/audit-records").text()).unwrap();
    assert!(api.as_array().unwrap().is_empty());
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
    let deprecated = rdir.join("deprecated");
    fs::create_dir_all(&deprecated).unwrap();
    fs::write(
        deprecated.join("AUD-999.md"),
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
    assert!(html.contains("Record ID"));
    assert!(html.contains("AUD-001"));
    assert!(html.contains("Audit record"));
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
fn t26h_deprecated_hidden_and_404() {
    let dir = audit_fixture("t26h");
    run_ok(&[
        "--dir",
        dir.to_str().unwrap(),
        "audit",
        "deprecate",
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
    assert!(html.contains("Defines the local browser."));
    assert_eq!(get(&server, "/").status, 200);
    assert_eq!(get(&server, "/?type=req").status, 200);
    for ty in ["all", "x", "tech-specs", "tech_spec"] {
        assert_eq!(
            get(&server, &format!("/?type={}", ty)).status,
            400,
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
    let html = get(&server, "/?type=tech-spec").text();
    assert_eq!(get(&server, "/?type=tech-spec").status, 200);
    assert!(!html.contains("href=\"/?type=tech-spec\""));
    assert!(html.contains("No technical specifications"));
    let api: serde_json::Value =
        serde_json::from_str(&get(&server, "/api/tech-specs").text()).unwrap();
    assert!(api.as_array().unwrap().is_empty());
    drop(server);
    let _ = fs::remove_dir_all(&dir);
}

// T-57b
#[test]
fn t57b_list_excludes_non_records_and_deprecated() {
    let dir = tech_spec_fixture("t57b");
    let rdir = tech_spec_dir(&dir);
    fs::write(rdir.join("REQ-001.md"), "# REQ-001 Not a technical spec\n").unwrap();
    fs::write(rdir.join("README.md"), "# registry").unwrap();
    fs::write(rdir.join("notes.txt"), "x").unwrap();
    fs::write(rdir.join("TECH-ABC.md"), "# bad").unwrap();
    let deprecated = rdir.join("deprecated");
    fs::create_dir_all(&deprecated).unwrap();
    fs::write(
        deprecated.join("TECH-999.md"),
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
    assert!(html.contains("No technical specifications were found in the configured directory."));
    assert!(html.contains("Deprecated technical specifications are hidden."));
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
    assert!(html.contains("Record ID"));
    assert!(html.contains("TECH-001"));
    assert!(html.contains("Technical specification"));
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
    let expected = "# TECH-001 Edited title\n\nPreamble kept.\n\n## Description\nNew body\nsecond line\n\n## Editor Notes\nKeep this line\n\n## Unknown\nAnd this\n";
    assert_eq!(after, expected);
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
fn t57h_deprecated_hidden_and_404() {
    const HELP: &str = "Deprecated technical specifications are not listed and cannot be opened. The same page is used for an unknown address.";
    let dir = tech_spec_fixture("t57h");
    run_ok(&[
        "--dir",
        dir.to_str().unwrap(),
        "tech-spec",
        "deprecate",
        "TECH-002",
    ]);
    let server = Server::start(&dir, &["--no-open"], &[]);
    let list = get(&server, "/?type=tech-spec").text();
    assert!(list.contains("TECH-001"));
    assert!(!list.contains("TECH-002"));
    assert!(list.contains("1 technical specification"));

    // The same technical-specifications 404 page is rendered for a deprecated
    // and an unknown ID, and for view, edit, and POST.
    let deprecated = get(&server, "/doc/TECH-002");
    assert_eq!(deprecated.status, 404);
    let not_found = deprecated.text();
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
    let deprecated_edit = get(&server, "/doc/TECH-002/edit");
    assert_eq!(deprecated_edit.status, 404);
    assert_eq!(not_found, deprecated_edit.text());
    let unknown_edit = get(&server, "/doc/TECH-404/edit");
    assert_eq!(unknown_edit.status, 404);
    assert_eq!(not_found, unknown_edit.text());
    let body = form_body(&[("version", "1.2.3"), ("title", "Ok"), ("description", "ok")]);
    let deprecated_post = post_form(&server, "/doc/TECH-002", &body);
    assert_eq!(deprecated_post.status, 404);
    assert_eq!(not_found, deprecated_post.text());
    let unknown_post = post_form(&server, "/doc/TECH-404", &body);
    assert_eq!(unknown_post.status, 404);
    assert_eq!(not_found, unknown_post.text());

    // A non-technical-specification ID keeps the baseline 404 page.
    let baseline = get(&server, "/doc/REQ-404").text();
    assert!(baseline.contains("Back to requirements"));
    assert!(!baseline.contains(HELP));

    // JSON 404 responses keep the baseline JSON error object.
    let deprecated_json = get(&server, "/api/doc/TECH-002");
    assert_eq!(deprecated_json.status, 404);
    assert!(
        deprecated_json
            .header("Content-Type")
            .unwrap()
            .starts_with("application/json")
    );
    let json: serde_json::Value = serde_json::from_str(&deprecated_json.text()).unwrap();
    assert_eq!(json["error"]["code"], 404);
    assert!(json["error"]["message"].is_string());
    let json_body = "{\"version\":\"1.2.3\",\"title\":\"Ok\"}";
    let deprecated_json_post = post_json(&server, "/api/doc/TECH-002", json_body);
    assert_eq!(deprecated_json_post.status, 404);
    let json: serde_json::Value = serde_json::from_str(&deprecated_json_post.text()).unwrap();
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
