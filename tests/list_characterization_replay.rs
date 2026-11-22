//! TASK-142: replay every TASK-141 list characterization fixture against the
//! shared list layout.
//!
//! Record set, row order, per-row values, grouping, archived exclusions, and
//! the heading/count contract are asserted from the pre-change capture. The
//! shared header sequence and row actions are asserted separately because the
//! shared layout is the intended change (feature specification §6). The task
//! fixture is replayed against the REQ-06.013 board.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use papyrus::browser::render::escape;
use serde_json::Value;

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

fn initial_fixtures() -> Vec<Value> {
    let mut fixtures = Vec::new();
    for entry in std::fs::read_dir(fixture_root().join("expected")).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy();
        if !name.ends_with("-empty.json") && name.ends_with(".json") {
            fixtures.push(
                serde_json::from_str::<Value>(&std::fs::read_to_string(&path).unwrap()).unwrap(),
            );
        }
    }
    fixtures.sort_by_key(|fixture| fixture["family"].as_str().unwrap().to_string());
    assert_eq!(fixtures.len(), 16, "16 populated fixtures");
    fixtures
}

fn empty_fixtures() -> Vec<Value> {
    let mut fixtures = Vec::new();
    for entry in std::fs::read_dir(fixture_root().join("expected")).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy();
        if name.ends_with("-empty.json") {
            fixtures.push(
                serde_json::from_str::<Value>(&std::fs::read_to_string(&path).unwrap()).unwrap(),
            );
        }
    }
    fixtures.sort_by_key(|fixture| fixture["family"].as_str().unwrap().to_string());
    assert_eq!(fixtures.len(), 16, "16 empty fixtures");
    fixtures
}

fn between<'a>(haystack: &'a str, start: &str, end: &str) -> Option<&'a str> {
    let from = haystack.find(start)? + start.len();
    let to = from + haystack[from..].find(end)?;
    Some(&haystack[from..to])
}

fn row_block<'a>(html: &'a str, id: &str) -> &'a str {
    let marker = format!("<tr data-row-id=\"{id}\"");
    let start = html
        .find(&marker)
        .unwrap_or_else(|| panic!("row {id} missing"));
    let end = html[start..].find("</tr>").unwrap() + start;
    &html[start..end]
}

fn row_ids(html: &str) -> Vec<String> {
    let mut ids = Vec::new();
    let mut rest = html;
    while let Some(found) = rest.find("data-row-id=\"") {
        let start = found + "data-row-id=\"".len();
        let end = rest[start..].find('"').unwrap() + start;
        ids.push(rest[start..end].to_string());
        rest = &rest[end..];
    }
    ids
}

fn fixture_family(fixture: &Value) -> &str {
    fixture["family"].as_str().unwrap()
}

fn expected_heading(family: &str, captured: &str) -> String {
    match family {
        "requirement" => "Business requirements".to_string(),
        "nfr" => "Non-functional requirements".to_string(),
        _ => captured.to_string(),
    }
}

fn expected_count_note(family: &str, captured: &str) -> String {
    match family {
        "note" => "2 notes".to_string(),
        _ => captured.to_string(),
    }
}

/// Expected header keys per family. Duplicated here on purpose: the replay
/// fixture records values, not the target header contract.
fn expected_columns(family: &str) -> &'static [&'static str] {
    match family {
        "milestone" => &[
            "id", "title", "status", "release", "created", "updated", "actions",
        ],
        "decision" => &[
            "id",
            "title",
            "status",
            "type",
            "superseded_by",
            "created",
            "updated",
            "actions",
        ],
        "requirement" | "nfr" => &[
            "id", "title", "status", "group", "created", "updated", "actions",
        ],
        "business-rule" => &[
            "id",
            "title",
            "status",
            "applicability",
            "created",
            "updated",
            "actions",
        ],
        "audit" => &[
            "id",
            "title",
            "status",
            "audit_type",
            "created",
            "updated",
            "actions",
        ],
        "ui-mockup" => &[
            "id", "title", "status", "task", "specs", "created", "updated", "actions",
        ],
        _ => &["id", "title", "status", "created", "updated", "actions"],
    }
}

fn expected_empty_count(family: &str) -> &'static str {
    match family {
        "requirement" | "nfr" => "0 groups · 0 requirements",
        "milestone" => "0 milestones",
        "release" => "0 releases",
        "decision" => "0 decisions",
        "glossary" => "0 glossary terms",
        "business-rule" => "0 business rules",
        "audit" => "0 audit records",
        "tech-spec" => "0 technical specifications",
        "ui-spec" => "0 UI specifications",
        "ui-mockup" => "0 UI mockups",
        "usecase" => "0 use cases",
        "testcase" => "0 test cases",
        "testreport" => "0 test reports",
        "note" => "0 notes",
        "task" => "No records",
        other => panic!("no empty count for {other}"),
    }
}

