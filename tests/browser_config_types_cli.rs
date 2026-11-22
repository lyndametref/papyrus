//! TASK-114: registry-derived browser views for config-defined document types.
//!
//! Scenarios T-114-01 .. T-114-14 from
//! `docs/specifications/browser-config-defined-document-types.md`.

use papyrus::browser::routes::BrowserRoutes;
use papyrus::commands::load_effective_config;
use papyrus::registry::{ArtifactCategory, ArtifactRegistry};
use std::collections::BTreeSet;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdout, Command, Stdio};
use std::time::Duration;

const NOTE_BLOCK: &str = "  note:\n    prefix: NOTE\n    directory: notes/\n    enabled: true\n    statuses: [active, review, approved, archived]\n";

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_papyrus"))
}

fn root(name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(".tmp");
    path.push(format!("t114-{}-{}", name, std::process::id()));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path
}

fn run(project: &Path, args: &[&str]) -> std::process::Output {
    let mut full = vec!["--dir", project.to_str().unwrap()];
    full.extend_from_slice(args);
    bin().args(&full).output().unwrap()
}

fn run_ok(project: &Path, args: &[&str]) -> String {
    let out = run(project, args);
    assert!(
        out.status.success(),
        "command failed: {:?}\nstderr: {}",
        args,
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn init(name: &str) -> PathBuf {
    let dir = root(name);
    run_ok(&dir, &["init", "Proj", "--no-git"]);
    add_note(&dir);
    dir
}

fn add_note(project: &Path) {
    let path = project.join("papyrus.config.yml");
    let cfg = fs::read_to_string(&path).unwrap();
    let updated = cfg.replacen(
        "document_types:\n",
        &format!("document_types:\n{}", NOTE_BLOCK),
        1,
    );
    assert_ne!(updated, cfg);
    fs::write(&path, updated).unwrap();
}

fn add_document_type(project: &Path, slug: &str, prefix: &str, directory: &str) {
    let path = project.join("papyrus.config.yml");
    let cfg = fs::read_to_string(&path).unwrap();
    let block = format!(
        "  {slug}:\n    prefix: {prefix}\n    directory: {directory}\n    enabled: true\n    statuses: [active, review, archived]\n"
    );
    let updated = cfg.replacen("document_types:\n", &format!("document_types:\n{block}"), 1);
    assert_ne!(updated, cfg);
    fs::write(&path, updated).unwrap();
}

fn create_note(project: &Path, title: &str, description: &str) -> String {
    run_ok(
        project,
        &["note", "create", title, "--description", description],
    )
}

fn write_record(project: &Path, id: &str, content: &str) {
    let dir = project.join("docs/notes");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join(format!("{}.md", id)), content).unwrap();
}

fn free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap().port()
}

struct Server {
    child: Child,
    port: u16,
    _stdout: BufReader<ChildStdout>,
}

impl Server {
    fn start(project: &Path) -> Server {
        for _ in 0..10 {
            let port = free_port();
            let stderr = project.join(format!(".stderr-{}", port));
            let err = fs::File::create(&stderr).unwrap();
            let mut child = bin()
                .arg("--dir")
                .arg(project)
                .arg("browser")
                .arg("--port")
                .arg(port.to_string())
                .arg("--no-open")
                .stdout(Stdio::piped())
                .stderr(Stdio::from(err))
                .spawn()
                .unwrap();
            let stdout = child.stdout.take().unwrap();
            let mut reader = BufReader::new(stdout);
            let mut startup = String::new();
            let _ = reader.read_line(&mut startup);
            if !startup.trim().is_empty() {
                return Server {
                    child,
                    port,
                    _stdout: reader,
                };
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

fn parse(bytes: &[u8]) -> HttpResponse {
    let split = bytes
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .expect("header terminator");
    let head = String::from_utf8_lossy(&bytes[..split]).to_string();
    let mut lines = head.split("\r\n");
    let status = lines
        .next()
        .unwrap()
        .split(' ')
        .nth(1)
        .unwrap()
        .parse()
        .unwrap();
    let headers = lines
        .filter_map(|line| {
            line.split_once(':')
                .map(|(name, value)| (name.trim().to_string(), value.trim().to_string()))
        })
        .collect();
    HttpResponse {
        status,
        headers,
        body: bytes[split + 4..].to_vec(),
    }
}

fn request(port: u16, method: &str, path: &str, body: Option<&str>) -> HttpResponse {
    let mut raw = format!(
        "{} {} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n",
        method, path, port
    );
    if let Some(body) = body {
        raw.push_str("Content-Type: application/x-www-form-urlencoded\r\n");
        raw.push_str(&format!("Content-Length: {}\r\n", body.len()));
    }
    raw.push_str("\r\n");
    if let Some(body) = body {
        raw.push_str(body);
    }
    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(15)))
        .unwrap();
    stream.write_all(raw.as_bytes()).unwrap();
    let mut bytes = Vec::new();
    stream.read_to_end(&mut bytes).unwrap();
    parse(&bytes)
}

fn get(s: &Server, path: &str) -> HttpResponse {
    request(s.port, "GET", path, None)
}

fn post(s: &Server, path: &str, body: &str) -> HttpResponse {
    request(s.port, "POST", path, Some(body))
}

fn post_content_type(s: &Server, path: &str, body: &str, content_type: &str) -> HttpResponse {
    let raw = format!(
        "POST {} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
        path,
        s.port,
        content_type,
        body.len(),
        body
    );
    let mut stream = TcpStream::connect(("127.0.0.1", s.port)).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(15)))
        .unwrap();
    stream.write_all(raw.as_bytes()).unwrap();
    let mut bytes = Vec::new();
    stream.read_to_end(&mut bytes).unwrap();
    parse(&bytes)
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

fn form(fields: &[(&str, &str)]) -> String {
    fields
        .iter()
        .map(|(key, value)| format!("{}={}", urlencode(key), urlencode(value)))
        .collect::<Vec<_>>()
        .join("&")
}

fn version_of(html: &str) -> String {
    let marker = "name=\"version\" value=\"";
    let start = html.find(marker).expect("version field") + marker.len();
    let rest = &html[start..];
    let end = rest.find('"').unwrap();
    rest[..end].to_string()
}

/// Values of the enabled `<option>` elements inside a filter form, in order.
fn enabled_option_values(filter: &str) -> Vec<String> {
    filter
        .split("<option ")
        .skip(1)
        .filter_map(|chunk| {
            let tag = &chunk[..chunk.find('>').expect("option tag")];
            if tag.contains("disabled") {
                return None;
            }
            let (_, rest) = tag.split_once("value=\"")?;
            Some(rest[..rest.find('"').expect("option value")].to_string())
        })
        .collect()
}

/// The inner HTML of the filter form.
fn filter_form(html: &str) -> &str {
    let start = html.find("class=\"filter\"").expect("filter form");
    let end = start + html[start..].find("</form>").expect("filter end");
    &html[start..end]
}

// T-114-01 -----------------------------------------------------------------
#[test]
fn t114_01_user_added_type_end_to_end() {
    let d = init("01");
    create_note(&d, "First note", "first-description");
    create_note(&d, "Second note", "second-description");
    let server = Server::start(&d);
    let html = get(&server, "/?type=note").text();
    assert_eq!(get(&server, "/?type=note").status, 200);
    // TASK-142: config-defined types render the shared six-column layout.
    assert!(html.contains("data-list-layout=\"shared\""));
    for key in ["id", "title", "status", "created", "updated", "actions"] {
        assert!(
            html.contains(&format!("<th scope=\"col\" data-column=\"{key}\">")),
            "note header {key}"
        );
    }
    assert!(html.contains("NOTE-001"));
    assert!(html.contains("NOTE-002"));
    assert!(html.contains("First note"));
    assert!(html.contains("Second note"));
    assert!(html.contains("href=\"/doc/NOTE-001\""));
    let sidebar_start = html.find(">Documents</p>").expect("Documents group");
    let sidebar_end = html.find("</nav>").expect("sidebar end");
    let sidebar = &html[sidebar_start..sidebar_end];
    assert!(sidebar.contains("Notes"), "sidebar={sidebar}");
    assert_eq!(get(&server, "/doc/NOTE-001").status, 200);
    assert_eq!(get(&server, "/doc/NOTE-001/edit").status, 200);
    let api: serde_json::Value =
        serde_json::from_str(&get(&server, "/api/docs/note").text()).unwrap();
    let rows = api.as_array().unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0]["id"], "NOTE-001");
    assert_eq!(rows[0]["kind"], "note");
    assert!(rows[0].get("title").is_some());
    assert!(rows[0].get("status").is_some());
    assert!(rows[0].get("labels").is_some());
    assert!(rows[0].get("references").is_some());
    assert!(rows[0].get("version").is_some());
    assert!(rows[0].get("path").is_some());
    assert!(rows[0].get("metadata").is_some());
    drop(server);
    let _ = fs::remove_dir_all(&d);
}

