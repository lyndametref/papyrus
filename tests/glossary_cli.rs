use std::fs;
use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_papyrus"))
}

fn tmpdir(name: &str) -> std::path::PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("papyrus-glostest-{}", name));
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
fn glossary_basic_lifecycle() {
    let d = tmpdir("life");
    init(&d);
    let p = d.to_str().unwrap();

    let out = bin()
        .args([
            "--dir",
            p,
            "glossary",
            "create",
            "--term",
            "First Term",
            "--definition",
            "def text",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "GLO-001");

    let out = bin()
        .args(["--dir", p, "glossary", "list"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("GLO-001 First Term"));

    let out = bin()
        .args(["--dir", p, "glossary", "view", "GLO-001"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("# GLO-001 First Term"));
    assert!(s.contains("## Definition\ndef text"));

    let out = bin()
        .args([
            "--dir",
            p,
            "glossary",
            "edit",
            "GLO-001",
            "--term",
            "First Term Updated",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let out = bin()
        .args(["--dir", p, "glossary", "view", "GLO-001"])
        .output()
        .unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("# GLO-001 First Term Updated"));
    assert!(s.contains("## Definition\ndef text"));

    let out = bin()
        .args([
            "--dir",
            p,
            "glossary",
            "edit",
            "GLO-001",
            "--definition",
            "def2",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let out = bin()
        .args(["--dir", p, "glossary", "view", "GLO-001"])
        .output()
        .unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("## Definition\ndef2"));

    let out = bin()
        .args(["--dir", p, "glossary", "deprecate", "GLO-001"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(d.join("docs/glossary/deprecated/GLO-001.md").exists());

    let out = bin()
        .args(["--dir", p, "glossary", "list"])
        .output()
        .unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(!s.contains("GLO-001"));

    let out = bin()
        .args(["--dir", p, "glossary", "view", "GLO-001"])
        .output()
        .unwrap();
    assert!(out.status.success());

    let out = bin()
        .args(["--dir", p, "glossary", "deprecate", "GLO-001"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));

    let out = bin()
        .args(["--dir", p, "glossary", "restore", "GLO-001"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(d.join("docs/glossary/GLO-001.md").exists());
    assert!(!d.join("docs/glossary/deprecated/GLO-001.md").exists());

    let out = bin()
        .args(["--dir", p, "glossary", "list"])
        .output()
        .unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("GLO-001"));

    let out = bin()
        .args(["--dir", p, "glossary", "restore", "GLO-001"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));

    let out = bin()
        .args(["--dir", p, "glossary", "view", "NOPE"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(3));

    let _ = fs::remove_dir_all(&d);
}

#[test]
fn glossary_edit_deprecated_allowed() {
    let d = tmpdir("editdep");
    init(&d);
    let p = d.to_str().unwrap();
    let _ = bin()
        .args([
            "--dir",
            p,
            "glossary",
            "create",
            "--term",
            "T",
            "--definition",
            "d",
        ])
        .output()
        .unwrap();
    let _ = bin()
        .args(["--dir", p, "glossary", "deprecate", "GLO-001"])
        .output()
        .unwrap();
    let out = bin()
        .args([
            "--dir",
            p,
            "glossary",
            "edit",
            "GLO-001",
            "--term",
            "T2",
            "--definition",
            "d2",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let out = bin()
        .args(["--dir", p, "glossary", "view", "GLO-001"])
        .output()
        .unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("# GLO-001 T2"));
    assert!(s.contains("## Definition\nd2"));
    let out = bin()
        .args(["--dir", p, "glossary", "list"])
        .output()
        .unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(!s.contains("GLO-001"));
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn glossary_id_not_reused() {
    let d = tmpdir("reuse");
    init(&d);
    let p = d.to_str().unwrap();

    let _ = bin()
        .args([
            "--dir",
            p,
            "glossary",
            "create",
            "--term",
            "D1",
            "--definition",
            "d",
        ])
        .output()
        .unwrap();
    let _ = bin()
        .args(["--dir", p, "glossary", "deprecate", "GLO-001"])
        .output()
        .unwrap();
    let out = bin()
        .args([
            "--dir",
            p,
            "glossary",
            "create",
            "--term",
            "D2",
            "--definition",
            "d",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "GLO-002");

    let out = bin()
        .args(["--dir", p, "glossary", "restore", "GLO-001"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let out = bin()
        .args(["--dir", p, "glossary", "view", "GLO-001"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("# GLO-001 D1"));

    let _ = fs::remove_dir_all(&d);
}

#[test]
fn glossary_sanitization() {
    let d = tmpdir("san");
    init(&d);
    let p = d.to_str().unwrap();

    for bad in ["a/b", "x\\y", "   ", "a\nb"] {
        let out = bin()
            .args([
                "--dir",
                p,
                "glossary",
                "create",
                "--term",
                bad,
                "--definition",
                "ok",
            ])
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(4), "term bad={}", bad);
    }

    let out = bin()
        .args([
            "--dir",
            p,
            "glossary",
            "create",
            "--term",
            "OK",
            "--definition",
            "x/y",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));

    let _ = bin()
        .args([
            "--dir",
            p,
            "glossary",
            "create",
            "--term",
            "OK",
            "--definition",
            "ok",
        ])
        .output()
        .unwrap();

    let out = bin()
        .args(["--dir", p, "glossary", "view", "a/b"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));

    let out = bin()
        .args(["--dir", p, "glossary", "edit", "GLO-001", "--term", "x\\y"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));

    let out = bin()
        .args([
            "--dir",
            p,
            "glossary",
            "edit",
            "GLO-001",
            "--definition",
            "a/b",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));

    let _ = fs::remove_dir_all(&d);
}

#[test]
fn glossary_project_directory_boundary() {
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
            "glossary",
            "list",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));

    let p = proj.to_str().unwrap();
    let _ = bin()
        .args([
            "--dir",
            p,
            "glossary",
            "create",
            "--term",
            "D",
            "--definition",
            "d",
        ])
        .output()
        .unwrap();

    let target = proj.join("docs/glossary/GLO-001.md");
    let outside_file = outside.join("evil.md");
    fs::write(&outside_file, "secret").unwrap();
    let _ = fs::remove_file(&target);
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let _ = symlink(&outside_file, &target);
    }
    let out = bin()
        .args(["--dir", p, "glossary", "view", "GLO-001"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));

    let out = bin()
        .args(["--dir", p, "glossary", "deprecate", "GLO-001"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn glossary_json_output() {
    let d = tmpdir("json");
    init(&d);
    let p = d.to_str().unwrap();
    let out = bin()
        .args([
            "--dir",
            p,
            "--json",
            "glossary",
            "create",
            "--term",
            "Foo",
            "--definition",
            "def",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(
        String::from_utf8_lossy(&out.stdout).trim(),
        r#"{"id":"GLO-001"}"#
    );

    let out = bin()
        .args(["--dir", p, "--json", "glossary", "list"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("\"id\":\"GLO-001\""));

    let out = bin()
        .args(["--dir", p, "--json", "glossary", "view", "GLO-001"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("\"content\""));

    let _ = fs::remove_dir_all(&d);
}

#[test]
fn glossary_missing_args_exit_2() {
    let d = tmpdir("missing");
    init(&d);
    let p = d.to_str().unwrap();
    let out = bin()
        .args(["--dir", p, "glossary", "create", "--definition", "d"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    let out = bin()
        .args(["--dir", p, "glossary", "create", "--term", "t"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn glossary_restore_unknown_exit_3() {
    let d = tmpdir("restunk");
    init(&d);
    let p = d.to_str().unwrap();
    let out = bin()
        .args(["--dir", p, "glossary", "restore", "NOPE"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(3));
    let out = bin()
        .args(["--dir", p, "glossary", "deprecate", "NOPE"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(3));
    let _ = fs::remove_dir_all(&d);
}
