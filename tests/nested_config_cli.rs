//! TASK-248: nested document-type configuration parsing, validation, fill,
//! selectors, and emission.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_papyrus"))
}

fn base_dir() -> PathBuf {
    PathBuf::from(".tmp").join("nested-config-cli")
}

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

    fn write(&self, content: &str) {
        fs::write(self.path.join("papyrus.config.yml"), content).expect("write config");
    }

    fn read(&self) -> String {
        fs::read_to_string(self.path.join("papyrus.config.yml")).expect("read config")
    }

    fn run(&self, args: &[&str]) -> Output {
        bin()
            .arg("--dir")
            .arg(&self.path)
            .args(args)
            .output()
            .expect("run papyrus")
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

    fn rejected(&self, args: &[&str]) -> String {
        let before = self.read();
        let out = self.run(args);
        assert_eq!(
            out.status.code(),
            Some(4),
            "command {:?} should exit 4: {}",
            args,
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(self.read(), before, "rejected command must not write");
        String::from_utf8_lossy(&out.stderr).to_string()
    }
}

impl Drop for Project {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
        let _ = fs::remove_dir(base_dir());
    }
}

fn deploy_with_fields() -> String {
    "document_types:\n  deploy:\n    prefix: DEP\n    directory: deploy/\n    fields:\n      - key: description\n        heading: '## Description'\n        kind: multiline\n        verbs: [create]\n"
        .to_string()
}

#[test]
fn unknown_entry_field_and_enum_diagnostics_are_aggregated_in_order() {
    let project = Project::new("aggregate");
    project.write(
        "document_types:\n  deploy:\n    prefix: DEP\n    directory: deploy/\n    reference_visibility: maybe\n    template: x\n    fields:\n      - key: description\n        heading: '## Description'\n        kind: paragraph\n        verbs: [create, delete]\n      - key: extra\n        heading: '## Extra'\n        template: y\n",
    );
    let stderr = project.rejected(&["deploy", "list"]);
    let lines: Vec<&str> = stderr.lines().collect();
    assert_eq!(
        lines,
        vec![
            "document_types.deploy.template: unknown key",
            "document_types.deploy.reference_visibility: must be visible or hidden",
            "document_types.deploy.fields[description].kind: must be text, multiline, or list",
            "document_types.deploy.fields[description].verbs: invalid verb delete; must be create or edit",
            "document_types.deploy.fields[extra].template: unknown key",
        ]
    );
}

#[test]
fn unknown_keys_and_enums_name_the_selector_path() {
    let project = Project::new("paths");
    project.write(
        "document_types:\n  deploy:\n    prefix: DEP\n    directory: deploy/\n    model: SingleMarkdown\n    fields:\n      - key: description\n        heading: '## Description'\n        order: 2\n",
    );
    let stderr = project.rejected(&["deploy", "list"]);
    assert!(stderr.contains("document_types.deploy.model"), "{stderr}");
    assert!(
        stderr.contains("document_types.deploy.fields[description].order"),
        "{stderr}"
    );
}

#[test]
fn field_rules_reject_reserved_missing_and_duplicate_keys() {
    let project = Project::new("field-rules");
    project.write(
        "document_types:\n  deploy:\n    prefix: DEP\n    directory: deploy/\n    fields:\n      - key: title\n        heading: '## Title'\n      - key: empty\n      - key: description\n        heading: '## One'\n      - key: description\n        heading: '## Two'\n      - key: front\n        frontmatter_key: status\n",
    );
    let stderr = project.rejected(&["deploy", "list"]);
    for needle in [
        "document_types.deploy.fields[title].key: reserved field key: title",
        "document_types.deploy.fields[empty]: must declare heading or frontmatter_key",
        "document_types.deploy.fields[2].key: duplicate field key: description",
        "document_types.deploy.fields[front].frontmatter_key: reserved field key: status",
    ] {
        assert!(stderr.contains(needle), "missing {needle:?}: {stderr}");
    }
}

#[test]
fn same_field_key_is_accepted_in_group_and_member_roles() {
    let project = Project::new("role-independence");
    project.write(
        "document_types:\n  req:\n    prefix: REQ\n    directory: requirements/business/\n    group:\n      fields:\n        - key: story\n          heading: '## User Story'\n    member:\n      fields:\n        - key: story\n          heading: '## User Story'\n",
    );
    project.ok(&["req", "list"]);
    assert_eq!(
        project
            .ok(&[
                "config",
                "get",
                "document_types.req.member.fields[story].heading"
            ])
            .trim(),
        "## User Story"
    );
    assert_eq!(
        project
            .ok(&[
                "config",
                "get",
                "document_types.req.group.fields[0].heading"
            ])
            .trim(),
        "## User Story"
    );
    let list = project.ok(&["config", "list"]);
    assert!(
        list.contains("document_types.req.group.fields[story].heading: ## User Story"),
        "{list}"
    );
    assert!(
        list.contains("document_types.req.member.fields[story].heading: ## User Story"),
        "{list}"
    );
}

