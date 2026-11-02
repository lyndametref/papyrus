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
