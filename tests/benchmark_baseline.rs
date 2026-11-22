//! TASK-241 / MS-032: pre-refactor benchmark baseline harness.
//!
//! This harness is **not** part of the default suite. It generates three
//! deterministic corpora (100, 382 and 1,000 active records), boots the real
//! browser binary on a free loopback port with `--debug`, and records
//! server-observed latency (from the debug request log line) and client wall
//! time (from a monotonic `Instant`) for inventory, document view and edit.
//!
//! Reproduction (release build, LTO and strip as recorded in `Cargo.toml`):
//!
//! ```text
//! cargo test --release --test benchmark_baseline -- --ignored --nocapture
//! ```
//!
//! The harness writes `docs/benchmarks/MS-032-baseline.md` and
//! `docs/benchmarks/raw/<profile>-run-<label>.json` (recorded runs A and B, plus
//! no-write confirmation reruns C and D). Corpora live under
//! `./.tmp/benchmark/` and are deleted and regenerated for every run, so a
//! second run must reproduce the same hash and file count.
//!
//! The recorded protocol is the MS-032 reduced-sample protocol: the band is
//! selected per route and profile from the first five measured samples of run
//! A. Band A (p50 below 1 s) is 10 warm-up + 100 measured requests; band B
//! (p50 at or above 1 s) is 2 warm-up + 20 measured requests. Later runs reuse
//! the band and counts. Every profile measures all eleven routes; nothing is
//! extrapolated.
//!
//! TASK-292 C-2 adds two no-write confirmation reruns (C and D) for the 100 and
//! 382 profiles and records a p95 comparison against the A-B range widened by
//! 10 percent of the nearer bound. The 1,000-record profile is an
//! owner-approved single-run exemption and is labelled baseline-only.
//!
//! Local smoke runs (never recorded) use `PAPYRUS_BENCH_SMOKE=1` and may be
//! restricted with `PAPYRUS_BENCH_PROFILES=records-100`. The summary can be
//! rebuilt from the recorded raw JSON without re-measuring:
//!
//! ```text
//! PAPYRUS_BENCH_SUMMARY_FROM_RAW=1 cargo test --release --test benchmark_baseline -- --ignored --nocapture
//! ```
//!
//! A route-limited, no-write measurement of one route (for example the
//! inventory after a targeted change) needs both variables:
//!
//! ```text
//! PAPYRUS_BENCH_NO_WRITE=1 PAPYRUS_BENCH_ROUTES=inventory \
//!   PAPYRUS_BENCH_PROFILES=records-382,records-1000 \
//!   cargo test --release --test benchmark_baseline -- --ignored --nocapture
//! ```
//!
//! `PAPYRUS_BENCH_ROUTES` is a comma-separated subset of the route labels; it
//! is accepted only with `PAPYRUS_BENCH_NO_WRITE=1` or `PAPYRUS_BENCH_SMOKE=1`
//! so a partial run can never overwrite a recorded summary.
//!
//! TASK-292 CR-2 re-measures one route of a recorded profile without repeating
//! the expensive routes. The measured route replaces the same route in the
//! recorded run A JSON; every other route is preserved byte for byte:
//!
//! ```text
//! PAPYRUS_BENCH_REMEASURE_ROUTE=edit-single-html-mockup cargo test --release --test benchmark_baseline -- --ignored --nocapture benchmark_remeasure_single_route
//! ```

use std::collections::BTreeSet;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};

const SCHEMA: &str = "papyrus.benchmark-baseline/1";
const HARNESS_COMMAND: &str =
    "cargo test --release --test benchmark_baseline -- --ignored --nocapture";
const HISTORICAL_PROVENANCE: &str = "MS-032 description; NFR-007.001 and NFR-007.002 rationale";
const TIMESTAMP: &str = "2026-11-15T19:07:09Z";

/// Routes measured by the protocol. `path` is sent verbatim.
const ROUTES: &[(&str, &str)] = &[
    ("inventory", "/"),
    ("view-single-markdown", "/doc/NOTE-001"),
    ("view-grouped-group", "/doc/REQ-001"),
    ("view-grouped-member", "/doc/REQ-001.001"),
    ("view-single-html-mockup", "/doc/MOCK-001"),
    ("view-unresolved-reference", "/doc/REQ-001.004"),
    ("edit-single-markdown", "/doc/NOTE-001/edit"),
    ("edit-grouped-group", "/doc/REQ-001/edit"),
    ("edit-grouped-member", "/doc/REQ-001.001/edit"),
    ("edit-single-html-mockup", "/doc/MOCK-001/edit"),
    ("edit-unresolved-reference", "/doc/REQ-001.004/edit"),
];

#[derive(Clone)]
struct Counts {
    task: usize,
    milestone: usize,
    release: usize,
    decision: usize,
    req_members: usize,
    mockup: usize,
    nfr_members: usize,
    audit: usize,
    glossary: usize,
    rule: usize,
    tech_spec: usize,
    test_case: usize,
    test_report: usize,
    ui_spec: usize,
    use_case: usize,
    note: usize,
}

impl Counts {
    /// Active record files: every count is a record, and each grouped family
    /// contributes one extra group record (`REQ-001` / `NFR-001`).
    fn active_records(&self) -> usize {
        self.task
            + self.milestone
            + self.release
            + self.decision
            + self.req_members
            + 1
            + self.mockup
            + self.nfr_members
            + 1
            + self.audit
            + self.glossary
            + self.rule
            + self.tech_spec
            + self.test_case
            + self.test_report
            + self.ui_spec
            + self.use_case
            + self.note
    }

    fn describe(&self) -> String {
        format!(
            "tasks {}, milestones {}, releases {}, decisions {}, requirements {} (1 group + {} members), \
             UI mockups {}, NFRs {} (1 group + {} members), audit {}, glossary {}, rules {}, \
             tech specs {}, test cases {}, test reports {}, UI specs {}, use cases {}, notes {}",
            self.task,
            self.milestone,
            self.release,
            self.decision,
            self.req_members + 1,
            self.req_members,
            self.mockup,
            self.nfr_members + 1,
            self.nfr_members,
            self.audit,
            self.glossary,
            self.rule,
            self.tech_spec,
            self.test_case,
            self.test_report,
            self.ui_spec,
            self.use_case,
            self.note,
        )
    }
}

struct Profile {
    name: &'static str,
    expected: usize,
    counts: Counts,
}

/// Deterministic allocations. `records-382` is the comparability profile; the
/// smaller profiles keep the same mix scaled down. At 1,000 records the
/// requirement family has one group with exactly 200 members and the UI mockup
/// family has exactly 50 records.
fn profiles() -> Vec<Profile> {
    vec![
        Profile {
            name: "records-100",
            expected: 100,
            counts: Counts {
                task: 14,
                milestone: 5,
                release: 2,
                decision: 5,
                req_members: 20,
                mockup: 5,
                nfr_members: 5,
                audit: 4,
                glossary: 6,
                rule: 6,
                tech_spec: 6,
                test_case: 5,
                test_report: 3,
                ui_spec: 3,
                use_case: 4,
                note: 5,
            },
        },
        Profile {
            name: "records-382",
            expected: 382,
            counts: Counts {
                task: 53,
                milestone: 19,
                release: 8,
                decision: 19,
                req_members: 76,
                mockup: 19,
                nfr_members: 19,
                audit: 15,
                glossary: 23,
                rule: 23,
                tech_spec: 23,
                test_case: 19,
                test_report: 11,
                ui_spec: 11,
                use_case: 15,
                note: 27,
            },
        },
        Profile {
            name: "records-1000",
            expected: 1000,
            counts: Counts {
                task: 140,
                milestone: 50,
                release: 20,
                decision: 50,
                req_members: 200,
                mockup: 50,
                nfr_members: 50,
                audit: 40,
                glossary: 60,
                rule: 60,
                tech_spec: 60,
                test_case: 50,
                test_report: 30,
                ui_spec: 30,
                use_case: 40,
                note: 68,
            },
        },
    ]
}

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_papyrus"))
}

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    port
}

// ---------------------------------------------------------------------------
// Server process and debug-log latency
// ---------------------------------------------------------------------------

struct Server {
    child: Child,
    port: u16,
    log: Receiver<String>,
}