#[test]
fn grouped_entry_rejects_top_level_fields_and_empty_fields_is_valid() {
    let project = Project::new("grouped-shape");
    project.write(
        "document_types:\n  req:\n    prefix: REQ\n    directory: requirements/business/\n    fields: []\n",
    );
    let stderr = project.rejected(&["req", "list"]);
    assert!(stderr.contains("document_types.req.fields"), "{stderr}");

    project.write(
        "document_types:\n  deploy:\n    prefix: DEP\n    directory: deploy/\n    fields: []\n",
    );
    project.ok(&["deploy", "list"]);

    project.write(
        "document_types:
  deploy:
    prefix: DEP
    directory: deploy/
    group:
      fields: []
",
    );
    let stderr = project.rejected(&["deploy", "list"]);
    assert!(stderr.contains("document_types.deploy.group"), "{stderr}");
}

#[test]
fn fill_and_replace_follow_the_seed_rules() {
    let project = Project::new("fill");
    project.write("document_types:\n  glossary:\n    prefix: GLO\n    directory: glossary/\n");
    assert_eq!(
        project
            .ok(&["config", "get", "document_types.glossary.statuses"])
            .trim(),
        "[active, archived]"
    );
    assert_eq!(
        project
            .ok(&["config", "get", "document_types.glossary.enabled"])
            .trim(),
        "true"
    );
    assert!(
        !project
            .ok(&["config", "list"])
            .contains("document_types.req.")
    );

    project.write(
        "document_types:\n  glossary:\n    prefix: GLO\n    directory: glossary/\n    enabled: false\n    statuses: [active, archived, review]\n",
    );
    assert_eq!(
        project
            .ok(&["config", "get", "document_types.glossary.enabled"])
            .trim(),
        "false"
    );
    assert_eq!(
        project
            .ok(&["config", "get", "document_types.glossary.statuses"])
            .trim(),
        "[active, archived, review]"
    );
}

#[test]
fn missing_prefix_or_directory_is_rejected_even_for_a_seed_slug() {
    let project = Project::new("required-prefix-directory");
    project.write("document_types:\n  req:\n    directory: requirements/business/\n");
    let stderr = project.rejected(&["req", "list"]);
    assert!(stderr.contains("document_types.req.prefix"), "{stderr}");

    project.write("document_types:\n  req:\n    prefix: REQ\n");
    let stderr = project.rejected(&["req", "list"]);
    assert!(stderr.contains("document_types.req.directory"), "{stderr}");
}

#[test]
fn emitter_round_trips_block_sequences_and_quoted_headings() {
    let project = Project::new("emitter");
    project.write(
        "document_types:\n  deploy:\n    prefix: DEP\n    directory: deploy/\n    fields:\n      - key: description\n        heading: '## User Story'\n        kind: multiline\n        verbs: [create]\n",
    );
    project.ok(&["config", "set", "task_prefix", "PAP"]);
    let rendered = project.read();
    assert!(
        rendered.contains(
            "    fields:\n      - key: description\n        heading: \"## User Story\"\n        kind: multiline\n        verbs: [create]\n"
        ),
        "{rendered}"
    );
    assert!(!rendered.contains("[{key:"), "{rendered}");
    let reparsed: yaml_serde::Value = yaml_serde::from_str(&rendered).unwrap();
    assert_eq!(
        papyrus::config::resolve_selector(
            &reparsed,
            "document_types.deploy.fields[description].heading"
        )
        .unwrap(),
        &yaml_serde::Value::String("## User Story".to_string())
    );
    assert_eq!(
        project
            .ok(&["config", "get", "document_types.deploy.fields[0].heading"])
            .trim(),
        "## User Story"
    );
}

#[test]
fn config_set_materializes_only_the_target_entry_and_uses_flow_statuses() {
    let project = Project::new("materialize");
    project.write("project_name: X\n");
    project.ok(&["config", "set", "document_types.req.enabled", "false"]);
    let config = project.read();
    assert!(
        config.contains(
            "document_types:\n  req:\n    prefix: REQ\n    directory: requirements/business/\n    enabled: false\n"
        ),
        "{config}"
    );
    assert!(!config.contains("\n  nfr:\n"), "{config}");
    assert_eq!(
        project
            .ok(&["config", "get", "document_types.req.statuses"])
            .trim(),
        "[active, archived]"
    );

    project.ok(&[
        "config",
        "set",
        "document_types.req.statuses",
        "active,archived,review",
    ]);
    let config = project.read();
    assert!(
        config.contains("statuses: [active, archived, review]"),
        "{config}"
    );

    let before = project.read();
    let out = project.run(&[
        "config",
        "set",
        "document_types.req.statuses",
        "active,review",
    ]);
    assert_eq!(out.status.code(), Some(4));
    assert_eq!(project.read(), before);

    // A slug with no seed entry cannot be introduced by config set, because
    // prefix and directory are never filled.
    let other = Project::new("materialize-unknown");
    other.write(
        "project_name: X
",
    );
    let before = other.read();
    let out = other.run(&["config", "set", "document_types.request.prefix", "RQ"]);
    assert_eq!(out.status.code(), Some(4));
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("document_types.request"),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(other.read(), before);
}

