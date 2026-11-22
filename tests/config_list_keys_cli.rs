//! TASK-111: list-valued configuration keys in `config set|get|list`.
//! TASK-122: document-type `default_status` in `config list`.
//!
//! Covers the comma-separated value form, flow-sequence persistence, validation
//! before write, repair mode, `config unset`, JSON output, and help/docs.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_papyrus"))
}

fn base_dir() -> PathBuf {
    PathBuf::from(".tmp").join("config-list-keys")
}

/// Temporary project directory that removes itself when the test ends.
struct Project {
    path: PathBuf,
}

impl Project {
    fn new(name: &str) -> Self {
        let path = base_dir().join(name);
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("create project directory");
        Self { path }
    }

    fn initialized(name: &str) -> Self {
        let project = Self::new(name);
        let out = project.run(&["init", "Proj", "--no-git"]);
        assert!(
            out.status.success(),
            "init failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        project
    }

    fn config_path(&self) -> PathBuf {
        self.path.join("papyrus.config.yml")
    }

    fn write_config(&self, content: &str) {
        fs::write(self.config_path(), content).expect("write config file");
    }

    fn read_config(&self) -> String {
        fs::read_to_string(self.config_path()).expect("read config file")
    }

    fn run(&self, args: &[&str]) -> Output {
        bin()
            .arg("--dir")
            .arg(&self.path)
            .args(args)
            .output()
            .expect("run papyrus")
    }

    fn run_json(&self, args: &[&str]) -> Output {
        let mut all = vec!["--json"];
        all.extend_from_slice(args);
        self.run(&all)
    }

    fn ok(&self, args: &[&str]) -> String {
        let out = self.run(args);
        assert!(
            out.status.success(),
            "command {:?} failed: {}",
            args,
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).to_string()
    }
}

impl Drop for Project {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
        let _ = fs::remove_dir(base_dir());
    }
}

fn assert_rejected(project: &Project, args: &[&str], needle: &str) {
    let before = project.read_config();
    let out = project.run(args);
    assert_eq!(
        out.status.code(),
        Some(4),
        "command {:?} should exit 4: {}",
        args,
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains(needle),
        "message must contain {:?}: {}",
        needle,
        stderr
    );
    assert_eq!(project.read_config(), before, "file must stay unchanged");
}

#[test]
fn set_top_level_statuses_writes_a_flow_sequence() {
    let project = Project::initialized("top-level");
    project.ok(&["config", "set", "statuses", "ToDo, Initiated, Done"]);

    let config = project.read_config();
    assert!(
        config.contains("statuses: [ToDo, Initiated, Done]\n"),
        "config must hold a flow sequence: {}",
        config
    );

    let value: yaml_serde::Value = yaml_serde::from_str(&config).expect("config reparse");
    let Some(yaml_serde::Value::Sequence(sequence)) = papyrus::config::yaml_get(&value, "statuses")
    else {
        panic!("statuses must parse as a sequence: {}", config);
    };
    let parsed: Vec<&str> = sequence
        .iter()
        .map(|item| match item {
            yaml_serde::Value::String(text) => text.as_str(),
            other => panic!("expected a string element: {:?}", other),
        })
        .collect();
    assert_eq!(parsed, vec!["ToDo", "Initiated", "Done"]);

    assert_eq!(
        project.ok(&["config", "get", "statuses"]).trim(),
        "[ToDo, Initiated, Done]"
    );
    assert!(
        project
            .ok(&["config", "list"])
            .contains("statuses: [ToDo, Initiated, Done]")
    );

    let out = project.run_json(&["config", "get", "statuses"]);
    assert!(out.status.success());
    let json: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&out.stdout).trim()).unwrap();
    assert_eq!(
        json,
        serde_json::json!(["ToDo", "Initiated", "Done"]),
        "config get --json keeps a JSON array"
    );
}