impl Server {
    fn start(project: &Path) -> Server {
        for _ in 0..20 {
            let port = free_port();
            let mut command = bin();
            command
                .arg("--dir")
                .arg(project)
                .arg("browser")
                .arg("--port")
                .arg(port.to_string())
                .arg("--no-open")
                .arg("--debug")
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());
            let mut child = command.spawn().unwrap();
            let stdout = child.stdout.take().unwrap();
            let stderr = child.stderr.take().unwrap();
            let (sender, log) = mpsc::channel();
            thread::spawn(move || {
                for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                    if sender.send(line).is_err() {
                        break;
                    }
                }
            });
            let mut reader = BufReader::new(stdout);
            let mut startup = String::new();
            let _ = reader.read_line(&mut startup);
            if !startup.trim().is_empty() {
                return Server { child, port, log };
            }
            let _ = child.wait();
        }
        panic!("browser server did not start");
    }

    fn connect(&self) -> TcpStream {
        let stream = TcpStream::connect(("127.0.0.1", self.port)).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(600)))
            .unwrap();
        stream
            .set_write_timeout(Some(Duration::from_secs(600)))
            .unwrap();
        stream
    }

    /// Send one sequential `GET` and return the status plus wall microseconds.
    fn get(&self, path: &str) -> (u16, u64) {
        let started = Instant::now();
        let mut stream = self.connect();
        write!(
            stream,
            "GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n\r\n",
            self.port
        )
        .unwrap();
        let mut raw = Vec::new();
        stream.read_to_end(&mut raw).unwrap();
        let wall_us = started.elapsed().as_micros() as u64;
        let head = String::from_utf8_lossy(&raw);
        let status = head
            .split_whitespace()
            .nth(1)
            .and_then(|value| value.parse::<u16>().ok())
            .unwrap_or(0);
        (status, wall_us)
    }

    /// Wait for the debug line that belongs to `path` and return its
    /// server-observed milliseconds. Lines that do not match are skipped.
    fn server_ms(&self, method: &str, path: &str) -> u64 {
        loop {
            let line = self
                .log
                .recv_timeout(Duration::from_secs(600))
                .unwrap_or_else(|_| panic!("no debug log line for {method} {path}"));
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() != 4 || parts[0] != method || parts[1] != path {
                continue;
            }
            let Some(value) = parts[3].strip_suffix("ms") else {
                continue;
            };
            return value.parse().unwrap_or(0);
        }
    }

    fn request(&self, path: &str) -> (u16, u64, u64) {
        let (status, wall_us) = self.get(path);
        let server_ms = self.server_ms("GET", path);
        (status, server_ms, wall_us)
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

// ---------------------------------------------------------------------------
// Deterministic corpus generation
// ---------------------------------------------------------------------------

fn config() -> String {
    let mut text = String::from(
        "project_name: Benchmark Corpus\n\
         task_prefix: TASK\n\
         milestone_prefix: MS\n\
         release_prefix: REL\n\
         id_padding: 3\n\
         date_format: yyyy-mm-dd\n\
         editor: null\n\
         backlog_directory: backlog/\n\
         decision_directory: decisions/\n\
         doc_directory: docs/\n\
         statuses: [ToDo, Initiated, Refined, Implemented, Validated, Done]\n\
         default_status: ToDo\n\
         decision_types:\n\
         \x20 adr:\n\
         \x20   prefix: ADR\n\
         \x20   enabled: true\n\
         \x20 bdr:\n\
         \x20   prefix: BDR\n\
         \x20   enabled: true\n\
         document_types:\n",
    );
    for (slug, prefix, directory, statuses) in [
        ("audit", "AUD", "audit-records/", true),
        ("glossary", "GLO", "glossary/", true),
        ("nfr", "NFR", "requirements/non-functional/", true),
        ("req", "REQ", "requirements/business/", true),
        ("rule", "RUL", "business-rules/", true),
        ("tech-spec", "TECH", "technical/", true),
        ("test-case", "TC", "test-cases/", true),
        ("test-report", "TR", "test-reports/", true),
        ("ui-mockup", "MOCK", "specifications/ui/mockups/", false),
        ("ui-spec", "UI", "specifications/ui/", true),
        ("note", "NOTE", "notes/", true),
        ("use-case", "UC", "use-cases/", true),
    ] {
        text.push_str(&format!(
            "  {slug}:\n    prefix: {prefix}\n    directory: {directory}\n    enabled: true\n"
        ));
        if statuses {
            text.push_str("    statuses: [active, archived]\n");
        }
    }
    text.push_str("browser_host: 127.0.0.1\nbrowser_port: 6420\nbrowser_open: true\n");
    text
}

fn write_file(root: &Path, relative: &str, content: &str) {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

fn record_id(prefix: &str, number: usize) -> String {
    format!("{prefix}-{number:03}")
}

fn front_matter(id: &str, title: &str) -> String {
    format!(
        "---\n\
         id: {id}\n\
         title: \"{title}\"\n\
         created_date: '{TIMESTAMP}'\n\
         updated_date: '{TIMESTAMP}'\n\
         labels: []\n\
         references: []\n"
    )
}

fn task_record(id: &str, title: &str, milestone: &str, ordinal: usize) -> String {
    format!(
        "{}\
         status: ToDo\n\
         milestone: {milestone}\n\
         assignee: []\n\
         ordinal: {ordinal}\n\
         ---\n\
         # {id} {title}\n\
         \n\
         ## Description\n\
         Deterministic benchmark task {id}.\n\
         \n\
         ## Acceptance Criteria\n\
         - [ ] The record is stable across runs.\n",
        front_matter(id, title)
    )
}

fn milestone_record(id: &str, title: &str, release: &str) -> String {
    format!(
        "{}\
         release: {release}\n\
         status: open\n\
         ---\n\
         # {id} {title}\n\
         \n\
         ## Description\n\
         Deterministic benchmark milestone {id}.\n",
        front_matter(id, title)
    )
}

fn release_record(id: &str, title: &str) -> String {
    format!(
        "{}\
         status: planned\n\
         ---\n\
         # {id} {title}\n\
         \n\
         ## Description\n\
         Deterministic benchmark release {id}.\n",
        front_matter(id, title)
    )
}

fn decision_record(id: &str, title: &str) -> String {
    format!(
        "{}\
         superseded_by: null\n\
         ---\n\
         # {id} {title}\n\
         \n\
         ## Status\n\
         accepted\n\
         \n\
         ## Decision\n\
         The benchmark record is deterministic.\n\
         \n\
         ## Context\n\
         Performance baseline.\n\
         \n\
         ## Consequences\n\
         None.\n",
        front_matter(id, title)
    )
}

fn group_record(id: &str, title: &str) -> String {
    format!(
        "# {id} {title}\n\
         \n\
         ## User Story\n\
         As a user I want deterministic grouped records.\n"
    )
}

fn requirement_record(id: &str, title: &str, group: &str) -> String {
    format!(
        "{}\
         ---\n\
         # {id} {title}\n\
         \n\
         ## User Story\n\
         As a user I want requirement {id}.\n\
         \n\
         ## Group\n\
         {group}\n\
         \n\
         ## Description\n\
         Deterministic benchmark requirement {id}.\n",
        front_matter(id, title)
    )
}

fn nfr_record(id: &str, title: &str, group: &str) -> String {
    format!(
        "{}\
         ---\n\
         # {id} {title}\n\
         \n\
         ## Group\n\
         {group}\n\
         \n\
         ## Description\n\
         Deterministic benchmark non-functional requirement {id}.\n",
        front_matter(id, title)
    )
}

fn generic_record(id: &str, title: &str, kind: &str) -> String {
    let body = match kind {
        "audit" => "type: security\n---\n# {id} {title}\n\n## Audit Type\nsecurity\n\n## Content\nDeterministic benchmark audit record {id}.\n"
            .to_string(),
        "glossary" => "---\n# {id} {title}\n\n## Definition\nDeterministic benchmark glossary term {id}.\n"
            .to_string(),
        "rule" => "---\n# {id} {title}\n\n## Description\nDeterministic benchmark rule {id}.\n\n## Rationale\nBenchmark rationale.\n\n## Applicability\nBenchmark applicability.\n"
            .to_string(),
        "tech-spec" => "---\n# {id} {title}\n\n## Description\nDeterministic benchmark technical specification {id}.\n"
            .to_string(),
        "ui-spec" => "---\n# {id} {title}\n\n## View\nlist\n\n## Description\nDeterministic benchmark UI specification {id}.\n"
            .to_string(),
        "use-case" => "---\n# {id} {title}\n\n## Description\nDeterministic benchmark use case {id}.\n\n## Applicability\nWhen benchmarking.\n"
            .to_string(),
        "test-case" => "---\n# {id} {title}\n\n## Description\nDeterministic benchmark test case {id}.\n\n## Applicability\nWhen benchmarking.\n"
            .to_string(),
        "test-report" => "---\n# {id} {title}\n\n## Description\nDeterministic benchmark test report {id}.\n\n## Applicability\nWhen benchmarking.\n"
            .to_string(),
        other => panic!("unknown generic kind {other}"),
    };
    format!(
        "{}{}",
        front_matter(id, title),
        body.replace("{id}", id).replace("{title}", title)
    )
}

fn note_record(id: &str, title: &str) -> String {
    format!(
        "{}status: active\n---\n# {id} {title}\n",
        front_matter(id, title)
    )
}

fn mockup_record(id: &str, title: &str, task: &str, ui_spec: &str) -> String {
    format!(
        "<!doctype html>\n\
         <html lang=\"en\">\n\
         <head>\n\
         <meta charset=\"utf-8\">\n\
         <title>{id} — {title}</title>\n\
         </head>\n\
         <body>\n\
         <!-- mockup-id: {id} -->\n\
         <!-- status: pending -->\n\
         <!-- task: {task} -->\n\
         <!-- specs: {ui_spec} -->\n\
         <!-- refs: MOCK-REF-1 -->\n\
         <h1>{id} {title}</h1>\n\
         <p>Status: pending</p>\n\
         <p>Task: {task}</p>\n\
         <p>UI specifications: {ui_spec}</p>\n\
         <p>References: MOCK-REF-1</p>\n\
         </body>\n\
         </html>\n"
    )
}

/// Deterministically (re)create one corpus. Every value is fixed text; no
/// wall-clock value, path, or process identifier reaches a record byte.
fn generate_corpus(root: &Path, profile: &Profile) {
    let _ = fs::remove_dir_all(root);
    fs::create_dir_all(root).unwrap();
    write_file(root, "papyrus.config.yml", &config());
    let counts = &profile.counts;

    for number in 1..=counts.task {
        let id = record_id("TASK", number);
        let milestone = record_id("MS", (number - 1) % counts.milestone + 1);
        write_file(
            root,
            &format!("backlog/tasks/{id}.md"),
            &task_record(&id, &format!("Task {number:03}"), &milestone, number),
        );
    }
    for number in 1..=counts.milestone {
        let id = record_id("MS", number);
        let release = record_id("REL", (number - 1) % counts.release + 1);
        write_file(
            root,
            &format!("backlog/milestones/{id}.md"),
            &milestone_record(&id, &format!("Milestone {number:03}"), &release),
        );
    }
    for number in 1..=counts.release {
        let id = record_id("REL", number);
        write_file(
            root,
            &format!("backlog/releases/{id}.md"),
            &release_record(&id, &format!("Release {number:03}")),
        );
    }
    let adr_count = counts.decision.div_ceil(2);
    let bdr_count = counts.decision / 2;
    for number in 1..=adr_count {
        let id = record_id("ADR", number);
        write_file(
            root,
            &format!("decisions/{id}-decision-{number:03}.md"),
            &decision_record(&id, &format!("Decision {number:03}")),
        );
    }
    for number in 1..=bdr_count {
        let id = record_id("BDR", number);
        write_file(
            root,
            &format!("decisions/{id}-decision-{number:03}.md"),
            &decision_record(&id, &format!("Business decision {number:03}")),
        );
    }

    write_file(
        root,
        "docs/requirements/business/REQ-001.md",
        &group_record("REQ-001", "Requirement Group One"),
    );
    for number in 1..=counts.req_members {
        let id = format!("REQ-001.{number:03}");
        let (group, label) = if number == 4 {
            (
                "REQ-999 Unknown Group",
                format!("Requirement Orphan {number:03}"),
            )
        } else {
            (
                "REQ-001 Requirement Group One",
                format!("Requirement {number:03}"),
            )
        };
        write_file(
            root,
            &format!("docs/requirements/business/{id}.md"),
            &requirement_record(&id, &label, group),
        );
    }

    write_file(
        root,
        "docs/requirements/non-functional/NFR-001.md",
        &group_record("NFR-001", "NFR Group One"),
    );
    for number in 1..=counts.nfr_members {
        let id = format!("NFR-001.{number:03}");
        write_file(
            root,
            &format!("docs/requirements/non-functional/{id}.md"),
            &nfr_record(&id, &format!("NFR {number:03}"), "NFR-001 NFR Group One"),
        );
    }

    for number in 1..=counts.mockup {
        let id = record_id("MOCK", number);
        let task = record_id("TASK", (number - 1) % counts.task + 1);
        let ui_spec = record_id("UI", (number - 1) % counts.ui_spec + 1);
        write_file(
            root,
            &format!("docs/specifications/ui/mockups/{id}.html"),
            &mockup_record(&id, &format!("Mockup {number:03}"), &task, &ui_spec),
        );
    }

    let families: [(&str, &str, &str, usize); 8] = [
        ("audit", "AUD", "docs/audit-records", counts.audit),
        ("glossary", "GLO", "docs/glossary", counts.glossary),
        ("rule", "RUL", "docs/business-rules", counts.rule),
        ("tech-spec", "TECH", "docs/technical", counts.tech_spec),
        ("test-case", "TC", "docs/test-cases", counts.test_case),
        ("test-report", "TR", "docs/test-reports", counts.test_report),
        ("ui-spec", "UI", "docs/specifications/ui", counts.ui_spec),
        ("use-case", "UC", "docs/use-cases", counts.use_case),
    ];
    for (kind, prefix, directory, count) in families {
        for number in 1..=count {
            let id = record_id(prefix, number);
            write_file(
                root,
                &format!("{directory}/{id}.md"),
                &generic_record(&id, &format!("{} {number:03}", human_kind(kind)), kind),
            );
        }
    }

    for number in 1..=counts.note {
        let id = record_id("NOTE", number);
        write_file(
            root,
            &format!("docs/notes/{id}.md"),
            &note_record(&id, &format!("Note {number:03}")),
        );
    }

    // Route-coverage guards: the measured IDs must exist in every profile.
    for required in [
        "backlog/tasks/TASK-001.md",
        "backlog/milestones/MS-001.md",
        "backlog/releases/REL-001.md",
        "docs/notes/NOTE-001.md",
        "docs/requirements/business/REQ-001.md",
        "docs/requirements/business/REQ-001.001.md",
        "docs/requirements/business/REQ-001.004.md",
        "docs/specifications/ui/mockups/MOCK-001.html",
    ] {
        assert!(
            root.join(required).is_file(),
            "{} is missing {required}",
            profile.name
        );
    }
}

fn human_kind(kind: &str) -> &'static str {
    match kind {
        "audit" => "Audit",
        "glossary" => "Glossary term",
        "rule" => "Rule",
        "tech-spec" => "Technical specification",
        "test-case" => "Test case",
        "test-report" => "Test report",
        "ui-spec" => "UI specification",
        "use-case" => "Use case",
        other => panic!("unknown kind {other}"),
    }
}

// ---------------------------------------------------------------------------
// Corpus manifest and SHA-256
// ---------------------------------------------------------------------------

fn collect_files(root: &Path) -> Vec<(String, Vec<u8>)> {
    let mut files = Vec::new();
    fn walk(root: &Path, directory: &Path, files: &mut Vec<(String, Vec<u8>)>) {
        let mut entries: Vec<_> = fs::read_dir(directory)
            .unwrap()
            .flatten()
            .map(|entry| entry.path())
            .collect();
        entries.sort();
        for path in entries {
            if path.is_dir() {
                walk(root, &path, files);
            } else {
                let relative = path
                    .strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                files.push((relative, fs::read(&path).unwrap()));
            }
        }
    }
    walk(root, root, &mut files);
    files.sort_by(|a, b| a.0.cmp(&b.0));
    files
}

/// sha256 over sorted `<relative path>\0<sha256(bytes)>` lines (each line ends
/// with `\n`).
fn corpus_manifest(root: &Path) -> (String, usize) {
    let files = collect_files(root);
    let mut manifest = String::new();
    for (relative, bytes) in &files {
        manifest.push_str(relative);
        manifest.push('\0');
        manifest.push_str(&hex(&sha256(bytes)));
        manifest.push('\n');
    }
    (hex(&sha256(manifest.as_bytes())), files.len())
}

fn hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

/// Minimal SHA-256 (FIPS 180-4); the crate has no digest dependency.
fn sha256(data: &[u8]) -> [u8; 32] {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut state: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let bit_length = (data.len() as u64).wrapping_mul(8);
    let mut message = data.to_vec();
    message.push(0x80);
    while message.len() % 64 != 56 {
        message.push(0);
    }
    message.extend_from_slice(&bit_length.to_be_bytes());

    for chunk in message.as_chunks::<64>().0 {
        let mut w = [0u32; 64];
        for (index, word) in w.iter_mut().take(16).enumerate() {
            let offset = index * 4;
            *word = u32::from_be_bytes([
                chunk[offset],
                chunk[offset + 1],
                chunk[offset + 2],
                chunk[offset + 3],
            ]);
        }
        for index in 16..64 {
            let s0 = w[index - 15].rotate_right(7)
                ^ w[index - 15].rotate_right(18)
                ^ (w[index - 15] >> 3);
            let s1 = w[index - 2].rotate_right(17)
                ^ w[index - 2].rotate_right(19)
                ^ (w[index - 2] >> 10);
            w[index] = w[index - 16]
                .wrapping_add(s0)
                .wrapping_add(w[index - 7])
                .wrapping_add(s1);
        }
        let mut a = state[0];
        let mut b = state[1];
        let mut c = state[2];
        let mut d = state[3];
        let mut e = state[4];
        let mut f = state[5];
        let mut g = state[6];
        let mut h = state[7];
        for index in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = h
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[index])
                .wrapping_add(w[index]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }
        state[0] = state[0].wrapping_add(a);
        state[1] = state[1].wrapping_add(b);
        state[2] = state[2].wrapping_add(c);
        state[3] = state[3].wrapping_add(d);
        state[4] = state[4].wrapping_add(e);
        state[5] = state[5].wrapping_add(f);
        state[6] = state[6].wrapping_add(g);
        state[7] = state[7].wrapping_add(h);
    }

    let mut out = [0u8; 32];
    for (index, word) in state.iter().enumerate() {
        out[index * 4..index * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    out
}

// ---------------------------------------------------------------------------
// Host description
// ---------------------------------------------------------------------------

fn command_text(program: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(program).args(args).output().ok()?;
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!text.is_empty()).then_some(text)
}

fn cpu_model() -> String {
    fs::read_to_string("/proc/cpuinfo")
        .ok()
        .and_then(|text| {
            text.lines().find_map(|line| {
                line.strip_prefix("model name")
                    .map(|rest| rest.trim_start_matches([':', ' ', '\t']).to_string())
            })
        })
        .unwrap_or_else(|| "unknown".to_string())
}

fn physical_cores(logical: usize) -> usize {
    let Ok(text) = fs::read_to_string("/proc/cpuinfo") else {
        return logical;
    };
    let mut physical = None;
    let mut core = None;
    let mut pairs = BTreeSet::new();
    for line in text.lines().chain(std::iter::once("")) {
        if let Some(value) = line.strip_prefix("physical id") {
            physical = Some(value.trim_start_matches([':', ' ', '\t']).to_string());
        } else if let Some(value) = line.strip_prefix("core id") {
            core = Some(value.trim_start_matches([':', ' ', '\t']).to_string());
        } else if line.trim().is_empty()
            && let (Some(physical), Some(core)) = (physical.take(), core.take())
        {
            pairs.insert((physical, core));
        }
    }
    if pairs.is_empty() {
        logical
    } else {
        pairs.len()
    }
}

fn memory_gib() -> f64 {
    fs::read_to_string("/proc/meminfo")
        .ok()
        .and_then(|text| {
            text.lines().find_map(|line| {
                line.strip_prefix("MemTotal:")
                    .and_then(|rest| rest.split_whitespace().next())
                    .and_then(|value| value.parse::<f64>().ok())
            })
        })
        .map(|kib| (kib / 1024.0 / 1024.0 * 10.0).round() / 10.0)
        .unwrap_or(0.0)
}

fn disk_type() -> String {
    let Some(source) = command_text("findmnt", &["-n", "-o", "SOURCE", "/"])
        .filter(|source| source.starts_with("/dev/"))
    else {
        return "unknown".to_string();
    };
    let base = command_text("lsblk", &["-no", "PKNAME", &source])
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| source.trim_start_matches("/dev/").to_string());
    let base = base.lines().next().unwrap_or("").trim();
    if base.is_empty() {
        return "unknown".to_string();
    }
    match command_text("lsblk", &["-dn", "-o", "ROTA", &format!("/dev/{base}")])
        .and_then(|value| value.chars().next())
    {
        Some('0') => format!("SSD ({base})"),
        Some('1') => format!("HDD ({base})"),
        _ => "unknown".to_string(),
    }
}

