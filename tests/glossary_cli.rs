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
            "--title",
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
        .args(["--dir", p, "glossary", "archive", "GLO-001"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(d.join("docs/glossary/archived/GLO-001.md").exists());

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
        .args(["--dir", p, "glossary", "archive", "GLO-001"])
        .output()
        .unwrap();
    assert!(out.status.success());

    let out = bin()
        .args(["--dir", p, "glossary", "view", "NOPE"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(3));

    let _ = fs::remove_dir_all(&d);
}

#[test]
fn glossary_edit_archived_allowed() {
    let d = tmpdir("editdep");
    init(&d);
    let p = d.to_str().unwrap();
    let _ = bin()
        .args(["--dir", p, "glossary", "create", "T", "--definition", "d"])
        .output()
        .unwrap();
    let _ = bin()
        .args(["--dir", p, "glossary", "archive", "GLO-001"])
        .output()
        .unwrap();
    let out = bin()
        .args([
            "--dir",
            p,
            "glossary",
            "edit",
            "GLO-001",
            "--title",
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
fn glossary_sanitization() {
    let d = tmpdir("san");
    init(&d);
    let p = d.to_str().unwrap();

    for bad in ["a/b", "x\\y", "   ", "a\nb"] {
        let out = bin()
            .args(["--dir", p, "glossary", "create", bad, "--definition", "ok"])
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
            "OK",
            "--definition",
            "x/y",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));

    let _ = bin()
        .args(["--dir", p, "glossary", "create", "OK", "--definition", "ok"])
        .output()
        .unwrap();

    let out = bin()
        .args(["--dir", p, "glossary", "view", "a/b"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));

    let out = bin()
        .args(["--dir", p, "glossary", "edit", "GLO-001", "--title", "x\\y"])
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
        .args(["--dir", p, "glossary", "create", "D", "--definition", "d"])
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
        .args(["--dir", p, "glossary", "archive", "GLO-001"])
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
fn glossary_missing_args_exit_4() {
    let d = tmpdir("missing");
    init(&d);
    let p = d.to_str().unwrap();
    // Title is a record invariant validated by the engine (TC-126 form).
    let out = bin()
        .args(["--dir", p, "glossary", "create", "--definition", "d"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));
    let message = String::from_utf8_lossy(&out.stderr);
    assert!(message.contains("title"), "{message}");
    // A missing required field is the same validation outcome.
    let out = bin()
        .args(["--dir", p, "glossary", "create", "t"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));
    let message = String::from_utf8_lossy(&out.stderr);
    assert!(message.contains("definition"), "{message}");
    assert!(!d.join("docs/glossary/GLO-001.md").exists());
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn glossary_term_is_the_record_title() {
    // TC-138: the term is the record title and the definition its field.
    let d = tmpdir("term-title");
    init(&d);
    let p = d.to_str().unwrap();
    let out = bin()
        .args([
            "--dir",
            p,
            "glossary",
            "create",
            "Cache",
            "--definition",
            "A store of reusable results",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "GLO-001");

    let content = fs::read_to_string(d.join("docs/glossary/GLO-001.md")).unwrap();
    assert!(content.contains("id: GLO-001"), "{content}");
    assert!(content.contains("title: Cache"), "{content}");
    // The seed create path does not write `status`; the term/title contract
    // adds no status key, so only the stored title and body are asserted here.
    assert!(!content.contains("term:"), "{content}");
    assert!(content.contains("# GLO-001 Cache"), "{content}");
    assert!(
        content.contains("## Definition\nA store of reusable results"),
        "{content}"
    );
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn glossary_title_sources_are_exclusive_and_term_is_a_usage_error() {
    // TC-125: both title sources on one create is one validation outcome.
    let d = tmpdir("term-title-conflict");
    init(&d);
    let p = d.to_str().unwrap();
    let out = bin()
        .args([
            "--dir",
            p,
            "glossary",
            "create",
            "Cache",
            "--title",
            "Caching",
            "--definition",
            "def",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));
    let message = String::from_utf8_lossy(&out.stderr);
    assert!(message.contains("conflicting title sources"), "{message}");
    assert!(!d.join("docs/glossary/GLO-001.md").exists());

    // `--term` is gone from every document command: it is a usage error.
    let out = bin()
        .args([
            "--dir",
            p,
            "glossary",
            "create",
            "--term",
            "Cache",
            "--definition",
            "def",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(!d.join("docs/glossary/GLO-001.md").exists());

    let out = bin()
        .args([
            "--dir",
            p,
            "glossary",
            "create",
            "Cache",
            "--definition",
            "def",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());

    let out = bin()
        .args(["--dir", p, "glossary", "edit", "GLO-001", "--term", "X"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    let content = fs::read_to_string(d.join("docs/glossary/GLO-001.md")).unwrap();
    assert!(content.contains("title: Cache"), "{content}");
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn glossary_edit_cannot_clear_the_title() {
    let d = tmpdir("term-title-clear");
    init(&d);
    let p = d.to_str().unwrap();
    let _ = bin()
        .args([
            "--dir",
            p,
            "glossary",
            "create",
            "Cache",
            "--definition",
            "def",
        ])
        .output()
        .unwrap();
    let path = d.join("docs/glossary/GLO-001.md");
    let before = fs::read_to_string(&path).unwrap();
    for bad in ["", "   ", "two\nlines"] {
        let out = bin()
            .args(["--dir", p, "glossary", "edit", "GLO-001", "--title", bad])
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(4), "bad title {bad:?}");
        assert_eq!(fs::read_to_string(&path).unwrap(), before);
    }
    let out = bin()
        .args([
            "--dir", p, "glossary", "edit", "GLO-001", "--title", "Cache 2",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let content = fs::read_to_string(&path).unwrap();
    assert!(content.contains("title: \"Cache 2\""), "{content}");
    assert!(content.contains("# GLO-001 Cache 2"), "{content}");
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
    assert_eq!(out.status.code(), Some(2));
    let out = bin()
        .args(["--dir", p, "glossary", "archive", "NOPE"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(3));
    let _ = fs::remove_dir_all(&d);
}