fn expected_empty_message(family: &str) -> &'static str {
    match family {
        "requirement" => "No business requirements in this project.",
        "nfr" => "No non-functional requirements in this project.",
        "milestone" => "No milestones in this project.",
        "release" => "No releases in this project.",
        "decision" => "No decisions in this project.",
        "glossary" => "No glossary terms in this project.",
        "business-rule" => "No business rules in this project.",
        "audit" => "No audit records in this project.",
        "tech-spec" => "No technical specifications in this project.",
        "ui-spec" => "No UI specifications in this project.",
        "ui-mockup" => "No UI mockups in this project.",
        "usecase" => "No use cases in this project.",
        "testcase" => "No test cases in this project.",
        "testreport" => "No test reports in this project.",
        "note" => "No notes in this project.",
        "task" => "No tasks in this project.",
        other => panic!("no empty message for {other}"),
    }
}

fn assert_shared_header(html: &str, family: &str) {
    assert!(
        html.contains("data-list-layout=\"shared\""),
        "{family} uses the shared layout"
    );
    let header = between(html, "<thead><tr>", "</tr></thead>").unwrap_or_else(|| {
        panic!("{family} table header");
    });
    let mut cursor = 0;
    for key in expected_columns(family) {
        let marker = format!("<th scope=\"col\" data-column=\"{key}\">");
        let position = header[cursor..]
            .find(&marker)
            .unwrap_or_else(|| panic!("{family} missing header {key}"));
        cursor += position + marker.len();
    }
}

fn normalized(value: &str, status: bool) -> String {
    if value.is_empty() || value == "(none)" {
        "<span class=\"muted\">(none)</span>".to_string()
    } else if status {
        format!("<span class=\"row-status tag\">{}</span>", escape(value))
    } else {
        escape(value)
    }
}

fn assert_record(html: &str, record: &Value) {
    let id = record["id"].as_str().unwrap();
    let row = row_block(html, id);
    let title = record["title"].as_str().unwrap();
    assert!(
        row.contains(&format!(">{}</a>", escape(title))),
        "{id} title"
    );
    assert!(
        row.contains(&format!("href=\"{}\"", record["href"].as_str().unwrap())),
        "{id} href"
    );
    let status = normalized(record["status"].as_str().unwrap(), true);
    assert!(row.contains(&status), "{id} status");
    let created = normalized(record["created"].as_str().unwrap(), false);
    assert!(
        row.contains(&format!("<td class=\"created-cell\">{created}</td>")),
        "{id} created"
    );
    let updated = normalized(record["updated"].as_str().unwrap(), false);
    assert!(
        row.contains(&format!("<td class=\"updated-cell\">{updated}</td>")),
        "{id} updated"
    );
    for label in record["labels"].as_array().unwrap() {
        assert!(
            row.contains(&format!(
                "<li class=\"tag\">{}</li>",
                escape(label.as_str().unwrap())
            )),
            "{id} label {label}"
        );
    }
    for reference in record["references"].as_array().unwrap() {
        assert!(
            row.contains(&format!("<li>{}</li>", escape(reference.as_str().unwrap()))),
            "{id} reference {reference}"
        );
    }
    for (key, value) in record["slots"].as_object().unwrap() {
        let expected = normalized(value.as_str().unwrap(), false);
        assert!(
            row.contains(&format!("<td class=\"slot-cell\">{expected}</td>")),
            "{id} slot {key}"
        );
    }
    assert!(row.contains(&format!("aria-label=\"View {id}\">View</a>")));
    assert!(row.contains(&format!("aria-label=\"Edit {id}\">Edit</a>")));
    let actions = row.find("class=\"actions-cell\"").expect("actions cell");
    assert!(actions > row.rfind("class=\"created-cell\"").unwrap_or(0));
}

/// ADR-035: Generic families omit every reference block; Typed and
/// HtmlMockup families keep it. Labels stay for every family.
fn assert_reference_visibility(html: &str, family: &str, fixture: &Value) {
    if family == "note" {
        assert!(
            !html.contains("row-refs"),
            "Generic family {family} renders no reference block in any row"
        );
        let row = row_block(html, "NOTE-001");
        assert!(
            row.contains("<li class=\"tag\">note</li>"),
            "Generic labels stay in the Title cell"
        );
        assert!(
            !row.contains("NOTE-REF-1"),
            "Generic row hides its references"
        );
        return;
    }
    for record in fixture["records"].as_array().unwrap() {
        let references = record["references"].as_array().unwrap();
        if references.is_empty() {
            continue;
        }
        let row = row_block(html, record["id"].as_str().unwrap());
        assert!(
            row.contains("<ul class=\"row-refs\">"),
            "{family} keeps the reference block"
        );
    }
    if family == "ui-mockup" {
        let row = row_block(html, "MOCK-002");
        assert!(
            row.contains("<ul class=\"row-refs\"><li>MOCK-REF-1</li></ul>"),
            "HtmlMockup family keeps the reference block"
        );
    }
}