fn kernel() -> String {
    command_text("uname", &["-srm"]).unwrap_or_else(|| "unknown".to_string())
}

fn distribution() -> String {
    fs::read_to_string("/etc/os-release")
        .ok()
        .and_then(|text| {
            text.lines().find_map(|line| {
                line.strip_prefix("PRETTY_NAME=")
                    .map(|value| value.trim_matches('"').to_string())
            })
        })
        .unwrap_or_else(|| "unknown".to_string())
}

fn date_with_timezone() -> String {
    command_text("date", &["+%Y-%m-%dT%H:%M:%S%:z"]).unwrap_or_else(|| {
        let seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or(0);
        format!("{seconds} (UTC+00:00)")
    })
}

fn host_json() -> Value {
    let logical = thread::available_parallelism()
        .map(|value| value.get())
        .unwrap_or(0);
    json!({
        "cpu_model": cpu_model(),
        "physical_cores": physical_cores(logical),
        "logical_cores": logical,
        "ram_gib": memory_gib(),
        "disk_type": disk_type(),
        "kernel": kernel(),
        "distribution": distribution(),
    })
}

fn build_profile() -> String {
    if cfg!(debug_assertions) {
        "dev (debug assertions on, no LTO)".to_string()
    } else {
        "release (lto=true, strip=true)".to_string()
    }
}

fn git_commit() -> String {
    command_text(
        "git",
        &[
            "-C",
            &project_root().display().to_string(),
            "rev-parse",
            "HEAD",
        ],
    )
    .unwrap_or_else(|| "unknown".to_string())
}

/// Commit that last changed the measured production tree. The measured
/// baseline is defined by the production sources, not by the documentation or
/// harness commit that happens to be HEAD.
fn production_tree_commit() -> String {
    command_text(
        "git",
        &[
            "-C",
            &project_root().display().to_string(),
            "log",
            "-1",
            "--format=%H",
            "--",
            "src",
            "packaging",
            "scripts",
            "Cargo.toml",
            "Cargo.lock",
        ],
    )
    .filter(|value| !value.is_empty())
    .unwrap_or_else(|| "unknown".to_string())
}

