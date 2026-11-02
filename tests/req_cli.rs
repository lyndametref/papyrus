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
            "-d",
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
    assert!(s.contains("## Group\nREQ-001 G"));

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
    assert!(s.contains("## Group\nREQ-001 G"));

    // deprecate hides
    let out = bin()
        .args(["--dir", p, "req", "deprecate", "REQ-001.001"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let out = bin().args(["--dir", p, "req", "list"]).output().unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(!s.contains("REQ-001.001"));

    // view deprecated allowed
    let out = bin()
        .args(["--dir", p, "req", "view", "REQ-001.001"])
        .output()
        .unwrap();
    assert!(out.status.success());

    // restore
    let out = bin()
        .args(["--dir", p, "req", "restore", "REQ-001.001"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let out = bin().args(["--dir", p, "req", "list"]).output().unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("REQ-001.001 R2"));

    let _ = fs::remove_dir_all(&d);
}

#[test]
fn req_errors() {
    let d = tmpdir("errs");
    init(&d);
    let p = d.to_str().unwrap();
    // missing group usage exit 2
    let out = bin()
        .args(["--dir", p, "req", "create", "R"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    // unknown group exit 3
    let out = bin()
        .args(["--dir", p, "req", "create", "R", "--group", "REQ-999"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(3));
    // conflicting title exit 4
    let out = bin()
        .args([
            "--dir", p, "req", "create", "A", "--title", "B", "--group", "REQ-999",
        ])
        .output()
        .unwrap();
    // group not found checked first? Actually title conflict resolved before group? Our create resolves title before loading config? It sanitizes title first, so conflict -> 4 even if group missing. Accept 4.
    assert_eq!(out.status.code(), Some(4));
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
        .args(["--dir", p, "req", "deprecate", "REQ-001.001"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let out = bin()
        .args(["--dir", p, "req", "deprecate", "REQ-001.001"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn nfreq_basic() {
    let d = tmpdir("nfreq");
    init(&d);
    let p = d.to_str().unwrap();
    let out = bin()
        .args([
            "--dir",
            p,
            "nfreq",
            "create-group",
            "--title",
            "NG",
            "-d",
            "desc",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "NFR-001");
    let out = bin()
        .args([
            "--dir", p, "nfreq", "create", "NR", "-d", "nd", "--group", "NFR-001",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "NFR-001.001");
    let out = bin().args(["--dir", p, "nfreq", "list"]).output().unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("NFR-001 NG"));
    assert!(s.contains("NFR-001.001 NR"));
    let out = bin()
        .args(["--dir", p, "--json", "nfreq", "view", "NFR-001.001"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_str(&String::from_utf8_lossy(&out.stdout)).unwrap();
    assert_eq!(v["id"], "NFR-001.001");
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn id_not_reused() {
    let d = tmpdir("reuse");
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
    let _ = bin()
        .args(["--dir", p, "req", "deprecate", "REQ-001.001"])
        .output()
        .unwrap();
    // restore then create new should get .002 not .001
    let _ = bin()
        .args(["--dir", p, "req", "restore", "REQ-001.001"])
        .output()
        .unwrap();
    let out = bin()
        .args(["--dir", p, "req", "create", "R2", "--group", "REQ-001"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "REQ-001.002");
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn sanitization_rejects_bad() {
    let d = tmpdir("san");
    init(&d);
    let p = d.to_str().unwrap();
    for bad in ["a/b", "x\\y", "   ", "a\nb"] {
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
    // deprecate should reject
    let out = bin()
        .args(["--dir", p, "req", "deprecate", "REQ-001.001"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let _ = fs::remove_dir_all(&root);
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
                "-d",
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
