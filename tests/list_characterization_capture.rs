//! TASK-141 / feature specification §6: characterization capture.
//!
//! This test is **not** part of the normal suite. It boots the browser over the
//! committed seed projects and writes `expected/<family>.json` from the
//! pre-change markup (document tables, `req-list` rows, board cards). It was
//! run once on the commit that produced the fixtures and is kept for audit
//! only; it must never be regenerated from the migrated shared-list code.
//!
//! Reproduction:
//!
//! ```text
//! PAPYRUS_CAPTURE_COMMIT=$(git rev-parse HEAD) \
//!   cargo test --test list_characterization_capture -- --ignored --nocapture
//! ```

use std::collections::BTreeSet;
use std::fs;
use std::io::{BufRead, BufReader};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use serde_json::{Value, json};

const SCHEMA: &str = "papyrus.list-characterization/1";

struct Family {
    key: &'static str,
    url: &'static str,
}

const FAMILIES: &[Family] = &[
    Family {
        key: "task",
        url: "/?type=task",
    },
    Family {
        key: "milestone",
        url: "/?type=milestone",
    },
    Family {
        key: "release",
        url: "/?type=release",
    },
    Family {
        key: "decision",
        url: "/?type=decision",
    },
    Family {
        key: "requirement",
        url: "/?type=req",
    },
    Family {
        key: "nfr",
        url: "/?type=nfr",
    },
    Family {
        key: "glossary",
        url: "/?type=glossary",
    },
    Family {
        key: "business-rule",
        url: "/?type=rule",
    },
    Family {
        key: "audit",
        url: "/?type=audit",
    },
    Family {
        key: "tech-spec",
        url: "/?type=tech-spec",
    },
    Family {
        key: "ui-spec",
        url: "/?type=ui-spec",
    },
    Family {
        key: "ui-mockup",
        url: "/?type=ui-mockup",
    },
    Family {
        key: "usecase",
        url: "/?type=use-case",
    },
    Family {
        key: "testcase",
        url: "/?type=test-case",
    },
    Family {
        key: "testreport",
        url: "/?type=test-report",
    },
    Family {
        key: "note",
        url: "/?type=note",
    },
];

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

fn unescape(value: &str) -> String {
    value
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&amp;", "&")
}

fn strip_tags(value: &str) -> String {
    let mut out = String::new();
    let mut depth = 0usize;
    for character in value.chars() {
        match character {
            '<' => depth += 1,
            '>' => depth = depth.saturating_sub(1),
            other if depth == 0 => out.push(other),
            _ => {}
        }
    }
    unescape(out.trim())
}

fn between<'a>(haystack: &'a str, start: &str, end: &str) -> Option<&'a str> {
    let from = haystack.find(start)? + start.len();
    let to = from + haystack[from..].find(end)?;
    Some(&haystack[from..to])
}

fn tag_text(html: &str, class: &str) -> Option<String> {
    let needle = format!("<span class=\"{class}\">");
    between(html, &needle, "</span>").map(strip_tags)
}

fn blocks<'a>(haystack: &'a str, start: &str, end: &str) -> Vec<&'a str> {
    let mut out = Vec::new();
    let mut rest = haystack;
    while let Some(from) = rest.find(start) {
        let body_start = from + start.len();
        let Some(len) = rest[body_start..].find(end) else {
            break;
        };
        out.push(&rest[body_start..body_start + len]);
        rest = &rest[body_start + len + end.len()..];
    }
    out
}

fn split_meta(
    parts: &[String],
) -> (
    Vec<String>,
    Vec<String>,
    Option<String>,
    Vec<String>,
    String,
) {
    let mut labels = Vec::new();
    let mut references = Vec::new();
    let mut status = String::new();
    let mut other = Vec::new();
    for part in parts {
        if let Some(value) = part.strip_prefix("labels") {
            labels = value.split_whitespace().map(str::to_string).collect();
        } else if let Some(value) = part.strip_prefix("references") {
            references = value.split_whitespace().map(str::to_string).collect();
        } else if let Some(value) = part.strip_prefix("status=") {
            status = value.to_string();
        } else if !part.starts_with("created ")
            && !part.starts_with("updated ")
            && !part.starts_with("superseded_by=")
        {
            other.push(part.clone());
        }
    }
    (labels, references, None, other, status)
}

fn meta_field(parts: &[String], prefix: &str) -> Option<String> {
    parts
        .iter()
        .find_map(|part| part.strip_prefix(prefix).map(str::trim).map(str::to_string))
}