// T-114-02 -----------------------------------------------------------------
#[test]
fn t114_02_canonical_api_equals_legacy() {
    let d = init("02");
    create_note(&d, "Note", "body");
    run_ok(
        &d,
        &[
            "glossary",
            "create",
            "--title",
            "Term",
            "--definition",
            "Definition",
        ],
    );
    run_ok(
        &d,
        &[
            "rule",
            "create",
            "Rule",
            "--description",
            "Description",
            "--rationale",
            "Why",
            "--applicability",
            "Always",
        ],
    );
    run_ok(
        &d,
        &[
            "audit",
            "create",
            "--title",
            "Audit",
            "--content",
            "Content",
            "--audit-type",
            "security",
        ],
    );
    let server = Server::start(&d);
    for (canonical, legacy) in [
        ("/api/docs/glossary", "/api/glossary"),
        ("/api/docs/rule", "/api/business-rules"),
        ("/api/docs/audit", "/api/audit-records"),
    ] {
        assert_eq!(
            get(&server, canonical).text(),
            get(&server, legacy).text(),
            "{canonical}"
        );
    }
    let req = get(&server, "/api/docs/req").text();
    let legacy_req = get(&server, "/api/requirements?type=req").text();
    assert_eq!(req, legacy_req);
    assert_eq!(
        get(&server, "/api/docs/ui-mockup").text(),
        get(&server, "/api/mockups").text()
    );
    drop(server);
    let _ = fs::remove_dir_all(&d);
}

// T-114-03 -----------------------------------------------------------------
#[test]
fn t114_03_status_read_only_and_rejected() {
    let d = init("03");
    create_note(&d, "Review me", "body");
    run_ok(&d, &["note", "status", "NOTE-001", "review"]);
    let server = Server::start(&d);
    let list = get(&server, "/?type=note").text();
    let status_at = list.find(">review<").expect("status cell");
    let id_at = list.find("NOTE-001").expect("id");
    assert!(status_at < id_at, "status must be left of ID");
    let detail = get(&server, "/doc/NOTE-001").text();
    assert!(detail.contains("review"));
    assert!(detail.contains("Created date"));
    let edit = get(&server, "/doc/NOTE-001/edit").text();
    assert!(!edit.contains("name=\"status\""));
    assert!(!edit.contains("name=\"status_change\""));
    let before = fs::read_to_string(d.join("docs/notes/NOTE-001.md")).unwrap();
    let version = version_of(&edit);
    let response = post(
        &server,
        "/doc/NOTE-001",
        &form(&[
            ("version", &version),
            ("status", "archived"),
            ("title", "Review me"),
        ]),
    );
    assert_eq!(response.status, 422);
    assert_eq!(
        before,
        fs::read_to_string(d.join("docs/notes/NOTE-001.md")).unwrap()
    );
    drop(server);
    let _ = fs::remove_dir_all(&d);
}

// T-114-04 -----------------------------------------------------------------
#[test]
fn t114_04_filter_values_order_and_default() {
    let d = init("04");
    create_note(&d, "Active", "a");
    create_note(&d, "Review", "r");
    run_ok(&d, &["note", "status", "NOTE-002", "review"]);
    let server = Server::start(&d);
    let html = get(&server, "/?type=note").text();
    let filter = filter_form(&html);
    // The select offers exactly active plus the declared custom statuses, in
    // declaration order. No empty/`All records` option and no archived option.
    assert_eq!(
        enabled_option_values(filter),
        vec!["active", "review", "approved"]
    );
    assert!(filter.contains("<option value=\"\" disabled selected>Status</option>"));
    assert!(!filter.contains("All records"));
    assert!(!filter.contains(">archived<"));
    assert!(!filter.contains("status=archived"));
    let review = get(&server, "/?type=note&status=review").text();
    assert!(review.contains("Review"));
    assert!(!review.contains("Active"));
    assert!(!review.contains("Approved"));
    let review_filter = filter_form(&review);
    assert!(review_filter.contains("<option value=\"review\" selected>review</option>"));
    assert!(!review_filter.contains("disabled selected"));
    let approved = get(&server, "/?type=note&status=approved").text();
    assert_eq!(approved.matches("href=\"/doc/NOTE-").count(), 0);
    // A type with no custom statuses offers only active.
    run_ok(
        &d,
        &[
            "glossary",
            "create",
            "--title",
            "Term",
            "--definition",
            "Definition",
        ],
    );
    let glossary = get(&server, "/?type=glossary").text();
    let filter = filter_form(&glossary);
    assert_eq!(enabled_option_values(filter), vec!["active"]);
    assert!(filter.contains("<option value=\"\" disabled selected>Status</option>"));
    assert!(!filter.contains("All records"));
    assert!(!filter.contains(">review<"));
    drop(server);
    let _ = fs::remove_dir_all(&d);
}

