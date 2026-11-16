//! TASK-146: Mermaid diagram figures, the PlantUML source block, both input
//! caps, the two CSP literals, the conditional bundle, and the client contract.
//!
//! The contract is `docs/specifications/browser-math-and-diagram-rendering.md`
//! Part II (§13–§21). The fixture starts as a copy of the characterization
//! project and replaces the prose bodies with diagram, math, and hostile
//! content.

use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

const CSP: &str = "default-src 'none'; style-src 'self'; script-src 'self'; img-src 'self'; font-src 'self'; form-action 'self'; base-uri 'none'; frame-ancestors 'none'";
const CSP_DIAGRAM: &str = "default-src 'none'; style-src 'self' 'unsafe-inline'; script-src 'self'; img-src 'self'; font-src 'self'; form-action 'self'; base-uri 'none'; frame-ancestors 'none'";

const BUNDLE: &str = "/assets/mermaid/mermaid.min.js";
const LIMIT_NOTE: &str =
    "Diagram not rendered: source exceeds the 100,000-byte or 1,000-line limit.";
const PLANTUML_NOTE: &str = "PlantUML diagrams are not supported in this release; showing source.";
const REJECTED_NOTE: &str = "Diagram not rendered: Mermaid rejected the source.";
const MISSING_NOTE: &str = "Diagram not rendered: the diagram script did not load.";

const FLOWCHART: &str = "flowchart TD\n  A --> B";
const GRAPH: &str = "graph LR\n  C --> D";
const SEQUENCE: &str = "sequenceDiagram\n  A->>B: Hello";
const CLASS: &str = "classDiagram\n  class Animal";
const STATE: &str = "stateDiagram-v2\n  [*] --> Idle";
const ER: &str = "erDiagram\n  CUSTOMER ||--o{ ORDER : places";
const GANTT: &str = "gantt\n  title Plan\n  section S\n  Task :a1, 2026-01-01, 1d";
const PIE: &str = "pie\n  title Share\n  \"A\" : 1";
const JOURNEY: &str = "journey\n  title Day\n  section S\n  Task: 5: Me";
const GIT_GRAPH: &str = "gitGraph\n  commit";

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_papyrus"))
}

fn fixture_source() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/characterization/lists/project")
}

struct TempProject {
    path: PathBuf,
}

impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn temp_project(name: &str) -> TempProject {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(".tmp");
    path.push(format!("browser-diagram-{}-{}", name, std::process::id()));
    let _ = fs::remove_dir_all(&path);
    copy_dir(&fixture_source(), &path);
    TempProject { path }
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), &target).unwrap();
        }
    }
}

/// Replace a record's sections after the front matter and first heading,
/// keeping the front matter and any `# ID Title` heading intact.
fn set_body(dir: &Path, relative: &str, body: &str) {
    let path = dir.join(relative);
    let text = fs::read_to_string(&path).unwrap();
    let front_end = match text.strip_prefix("---\n") {
        Some(rest) => rest
            .find("\n---\n")
            .map(|offset| 4 + offset + 5)
            .expect("front matter terminator"),
        None => 0,
    };
    let rest = &text[front_end..];
    let cut = rest
        .find("\n## ")
        .map(|offset| front_end + offset + 1)
        .unwrap_or(text.len());
    let prefix = text[..cut].trim_end();
    fs::write(&path, format!("{prefix}\n\n{body}\n")).unwrap();
}

fn note(dir: &Path, id: &str, title: &str, body: &str) {
    fs::write(
        dir.join(format!("docs/notes/{id}.md")),
        format!(
            "---
id: {id}
title: \"{title}\"
created_date: '2026-11-16T00:00:00Z'
updated_date: '2026-11-16T00:00:00Z'
labels: []
references: []
status: active
---
# {id} {title}

{body}
"
        ),
    )
    .unwrap();
}

fn mermaid(source: &str) -> String {
    format!("```mermaid\n{source}\n```")
}

const REQ_BODY: &str = "## User Story
As a reader I want $x$ math and diagrams.

