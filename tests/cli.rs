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
