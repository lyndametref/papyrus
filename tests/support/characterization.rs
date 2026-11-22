//! Shared harness for the characterization capture and replay tests.
//!
//! TASK-240: boots `CARGO_BIN_EXE_papyrus` for the CLI matrix and a
//! `browser --no-open` server for the HTTP matrix, normalises the volatile
//! tokens, and writes or compares the expected tree.
//!
//! This module is test-only. It is included by `tests/characterization_capture.rs`
//! and `tests/characterization_replay.rs`; Cargo never compiles it on its own.

use std::collections::BTreeMap;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use serde::Deserialize;

/// Literal token that replaces every ISO-8601 UTC timestamp.
pub const TIMESTAMP: &str = "{{TIMESTAMP}}";
/// Literal token that replaces the shell bind note (`host:port`).
pub const BIND: &str = "{{BIND}}";
/// Literal token that replaces the absolute project directory in payloads.
pub const PROJECT: &str = "{{PROJECT}}";
/// Literal token that replaces the mtime-derived edit-form version token.
pub const VERSION: &str = "{{VERSION}}";
/// Request-body placeholder replaced with the record's current version token,
/// so a success case can submit without hard-coding an mtime-derived value.
const VERSION_PLACEHOLDER: &str = "{{VERSION}}";
/// Literal token that replaces a response Content-Length whose body echoes a
/// variable-length version token.
const LENGTH: &str = "{{LENGTH}}";

/// Artifact names the corpus may declare.
pub const ARTIFACTS: &[&str] = &[
    "record.md",
    "record.html",
    "stdout.txt",
    "stderr.txt",
    "exit.json",
    "payload.json",
    "http.json",
    "page.html",
    "body.txt",
];

#[derive(Debug, Clone, Deserialize)]
pub struct RequestSpec {
    pub method: String,
    pub path: String,
    #[serde(default)]
    pub body: Option<String>,
    /// Request `Content-Type`. Defaults to form encoding when a body is sent,
    /// because every pre-existing POST case submits an HTML form.
    #[serde(default)]
    pub content_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Case {
    pub id: String,
    pub kind: String,
    #[serde(default)]
    pub argv: Vec<String>,
    #[serde(default)]
    pub request: Option<RequestSpec>,
    #[serde(default)]
    pub document_type: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub renderer: Option<String>,
    #[serde(default)]
    pub record: Option<String>,
    /// Seed project directory name; defaults to `project`.
    #[serde(default)]
    pub project: Option<String>,
    #[serde(default)]
    pub artifacts: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Manifest {
    pub schema: String,
    #[serde(default)]
    pub unreachable: Vec<serde_json::Value>,
    pub cases: Vec<Case>,
}

/// The committed corpus root.
pub fn corpus_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/characterization/corpus")
}

pub fn load_manifest(root: &Path) -> Manifest {
    let path = root.join("cases.json");
    let text = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|error| panic!("parse {}: {error}", path.display()))
}

/// One replay/capture working area under `./.tmp/`.
pub struct Ctx {
    work: PathBuf,
}

impl Ctx {
    pub fn new(tag: &str) -> Ctx {
        let work = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(".tmp")
            .join(format!("characterization-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&work);
        fs::create_dir_all(&work).expect("create characterization workspace");
        Ctx { work }
    }

    fn case_dir(&self, id: &str) -> PathBuf {
        let slug: String = id.chars().map(|c| if c == '/' { '-' } else { c }).collect();
        self.work.join(slug)
    }
}

impl Drop for Ctx {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.work);
    }
}

fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).expect("create copy root");
    for entry in fs::read_dir(from).expect("read seed project") {
        let entry = entry.expect("seed entry");
        let source = entry.path();
        let destination = to.join(entry.file_name());
        if source.is_dir() {
            copy_tree(&source, &destination);
        } else {
            fs::copy(&source, &destination).expect("copy seed file");
        }
    }
}