fn record_from_li(li: &str) -> Value {
    let link = between(li, "<a href=\"", "</a>").unwrap_or_default();
    let href = between(link, "", "\"").unwrap_or_default().to_string();
    let id = tag_text(link, "req-id").unwrap_or_default();
    let title = link
        .rfind("</span> ")
        .map(|position| strip_tags(&link[position + "</span> ".len()..]))
        .unwrap_or_default();
    let status = tag_text(li, "row-status").unwrap_or_default();
    let mut created = tag_text(li, "row-created").unwrap_or_default();
    let mut updated = tag_text(li, "row-updated").unwrap_or_default();
    let mut parts: Vec<String> = blocks(li, "<span class=\"row-meta\">", "</span>")
        .into_iter()
        .flat_map(|meta| {
            strip_tags(meta)
                .split(" \u{b7} ")
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .collect();
    parts.retain(|part| !part.is_empty());
    if created.is_empty() {
        created = meta_field(&parts, "created ").unwrap_or_default();
    }
    if updated.is_empty() {
        updated = meta_field(&parts, "updated ").unwrap_or_default();
    }
    let (labels, references, _, mut other, meta_status) = split_meta(&parts);
    let status = if !status.is_empty() {
        status
    } else if !meta_status.is_empty() {
        meta_status
    } else if !other.is_empty() {
        other.remove(0)
    } else {
        String::new()
    };
    json!({
        "id": id,
        "title": title,
        "status": status,
        "created": created,
        "updated": updated,
        "labels": labels,
        "references": references,
        "slots": {},
        "href": href,
        "actions": [],
    })
}

fn heading(html: &str) -> String {
    between(html, "<h1 class=\"h1\">", "</h1>")
        .map(strip_tags)
        .unwrap_or_default()
}

fn count_note(html: &str) -> String {
    between(html, "<p class=\"count\">", "</p>")
        .map(strip_tags)
        .unwrap_or_default()
}

fn document_table(html: &str) -> Value {
    let mut columns: Vec<String> = blocks(html, "<th scope=\"col\">", "</th>")
        .into_iter()
        .map(strip_tags)
        .collect();
    if columns.is_empty() {
        columns = blocks(html, "<th>", "</th>")
            .into_iter()
            .map(strip_tags)
            .collect();
    }
    let body = between(html, "<tbody>", "</tbody>").unwrap_or_default();
    let mut records = Vec::new();
    let mut order = Vec::new();
    for row in blocks(body, "<tr>", "</tr>") {
        let cells = blocks(row, "<td", "</td>");
        let values: Vec<String> = cells
            .iter()
            .map(|cell| strip_tags(&cell[cell.find('>').map(|i| i + 1).unwrap_or(0)..]))
            .collect();
        if values.len() < 5 {
            continue;
        }
        let href = between(row, "<a href=\"", "\"")
            .unwrap_or_default()
            .to_string();
        let id = values[1].clone();
        order.push(id.clone());
        records.push(json!({
            "id": id,
            "title": values[2],
            "status": values[0],
            "created": values[3],
            "updated": values[4],
            "labels": [],
            "references": [],
            "slots": {},
            "href": href,
            "actions": [],
        }));
    }
    json!({ "columns": columns, "records": records, "order": order })
}

fn ul_list(html: &str) -> Value {
    if !html.contains("<ul class=\"req-list\">")
        && !html.contains("<ul class=\"other-list\">")
        && !html.contains("<section class=\"section\"")
    {
        return json!({ "columns": [], "records": [], "order": [], "grouped": [] });
    }
    let section = between(html, "<section class=\"section\"", "</section>").unwrap_or(html);
    let mut records = Vec::new();
    let mut order = Vec::new();
    let mut grouped = Vec::new();

    for article in blocks(section, "<article class=\"group\">", "</article>") {
        let group_id = tag_text(article, "group-id").unwrap_or_default();
        let title = between(article, "</span> ", "</a>")
            .or_else(|| between(article, "</span> ", "</h3>"))
            .map(strip_tags)
            .unwrap_or_default();
        let children: Vec<String> = blocks(article, "<li>", "</li>")
            .iter()
            .map(|li| tag_text(li, "req-id").unwrap_or_default())
            .collect();
        grouped.push(json!({ "group": group_id, "children": children }));
        records.push(json!({
            "id": group_id,
            "title": title,
            "status": "",
            "created": "",
            "updated": "",
            "labels": [],
            "references": [],
            "slots": {},
            "href": format!("/doc/{group_id}"),
            "actions": [],
        }));
        order.push(group_id.clone());
        for li in blocks(article, "<li>", "</li>") {
            let record = record_from_li(li);
            order.push(record["id"].as_str().unwrap_or_default().to_string());
            records.push(record);
        }
    }

    // Flat list rows (no `<article class="group">` wrapper).
    if grouped.is_empty() {
        for li in blocks(section, "<li>", "</li>") {
            let record = record_from_li(li);
            order.push(record["id"].as_str().unwrap_or_default().to_string());
            records.push(record);
        }
    }

    // "Other requirements": unresolved group references.
    if let Some(other) = between(section, "<ul class=\"other-list\">", "</ul>") {
        let children: Vec<String> = blocks(other, "<li>", "</li>")
            .iter()
            .map(|li| tag_text(li, "req-id").unwrap_or_default())
            .collect();
        if !children.is_empty() {
            grouped.push(json!({ "group": "other", "children": children }));
            for li in blocks(other, "<li>", "</li>") {
                let record = record_from_li(li);
                order.push(record["id"].as_str().unwrap_or_default().to_string());
                records.push(record);
            }
        }
    }

    json!({ "columns": [], "records": records, "order": order, "grouped": grouped })
}

fn board(html: &str) -> Value {
    let mut records = Vec::new();
    let mut order = Vec::new();
    let mut grouped = Vec::new();
    let mut release = String::new();
    let mut cursor = 0usize;
    loop {
        let next_release = html[cursor..]
            .find("<h2 class=\"h2 release-sep\">")
            .map(|i| cursor + i);
        let next_section = html[cursor..]
            .find("<details class=\"board-milestone\"")
            .map(|i| cursor + i);
        let Some(position) = next_release.into_iter().chain(next_section).min() else {
            break;
        };
        if next_release == Some(position) {
            let start = position + "<h2 class=\"h2 release-sep\">".len();
            let end = html[start..]
                .find("</h2>")
                .map(|i| start + i)
                .unwrap_or(start);
            release = strip_tags(&html[start..end]);
            cursor = end;
            continue;
        }
        let open = html[position..]
            .find('>')
            .map(|i| position + i + 1)
            .unwrap_or(position);
        let end = html[open..]
            .find("</details>")
            .map(|i| open + i)
            .unwrap_or(html.len());
        let section = &html[open..end];
        let summary = between(section, "<summary>", "</summary>")
            .map(strip_tags)
            .unwrap_or_default();
        let unassigned = summary.starts_with("Tasks without a milestone");
        let group_id = if unassigned {
            "unassigned".to_string()
        } else {
            summary.split(' ').next().unwrap_or_default().to_string()
        };
        let mut children = Vec::new();
        let mut position = 0usize;
        while let Some(found) =
            section[position..].find("<section class=\"board-column\" data-status=\"")
        {
            let column_start = position + found;
            let status_start =
                column_start + "<section class=\"board-column\" data-status=\"".len();
            let status_end = section[status_start..].find('"').unwrap() + status_start;
            let column_status = section[status_start..status_end].to_string();
            let column_end = section[status_start..]
                .find("</section>")
                .map(|i| status_start + i)
                .unwrap_or(section.len());
            let column = &section[status_start..column_end];
            for card in blocks(column, "<article class=\"board-card\"", "</article>") {
                let id = between(card, "data-task=\"", "\"")
                    .unwrap_or_default()
                    .to_string();
                let title = between(card, "</span> ", "</a>")
                    .map(strip_tags)
                    .unwrap_or_default();
                let assignee = between(card, "<p class=\"board-assignee\">", "</p>")
                    .map(|text| {
                        strip_tags(text)
                            .trim_start_matches("Assignee: ")
                            .to_string()
                    })
                    .unwrap_or_default();
                let labels: Vec<String> = blocks(card, "<span class=\"tag\">", "</span>")
                    .into_iter()
                    .map(strip_tags)
                    .collect();
                let milestone = if unassigned {
                    String::new()
                } else {
                    group_id.clone()
                };
                children.push(id.clone());
                order.push(id.clone());
                records.push(json!({
                    "id": id,
                    "title": title,
                    "status": column_status,
                    "created": "",
                    "updated": "",
                    "labels": labels,
                    "references": [],
                    "slots": { "milestone": milestone, "assignee": assignee },
                    "href": format!("/doc/{}", between(card, "<a href=\"/doc/", "\"").unwrap_or_default()),
                    "actions": [],
                }));
            }
            position = column_end;
        }
        grouped.push(json!({ "group": group_id, "release": release, "children": children }));
        cursor = end;
    }
    json!({ "columns": [], "records": records, "order": order, "grouped": grouped })
}

fn find_record_file(project: &Path, family: &str, id: &str) -> Option<PathBuf> {
    let candidates: Vec<String> = match family {
        "milestone" => vec![format!("backlog/milestones/{}.md", id)],
        "release" => vec![format!("backlog/releases/{}.md", id)],
        "task" => vec![format!("backlog/tasks/{}.md", id)],

        "requirement" => vec![format!("docs/requirements/business/{}.md", id)],
        "nfr" => vec![format!("docs/requirements/non-functional/{}.md", id)],
        "glossary" => vec![format!("docs/glossary/{}.md", id)],
        "business-rule" => vec![format!("docs/business-rules/{}.md", id)],
        "audit" => vec![format!("docs/audit-records/{}.md", id)],
        "tech-spec" => vec![format!("docs/technical/{}.md", id)],
        "ui-spec" => vec![format!("docs/specifications/ui/{}.md", id)],
        "ui-mockup" => vec![format!("docs/specifications/ui/mockups/{}.html", id)],
        "usecase" => vec![format!("docs/use-cases/{}.md", id)],
        "testcase" => vec![format!("docs/test-cases/{}.md", id)],
        "testreport" => vec![format!("docs/test-reports/{}.md", id)],
        "note" => vec![format!("docs/notes/{}.md", id)],
        _ => Vec::new(),
    };
    if family == "decision" {
        let directory = project.join("decisions");
        let prefix = format!("{id}-");
        if let Ok(entries) = fs::read_dir(&directory) {
            for entry in entries.flatten() {
                if entry.file_name().to_string_lossy().starts_with(&prefix) {
                    return Some(entry.path());
                }
            }
        }
        return None;
    }
    for candidate in candidates {
        let path = project.join(&candidate);
        if path.is_file() {
            return Some(path);
        }
    }
    None
}

fn read_text(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_default()
}

fn front_matter_value(text: &str, key: &str) -> Option<String> {
    for line in text.lines() {
        if let Some(value) = line.strip_prefix(&format!("{key}: ")) {
            let value = value.trim();
            if !value.is_empty() && value != "null" && !value.starts_with('[') {
                return Some(value.to_string());
            }
        }
    }
    None
}

fn front_matter_list(text: &str, key: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut lines = text.lines();
    while let Some(line) = lines.next() {
        if line.trim() == format!("{key}:") {
            for item in lines.by_ref() {
                let item = item.trim();
                if let Some(value) = item.strip_prefix("- ") {
                    items.push(value.trim_matches('"').to_string());
                } else {
                    break;
                }
            }
            break;
        }
        if let Some(value) = line.strip_prefix(&format!("{key}: ")) {
            let value = value.trim();
            if !value.is_empty() && value != "null" && !value.starts_with('[') {
                items.push(value.trim_matches('"').to_string());
            }
            break;
        }
    }
    items
}

fn body_section_value(text: &str, heading: &str) -> Option<String> {
    let mut lines = text.lines();
    while let Some(line) = lines.next() {
        if line.trim() == format!("## {heading}") {
            return lines.next().map(|value| value.trim().to_string());
        }
    }
    None
}

fn slot_values(project: &Path, family: &str, id: &str) -> serde_json::Map<String, Value> {
    let mut slots = serde_json::Map::new();
    let keys: &[&str] = match family {
        "milestone" => &["release"],
        "task" => &["milestone", "assignee"],
        "decision" => &["type"],
        "requirement" | "nfr" => &["group"],
        "audit" => &["audit_type"],
        "business-rule" => &["applicability"],
        "ui-mockup" => &["task", "specs"],
        _ => &[],
    };
    for key in keys {
        slots.insert((*key).to_string(), json!(""));
    }
    let Some(path) = find_record_file(project, family, id) else {
        return slots;
    };
    let text = read_text(&path);
    let mut set = |key: &str, value: Option<String>| {
        if let Some(value) = value
            && !value.is_empty()
        {
            slots.insert(key.to_string(), json!(value));
        }
    };
    match family {
        "milestone" => set("release", front_matter_value(&text, "release")),
        "task" => {
            set("milestone", front_matter_value(&text, "milestone"));
            let assignee = front_matter_list(&text, "assignee").join(", ");
            set("assignee", Some(assignee));
        }
        "decision" => {
            let kind = if id.starts_with("BDR") { "bdr" } else { "adr" };
            set("type", Some(kind.to_string()));
        }
        "requirement" | "nfr" => set(
            "group",
            body_section_value(&text, "Group")
                .map(|group| group.split(' ').next().unwrap_or_default().to_string()),
        ),
        "audit" => set("audit_type", front_matter_value(&text, "type")),
        "business-rule" => set("applicability", body_section_value(&text, "Applicability")),
        "ui-mockup" => {
            for line in text.lines() {
                let line = line.trim();
                if let Some(value) = line.strip_prefix("<!-- task: ") {
                    set("task", Some(value.trim_end_matches(" -->").to_string()));
                }
                if let Some(value) = line.strip_prefix("<!-- specs: ") {
                    set("specs", Some(value.trim_end_matches(" -->").to_string()));
                }
            }
        }
        _ => {}
    }
    slots
}

fn archive_dir(family: &str) -> &'static str {
    match family {
        "task" => "backlog/archive/tasks",
        "milestone" => "backlog/archive/milestones",
        "release" => "backlog/archive/releases",
        "decision" => "decisions/archived",
        "requirement" => "docs/requirements/business/archived",
        "nfr" => "docs/requirements/non-functional/archived",
        "glossary" => "docs/glossary/archived",
        "business-rule" => "docs/business-rules/archived",
        "audit" => "docs/audit-records/archived",
        "tech-spec" => "docs/technical/archived",
        "ui-spec" => "docs/specifications/ui/archived",
        "ui-mockup" => "docs/specifications/ui/mockups/archived",
        "usecase" => "docs/use-cases/archived",
        "testcase" => "docs/test-cases/archived",
        "testreport" => "docs/test-reports/archived",
        "note" => "docs/notes/archived",
        _ => "",
    }
}

fn archived_ids(project: &Path, family: &str) -> Vec<String> {
    fn walk(dir: &Path, inside_archive: bool, ids: &mut BTreeSet<String>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if path.is_dir() {
                walk(
                    &path,
                    inside_archive || name == "archive" || name == "archived",
                    ids,
                );
            } else if inside_archive && (name.ends_with(".md") || name.ends_with(".html")) {
                ids.insert(
                    name.trim_end_matches(".md")
                        .trim_end_matches(".html")
                        .to_string(),
                );
            }
        }
    }
    let mut ids = BTreeSet::new();
    let dir = project.join(archive_dir(family));
    if dir.is_dir() {
        walk(&dir, true, &mut ids);
    }
    ids.into_iter().collect()
}