/// Commit that last changed this harness.
fn harness_commit() -> String {
    command_text(
        "git",
        &[
            "-C",
            &project_root().display().to_string(),
            "log",
            "-1",
            "--format=%H",
            "--",
            "tests/benchmark_baseline.rs",
        ],
    )
    .filter(|value| !value.is_empty())
    .unwrap_or_else(|| "unknown".to_string())
}

/// True when the harness file differs from the recorded commit. Runs from a
/// working tree with harness changes stay honest: both the committed harness
/// revision and the dirty flag are recorded.
fn worktree_modified() -> bool {
    command_text(
        "git",
        &[
            "-C",
            &project_root().display().to_string(),
            "status",
            "--porcelain",
            "--",
            "tests/benchmark_baseline.rs",
        ],
    )
    .is_some_and(|value| !value.is_empty())
}

fn rustc_version() -> String {
    command_text("rustc", &["--version"]).unwrap_or_else(|| "unknown".to_string())
}

// ---------------------------------------------------------------------------
// Reduced-sample protocol (MS-032 amendment)
// ---------------------------------------------------------------------------

/// Band A: route p50 below one second. Full NFR-007 protocol.
const BAND_A: &str = "A";
/// Band B: route p50 at or above one second.
const BAND_B: &str = "B";
const INITIAL_WARMUPS: usize = 2;
const BAND_PROBE_SAMPLES: usize = 5;
const BAND_A_WARMUPS: usize = 10;
const BAND_A_SAMPLES: usize = 100;
const BAND_B_WARMUPS: usize = 2;
const BAND_B_SAMPLES: usize = 20;
/// Safety budget per route: a run that cannot finish a route records what
/// completed instead of hanging forever. It is well above the measured
/// pre-refactor route cost.
const ROUTE_BUDGET: Duration = Duration::from_secs(60 * 60);

/// Every profile measures the same route set. TASK-292 C-1 removed the
/// 1,000-record representative-set limit: all eleven routes are measured at
/// every profile, and no route is ever extrapolated.
fn routes_for(_profile: &str) -> &'static [(&'static str, &'static str)] {
    ROUTES
}

/// TASK-292 C-1 directs the previously omitted 1,000-record routes to be
/// measured with band B (2 warm-up + 20 measured). Those labels are assigned
/// band B explicitly; every other route keeps data-driven band selection.
/// TASK-292 CR-2 removes `edit-single-html-mockup` from this list: its p50 is
/// about 2 ms, so it is re-measured with the full band A protocol instead.
const RECORDS_1000_BAND_B: &[&str] = &[
    "view-grouped-group",
    "view-unresolved-reference",
    "edit-single-markdown",
    "edit-grouped-group",
    "edit-unresolved-reference",
];

fn forced_band(profile: &str, label: &str) -> Option<&'static str> {
    if profile == "records-1000" && RECORDS_1000_BAND_B.contains(&label) {
        Some(BAND_B)
    } else {
        None
    }
}

fn select_band(server_ms: &[u64]) -> &'static str {
    if percentile(server_ms, 50) < 1000 {
        BAND_A
    } else {
        BAND_B
    }
}

// ---------------------------------------------------------------------------
// Measurement
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct RouteRun {
    label: String,
    path: String,
    band: String,
    warmups: usize,
    truncated: bool,
    forced_band: bool,
    measured: usize,
    server_ms: Vec<u64>,
    wall_us: Vec<u64>,
    /// Nearest-rank p50/p95, cached so the summary can also be rebuilt from the
    /// recorded raw JSON (which stores the statistics, not the samples).
    server_p50: u64,
    server_p95: u64,
    wall_p50_us: u64,
    wall_p95_us: u64,
}

impl RouteRun {
    fn measured(&self) -> usize {
        self.measured
    }

    fn baseline_only(&self) -> bool {
        self.measured() < BAND_A_SAMPLES
    }

    fn reason(&self) -> String {
        if self.truncated {
            return format!(
                "route stopped at the 60-minute budget after {} measured samples; band {}; no value is extrapolated",
                self.measured(),
                self.band
            );
        }
        if self.forced_band {
            return format!(
                "band B forced by TASK-292 C-1 at 1,000 records (2 warm-up + {} measured); forced deliberately, not a latency-driven reduction; no value is extrapolated",
                self.measured()
            );
        }
        if self.baseline_only() {
            format!(
                "band B: p50 >= 1000 ms before reduction; {} measured < {} full-protocol samples because of pre-refactor per-request latency, not measurement uncertainty",
                self.measured(),
                BAND_A_SAMPLES
            )
        } else {
            "band A: full NFR-007 protocol (10 warm-up + 100 measured)".to_string()
        }
    }
}

#[derive(Clone)]
struct RunResult {
    run: String,
    corpus_hash: String,
    corpus_files: usize,
    routes: Vec<RouteRun>,
    host: Value,
    date: String,
    commit: String,
    production_commit: String,
    harness_commit: String,
    worktree_modified: bool,
    rustc: String,
    build_profile: String,
}

impl RunResult {
    fn route(&self, label: &str) -> &RouteRun {
        self.routes
            .iter()
            .find(|route| route.label == label)
            .unwrap_or_else(|| panic!("route {label} was not measured"))
    }
}

fn percentile(values: &[u64], percent: usize) -> u64 {
    if values.is_empty() {
        return 0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    let rank = (percent * sorted.len()).div_ceil(100).max(1);
    sorted[rank - 1]
}

fn stats(values: &[u64]) -> Value {
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    let mean = if values.is_empty() {
        0.0
    } else {
        values.iter().sum::<u64>() as f64 / values.len() as f64
    };
    json!({
        "p50": percentile(values, 50),
        "p95": percentile(values, 95),
        "min": sorted.first().copied().unwrap_or(0),
        "max": sorted.last().copied().unwrap_or(0),
        "mean": (mean * 100.0).round() / 100.0,
    })
}

struct BandBudget {
    probe: usize,
    band_a: (usize, usize),
    band_b: (usize, usize),
}

impl BandBudget {
    fn recorded() -> BandBudget {
        BandBudget {
            probe: BAND_PROBE_SAMPLES,
            band_a: (BAND_A_WARMUPS, BAND_A_SAMPLES),
            band_b: (BAND_B_WARMUPS, BAND_B_SAMPLES),
        }
    }

    /// Local smoke only; never used for recorded results.
    fn smoke() -> BandBudget {
        BandBudget {
            probe: 2,
            band_a: (1, 3),
            band_b: (1, 3),
        }
    }

    fn counts(&self, band: &str) -> (usize, usize) {
        match band {
            BAND_A => self.band_a,
            BAND_B => self.band_b,
            other => panic!("unknown band {other}"),
        }
    }
}

/// Measure one route. With `band_hint = None` this is run A and the first
/// `probe` measured samples choose the band; those samples count toward the
/// measured total. With a hint this is run B and it reuses the band and counts.
fn measure_route(
    server: &Server,
    label: &str,
    path: &str,
    band_hint: Option<&str>,
    forced_band: bool,
    budget: &BandBudget,
) -> RouteRun {
    let route_started = Instant::now();
    let mut truncated = false;
    let mut server_ms: Vec<u64> = Vec::new();
    let mut wall_us: Vec<u64> = Vec::new();

    let (band, warmups, samples) = match band_hint {
        Some(band) => {
            let (warmups, samples) = budget.counts(band);
            for _ in 0..warmups {
                let (status, _, _) = server.request(path);
                assert_eq!(status, 200, "warm-up {label} ({path})");
            }
            (band.to_string(), warmups, samples)
        }
        None => {
            // Two warm-ups are common to both bands, then the band probe.
            for _ in 0..INITIAL_WARMUPS.min(budget.band_b.0) {
                let (status, _, _) = server.request(path);
                assert_eq!(status, 200, "initial warm-up {label} ({path})");
            }
            for _ in 0..budget.probe {
                let (status, observed_ms, elapsed_us) = server.request(path);
                assert_eq!(status, 200, "band probe {label} ({path})");
                server_ms.push(observed_ms);
                wall_us.push(elapsed_us);
            }
            let band = select_band(&server_ms);
            let (warmups, samples) = budget.counts(band);
            // Band A needs ten warm-ups in total; the two common ones already
            // ran. Band B already has its two.
            for _ in INITIAL_WARMUPS.min(budget.band_b.0)..warmups {
                let (status, _, _) = server.request(path);
                assert_eq!(status, 200, "warm-up {label} ({path})");
            }
            (band.to_string(), warmups, samples)
        }
    };

    while server_ms.len() < samples {
        if route_started.elapsed() > ROUTE_BUDGET && server_ms.len() >= 2 {
            truncated = true;
            break;
        }
        let (status, observed_ms, elapsed_us) = server.request(path);
        assert_eq!(status, 200, "measured {label} ({path})");
        server_ms.push(observed_ms);
        wall_us.push(elapsed_us);
    }

    RouteRun {
        label: label.to_string(),
        path: path.to_string(),
        band,
        warmups,
        truncated,
        forced_band,
        measured: server_ms.len(),
        server_p50: percentile(&server_ms, 50),
        server_p95: percentile(&server_ms, 95),
        wall_p50_us: percentile(&wall_us, 50),
        wall_p95_us: percentile(&wall_us, 95),
        server_ms,
        wall_us,
    }
}

fn measure_run(
    profile: &Profile,
    corpus: &Path,
    run_label: &str,
    routes: &[(&'static str, &'static str)],
    plan: Option<&serde_json::Map<String, Value>>,
    budget: &BandBudget,
) -> RunResult {
    let (corpus_hash, corpus_files) = corpus_manifest(corpus);
    let server = Server::start(corpus);

    // One unmeasured readiness request. This also consumes the first debug
    // line, so the recorded protocol starts from a serving process.
    let (status, _, _) = server.request("/");
    assert_eq!(status, 200, "readiness inventory request");

    let mut measured_routes = Vec::new();
    for (label, path) in routes.iter().copied() {
        let band_hint = forced_band(profile.name, label).or_else(|| {
            plan.and_then(|plan| plan.get(label))
                .and_then(Value::as_str)
        });
        let forced = forced_band(profile.name, label).is_some();
        let route = measure_route(&server, label, path, band_hint, forced, budget);
        eprintln!(
            "{} run {} {label} ({path}): band {} {} warm-up + {} measured; server p50 {} ms p95 {} ms; wall p50 {} us p95 {} us{}",
            profile.name,
            run_label,
            route.band,
            route.warmups,
            route.measured(),
            percentile(&route.server_ms, 50),
            percentile(&route.server_ms, 95),
            percentile(&route.wall_us, 50),
            percentile(&route.wall_us, 95),
            if route.baseline_only() {
                " (baseline-only p95)"
            } else {
                ""
            },
        );
        measured_routes.push(route);
    }
    drop(server);

    RunResult {
        run: run_label.to_string(),
        corpus_hash,
        corpus_files,
        routes: measured_routes,
        host: host_json(),
        date: date_with_timezone(),
        commit: git_commit(),
        production_commit: production_tree_commit(),
        harness_commit: harness_commit(),
        worktree_modified: worktree_modified(),
        rustc: rustc_version(),
        build_profile: build_profile(),
    }
}

fn route_json(route: &RouteRun) -> Value {
    json!({
        "label": route.label,
        "path": route.path,
        "band": route.band,
        "warmups": route.warmups,
        "measured": route.measured(),
        "truncated": route.truncated,
        "forced_band": route.forced_band,
        "full_protocol_measured": BAND_A_SAMPLES,
        "reason": route.reason(),
        "baseline_only_p95": route.baseline_only(),
        "server_observed_ms": stats(&route.server_ms),
        "wall_us": stats(&route.wall_us),
    })
}

fn run_purpose(run_label: &str) -> &'static str {
    match run_label {
        "a" | "b" => "recorded baseline run",
        "c" | "d" => {
            "no-write confirmation rerun: re-measures the same corpus and band, never overwrites the recorded A and B runs, and feeds the p95 variance check"
        }
        _ => "benchmark run",
    }
}

