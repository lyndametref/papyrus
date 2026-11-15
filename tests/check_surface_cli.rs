//! TASK-139: the reserved `check` slug and the repair paths around it.
//!
//! A configuration that declares a document type named `check`, `Check`, or
//! `CHECK` is rejected with the pinned diagnostic, and both documented repair
//! surfaces — `config unset` and `init --force` — restore a working project.

use serde_json::{Value, json};
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

const CHECK_ENTRY: &str = "  check:\n    prefix: CHECK\n    directory: checks/\n    enabled: true\n    statuses: [active, archived]\n\n";

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_papyrus"))
}

fn base_dir() -> PathBuf {
    PathBuf::from(".tmp").join("check-surface")
}

struct Project {
    path: PathBuf,
}

impl Project {
    fn initialized(name: &str) -> Self {
        let path = base_dir().join(name);
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("create project directory");
        let project = Self { path };
        project.ok(&["init", "Proj", "--no-git"]);
        project
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
            "command {:?} failed (exit {:?}): {}",
            args,
            out.status.code(),
            stderr_of(&out)
        );
        stdout_of(&out)
    }

    fn path(&self, relative: &str) -> PathBuf {
        self.path.join(relative)
    }

    fn config(&self) -> String {
        fs::read_to_string(self.path("papyrus.config.yml")).expect("read configuration")
    }

    fn write_config(&self, content: &str) {
        fs::write(self.path("papyrus.config.yml"), content).expect("write configuration");
    }

    /// Hand-write the `check` entry, as a user repairing by hand would.
    fn declare_check_slug(&self, key: &str) {
        let entry = CHECK_ENTRY.replacen("check:", &format!("{}:", key), 1);
        let config = self.config();
        assert!(
            config.contains("browser_host:"),
            "the generated configuration has an anchor key"
        );
        let patched = config.replacen("browser_host:", &format!("{}{}", entry, "browser_host:"), 1);
        self.write_config(&patched);
    }

    fn json(&self, args: &[&str]) -> (Value, Option<i32>) {
        let mut all = vec!["--json"];
        all.extend_from_slice(args);
        let out = self.run(&all);
        let stdout = stdout_of(&out);
        let value = serde_json::from_str(stdout.trim()).expect("check emits JSON");
        (value, out.status.code())
    }
}

impl Drop for Project {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn stdout_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn stderr_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).to_string()
}

fn pinned(key: &str) -> String {
    format!(
        "document_types.{}: slug conflicts with built-in type check",
        key
    )
}

#[test]
fn check_slug_spellings_are_rejected_and_repairable_with_config_unset() {
    for (index, key) in ["check", "Check", "CHECK"].iter().enumerate() {
        let project = Project::initialized(&format!("unset-{index}"));
        project.declare_check_slug(key);

        // AC1: a fail-closed command reports the pinned validation outcome.
        let rejected = project.run(&["task", "list"]);
        assert_eq!(
            rejected.status.code(),
            Some(4),
            "a check document type is a validation outcome"
        );
        assert_eq!(stderr_of(&rejected), format!("{}\n", pinned(key)));

        // AC1 through the tolerant surface: one config-invalid finding.
        let (report, code) = project.json(&["check"]);
        assert_eq!(code, Some(4));
        let findings = report["findings"].as_array().expect("findings array");
        assert_eq!(
            findings.len(),
            1,
            "one diagnostic line, one finding: {}",
            report
        );
        assert_eq!(
            findings[0],
            json!({
                "kind": "config-invalid",
                "family": null,
                "subject": null,
                "locations": ["papyrus.config.yml"],
                "detail": pinned(key),
                "fixable": false
            })
        );

        // AC2: `config unset` repairs the project and persists.
        let key_path = format!("document_types.{}", key);
        let unset = project.run(&["config", "unset", &key_path]);
        assert_eq!(
            unset.status.code(),
            Some(0),
            "config unset {} failed: {}",
            key_path,
            stderr_of(&unset)
        );
        assert!(
            !project.config().contains("checks/"),
            "the removed entry is not persisted"
        );
        assert_eq!(
            project.ok(&["task", "list"]).trim(),
            "",
            "the fail-closed command runs again"
        );
    }
}

#[test]
fn init_force_rewrites_a_check_slug_configuration() {
    let project = Project::initialized("init-force");
    project.declare_check_slug("check");
    assert_eq!(project.run(&["task", "list"]).status.code(), Some(4));

    let out = project.run(&["init", "Proj", "--force", "--no-git"]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr_of(&out));

    let config = project.config();
    assert!(
        !config.contains("checks/"),
        "init --force replaces the configuration with the default set:\n{}",
        config
    );
    project.ok(&["task", "list"]);
    project.ok(&["config", "list"]);
}

#[test]
fn a_repaired_project_runs_check_clean() {
    let project = Project::initialized("repaired-check");
    project.declare_check_slug("Check");
    assert_eq!(project.run(&["check"]).status.code(), Some(4));

    project.ok(&["config", "unset", "document_types.Check"]);

    let out = project.run(&["check"]);
    assert_eq!(out.status.code(), Some(0), "{}", stdout_of(&out));
    assert_eq!(stdout_of(&out).trim(), "no findings");
}

#[test]
fn config_set_into_a_check_slug_project_is_refused_and_writes_nothing() {
    let project = Project::initialized("config-set-refused");
    project.declare_check_slug("check");
    let before = project.config();

    let out = project.run(&["config", "set", "task_prefix", "ITEM"]);
    assert_eq!(out.status.code(), Some(4), "{}", stderr_of(&out));
    assert_eq!(
        project.config(),
        before,
        "an invalid resulting configuration is never written"
    );
}
