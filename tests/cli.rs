use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_papyrus"))
}

#[test]
fn help_long_exits_zero() {
    let out = bin().arg("--help").output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Usage:"));
}

#[test]
fn help_short_exits_zero() {
    let out = bin().arg("-h").output().unwrap();
    assert!(out.status.success());
}

#[test]
fn version_prints_package_version() {
    let out = bin().arg("--version").output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(stdout.trim(), "papyrus 0.1.0");
}

#[test]
fn unknown_flag_exits_two_and_usage_on_stderr() {
    let out = bin().arg("--definitely-unknown").output().unwrap();
    assert_eq!(out.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("Usage:"));
}

use std::fs;
use std::path::PathBuf;

fn tmpdir(name: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("papyrus-test-{}", name));
    let _ = fs::remove_dir_all(&p);
    fs::create_dir_all(&p).unwrap();
    p
}

#[test]
fn init_creates_config() {
    let d = tmpdir("init1");
    let out = bin()
        .args(["--dir", d.to_str().unwrap(), "init", "Proj"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{:?}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(d.join("papyrus.config.yml").exists());
    let cfg = fs::read_to_string(d.join("papyrus.config.yml")).unwrap();
    assert!(cfg.contains("project_name: Proj"));
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn init_no_git_skips_git() {
    let d = tmpdir("nogit");
    let out = bin()
        .args(["--dir", d.to_str().unwrap(), "init", "Proj", "--no-git"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(!d.join(".git").exists());
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn init_nonempty_without_force_exits_four() {
    let d = tmpdir("nonempty");
    fs::write(d.join("file.txt"), "x").unwrap();
    let out = bin()
        .args(["--dir", d.to_str().unwrap(), "init", "Proj"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn init_force_leaves_files() {
    let d = tmpdir("force");
    fs::write(d.join("file.txt"), "x").unwrap();
    let out = bin()
        .args(["--dir", d.to_str().unwrap(), "init", "Proj", "--force"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(fs::read_to_string(d.join("file.txt")).unwrap(), "x");
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn config_list_and_get() {
    let d = tmpdir("cfg");
    bin()
        .args(["--dir", d.to_str().unwrap(), "init", "Proj", "--no-git"])
        .output()
        .unwrap();
    let out = bin()
        .args(["--dir", d.to_str().unwrap(), "config", "list"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("task_prefix: TASK"));
    let out = bin()
        .args(["--dir", d.to_str().unwrap(), "config", "get", "task_prefix"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "TASK");
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn config_set_unset_roundtrip() {
    let d = tmpdir("set");
    bin()
        .args(["--dir", d.to_str().unwrap(), "init", "Proj", "--no-git"])
        .output()
        .unwrap();
    let out = bin()
        .args([
            "--dir",
            d.to_str().unwrap(),
            "config",
            "set",
            "task_prefix",
            "PAP",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let out = bin()
        .args(["--dir", d.to_str().unwrap(), "config", "get", "task_prefix"])
        .output()
        .unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "PAP");
    let out = bin()
        .args([
            "--dir",
            d.to_str().unwrap(),
            "config",
            "unset",
            "task_prefix",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let out = bin()
        .args(["--dir", d.to_str().unwrap(), "config", "get", "task_prefix"])
        .output()
        .unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "TASK");
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn config_unknown_key_exits_three() {
    let d = tmpdir("unk");
    bin()
        .args(["--dir", d.to_str().unwrap(), "init", "Proj", "--no-git"])
        .output()
        .unwrap();
    let out = bin()
        .args(["--dir", d.to_str().unwrap(), "config", "get", "nope.key"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(3));
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn config_json_error() {
    let d = tmpdir("jsonerr");
    bin()
        .args(["--dir", d.to_str().unwrap(), "init", "Proj", "--no-git"])
        .output()
        .unwrap();
    let out = bin()
        .args([
            "--dir",
            d.to_str().unwrap(),
            "--json",
            "config",
            "get",
            "nope",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(3));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("\"code\":3") || stderr.contains("\"code\": 3"));
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn version_output() {
    let out = bin().arg("--version").output().unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "papyrus 0.1.0");
}

#[test]
fn config_explicit_config_outside_project_exits_one() {
    let root = tmpdir("boundary1");
    let proj = root.join("proj");
    let outside = root.join("outside");
    fs::create_dir_all(&proj).unwrap();
    fs::create_dir_all(&outside).unwrap();
    bin()
        .args(["--dir", proj.to_str().unwrap(), "init", "Proj", "--no-git"])
        .output()
        .unwrap();
    let out = bin()
        .args([
            "--dir",
            proj.to_str().unwrap(),
            "--config",
            outside.join("evil.yml").to_str().unwrap(),
            "config",
            "list",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(!outside.join("evil.yml").exists());
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn config_symlink_escape_exits_one() {
    let root = tmpdir("boundary2");
    let proj = root.join("proj");
    let outside = root.join("outside");
    fs::create_dir_all(&proj).unwrap();
    fs::create_dir_all(&outside).unwrap();
    bin()
        .args(["--dir", proj.to_str().unwrap(), "init", "Proj", "--no-git"])
        .output()
        .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let _ = symlink(outside.join("evil.yml"), proj.join("escape.yml"));
    }
    let out = bin()
        .args([
            "--dir",
            proj.to_str().unwrap(),
            "--config",
            proj.join("escape.yml").to_str().unwrap(),
            "config",
            "list",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn config_sanitization_rejects_bad_value() {
    let d = tmpdir("san1");
    bin()
        .args(["--dir", d.to_str().unwrap(), "init", "Proj", "--no-git"])
        .output()
        .unwrap();
    for bad in [".", "..", "a/b", "x\\y"] {
        let out = bin()
            .args([
                "--dir",
                d.to_str().unwrap(),
                "config",
                "set",
                "task_prefix",
                bad,
            ])
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(4), "bad={}", bad);
    }
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn config_sanitization_rejects_bad_key() {
    let d = tmpdir("san2");
    bin()
        .args(["--dir", d.to_str().unwrap(), "init", "Proj", "--no-git"])
        .output()
        .unwrap();
    let out = bin()
        .args(["--dir", d.to_str().unwrap(), "config", "get", ".."])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn config_project_name_defaults_to_dir_name() {
    let d = tmpdir("defname");
    bin()
        .args(["--dir", d.to_str().unwrap(), "init", "Proj", "--no-git"])
        .output()
        .unwrap();
    bin()
        .args([
            "--dir",
            d.to_str().unwrap(),
            "config",
            "unset",
            "project_name",
        ])
        .output()
        .unwrap();
    let out = bin()
        .args([
            "--dir",
            d.to_str().unwrap(),
            "config",
            "get",
            "project_name",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(
        String::from_utf8_lossy(&out.stdout).trim(),
        d.file_name().unwrap().to_str().unwrap()
    );
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn config_editor_defaults_to_env() {
    let d = tmpdir("defed");
    bin()
        .args(["--dir", d.to_str().unwrap(), "init", "Proj", "--no-git"])
        .output()
        .unwrap();
    let out = bin()
        .env("VISUAL", "myeditor")
        .args(["--dir", d.to_str().unwrap(), "config", "get", "editor"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "myeditor");
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn config_editor_json_null_when_unset() {
    let d = tmpdir("defedjson");
    bin()
        .args(["--dir", d.to_str().unwrap(), "init", "Proj", "--no-git"])
        .output()
        .unwrap();
    let out = bin()
        .args([
            "--dir",
            d.to_str().unwrap(),
            "--json",
            "config",
            "get",
            "editor",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "null");
    let _ = fs::remove_dir_all(&d);
}