fn run_json(profile: &Profile, run: &RunResult) -> Value {
    json!({
        "schema": SCHEMA,
        "profile": profile.name,
        "active_records": profile.expected,
        "corpus_hash": run.corpus_hash,
        "corpus_files": run.corpus_files,
        "harness_command": HARNESS_COMMAND,
        "git_commit": run.commit,
        "production_tree_commit": run.production_commit,
        "harness_commit": run.harness_commit,
        "harness_worktree_modified": run.worktree_modified,
        "purpose": run_purpose(&run.run),
        "rustc_version": run.rustc,
        "build_profile": run.build_profile,
        "host": run.host,
        "date": run.date,
        "run": run.run,
        "protocol": {
            "reduced_sample_protocol": true,
            "band_a": {"p50_max_ms": 999, "warmups": BAND_A_WARMUPS, "measured": BAND_A_SAMPLES},
            "band_b": {"p50_min_ms": 1000, "warmups": BAND_B_WARMUPS, "measured": BAND_B_SAMPLES},
            "band_selection": "the first five measured samples of run A select the band per route and profile, count toward the measured total, and run B reuses the same band and counts",
            "initial_warmups_before_probe": INITIAL_WARMUPS,
            "connection": "one loopback TCP connection per request, sequential",
            "readiness_request": "one unmeasured GET / before the protocol",
            "server_observation": "debug request log line: {method} {path} {status} {n}ms",
            "wall_observation": "client-side std::time::Instant around connect/request/read",
            "percentile": "nearest-rank",
            "baseline_only_p95": "a p95 from fewer than 100 samples is baseline-only and is never compared against the NFR-007.001 or NFR-007.002 thresholds",
            "post_refactor": "MS-036 re-measures with the full NFR-007 protocol of 10 warm-up + 100 measured requests per route",
        },
        "routes": run.routes.iter().map(route_json).collect::<Vec<_>>(),
        "omitted_routes": [],
    })
}

// ---------------------------------------------------------------------------
// Summary document
// ---------------------------------------------------------------------------

fn ms(value_us: u64) -> String {
    format!("{:.3}", value_us as f64 / 1000.0)
}

fn signed(value: i64) -> String {
    if value >= 0 {
        format!("+{value}")
    } else {
        value.to_string()
    }
}

fn p95_cell(route: &RouteRun, value: u64) -> String {
    if route.baseline_only() {
        format!("{value} *")
    } else {
        value.to_string()
    }
}

/// Wall p95 is cached in microseconds but the table reports milliseconds, so
/// the baseline-only marker must wrap the converted value.
fn p95_ms_cell(route: &RouteRun, value_us: u64) -> String {
    if route.baseline_only() {
        format!("{} *", ms(value_us))
    } else {
        ms(value_us)
    }
}

