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

    fn get_raw(&self, path: &str) -> String {
        let mut stream = TcpStream::connect(("127.0.0.1", self.port)).unwrap();
        write!(
            stream,
            "GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n\r\n",
            self.port
        )
        .unwrap();
        let mut raw = Vec::new();
        stream.read_to_end(&mut raw).unwrap();
        String::from_utf8_lossy(&raw).to_string()
    }

    fn get(&self, path: &str) -> String {
        Self::body(&self.get_raw(path))
    }

    fn body(raw: &str) -> String {
        raw.split_once("\r\n\r\n")
            .map(|(_, body)| body.to_string())
            .unwrap_or_default()
    }
}

fn header_value(raw: &str, name: &str) -> Option<String> {
    raw.split_once("\r\n\r\n")
        .map(|(head, _)| head)?
        .lines()
        .find_map(|line| {
            let (key, value) = line.split_once(':')?;
            key.eq_ignore_ascii_case(name)
                .then(|| value.trim().to_string())
        })
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
        "/doc/TASK-001",
        "/doc/TASK-001/edit",
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
            html.matches("<nav id=\"site-nav\" aria-label=\"Document types\">")
                .count(),
            1,
            "{path} shell navigation"
        );
        assert_eq!(html.matches("<main").count(), 1, "{path} main");
        assert_eq!(html.matches("<footer").count(), 1, "{path} footer");
        assert_eq!(html.matches("</html>").count(), 1, "{path} document end");
        if path == "/?type=task" {
            assert!(
                html.contains("<div class=\"app app--full\">"),
                "{path} is the board and must render full width"
            );
        } else {
            assert!(
                !html.contains("app--full"),
                "{path} must stay on the capped width class"
            );
        }
        assert!(
            !html.contains("<div class=\"app\"><div class=\"app\""),
            "{path} must not nest shells"
        );
    }
}

#[test]
fn every_page_carries_one_hidden_nav_toggle_and_no_preset_state() {
    let server = Server::start(&fixture_root().join("project"));
    for path in shell_pages() {
        let html = server.get(path);
        assert_eq!(
            html.matches("<button class=\"nav-toggle\"").count(),
            1,
            "{path} has exactly one toggle"
        );
        assert!(
            html.contains(
                "type=\"button\" aria-expanded=\"true\" aria-controls=\"site-nav\" hidden>Navigation</button>"
            ),
            "{path} toggle contract"
        );
        assert!(
            html.contains("<nav id=\"site-nav\" aria-label=\"Document types\">"),
            "{path} controls the un-hidden navigation"
        );
        assert!(
            !html.contains("data-nav"),
            "{path} must not preselect a collapsed state"
        );
    }
}

#[test]
fn width_class_is_server_side_only() {
    let css = include_str!("../src/browser/app.css");
    let app_js = include_str!("../src/browser/app.js");
    let nav_js = include_str!("../src/browser/nav-state.js");
    assert!(
        css.contains(".app--full { max-width: none; }"),
        "the full-width class lifts only the cap"
    );
    assert!(
        css.contains(".app { max-width: 1040px; margin: 0 auto; padding: 0 24px 56px; }"),
        "the capped width stays unchanged"
    );
    assert!(
        !app_js.contains("app--full"),
        "no script may change the width class"
    );
    assert!(!nav_js.contains("app--full"));
    assert!(!app_js.contains("PageWidth"), "no script decides the width");
}

#[test]
fn board_is_full_width_and_every_other_route_is_capped() {
    let server = Server::start(&fixture_root().join("project"));
    for path in shell_pages() {
        let html = server.get(path);
        if path == "/?type=task" {
            assert!(
                html.contains("<div class=\"app app--full\">"),
                "{path} is the board"
            );
        } else {
            assert!(html.contains("<div class=\"app\">"), "{path} is capped");
            assert!(!html.contains("app--full"), "{path} is not the board");
        }
    }

    // The board is a route, not a record type: task detail and edit share the
    // `task` slug and must stay capped (REQ-06.015, AC6).
    for path in ["/doc/TASK-001", "/doc/TASK-001/edit"] {
        let html = server.get(path);
        assert!(
            html.contains("<div class=\"app\">"),
            "{path} must be capped"
        );
        assert!(!html.contains("app--full"), "{path} must not be full width");
    }
}

#[test]
fn pages_serve_the_unchanged_csp_and_only_external_scripts() {
    let server = Server::start(&fixture_root().join("project"));
    for path in ["/", "/?type=task", "/?type=req"] {
        let raw = server.get_raw(path);
        assert_eq!(
            header_value(&raw, "Content-Security-Policy").as_deref(),
            Some(papyrus::browser::http::CSP),
            "{path} CSP"
        );
        let html = Server::body(&raw);
        assert!(
            html.contains("<script src=\"/assets/nav-state.js\"></script>"),
            "{path} loads the pre-paint script synchronously"
        );
        assert!(
            html.contains("<script src=\"/assets/app.js\" defer></script>"),
            "{path} keeps the deferred app script"
        );
        for (index, _) in html.match_indices("<script") {
            assert!(
                html[index + "<script".len()..].starts_with(" src=\""),
                "{path} has an inline script element"
            );
        }
        assert!(!html.contains("<script>"), "{path} inline script body");
    }
}