/// Normalise the volatile tokens of one captured artifact.
pub struct Normaliser {
    project: Vec<String>,
    bind: String,
}

impl Normaliser {
    pub fn new(project: &Path, bind: &str) -> Normaliser {
        let mut paths = vec![project.to_string_lossy().to_string()];
        if let Ok(canonical) = fs::canonicalize(project) {
            paths.push(canonical.to_string_lossy().to_string());
        }
        paths.sort_by_key(|path| std::cmp::Reverse(path.len()));
        paths.dedup();
        Normaliser {
            project: paths,
            bind: bind.to_string(),
        }
    }

    pub fn apply(&self, bytes: &[u8]) -> Vec<u8> {
        let text = String::from_utf8_lossy(bytes);
        let mut text = text.to_string();
        if !self.bind.is_empty() {
            text = text.replace(&self.bind, BIND);
        }
        for path in &self.project {
            text = text.replace(path, PROJECT);
        }
        let text = normalise_versions(&text);
        normalise_timestamps(&text).into_bytes()
    }
}

fn normalise_versions(text: &str) -> String {
    const ATTR: &str = "name=\"version\" value=\"";
    const JSON: &str = "\"version\":\"";
    let text = replace_version_after(text, ATTR);
    replace_version_after(&text, JSON)
}

/// Replace the quoted value after `prefix` with [`VERSION`] when the value is
/// an mtime-derived version token. Both the edit-form hidden input and the
/// JSON `version` field carry the same volatile token.
fn replace_version_after(text: &str, prefix: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(position) = rest.find(prefix) {
        out.push_str(&rest[..position + prefix.len()]);
        let value = &rest[position + prefix.len()..];
        let Some(end) = value.find('"') else {
            out.push_str(value);
            return out;
        };
        if is_version_token(&value[..end]) {
            out.push_str(VERSION);
        } else {
            out.push_str(&value[..end]);
        }
        out.push('"');
        rest = &value[end + 1..];
    }
    out.push_str(rest);
    out
}

fn is_version_token(value: &str) -> bool {
    let parts: Vec<&str> = value.split('.').collect();
    parts.len() == 3 && parts.iter().all(|part| digits(part.as_bytes()))
}

fn digits(bytes: &[u8]) -> bool {
    !bytes.is_empty() && bytes.iter().all(u8::is_ascii_digit)
}

fn is_timestamp(candidate: &[u8]) -> bool {
    candidate.len() == 20
        && digits(&candidate[0..4])
        && candidate[4] == b'-'
        && digits(&candidate[5..7])
        && candidate[7] == b'-'
        && digits(&candidate[8..10])
        && candidate[10] == b'T'
        && digits(&candidate[11..13])
        && candidate[13] == b':'
        && digits(&candidate[14..16])
        && candidate[16] == b':'
        && digits(&candidate[17..19])
        && candidate[19] == b'Z'
}

fn normalise_timestamps(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = String::with_capacity(text.len());
    let mut index = 0;
    while index < bytes.len() {
        if index + 20 <= bytes.len() && is_timestamp(&bytes[index..index + 20]) {
            out.push_str(TIMESTAMP);
            index += 20;
            continue;
        }
        let character = text[index..].chars().next().expect("character boundary");
        out.push(character);
        index += character.len_utf8();
    }
    out
}

/// A run of the corpus: `write` captures, otherwise replay compares.
pub struct Runner<'a> {
    pub root: PathBuf,
    pub ctx: &'a Ctx,
    pub write: bool,
    /// Restrict the run to one case kind (`cli` or `http`); `None` runs all.
    pub only: Option<&'a str>,
}