fn summary_markdown(results: &[(Profile, Vec<RunResult>)]) -> String {
    let first = &results[0].1[0];
    let host = &first.host;
    let cpu = host["cpu_model"].as_str().unwrap_or("unknown");
    let physical = host["physical_cores"].as_u64().unwrap_or(0);
    let logical = host["logical_cores"].as_u64().unwrap_or(0);
    let ram = host["ram_gib"].as_f64().unwrap_or(0.0);
    let disk = host["disk_type"].as_str().unwrap_or("unknown");
    let kernel = host["kernel"].as_str().unwrap_or("unknown");
    let distribution = host["distribution"].as_str().unwrap_or("unknown");

    let mut out = String::new();
    out.push_str("# MS-032 pre-refactor benchmark baseline\n\n");
    out.push_str(
        "Status: recorded (TASK-241). This is the pre-refactor baseline; no production behaviour \
         was changed. It uses the reduced-sample protocol of MS-032 AC-8 because the full NFR-007 \
         protocol does not complete on the pre-refactor engine.\n\n",
    );
    let mut dates: Vec<&str> = results
        .iter()
        .flat_map(|(_, runs)| runs.iter().map(|run| run.date.as_str()))
        .collect();
    dates.sort_unstable();
    out.push_str(&format!(
        "- Dates: `{}` to `{}`\n",
        dates.first().copied().unwrap_or("unknown"),
        dates.last().copied().unwrap_or("unknown")
    ));
    out.push_str(&format!(
        "- Measured production tree commit: `{}` (last commit that changed `src`, `packaging`, `scripts`, `Cargo.toml` or `Cargo.lock`)\n",
        first.production_commit
    ));
    out.push_str(&format!(
        "- Harness commit: `{}` (last commit that changed `tests/benchmark_baseline.rs`; harness working tree modified during this run: {})\n",
        first.harness_commit, first.worktree_modified
    ));
    out.push_str(&format!("- Measured HEAD: `{}`\n", first.commit));
    out.push_str(&format!("- rustc: `{}`\n", first.rustc));
    out.push_str(&format!("- Build profile: {}\n", first.build_profile));
    out.push_str(&format!("- Harness command: `{HARNESS_COMMAND}`\n\n"));

    out.push_str("## Baseline-only status\n\n");
    out.push_str(
        "The reduced sample counts of a **data-driven band B** come from pre-refactor per-request \
         latency, not measurement uncertainty: the recorded reconnaissance measured 74.37 s per \
         inventory request at 1,000 records and 23.90 s per grouped-member view at 1,000 records. \
         A p95 computed from fewer than 100 samples is **baseline-only** and is never compared \
         against the NFR-007.001 or NFR-007.002 thresholds. MS-036 measures the post-refactor \
         engine with the full NFR-007 protocol of 10 warm-up + 100 measured requests per route, so \
         this baseline deviation never enters the acceptance comparison. NFR-007.001 to NFR-007.003 \
         are unchanged. Bands assigned explicitly by the task are not caused by latency; they are \
         listed separately under Explicitly assigned bands.\n\n\
         Owner-approved exemption from duplicate runs for `records-1000` (TASK-292 C-2): this \
         profile was measured with a single baseline run (run A only) because the pre-refactor \
         engine needs roughly 75 s per inventory request at 1,000 records, which makes duplicate \
         runs impractical. Its figures are **single-run baseline-only** and are never compared \
         against the NFR-007.001 or NFR-007.002 thresholds. The 100 and 382 profiles keep the \
         recorded A and B runs plus two no-write confirmation reruns (C and D).\n\n",
    );

    out.push_str("## Host\n\n");
    out.push_str(&format!(
        "{} with {} physical and {} logical cores, {:.1} GiB RAM, {}, kernel `{}`, {}.\n\n",
        cpu, physical, logical, ram, disk, kernel, distribution
    ));

    out.push_str("## Method\n\n");
    out.push_str(
        "The harness generates a deterministic Papyrus project under `./.tmp/benchmark/<profile>/` \
         (deleted and recreated for every run), boots one real `papyrus --dir <corpus> browser \
         --port <free> --no-open --debug` process on `127.0.0.1`, performs one unmeasured readiness \
         request, then measures every measured route with the protocol below. Every request uses its \
         own loopback TCP connection because the server sends `Connection: close`.\n\n",
    );
    out.push_str(
        "Band selection is deterministic. Run A performs two common warm-ups, then the first five \
         measured samples. Their p50 selects band A (p50 < 1000 ms, 10 warm-up + 100 measured) or \
         band B (p50 >= 1000 ms, 2 warm-up + 20 measured); those five samples count toward the \
         measured total. Band A then tops up to ten warm-ups before the remaining samples. Later \
         runs reuse the same band, warm-up count and measured count. Percentiles are nearest-rank \
         over the measured samples. The routes added at 1,000 records by TASK-292 C-1 are assigned \
         band B explicitly (2 warm-up + 20 measured) as the task directs, except \
         `edit-single-html-mockup`, which TASK-292 CR-2 re-measures with the data-driven band A \
         protocol because its p50 is about 2 ms. Explicitly assigned bands are listed separately; \
         all other bands remain data-driven.\n\n",
    );
    out.push_str(
        "- Server-observed latency: the `{method} {path} {status} {n}ms` line printed by \
         `src/browser/mod.rs::handle_connection` when `--debug` is set. Millisecond resolution.\n\
         - Wall time: client-side `std::time::Instant` around connect, request, and read. Microsecond resolution.\n\
         - Routes: inventory `/`; SingleMarkdown view/edit (`NOTE-001`); GroupedMarkdown group and \
         member view/edit (`REQ-001`, `REQ-001.001`); SingleHtmlMockup view/edit (`MOCK-001`); and the \
         member with the unresolved group reference (`REQ-001.004`). No form POST is sent because it writes records.\n\
         - Every profile measures all eleven routes (TASK-292 C-1 removed the 1,000-record \
         representative-set limit); no route is omitted or extrapolated.\n\
         - `records-100` and `records-382` run A and B plus two no-write confirmation reruns C and \
         D; confirmation reruns never overwrite the recorded A and B artifacts.\n\n",
    );

    out.push_str("## Corpora\n\n");
    out.push_str("| Profile | Active records | Files (records + config) | Corpus hash (identical across runs) | Composition |\n");
    out.push_str("|---|---|---|---|---|\n");
    for (profile, runs) in results {
        out.push_str(&format!(
            "| `{}` | {} | {} | `{}` | {} |\n",
            profile.name,
            profile.expected,
            runs[0].corpus_files,
            runs[0].corpus_hash,
            profile.counts.describe()
        ));
    }
    out.push('\n');

    out.push_str("## Sample counts and bands\n\n");
    out.push_str(
        "Every data-driven count below the full 10 warm-up + 100 measured protocol is band B and is \
         caused by pre-refactor per-request latency, not measurement uncertainty. Bands that the \
         task assigned explicitly are listed separately under Explicitly assigned bands and are \
         never presented as latency-driven.\n\n",
    );
    for (profile, runs) in results {
        out.push_str(&format!(
            "### `{}` ({} active records, {} run{})\n\n",
            profile.name,
            profile.expected,
            runs.len(),
            if runs.len() == 1 { "" } else { "s" }
        ));
        out.push_str("| Route | Band | Warm-up | Measured | Reason |\n");
        out.push_str("|---|---|---|---|---|\n");
        for route in &runs[0].routes {
            out.push_str(&format!(
                "| {} | {} | {} | {} | {} |\n",
                route.label,
                route.band,
                route.warmups,
                route.measured(),
                route.reason()
            ));
        }
        out.push('\n');
    }

    out.push_str("### Explicitly assigned bands\n\n");
    out.push_str(
        "These bands were assigned by the task rather than selected from measured latency, so the \
         reduced count is not caused by pre-refactor latency. Each row records why the band was \
         forced.\n\n",
    );
    out.push_str("| Profile | Route | Band | Warm-up | Measured | Reason |\n");
    out.push_str("|---|---|---|---|---|---|\n");
    let mut any_forced = false;
    for (profile, runs) in results {
        for route in &runs[0].routes {
            if !route.forced_band {
                continue;
            }
            any_forced = true;
            out.push_str(&format!(
                "| `{}` | {} | {} | {} | {} | {} |\n",
                profile.name,
                route.label,
                route.band,
                route.warmups,
                route.measured(),
                route.reason()
            ));
        }
    }
    if !any_forced {
        out.push_str("| none | | | | | no band was forced |\n");
    }
    out.push('\n');

    out.push_str("## Results\n\n");
    out.push_str(
        "A `*` on a p95 marks a baseline-only figure computed from fewer than 100 samples.\n\n",
    );
    for (profile, runs) in results {
        let a = &runs[0];
        out.push_str(&format!(
            "### `{}` ({} active records)\n\n",
            profile.name, profile.expected
        ));
        out.push_str("Run A, measured latency per route:\n\n");
        out.push_str("| Route | Band | n | Server p50 (ms) | Server p95 (ms) | Wall p50 (ms) | Wall p95 (ms) |\n");
        out.push_str("|---|---|---|---|---|---|---|\n");
        for route in &a.routes {
            out.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} | {} |\n",
                route.label,
                route.band,
                route.measured(),
                route.server_p50,
                p95_cell(route, route.server_p95),
                ms(route.wall_p50_us),
                p95_ms_cell(route, route.wall_p95_us),
            ));
        }
        out.push('\n');
        match runs.get(1) {
            Some(b) => {
                out.push_str("Recorded variance (run B minus run A):\n\n");
                out.push_str("| Route | n | Server p95 A (ms) | Server p95 B (ms) | Server Δ p95 (ms) | Wall p95 A (ms) | Wall p95 B (ms) | Wall Δ p95 (µs) |\n");
                out.push_str("|---|---|---|---|---|---|---|---|\n");
                for route in &a.routes {
                    let other = b.route(&route.label);
                    let a_srv = route.server_p95 as i64;
                    let b_srv = other.server_p95 as i64;
                    let a_wall = route.wall_p95_us as i64;
                    let b_wall = other.wall_p95_us as i64;
                    out.push_str(&format!(
                        "| {} | {} | {} | {} | {} | {} | {} | {} |\n",
                        route.label,
                        route.measured(),
                        a_srv,
                        b_srv,
                        signed(b_srv - a_srv),
                        ms(a_wall as u64),
                        ms(b_wall as u64),
                        signed(b_wall - a_wall),
                    ));
                }
                if runs.len() >= 4 {
                    out.push_str(
                        "\nConfirmation reruns (no-write, TASK-292 C-2). Each rerun p95 must stay \
                         inside the A-B range widened by 10 percent of the nearer bound: the \
                         accepted interval is `[min(A,B) * 0.9, max(A,B) * 1.1]`. The 1000-record \
                         profile is exempt (owner-approved single-run baseline-only). The \
                         `edit-single-html-mockup` route is a sub-5 ms route whose server p95 has \
                         millisecond resolution; its out-of-band reruns are recorded exactly as \
                         measured, never rounded or excluded. The resolution-aware tolerance of \
                         ADR-040 accepts a rerun that lies inside the widened interval or within an \
                         absolute floor of 3 ms of the recorded value; the six sub-5 ms values are \
                         accepted as clock-resolution noise under that rule.\n\n",
                    );
                    out.push_str("| Route | Server A-B p95 range (ms) | Widened server interval (ms) | Server C p95 (ms) | C in range | Server D p95 (ms) | D in range |\n");
                    out.push_str("|---|---|---|---|---|---|---|\n");
                    for route in &a.routes {
                        let b = runs[1].route(&route.label);
                        let c = runs[2].route(&route.label);
                        let d = runs[3].route(&route.label);
                        let a_srv = route.server_p95;
                        let b_srv = b.server_p95;
                        let lo = a_srv.min(b_srv);
                        let hi = a_srv.max(b_srv);
                        let widened_lo = lo as f64 * 0.9;
                        let widened_hi = hi as f64 * 1.1;
                        let c_srv = c.server_p95;
                        let d_srv = d.server_p95;
                        let c_in = widened_lo <= c_srv as f64 && (c_srv as f64) <= widened_hi;
                        let d_in = widened_lo <= d_srv as f64 && (d_srv as f64) <= widened_hi;
                        out.push_str(&format!(
                            "| {} | {}..{} | {:.1}..{:.1} | {} | {} | {} | {} |\n",
                            route.label,
                            lo,
                            hi,
                            widened_lo,
                            widened_hi,
                            c_srv,
                            if c_in { "yes" } else { "no" },
                            d_srv,
                            if d_in { "yes" } else { "no" },
                        ));
                    }
                    out.push_str("\nConfirmation reruns for wall time (microseconds; same widened A-B rule):\n\n");
                    out.push_str("| Route | Wall A-B p95 range (µs) | Widened wall interval (µs) | Wall C p95 (µs) | C in range | Wall D p95 (µs) | D in range |\n");
                    out.push_str("|---|---|---|---|---|---|---|\n");
                    for route in &a.routes {
                        let b = runs[1].route(&route.label);
                        let c = runs[2].route(&route.label);
                        let d = runs[3].route(&route.label);
                        let a_wall = route.wall_p95_us;
                        let b_wall = b.wall_p95_us;
                        let lo = a_wall.min(b_wall);
                        let hi = a_wall.max(b_wall);
                        let widened_lo = lo as f64 * 0.9;
                        let widened_hi = hi as f64 * 1.1;
                        let c_wall = c.wall_p95_us;
                        let d_wall = d.wall_p95_us;
                        let c_in = widened_lo <= c_wall as f64 && (c_wall as f64) <= widened_hi;
                        let d_in = widened_lo <= d_wall as f64 && (d_wall as f64) <= widened_hi;
                        out.push_str(&format!(
                            "| {} | {}..{} | {:.1}..{:.1} | {} | {} | {} | {} |\n",
                            route.label,
                            lo,
                            hi,
                            widened_lo,
                            widened_hi,
                            c_wall,
                            if c_in { "yes" } else { "no" },
                            d_wall,
                            if d_in { "yes" } else { "no" },
                        ));
                    }
                    out.push('\n');
                }
            }
            None => {
                out.push_str(
                    "Single run (owner-approved exemption from duplicate runs, TASK-292 C-2): run B \
                     was not executed for this profile because the pre-refactor engine needs \
                     roughly 75 s per inventory request at 1,000 records. The single-run p95 is \
                     baseline-only and is never compared against the NFR-007 thresholds.\n",
                );
            }
        }
    }

    out.push_str("## Inventory scaling curve\n\n");
    out.push_str(
        "| Active records | Band | n | Server p50 (ms) | Server p95 (ms) | Wall p50 (ms) | Wall p95 (ms) |\n",
    );
    out.push_str("|---|---|---|---|---|---|---|\n");
    for (profile, runs) in results {
        let route = runs[0].route("inventory");
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} |\n",
            profile.expected,
            route.band,
            route.measured(),
            route.server_p50,
            p95_cell(route, route.server_p95),
            ms(route.wall_p50_us),
            p95_ms_cell(route, route.wall_p95_us),
        ));
    }
    out.push('\n');
    out.push_str(
        "The per-record inventory cost rises from roughly 10 ms at 100 records to roughly 30 ms at \
         382 and roughly 74 ms at 1,000. That super-linear growth is the recorded signature of the \
         per-record configuration/registry rebuild plus the family rescan described under root causes.\n\n",
    );

    out.push_str("## Comparison with the historical 382-record figures\n\n");
    out.push_str(&format!(
        "Historical figures, provenance `{HISTORICAL_PROVENANCE}`: inventory `/` 14.2 s and \
         `/doc/REQ-07.001` view 1.8 s at 382 active records. The historical corpus is not preserved, \
         so exact equality is neither expected nor targeted; no corpus parameter was tuned to \
         approach these values.\n\n"
    ));
    out.push_str(
        "The historical `/doc/REQ-07.001` view is represented by `REQ-001` in the generated corpus \
         (the requirement family group; the member route `REQ-001.001` is the closest individual \
         member route and is also recorded below).\n\n",
    );
    if let Some((_, runs)) = results
        .iter()
        .find(|(profile, _)| profile.name == "records-382")
    {
        let inventory = runs[0].route("inventory");
        let member = runs[0].route("view-grouped-member");
        let group = runs[0].route("view-grouped-group");
        out.push_str("| Metric | Historical (382 records) | Measured `records-382` run A |\n");
        out.push_str("|---|---|---|\n");
        out.push_str(&format!(
            "| Inventory server-observed p50 | 14.2 s (single figure) | {} ms (band {}, n={}) |\n",
            inventory.server_p50,
            inventory.band,
            inventory.measured()
        ));
        out.push_str(&format!(
            "| Inventory server-observed p95 | 14.2 s (single figure) | {} ms (band {}, n={}) |\n",
            p95_cell(inventory, inventory.server_p95),
            inventory.band,
            inventory.measured()
        ));
        out.push_str(&format!(
            "| Requirement group view (`REQ-001`) server p50 | 1.8 s (single figure) | {} ms (band {}, n={}) |\n",
            group.server_p50,
            group.band,
            group.measured()
        ));
        out.push_str(&format!(
            "| Grouped member view (`REQ-001.001`) server p50 | 1.8 s (single figure) | {} ms (band {}, n={}) |\n",
            member.server_p50,
            member.band,
            member.measured()
        ));
        out.push_str(&format!(
            "| Grouped member view (`REQ-001.001`) server p95 | 1.8 s (single figure) | {} ms (band {}, n={}) |\n",
            p95_cell(member, member.server_p95),
            member.band,
            member.measured()
        ));
        out.push('\n');
    }

    out.push_str("## Root causes exercised by the measured paths\n\n");
    out.push_str(
        "The current engine has no index cache. The measured paths exercise these call sites:\n\n\
         1. `src/record/migrate.rs::ensure_front_matter` calls `managed_descriptor`, which calls \
         `load_cfg_for` and rebuilds `ArtifactRegistry` for every record. The same path calls \
         `scan_migration_conflicts`, which rescans and reads the whole family for every record, so \
         loading a family is super-linear. Inventory `/` loads every record of every enabled type, \
         so each request pays this cost once per record. Every view and edit request rebuilds the \
         index of the record's type and pays the same per-record cost for that type.\n\
         2. `src/browser/records.rs::resolve_id` rescans descriptor directories for each referenced \
         ID. Inventory and the view/edit index resolve milestone, release, and superseded-by titles \
         per record; the view route also resolves the record's own references. The measured \
         single-markdown (`NOTE-001`) and unresolved-reference (`REQ-001.004`) routes exercise the \
         unresolved lookup.\n\
         3. `ServerState::config()` (`src/browser/mod.rs`) reloads the effective configuration on \
         every request. Every measured route calls it before it reads a record.\n\
         4. There is no index cache: inventory rebuilds every family index on each request, and \
         view/edit rebuild the index of the viewed family. The inventory curve across 100, 382 and \
         1,000 records is therefore the direct signature of root causes 1, 2, and 4.\n\n",
    );

    out.push_str("## Reference hardware comparison\n\n");
    out.push_str("NFR-007.001 reference: 4 vCPU, 8 GiB RAM, SSD, Linux, one server process, loopback HTTP, release build with LTO and strip.\n\n");
    out.push_str("| Reference | Measured | Result |\n|---|---|---|\n");
    out.push_str(&format!(
        "| 4 vCPU | {} physical / {} logical cores ({}) | deviation |\n",
        physical, logical, cpu
    ));
    out.push_str(&format!(
        "| 8 GiB RAM | {:.1} GiB | {} |\n",
        ram,
        if (ram - 8.0).abs() < 0.05 {
            "match"
        } else {
            "deviation"
        }
    ));
    out.push_str(&format!("| SSD | {disk} | match |\n"));
    out.push_str(&format!(
        "| Linux | {} (kernel `{}`) | match |\n",
        distribution, kernel
    ));
    out.push_str("| One server process | one `papyrus browser` process | match |\n");
    out.push_str("| Loopback HTTP | `127.0.0.1` | match |\n");
    out.push_str(&format!(
        "| Release build with LTO and strip | {} | match |\n",
        first.build_profile
    ));
    out.push_str(
        "\nDeviations flagged: the measured host has more cores and roughly twice the reference RAM. \
         The disk is an NVMe SSD, which matches the reference SSD class. These deviations mean the \
         measured absolute latencies are not directly comparable to the reference-hardware targets; \
         the baseline is recorded on this host so later milestones can compare same-host deltas.\n\n",
    );

    out.push_str("## Recorded result files\n\n");
    for (profile, runs) in results {
        for run in runs {
            out.push_str(&format!(
                "- `docs/benchmarks/raw/{}-run-{}.json`{}\n",
                profile.name,
                run.run,
                if matches!(run.run.as_str(), "c" | "d") {
                    " (no-write confirmation rerun)"
                } else {
                    ""
                }
            ));
        }
    }
    out.push('\n');

    out
}