// T-114-05 -----------------------------------------------------------------
#[test]
fn t114_05_invalid_filters_are_400() {
    let d = init("05");
    create_note(&d, "Note", "body");
    let server = Server::start(&d);
    for query in [
        "status=archived",
        "status=bogus",
        "status=Review",
        "status=",
        "status=review&status=active",
    ] {
        let response = get(&server, &format!("/?type=note&{}", query));
        assert_eq!(response.status, 400, "query={}", query);
        assert!(!response.text().contains("NOTE-001"));
    }
    drop(server);
    let _ = fs::remove_dir_all(&d);
}

// T-114-06 -----------------------------------------------------------------
#[test]
fn t114_06_default_lists_every_status() {
    let d = init("06");
    create_note(&d, "Active", "a");
    create_note(&d, "Review", "r");
    run_ok(&d, &["note", "status", "NOTE-002", "review"]);
    create_note(&d, "Approved", "p");
    run_ok(&d, &["note", "status", "NOTE-003", "approved"]);
    let server = Server::start(&d);
    let html = get(&server, "/?type=note").text();
    assert!(html.contains("NOTE-001"));
    assert!(html.contains("NOTE-002"));
    assert!(html.contains("NOTE-003"));
    // Untouched submit: the placeholder is selected (and disabled), so no
    // `status` parameter is sent and every status is listed.
    let filter = filter_form(&html);
    assert!(filter.contains("<option value=\"\" disabled selected>Status</option>"));
    assert!(!filter.contains("selected>active</option>"));
    drop(server);
    let _ = fs::remove_dir_all(&d);
}

// T-114-07 -----------------------------------------------------------------
#[test]
fn t114_07_archived_and_unknown_are_identical_404() {
    let d = init("07");
    create_note(&d, "Archive me", "body");
    run_ok(&d, &["note", "archive", "NOTE-001"]);
    let server = Server::start(&d);
    let archived_detail = get(&server, "/doc/NOTE-001");
    let unknown_detail = get(&server, "/doc/NOTE-999");
    assert_eq!(archived_detail.status, 404);
    assert_eq!(unknown_detail.status, 404);
    assert_eq!(archived_detail.text(), unknown_detail.text());
    let archived_edit = get(&server, "/doc/NOTE-001/edit");
    assert_eq!(archived_edit.text(), unknown_detail.text());
    assert_eq!(get(&server, "/api/doc/NOTE-001").status, 404);
    assert_eq!(
        get(&server, "/api/doc/NOTE-001").text(),
        get(&server, "/api/doc/NOTE-999").text()
    );
    assert!(!get(&server, "/?type=note").text().contains("NOTE-001"));
    assert!(!get(&server, "/api/docs/note").text().contains("NOTE-001"));
    drop(server);
    let _ = fs::remove_dir_all(&d);
}

// T-114-08 -----------------------------------------------------------------
#[test]
fn t114_08_archived_in_active_is_relocated_and_hidden() {
    let d = init("08");
    write_record(
        &d,
        "NOTE-002",
        "---\nid: NOTE-002\ntitle: Hand edited\nstatus: archived\ncreated_date: 2026-01-01T00:00:00Z\nupdated_date: 2026-01-01T00:00:00Z\n---\n# NOTE-002 Hand edited\n\n## Description\nBody\n",
    );
    let server = Server::start(&d);
    let html = get(&server, "/?type=note").text();
    assert!(!html.contains("NOTE-002"));
    assert!(d.join("docs/notes/archived/NOTE-002.md").exists());
    assert!(!d.join("docs/notes/NOTE-002.md").exists());
    assert_eq!(get(&server, "/doc/NOTE-002").status, 404);
    assert_eq!(get(&server, "/doc/NOTE-002/edit").status, 404);
    assert_eq!(get(&server, "/api/doc/NOTE-002").status, 404);
    drop(server);
    let _ = fs::remove_dir_all(&d);
}

// T-114-09 -----------------------------------------------------------------
#[test]
fn t114_09_disabled_type_is_invisible_but_lifecycle_lists_it() {
    let d = init("09");
    create_note(&d, "Note", "body");
    run_ok(
        &d,
        &["config", "set", "document_types.note.enabled", "false"],
    );
    let server = Server::start(&d);
    let inventory = get(&server, "/").text();
    assert!(!inventory.contains("href=\"/?type=note\""));
    assert_eq!(get(&server, "/?type=note").status, 404);
    assert_eq!(get(&server, "/api/docs/note").status, 404);
    assert_eq!(get(&server, "/doc/NOTE-001").status, 404);
    drop(server);
    let _ = fs::remove_dir_all(&d);
}

// T-114-10 -----------------------------------------------------------------
#[test]
fn t114_10_generic_edit_round_trip_noop_and_conflict() {
    let d = init("10");
    write_record(
        &d,
        "NOTE-003",
        "---\nid: NOTE-003\ntitle: Old\nstatus: review\ncreated_date: 2026-01-01T00:00:00Z\nupdated_date: 2026-01-01T00:00:00Z\nlabels:\n  - old\nreferences:\n  - REF-001\nunknown_key: keep-me\n---\n# NOTE-003 Old\n\n## Description\nOld body\n\n## Notes\nUnknown section body\n",
    );
    let server = Server::start(&d);
    let edit = get(&server, "/doc/NOTE-003/edit").text();
    let stale_version = version_of(&edit);
    let body = form(&[
        ("version", &stale_version),
        ("title", "New"),
        ("description", "New body"),
        ("labels", "one"),
        ("references", "REF-002"),
    ]);
    let saved = post(&server, "/doc/NOTE-003", &body);
    assert_eq!(saved.status, 303);
    let text = fs::read_to_string(d.join("docs/notes/NOTE-003.md")).unwrap();
    assert!(text.contains("title: New"));
    assert!(text.contains("# NOTE-003 New"));
    assert!(text.contains("## Description\nNew body"));
    assert!(text.contains("## Notes\nUnknown section body"));
    assert!(text.contains("unknown_key: keep-me"));
    assert!(text.contains("status: review"));
    assert!(!text.contains("updated_date: 2026-01-01T00:00:00Z"));
    // Identical second save performs no write.
    let edit = get(&server, "/doc/NOTE-003/edit").text();
    let version = version_of(&edit);
    let same = form(&[
        ("version", &version),
        ("title", "New"),
        ("description", "New body"),
        ("labels", "one"),
        ("references", "REF-002"),
    ]);
    let before = fs::read_to_string(d.join("docs/notes/NOTE-003.md")).unwrap();
    assert_eq!(post(&server, "/doc/NOTE-003", &same).status, 303);
    assert_eq!(
        before,
        fs::read_to_string(d.join("docs/notes/NOTE-003.md")).unwrap()
    );
    // A pre-save version token is stale and returns 409 with preserved values.
    let stale = form(&[
        ("version", &stale_version),
        ("title", "Stale"),
        ("description", "x"),
    ]);
    let response = post(&server, "/doc/NOTE-003", &stale);
    assert_eq!(response.status, 409);
    assert!(response.text().contains("Stale"));
    drop(server);
    let _ = fs::remove_dir_all(&d);
}