impl Runner<'_> {
    pub fn run(&self) {
        let manifest = load_manifest(&self.root);
        assert_eq!(manifest.schema, "papyrus.characterization-cases/1");
        assert!(!manifest.cases.is_empty(), "the case manifest is empty");
        for case in &manifest.unreachable {
            assert!(
                case.get("renderer").is_some() && case.get("reason").is_some(),
                "every unreachable renderer names itself and its reason"
            );
        }
        for case in &manifest.cases {
            validate_case(case);
            for artifact in &case.artifacts {
                assert!(
                    ARTIFACTS.contains(&artifact.as_str()),
                    "{} declares unknown artifact {artifact}",
                    case.id
                );
            }
        }

        let expected = self.root.join("expected");
        if self.write && self.only.is_none() {
            let _ = fs::remove_dir_all(&expected);
            fs::create_dir_all(&expected).expect("create expected tree");
        }

        let selected: Vec<&Case> = manifest
            .cases
            .iter()
            .filter(|case| self.only.is_none_or(|kind| case.kind == kind))
            .collect();
        let mut servers: BTreeMap<String, (Server, PathBuf, String)> = BTreeMap::new();
        let mut failures = Vec::new();
        for case in selected {
            let project = case
                .project
                .clone()
                .unwrap_or_else(|| "project".to_string());
            let seed = self.root.join(&project);
            assert!(seed.is_dir(), "{}: missing seed project {project}", case.id);
            let observed = match case.kind.as_str() {
                "cli" => observe_cli(case, self.ctx, &seed),
                "http" => {
                    // HTTP cases run against a throwaway copy: successful form
                    // and API writes must never touch the committed seed.
                    let entry = servers.entry(project.clone()).or_insert_with(|| {
                        let directory = self.ctx.work.join(format!("http-{project}"));
                        let _ = fs::remove_dir_all(&directory);
                        copy_tree(&seed, &directory);
                        let server = Server::start(&directory);
                        let bind = server.bind_note();
                        (server, directory, bind)
                    });
                    observe_http(case, &entry.0, &entry.1, &entry.2)
                }
                other => panic!("{}: unknown case kind {other}", case.id),
            };
            let directory = expected.join(&case.id);
            if self.write {
                write_case(&directory, &observed);
            } else if let Err(error) = compare_case(&directory, &observed, case) {
                failures.push(format!("{}\n{error}", case.id));
            }
        }
        assert!(
            failures.is_empty(),
            "characterization replay drifted from the expected snapshot:\n\n{}",
            failures.join("\n\n")
        );
    }
}

fn validate_case(case: &Case) {
    assert!(!case.id.is_empty(), "every case has an id");
    assert!(
        case.document_type
            .as_deref()
            .is_some_and(|value| !value.is_empty()),
        "{}: document_type is required",
        case.id
    );
    if let Some(model) = &case.model {
        assert!(
            matches!(
                model.as_str(),
                "SingleMarkdown" | "GroupedMarkdown" | "SingleHtmlMockup"
            ),
            "{}: unknown content model {model}",
            case.id
        );
    }
    assert!(
        !case.artifacts.is_empty(),
        "{}: declares no artifacts",
        case.id
    );
    if let Some(project) = &case.project {
        assert!(
            project == "project" || project == "project-empty",
            "{}: unknown seed project {project}",
            case.id
        );
    }
    if case
        .artifacts
        .iter()
        .any(|artifact| artifact == "page.html")
    {
        assert!(
            case.renderer
                .as_deref()
                .is_some_and(|value| !value.is_empty()),
            "{}: rendered pages name their renderer",
            case.id
        );
    }
}

