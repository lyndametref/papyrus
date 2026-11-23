//! TASK-142: every enabled artifact family renders through the shared list
//! layout, with the fixed header sequence and its declared family slots.
//!
//! The task descriptor is the REQ-006.013 board and is asserted there instead.
//! Archived records are absent from every list and from the board's cards.

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use papyrus::browser::routes::{self, BrowserRoutes};
use papyrus::commands::load_effective_config;
use papyrus::registry::{ArtifactRegistry, TypeDescriptor};
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

fn between<'a>(haystack: &'a str, start: &str, end: &str) -> Option<&'a str> {
    let from = haystack.find(start)? + start.len();
    let to = from + haystack[from..].find(end)?;
    Some(&haystack[from..to])
}

fn registry() -> (ArtifactRegistry, BrowserRoutes) {
    let project = fixture_root().join("project");
    let cfg = load_effective_config(&project.join("papyrus.config.yml"), &project).unwrap();
    let registry = ArtifactRegistry::build(&cfg).unwrap();
    let routes = BrowserRoutes::build(&registry);
    (registry, routes)
}

/// One representative list route per presentation, in descriptor order.
fn representative_routes(
    registry: &ArtifactRegistry,
    routes: &BrowserRoutes,
) -> BTreeMap<String, (String, String)> {
    let mut out = BTreeMap::new();
    for descriptor in registry
        .descriptors()
        .filter(|descriptor| descriptor.enabled)
    {
        let presentation = routes.list_slug_for(&descriptor.slug);
        let url = format!("/?type={presentation}");
        out.entry(url)
            .or_insert_with(|| (descriptor.slug.clone(), presentation));
    }
    out
}

fn expected_columns(descriptor: &TypeDescriptor) -> Vec<String> {
    let mut keys = vec!["id".to_string(), "title".to_string(), "status".to_string()];
    if descriptor.category == papyrus::registry::ArtifactCategory::Document {
        for slot in &descriptor.slots {
            keys.push(slot.key.clone());
        }
    } else {
        for (key, _) in routes::list_slots(routes::static_slug(descriptor)) {
            keys.push((*key).to_string());
        }
    }
    keys.push("created".to_string());
    keys.push("updated".to_string());
    keys.push("actions".to_string());
    keys
}

fn assert_shared_table(html: &str, descriptor: &TypeDescriptor, presentation: &str) {
    assert!(
        html.contains("<table class=\"table list list--shared\" data-list-layout=\"shared\""),
        "{presentation} shared table"
    );
    let header = between(html, "<thead><tr>", "</tr></thead>")
        .unwrap_or_else(|| panic!("{presentation} header"));
    let mut actual = Vec::new();
    let mut rest = header;
    while let Some(found) = rest.find("data-column=\"") {
        let start = found + "data-column=\"".len();
        let end = rest[start..].find('"').unwrap() + start;
        actual.push(rest[start..end].to_string());
        rest = &rest[end..];
    }
    assert_eq!(
        actual,
        expected_columns(descriptor),
        "{presentation} columns"
    );
    for key in expected_columns(descriptor) {
        assert!(
            header.contains(&format!("<th scope=\"col\" data-column=\"{key}\">")),
            "{presentation} header {key} is a column header"
        );
    }
}

#[test]
fn every_enabled_family_renders_the_shared_layout_with_its_declared_slots() {
    let (registry, routes) = registry();
    let listed: Vec<TypeDescriptor> = registry
        .descriptors()
        .filter(|descriptor| descriptor.enabled)
        .cloned()
        .collect();
    let representatives = representative_routes(&registry, &routes);
    assert!(
        representatives.len() >= 14,
        "one representative route per presentation: {representatives:?}"
    );
    assert!(representatives.contains_key("/?type=note"));
    assert!(representatives.contains_key("/?type=decision"));
    assert!(representatives.contains_key("/?type=milestone"));
    assert!(representatives.contains_key("/?type=release"));

    let server = Server::start(&fixture_root().join("project"));
    for (url, (descriptor, presentation)) in &representatives {
        let html = server.get(url);
        if presentation == "task" {
            assert_eq!(
                descriptor, "task",
                "the only board presentation is the task family"
            );
            assert!(!html.contains("data-list-layout=\"shared\""));
            assert!(html.contains("<h1 class=\"h1\">Backlog</h1>"));
            assert!(html.contains("data-task=\"TASK-001\""));
            assert!(!html.contains("<table"));
            continue;
        }
        let descriptor = registry
            .descriptor(descriptor)
            .expect("representative descriptor");
        assert_shared_table(&html, descriptor, presentation);
    }
    assert_eq!(
        representatives.len(),
        listed
            .iter()
            .map(|descriptor| routes.list_slug_for(&descriptor.slug))
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        "every enabled descriptor resolves to a representative presentation"
    );
}