// T-114-11 -----------------------------------------------------------------
#[test]
fn t114_11_builtin_list_uses_shared_columns_and_keeps_sections() {
    let d = init("11");
    run_ok(
        &d,
        &[
            "glossary",
            "create",
            "--title",
            "Term",
            "--definition",
            "Definition",
        ],
    );
    run_ok(
        &d,
        &[
            "tech-spec",
            "create",
            "Spec",
            "--description",
            "Description",
        ],
    );
    run_ok(
        &d,
        &[
            "rule",
            "create",
            "Rule",
            "--description",
            "Description",
            "--rationale",
            "Why",
            "--applicability",
            "Always",
        ],
    );
    let server = Server::start(&d);
    let glossary = get(&server, "/?type=glossary").text();
    // TASK-142: the glossary list is the shared table, not the legacy ul row.
    assert!(glossary.contains("data-list-layout=\"shared\""));
    for key in ["id", "title", "status", "created", "updated", "actions"] {
        assert!(
            glossary.contains(&format!("<th scope=\"col\" data-column=\"{key}\">")),
            "glossary header {key}"
        );
    }
    assert!(glossary.contains("class=\"filter\""));
    let tech = get(&server, "/?type=tech-spec").text();
    assert!(tech.contains("Technical Specifications"));
    // TASK-142: the shared list no longer renders the description preview.
    assert!(tech.contains("data-list-layout=\"shared\""));
    assert!(!tech.contains("Description"));
    let rule = get(&server, "/?type=business-rule").text();
    assert!(rule.contains("Business Rules"));
    let spec = get(&server, "/doc/TECH-001").text();
    assert!(!spec.contains("Technical specification"));
    assert!(spec.contains("<p class=\"id\">"));
    drop(server);
    let _ = fs::remove_dir_all(&d);
}

// T-114-13 -----------------------------------------------------------------
#[test]
fn t114_13_no_category_enum_or_form_dispatch_remains() {
    let browser = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/browser");
    for entry in fs::read_dir(&browser).unwrap().flatten() {
        if entry.path().extension().and_then(|value| value.to_str()) != Some("rs") {
            continue;
        }
        let text = fs::read_to_string(entry.path()).unwrap();
        assert!(
            !text.contains("enum Category"),
            "{}",
            entry.path().display()
        );
        assert!(!text.contains("Category::"), "{}", entry.path().display());
        assert!(
            !text.contains("pub enum Form"),
            "{}",
            entry.path().display()
        );
    }
}

// T-114-14 -----------------------------------------------------------------
#[test]
fn t114_14_methods_aliases_and_path_safety() {
    let d = init("14");
    create_note(&d, "Note", "body");
    let server = Server::start(&d);
    let post_api = request(server.port, "POST", "/api/docs/note", Some("{}"));
    assert_eq!(post_api.status, 405);
    assert_eq!(post_api.header("Allow"), Some("GET, HEAD"));
    assert_eq!(get(&server, "/api/docs/unknown").status, 404);
    assert_eq!(get(&server, "/?type=../other").status, 404);
    assert_eq!(get(&server, "/?type=note/../../other").status, 404);
    let legacy = get(&server, "/?type=business-rule");
    let canonical = get(&server, "/?type=rule");
    assert_eq!(legacy.status, 200);
    assert_eq!(canonical.status, 200);
    assert_eq!(legacy.text(), canonical.text());
    drop(server);
    let _ = fs::remove_dir_all(&d);
}

// T-114-05 / DoD #5 ---------------------------------------------------------
#[test]
fn t114_14b_directory_outside_project_is_rejected_without_a_page() {
    let d = root("14b");
    run_ok(&d, &["init", "Proj", "--no-git"]);
    let path = d.join("papyrus.config.yml");
    let cfg = fs::read_to_string(&path).unwrap();
    let updated = cfg.replacen(
        "document_types:\n",
        "document_types:\n  note:\n    prefix: NOTE\n    directory: ../outside/\n    enabled: true\n    statuses: [active, archived]\n",
        1,
    );
    fs::write(&path, updated).unwrap();
    let out = run(&d, &["browser", "--port", "0", "--no-open"]);
    assert_eq!(out.status.code(), Some(4));
    assert!(String::from_utf8_lossy(&out.stderr).contains("stay inside the project directory"));
    let _ = fs::remove_dir_all(&d);
}