fn decode(project: &Path, family: &str, url: &str, html: &str, commit: &str, empty: bool) -> Value {
    let decoded = if html.contains("<article class=\"board-card\"") || html.contains("board-column")
    {
        board(html)
    } else if html.contains("<table class=\"table\"") {
        document_table(html)
    } else {
        ul_list(html)
    };
    let shape = if html.contains("board-column") {
        "board"
    } else if html.contains("<table class=\"table\"") {
        "table"
    } else {
        "ul"
    };
    let mut records = decoded["records"].as_array().cloned().unwrap_or_default();
    for record in &mut records {
        let id = record["id"].as_str().unwrap_or_default().to_string();
        record["slots"] = Value::Object(slot_values(project, family, &id));
    }
    json!({
        "schema": SCHEMA,
        "family": family,
        "url": url,
        "captured_from_commit": commit,
        "heading": heading(html),
        "count_note": count_note(html),
        "row_order": decoded["order"],
        "baseline_columns": decoded["columns"],
        "baseline_row_shape": shape,
        "records": records,
        "grouped": decoded.get("grouped").cloned().unwrap_or(json!([])),
        "excluded": archived_ids(project, family),
        "empty": empty,
    })
}

#[test]
#[ignore = "characterization capture: runs once against the pre-change binary"]
fn capture_list_characterization_fixtures() {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/characterization/lists");
    let commit = std::env::var("PAPYRUS_CAPTURE_COMMIT").expect("PAPYRUS_CAPTURE_COMMIT");
    let out = root.join("expected");
    fs::create_dir_all(&out).unwrap();

    let populated = root.join("project");
    let server = Server::start(&populated);
    for family in FAMILIES {
        let html = server.get(family.url);
        let value = decode(&populated, family.key, family.url, &html, &commit, false);
        let path = out.join(format!("{}.json", family.key));
        fs::write(&path, serde_json::to_string_pretty(&value).unwrap() + "\n").unwrap();
        println!("wrote {}", path.display());
    }
    drop(server);

    let empty_project = root.join("project-empty");
    let server = Server::start(&empty_project);
    for family in FAMILIES {
        let html = server.get(family.url);
        let value = decode(&empty_project, family.key, family.url, &html, &commit, true);
        let path = out.join(format!("{}-empty.json", family.key));
        fs::write(&path, serde_json::to_string_pretty(&value).unwrap() + "\n").unwrap();
        println!("wrote {}", path.display());
    }
}
