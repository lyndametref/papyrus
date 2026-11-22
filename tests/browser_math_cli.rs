//! TASK-145: the math delimiter pre-pass, conditional KaTeX assets, the CSP
//! delta, the non-Markdown boundary, and the fallback in the served HTML.
//!
//! The contract is `docs/specifications/browser-math-and-diagram-rendering.md`.
//! The fixture starts as a copy of the characterization project and replaces
//! the prose bodies with math, hostile, and Markdown-heavy content.

use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

const CSP: &str = "default-src 'none'; style-src 'self'; script-src 'self'; img-src 'self'; font-src 'self'; form-action 'self'; base-uri 'none'; frame-ancestors 'none'";
const IMMUTABLE_CACHE: &str = "public, max-age=31536000, immutable";
const KATEX_CSS: &str = "/assets/katex/0.16.11/katex.min.css";
const KATEX_JS: &str = "/assets/katex/0.16.11/katex.min.js";

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
    path.push(format!("browser-math-{}-{}", name, std::process::id()));
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

fn set_title(dir: &Path, relative: &str, title: &str) {
    let path = dir.join(relative);
    let text = fs::read_to_string(&path).unwrap();
    let stem = Path::new(relative)
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .to_string();
    let mut out = String::new();
    for line in text.lines() {
        if line.starts_with("title: ") {
            out.push_str(&format!("title: \"{title}\"\n"));
        } else if line.starts_with("# ") {
            out.push_str(&format!("# {stem} {title}\n"));
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }
    fs::write(path, out).unwrap();
}

const REQ_TITLE: &str = "Requirement One $title$";
const TASK_TITLE: &str = "Task One $tasktitle$";
const REQ_BODY: &str = "\
## User Story
As a user I want $$\\sum_{i=1}^n i$$

## Description
Inline $a^2+b^2=c^2$ and $hyperbolic$ plus a display:

$$\\int_0^1 x\\,dx$$

```math
E = mc^2
```

Price \\$5.";

fn project(name: &str) -> TempProject {
    let project = temp_project(name);
    let dir = &project.path;

    set_title(dir, "docs/requirements/business/REQ-001.001.md", REQ_TITLE);
    let req_path = dir.join("docs/requirements/business/REQ-001.001.md");
    let req_text = fs::read_to_string(&req_path).unwrap();
    fs::write(
        &req_path,
        req_text.replace("labels:\n  - core", "labels:\n  - core\n  - $lab$"),
    )
    .unwrap();
    set_body(dir, "docs/requirements/business/REQ-001.001.md", REQ_BODY);
    set_title(dir, "backlog/tasks/TASK-001.md", TASK_TITLE);
    set_body(
        dir,
        "backlog/tasks/TASK-001.md",
        "\
## Description
Task math $t=1$.

## Acceptance Criteria
- [ ] Acceptance keeps $x$ here.",
    );
    set_body(
        dir,
        "backlog/milestones/MS-001.md",
        "## Description\nMilestone $m$.",
    );
    set_body(
        dir,
        "backlog/releases/REL-001.md",
        "## Description\nRelease $$r$$.",
    );
    set_body(
        dir,
        "decisions/ADR-001-decision-one.md",
        "\
## Status
proposed

## Decision
$$d$$

## Context
$c$

## Consequences
```latex
k = 1
```",
    );
    set_body(
        dir,
        "docs/glossary/GLO-001.md",
        "## Definition\nGlossary $g$.",
    );
    set_body(
        dir,
        "docs/specifications/ui/UI-002.md",
        "\
## View
$view$

## Description
```tex
u = 1
```",
    );
    set_body(
        dir,
        "docs/business-rules/RUL-001.md",
        "\
## Description
$rule$

## Rationale
$$why$$

## Applicability
$scope$",
    );
    set_body(
        dir,
        "docs/audit-records/AUD-002.md",
        "## Audit Type\nsecurity\n\n## Content\n$audit$.",
    );
    set_body(dir, "docs/technical/TECH-002.md", "## Description\n$tech$.");
    set_body(
        dir,
        "docs/use-cases/UC-002.md",
        "## Description\n$uc$.\n\n## Applicability\n$when$.",
    );
    set_body(
        dir,
        "docs/test-cases/TC-002.md",
        "## Description\n$tc$.\n\n## Applicability\n$when$.",
    );
    set_body(
        dir,
        "docs/test-reports/TR-002.md",
        "## Description\n$tr$.\n\n## Applicability\n$when$.",
    );
    set_body(
        dir,
        "docs/notes/NOTE-001.md",
        "## Description\nGeneric document body with $n$ math.",
    );
    set_body(
        dir,
        "docs/notes/NOTE-002.md",
        "## Description\nPlain note body with no formulas.",
    );

    fs::write(
        dir.join("docs/notes/NOTE-005.md"),
        "\
---
id: NOTE-005
title: \"Hostile Note\"
created_date: '2026-11-16T00:00:00Z'
updated_date: '2026-11-16T00:00:00Z'
labels: []
references: []
status: active
---
# NOTE-005 Hostile Note

## Description

Inline $\\href{javascript:alert(1)}{x}$ and $\\url{https://example.test}$ and $\\includegraphics{https://example.test/x.png}$.

Escaped $\\<script>alert(1)</script>$ and $\"><img src=x onerror=alert(1)>$.

Literal <script>alert(1)</script> and <img src=x onerror=alert(1)>.
",
    )
    .unwrap();

    fs::write(
        dir.join("docs/notes/NOTE-006.md"),
        "\
---
id: NOTE-006
title: \"Markdown Note\"
created_date: '2026-11-16T00:00:00Z'
updated_date: '2026-11-16T00:00:00Z'
labels: []
references: []
status: active
---
# NOTE-006 Markdown Note

## Description

# Heading stays literal
- list item
1. ordered item
| a | b |
**bold** and *em* and [link](https://example.test) and `code`
> quote

```rust
$x$ and $$y$$ stay literal
```
",
    )
    .unwrap();

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

/// Walk every `data-tex` element and assert its text content is its source.
fn assert_fallback_equals_source(html: &str) {
    let mut rest = html;
    let mut found = 0;
    while let Some(start) = rest.find("data-tex=\"") {
        let after = &rest[start + "data-tex=\"".len()..];
        let Some(end) = after.find("\">") else {
            panic!("unterminated data-tex attribute");
        };
        let source = &after[..end];
        let content = &after[end + 2..];
        let Some(text_end) = content.find('<') else {
            panic!("unterminated math element");
        };
        assert_eq!(
            &content[..text_end],
            source,
            "element text must equal data-tex"
        );
        found += 1;
        rest = &content[text_end..];
    }
    assert!(found > 0, "expected at least one math element");
}

#[test]
fn prose_fields_render_all_forms_and_nothing_else() {
    let project = project("fields");
    let server = Server::start(&project.path);

    let req = server.get("/doc/REQ-001.001");
    for source in [
        "a^2+b^2=c^2",
        "hyperbolic",
        "\\int_0^1 x\\,dx",
        "E = mc^2",
        "\\sum_{i=1}^n i",
    ] {
        assert!(
            req.contains(&format!("data-tex=\"{source}\"")),
            "missing data-tex {source} in requirement view"
        );
    }
    assert_eq!(
        req.matches("data-tex=\"").count(),
        5,
        "only the two story/description prose fields carry math"
    );
    assert!(req.contains("<span class=\"math\" data-tex=\"a^2+b^2=c^2\">"));
    assert!(req.contains("<div class=\"math math--display\" data-tex=\"E = mc^2\">"));
    assert!(req.contains(REQ_TITLE), "title stays literal text");
    assert!(!req.contains("data-tex=\"title\""), "title is not math");
    assert!(
        req.contains("<span class=\"tag\">$lab$</span>"),
        "labels stay literal text"
    );
    assert!(!req.contains("data-tex=\"lab\""), "labels are not math");

    let req_list = server.get("/?type=req");
    assert!(
        !req_list.contains("data-tex"),
        "list cells never run the pre-pass"
    );
    assert!(req_list.contains(REQ_TITLE));

    let task = server.get("/doc/TASK-001");
    assert!(task.contains("data-tex=\"t=1\""));
    assert!(task.contains("$x$"), "acceptance criteria stay literal");
    assert!(!task.contains("data-tex=\"x\""));

    let board = server.get("/?type=task");
    assert!(
        !board.contains("data-tex"),
        "board cards never run the pre-pass"
    );
    assert!(board.contains(TASK_TITLE));

    for (path, source) in [
        ("/doc/MS-001", "m"),
        ("/doc/REL-001", "r"),
        ("/doc/ADR-001", "d"),
        ("/doc/ADR-001", "c"),
        ("/doc/ADR-001", "k = 1"),
        ("/doc/GLO-001", "g"),
        ("/doc/UI-002", "view"),
        ("/doc/UI-002", "u = 1"),
        ("/doc/RUL-001", "rule"),
        ("/doc/RUL-001", "why"),
        ("/doc/RUL-001", "scope"),
        ("/doc/AUD-002", "audit"),
        ("/doc/TECH-002", "tech"),
        ("/doc/UC-002", "uc"),
        ("/doc/UC-002", "when"),
        ("/doc/TC-002", "tc"),
        ("/doc/TR-002", "tr"),
        ("/doc/NOTE-001", "n"),
    ] {
        let html = server.get(path);
        assert!(
            html.contains(&format!("data-tex=\"{source}\"")),
            "{path} must render data-tex {source}"
        );
    }
    assert!(
        server
            .get("/doc/NOTE-001")
            .contains("<p class=\"text\">Generic document body with")
    );
}

#[test]
fn katex_assets_appear_once_only_on_math_pages() {
    let project = project("assets");
    let server = Server::start(&project.path);

    for path in ["/doc/REQ-001.001", "/doc/NOTE-001", "/doc/NOTE-005"] {
        let html = server.get(path);
        assert_eq!(
            html.matches("/assets/katex/0.16.11/katex.min.css").count(),
            1,
            "{path} stylesheet exactly once"
        );
        assert_eq!(
            html.matches("/assets/katex/0.16.11/katex.min.js").count(),
            1,
            "{path} script exactly once"
        );
        assert!(
            !html.contains("/assets/katex/katex.min.css"),
            "{path} references the unversioned KaTeX stylesheet"
        );
        assert!(
            !html.contains("/assets/katex/katex.min.js"),
            "{path} references the unversioned KaTeX script"
        );
        let css = html.find("/assets/katex/0.16.11/katex.min.css").unwrap();
        let js = html.find("/assets/katex/0.16.11/katex.min.js").unwrap();
        let app = html.find("/assets/app.js").unwrap();
        let app_css = html.find("/assets/app.css").unwrap();
        assert!(
            html.contains("<script src=\"/assets/katex/0.16.11/katex.min.js\" defer></script>"),
            "{path} katex script is deferred"
        );
        assert!(
            !html.contains("/assets/mermaid/"),
            "{path} must not request Mermaid"
        );
        assert!(app_css < css, "{path} math CSS follows app.css");
        assert!(js < app, "{path} katex script precedes app.js");
        assert!(css < js, "{path} stylesheet precedes script");
        for (index, _) in html.match_indices("<script") {
            assert!(
                html[index + "<script".len()..].starts_with(" src=\""),
                "{path} has an inline script element"
            );
        }
    }

    for path in [
        "/",
        "/?type=req",
        "/?type=task",
        "/lifecycle",
        "/config",
        "/skills",
        "/?type=task&cleanup=MS-001",
        "/nope",
        "/doc/NOTE-002",
        "/doc/NOTE-001/edit",
        "/doc/TASK-001/edit",
    ] {
        let html = server.get(path);
        assert!(
            !html.contains("/assets/katex/"),
            "{path} must not ship KaTeX assets"
        );
        assert!(!html.contains("data-tex"), "{path} runs no pre-pass");
        assert!(
            !html.contains("/assets/mermaid/"),
            "{path} must not request Mermaid"
        );
    }
}

// TASK-277 AC #5, #6: KaTeX is available exactly where math exists, from the
// versioned immutable vendor class, and a math page never requests Mermaid.
#[test]
fn math_assets_are_immutable_and_math_pages_request_no_mermaid() {
    let project = project("immutable-math");
    let server = Server::start(&project.path);

    for asset in [KATEX_CSS, KATEX_JS] {
        let raw = server.get_raw(asset);
        assert!(status(&raw).contains(" 200"), "{asset}: {}", status(&raw));
        assert_eq!(
            header_value(&raw, "Cache-Control").as_deref(),
            Some(IMMUTABLE_CACHE),
            "{asset} cache class"
        );
        assert!(header_value(&raw, "ETag").is_none(), "{asset} has no ETag");
        assert!(
            header_value(&raw, "Content-Security-Policy").is_none(),
            "{asset} has no CSP"
        );
    }

    for path in ["/doc/REQ-001.001", "/doc/NOTE-001", "/doc/NOTE-005"] {
        let html = server.get(path);
        assert!(html.contains(KATEX_CSS), "{path} without KaTeX CSS");
        assert!(
            html.contains("<script src=\"/assets/katex/0.16.11/katex.min.js\" defer></script>"),
            "{path} without the deferred KaTeX script"
        );
        assert!(
            !html.contains("/assets/mermaid/"),
            "{path} must not request Mermaid"
        );
    }
}

#[test]
fn asset_responses_have_types_and_no_csp() {
    let project = project("asset-headers");
    let server = Server::start(&project.path);
    for (path, mime) in [
        ("/assets/app.css", "text/css; charset=utf-8"),
        (
            "/assets/katex/0.16.11/katex.min.css",
            "text/css; charset=utf-8",
        ),
        (
            "/assets/katex/0.16.11/katex.min.js",
            "text/javascript; charset=utf-8",
        ),
        (
            "/assets/katex/0.16.11/fonts/KaTeX_Main-Regular.woff2",
            "font/woff2",
        ),
    ] {
        let raw = server.get_raw(path);
        assert!(status(&raw).contains(" 200"), "{path}: {}", status(&raw));
        assert_eq!(
            header_value(&raw, "Content-Type").as_deref(),
            Some(mime),
            "{path} content type"
        );
        assert!(
            header_value(&raw, "Content-Security-Policy").is_none(),
            "{path} must not carry a CSP header"
        );
    }
}

#[test]
fn csp_is_the_exact_literal_on_every_html_class() {
    let project = project("csp");
    let server = Server::start(&project.path);
    for path in [
        "/",
        "/?type=req",
        "/doc/REQ-001.001",
        "/doc/NOTE-002",
        "/doc/NOTE-001/edit",
        "/nope",
    ] {
        let raw = server.get_raw(path);
        assert_eq!(
            header_value(&raw, "Content-Security-Policy").as_deref(),
            Some(CSP),
            "{path} CSP"
        );
        let html = body(&raw);
        assert!(!html.contains("unsafe-inline"), "{path} unsafe-inline");
        assert!(!html.contains("unsafe-hashes"), "{path} unsafe-hashes");
        assert!(!html.contains("<script>"), "{path} inline script body");
    }
}

#[test]
fn javascript_disabled_fallback_is_the_escaped_source() {
    let project = project("fallback");
    let server = Server::start(&project.path);
    for path in ["/doc/REQ-001.001", "/doc/ADR-001", "/doc/NOTE-005"] {
        assert_fallback_equals_source(&server.get(path));
    }
}

#[test]
fn client_pass_pins_the_katex_options_and_error_marker() {
    let js = include_str!("../src/browser/app.js");
    for needle in [
        "window.katex.render",
        "throwOnError: false",
        "trust: false",
        "strict: \"warn\"",
        "output: \"htmlAndMathml\"",
        "data-render-error",
        "math--display",
    ] {
        assert!(js.contains(needle), "app.js missing {needle}");
    }
    assert!(!js.contains("fetch("), "no fetch");
    assert!(!js.contains("import("), "no dynamic import");
    assert!(!js.contains("MutationObserver"), "no observer");
    assert!(!js.contains("trust: true"));
}

#[test]
fn client_pass_restores_the_source_before_marking_a_thrown_error() {
    let js = include_str!("../src/browser/app.js");
    let catch_block = "} catch (e) {\n            element.textContent = source;\n            element.setAttribute(\"data-render-error\", \"math\");\n          }";
    assert!(
        js.contains(catch_block),
        "a thrown render must restore the source, then mark the element"
    );
}

#[test]
fn markdown_stays_literal_beyond_the_math_delimiters() {
    let project = project("markdown");
    let server = Server::start(&project.path);
    let html = server.get("/doc/NOTE-006");
    for needle in [
        "# Heading stays literal",
        "- list item",
        "1. ordered item",
        "| a | b |",
        "**bold**",
        "*em*",
        "[link](https://example.test)",
        "`code`",
        "&gt; quote",
        "```rust",
        "$x$ and $$y$$ stay literal",
    ] {
        assert!(html.contains(needle), "missing literal text: {needle}");
    }
    assert!(!html.contains("data-tex"), "non-math fences stay opaque");
    assert!(!html.contains("<em>"), "no emphasis element");
    assert!(!html.contains("<table"), "no table element");
    assert!(!html.contains("<h2>Heading"), "no heading interpretation");
    assert!(
        !html.contains("href=\"https://example.test\""),
        "no link interpretation"
    );
}

#[test]
fn hostile_tex_and_html_add_no_script_or_remote_reference() {
    let project = project("hostile");
    let server = Server::start(&project.path);
    let html = server.get("/doc/NOTE-005");

    assert!(
        !html.contains("<script>alert"),
        "no injected script element"
    );
    assert!(!html.contains("href=\"javascript:"), "no javascript href");
    assert!(!html.contains("src=\"javascript:"), "no javascript src");
    assert!(!html.contains("src=\"http"), "no remote img source");
    assert!(!html.contains("href=\"http"), "no remote link");
    assert_eq!(
        html.matches("javascript:").count(),
        2,
        "the only javascript: text is the escaped TeX payload and its fallback text"
    );
    assert_eq!(
        html.matches("https://").count(),
        4,
        "the only https text is the two escaped TeX payloads"
    );
    for (index, _) in html.match_indices("<script") {
        assert!(
            html[index + "<script".len()..].starts_with(" src=\""),
            "only external script elements are allowed"
        );
    }
    for payload in [
        "\\href{javascript:alert(1)}{x}",
        "\\url{https://example.test}",
        "\\includegraphics{https://example.test/x.png}",
        "\\&lt;script&gt;alert(1)&lt;/script&gt;",
        "&quot;&gt;&lt;img src=x onerror=alert(1)&gt;",
    ] {
        assert!(
            html.contains(&format!("data-tex=\"{payload}\"")),
            "payload must stay escaped inside data-tex: {payload}"
        );
    }
    assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    assert_fallback_equals_source(&html);
}

#[test]
fn cli_search_and_stored_bytes_keep_the_math_source() {
    let project = project("cli");
    let server = Server::start(&project.path);
    let _ = server.get("/doc/REQ-001.001");

    let view = run_ok(&project.path, &["req", "view", "REQ-001.001"]);
    for needle in ["$$\\sum_{i=1}^n i$$", "$hyperbolic$", "\\$5"] {
        assert!(view.contains(needle), "CLI view lost {needle}");
    }
    let search = run_ok(&project.path, &["search", "hyperbolic"]);
    assert!(
        search.contains("REQ-001.001"),
        "search must match a term inside a formula: {search}"
    );

    let stored = fs::read_to_string(
        project
            .path
            .join("docs/requirements/business/REQ-001.001.md"),
    )
    .unwrap();
    assert!(stored.contains("E = mc^2"));
    assert!(stored.contains("\\$5"));
    assert!(stored.contains("$$\\int_0^1 x\\,dx$$"));
}

#[test]
fn serve_smoke_math_page_and_math_free_page() {
    // Mirrors the manual browser check: one math requirement, one milestone,
    // one generic document; a math-free page ships no KaTeX bytes.
    let project = project("smoke");
    let server = Server::start(&project.path);
    for path in ["/doc/REQ-001.001", "/doc/MS-001", "/doc/NOTE-001"] {
        let html = server.get(path);
        assert!(
            html.contains("/assets/katex/0.16.11/katex.min.js"),
            "{path}"
        );
        assert_fallback_equals_source(&html);
    }
    let plain = server.get("/doc/NOTE-002");
    assert!(!plain.contains("katex"));
}