#[test]
fn pages_have_no_inline_script_style_or_event_handler() {
    let server = Server::start(&fixture_root().join("project"));
    for path in shell_pages() {
        let html = server.get(path);
        for needle in ["onclick=", " onchange=", " style=", "<script>"] {
            assert!(
                !html.contains(needle),
                "{path} contains inline markup: {needle}"
            );
        }
    }
}

#[test]
fn nav_state_asset_is_served_synchronously_and_stays_small() {
    let server = Server::start(&fixture_root().join("project"));
    let raw = server.get_raw("/assets/nav-state.js");
    assert!(raw.starts_with("HTTP/1.1 200"), "status line: {raw:.80}");
    assert_eq!(
        header_value(&raw, "Content-Type").as_deref(),
        Some("text/javascript; charset=utf-8")
    );
    let body = Server::body(&raw);
    assert!(body.len() < 1024, "nav-state.js is {} bytes", body.len());
    assert!(body.contains("localStorage.getItem(\"papyrus.nav\")"));
    assert!(body.contains("\"collapsed\""));
    assert!(body.contains("data-nav"));
    assert!(body.contains("removeAttribute(\"hidden\")"));
}

#[test]
fn toggle_width_and_bleed_contracts_are_in_the_assets() {
    let css = include_str!("../src/browser/app.css");
    let app_js = include_str!("../src/browser/app.js");
    let nav_js = include_str!("../src/browser/nav-state.js");

    assert!(css.contains(":root[data-nav=\"collapsed\"] .layout"));
    assert!(
        css.contains("grid-template-columns: minmax(0, 1fr); gap: 0;"),
        "the collapsed layout keeps one real content track"
    );
    assert!(
        !css.contains("grid-template-columns: 0 "),
        "a zero track would take the removed sidebar's place and zero the main column"
    );
    assert!(css.contains(":root[data-nav=\"collapsed\"] .sidebar { display: none; }"));
    assert!(
        css.contains(":root[data-nav=\"collapsed\"] .main { border-left: 0; padding-left: 0; }")
    );
    assert!(
        css.contains(
            ".h1 { font-size: 30px; line-height: 1.2; margin: 0 0 6px; overflow-wrap: anywhere; }"
        ),
        "the page heading must break inside a narrow content column"
    );
    assert!(css.contains("pre { max-width: 100%; overflow-x: auto; }"));
    assert!(
        css.contains("h2, h3 { overflow-wrap: anywhere; }"),
        "plain section headings wrap inside the narrow content column"
    );
    assert!(
        css.contains(".board-columns:focus-visible"),
        "the scrollable board region is keyboard reachable"
    );
    assert!(css.contains(".nav-toggle[hidden] { display: none; }"));
    assert!(css.contains(".bleed { width: 100%; overflow-x: auto; }"));
    assert!(css.contains(".bleed:focus-visible"));
    assert!(!css.contains("100vw"), "no viewport-unit breakout");
    assert!(
        css.contains("flex-wrap: wrap; align-items: baseline;"),
        "section head wraps"
    );
    assert!(!css.contains("sidebar-toggle"));
    assert!(!css.contains("data-collapsed"));

    assert!(!app_js.contains("matchMedia"));
    assert!(!app_js.contains("sidebar-toggle"));
    assert!(
        !app_js.contains("viewport"),
        "no automatic viewport collapse"
    );
    assert!(app_js.contains("form[data-dirty-guard]"));
    assert!(app_js.contains("localStorage.setItem(\"papyrus.nav\""));

    assert!(nav_js.contains("try {"));
    assert!(nav_js.contains("document.readyState === \"loading\""));
}

#[test]
fn config_and_skills_own_their_horizontal_scrolling() {
    let server = Server::start(&fixture_root().join("project"));

    let config = server.get("/config");
    assert!(
        config.contains(
            "<div class=\"table-scroll\" role=\"region\" aria-label=\"Effective configuration\" tabindex=\"0\">"
        ),
        "the configuration table scrolls inside its own region"
    );

    let skills = server.get("/skills");
    assert!(skills.contains("<pre>"));
    assert!(
        include_str!("../src/browser/app.css")
            .contains("pre { max-width: 100%; overflow-x: auto; }")
    );

    let board = server.get("/?type=task");
    assert!(
        board.contains(
            "<div class=\"board-columns\" role=\"region\" aria-label=\"Task board columns\" tabindex=\"0\">"
        ),
        "the board columns own their horizontal scrolling"
    );
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