fn assert_grouped(html: &str, fixture: &Value) {
    for group in fixture["grouped"].as_array().unwrap() {
        let key = group["group"].as_str().unwrap();
        let children = group["children"].as_array().unwrap();
        if key == "other" {
            for child in children {
                let id = child.as_str().unwrap();
                let row = row_block(html, id);
                assert!(
                    !row.contains("data-child-of"),
                    "pseudo-group `other` member {id} is top-level"
                );
            }
            continue;
        }
        let group_row = row_block(html, key);
        assert!(group_row.contains(&format!("data-row-id=\"{key}\"")));
        let group_at = html.find(&format!("data-row-id=\"{key}\"")).unwrap();
        for child in children {
            let id = child.as_str().unwrap();
            let child_row = row_block(html, id);
            assert!(
                child_row.contains(&format!("data-child-of=\"{key}\"")),
                "{id} belongs to {key}"
            );
            let child_at = html.find(&format!("data-row-id=\"{id}\"")).unwrap();
            assert!(group_at < child_at, "{key} precedes {id}");
        }
    }
    assert!(
        !html.contains("<article class=\"group\">"),
        "no group wrapper"
    );
    assert!(
        !html.contains("<ul class=\"req-list\">"),
        "no legacy row list"
    );
    assert!(
        !html.contains("<ul class=\"other-list\">"),
        "no legacy other list"
    );
    assert!(!html.contains("(not found)"), "no unresolved annotation");
}

fn assert_excluded_absent(html: &str, fixture: &Value, family: &str) {
    for excluded in fixture["excluded"].as_array().unwrap() {
        let id = excluded.as_str().unwrap();
        if family == "task" {
            assert!(
                !html.contains(&format!("data-task=\"{id}\"")),
                "archived {id} must stay off the board"
            );
        } else {
            assert!(
                !html.contains(&format!("data-row-id=\"{id}\"")),
                "archived {id} must stay absent"
            );
        }
        assert!(
            !html.contains(&format!("href=\"/doc/{id}\"")),
            "archived {id} must have no link"
        );
    }
}

fn board_details<'a>(html: &'a str, summary: &str) -> &'a str {
    let summary_at = html
        .find(&format!("<summary>{summary}"))
        .unwrap_or_else(|| panic!("board summary {summary} missing"));
    let details_at = html[..summary_at]
        .rfind("<details class=\"board-milestone\"")
        .unwrap();
    let end = html[details_at..].find("</details>").unwrap() + details_at;
    &html[details_at..end]
}

fn assert_board(fixture: &Value, html: &str) {
    assert!(
        !html.contains("data-list-layout=\"shared\""),
        "the task family is the board"
    );
    assert!(html.contains("<h1 class=\"h1\">Backlog</h1>"));
    let expected_order: Vec<String> = fixture["row_order"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap().to_string())
        .collect();
    let mut actual = Vec::new();
    let mut rest = html;
    while let Some(found) = rest.find("data-task=\"") {
        let start = found + "data-task=\"".len();
        let end = rest[start..].find('"').unwrap() + start;
        actual.push(rest[start..end].to_string());
        rest = &rest[end..];
    }
    assert_eq!(actual, expected_order, "board card order");
    for record in fixture["records"].as_array().unwrap() {
        let id = record["id"].as_str().unwrap();
        assert!(
            html.contains(&format!("data-task=\"{id}\"")),
            "card {id} present"
        );
        assert!(
            html.contains(&format!(
                "</span> {}</a>",
                escape(record["title"].as_str().unwrap())
            )),
            "card {id} title"
        );
        for label in record["labels"].as_array().unwrap() {
            assert!(
                html.contains(&escape(label.as_str().unwrap())),
                "card {id} label"
            );
        }
    }
    for group in fixture["grouped"].as_array().unwrap() {
        let key = group["group"].as_str().unwrap();
        if key == "unassigned" {
            let block = board_details(html, "Tasks without a milestone");
            let mut cursor = 0;
            for child in group["children"].as_array().unwrap() {
                let marker = format!("data-task=\"{}\"", child.as_str().unwrap());
                let position = block[cursor..]
                    .find(&marker)
                    .unwrap_or_else(|| panic!("unassigned card {} missing", child));
                cursor += position + marker.len();
            }
            continue;
        }
        let summary = format!("{key} ");
        let block = board_details(html, &summary);
        let mut cursor = 0;
        for child in group["children"].as_array().unwrap() {
            let marker = format!("data-task=\"{}\"", child.as_str().unwrap());
            let position = block[cursor..]
                .find(&marker)
                .unwrap_or_else(|| panic!("{key} card {} missing", child));
            cursor += position + marker.len();
        }
    }
    assert_excluded_absent(html, fixture, "task");
}