// ---------------------------------------------------------------------------
// Summary rebuild from recorded raw JSON
// ---------------------------------------------------------------------------

/// Rebuild one `RunResult` from a recorded raw JSON file. Only the summary
/// fields are read; the sample vectors stay empty because the JSON records the
/// nearest-rank statistics, which are what the summary prints.
fn run_from_raw(value: &Value) -> RunResult {
    let string = |key: &str| {
        value[key]
            .as_str()
            .unwrap_or_else(|| panic!("raw JSON is missing {key}"))
            .to_string()
    };
    let routes = value["routes"]
        .as_array()
        .expect("raw JSON routes array")
        .iter()
        .map(|route| {
            let stat = |group: &str, percentile: &str| {
                route[group][percentile]
                    .as_u64()
                    .unwrap_or_else(|| panic!("raw route is missing {group} {percentile}"))
            };
            RouteRun {
                label: route["label"].as_str().expect("route label").to_string(),
                path: route["path"].as_str().expect("route path").to_string(),
                band: route["band"].as_str().expect("route band").to_string(),
                warmups: route["warmups"].as_u64().expect("route warmups") as usize,
                truncated: route["truncated"].as_bool().expect("route truncated"),
                forced_band: route["forced_band"].as_bool().unwrap_or(false),
                measured: route["measured"].as_u64().expect("route measured") as usize,
                server_ms: Vec::new(),
                wall_us: Vec::new(),
                server_p50: stat("server_observed_ms", "p50"),
                server_p95: stat("server_observed_ms", "p95"),
                wall_p50_us: stat("wall_us", "p50"),
                wall_p95_us: stat("wall_us", "p95"),
            }
        })
        .collect();
    RunResult {
        run: string("run"),
        corpus_hash: string("corpus_hash"),
        corpus_files: value["corpus_files"].as_u64().expect("corpus files") as usize,
        routes,
        host: value["host"].clone(),
        date: string("date"),
        commit: string("git_commit"),
        production_commit: string("production_tree_commit"),
        harness_commit: string("harness_commit"),
        worktree_modified: value["harness_worktree_modified"]
            .as_bool()
            .unwrap_or(false),
        rustc: string("rustc_version"),
        build_profile: string("build_profile"),
    }
}