#[test]
fn decision_aliases_share_the_decisions_list() {
    let (registry, _) = registry();
    let server = Server::start(&fixture_root().join("project"));
    let mut baseline_rows = None;
    for url in ["/?type=decision", "/?type=adr", "/?type=bdr"] {
        let html = server.get(url);
        let descriptor = registry
            .descriptors()
            .find(|descriptor| descriptor.slug == "adr")
            .expect("decision descriptor");
        assert_shared_table(&html, descriptor, "decision");
        assert!(html.contains("<h1 class=\"h1\">Decisions</h1>"), "{url}");
        assert!(
            html.contains("<th scope=\"col\" data-column=\"type\">Type</th>"),
            "{url}"
        );
        assert!(
            html.contains("<th scope=\"col\" data-column=\"superseded_by\">Superseded by</th>"),
            "{url}"
        );
        assert!(html.contains("<td class=\"slot-cell\">adr</td>"), "{url}");
        assert!(html.contains("<td class=\"slot-cell\">bdr</td>"), "{url}");
        assert!(
            html.contains("<td class=\"slot-cell\"><span class=\"muted\">(none)</span></td>"),
            "{url} superseded slot is empty"
        );
        let mut ids = Vec::new();
        let mut rest = html.as_str();
        while let Some(found) = rest.find("data-row-id=\"") {
            let start = found + "data-row-id=\"".len();
            let end = rest[start..].find('"').unwrap() + start;
            ids.push(rest[start..end].to_string());
            rest = &rest[end..];
        }
        if let Some(expected) = &baseline_rows {
            assert_eq!(&ids, expected, "{url} renders the same rows");
        } else {
            baseline_rows = Some(ids);
        }
    }
}

fn expected_fixtures() -> Vec<Value> {
    let mut fixtures = Vec::new();
    for entry in std::fs::read_dir(fixture_root().join("expected")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|value| value.to_str()) == Some("json") {
            fixtures.push(
                serde_json::from_str::<Value>(&std::fs::read_to_string(&path).unwrap()).unwrap(),
            );
        }
    }
    assert_eq!(fixtures.len(), 32, "all characterization fixtures");
    fixtures
}

#[test]
fn archived_records_are_absent_from_every_list_and_the_board() {
    let server = Server::start(&fixture_root().join("project"));
    let empty = Server::start(&fixture_root().join("project-empty"));
    for fixture in expected_fixtures() {
        let family = fixture["family"].as_str().unwrap();
        let html = if fixture["empty"].as_bool().unwrap() {
            empty.get(fixture["url"].as_str().unwrap())
        } else {
            server.get(fixture["url"].as_str().unwrap())
        };
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
                    "archived {id} must stay absent from {family}"
                );
            }
            assert!(
                !html.contains(&format!("href=\"/doc/{id}\"")),
                "archived {id} must have no link in {family}"
            );
        }
    }
}

// TC-055 / TC-057: a user-defined family keeps the shared layout, labels, and
// row actions; its references default to visible.
#[test]
fn user_defined_family_keeps_the_shared_layout_with_visible_references() {
    let (registry, _) = registry();
    let note = registry.descriptor("note").expect("note descriptor");
    let server = Server::start(&fixture_root().join("project"));
    let html = server.get("/?type=note");
    assert_shared_table(&html, note, "note");
    assert!(html.contains("<li class=\"tag\">note</li>"), "labels stay");
    assert!(
        html.contains("row-refs") && html.contains("NOTE-REF-1"),
        "a user-defined type defaults to visible references"
    );
    let start = html.find("data-row-id=\"NOTE-001\"").expect("NOTE-001 row");
    let row = &html[start..];
    let row = &row[..row.find("</tr>").unwrap()];
    assert!(row.contains("aria-label=\"View NOTE-001\">View</a>"));
    assert!(row.contains("aria-label=\"Edit NOTE-001\">Edit</a>"));
}
