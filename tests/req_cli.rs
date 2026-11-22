use std::fs;
use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_papyrus"))
}

fn tmpdir(name: &str) -> std::path::PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("papyrus-reqtest-{}", name));
    let _ = fs::remove_dir_all(&p);
    fs::create_dir_all(&p).unwrap();
    p
}

fn init(d: &std::path::Path) {
    let out = bin()
        .args(["--dir", d.to_str().unwrap(), "init", "Proj", "--no-git"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn req_group_and_requirement_lifecycle() {
    let d = tmpdir("life");
    init(&d);
    let p = d.to_str().unwrap();
    let out = bin()
        .args([
            "--dir",
            p,
            "req",
            "create-group",
            "--title",
            "G",
            "--story",
            "s",
            "--description",
            "d",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "REQ-001");

    let out = bin()
        .args(["--dir", p, "req", "create", "R", "--group", "REQ-001"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "REQ-001.001");

    let out = bin().args(["--dir", p, "req", "list"]).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("REQ-001 G"));
    assert!(s.contains("REQ-001.001 R"));

    let out = bin()
        .args(["--dir", p, "req", "view", "REQ-001.001"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("group: REQ-001\n"), "{s}");

    // edit preserves
    let out = bin()
        .args(["--dir", p, "req", "edit", "REQ-001.001", "--title", "R2"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let out = bin()
        .args(["--dir", p, "req", "view", "REQ-001.001"])
        .output()
        .unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("# REQ-001.001 R2"));
    assert!(s.contains("group: REQ-001\n"), "{s}");

    // archive hides
    let out = bin()
        .args(["--dir", p, "req", "archive", "REQ-001.001"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let out = bin().args(["--dir", p, "req", "list"]).output().unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(!s.contains("REQ-001.001"));

    // view archived allowed
    let out = bin()
        .args(["--dir", p, "req", "view", "REQ-001.001"])
        .output()
        .unwrap();
    assert!(out.status.success());

    let _ = fs::remove_dir_all(&d);
}

#[test]
fn req_errors() {
    let d = tmpdir("errs");
    init(&d);
    let p = d.to_str().unwrap();
    // missing group is one validation outcome (TC-120)
    let out = bin()
        .args(["--dir", p, "req", "create", "R"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("missing required argument: group"),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(!d.join("docs/requirements/business/REQ-001.md").exists());
    // unknown group is one validation outcome (BR-009)
    let out = bin()
        .args(["--dir", p, "req", "create", "R", "--group", "REQ-999"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("group not found: REQ-999"),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(!d.join("docs/requirements/business/REQ-999.001.md").exists());
    // conflicting title exit 4, with a concrete cause
    let out = bin()
        .args([
            "--dir", p, "req", "create", "A", "--title", "B", "--group", "REQ-999",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("conflicting title sources"),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    // unknown view exit 3
    let out = bin()
        .args(["--dir", p, "req", "view", "NOPE"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(3));
    // state conflict exit 4
    // create group first
    let _ = bin()
        .args(["--dir", p, "req", "create-group", "--title", "G"])
        .output()
        .unwrap();
    let _ = bin()
        .args(["--dir", p, "req", "create", "R", "--group", "REQ-001"])
        .output()
        .unwrap();
    let out = bin()
        .args(["--dir", p, "req", "archive", "REQ-001.001"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let out = bin()
        .args(["--dir", p, "req", "archive", "REQ-001.001"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn nfr_basic() {
    let d = tmpdir("nfr");
    init(&d);
    let p = d.to_str().unwrap();
    let out = bin()
        .args([
            "--dir",
            p,
            "nfr",
            "create-group",
            "--title",
            "NG",
            "--description",
            "desc",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "NFR-001");
    let out = bin()
        .args([
            "--dir",
            p,
            "nfr",
            "create",
            "NR",
            "--description",
            "nd",
            "--group",
            "NFR-001",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "NFR-001.001");
    let out = bin().args(["--dir", p, "nfr", "list"]).output().unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("NFR-001 NG"));
    assert!(s.contains("NFR-001.001 NR"));
    let out = bin()
        .args(["--dir", p, "--json", "nfr", "view", "NFR-001.001"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_str(&String::from_utf8_lossy(&out.stdout)).unwrap();
    assert_eq!(v["id"], "NFR-001.001");
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn sanitization_rejects_bad() {
    let d = tmpdir("san");
    init(&d);
    let p = d.to_str().unwrap();
    // A title is display text: path separators are legal (F8).
    for good in ["a/b", "x\\y"] {
        let out = bin()
            .args(["--dir", p, "req", "create-group", "--title", good])
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(0), "good={}", good);
    }
    fs::remove_dir_all(d.join("docs/requirements")).unwrap();
    fs::create_dir_all(d.join("docs/requirements")).unwrap();
    for bad in ["   ", "a\nb"] {
        let out = bin()
            .args(["--dir", p, "req", "create-group", "--title", bad])
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(4), "bad={}", bad);
    }
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn boundary_config_escape_exits_one() {
    let root = tmpdir("bnd");
    let proj = root.join("proj");
    let outside = root.join("outside");
    fs::create_dir_all(&proj).unwrap();
    fs::create_dir_all(&outside).unwrap();
    let _ = bin()
        .args(["--dir", proj.to_str().unwrap(), "init", "Proj", "--no-git"])
        .output()
        .unwrap();
    let out = bin()
        .args([
            "--dir",
            proj.to_str().unwrap(),
            "--config",
            outside.join("evil.yml").to_str().unwrap(),
            "req",
            "list",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn req_symlink_escape_exits_one() {
    let root = tmpdir("symlinkreq");
    let proj = root.join("proj");
    let outside = root.join("outside");
    fs::create_dir_all(&proj).unwrap();
    fs::create_dir_all(&outside).unwrap();
    init(&proj);
    let p = proj.to_str().unwrap();
    let _ = bin()
        .args(["--dir", p, "req", "create-group", "--title", "G"])
        .output()
        .unwrap();
    let _ = bin()
        .args(["--dir", p, "req", "create", "R", "--group", "REQ-001"])
        .output()
        .unwrap();
    let target = proj.join("docs/requirements/business/REQ-001.001.md");
    let outside_file = outside.join("evil.md");
    fs::write(&outside_file, "secret").unwrap();
    let _ = fs::remove_file(&target);
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let _ = symlink(&outside_file, &target);
    }
    // view should reject
    let out = bin()
        .args(["--dir", p, "req", "view", "REQ-001.001"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    // archive should reject
    let out = bin()
        .args(["--dir", p, "req", "archive", "REQ-001.001"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn grouped_create_and_create_group_write_the_default_status() {
    let d = tmpdir("group-status");
    init(&d);
    let p = d.to_str().unwrap();
    let out = bin()
        .args(["--dir", p, "req", "create-group", "--title", "Group one"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let group = fs::read_to_string(record_path(&d, "REQ-001")).unwrap();
    assert!(group.contains("status: active"), "{group}");

    let out = bin()
        .args([
            "--dir",
            p,
            "req",
            "create",
            "Member one",
            "--group",
            "REQ-001",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let member = fs::read_to_string(record_path(&d, "REQ-001.001")).unwrap();
    assert!(member.contains("status: active"), "{member}");

    let _ = fs::remove_dir_all(&d);
}

fn record_path(d: &std::path::Path, id: &str) -> std::path::PathBuf {
    d.join("docs/requirements/business")
        .join(format!("{id}.md"))
}

fn run_help(d: &std::path::Path, args: &[&str]) -> String {
    let out = bin()
        .args(["--dir", d.to_str().unwrap()])
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).to_string()
}

#[test]
fn req_grouped_help_surface_only_exposes_grouped_verbs() {
    let d = tmpdir("group-help");
    init(&d);
    for slug in ["req", "nfr"] {
        let help = run_help(&d, &[slug, "--help"]);
        for verb in [
            "create",
            "create-group",
            "list",
            "view",
            "edit",
            "archive",
            "delete",
            "status",
        ] {
            assert!(help.contains(verb), "{slug} help missing {verb}");
        }
        assert!(
            !help.contains("supersede"),
            "{slug} must not expose the mockup-only supersede verb"
        );
    }
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn member_create_requires_an_existing_active_same_type_group() {
    let d = tmpdir("group-validation");
    init(&d);
    let p = d.to_str().unwrap();

    // Missing group: one validation outcome naming the group, no file.
    let out = bin()
        .args(["--dir", p, "req", "create", "R"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));
    assert!(String::from_utf8_lossy(&out.stderr).contains("group"));
    assert!(!record_path(&d, "REQ-001.001").exists());

    // Cross-type group: one validation outcome, no file.
    let out = bin()
        .args(["--dir", p, "nfr", "create-group", "--title", "NG"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let out = bin()
        .args(["--dir", p, "req", "create", "R", "--group", "NFR-001"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));
    assert!(String::from_utf8_lossy(&out.stderr).contains("mismatch"));
    assert!(!record_path(&d, "REQ-001.001").exists());

    // Unknown group: one validation outcome, no file.
    let out = bin()
        .args(["--dir", p, "req", "create", "R", "--group", "REQ-999"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));
    assert!(String::from_utf8_lossy(&out.stderr).contains("not found"));
    assert!(!record_path(&d, "REQ-001.001").exists());

    // Archived group: one validation outcome, no file.
    let out = bin()
        .args(["--dir", p, "req", "create-group", "--title", "G"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let out = bin()
        .args(["--dir", p, "req", "archive", "REQ-001"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let out = bin()
        .args(["--dir", p, "req", "create", "R", "--group", "REQ-001"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));
    assert!(String::from_utf8_lossy(&out.stderr).contains("archived"));
    assert!(!record_path(&d, "REQ-001.001").exists());

    let _ = fs::remove_dir_all(&d);
}

#[test]
fn member_create_writes_the_engine_owned_group_front_matter_key() {
    let d = tmpdir("group-link");
    init(&d);
    let p = d.to_str().unwrap();
    let out = bin()
        .args([
            "--dir",
            p,
            "req",
            "create-group",
            "--title",
            "Group one",
            "--story",
            "Group story",
            "--description",
            "Group description",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let out = bin()
        .args([
            "--dir",
            p,
            "req",
            "create",
            "Member one",
            "--group",
            "REQ-001",
            "--story",
            "Member story",
            "--description",
            "Member description",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "REQ-001.001");

    let content = fs::read_to_string(record_path(&d, "REQ-001.001")).unwrap();
    assert!(content.contains("group: REQ-001\n"), "{content}");
    assert!(
        !content.contains("## Group"),
        "the legacy group section must not be written: {content}"
    );
    assert!(content.contains("## User Story\nMember story"), "{content}");
    assert!(
        content.contains("## Description\nMember description"),
        "{content}"
    );

    let out = bin()
        .args(["--dir", p, "req", "view", "REQ-001.001"])
        .output()
        .unwrap();
    let viewed = String::from_utf8_lossy(&out.stdout);
    assert!(viewed.contains("group: REQ-001"), "{viewed}");

    let _ = fs::remove_dir_all(&d);
}

#[test]
fn membership_is_immutable_after_creation() {
    let d = tmpdir("group-immutable");
    init(&d);
    let p = d.to_str().unwrap();
    assert!(
        bin()
            .args(["--dir", p, "req", "create-group", "--title", "G1"])
            .output()
            .unwrap()
            .status
            .success()
    );
    assert!(
        bin()
            .args(["--dir", p, "req", "create-group", "--title", "G2"])
            .output()
            .unwrap()
            .status
            .success()
    );
    assert!(
        bin()
            .args(["--dir", p, "req", "create", "M", "--group", "REQ-001"])
            .output()
            .unwrap()
            .status
            .success()
    );

    // Group link cannot be changed: --group is not an edit option.
    let out = bin()
        .args([
            "--dir",
            p,
            "req",
            "edit",
            "REQ-001.001",
            "--group",
            "REQ-002",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));

    // A normal edit leaves the group link unchanged.
    let out = bin()
        .args([
            "--dir",
            p,
            "req",
            "edit",
            "REQ-001.001",
            "--title",
            "Renamed",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let content = fs::read_to_string(record_path(&d, "REQ-001.001")).unwrap();
    assert!(content.contains("group: REQ-001\n"), "{content}");
    assert!(content.contains("# REQ-001.001 Renamed"), "{content}");

    let _ = fs::remove_dir_all(&d);
}

#[test]
fn archive_and_delete_do_not_cascade_between_groups_and_members() {
    let d = tmpdir("group-cascade");
    init(&d);
    let p = d.to_str().unwrap();
    for title in ["G", "G2"] {
        assert!(
            bin()
                .args(["--dir", p, "req", "create-group", "--title", title])
                .output()
                .unwrap()
                .status
                .success()
        );
    }
    assert!(
        bin()
            .args(["--dir", p, "req", "create", "M1", "--group", "REQ-001"])
            .output()
            .unwrap()
            .status
            .success()
    );
    assert!(
        bin()
            .args(["--dir", p, "req", "create", "M2", "--group", "REQ-001"])
            .output()
            .unwrap()
            .status
            .success()
    );
    let m1 = fs::read_to_string(record_path(&d, "REQ-001.001")).unwrap();
    let m2 = fs::read_to_string(record_path(&d, "REQ-001.002")).unwrap();

    let out = bin()
        .args(["--dir", p, "req", "archive", "REQ-001"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(!record_path(&d, "REQ-001").exists());
    assert!(
        d.join("docs/requirements/business/archived/REQ-001.md")
            .exists()
    );
    assert_eq!(
        fs::read_to_string(record_path(&d, "REQ-001.001")).unwrap(),
        m1
    );
    assert_eq!(
        fs::read_to_string(record_path(&d, "REQ-001.002")).unwrap(),
        m2
    );

    // Archived group is excluded from the list; its active members remain.
    let out = bin().args(["--dir", p, "req", "list"]).output().unwrap();
    let listed = String::from_utf8_lossy(&out.stdout);
    assert!(!listed.contains("REQ-001 G\n"), "{listed}");
    assert!(listed.contains("REQ-001.001 M1"), "{listed}");
    assert!(listed.contains("REQ-001.002 M2"), "{listed}");

    // Deleting a group removes no member.
    let out = bin()
        .args(["--dir", p, "req", "delete", "REQ-001", "--force"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(!record_path(&d, "REQ-001").exists());
    assert!(record_path(&d, "REQ-001.001").exists());
    assert!(record_path(&d, "REQ-001.002").exists());

    // Archiving a member changes neither its group nor another member.
    let out = bin()
        .args(["--dir", p, "req", "archive", "REQ-001.001"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(record_path(&d, "REQ-001.002").exists());
    assert!(
        d.join("docs/requirements/business/archived/REQ-001.001.md")
            .exists()
    );
    assert_eq!(
        fs::read_to_string(record_path(&d, "REQ-001.002")).unwrap(),
        m2
    );

    let _ = fs::remove_dir_all(&d);
}

#[test]
fn list_merges_groups_then_their_members_in_order() {
    let d = tmpdir("group-order");
    init(&d);
    let p = d.to_str().unwrap();
    for (group, title, member) in [("G1", "Group one", "M1"), ("G2", "Group two", "M2")] {
        let out = bin()
            .args(["--dir", p, "req", "create-group", "--title", title])
            .output()
            .unwrap();
        assert!(out.status.success(), "{}", group);
        let group_id = String::from_utf8_lossy(&out.stdout).trim().to_string();
        let out = bin()
            .args(["--dir", p, "req", "create", member, "--group", &group_id])
            .output()
            .unwrap();
        assert!(out.status.success());
    }
    let out = bin().args(["--dir", p, "req", "list"]).output().unwrap();
    let ids: Vec<String> = String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(|line| line.split(' ').next().unwrap_or_default().to_string())
        .collect();
    assert_eq!(
        ids,
        vec!["REQ-001", "REQ-001.001", "REQ-002", "REQ-002.001"]
    );
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn group_and_member_field_sequences_drive_help_order() {
    let d = tmpdir("group-options");
    init(&d);
    let create_group = run_help(&d, &["req", "create-group", "--help"]);
    let story = create_group.find("--story").expect("story option");
    let description = create_group
        .find("--description")
        .expect("description option");
    assert!(story < description, "{create_group}");

    let create = run_help(&d, &["req", "create", "--help"]);
    let story = create.find("--story").expect("story option");
    let description = create.find("--description").expect("description option");
    let group = create.find("--group").expect("group option");
    assert!(story < description && description < group, "{create}");

    let edit = run_help(&d, &["req", "edit", "--help"]);
    assert!(edit.contains("--story"));
    assert!(edit.contains("--description"));
    assert!(
        !edit.contains("--group"),
        "membership is immutable and edit exposes no group option: {edit}"
    );

    let _ = fs::remove_dir_all(&d);
}

#[test]
fn nfr_writes_the_story_field_declared_by_the_seed() {
    let d = tmpdir("nfr-story");
    init(&d);
    let p = d.to_str().unwrap();
    let out = bin()
        .args([
            "--dir",
            p,
            "nfr",
            "create-group",
            "--title",
            "NG",
            "--story",
            "NFR group story",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let group = fs::read_to_string(d.join("docs/requirements/non-functional/NFR-001.md")).unwrap();
    assert!(group.contains("## User Story\nNFR group story"), "{group}");

    let out = bin()
        .args([
            "--dir",
            p,
            "nfr",
            "create",
            "NR",
            "--group",
            "NFR-001",
            "--story",
            "NFR member story",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let member =
        fs::read_to_string(d.join("docs/requirements/non-functional/NFR-001.001.md")).unwrap();
    assert!(
        member.contains("## User Story\nNFR member story"),
        "{member}"
    );
    assert!(member.contains("group: NFR-001\n"), "{member}");

    let _ = fs::remove_dir_all(&d);
}

#[test]
fn req_editor_body_round_trip() {
    let d = tmpdir("editor");
    init(&d);
    let p = d.to_str().unwrap();
    let _ = bin()
        .args(["--dir", p, "req", "create-group", "--title", "G"])
        .output()
        .unwrap();
    let _ = bin()
        .args(["--dir", p, "req", "create", "R", "--group", "REQ-001"])
        .output()
        .unwrap();
    let script = d.join("editor.sh");
    fs::write(
        &script,
        "#!/bin/sh\nprintf '\\n## Notes\\nhello from editor\\n' >> \"$1\"\n",
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script, perms).unwrap();
    }
    let out = bin()
        .env("EDITOR", script.to_str().unwrap())
        .args(["--dir", p, "req", "edit", "REQ-001.001", "--editor"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let out = bin()
        .args(["--dir", p, "req", "view", "REQ-001.001"])
        .output()
        .unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("hello from editor"));
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn sanitization_story_desc_group_id() {
    let d = tmpdir("san2");
    init(&d);
    let p = d.to_str().unwrap();
    let _ = bin()
        .args(["--dir", p, "req", "create-group", "--title", "G"])
        .output()
        .unwrap();
    for bad in ["a/b", "x\\y", "   ", "a\nb"] {
        let out = bin()
            .args([
                "--dir",
                p,
                "req",
                "create-group",
                "--title",
                "G2",
                "--story",
                bad,
            ])
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(4), "story bad={}", bad);
    }
    for bad in ["a/b", "x\\y", "   ", "a\nb"] {
        let out = bin()
            .args([
                "--dir",
                p,
                "req",
                "create-group",
                "--title",
                "G3",
                "--description",
                bad,
            ])
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(4), "desc bad={}", bad);
    }
    for bad in ["a/b", "x\\y", "   ", "a\nb"] {
        let out = bin()
            .args(["--dir", p, "req", "create", "R", "--group", bad])
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(4), "group bad={}", bad);
    }
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn req_type_mismatch_exits_four() {
    let d = tmpdir("reqtm");
    init(&d);
    let p = d.to_str().unwrap();
    let out = bin()
        .args(["--dir", p, "nfr", "create-group", "--title", "NG"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "NFR-001");
    let out = bin()
        .args(["--dir", p, "req", "create", "R", "--group", "NFR-001"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn nfr_type_mismatch_exits_four() {
    let d = tmpdir("nfrtm");
    init(&d);
    let p = d.to_str().unwrap();
    let out = bin()
        .args(["--dir", p, "req", "create-group", "--title", "G"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "REQ-001");
    let out = bin()
        .args(["--dir", p, "nfr", "create", "NR", "--group", "REQ-001"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn create_group_accepts_positional_and_flag_titles_additively() {
    let d = tmpdir("grouptitle");
    init(&d);
    let p = d.to_str().unwrap();
    // The positional TITLE is the primary form.
    let out = bin()
        .args(["--dir", p, "req", "create-group", "Grouped"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "REQ-001");
    let group = fs::read_to_string(d.join("docs/requirements/business/REQ-001.md")).unwrap();
    assert!(group.contains("# REQ-001 Grouped"), "{group}");

    // `--title` keeps its documented name and meaning.
    let out = bin()
        .args(["--dir", p, "req", "create-group", "--title", "Flagged"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "REQ-002");

    // Both sources on one invocation are rejected and write nothing.
    let out = bin()
        .args(["--dir", p, "req", "create-group", "A", "--title", "B"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("conflicting title sources"),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(!d.join("docs/requirements/business/REQ-003.md").exists());
    let _ = fs::remove_dir_all(&d);
}