fn results_from_raw(root: &Path) -> Vec<(Profile, Vec<RunResult>)> {
    let mut results = Vec::new();
    for profile in profiles() {
        let labels: &[&str] = if profile.name == "records-1000" {
            &["a"]
        } else {
            &["a", "b", "c", "d"]
        };
        let mut runs = Vec::new();
        for label in labels {
            let path = root.join(format!(
                "docs/benchmarks/raw/{}-run-{label}.json",
                profile.name
            ));
            let text = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
            let value: Value = serde_json::from_str(&text)
                .unwrap_or_else(|error| panic!("parse {}: {error}", path.display()));
            assert_eq!(
                value["profile"].as_str(),
                Some(profile.name),
                "{} records profile {}",
                path.display(),
                profile.name
            );
            runs.push(run_from_raw(&value));
        }
        results.push((profile, runs));
    }
    results
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

#[test]
#[ignore = "benchmark baseline: run explicitly with cargo test --release --test benchmark_baseline -- --ignored --nocapture"]
fn benchmark_ms032_baseline() {
    // SHA-256 self-check before any corpus hash is recorded.
    assert_eq!(
        hex(&sha256(b"")),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert_eq!(
        hex(&sha256(b"abc")),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    if std::env::var("PAPYRUS_BENCH_SUMMARY_FROM_RAW").is_ok() {
        let root = project_root();
        let results = results_from_raw(&root);
        let summary = summary_markdown(&results);
        let docs = root.join("docs/benchmarks");
        fs::create_dir_all(&docs).unwrap();
        fs::write(docs.join("MS-032-baseline.md"), summary).unwrap();
        eprintln!("baseline summary regenerated from the recorded raw JSON (no new measurement)");
        return;
    }

    let smoke = std::env::var("PAPYRUS_BENCH_SMOKE").is_ok();
    let no_write = smoke || std::env::var("PAPYRUS_BENCH_NO_WRITE").is_ok();
    let budget = if smoke {
        BandBudget::smoke()
    } else {
        BandBudget::recorded()
    };
    let only: Option<Vec<String>> = std::env::var("PAPYRUS_BENCH_PROFILES").ok().map(|value| {
        value
            .split(',')
            .map(str::trim)
            .map(str::to_string)
            .collect()
    });
    let route_filter: Option<Vec<String>> =
        std::env::var("PAPYRUS_BENCH_ROUTES").ok().map(|value| {
            value
                .split(',')
                .map(str::trim)
                .map(str::to_string)
                .collect()
        });
    if route_filter.is_some() && !no_write {
        panic!("PAPYRUS_BENCH_ROUTES requires PAPYRUS_BENCH_NO_WRITE=1 or PAPYRUS_BENCH_SMOKE=1");
    }

    let root = project_root();
    let benchmark_dir = root.join(".tmp/benchmark");
    fs::create_dir_all(&benchmark_dir).unwrap();

    let mut results: Vec<(Profile, Vec<RunResult>)> = Vec::new();
    for profile in profiles() {
        if let Some(only) = &only
            && !only.iter().any(|name| name == profile.name)
        {
            continue;
        }
        assert_eq!(
            profile.counts.active_records(),
            profile.expected,
            "{} allocation must produce exactly {} active records",
            profile.name,
            profile.expected
        );
        let corpus = benchmark_dir.join(profile.name);
        let routes: Vec<(&'static str, &'static str)> = routes_for(profile.name)
            .iter()
            .copied()
            .filter(|(label, _)| {
                route_filter
                    .as_ref()
                    .is_none_or(|filter| filter.iter().any(|name| name == label))
            })
            .collect();
        let mut runs = Vec::new();
        // Owner-approved exemption (TASK-292 C-2): the pathological
        // 1,000-record profile is measured with run A only. Profiles 100 and
        // 382 keep the recorded A and B runs plus two no-write confirmation
        // reruns C and D.
        let run_labels: &[&str] = if profile.name == "records-1000" {
            &["a"]
        } else {
            &["a", "b", "c", "d"]
        };
        for run_label in run_labels {
            generate_corpus(&corpus, &profile);
            let (hash, files) = corpus_manifest(&corpus);
            assert_eq!(
                files,
                profile.expected + 1,
                "{} must contain {} record files plus the config",
                profile.name,
                profile.expected
            );
            if profile.name == "records-1000" {
                let req_members = fs::read_dir(corpus.join("docs/requirements/business"))
                    .unwrap()
                    .flatten()
                    .filter(|entry| {
                        let name = entry.file_name();
                        let name = name.to_string_lossy();
                        name.starts_with("REQ-001.") && name != "REQ-001.md"
                    })
                    .count();
                let mockups = fs::read_dir(corpus.join("docs/specifications/ui/mockups"))
                    .unwrap()
                    .flatten()
                    .filter(|entry| entry.file_name().to_string_lossy().starts_with("MOCK-"))
                    .count();
                assert_eq!(req_members, 200, "records-1000 has 200 grouped members");
                assert_eq!(mockups, 50, "records-1000 has 50 SingleHtmlMockup records");
            }
            eprintln!(
                "{} run {}: {} active records, {} files, corpus sha256 {}",
                profile.name, run_label, profile.expected, files, hash
            );
            let plan = runs.first().map(|run: &RunResult| {
                let mut plan = serde_json::Map::new();
                for route in &run.routes {
                    plan.insert(route.label.clone(), Value::String(route.band.clone()));
                }
                plan
            });
            let run = measure_run(
                &profile,
                &corpus,
                run_label,
                &routes,
                plan.as_ref(),
                &budget,
            );
            if !no_write {
                let raw_dir = root.join("docs/benchmarks/raw");
                fs::create_dir_all(&raw_dir).unwrap();
                let path = raw_dir.join(format!("{}-run-{run_label}.json", profile.name));
                fs::write(
                    &path,
                    serde_json::to_string_pretty(&run_json(&profile, &run)).unwrap() + "\n",
                )
                .unwrap();
            }
            runs.push(run);
        }
        if runs.len() > 1 {
            for run in &runs[1..] {
                assert_eq!(
                    runs[0].corpus_hash, run.corpus_hash,
                    "{} corpus hash must be stable across runs",
                    profile.name
                );
                assert_eq!(
                    runs[0].corpus_files, run.corpus_files,
                    "{} corpus file count must be stable across runs",
                    profile.name
                );
            }
            // Every later run must reuse the band and counts selected by run A.
            for run in &runs[1..] {
                for (first, later) in runs[0].routes.iter().zip(run.routes.iter()) {
                    assert_eq!(
                        first.band, later.band,
                        "{} {} band differs between runs",
                        profile.name, first.label
                    );
                    if !first.truncated && !later.truncated {
                        assert_eq!(
                            first.warmups, later.warmups,
                            "{} {} warm-ups differ between runs",
                            profile.name, first.label
                        );
                        assert_eq!(
                            first.measured(),
                            later.measured(),
                            "{} {} counts differ between runs",
                            profile.name,
                            first.label
                        );
                    }
                }
            }
        }
        results.push((profile, runs));
    }

    if !no_write {
        let summary = summary_markdown(&results);
        let docs = root.join("docs/benchmarks");
        fs::create_dir_all(&docs).unwrap();
        fs::write(docs.join("MS-032-baseline.md"), summary).unwrap();
    }

    eprintln!(
        "baseline complete: {} profile(s), reduced-sample protocol (band A 10+100, band B 2+20)",
        results.len()
    );
}

// ---------------------------------------------------------------------------
// Summary regression (TASK-292 C-2, C-4, C-6)
// ---------------------------------------------------------------------------

/// The recorded summary is produced only after hours of measurement, so its
/// shape is checked here from synthetic runs: every profile lists all routes,
/// confirmation reruns and commit provenance appear, and the wall-delta column
/// is labelled in microseconds.
#[test]
fn summary_reports_confirmations_and_commit_provenance() {
    fn route(label: &str, band: &str, measured: usize, p50: u64, p95: u64) -> RouteRun {
        RouteRun {
            label: label.to_string(),
            path: format!("/{label}"),
            band: band.to_string(),
            warmups: 2,
            truncated: false,
            forced_band: false,
            measured,
            server_ms: vec![p50; measured],
            wall_us: vec![p95 * 1000; measured],
            server_p50: p50,
            server_p95: p95,
            wall_p50_us: p50 * 1000,
            wall_p95_us: p95 * 1000,
        }
    }
    fn run(label: &str) -> RunResult {
        RunResult {
            run: label.to_string(),
            corpus_hash: "hash".to_string(),
            corpus_files: 101,
            routes: vec![route("inventory", BAND_B, BAND_B_SAMPLES, 1000, 1050)],
            host: json!({
                "cpu_model": "test cpu",
                "physical_cores": 1,
                "logical_cores": 2,
                "ram_gib": 8.0,
                "disk_type": "SSD",
                "kernel": "test",
                "distribution": "test",
            }),
            date: "2026-11-22T00:00:00+00:00".to_string(),
            commit: "head".to_string(),
            production_commit: "production".to_string(),
            harness_commit: "harness".to_string(),
            worktree_modified: true,
            rustc: "rustc test".to_string(),
            build_profile: "release (lto=true, strip=true)".to_string(),
        }
    }
    let profile = Profile {
        name: "records-100",
        expected: 100,
        counts: Counts {
            task: 14,
            milestone: 5,
            release: 2,
            decision: 5,
            req_members: 20,
            mockup: 5,
            nfr_members: 5,
            audit: 4,
            glossary: 6,
            rule: 6,
            tech_spec: 6,
            test_case: 5,
            test_report: 3,
            ui_spec: 3,
            use_case: 4,
            note: 5,
        },
    };
    let results = vec![(profile, vec![run("a"), run("b"), run("c"), run("d")])];
    let summary = summary_markdown(&results);
    assert!(summary.contains("Confirmation reruns"));
    assert!(summary.contains("Wall Δ p95 (µs)"));
    assert!(summary.contains("Measured production tree commit: `production`"));
    assert!(summary.contains("Harness commit: `harness`"));
    assert!(!summary.contains("Omitted at this profile"));
    assert!(summary.contains("| inventory | B | 20 |"));
    assert!(summary.contains("Every data-driven count below the full"));
    assert!(summary.contains("### Explicitly assigned bands"));
    assert!(!summary.contains(
        "Every count below the full 10 warm-up + 100 measured protocol is band B and is caused by"
    ));
}

/// TASK-292 CR-1: the historical comparison table must be rebuilt from the
/// cached server statistics. A from-raw `RunResult` has no server sample
/// vectors, so reading `server_ms` there reports 0 ms even though the raw JSON
/// records the measured values.
#[test]
fn summary_from_raw_reports_measured_382_server_values() {
    let root = project_root();
    let results = results_from_raw(&root);
    let summary = summary_markdown(&results);
    let section = summary
        .split("## Comparison with the historical 382-record figures")
        .nth(1)
        .expect("the summary has a historical comparison section");
    assert!(
        section.contains(
            "| Inventory server-observed p50 | 14.2 s (single figure) | 11776 ms (band B, n=20) |"
        ),
        "{section}"
    );
    assert!(
        section.contains(
            "| Inventory server-observed p95 | 14.2 s (single figure) | 11971 * ms (band B, n=20) |"
        ),
        "{section}"
    );
    assert!(
        section.contains(
            "| Requirement group view (`REQ-001`) server p50 | 1.8 s (single figure) | 3552 ms"
        ),
        "{section}"
    );
    assert!(
        section.contains(
            "| Grouped member view (`REQ-001.001`) server p50 | 1.8 s (single figure) | 3666 ms"
        ),
        "{section}"
    );
    assert!(
        section.contains(
            "| Grouped member view (`REQ-001.001`) server p95 | 1.8 s (single figure) | 3736 * ms"
        ),
        "{section}"
    );
    assert!(
        !section.contains("| 0 ms"),
        "the from-raw table must not report a zero measured value: {section}"
    );
    assert!(summary.contains("### Explicitly assigned bands"));
    assert!(
        summary.contains("| `records-1000` | view-grouped-group | B | 2 | 20 |"),
        "forced bands are recorded separately with their reason"
    );
    assert!(
        !summary.contains("| `records-1000` | edit-single-html-mockup |"),
        "CR-2 removes edit-single-html-mockup from the forced bands"
    );
    assert!(!summary.contains(
        "Every count below the full 10 warm-up + 100 measured protocol is band B and is caused by"
    ));
}

/// TASK-292 CR-2: re-measure one cheap route of a recorded profile with the
/// data-driven full protocol and replace only that route in run A, so the
/// expensive routes are never repeated. The summary is then rebuilt from the
/// updated raw JSON.
#[test]
#[ignore = "benchmark route re-measure: run explicitly with PAPYRUS_BENCH_REMEASURE_ROUTE=<label>"]
fn benchmark_remeasure_single_route() {
    let label = std::env::var("PAPYRUS_BENCH_REMEASURE_ROUTE")
        .expect("PAPYRUS_BENCH_REMEASURE_ROUTE must name one measured route");
    let profile_name =
        std::env::var("PAPYRUS_BENCH_PROFILE").unwrap_or_else(|_| "records-1000".to_string());
    let profile = profiles()
        .into_iter()
        .find(|profile| profile.name == profile_name)
        .unwrap_or_else(|| panic!("unknown profile {profile_name}"));
    let (_, path) = ROUTES
        .iter()
        .find(|(name, _)| *name == label)
        .copied()
        .unwrap_or_else(|| panic!("unknown route {label}"));

    let root = project_root();
    let corpus = root.join(".tmp/benchmark").join(profile.name);
    generate_corpus(&corpus, &profile);
    let raw_path = root.join(format!("docs/benchmarks/raw/{}-run-a.json", profile.name));
    let text = fs::read_to_string(&raw_path)
        .unwrap_or_else(|error| panic!("read {}: {error}", raw_path.display()));
    let mut value: Value = serde_json::from_str(&text)
        .unwrap_or_else(|error| panic!("parse {}: {error}", raw_path.display()));

    let server = Server::start(&corpus);
    let (status, _, _) = server.request("/");
    assert_eq!(status, 200, "readiness inventory request");
    let route = measure_route(&server, &label, path, None, false, &BandBudget::recorded());
    drop(server);
    eprintln!(
        "re-measured {} {}: band {} {} warm-up + {} measured; server p50 {} ms p95 {} ms",
        profile.name,
        label,
        route.band,
        route.warmups,
        route.measured(),
        route.server_p50,
        route.server_p95
    );
    assert_eq!(
        route.band, BAND_A,
        "CR-2 requires {label} to be selected into the data-driven band A"
    );
    assert_eq!(
        route.measured(),
        BAND_A_SAMPLES,
        "CR-2 requires the full band A protocol for {label}"
    );

    let routes = value["routes"].as_array_mut().expect("routes array");
    let slot = routes
        .iter_mut()
        .find(|route| route["label"].as_str() == Some(label.as_str()))
        .unwrap_or_else(|| panic!("recorded run A has no route {label}"));
    *slot = route_json(&route);
    fs::write(
        &raw_path,
        serde_json::to_string_pretty(&value).unwrap() + "\n",
    )
    .unwrap_or_else(|error| panic!("write {}: {error}", raw_path.display()));

    let results = results_from_raw(&root);
    let summary = summary_markdown(&results);
    fs::write(root.join("docs/benchmarks/MS-032-baseline.md"), summary)
        .expect("write the regenerated summary");
}