#[test]
fn set_document_statuses_preserves_entry_fields() {
    let project = Project::initialized("document");
    project.ok(&[
        "config",
        "set",
        "document_types.audit.statuses",
        "active, archived, open",
    ]);

    let config = project.read_config();
    let audit_block = config
        .lines()
        .skip_while(|line| line.trim_end() != "  audit:")
        .skip(1)
        .take_while(|line| line.starts_with("    "))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(audit_block.contains("prefix: AUD"), "{}", audit_block);
    assert!(
        audit_block.contains("directory: audit-records/"),
        "{}",
        audit_block
    );
    assert!(
        audit_block.contains("statuses: [active, archived, open]"),
        "{}",
        audit_block
    );

    assert_eq!(
        project
            .ok(&["config", "get", "document_types.audit.statuses"])
            .trim(),
        "[active, archived, open]"
    );
    let list = project.ok(&["config", "list"]);
    assert!(
        list.contains("document_types.audit.statuses: [active, archived, open]"),
        "{}",
        list
    );
    assert_eq!(
        project
            .ok(&["config", "get", "document_types.req.statuses"])
            .trim(),
        "[active, archived]"
    );
    assert!(
        !list.contains("document_types.ui-mockup.statuses"),
        "ui-mockup declares no status list: {}",
        list
    );

    let out = project.run_json(&["config", "list"]);
    assert!(out.status.success());
    let json: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&out.stdout).trim()).unwrap();
    assert_eq!(
        json["document_types"]["audit"]["statuses"],
        serde_json::json!(["active", "archived", "open"])
    );
    assert_eq!(
        json["statuses"],
        serde_json::json!([
            "ToDo",
            "Initiated",
            "Refined",
            "Implemented",
            "Validated",
            "Done"
        ])
    );
}

#[test]
fn list_elements_are_trimmed_strings_and_scalar_commas_stay_data() {
    let project = Project::initialized("parsing");
    project.ok(&["config", "set", "statuses", " ToDo , Done "]);
    assert_eq!(
        project.ok(&["config", "get", "statuses"]).trim(),
        "[ToDo, Done]"
    );

    // Scalar keys keep their YAML-scalar semantics: a comma is data.
    project.ok(&["config", "set", "project_name", "Acme, Inc"]);
    assert_eq!(
        project.ok(&["config", "get", "project_name"]).trim(),
        "Acme, Inc"
    );
    assert!(project.read_config().contains("project_name: Acme, Inc"));

    // Elements are never YAML-interpreted.
    project.ok(&[
        "config",
        "set",
        "document_types.audit.statuses",
        "active, archived, true",
    ]);
    let list = project.ok(&["config", "list"]);
    assert!(
        list.contains("document_types.audit.statuses: [active, archived, true]"),
        "{}",
        list
    );
    let config = project.read_config();
    let value: yaml_serde::Value = yaml_serde::from_str(&config).unwrap();
    let Some(yaml_serde::Value::Sequence(sequence)) =
        papyrus::config::yaml_get(&value, "document_types.audit.statuses")
    else {
        panic!("statuses must be a sequence");
    };
    assert!(matches!(sequence[2], yaml_serde::Value::String(_)));
}

#[test]
fn invalid_list_values_are_rejected_without_writing() {
    let project = Project::initialized("rejections");
    for raw in ["", "   ", "a,,b", "a,", ",a", "a\nb", "a\tb"] {
        assert_rejected(&project, &["config", "set", "statuses", raw], "statuses:");
    }

    for raw in [
        "active, open",
        "active, archived, active",
        "active, archived, bad status",
    ] {
        assert_rejected(
            &project,
            &["config", "set", "document_types.audit.statuses", raw],
            "document_types.audit.statuses",
        );
    }
    assert_rejected(
        &project,
        &[
            "config",
            "set",
            "document_types.ui-mockup.statuses",
            "active, archived",
        ],
        "document_types.ui-mockup.statuses",
    );
    // The new task list no longer declares the configured default status.
    assert_rejected(
        &project,
        &["config", "set", "statuses", "New, Other"],
        "default_status",
    );
}

#[test]
fn block_style_lists_are_read_and_rewritten_in_flow_form() {
    let project = Project::new("block-style");
    project.write_config("statuses:\n  - ToDo\n  - Done\n");
    assert_eq!(
        project.ok(&["config", "get", "statuses"]).trim(),
        "[ToDo, Done]"
    );
    assert_eq!(
        project.read_config(),
        "statuses:\n  - ToDo\n  - Done\n",
        "get must not modify the file"
    );

    project.ok(&["config", "set", "task_prefix", "PAP"]);
    let config = project.read_config();
    assert!(
        config.contains("statuses: [ToDo, Done]\n"),
        "a later set rewrites declared lists in flow form: {}",
        config
    );
    assert!(config.contains("task_prefix: PAP"));
}

#[test]
fn repair_path_writes_a_valid_list_and_refuses_an_invalid_one() {
    let project = Project::new("repair");
    let invalid = "document_types:\n  audit:\n    prefix: AUD\n    directory: audit-records/\n    statuses:\n      - active\n";
    project.write_config(invalid);

    // Tolerant read surfaces still work on an invalid document status list.
    assert_eq!(
        project
            .ok(&["config", "get", "document_types.audit.statuses"])
            .trim(),
        "[active]"
    );
    assert!(
        project
            .ok(&["config", "list"])
            .contains("document_types.audit.statuses: [active]")
    );

    project.ok(&[
        "config",
        "set",
        "document_types.audit.statuses",
        "active,archived",
    ]);
    assert_eq!(
        project
            .ok(&["config", "get", "document_types.audit.statuses"])
            .trim(),
        "[active, archived]"
    );

    project.write_config(invalid);
    assert_rejected(
        &project,
        &[
            "config",
            "set",
            "document_types.audit.statuses",
            "active, archived, bad status",
        ],
        "document_types.audit.statuses",
    );
}

