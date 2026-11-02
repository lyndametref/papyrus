use std::fs;
use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_papyrus"))
}

fn tmpdir(name: &str) -> std::path::PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("papyrus-milestone-{}", name));
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
fn consolidated_req_and_nfreq_lifecycle() {
    let d = tmpdir("consolidated");
    init(&d);
    let p = d.to_str().unwrap();

    // req group + requirement
    let out = bin()
        .args(["--dir", p, "req", "create-group", "--title", "RG"])
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

    // nfreq group + requirement (same project, distinct namespace)
    let out = bin()
        .args(["--dir", p, "nfreq", "create-group", "--title", "NG"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "NFR-001");

    let out = bin()
        .args(["--dir", p, "nfreq", "create", "NR", "--group", "NFR-001"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "NFR-001.001");

    // list ordering shows both ids
    let out = bin().args(["--dir", p, "req", "list"]).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("REQ-001 RG"));
    assert!(s.contains("REQ-001.001 R"));

    let out = bin().args(["--dir", p, "nfreq", "list"]).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("NFR-001 NG"));
    assert!(s.contains("NFR-001.001 NR"));

    // view full persisted text
    let out = bin()
        .args(["--dir", p, "req", "view", "REQ-001.001"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("## Group\nREQ-001 RG"));

    // edit updates only supplied field, id preserved
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

    // deprecate hides from list
    let out = bin()
        .args(["--dir", p, "req", "deprecate", "REQ-001.001"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let out = bin().args(["--dir", p, "req", "list"]).output().unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(!s.contains("REQ-001.001"));

    // view deprecated still allowed
    let out = bin()
        .args(["--dir", p, "req", "view", "REQ-001.001"])
        .output()
        .unwrap();
    assert!(out.status.success());

    // restore returns same id and reappears
    let out = bin()
        .args(["--dir", p, "req", "restore", "REQ-001.001"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let out = bin().args(["--dir", p, "req", "list"]).output().unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("REQ-001.001 R2"));

    // nfreq parity: deprecate/restore
    let out = bin()
        .args(["--dir", p, "nfreq", "deprecate", "NFR-001.001"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let out = bin().args(["--dir", p, "nfreq", "list"]).output().unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(!s.contains("NFR-001.001"));
    let out = bin()
        .args(["--dir", p, "nfreq", "restore", "NFR-001.001"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let out = bin().args(["--dir", p, "nfreq", "list"]).output().unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("NFR-001.001 NR"));

    let _ = fs::remove_dir_all(&d);
}

#[test]
fn id_format_padding_and_ordering() {
    let d = tmpdir("padding");
    init(&d);
    let p = d.to_str().unwrap();

    for i in 1..=3 {
        let out = bin()
            .args([
                "--dir",
                p,
                "req",
                "create-group",
                "--title",
                &format!("G{}", i),
            ])
            .output()
            .unwrap();
        assert!(out.status.success());
        assert_eq!(
            String::from_utf8_lossy(&out.stdout).trim(),
            format!("REQ-{:03}", i)
        );
    }

    // list is in id order
    let out = bin().args(["--dir", p, "req", "list"]).output().unwrap();
    assert!(out.status.success());
    let binding = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = binding.lines().collect();
    // group lines only (top-level groups); check ordering of group ids
    let group_lines: Vec<&str> = lines.iter().filter(|l| !l.contains('.')).copied().collect();
    assert_eq!(group_lines.len(), 3);
    assert!(group_lines[0].starts_with("REQ-001"));
    assert!(group_lines[1].starts_with("REQ-002"));
    assert!(group_lines[2].starts_with("REQ-003"));

    // requirement seq padding under a group
    let _ = bin()
        .args(["--dir", p, "req", "create", "R", "--group", "REQ-002"])
        .output()
        .unwrap();
    let out = bin()
        .args(["--dir", p, "req", "create", "R2", "--group", "REQ-002"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "REQ-002.002");

    let _ = fs::remove_dir_all(&d);
}

#[test]
fn id_preservation_across_deprecate_restore() {
    let d = tmpdir("preserve");
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

    // deprecate then restore keeps the same id
    let out = bin()
        .args(["--dir", p, "req", "deprecate", "REQ-001.001"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let out = bin()
        .args(["--dir", p, "req", "restore", "REQ-001.001"])
        .output()
        .unwrap();
    assert!(out.status.success());

    // new requirement gets next seq, not reusing .001
    let out = bin()
        .args(["--dir", p, "req", "create", "R2", "--group", "REQ-001"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "REQ-001.002");

    // restore id still present
    let out = bin().args(["--dir", p, "req", "list"]).output().unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("REQ-001.001"));
    assert!(s.contains("REQ-001.002"));

    let _ = fs::remove_dir_all(&d);
}

#[test]
fn exit_code_matrix() {
    let d = tmpdir("exitcodes");
    init(&d);
    let p = d.to_str().unwrap();

    // 2: missing required --group
    let out = bin()
        .args(["--dir", p, "req", "create", "R"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));

    // 3: unknown group
    let out = bin()
        .args(["--dir", p, "req", "create", "R", "--group", "REQ-999"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(3));

    // 3: unknown view id
    let out = bin()
        .args(["--dir", p, "req", "view", "NOPE"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(3));

    // 4: validation / conflict (title conflict)
    let out = bin()
        .args([
            "--dir", p, "req", "create", "A", "--title", "B", "--group", "REQ-999",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));

    // 4: sanitization reject
    let out = bin()
        .args(["--dir", p, "req", "create-group", "--title", "a/b"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));

    // 4: type mismatch (nfreq group used by req)
    let _ = bin()
        .args(["--dir", p, "nfreq", "create-group", "--title", "NG"])
        .output()
        .unwrap();
    let out = bin()
        .args(["--dir", p, "req", "create", "R", "--group", "NFR-001"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));

    // 4: state conflict (double deprecate)
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
    let out = bin()
        .args(["--dir", p, "req", "deprecate", "REQ-001.001"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));

    // 1: IO/boundary (config escape)
    let root = tmpdir("exitcodes-bnd");
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

    let _ = fs::remove_dir_all(&d);
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn nfreq_exit_codes_parity() {
    let d = tmpdir("nfreqexit");
    init(&d);
    let p = d.to_str().unwrap();

    let out = bin()
        .args(["--dir", p, "nfreq", "create", "NR"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));

    let out = bin()
        .args(["--dir", p, "nfreq", "create", "NR", "--group", "NFR-999"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(3));

    let out = bin()
        .args(["--dir", p, "nfreq", "view", "NOPE"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(3));

    let _ = bin()
        .args(["--dir", p, "req", "create-group", "--title", "G"])
        .output()
        .unwrap();
    let out = bin()
        .args(["--dir", p, "nfreq", "create", "NR", "--group", "REQ-001"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));

    let _ = fs::remove_dir_all(&d);
}