fn observe_cli(case: &Case, ctx: &Ctx, seed: &Path) -> BTreeMap<String, Vec<u8>> {
    let directory = ctx.case_dir(&case.id);
    let _ = fs::remove_dir_all(&directory);
    copy_tree(seed, &directory);
    let output = Command::new(env!("CARGO_BIN_EXE_papyrus"))
        .arg("--dir")
        .arg(&directory)
        .args(&case.argv)
        .stdin(Stdio::null())
        .output()
        .unwrap_or_else(|error| panic!("{}: run papyrus: {error}", case.id));

    let normaliser = Normaliser::new(&directory, "");
    let mut observed = BTreeMap::new();
    observed.insert("stdout.txt".to_string(), normaliser.apply(&output.stdout));
    observed.insert("stderr.txt".to_string(), normaliser.apply(&output.stderr));
    observed.insert(
        "exit.json".to_string(),
        format!("{{\"exit\":{}}}\n", output.status.code().unwrap_or(-1)).into_bytes(),
    );
    if case.argv.iter().any(|argument| argument == "--json") {
        observed.insert("payload.json".to_string(), normaliser.apply(&output.stdout));
    }
    if let Some(relative) = &case.record {
        let path = directory.join(relative);
        let bytes = fs::read(&path).unwrap_or_else(|error| {
            panic!(
                "{}: read recorded file {}: {error}",
                case.id,
                path.display()
            )
        });
        let name = if path.extension().and_then(|value| value.to_str()) == Some("html") {
            "record.html"
        } else {
            "record.md"
        };
        observed.insert(name.to_string(), normaliser.apply(&bytes));
    }
    expected_subset(case, observed)
}

fn expected_subset(case: &Case, observed: BTreeMap<String, Vec<u8>>) -> BTreeMap<String, Vec<u8>> {
    let mut declared: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    for artifact in &case.artifacts {
        let bytes = observed.get(artifact).unwrap_or_else(|| {
            panic!(
                "{}: artifact {artifact} is declared but the case does not produce it",
                case.id
            )
        });
        declared.insert(artifact.clone(), bytes.clone());
    }
    if declared.len() != observed.len() {
        let extra: Vec<&String> = observed
            .keys()
            .filter(|key| !declared.contains_key(*key))
            .collect();
        panic!("{}: produced undeclared artifacts {extra:?}", case.id);
    }
    declared
}

/// The document ID addressed by a `POST /doc/<ID>` or `POST /api/doc/<ID>`
/// route, used only to look up the record's current version token.
fn request_target_id(path: &str) -> Option<&str> {
    let rest = path
        .strip_prefix("/api/doc/")
        .or_else(|| path.strip_prefix("/doc/"))?;
    let id = rest.strip_suffix("/edit").unwrap_or(rest);
    (!id.is_empty() && !id.contains('/')).then_some(id)
}

/// Read the record's current version token from its edit form. The version is
/// mtime-derived, so it must be fetched immediately before the POST.
fn current_version(server: &Server, id: &str) -> String {
    let path = format!("/doc/{id}/edit");
    let reply = server.request("GET", &path, None, None);
    assert_eq!(reply.status, 200, "edit page for {id}");
    let text = String::from_utf8_lossy(&reply.body);
    const MARKER: &str = "name=\"version\" value=\"";
    let start = text
        .find(MARKER)
        .unwrap_or_else(|| panic!("edit page for {id} has no version field"))
        + MARKER.len();
    let value: String = text[start..]
        .chars()
        .take_while(|character| *character != '"')
        .collect();
    assert!(
        !value.is_empty() && !value.contains("{{"),
        "edit page for {id} exposes a version token"
    );
    value
}

