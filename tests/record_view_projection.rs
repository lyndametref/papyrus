//! TASK-67: RecordView CLI/browser parity, front-matter round trip on
//! browser save, read-only timestamps, and identity-position tests.

use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::Duration;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_papyrus"))
}

fn test_root(name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(".tmp");
    path.push(format!("recordview-{}-{}", name, std::process::id()));
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

fn arg_dir(dir: &Path) -> String {
    dir.to_str().unwrap().to_string()
}

fn json_ok(args: &[&str]) -> serde_json::Value {
    let out = run_ok(args);
    serde_json::from_str(&out).unwrap()
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
                .arg("browser")
                .arg("--port")
                .arg(port.to_string())
                .arg("--no-open")
                .stdout(Stdio::piped())
                .stderr(Stdio::null());
            let mut child = command.spawn().unwrap();
            let stdout = child.stdout.take().unwrap();
            let mut reader = BufReader::new(stdout);
            let mut startup = String::new();
            let _ = reader.read_line(&mut startup);
            if !startup.trim().is_empty() {
                return Server { child, port };
            }
            let _ = child.wait();
        }
        panic!("server did not start");
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

fn request(
    port: u16,
    method: &str,
    path: &str,
    headers: &[(&str, &str)],
    body: Option<&[u8]>,
) -> HttpResponse {
    let mut raw =
        format!("{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n")
            .into_bytes();
    for (name, value) in headers {
        raw.extend_from_slice(format!("{name}: {value}\r\n").as_bytes());
    }
    if let Some(body) = body {
        raw.extend_from_slice(format!("Content-Length: {}\r\n", body.len()).as_bytes());
    }
    raw.extend_from_slice(b"\r\n");
    if let Some(body) = body {
        raw.extend_from_slice(body);
    }
    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(15)))
        .unwrap();
    stream.write_all(&raw).unwrap();
    let mut bytes = Vec::new();
    stream.read_to_end(&mut bytes).unwrap();
    let split = bytes
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .expect("headers");
    let head = String::from_utf8_lossy(&bytes[..split]).to_string();
    let status: u16 = head
        .split(' ')
        .nth(1)
        .and_then(|v| v.parse().ok())
        .expect("status");
    HttpResponse {
        status,
        body: bytes[split + 4..].to_vec(),
    }
}

fn get(server: &Server, path: &str) -> HttpResponse {
    request(server.port, "GET", path, &[], None)
}