#[test]
fn unset_restores_defaults_and_refuses_a_dependent_default_status() {
    let project = Project::initialized("unset");
    project.ok(&["config", "set", "statuses", "ToDo, Done"]);
    project.ok(&["config", "unset", "statuses"]);
    assert_eq!(
        project.ok(&["config", "get", "statuses"]).trim(),
        "[ToDo, Initiated, Refined, Implemented, Validated, Done]"
    );

    project.ok(&[
        "config",
        "set",
        "document_types.audit.statuses",
        "active, archived, open",
    ]);
    project.ok(&["config", "unset", "document_types.audit.statuses"]);
    let config = project.read_config();
    assert!(!config.contains("statuses: []"), "{}", config);
    assert_eq!(
        project
            .ok(&["config", "get", "document_types.audit.statuses"])
            .trim(),
        "[active, archived]",
        "unset restores the seed collection"
    );

    // A declaration whose removal would orphan default_status is refused.
    project.write_config(
        "document_types:\n  audit:\n    prefix: AUD\n    directory: audit-records/\n    statuses: [active, archived, open]\n    default_status: open\n",
    );
    let before = project.read_config();
    let out = project.run(&["config", "unset", "document_types.audit.statuses"]);
    assert_eq!(out.status.code(), Some(4));
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("document_types.audit.default_status"),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(project.read_config(), before);
}

#[test]
fn no_escape_form_and_help_text() {
    let project = Project::initialized("no-escape");
    // `active\,archived` splits at the comma; the backslash survives and fails
    // the document status character rule.
    assert_rejected(
        &project,
        &[
            "config",
            "set",
            "document_types.audit.statuses",
            "active\\,archived",
        ],
        "document_types.audit.statuses",
    );

    let help = String::from_utf8_lossy(
        &bin()
            .args(["config", "set", "--help"])
            .output()
            .unwrap()
            .stdout,
    )
    .to_string();
    assert!(help.contains("comma-separated"), "{}", help);
    assert!(help.contains("flow sequence"), "{}", help);
    assert!(help.contains("not escaped"), "{}", help);
}

fn config_list_lines(project: &Project) -> Vec<String> {
    project
        .ok(&["config", "list"])
        .lines()
        .map(str::to_string)
        .collect()
}

fn line_index(lines: &[String], expected: &str) -> usize {
    lines
        .iter()
        .position(|line| line == expected)
        .unwrap_or_else(|| panic!("missing line {expected:?} in:\n{}", lines.join("\n")))
}