fn observe_http(
    case: &Case,
    server: &Server,
    seed: &Path,
    bind: &str,
) -> BTreeMap<String, Vec<u8>> {
    let request = case
        .request
        .as_ref()
        .unwrap_or_else(|| panic!("{}: http case without a request", case.id));
    let body = request.body.as_deref().map(|body| {
        if body.contains(VERSION_PLACEHOLDER) {
            let id = request_target_id(&request.path).unwrap_or_else(|| {
                panic!(
                    "{}: a body with {VERSION_PLACEHOLDER} needs a document route",
                    case.id
                )
            });
            body.replace(VERSION_PLACEHOLDER, &current_version(server, id))
        } else {
            body.to_string()
        }
    });
    let reply = server.request(
        &request.method,
        &request.path,
        body.as_deref(),
        request.content_type.as_deref(),
    );
    let normaliser = Normaliser::new(seed, bind);
    let body = normaliser.apply(&reply.body);
    let mut observed = BTreeMap::new();
    // A version token is mtime-derived and its digit length can change between
    // runs, so the raw Content-Length of a response that echoes it is volatile.
    // Record a stable placeholder instead of a length that would flake.
    let body_echoes_version = body
        .windows(VERSION.len())
        .any(|window| window == VERSION.as_bytes());
    let headers: Vec<(String, String)> = reply
        .headers
        .into_iter()
        .map(|(name, value)| {
            if body_echoes_version && name.eq_ignore_ascii_case("content-length") {
                (name, LENGTH.to_string())
            } else {
                (name, value)
            }
        })
        .collect();
    let http = serde_json::json!({
        "status": reply.status,
        "headers": headers,
    });
    observed.insert("http.json".to_string(), format!("{http}\n").into_bytes());
    for artifact in &case.artifacts {
        match artifact.as_str() {
            "page.html" | "payload.json" | "body.txt" => {
                observed.insert(artifact.clone(), body.clone());
            }
            "http.json" => {}
            other => panic!("{}: HTTP case cannot produce {other}", case.id),
        }
    }
    expected_subset(case, observed)
}

fn write_case(directory: &Path, observed: &BTreeMap<String, Vec<u8>>) {
    let _ = fs::remove_dir_all(directory);
    fs::create_dir_all(directory).expect("create case directory");
    for (name, bytes) in observed {
        fs::write(directory.join(name), bytes).expect("write expected artifact");
    }
}

/// Whether this case renders a descriptor-driven document HTML page.
fn document_html_case(case: &Case) -> bool {
    matches!(
        case.renderer.as_deref(),
        Some(
            "list_page"
                | "view_page"
                | "edit_page"
                | "edit_page_glossary"
                | "edit_page_ui_spec"
                | "edit_page_ui_mockup"
                | "edit_page_business_rule"
                | "edit_page_audit_record"
                | "edit_page_tech_spec"
                | "edit_page_use_case"
                | "document_detail_page"
                | "document_edit_page"
                | "document_conflict_page"
                | "document_not_found_page"
        )
    )
}

/// Compare one expected directory against an observation. Public so the
/// replay suite can prove that a one-byte change fails.
pub fn compare_case(
    directory: &Path,
    observed: &BTreeMap<String, Vec<u8>>,
    case: &Case,
) -> Result<(), String> {
    let mut expected_files = Vec::new();
    let entries = fs::read_dir(directory).map_err(|error| {
        format!(
            "missing expected directory {}: {error}",
            directory.display()
        )
    })?;
    for entry in entries {
        let entry = entry.map_err(|error| format!("read expected entry: {error}"))?;
        if entry.path().is_file() {
            expected_files.push(entry.file_name().to_string_lossy().to_string());
        }
    }
    expected_files.sort();
    let mut declared: Vec<String> = case.artifacts.clone();
    declared.sort();
    if expected_files != declared {
        return Err(format!(
            "artifact set differs: expected {expected_files:?}, manifest declares {declared:?}"
        ));
    }
    for (name, actual) in observed {
        let path = directory.join(name);
        let expected =
            fs::read(&path).map_err(|error| format!("read {}: {error}", path.display()))?;
        // Version tokens come from file mtimes, which a checkout does not
        // preserve. Normalise both sides so the replay compares content, not
        // the working tree's timestamps. Files written by a capture already
        // carry `{{VERSION}}`; the two normalisations are idempotent.
        let expected_text = normalise_versions(&String::from_utf8_lossy(&expected));
        let actual_text = normalise_versions(&String::from_utf8_lossy(actual));
        if name == "page.html" && document_html_case(case) {
            // Document HTML is compared semantically (TC-184); backlog and
            // decision pages stay byte-for-byte above.
            crate::semantic_html::compare_document_html(
                &case.id,
                &expected_text,
                &actual_text,
                &crate::semantic_html::Allowlist::empty(),
            )?;
            continue;
        }
        if expected_text.as_bytes() != actual_text.as_bytes() {
            return Err(first_difference(
                name,
                expected_text.as_bytes(),
                actual_text.as_bytes(),
            ));
        }
    }
    Ok(())
}