## Description
```mermaid
flowchart TD
  A --> B
```

```mermaid
graph LR
  C --> D
```

```mermaid
sequenceDiagram
  A->>B: Hello
```

```mermaid
classDiagram
  class Animal
```

```mermaid
stateDiagram-v2
  [*] --> Idle
```

```mermaid
erDiagram
  CUSTOMER ||--o{ ORDER : places
```

```mermaid
gantt
  title Plan
  section S
  Task :a1, 2026-01-01, 1d
```

```mermaid
pie
  title Share
  \"A\" : 1
```

```mermaid
journey
  title Day
  section S
  Task: 5: Me
```

```mermaid
gitGraph
  commit
```";

fn project(name: &str) -> TempProject {
    let project = temp_project(name);
    let dir = &project.path;

    set_body(dir, "docs/requirements/business/REQ-001.001.md", REQ_BODY);
    set_body(
        dir,
        "docs/requirements/business/REQ-001.002.md",
        "## User Story
As a reader I want a story diagram.

```mermaid
flowchart TD
  Story --> First
```

## Group
REQ-001 Requirement Group One

## Description
```mermaid
graph LR
  Desc --> One
```

```mermaid
sequenceDiagram
  A->>B: Hi
```",
    );
    note(
        dir,
        "NOTE-010",
        "PlantUML Note",
        "## Description

```plantuml
@startuml
A -> B
@enduml
```

```puml
C -> D
```",
    );
    note(
        dir,
        "NOTE-011",
        "Literal Fences",
        "## Description

```uml
A -> B
```

```dot
digraph { A -> B }
```

```graphviz
graph { A -- B }
```

```mmd
graph TD
  M --> N
```

```rust
fn main() {}
```",
    );
    note(
        dir,
        "NOTE-012",
        "Unknown and Malformed",
        &format!(
            "## Description\n\n{}\n\n{}\n\n{}",
            // A type the pinned bundle cannot detect: the failure-path fixture.
            mermaid("notARealDiagramType\n  A --> B"),
            // Not promised either, but 11.17.2 renders it (spec §18).
            mermaid("flowchart-elk TD\n  C --> D"),
            mermaid("flowchart TD\n  A --> B\n  A -->")
        ),
    );
    let over_bytes = "a".repeat(100_001);
    note(
        dir,
        "NOTE-013",
        "Over Byte",
        &format!("## Description\n\n{}", mermaid(&over_bytes)),
    );
    let over_lines = "l\n".repeat(1_000) + "l";
    note(
        dir,
        "NOTE-014",
        "Over Line",
        &format!("## Description\n\n{}", mermaid(&over_lines)),
    );
    note(
        dir,
        "NOTE-015",
        "Isolation",
        &format!(
            "## Description\n\n{}\n\n{}",
            mermaid(&over_bytes),
            mermaid("flowchart TD\n  Good --> Figure")
        ),
    );
    note(
        dir,
        "NOTE-016",
        "Hostile Diagram",
        "## Description

```mermaid
flowchart TD
  A[\"<img src=x onerror=alert(1)>\"] --> B
  classDef evil fill:#fff,background:url(javascript:alert(1))
  click B \"javascript:alert(1)\"
```",
    );
    note(
        dir,
        "NOTE-017",
        "Math Only",
        "## Description\n\nInline $a^2$ only, no diagram.",
    );
    note(
        dir,
        "NOTE-018",
        "PlantUML Only",
        "## Description\n\n```plantuml\nA -> B\n```",
    );
    note(
        dir,
        "NOTE-019",
        "No Prose Marks",
        "## Description\n\nPlain prose without math or diagrams.",
    );

    project
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
            let stderr = project.join(format!(".stderr-{port}"));
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
        body(&self.get_raw(path))
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn body(raw: &str) -> String {
    raw.split_once("\r\n\r\n")
        .map(|(_, body)| body.to_string())
        .unwrap_or_default()
}

fn status(raw: &str) -> &str {
    raw.lines().next().unwrap_or_default()
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

fn run(project: &Path, args: &[&str]) -> std::process::Output {
    let mut full = vec!["--dir", project.to_str().unwrap()];
    full.extend_from_slice(args);
    bin().args(&full).output().unwrap()
}

fn run_ok(project: &Path, args: &[&str]) -> String {
    let out = run(project, args);
    assert!(
        out.status.success(),
        "command failed: {args:?}\nstderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn escape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            other => out.push(other),
        }
    }
    out
}

fn figure(index: usize, source: &str) -> String {
    format!(
        "<figure class=\"diagram bleed\" data-diagram=\"mermaid\"><div class=\"diagram-scroll\" role=\"region\" aria-label=\"Diagram {index}\" tabindex=\"0\"><pre>{}</pre></div></figure>",
        escape(source)
    )
}

fn over_limit_figure(index: usize, source: &str) -> String {
    format!(
        "<figure class=\"diagram bleed\" data-diagram=\"mermaid\" data-render-error=\"diagram\"><div class=\"diagram-scroll\" role=\"region\" aria-label=\"Diagram {index}\" tabindex=\"0\"><pre>{}</pre></div><figcaption class=\"render-note\">{LIMIT_NOTE}</figcaption></figure>",
        escape(source)
    )
}

fn plantuml_figure(source: &str) -> String {
    format!(
        "<figure class=\"diagram diagram--unsupported\" data-diagram=\"plantuml\"><pre>{}</pre><figcaption class=\"render-note\">{PLANTUML_NOTE}</figcaption></figure>",
        escape(source)
    )
}

fn diagram_labels(html: &str) -> Vec<String> {
    let needle = "class=\"diagram-scroll\" role=\"region\" aria-label=\"";
    let mut labels = Vec::new();
    let mut rest = html;
    while let Some(start) = rest.find(needle) {
        let after = &rest[start + needle.len()..];
        let Some(end) = after.find('"') else {
            panic!("unterminated aria-label");
        };
        labels.push(after[..end].to_string());
        rest = &after[end..];
    }
    labels
}

fn assert_external_scripts_only(html: &str) {
    for (index, _) in html.match_indices("<script") {
        assert!(
            html[index + "<script".len()..].starts_with(" src=\""),
            "inline script element in served HTML"
        );
    }
}

fn script_src(csp: &str) -> &str {
    let after = csp.split_once("script-src ").unwrap().1;
    after.split(';').next().unwrap()
}

#[test]
fn promised_types_render_as_figures_with_source() {
    let project = project("types");
    let server = Server::start(&project.path);
    let html = server.get("/doc/REQ-001.001");

    for (index, source) in [
        FLOWCHART, GRAPH, SEQUENCE, CLASS, STATE, ER, GANTT, PIE, JOURNEY, GIT_GRAPH,
    ]
    .into_iter()
    .enumerate()
    {
        assert!(
            html.contains(&figure(index + 1, source)),
            "missing exact figure for source: {source}"
        );
    }
    assert_eq!(
        html.matches("<figure class=\"diagram bleed\"").count(),
        10,
        "one figure per promised type"
    );
    assert_eq!(
        html.matches("data-diagram=\"mermaid\"").count(),
        10,
        "ten mermaid figures"
    );
    assert!(
        html.contains("<span class=\"math\" data-tex=\"x\">x</span>"),
        "the same page still renders math"
    );
    assert_external_scripts_only(&html);
}

#[test]
fn renderable_figure_is_a_focusable_labelled_scroll_region() {
    let project = project("a11y");
    let server = Server::start(&project.path);
    let html = server.get("/doc/REQ-001.001");

    assert!(
        html.contains(
            "<div class=\"diagram-scroll\" role=\"region\" aria-label=\"Diagram 1\" tabindex=\"0\">"
        ),
        "the first scroll region is a labelled, focusable region"
    );
    assert_eq!(
        html.matches("class=\"diagram-scroll\" role=\"region\"")
            .count(),
        10,
        "every mermaid figure carries the region role"
    );
    assert_eq!(
        html.matches("aria-label=\"Diagram ").count(),
        10,
        "every mermaid figure carries a deterministic name"
    );
    assert!(html.contains("aria-label=\"Diagram 10\""));
    assert_eq!(
        html.matches("tabindex=\"0\"").count(),
        10,
        "every mermaid figure is keyboard reachable"
    );
    assert!(
        !html.contains("class=\"diagram-scroll\"><pre"),
        "no unlabelled scroll container remains"
    );
}

#[test]
fn diagram_names_are_page_unique_across_prose_fields() {
    let project = project("names");
    let server = Server::start(&project.path);
    let path = "/doc/REQ-001.002";

    let first = server.get(path);
    let labels = diagram_labels(&first);
    assert_eq!(
        labels,
        vec!["Diagram 1", "Diagram 2", "Diagram 3"],
        "numbering is continuous across the story and description fields"
    );
    let unique: std::collections::BTreeSet<&String> = labels.iter().collect();
    assert_eq!(unique.len(), labels.len(), "every name is page-unique");
    assert_eq!(
        first
            .matches("class=\"diagram-scroll\" role=\"region\"")
            .count(),
        3
    );
    assert_eq!(first.matches("tabindex=\"0\"").count(), 3);

    let second = server.get(path);
    assert_eq!(
        diagram_labels(&second),
        labels,
        "the names are stable across renders"
    );
}

#[test]
fn plantuml_and_puml_render_the_unsupported_figure() {
    let project = project("plantuml");
    let server = Server::start(&project.path);
    let html = server.get("/doc/NOTE-010");

    assert!(html.contains(&plantuml_figure("@startuml\nA -> B\n@enduml")));
    assert!(html.contains(&plantuml_figure("C -> D")));
    assert_eq!(html.matches("data-diagram=\"plantuml\"").count(), 2);
    assert!(!html.contains("diagram bleed"), "no bleed class");
    assert!(!html.contains("diagram-scroll"), "no scroll container");
    assert!(!html.contains(BUNDLE), "plantuml never links the bundle");
    assert_eq!(
        header_value(&server.get_raw("/doc/NOTE-010"), "Content-Security-Policy").as_deref(),
        Some(CSP)
    );
    assert_external_scripts_only(&html);
}

#[test]
fn other_fences_stay_literal() {
    let project = project("literal");
    let server = Server::start(&project.path);
    let html = server.get("/doc/NOTE-011");

    for needle in [
        "```uml",
        "```dot",
        "```graphviz",
        "```mmd",
        "graph TD",
        "M --&gt; N",
        "```rust",
        "fn main() {}",
    ] {
        assert!(
            html.contains(needle),
            "missing literal fence text: {needle}"
        );
    }
    assert!(
        !html.contains("<figure"),
        "no literal fence becomes a figure"
    );
    assert!(!html.contains(BUNDLE));
    assert!(
        html.contains("```uml\nA -&gt; B\n```"),
        "fence lines stay literal"
    );
}

#[test]
fn unknown_type_and_malformed_sources_keep_the_client_failure_path() {
    let project = project("unknown");
    let server = Server::start(&project.path);
    let raw = server.get_raw("/doc/NOTE-012");
    assert!(status(&raw).contains(" 200"), "{}", status(&raw));
    let html = body(&raw);

    let unknown = "notARealDiagramType\n  A --> B";
    assert!(
        html.contains(&figure(1, unknown)),
        "the unknown type keeps its full source"
    );
    assert!(
        html.contains(&figure(2, "flowchart-elk TD\n  C --> D")),
        "flowchart-elk is served as a best-effort figure, not a failure"
    );
    assert!(html.contains(&figure(3, "flowchart TD\n  A --> B\n  A -->")));
    assert_eq!(
        html.matches("data-render-error=\"diagram\"").count(),
        0,
        "the server never parses or pre-rejects a type"
    );
    assert!(html.contains(BUNDLE));

    // `notARealDiagramType` reaches the failure path in the client, not on the
    // server: the marker, the note, and the preserved source are client-side.
    let js = include_str!("../src/browser/app.js");
    assert!(
        js.contains("figure.setAttribute(\"data-render-error\", \"diagram\")"),
        "the client sets the failure marker"
    );
    assert!(
        js.contains(REJECTED_NOTE),
        "the client appends the rejection note"
    );
    assert!(
        js.contains("var pre = figure.querySelector(\"pre\");"),
        "the client reads the preserved pre source"
    );
}

#[test]
fn over_limit_fixtures_keep_source_and_take_the_limit_note() {
    let project = project("caps");
    let server = Server::start(&project.path);

    let over_bytes = "a".repeat(100_001);
    let raw = server.get_raw("/doc/NOTE-013");
    let html = body(&raw);
    assert!(html.contains(&over_limit_figure(1, &over_bytes)));
    assert_eq!(html.matches(LIMIT_NOTE).count(), 1, "one limit note");
    assert!(!html.contains(BUNDLE), "over-limit page ships no bundle");
    assert_eq!(
        header_value(&raw, "Content-Security-Policy").as_deref(),
        Some(CSP),
        "over-limit-only page keeps the strict CSP"
    );

    let over_lines = "l\n".repeat(1_000) + "l";
    let raw = server.get_raw("/doc/NOTE-014");
    let html = body(&raw);
    assert!(html.contains(&over_limit_figure(1, &over_lines)));
    assert_eq!(html.matches("data-render-error=\"diagram\"").count(), 1);
    assert_eq!(html.matches(LIMIT_NOTE).count(), 1);
    assert!(!html.contains(BUNDLE));
    assert_eq!(
        header_value(&raw, "Content-Security-Policy").as_deref(),
        Some(CSP)
    );
}

#[test]
fn per_figure_isolation_marks_only_the_bad_figure() {
    let project = project("isolation");
    let server = Server::start(&project.path);
    let raw = server.get_raw("/doc/NOTE-015");
    let html = body(&raw);

    let over_bytes = "a".repeat(100_001);
    assert!(html.contains(&over_limit_figure(1, &over_bytes)));
    assert!(html.contains(&figure(2, "flowchart TD\n  Good --> Figure")));
    assert_eq!(html.matches("data-render-error=\"diagram\"").count(), 1);
    assert_eq!(html.matches("<figure class=\"diagram bleed\"").count(), 2);
    assert_eq!(
        header_value(&raw, "Content-Security-Policy").as_deref(),
        Some(CSP_DIAGRAM),
        "a renderable figure on the page selects the diagram CSP"
    );
    assert_eq!(html.matches(BUNDLE).count(), 1);
}

#[test]
fn client_block_pins_the_mermaid_contract() {
    let js = include_str!("../src/browser/app.js");
    let start = js.find("Diagrams (TASK-146)").expect("diagram block");
    let block = &js[start..];

    for needle in [
        "securityLevel: \"strict\"",
        "htmlLabels: false",
        "theme: \"neutral\"",
        "fontFamily: \"system-ui, sans-serif\"",
        "flowchart: { htmlLabels: false }",
        "startOnLoad: false",
        "maxTextSize: 100000",
        "maxEdges: 500",
        "window.mermaid.initialize",
        "window.mermaid.render",
        "\"papyrus-mmd-\"",
        "window.location.pathname",
        "/[A-Za-z0-9_-]/",
        "getAttribute(\"data-render-error\")",
        "figcaption.render-note",
        REJECTED_NOTE,
        MISSING_NOTE,
    ] {
        assert!(block.contains(needle), "diagram block missing {needle}");
    }
    for forbidden in [
        "bindFunctions",
        "fetch(",
        "import(",
        "MutationObserver",
        "localStorage",
        "layout:",
        "markdownAutoWrap",
        "startOnLoad: true",
    ] {
        assert!(
            !block.contains(forbidden),
            "diagram block must not contain {forbidden}"
        );
    }
}

#[test]
fn client_does_not_install_a_cssstylesheet_substitute() {
    let js = include_str!("../src/browser/app.js");
    let start = js.find("Diagrams (TASK-146)").expect("diagram block");
    let block = &js[start..];

    assert!(
        !block.contains("function ensureConstructibleStyleSheet()"),
        "the shim function must be absent"
    );
    assert!(
        !block.contains("ensureConstructibleStyleSheet();"),
        "the shim call must be absent"
    );
    assert_eq!(
        block.matches("ensureConstructibleStyleSheet").count(),
        0,
        "no shim reference remains"
    );

    for needle in [
        "var Native = window.CSSStyleSheet;",
        "if (!Native) {",
        "new Native();",
        "catch (missing)",
        "function ConstructibleStyleSheet() {",
        "document.implementation.createHTMLDocument(\"\")",
        "inert.createElement(\"style\")",
        "inert.head.appendChild(style)",
        "return style.sheet;",
        "ConstructibleStyleSheet.prototype = Native.prototype;",
        "window.CSSStyleSheet = ConstructibleStyleSheet;",
        "catch (unwritable)",
    ] {
        assert!(!block.contains(needle), "shim residue: {needle}");
    }

    assert_eq!(
        block.matches("window.CSSStyleSheet =").count(),
        0,
        "the sheet constructor is not replaced"
    );
    assert!(!block.contains("globalThis."), "no globalThis override");
    assert!(!block.contains("self.CSSStyleSheet"), "no self override");
    assert!(
        !block.contains("Object.defineProperty"),
        "no property redefinition"
    );
    assert!(
        !block.contains("window.CSSStyleSheetShim"),
        "no extra global name"
    );
}

#[test]
fn vendored_pin_and_manifest_match_app_js() {
    let manifest = include_str!("../src/browser/vendor/MANIFEST.md");
    assert!(
        manifest.contains(
            "| `/assets/mermaid/mermaid.min.js` | mermaid | 11.17.2 | MIT | https://registry.npmjs.org/mermaid/-/mermaid-11.17.2.tgz | 3572661 | `581ed7d74bd9048d0e3a91363927d72ef22942d7722546b27f7cc29e35390eb8` |"
        ),
        "the Mermaid pin, size, and digest are unchanged"
    );
    let bundle = include_bytes!("../src/browser/vendor/mermaid/11.17.2/mermaid.min.js");
    assert_eq!(bundle.len(), 3_572_661);
    let app = include_str!("../src/browser/app.js");
    assert!(
        manifest.contains(&format!("| `/assets/app.js` | {} |", app.len())),
        "the app.js manifest row matches the file"
    );
}

#[test]
fn client_neutralises_rendered_diagram_links() {
    let js = include_str!("../src/browser/app.js");
    let start = js.find("Diagrams (TASK-146)").expect("diagram block");
    let block = &js[start..];

    for needle in [
        "function neutraliseDiagramLinks(root)",
        "root.querySelectorAll(\"a\")",
        "removeAttribute(\"href\")",
        "removeAttribute(\"xlink:href\")",
        "removeAttribute(\"target\")",
        "document.createRange()",
        "range.createContextualFragment(svg)",
        "neutraliseDiagramLinks(scroll);",
    ] {
        assert!(block.contains(needle), "diagram block missing {needle}");
    }
    assert!(
        !block.contains("http://"),
        "diagram block adds no external origin"
    );
    let mount = &block[block.find("function mountDiagram").unwrap()..];
    let insert = mount.find("scroll.appendChild(fragment);").unwrap();
    let neutralise = mount.find("neutraliseDiagramLinks(scroll);").unwrap();
    assert!(
        insert < neutralise,
        "every rendered SVG is neutralised after insertion"
    );
    assert!(
        !block.contains(".removeChild") && !block.contains(".remove()"),
        "links and nodes stay in the DOM"
    );
    assert!(
        !block.contains("innerHTML") && !block.contains("eval("),
        "the success path uses DOM APIs only"
    );
}

#[test]
fn client_missing_script_branch_keeps_content() {
    let js = include_str!("../src/browser/app.js");
    let start = js.find("Diagrams (TASK-146)").expect("diagram block");
    let block = &js[start..];

    assert!(
        block.contains("if (!window.mermaid || typeof window.mermaid.render !== \"function\")"),
        "missing-window branch exists"
    );
    assert!(block.contains(MISSING_NOTE));
    assert!(
        !block.contains(".remove()") && !block.contains("removeChild"),
        "the diagram block removes no content"
    );
    assert!(
        block.contains("figure.setAttribute(\"data-render-error\", \"diagram\")"),
        "figures are marked"
    );
    assert!(
        block.contains("scroll.textContent = \"\";")
            && block.contains("scroll.appendChild(fragment);"),
        "only a successful render replaces the pre"
    );
}

#[test]
fn hostile_fixtures_add_no_script_no_handler_and_no_javascript_url() {
    let project = project("hostile");
    let server = Server::start(&project.path);
    let html = server.get("/doc/NOTE-016");

    assert!(!html.contains("<img"), "no image element from record data");
    assert!(!html.contains(" onerror=\""), "no event-handler attribute");
    assert!(!html.contains(" onload=\""));
    assert!(!html.contains("href=\"javascript:"));
    assert!(!html.contains("src=\"javascript:"));
    assert_external_scripts_only(&html);
    assert!(
        html.contains("&lt;img src=x onerror=alert(1)&gt;"),
        "hostile payload stays escaped text"
    );
    assert!(
        html.contains("background:url(javascript:alert(1))"),
        "the classDef payload stays text inside the pre"
    );
}

#[test]
fn both_csp_literals_per_response_class_and_no_csp_on_assets() {
    let project = project("csp");
    let server = Server::start(&project.path);

    assert_eq!(
        header_value(
            &server.get_raw("/doc/REQ-001.001"),
            "Content-Security-Policy"
        )
        .as_deref(),
        Some(CSP_DIAGRAM),
        "diagram record view"
    );
    assert_eq!(
        header_value(&server.get_raw("/doc/NOTE-015"), "Content-Security-Policy").as_deref(),
        Some(CSP_DIAGRAM),
        "isolation page with one renderable figure"
    );

    for path in [
        "/doc/NOTE-017",
        "/doc/NOTE-019",
        "/doc/NOTE-002",
        "/",
        "/?type=req",
        "/?type=task",
        "/doc/NOTE-002/edit",
        "/nope",
    ] {
        let raw = server.get_raw(path);
        assert_eq!(
            header_value(&raw, "Content-Security-Policy").as_deref(),
            Some(CSP),
            "{path} must keep the strict literal"
        );
        let html = body(&raw);
        assert_external_scripts_only(&html);
        assert!(!html.contains("unsafe-inline"), "{path} html");
    }

    for csp in [CSP, CSP_DIAGRAM] {
        assert_eq!(script_src(csp), "'self'");
        assert!(!script_src(csp).contains("unsafe-inline"));
        assert!(!script_src(csp).contains("unsafe-eval"));
        assert!(!csp.contains("nonce-"));
        assert!(!csp.contains("sha256-"));
    }

    let raw = server.get_raw(BUNDLE);
    assert!(status(&raw).contains(" 200"), "{}", status(&raw));
    assert_eq!(
        header_value(&raw, "Content-Type").as_deref(),
        Some("text/javascript; charset=utf-8")
    );
    assert!(
        header_value(&raw, "Content-Security-Policy").is_none(),
        "asset responses carry no CSP"
    );
}

#[test]
fn bundle_is_linked_once_on_diagram_pages_only_in_script_order() {
    let project = project("bundle");
    let server = Server::start(&project.path);

    let html = server.get("/doc/REQ-001.001");
    assert_eq!(html.matches(BUNDLE).count(), 1, "bundle exactly once");
    let app_css = html.find("/assets/app.css").unwrap();
    let katex_css = html.find("/assets/katex/katex.min.css").unwrap();
    let katex_js = html.find("/assets/katex/katex.min.js").unwrap();
    let mermaid = html.find(BUNDLE).unwrap();
    let app_js = html.find("/assets/app.js").unwrap();
    assert!(app_css < katex_css, "KaTeX CSS after app.css");
    assert!(katex_css < katex_js, "KaTeX CSS before KaTeX JS");
    assert!(katex_js < mermaid, "KaTeX JS before Mermaid JS");
    assert!(mermaid < app_js, "Mermaid before app.js");
    assert_external_scripts_only(&html);

    for path in [
        "/doc/NOTE-017",
        "/doc/NOTE-013",
        "/doc/NOTE-014",
        "/doc/NOTE-010",
        "/doc/NOTE-018",
        "/doc/NOTE-019",
        "/?type=req",
        "/?type=task",
        "/doc/NOTE-002/edit",
        "/nope",
    ] {
        let html = server.get(path);
        assert!(
            !html.contains(BUNDLE),
            "{path} must not link the diagram bundle"
        );
    }

    let root = server.get("/");
    assert!(!root.contains(BUNDLE));
}

#[test]
fn cli_json_search_and_stored_bytes_keep_the_source() {
    let project = project("cli");
    let server = Server::start(&project.path);
    let path = project
        .path
        .join("docs/requirements/business/REQ-001.001.md");
    let before = fs::read(&path).unwrap();

    let _ = server.get("/doc/REQ-001.001");
    assert_eq!(
        fs::read(&path).unwrap(),
        before,
        "stored bytes are unchanged"
    );

    let view = run_ok(&project.path, &["req", "view", "REQ-001.001"]);
    for needle in ["```mermaid", "flowchart TD", "gitGraph"] {
        assert!(view.contains(needle), "CLI view lost {needle}");
    }
    let json = run_ok(&project.path, &["req", "view", "REQ-001.001", "--json"]);
    assert!(json.contains("flowchart TD"), "JSON view lost the source");
    assert!(json.contains("gitGraph"), "JSON view lost the source");

    let search = run_ok(&project.path, &["search", "sequenceDiagram"]);
    assert!(
        search.contains("REQ-001.001"),
        "search must find diagram source: {search}"
    );
}

#[test]
fn no_jvm_or_plantuml_invocation_exists() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let manifest = fs::read_to_string(root.join("Cargo.toml")).unwrap();
    for forbidden in ["plantuml", "mermaid"] {
        assert!(
            !manifest.contains(forbidden),
            "Cargo.toml must not mention {forbidden}"
        );
    }

    fn scan(dir: &Path) {
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if entry.file_type().unwrap().is_dir() {
                scan(&path);
                continue;
            }
            if path.extension().is_some_and(|value| value == "rs") {
                let text = fs::read_to_string(&path).unwrap();
                for forbidden in [
                    "plantuml.jar",
                    "java -jar",
                    "Command::new(\"java\")",
                    "Command::new(\"plantuml\")",
                ] {
                    assert!(
                        !text.contains(forbidden),
                        "{} contains {forbidden}",
                        path.display()
                    );
                }
            }
        }
    }
    scan(&root.join("src"));
}

#[test]
fn diagram_styling_supports_full_width_scroll() {
    let css = include_str!("../src/browser/app.css");
    for needle in [
        ".diagram { margin: 12px 0; }",
        ".diagram-scroll { overflow-x: auto; }",
        ".diagram-scroll:focus-visible { outline: 2px solid var(--ink); outline-offset: 2px; }",
        ".diagram--unsupported",
        ".render-note",
        ".bleed { width: 100%; overflow-x: auto; }",
    ] {
        assert!(css.contains(needle), "app.css missing {needle}");
    }
    assert!(
        !css.contains("svg {"),
        "no rule targets a Mermaid-generated svg"
    );
    assert!(
        !css.lines()
            .any(|line| line.contains("max-width") && line.contains("svg")),
        "no max-width on a rendered SVG"
    );
    assert!(!css.contains("url("), "no url() in app.css");
}
