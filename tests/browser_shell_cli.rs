//! TASK-141: the browser shell renders once for every page type, the width
//! class stays server-side, and the board keeps its card set.
//!
//! These tests browse the committed characterization seed project: it has at
//! least one record per family, so every page type renders with content.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_papyrus"))
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
    fn start(project: &PathBuf) -> Server {
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

    fn get(&self, path: &str) -> String {
        let mut stream = TcpStream::connect(("127.0.0.1", self.port)).unwrap();
        write!(
            stream,
            "GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n\r\n",
            self.port
        )
        .unwrap();
        let mut raw = Vec::new();
        stream.read_to_end(&mut raw).unwrap();
        let text = String::from_utf8_lossy(&raw).to_string();
        text.split_once("\r\n\r\n")
            .map(|(_, body)| body.to_string())
            .unwrap_or_default()
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/characterization/lists")
}

fn shell_pages() -> Vec<&'static str> {
    vec![
        "/",
        "/?type=milestone",
        "/?type=task",
        "/?type=release",
        "/?type=decision",
        "/?type=req",
        "/?type=nfr",
        "/?type=glossary",
        "/?type=rule",
        "/?type=audit",
        "/?type=tech-spec",
        "/?type=ui-spec",
        "/?type=ui-mockup",
        "/?type=use-case",
        "/?type=test-case",
        "/?type=test-report",
        "/?type=note",
        "/doc/MS-001",
        "/doc/MS-001/edit",
        "/doc/NOTE-001",
        "/doc/NOTE-001/edit",
        "/lifecycle",
        "/config",
        "/skills",
        "/nope",
    ]
}

#[test]
fn every_page_renders_the_shell_exactly_once() {
    let server = Server::start(&fixture_root().join("project"));
    for path in shell_pages() {
        let html = server.get(path);
        assert!(
            html.starts_with("<!doctype html>"),
            "{path} must be a complete document"
        );
        assert_eq!(
            html.matches("<div class=\"app").count(),
            1,
            "{path} shell wrapper"
        );
        assert_eq!(html.matches("<header").count(), 1, "{path} header");
        assert_eq!(
            html.matches("<nav aria-label=\"Document types\">").count(),
            1,
            "{path} shell navigation"
        );
        assert_eq!(html.matches("<main").count(), 1, "{path} main");
        assert_eq!(html.matches("<footer").count(), 1, "{path} footer");
        assert_eq!(html.matches("</html>").count(), 1, "{path} document end");
        assert!(
            !html.contains("app--full"),
            "{path} must stay on the capped width class"
        );
        assert!(
            !html.contains("<div class=\"app\"><div class=\"app\""),
            "{path} must not nest shells"
        );
    }
}

#[test]
fn width_class_is_server_side_only() {
    let css = include_str!("../src/browser/app.css");
    let js = include_str!("../src/browser/app.js");
    assert!(
        !css.contains(".app--full"),
        "TASK-144 owns the full-width CSS"
    );
    assert!(
        !js.contains("app--full"),
        "no script may change the width class"
    );
    assert!(!js.contains("PageWidth"), "no script decides the width");
}

#[test]
fn every_family_list_uses_the_shared_layout() {
    let server = Server::start(&fixture_root().join("project"));
    let milestone = server.get("/?type=milestone");
    assert!(milestone.contains("data-list-layout=\"shared\""));
    assert!(milestone.contains("<table class=\"table list list--shared\""));
    assert!(milestone.contains("data-column=\"release\">Release"));
    assert!(milestone.contains("data-row-id=\"MS-001\""));
    assert!(milestone.contains("aria-label=\"View MS-001\""));
    assert!(
        milestone.contains(
            "<div class=\"table-scroll\" role=\"region\" aria-label=\"Milestones\" tabindex=\"0\">"
        ),
        "the narrow-width scroll container is focusable and labelled from the list heading"
    );

    // TASK-142: the config-defined note list and the release list joined the
    // shared layout; the legacy document table and ul row list are gone.
    let note = server.get("/?type=note");
    assert!(note.contains("data-list-layout=\"shared\""));
    assert!(note.contains("<table class=\"table list list--shared\""));
    assert!(!note.contains("<table class=\"table\">"));

    let release = server.get("/?type=release");
    assert!(release.contains("data-list-layout=\"shared\""));
    assert!(!release.contains("<ul class=\"req-list\">"));
    assert!(release.contains("<th scope=\"col\" data-column=\"status\">Status</th>"));
}