fn first_difference(name: &str, expected: &[u8], actual: &[u8]) -> String {
    let expected_text = String::from_utf8_lossy(expected);
    let actual_text = String::from_utf8_lossy(actual);
    for (index, (left, right)) in expected_text.lines().zip(actual_text.lines()).enumerate() {
        if left != right {
            return format!(
                "{name} differs at line {}:\n  expected: {left}\n  actual:   {right}",
                index + 1
            );
        }
    }
    format!(
        "{name} differs in length: expected {} bytes, actual {} bytes",
        expected.len(),
        actual.len()
    )
}

/// A browser process serving the committed seed project on a free loopback port.
pub struct Server {
    child: Child,
    port: u16,
}

impl Server {
    pub fn start(project: &Path) -> Server {
        for _ in 0..10 {
            let port = free_port();
            let mut command = Command::new(env!("CARGO_BIN_EXE_papyrus"));
            command
                .arg("--dir")
                .arg(project)
                .arg("browser")
                .arg("--port")
                .arg(port.to_string())
                .arg("--no-open")
                .stdout(Stdio::piped())
                .stderr(Stdio::null());
            let mut child = command.spawn().expect("start browser");
            let stdout = child.stdout.take().expect("browser stdout");
            let mut reader = BufReader::new(stdout);
            let mut startup = String::new();
            let _ = reader.read_line(&mut startup);
            if !startup.trim().is_empty() {
                return Server { child, port };
            }
            let _ = child.wait();
        }
        panic!("browser did not start");
    }

    pub fn bind_note(&self) -> String {
        format!("127.0.0.1:{}", self.port)
    }

    fn request(
        &self,
        method: &str,
        path: &str,
        body: Option<&str>,
        content_type: Option<&str>,
    ) -> Reply {
        let mut stream = TcpStream::connect(("127.0.0.1", self.port)).expect("connect to browser");
        let mut request = format!(
            "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n",
            self.port
        );
        if let Some(body) = body {
            let content_type =
                content_type.unwrap_or("application/x-www-form-urlencoded; charset=utf-8");
            request.push_str(&format!("Content-Type: {content_type}\r\n"));
            request.push_str(&format!("Content-Length: {}\r\n", body.len()));
        }
        request.push_str("\r\n");
        if let Some(body) = body {
            request.push_str(body);
        }
        stream.write_all(request.as_bytes()).expect("write request");
        stream.flush().expect("flush request");
        let mut raw = Vec::new();
        stream.read_to_end(&mut raw).expect("read response");
        parse_reply(&raw)
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

struct Reply {
    status: u16,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

fn parse_reply(raw: &[u8]) -> Reply {
    let split = raw
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .expect("response head terminator");
    let head = String::from_utf8_lossy(&raw[..split]).to_string();
    let body = raw[split + 4..].to_vec();
    let mut lines = head.split("\r\n");
    let status_line = lines.next().expect("status line");
    let status = status_line
        .split_whitespace()
        .nth(1)
        .expect("status code")
        .parse::<u16>()
        .expect("numeric status");
    let headers = lines
        .filter(|line| !line.is_empty())
        .map(|line| {
            let (name, value) = line.split_once(':').expect("header separator");
            (name.to_string(), value.trim().to_string())
        })
        .collect();
    Reply {
        status,
        headers,
        body,
    }
}

fn free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind free port");
    let port = listener.local_addr().expect("local address").port();
    drop(listener);
    port
}