fn urlencode(value: &str) -> String {
    let mut out = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            out.push(byte as char);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

fn form_body(fields: &[(&str, &str)]) -> String {
    fields
        .iter()
        .map(|(k, v)| format!("{}={}", urlencode(k), urlencode(v)))
        .collect::<Vec<_>>()
        .join("&")
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

fn extract_version(html: &str) -> String {
    let marker = "name=\"version\" value=\"";
    let start = html.find(marker).expect("version field") + marker.len();
    let rest = &html[start..];
    rest[..rest.find('"').expect("version terminator")].to_string()
}

fn fm_value(file_text: &str, key: &str) -> String {
    let prefix = format!("{key}:");
    for line in file_text.lines() {
        if let Some(rest) = line.strip_prefix(&prefix) {
            return rest.trim().trim_matches('\'').to_string();
        }
    }
    panic!("front-matter key {key} not found")
}

fn body_of(file_text: &str) -> String {
    let mut dashes = 0;
    for (i, line) in file_text.lines().enumerate() {
        if line.trim() == "---" {
            dashes += 1;
            if dashes == 2 {
                return file_text.lines().skip(i + 1).collect::<Vec<_>>().join("\n");
            }
        }
    }
    file_text.to_string()
}

fn assert_metadata_shape(metadata: &serde_json::Value, display_id: &str) {
    let object = metadata.as_object().expect("metadata is an object");
    for key in [
        "id",
        "title",
        "created_date",
        "updated_date",
        "labels",
        "references",
    ] {
        assert!(
            object.contains_key(key),
            "metadata missing {key}: {metadata}"
        );
    }
    assert_eq!(
        metadata["id"], display_id,
        "metadata.id is the record identity in metadata"
    );
}

#[test]
fn cli_json_metadata_for_every_managed_type() {
    let dir = test_root("all");
    run_ok(&["--dir", &arg_dir(&dir), "init", "Proj", "--no-git"]);
    let d = arg_dir(&dir);

    run_ok(&["--dir", &d, "release", "create", "Rel"]);
    run_ok(&["--dir", &d, "milestone", "create", "Mil"]);
    run_ok(&["--dir", &d, "task", "create", "Task"]);
    run_ok(&["--dir", &d, "decision", "create", "Dec"]);
    run_ok(&[
        "--dir",
        &d,
        "glossary",
        "create",
        "--term",
        "Term",
        "--definition",
        "defs",
    ]);
    run_ok(&["--dir", &d, "req", "create-group", "--title", "G"]);
    run_ok(&[
        "--dir",
        &d,
        "req",
        "create",
        "Req",
        "--group",
        "REQ-001",
        "--story",
        "As a user",
    ]);
    run_ok(&[
        "--dir",
        &d,
        "rule",
        "create",
        "Rule",
        "--description",
        "d",
        "--rationale",
        "r",
        "--applicability",
        "a",
    ]);
    run_ok(&[
        "--dir",
        &d,
        "audit",
        "create",
        "--title",
        "Aud",
        "--content",
        "c",
    ]);
    run_ok(&[
        "--dir",
        &d,
        "tech-spec",
        "create",
        "Tech",
        "--description",
        "d",
    ]);

    let cases: &[(&str, &str)] = &[
        ("release", "REL-001"),
        ("milestone", "MS-001"),
        ("task", "TASK-001"),
        ("decision", "ADR-001"),
        ("glossary", "GLO-001"),
        ("req", "REQ-001.001"),
        ("rule", "RUL-001"),
        ("audit", "AUD-001"),
        ("tech-spec", "TECH-001"),
    ];
    for (ty, id) in cases {
        let view = json_ok(&["--dir", &d, "--json", ty, "view", id]);
        assert_eq!(
            view["id"], *id,
            "{ty}: top-level identity is the display ID"
        );
        assert_metadata_shape(&view["metadata"], id);
        // Canonical order: general keys before type-specific keys.
        let keys: Vec<&str> = view["metadata"]
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        assert!(
            keys.starts_with(&[
                "id",
                "title",
                "created_date",
                "updated_date",
                "labels",
                "references"
            ]),
            "{ty} view keys: {keys:?}"
        );

        let list = json_ok(&["--dir", &d, "--json", ty, "list"]);
        let item = list
            .as_array()
            .expect("list json is an array")
            .iter()
            .find(|item| item["id"] == *id)
            .expect("list item");
        assert_metadata_shape(&item["metadata"], id);

        // Plain-text list lines start with the display id + title and
        // carry the stored created_date.
        let text = run_ok(&["--dir", &d, ty, "list"]);
        let line = text.lines().find(|l| l.starts_with(id)).expect("list line");
        let created = view["metadata"]["created_date"].as_str().unwrap();
        assert!(
            line.contains(created),
            "{ty} list line missing created_date: {line}"
        );
        let updated = view["metadata"]["updated_date"].as_str().unwrap();
        assert!(
            line.contains(updated),
            "{ty} list line missing updated_date: {line}"
        );
        assert!(
            line.contains("labels="),
            "{ty} list line missing labels: {line}"
        );
        assert!(
            line.contains("references="),
            "{ty} list line missing references: {line}"
        );
    }
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn identity_positions_use_display_id() {
    let dir = test_root("identity");
    run_ok(&["--dir", &arg_dir(&dir), "init", "Proj", "--no-git"]);
    let d = arg_dir(&dir);
    run_ok(&["--dir", &d, "task", "create", "T"]);

    let view = json_ok(&["--dir", &d, "--json", "task", "view", "TASK-001"]);
    assert_eq!(view["id"], "TASK-001");
    assert_eq!(view["metadata"]["id"], "TASK-001");

    let text = run_ok(&["--dir", &d, "task", "list"]);
    assert!(
        text.lines().next().unwrap().starts_with("TASK-001 "),
        "list line identity: {text}"
    );

    let server = Server::start(&dir);
    let api: serde_json::Value =
        serde_json::from_str(&get(&server, "/api/doc/TASK-001").text()).unwrap();
    assert_eq!(
        api["id"], "TASK-001",
        "browser JSON identity is the display ID"
    );
    let view_html = get(&server, "/doc/TASK-001").text();
    assert!(
        view_html.contains("<h1 class=\"h1\">T</h1>"),
        "heading uses display id context"
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn browser_and_cli_metadata_parity() {
    let dir = test_root("parity");
    run_ok(&["--dir", &arg_dir(&dir), "init", "Proj", "--no-git"]);
    let d = arg_dir(&dir);
    run_ok(&["--dir", &d, "task", "create", "Parity", "--label", "bug"]);

    let cli = json_ok(&["--dir", &d, "--json", "task", "view", "TASK-001"]);
    let server = Server::start(&dir);
    let api: serde_json::Value =
        serde_json::from_str(&get(&server, "/api/doc/TASK-001").text()).unwrap();
    assert_eq!(
        api["metadata"], cli["metadata"],
        "browser detail JSON metadata equals CLI view metadata"
    );
    // List page shows the same metadata values.
    let list_html = get(&server, "/?type=task").text();
    let created = cli["metadata"]["created_date"].as_str().unwrap();
    assert!(list_html.contains(created), "list HTML shows created_date");
    let detail_html = get(&server, "/doc/TASK-001").text();
    assert!(
        detail_html.contains("Created date"),
        "detail HTML shows Created date"
    );
    assert!(
        detail_html.contains("Updated date"),
        "detail HTML shows Updated date"
    );
    assert!(detail_html.contains("Labels"), "detail HTML shows Labels");
    assert!(
        detail_html.contains("References"),
        "detail HTML shows References"
    );
    let _ = fs::remove_dir_all(&dir);
}

fn task_path(dir: &Path) -> PathBuf {
    dir.join("backlog/tasks/TASK-001.md")
}

#[test]
fn browser_save_round_trip_patches_front_matter() {
    let dir = test_root("roundtrip");
    run_ok(&["--dir", &arg_dir(&dir), "init", "Proj", "--no-git"]);
    run_ok(&["--dir", &arg_dir(&dir), "task", "create", "Before"]);
    // Unknown foreign key that must survive the save.
    let path = task_path(&dir);
    let original = fs::read_to_string(&path).unwrap();
    let with_foreign = original.replace("\n---\n", "\nforeign_key: keep-me\n---\n");
    assert!(with_foreign.contains("foreign_key: keep-me"));
    fs::write(&path, &with_foreign).unwrap();
    let created_before = fm_value(&with_foreign, "created_date");

    let server = Server::start(&dir);
    let edit = get(&server, "/doc/TASK-001/edit").text();
    assert!(
        edit.contains("name=\"created_date\"")
            && edit.contains("disabled")
            && edit.contains("(read-only)"),
        "edit form has read-only created_date"
    );
    assert!(
        edit.contains("name=\"updated_date\"") && edit.contains("disabled"),
        "edit form has read-only updated_date"
    );
    let version = extract_version(&edit);

    std::thread::sleep(Duration::from_millis(1100));
    let body = form_body(&[
        ("version", &version),
        ("title", "After"),
        ("status", "ToDo"),
        ("description", ""),
        ("milestone", ""),
        ("assignee", ""),
        ("labels", ""),
        ("dependencies", ""),
        ("acceptance_criteria", ""),
        // Forged timestamps must be ignored (read-only).
        ("created_date", "1999-01-01T00:00:00Z"),
        ("updated_date", "1999-01-01T00:00:00Z"),
    ]);
    let response = post_form(&server, "/doc/TASK-001", &body);
    assert_eq!(response.status, 303, "{}", response.text());
    drop(server);

    let saved = fs::read_to_string(&path).unwrap();
    assert_eq!(
        fm_value(&saved, "title"),
        "After",
        "title patched in front matter"
    );
    assert_eq!(
        fm_value(&saved, "created_date"),
        created_before,
        "created_date never changes"
    );
    assert_ne!(
        fm_value(&saved, "updated_date"),
        "1999-01-01T00:00:00Z",
        "forged updated_date ignored"
    );
    assert!(
        fm_value(&saved, "updated_date") >= created_before,
        "updated_date refreshed"
    );
    assert!(
        saved.contains("foreign_key: keep-me"),
        "unknown front-matter key preserved"
    );
    assert!(
        !body_of(&saved).contains("created_date:"),
        "metadata never copied into body: {}",
        body_of(&saved)
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn browser_noop_save_does_not_refresh_updated_date() {
    let dir = test_root("noop");
    run_ok(&["--dir", &arg_dir(&dir), "init", "Proj", "--no-git"]);
    run_ok(&["--dir", &arg_dir(&dir), "task", "create", "Same"]);
    let path = task_path(&dir);
    let server = Server::start(&dir);
    let edit = get(&server, "/doc/TASK-001/edit").text();
    let version = extract_version(&edit);
    let before = fs::read_to_string(&path).unwrap();
    let body = form_body(&[
        ("version", &version),
        ("title", "Same"),
        ("status", "ToDo"),
        ("description", ""),
        ("milestone", ""),
        ("assignee", ""),
        ("labels", ""),
        ("dependencies", ""),
        ("acceptance_criteria", ""),
    ]);
    let response = post_form(&server, "/doc/TASK-001", &body);
    assert_eq!(response.status, 303, "{}", response.text());
    drop(server);
    let after = fs::read_to_string(&path).unwrap();
    assert_eq!(
        before, after,
        "no-op save leaves the file byte-identical (no updated_date refresh)"
    );
    let _ = fs::remove_dir_all(&dir);
}