// Regression: generic front-matter labels/references projection and no-op save.
#[test]
fn t114_15_generic_labels_and_references_round_trip() {
    let d = init("15");
    let content = "---\nid: NOTE-004\ntitle: Hand note\nstatus: review\ncreated_date: 2026-01-01T00:00:00Z\nupdated_date: 2026-01-02T00:00:00Z\nlabels:\n  - alpha\n  - beta\nreferences:\n  - REF-001\n  - REF-002\n---\n# NOTE-004 Hand note\n\n## Description\nBody text\n";
    write_record(&d, "NOTE-004", content);
    let server = Server::start(&d);

    let detail = get(&server, "/doc/NOTE-004").text();
    for value in ["alpha", "beta", "REF-001", "REF-002"] {
        assert!(detail.contains(value), "detail missing {value}: {detail}");
    }
    assert!(
        detail.contains("<details><summary>References (2)</summary>"),
        "generic single-document view keeps its collapsible References section"
    );

    // TC-055: the generic list omits the reference block entirely but keeps
    // labels in the Title cell.
    let list = get(&server, "/?type=note").text();
    let start = list.find("data-row-id=\"NOTE-004\"").expect("NOTE-004 row");
    let row = &list[start..];
    let row = &row[..row.find("</tr>").unwrap()];
    assert!(row.contains("<li class=\"tag\">alpha</li>"));
    assert!(row.contains("<li class=\"tag\">beta</li>"));
    assert!(
        !row.contains("row-refs"),
        "no reference element in a generic row"
    );
    assert!(!row.contains("REF-001"));

    let api: serde_json::Value =
        serde_json::from_str(&get(&server, "/api/docs/note").text()).unwrap();
    let note = api
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == "NOTE-004")
        .expect("NOTE-004 in API");
    assert_eq!(note["labels"], serde_json::json!(["alpha", "beta"]));
    assert_eq!(
        note["references"],
        serde_json::json!(["REF-001", "REF-002"])
    );

    let edit = get(&server, "/doc/NOTE-004/edit").text();
    assert!(edit.contains("name=\"labels\" type=\"text\" value=\"alpha\""));
    assert!(edit.contains("name=\"labels\" type=\"text\" value=\"beta\""));
    assert!(edit.contains("name=\"references\" type=\"text\" value=\"REF-001\""));
    assert!(edit.contains("name=\"references\" type=\"text\" value=\"REF-002\""));

    let path = d.join("docs/notes/NOTE-004.md");
    let record_before = fs::read(&path).unwrap();
    let config_before = fs::read(d.join("papyrus.config.yml")).unwrap();
    let _ = get(&server, "/?type=note");
    let _ = get(&server, "/doc/NOTE-004");
    assert_eq!(
        record_before,
        fs::read(&path).unwrap(),
        "list rendering changes no record file"
    );
    assert_eq!(
        config_before,
        fs::read(d.join("papyrus.config.yml")).unwrap(),
        "list rendering changes no configuration file"
    );
    let before = fs::read(&path).unwrap();
    let version = version_of(&edit);
    let body = form(&[
        ("version", &version),
        ("title", "Hand note"),
        ("description", "Body text"),
        ("labels", "alpha"),
        ("labels", "beta"),
        ("references", "REF-001"),
        ("references", "REF-002"),
    ]);
    assert_eq!(post(&server, "/doc/NOTE-004", &body).status, 303);
    let after = fs::read(&path).unwrap();
    assert_eq!(before, after, "unchanged save must be byte-identical");
    assert!(String::from_utf8_lossy(&after).contains("updated_date: 2026-01-02T00:00:00Z"));

    drop(server);
    let _ = fs::remove_dir_all(&d);
}

// TASK-142: release, milestone, and decision lists all use the shared table
// and still render no document status filter. The obsolete "keep previous
// layout" contract is replaced by this shared-layout assertion.
#[test]
fn t114_16_non_document_lists_use_the_shared_layout_without_a_filter() {
    let d = init("16");
    run_ok(
        &d,
        &[
            "release",
            "create",
            "Release One",
            "--description",
            "Release description",
        ],
    );
    run_ok(
        &d,
        &[
            "milestone",
            "create",
            "Milestone One",
            "--description",
            "Milestone description",
            "--release",
            "REL-001",
        ],
    );
    run_ok(&d, &["decision", "create", "Decision One"]);
    let server = Server::start(&d);
    for slug in ["release", "milestone", "decision"] {
        let html = get(&server, &format!("/?type={}", slug)).text();
        assert!(
            html.contains("data-list-layout=\"shared\""),
            "{slug} uses the shared layout"
        );
        assert!(
            html.contains("class=\"table list list--shared\""),
            "{slug} renders the shared table"
        );
        assert!(
            !html.contains("class=\"filter\""),
            "{slug} must not render the document status filter"
        );
        assert!(
            !html.contains("<ul class=\"req-list\">"),
            "{slug} is no longer a ul row list"
        );
    }
    assert!(get(&server, "/?type=milestone&status=bogus").status == 200);
    drop(server);
    let _ = fs::remove_dir_all(&d);
}

// Regression (AC11): a hand-written record without labels/references keys is
// not rewritten by an unchanged save, and updated_date stays untouched.
#[test]
fn t114_17_noop_save_without_label_keys_is_byte_identical() {
    let d = init("17");
    let content = "---\nid: NOTE-005\ntitle: Plain\nstatus: active\ncreated_date: 2026-01-01T00:00:00Z\nupdated_date: 2026-01-02T00:00:00Z\n---\n# NOTE-005 Plain\n\n## Description\nBody\n";
    write_record(&d, "NOTE-005", content);
    let path = d.join("docs/notes/NOTE-005.md");
    assert_eq!(fs::read_to_string(&path).unwrap(), content);
    let server = Server::start(&d);

    let edit = get(&server, "/doc/NOTE-005/edit").text();
    let version = version_of(&edit);
    let body = form(&[
        ("version", &version),
        ("title", "Plain"),
        ("description", "Body"),
    ]);
    assert_eq!(post(&server, "/doc/NOTE-005", &body).status, 303);
    let after = fs::read_to_string(&path).unwrap();
    assert_eq!(after, content, "unchanged save must not add keys or dates");
    assert!(after.contains("updated_date: 2026-01-02T00:00:00Z"));
    assert!(!after.contains("labels:"));
    assert!(!after.contains("references:"));

    drop(server);
    let _ = fs::remove_dir_all(&d);
}

// Regression (AC1 / DoD #4): the rendered inventory card set equals
// registry.enabled(Document), exactly one card per enabled document type.
#[test]
fn t114_18_inventory_cards_match_enabled_documents() {
    let d = init("18");
    create_note(&d, "Note", "body");
    let cfg = load_effective_config(&d.join("papyrus.config.yml"), &d).unwrap();
    let registry = ArtifactRegistry::build(&cfg).unwrap();
    let routes = BrowserRoutes::build(&registry);
    let expected: BTreeSet<String> = registry
        .enabled(ArtifactCategory::Document)
        .map(|descriptor| descriptor.slug.clone())
        .collect();

    let server = Server::start(&d);
    let html = get(&server, "/").text();
    let section = html
        .split("<section class=\"inventory\">")
        .nth(1)
        .expect("inventory section")
        .split("</section>")
        .next()
        .unwrap();
    let mut actual: BTreeSet<String> = BTreeSet::new();
    for article in section.split("<article class=\"card\">").skip(1) {
        let Some(start) = article.find("href=\"/?type=") else {
            continue;
        };
        let rest = &article[start + "href=\"/?type=".len()..];
        let end = rest.find('"').expect("card href terminator");
        let key = &rest[..end];
        let Some(slug) = routes.slug_for_type(key) else {
            continue;
        };
        if registry
            .descriptor(slug)
            .is_some_and(|descriptor| descriptor.category == ArtifactCategory::Document)
        {
            assert!(actual.insert(slug.to_string()), "duplicate card for {slug}");
        }
    }
    assert_eq!(actual, expected);
    assert_eq!(
        actual.len(),
        registry.enabled(ArtifactCategory::Document).count()
    );

    drop(server);
    let _ = fs::remove_dir_all(&d);
}

