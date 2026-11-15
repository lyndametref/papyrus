//! TASK-141: replay the reference family's characterization fixture against the
//! shared list layout.
//!
//! Record set, row order, per-row values, heading, count note, and archived
//! exclusions must match the pre-change capture. Header sequence and row
//! actions are asserted separately because the shared layout is the intended
//! change (feature specification §6).

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

#[test]
fn reference_family_replays_its_characterization_fixture() {
    let root = fixture_root();
    let fixture: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root.join("expected/milestone.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(fixture["schema"], "papyrus.list-characterization/1");
    let server = Server::start(&root.join("project"));
    let html = server.get(fixture["url"].as_str().unwrap());

    // Heading and count note are unchanged by the migration.
    assert!(html.contains(&format!(
        "<h1 class=\"h1\">{}</h1>",
        fixture["heading"].as_str().unwrap()
    )));
    assert!(html.contains(&format!(
        "<p class=\"count\">{}</p>",
        fixture["count_note"].as_str().unwrap()
    )));

    // The shared layout carries the stable hooks and the amended header order.
    assert!(html.contains("data-list-layout=\"shared\""));
    let header = between(&html, "<thead><tr>", "</tr></thead>").expect("table header");
    let mut cursor = 0;
    for key in [
        "id", "title", "status", "release", "created", "updated", "actions",
    ] {
        let marker = format!("data-column=\"{key}\"");
        let position = header[cursor..]
            .find(&marker)
            .unwrap_or_else(|| panic!("missing header {key}"));
        cursor += position + marker.len();
    }

    // Row set and relative order match the capture.
    let expected_order: Vec<String> = fixture["row_order"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap().to_string())
        .collect();
    let mut actual_order = Vec::new();
    let mut rest: &str = &html;
    while let Some(found) = rest.find("data-row-id=\"") {
        let start = found + "data-row-id=\"".len();
        let end = rest[start..].find('"').unwrap() + start;
        actual_order.push(rest[start..end].to_string());
        rest = &rest[end..];
    }
    assert_eq!(actual_order, expected_order, "row order and set");

    for record in fixture["records"].as_array().unwrap() {
        let id = record["id"].as_str().unwrap();
        let row = row_block(&html, id);
        assert!(
            row.contains(&format!(">{}</a>", record["title"].as_str().unwrap())),
            "{id} title"
        );
        let status = record["status"].as_str().unwrap();
        assert!(
            row.contains(&format!("<span class=\"row-status tag\">{status}</span>")),
            "{id} status"
        );
        let created = record["created"].as_str().unwrap();
        assert!(
            row.contains(&format!("<td class=\"created-cell\">{created}</td>")),
            "{id} created"
        );
        let updated = record["updated"].as_str().unwrap();
        assert!(
            row.contains(&format!("<td class=\"updated-cell\">{updated}</td>")),
            "{id} updated"
        );
        for label in record["labels"].as_array().unwrap() {
            assert!(
                row.contains(&format!(
                    "<li class=\"tag\">{}</li>",
                    label.as_str().unwrap()
                )),
                "{id} label {label}"
            );
        }
        for reference in record["references"].as_array().unwrap() {
            assert!(
                row.contains(&format!("<li>{}</li>", reference.as_str().unwrap())),
                "{id} reference {reference}"
            );
        }
        for (key, value) in record["slots"].as_object().unwrap() {
            let value = value.as_str().unwrap();
            let expected = if value.is_empty() {
                "<span class=\"muted\">(none)</span>".to_string()
            } else {
                value.to_string()
            };
            assert!(
                row.contains(&format!("<td class=\"slot-cell\">{expected}</td>")),
                "{id} slot {key}"
            );
        }
        // The layout change: View and Edit with record-specific names, last cell.
        assert!(row.contains(&format!("aria-label=\"View {id}\">View</a>")));
        assert!(row.contains(&format!("aria-label=\"Edit {id}\">Edit</a>")));
        let actions = row.find("class=\"actions-cell\"").expect("actions cell");
        assert!(actions > row.rfind("class=\"slot-cell\"").unwrap_or(0));
    }

    // Archived records never reach the list.
    for excluded in fixture["excluded"].as_array().unwrap() {
        let id = excluded.as_str().unwrap();
        assert!(
            !html.contains(&format!("data-row-id=\"{id}\"")),
            "archived {id} must stay absent"
        );
        assert!(
            !html.contains(&format!("href=\"/doc/{id}\"")),
            "archived {id} must have no link"
        );
    }
}