#[test]
fn detail_and_edit_field_sets_are_unchanged_per_family() {
    // One record per family, taken from the committed characterization
    // fixtures. The typed field sets must not change with the shell move.
    let expected: &[(&str, &[&str])] = &[
        (
            "task",
            &[
                "Status",
                "Milestone",
                "Assignee",
                "Labels",
                "Dependencies",
                "Acceptance criteria",
                "Description",
            ],
        ),
        ("milestone", &["Release", "Description"]),
        ("release", &["Description"]),
        (
            "decision",
            &[
                "Status",
                "Decision",
                "Context",
                "Consequences",
                "Superseded by",
            ],
        ),
        ("requirement", &["User story", "Description"]),
        ("nfr", &["Description"]),
        ("glossary", &["Term", "Definition"]),
        (
            "business-rule",
            &["Description", "Rationale", "Applicability", "Requirements"],
        ),
        ("audit", &["Audit type", "Content"]),
        ("tech-spec", &["Description"]),
        ("ui-spec", &["View", "Description"]),
        ("ui-mockup", &["UI specifications", "References"]),
        (
            "usecase",
            &["Description", "Applicability", "Labels", "References"],
        ),
        (
            "testcase",
            &["Description", "Applicability", "Labels", "References"],
        ),
        (
            "testreport",
            &["Description", "Applicability", "Labels", "References"],
        ),
        ("note", &["Title", "Description"]),
    ];
    let server = Server::start(&fixture_root().join("project"));
    for (family, labels) in expected {
        let fixture: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(fixture_root().join(format!("expected/{family}.json")))
                .unwrap(),
        )
        .unwrap();
        let id = fixture["records"][0]["id"].as_str().unwrap();
        let detail = server.get(&format!("/doc/{id}"));
        for label in ["Record ID", "Created date", "Updated date"] {
            assert!(
                detail.contains(&format!("<dt>{label}</dt>")),
                "{family} {id} detail {label}"
            );
        }
        let edit = server.get(&format!("/doc/{id}/edit"));
        for label in *labels {
            assert!(edit.contains(label), "{family} {id} edit label {label}");
        }
        assert!(
            edit.contains("name=\"version\""),
            "{family} edit keeps the version token"
        );
    }
    // Error page field labels are unchanged.
    let error = server.get("/nope");
    for label in ["status-code", "Back to requirements", "Record not found"] {
        assert!(error.contains(label), "error page label {label}");
    }
}

#[test]
fn board_card_set_matches_its_characterization_fixture() {
    let fixture: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(fixture_root().join("expected/task.json")).unwrap(),
    )
    .unwrap();
    let expected: Vec<String> = fixture["row_order"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap().to_string())
        .collect();
    assert!(!expected.is_empty(), "board fixture covers tasks");
    let server = Server::start(&fixture_root().join("project"));
    let html = server.get("/?type=task");
    let mut actual = Vec::new();
    let mut rest: &str = &html;
    while let Some(found) = rest.find("data-task=\"") {
        let start = found + "data-task=\"".len();
        let end = rest[start..].find('"').unwrap() + start;
        actual.push(rest[start..end].to_string());
        rest = &rest[end..];
    }
    assert_eq!(actual, expected, "board cards unchanged");
    assert!(html.contains("class=\"board-card\""));
}