// TC-056 / TC-057: a document type configured after the change inherits the
// Generic behaviour with no per-type presentation configuration, and the rest
// of the shared list contract is unchanged.
#[test]
fn t182_01_newly_configured_type_omits_references_and_keeps_the_shared_layout() {
    let d = root("182-01");
    run_ok(&d, &["init", "Proj", "--no-git"]);
    add_document_type(&d, "recipe", "REC", "recipes/");
    run_ok(
        &d,
        &[
            "recipe",
            "create",
            "Recipe One",
            "--description",
            "body one",
            "--label",
            "cooking",
            "--reference",
            "REF-R1",
        ],
    );
    run_ok(
        &d,
        &[
            "recipe",
            "create",
            "Recipe Two",
            "--description",
            "body two",
        ],
    );
    run_ok(&d, &["recipe", "status", "REC-002", "review"]);
    let server = Server::start(&d);

    let html = get(&server, "/?type=recipe").text();
    assert!(html.contains("data-list-layout=\"shared\""));
    for key in ["id", "title", "status", "created", "updated", "actions"] {
        assert!(
            html.contains(&format!("<th scope=\"col\" data-column=\"{key}\">")),
            "recipe header {key}"
        );
    }
    let start = html.find("data-row-id=\"REC-001\"").expect("REC-001 row");
    let row = &html[start..];
    let row = &row[..row.find("</tr>").unwrap()];
    assert!(row.contains("<li class=\"tag\">cooking</li>"));
    assert!(
        !row.contains("row-refs"),
        "no reference block for a new Generic type"
    );
    assert!(!row.contains("REF-R1"));
    assert!(row.contains("aria-label=\"View REC-001\">View</a>"));
    assert!(row.contains("aria-label=\"Edit REC-001\">Edit</a>"));
    assert!(html.contains("class=\"filter\""));
    assert!(
        html.contains("<div class=\"app\">"),
        "shared page width and shell"
    );

    let review = get(&server, "/?type=recipe&status=review").text();
    assert!(review.contains("REC-002"));
    assert!(!review.contains("REC-001"));
    let active = get(&server, "/?type=recipe&status=active").text();
    assert!(active.contains("REC-001"));
    assert!(!active.contains("REC-002"));

    drop(server);
    let _ = fs::remove_dir_all(&d);
}

// TC-059: CLI list and view keep references for config-defined document types.
#[test]
fn t182_02_cli_list_and_view_keep_generic_references() {
    let d = init("182-02");
    run_ok(
        &d,
        &[
            "note",
            "create",
            "CLI note",
            "--description",
            "body",
            "--label",
            "alpha",
            "--reference",
            "REF-CLI-1",
        ],
    );
    let list = run_ok(&d, &["note", "list"]);
    assert!(list.contains("NOTE-001"));
    assert!(list.contains("references=REF-CLI-1"), "list={list}");
    let view = run_ok(&d, &["note", "view", "NOTE-001"]);
    assert!(view.contains("REF-CLI-1"), "view={view}");
    assert!(view.contains("alpha"), "view={view}");
    let _ = fs::remove_dir_all(&d);
}

// TC-063: the glossary landing page display is unchanged and gains no
// page-level reference section or field.
#[test]
fn t182_03_glossary_landing_page_display_is_unchanged() {
    let d = init("182-03");
    run_ok(
        &d,
        &[
            "glossary",
            "create",
            "--title",
            "Term One",
            "--definition",
            "Definition One",
            "--label",
            "glossary-label",
        ],
    );
    run_ok(
        &d,
        &[
            "glossary",
            "create",
            "--title",
            "Term Two",
            "--definition",
            "Definition Two",
        ],
    );
    let server = Server::start(&d);
    let html = get(&server, "/?type=glossary").text();
    assert!(html.contains("<h1 class=\"h1\">Glossary</h1>"));
    assert!(html.contains("data-list-layout=\"shared\""));
    assert!(html.contains("Term One"));
    assert!(html.contains("Term Two"));
    assert!(html.contains("<li class=\"tag\">glossary-label</li>"));
    assert!(
        !html.contains("<h2>References</h2>"),
        "no page-level reference list"
    );
    assert!(
        !html.contains("<dt>References</dt>"),
        "no reference field added"
    );

    drop(server);
    let _ = fs::remove_dir_all(&d);
}

// TC-065 / NFR-01.003: the list views and their assets are fully local.
#[test]
fn t182_04_list_views_and_assets_require_no_network_access() {
    let d = init("182-04");
    run_ok(
        &d,
        &[
            "milestone",
            "create",
            "Milestone One",
            "--description",
            "body",
        ],
    );
    let server = Server::start(&d);
    for path in ["/?type=note", "/?type=milestone", "/?type=ui-mockup"] {
        let html = get(&server, path).text();
        assert!(!html.contains("http://"), "{path} references http");
        assert!(!html.contains("https://"), "{path} references https");
        assert!(
            !html.contains("href=\"//"),
            "{path} references a protocol-relative URL"
        );
        assert!(
            !html.contains("src=\"//"),
            "{path} references a protocol-relative script"
        );
        assert!(
            html.contains("href=\"/assets/app.css\""),
            "{path} uses local assets"
        );
    }
    for asset in ["/assets/app.css", "/assets/app.js", "/assets/nav-state.js"] {
        let body = get(&server, asset).text();
        assert!(!body.contains("http://"), "{asset} references http");
        assert!(!body.contains("https://"), "{asset} references https");
        assert!(
            !body.contains("@import"),
            "{asset} imports a remote stylesheet"
        );
    }

    drop(server);
    let _ = fs::remove_dir_all(&d);
}

// ---------------------------------------------------------------------------
// TASK-266: one generic document edit form and one form_post handler.
// ---------------------------------------------------------------------------