#[test]
fn config_set_nested_leaf_creates_only_its_path() {
    let project = Project::new("nested-leaf");
    project.write(
        "project_name: X
",
    );
    project.ok(&[
        "config",
        "set",
        "document_types.req.group.reference_visibility",
        "hidden",
    ]);
    let config = project.read();
    assert!(
        config.contains(
            "  req:
    prefix: REQ
    directory: requirements/business/
    group:
      reference_visibility: hidden
"
        ),
        "{config}"
    );
    assert!(!config.contains("member:"), "{config}");
    assert!(!config.contains("fields:"), "{config}");
    assert!(!config.contains("statuses"), "{config}");
}

#[test]
fn config_set_refuses_field_sequence_edits() {
    let project = Project::new("field-edit");
    project.write(&deploy_with_fields());
    let before = project.read();
    for key in [
        "document_types.deploy.fields",
        "document_types.deploy.fields[description].heading",
        "document_types.deploy.fields[0].kind",
    ] {
        let out = project.run(&["config", "set", key, "x"]);
        assert_eq!(out.status.code(), Some(4), "{key}");
        assert!(
            String::from_utf8_lossy(&out.stderr).contains(key),
            "{key}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(project.read(), before);
    }
}

#[test]
fn config_unset_restores_seed_collections_and_removes_scalar_overrides() {
    let project = Project::new("unset");
    project.write(
        "document_types:\n  glossary:\n    prefix: GLO\n    directory: glossary/\n    enabled: false\n    statuses: [active, archived, review]\n",
    );
    project.ok(&["config", "unset", "document_types.glossary.statuses"]);
    assert_eq!(
        project
            .ok(&["config", "get", "document_types.glossary.statuses"])
            .trim(),
        "[active, archived]"
    );
    project.ok(&["config", "unset", "document_types.glossary.enabled"]);
    assert_eq!(
        project
            .ok(&["config", "get", "document_types.glossary.enabled"])
            .trim(),
        "true"
    );
}

#[test]
fn selectors_resolve_keyed_and_indexed_forms_and_reject_bad_paths() {
    let project = Project::new("selectors");
    project.write(&deploy_with_fields());
    let keyed = project
        .ok(&[
            "config",
            "get",
            "document_types.deploy.fields[description].heading",
        ])
        .trim()
        .to_string();
    let indexed = project
        .ok(&["config", "get", "document_types.deploy.fields[0].heading"])
        .trim()
        .to_string();
    assert_eq!(keyed, indexed);

    for selector in [
        "document_types.deploy.fields[unknown].heading",
        "document_types.deploy.fields[9].heading",
    ] {
        let out = project.run(&["config", "get", selector]);
        assert_eq!(out.status.code(), Some(4), "{selector}");
        assert!(
            String::from_utf8_lossy(&out.stderr).contains(selector),
            "{selector}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

#[test]
fn config_list_prints_seed_filled_values_and_selector_paths() {
    let project = Project::new("list-paths");
    project.write(&format!(
        "{}  glossary:
    prefix: GLO
    directory: glossary/
",
        deploy_with_fields()
    ));
    let list = project.ok(&["config", "list"]);
    for line in [
        "document_types.deploy.prefix: DEP",
        "document_types.deploy.directory: deploy/",
        "document_types.deploy.enabled: true",
        "document_types.deploy.fields[description].key: description",
        "document_types.deploy.fields[description].heading: ## Description",
        "document_types.deploy.fields[description].kind: multiline",
        "document_types.deploy.fields[description].required: false",
        "document_types.deploy.fields[description].verbs: [create]",
        "document_types.glossary.statuses: [active, archived]",
        "document_types.glossary.enabled: true",
    ] {
        assert!(list.contains(line), "missing {line:?}:\n{list}");
    }
}

#[test]
fn init_writes_minimal_entries_that_resolve_from_the_seed() {
    let project = Project::new("init");
    let out = project.run(&["init", "Proj", "--no-git"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let config = project.read();
    let doc_start = config.find("document_types:").expect("document types");
    let block = &config[doc_start..];
    assert!(
        block.contains("  req:\n    prefix: REQ\n    directory: requirements/business/\n"),
        "{block}"
    );
    assert!(!block.contains("statuses"), "{block}");
    assert!(!block.contains("fields"), "{block}");
    assert!(!block.contains("default_status"), "{block}");
    assert!(!block.contains("label"), "{block}");

    assert_eq!(
        project
            .ok(&["config", "get", "document_types.req.statuses"])
            .trim(),
        "[active, archived]"
    );
    project.ok(&["req", "list"]);

    project.write("document_types:\n  bad slug:\n    prefix: BAD\n    directory: bad/\n");
    let repair = project.run(&["init", "Proj", "--force", "--no-git"]);
    assert!(
        repair.status.success(),
        "{}",
        String::from_utf8_lossy(&repair.stderr)
    );
    assert!(!project.read().contains("bad slug"));
}