fn list_json(project: &Project) -> serde_json::Value {
    let out = project.run_json(&["config", "list"]);
    assert!(
        out.status.success(),
        "config list --json failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_str(String::from_utf8_lossy(&out.stdout).trim()).unwrap()
}

const AUDIT_WITH_DEFAULT: &str = "document_types:\n  audit:\n    prefix: AUD\n    directory: audit-records/\n    statuses: [active, archived, open]\n    default_status: open\n";

#[test]
fn document_default_status_is_listed_after_statuses() {
    let project = Project::new("default-status-declared");
    project.write_config(
        "document_types:\n  audit:\n    prefix: AUD\n    directory: audit-records/\n    enabled: true\n    statuses: [active, archived, open]\n    default_status: open\n  draft:\n    prefix: DRAFT\n    directory: drafts/\n    default_status: open\n  note:\n    prefix: NOTE\n    directory: notes/\n",
    );
    let lines = config_list_lines(&project);

    let statuses = line_index(
        &lines,
        "document_types.audit.statuses: [active, archived, open]",
    );
    assert_eq!(
        lines[statuses + 1],
        "document_types.audit.default_status: open",
        "default_status follows the same slug's statuses"
    );

    let enabled = line_index(&lines, "document_types.draft.enabled: true");
    assert_eq!(
        lines[enabled + 1],
        "document_types.draft.default_status: open",
        "default_status follows enabled when no status list is declared"
    );

    assert!(
        !lines
            .iter()
            .any(|line| line.starts_with("document_types.note.default_status")),
        "an entry without default_status gets no key: {:?}",
        lines
    );

    let audit = line_index(&lines, "document_types.audit.prefix: AUD");
    let draft = line_index(&lines, "document_types.draft.prefix: DRAFT");
    let note = line_index(&lines, "document_types.note.prefix: NOTE");
    assert!(
        audit < draft && draft < note,
        "slugs stay in ascending key order: {:?}",
        lines
    );
}

#[test]
fn document_default_status_is_absent_when_undeclared() {
    let project = Project::initialized("default-status-absent");
    let lines = config_list_lines(&project);
    assert!(
        !lines
            .iter()
            .any(|line| line.starts_with("document_types.audit.default_status")),
        "generated entries declare no default_status: {:?}",
        lines
    );

    let json = list_json(&project);
    assert!(
        json["document_types"]["audit"]
            .get("default_status")
            .is_none()
    );
    assert!(
        json["document_types"]["ui-mockup"]
            .get("default_status")
            .is_none()
    );
}

#[test]
fn unset_removes_the_listed_default_status() {
    let project = Project::new("default-status-unset");
    project.write_config(AUDIT_WITH_DEFAULT);
    assert!(
        project
            .ok(&["config", "list"])
            .contains("document_types.audit.default_status: open")
    );

    project.ok(&["config", "unset", "document_types.audit.default_status"]);

    let list = project.ok(&["config", "list"]);
    assert!(
        !list.contains("document_types.audit.default_status"),
        "unset must remove the listing line: {}",
        list
    );
    assert!(
        list_json(&project)["document_types"]["audit"]
            .get("default_status")
            .is_none()
    );
    let unset_default = project.run(&["config", "get", "document_types.audit.default_status"]);
    assert_eq!(unset_default.status.code(), Some(4));
    assert!(
        String::from_utf8_lossy(&unset_default.stderr)
            .contains("document_types.audit.default_status"),
        "{}",
        String::from_utf8_lossy(&unset_default.stderr)
    );
}

#[test]
fn invalid_default_status_stays_listed_while_set_and_unset_reject() {
    let project = Project::new("default-status-invalid");
    project.write_config(
        "document_types:\n  audit:\n    prefix: AUD\n    directory: audit-records/\n    statuses: [active, archived]\n    default_status: open\n",
    );

    // Tolerant load: the rejected value stays inspectable.
    let out = project.run(&["config", "list"]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("document_types.audit.default_status: open"),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );
    assert_eq!(
        project
            .ok(&["config", "get", "document_types.audit.default_status"])
            .trim(),
        "open"
    );

    // `config set` validates before write and names the offending key.
    let before = project.read_config();
    let out = project.run(&["config", "set", "task_prefix", "PAP"]);
    assert_eq!(out.status.code(), Some(4));
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("document_types.audit.default_status"),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(project.read_config(), before, "file must stay unchanged");

    // `config unset` also validates before write.
    let out = project.run(&["config", "unset", "document_types.audit.statuses"]);
    assert_eq!(out.status.code(), Some(4));
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("document_types.audit.default_status"),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(project.read_config(), before, "file must stay unchanged");

    // Documented repair path: set a declared status and the listing follows.
    project.ok(&[
        "config",
        "set",
        "document_types.audit.default_status",
        "active",
    ]);
    let list = project.ok(&["config", "list"]);
    assert!(
        list.contains("document_types.audit.default_status: active"),
        "{}",
        list
    );
    assert!(!list.contains("default_status: open"), "{}", list);
}

#[test]
fn text_and_json_default_status_match_config_get() {
    let project = Project::new("default-status-parity");
    project.write_config(AUDIT_WITH_DEFAULT);

    let text = project.ok(&["config", "list"]);
    let line = text
        .lines()
        .find(|line| line.starts_with("document_types.audit.default_status: "))
        .unwrap_or_else(|| panic!("missing default_status line: {}", text));
    let text_value = line
        .strip_prefix("document_types.audit.default_status: ")
        .unwrap();

    let json = list_json(&project);
    assert_eq!(
        json["document_types"]["audit"]["default_status"],
        text_value
    );
    assert_eq!(
        project
            .ok(&["config", "get", "document_types.audit.default_status"])
            .trim(),
        text_value
    );
}

#[test]
fn readme_documents_the_list_syntax() {
    let readme = fs::read_to_string("README.md").expect("README must exist");
    assert!(
        readme.contains(
            "The list-valued keys <b>statuses</b> and <b>document_types.&lt;slug&gt;.statuses</b>"
        ),
        "papyrus(1) config set paragraph must document list keys"
    );
    assert!(
        readme.contains("This is a list-valued key:"),
        "papyrus-config(5) statuses entry"
    );
    assert!(
        readme.contains("<b>document_types.&lt;slug&gt;.statuses</b>"),
        "papyrus-config(5) document_types entry"
    );
    assert!(
        readme.matches("flow sequence").count() >= 2,
        "README must state flow-sequence storage"
    );
    assert!(
        readme.matches("not escaped").count() >= 2,
        "README must state the no-escape limitation"
    );
}