fn write_doc(project: &Path, relative: &str, content: &str) {
    let path = project.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

fn position(html: &str, needle: &str) -> usize {
    html.find(needle)
        .unwrap_or_else(|| panic!("missing {needle:?} in page"))
}

#[test]
fn t266_form_single_and_group_roles() {
    let d = init("266-form");
    write_doc(
        &d,
        "docs/glossary/GLO-001.md",
        "---\nid: GLO-001\ntitle: Term one\nstatus: active\ncreated_date: 2026-01-01T00:00:00Z\nupdated_date: 2026-01-01T00:00:00Z\nlabels:\n  - core\nreferences:\n  - REF-001\n---\n# GLO-001 Term one\n\n## Definition\nA term.\n",
    );
    write_doc(
        &d,
        "docs/requirements/business/REQ-001.md",
        "---\nid: REQ-001\ntitle: Group\nstatus: active\ncreated_date: 2026-01-01T00:00:00Z\nupdated_date: 2026-01-01T00:00:00Z\n---\n# REQ-001 Group\n\n## User Story\nGroup story\n\n## Description\nGroup description\n",
    );
    write_doc(
        &d,
        "docs/requirements/business/REQ-001.001.md",
        "---\nid: REQ-001.001\ntitle: Member\nstatus: active\ngroup: REQ-001\ncreated_date: 2026-01-01T00:00:00Z\nupdated_date: 2026-01-01T00:00:00Z\n---\n# REQ-001.001 Member\n\n## User Story\nMember story\n\n## Description\nMember description\n",
    );
    let server = Server::start(&d);

    let edit = get(&server, "/doc/GLO-001/edit").text();
    assert!(!edit.contains("name=\"status\""));
    assert!(edit.contains("<label for=\"definition\">Definition"));
    // Title, declared fields, general controls, read-only status/dates, token.
    let title = position(&edit, "id=\"title\"");
    let definition = position(&edit, "id=\"definition\"");
    let labels = position(&edit, "id=\"labels\"");
    let references = position(&edit, "id=\"references\"");
    let status = position(&edit, "id=\"status\"");
    let created = position(&edit, "id=\"created_date\"");
    let updated = position(&edit, "id=\"updated_date\"");
    let version = position(&edit, "name=\"version\"");
    assert!(title < definition);
    assert!(definition < labels);
    assert!(labels < references);
    assert!(references < status);
    assert!(status < created);
    assert!(created < updated);
    assert!(updated < version);
    assert!(
        edit.contains("<label for=\"definition\">Definition <span class=\"req\">(required)</span>")
    );
    assert!(edit.contains("aria-describedby=\"definition-help\""));

    let group = get(&server, "/doc/REQ-001/edit").text();
    assert!(group.contains("<label for=\"story\">"));
    assert!(group.contains("<label for=\"description\">"));
    assert!(!group.contains("name=\"group\""));
    let group_story = position(&group, "id=\"story\"");
    let group_description = position(&group, "id=\"description\"");
    assert!(group_story < group_description);

    let member = get(&server, "/doc/REQ-001.001/edit").text();
    assert!(member.contains("<label for=\"story\">"));
    assert!(member.contains("<label for=\"description\">"));
    assert!(!member.contains("name=\"group\""));
    // The group link is read-only, not a control.
    assert!(member.contains("href=\"/doc/REQ-001\""));
    assert!(member.contains("<dt>Owning group</dt>"));
    drop(server);
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn t266_mockup_form_offers_only_title_specs_and_refs() {
    let d = init("266-mockup");
    write_doc(
        &d,
        "docs/specifications/ui/mockups/MOCK-001.html",
        "<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<title>MOCK-001 \u{2014} Pending mockup</title>\n</head>\n<body>\n<!-- mockup-id: MOCK-001 -->\n<!-- status: pending -->\n<!-- task: TASK-001 -->\n<!-- specs: UI-001 -->\n<!-- refs: MOCK-REF-1 -->\n<!-- superseded-by: -->\n<h1>MOCK-001 Pending mockup</h1>\n<p>Status: pending</p>\n<p>Task: TASK-001</p>\n<p>UI specifications: UI-001</p>\n<p>References: MOCK-REF-1</p>\n</body>\n</html>\n",
    );
    let server = Server::start(&d);
    let edit = get(&server, "/doc/MOCK-001/edit").text();
    assert!(edit.contains("<label for=\"title\">"));
    assert!(edit.contains("name=\"specs\""));
    assert!(edit.contains("name=\"refs\""));
    for forbidden in [
        "name=\"labels\"",
        "name=\"references\"",
        "name=\"status\"",
        "name=\"task\"",
        "name=\"superseded_by\"",
        "name=\"description\"",
    ] {
        assert!(!edit.contains(forbidden), "{forbidden}");
    }
    for read_only in ["Labels", "References", "Status", "Task", "Superseded by"] {
        assert!(edit.contains(read_only), "{read_only}");
    }
    drop(server);
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn t266_status_codes_title_and_noop() {
    let d = init("266-http");
    let path = d.join("docs/glossary/GLO-001.md");
    let source = "---\nid: GLO-001\ntitle: Term one\nstatus: active\ncreated_date: 2026-01-01T00:00:00Z\nupdated_date: 2026-01-01T00:00:00Z\nlabels:\n  - core\nreferences:\n  - REF-001\n---\n# GLO-001 Term one\n\n## Definition\nA term.\n";
    write_doc(&d, "docs/glossary/GLO-001.md", source);
    let before = fs::read(&path).unwrap();
    let server = Server::start(&d);
    let edit = get(&server, "/doc/GLO-001/edit").text();
    let version = version_of(&edit);

    // 415: a non-form media type.
    assert_eq!(
        post_content_type(&server, "/doc/GLO-001", "{}", "application/json").status,
        415
    );
    // 400: missing or malformed token.
    assert_eq!(
        post(&server, "/doc/GLO-001", &form(&[("title", "x")])).status,
        400
    );
    assert_eq!(
        post(
            &server,
            "/doc/GLO-001",
            &form(&[("version", "bad"), ("title", "x")]),
        )
        .status,
        400
    );
    // 404: unknown ID through the one not-found page.
    assert_eq!(
        post(&server, "/doc/GLO-404", &form(&[("version", "1.2.3")])).status,
        404
    );
    assert_eq!(fs::read(&path).unwrap(), before);

    // 409: stale token, submitted values, live token, no write.
    let conflict = post(
        &server,
        "/doc/GLO-001",
        &form(&[("version", "1.2.3"), ("title", "Stale")]),
    );
    assert_eq!(conflict.status, 409);
    let html = conflict.text();
    assert!(html.contains("record changed on disk"));
    assert!(html.contains("Stale"));
    assert_eq!(version_of(&html), version);
    assert_eq!(fs::read(&path).unwrap(), before);

    // 422: status is a reserved key that cannot be written.
    let status_rejected = post(
        &server,
        "/doc/GLO-001",
        &form(&[
            ("version", &version),
            ("title", "Ok"),
            ("status", "archived"),
        ]),
    );
    assert_eq!(status_rejected.status, 422);
    assert_eq!(fs::read(&path).unwrap(), before);

    // 422: title contract.
    for title in ["", &"x".repeat(201), "a\tb"] {
        let response = post(
            &server,
            "/doc/GLO-001",
            &form(&[("version", &version), ("title", title), ("definition", "D")]),
        );
        assert_eq!(response.status, 422, "title={title:?}");
        assert_eq!(fs::read(&path).unwrap(), before);
    }
    // 422: a required field may change but not be cleared.
    let cleared = post(
        &server,
        "/doc/GLO-001",
        &form(&[("version", &version), ("title", "Ok"), ("definition", "")]),
    );
    assert_eq!(cleared.status, 422);
    assert_eq!(fs::read(&path).unwrap(), before);

    // 303: one generic save.
    let saved = post(
        &server,
        "/doc/GLO-001",
        &form(&[
            ("version", &version),
            ("title", "Renamed term"),
            ("definition", "New definition"),
            ("labels", "one"),
            ("references", "REF-002"),
        ]),
    );
    assert_eq!(saved.status, 303);
    assert_eq!(saved.header("Location"), Some("/doc/GLO-001"));
    let saved_text = fs::read_to_string(&path).unwrap();
    assert!(saved_text.contains("Renamed term"));
    assert!(saved_text.contains("# GLO-001 Renamed term"));
    assert!(saved_text.contains("## Definition\nNew definition"));
    assert!(saved_text.contains("status: active"));

    // A second identical submission writes nothing.
    let edit = get(&server, "/doc/GLO-001/edit").text();
    let version = version_of(&edit);
    let same = post(
        &server,
        "/doc/GLO-001",
        &form(&[
            ("version", &version),
            ("title", "Renamed term"),
            ("definition", "New definition"),
            ("labels", "one"),
            ("references", "REF-002"),
        ]),
    );
    assert_eq!(same.status, 303);
    assert_eq!(fs::read(&path).unwrap(), saved_text.as_bytes());

    drop(server);
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn t266_group_key_rejected_and_list_order() {
    let d = init("266-group-list");
    write_doc(
        &d,
        "docs/requirements/business/REQ-001.md",
        "---\nid: REQ-001\ntitle: Group\nstatus: active\n---\n# REQ-001 Group\n\n## User Story\nS\n\n## Description\nD\n",
    );
    write_doc(
        &d,
        "docs/requirements/business/REQ-001.001.md",
        "---\nid: REQ-001.001\ntitle: Member\nstatus: active\ngroup: REQ-001\n---\n# REQ-001.001 Member\n\n## User Story\nS\n\n## Description\nD\n",
    );
    write_doc(
        &d,
        "docs/business-rules/RUL-001.md",
        "---\nid: RUL-001\ntitle: Rule\nstatus: active\n---\n# RUL-001 Rule\n\n## Description\nD\n\n## Rationale\nR\n\n## Applicability\nA\n\n## Requirements\n- REQ-001.001\n",
    );
    let server = Server::start(&d);

    // A group key posted to a member is undeclared: 422 and no write.
    let member_path = d.join("docs/requirements/business/REQ-001.001.md");
    let before = fs::read(&member_path).unwrap();
    let version = version_of(&get(&server, "/doc/REQ-001.001/edit").text());
    let response = post(
        &server,
        "/doc/REQ-001.001",
        &form(&[
            ("version", &version),
            ("title", "Moved"),
            ("group", "REQ-999"),
        ]),
    );
    assert_eq!(response.status, 422);
    assert_eq!(fs::read(&member_path).unwrap(), before);

    // List controls accept repeated values in submission order, collapsing
    // duplicates; create-only fields are never writable.
    let rule_path = d.join("docs/business-rules/RUL-001.md");
    let version = version_of(&get(&server, "/doc/RUL-001/edit").text());
    let saved = post(
        &server,
        "/doc/RUL-001",
        &form(&[
            ("version", &version),
            ("title", "Rule renamed"),
            ("description", "D2"),
            ("rationale", "Ignored rationale"),
            ("applicability", "Ignored applicability"),
            ("requirements", "REQ-001.001"),
            ("requirements", "REQ-001.001"),
            ("requirements", ""),
        ]),
    );
    assert_eq!(saved.status, 303);
    let text = fs::read_to_string(&rule_path).unwrap();
    assert!(text.contains("## Rationale\nR"));
    assert!(text.contains("## Applicability\nA"));
    assert!(!text.contains("Ignored"));
    assert!(text.contains("## Requirements\n- REQ-001.001\n"));
    drop(server);
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn t266_document_path_has_no_per_type_builder_or_template() {
    let browser = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/browser");
    let mod_rs = std::fs::read_to_string(browser.join("mod.rs")).unwrap();
    // The HTML document path is the generic builder and the generic handler.
    // The JSON api_post path stays on its own builders until TASK-269.
    let handler_start = mod_rs.find("fn document_form_post").unwrap();
    let handler_end = mod_rs.find("fn board_failure_response").unwrap();
    let handler = &mod_rs[handler_start..handler_end];
    for builder in [
        "generic_document_ops",
        "requirement_ops",
        "glossary_ops",
        "ui_spec_ops",
        "business_rule_ops",
        "audit_ops",
        "tech_spec_ops",
        "use_case_ops",
    ] {
        assert!(
            !handler.contains(&format!("edit::{builder}")),
            "document handler must not call {builder}"
        );
    }
    let edit_start = mod_rs.find("fn edit_page(").unwrap();
    let edit_end = mod_rs.find("fn media_type(").unwrap();
    let edit_page = &mod_rs[edit_start..edit_end];
    for renderer in [
        "edit_page_glossary(",
        "edit_page_ui_spec(",
        "edit_page_ui_mockup(",
        "edit_page_business_rule(",
        "edit_page_audit_record(",
        "edit_page_tech_spec(",
        "edit_page_use_case(",
    ] {
        assert!(
            !edit_page.contains(&format!("render::{renderer}")),
            "document edit path must not call {renderer}"
        );
    }
    assert!(handler.contains("edit::document_edit_ops"));
    assert!(mod_rs.contains("render::document_edit_page"));
    let render_rs = std::fs::read_to_string(browser.join("render.rs")).unwrap();
    assert!(!render_rs.contains("DOCUMENT_EDIT_TEMPLATE"));
    assert!(
        !browser.join("templates/document_edit.html").exists(),
        "the document-specific template is deleted"
    );
    for template in [
        "shell.html",
        "list.html",
        "view.html",
        "edit.html",
        "error.html",
    ] {
        assert!(browser.join("templates").join(template).exists());
    }
}