#[test]
fn every_populated_fixture_replays_against_the_shared_layout() {
    let root = fixture_root();
    let server = Server::start(&root.join("project"));
    for fixture in initial_fixtures() {
        assert_eq!(fixture["schema"], "papyrus.list-characterization/1");
        assert!(!fixture["empty"].as_bool().unwrap());
        let family = fixture_family(&fixture);
        let html = server.get(fixture["url"].as_str().unwrap());
        if family == "task" {
            assert_board(&fixture, &html);
            continue;
        }
        assert_shared_header(&html, family);
        let heading = expected_heading(family, fixture["heading"].as_str().unwrap());
        assert!(
            html.contains(&format!("<h1 class=\"h1\">{heading}</h1>")),
            "{family} heading"
        );
        let count = expected_count_note(family, fixture["count_note"].as_str().unwrap());
        assert!(
            html.contains(&format!("<p class=\"count\">{count}</p>")),
            "{family} count note"
        );
        let expected_order: Vec<String> = fixture["row_order"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_str().unwrap().to_string())
            .collect();
        assert_eq!(row_ids(&html), expected_order, "{family} row order");
        for record in fixture["records"].as_array().unwrap() {
            assert_record(&html, record);
        }
        assert_reference_visibility(&html, family, &fixture);
        if family == "requirement" || family == "nfr" {
            assert_grouped(&html, &fixture);
        }
        assert_excluded_absent(&html, &fixture, family);
    }
}

#[test]
fn every_empty_fixture_replays_the_shared_empty_state() {
    let root = fixture_root();
    let server = Server::start(&root.join("project-empty"));
    for fixture in empty_fixtures() {
        assert_eq!(fixture["schema"], "papyrus.list-characterization/1");
        assert!(fixture["empty"].as_bool().unwrap());
        let family = fixture_family(&fixture);
        let html = server.get(fixture["url"].as_str().unwrap());
        assert!(!html.contains("<table"), "{family} renders no table");
        assert!(!html.contains("data-row-id="), "{family} renders no rows");
        if family == "task" {
            assert!(html.contains("<h1 class=\"h1\">Backlog</h1>"));
            assert!(html.contains("<p class=\"count\">No records</p>"));
            assert!(html.contains("No tasks in this project."));
            assert!(html.contains("Records are created from the command line"));
            assert!(html.contains("<h2 class=\"h2\">Tasks</h2>"));
            continue;
        }
        let heading = expected_heading(family, fixture["heading"].as_str().unwrap());
        assert!(
            html.contains(&format!("<h1 class=\"h1\">{heading}</h1>")),
            "{family} heading"
        );
        let count = expected_empty_count(family);
        assert!(
            html.contains(&format!("<p class=\"count\">{count}</p>")),
            "{family} count note"
        );
        assert!(
            html.contains(expected_empty_message(family)),
            "{family} empty message"
        );
        assert!(
            html.contains(
                "Records are created from the command line; this browser does not create records."
            ),
            "{family} shared help"
        );
    }
}

/// TASK-292 C-5: the pinned `project-empty` fixture must contain the shared
/// configuration and no record files. TASK-241 removed three stray records to
/// restore this documented invariant; this guard keeps it from regressing.
#[test]
fn pinned_project_empty_fixture_contains_no_record_files() {
    let root = fixture_root().join("project-empty");
    let mut files = Vec::new();
    fn walk(directory: &Path, files: &mut Vec<PathBuf>) {
        let mut entries: Vec<PathBuf> = std::fs::read_dir(directory)
            .unwrap_or_else(|error| panic!("read {}: {error}", directory.display()))
            .flatten()
            .map(|entry| entry.path())
            .collect();
        entries.sort();
        for path in entries {
            if path.is_dir() {
                walk(&path, files);
            } else {
                files.push(path);
            }
        }
    }
    walk(&root, &mut files);
    let relative: Vec<String> = files
        .iter()
        .map(|path| {
            path.strip_prefix(&root)
                .expect("fixture file lives under project-empty")
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect();
    assert_eq!(
        relative,
        vec!["papyrus.config.yml".to_string()],
        "project-empty must hold only the configuration; stray record files break the pinned empty-fixture invariant"
    );
}
