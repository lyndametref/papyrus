use std::fs;
use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_papyrus"))
}

fn tmpdir(name: &str) -> std::path::PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("papyrus-reloc-{}", name));
    let _ = fs::remove_dir_all(&p);
    fs::create_dir_all(&p).unwrap();
    p
}

#[test]
fn relocate_backlog_success_moves_all_and_removes_source() {
    let d = tmpdir("succ");
    let proj = d.join("proj");
    fs::create_dir_all(&proj).unwrap();
    assert!(
        bin()
            .args(["--dir", proj.to_str().unwrap(), "init", "Proj", "--no-git"])
            .output()
            .unwrap()
            .status
            .success()
    );
    // populate source folder with files and a subfolder
    fs::write(proj.join("backlog/extra.txt"), "x").unwrap();
    fs::create_dir_all(proj.join("backlog/sub")).unwrap();
    fs::write(proj.join("backlog/sub/f.txt"), "y").unwrap();

    let out = bin()
        .args([
            "--dir",
            proj.to_str().unwrap(),
            "config",
            "set",
            "backlog_directory",
            "archive",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(proj.join("archive/extra.txt").exists());
    assert!(proj.join("archive/sub/f.txt").exists());
    assert!(
        !proj.join("backlog").exists()
            || fs::read_dir(proj.join("backlog")).unwrap().next().is_none()
    );
    // config persisted
    let cfg = fs::read_to_string(proj.join("papyrus.config.yml")).unwrap();
    assert!(cfg.contains("backlog_directory: archive"));
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn relocate_destination_conflict_exits_four_no_changes() {
    let d = tmpdir("conflict");
    let proj = d.join("proj");
    fs::create_dir_all(&proj).unwrap();
    assert!(
        bin()
            .args(["--dir", proj.to_str().unwrap(), "init", "Proj", "--no-git"])
            .output()
            .unwrap()
            .status
            .success()
    );
    fs::write(proj.join("backlog/extra.txt"), "x").unwrap();
    fs::create_dir_all(proj.join("archive")).unwrap();
    fs::write(proj.join("archive/extra.txt"), "y").unwrap();

    let out = bin()
        .args([
            "--dir",
            proj.to_str().unwrap(),
            "config",
            "set",
            "backlog_directory",
            "archive",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));
    // no changes
    assert!(proj.join("backlog/extra.txt").exists());
    assert_eq!(
        fs::read_to_string(proj.join("archive/extra.txt")).unwrap(),
        "y"
    );
    let cfg = fs::read_to_string(proj.join("papyrus.config.yml")).unwrap();
    assert!(cfg.contains("backlog_directory: backlog"));
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn relocate_unchanged_config_on_failure() {
    let d = tmpdir("unchanged");
    let proj = d.join("proj");
    fs::create_dir_all(&proj).unwrap();
    assert!(
        bin()
            .args(["--dir", proj.to_str().unwrap(), "init", "Proj", "--no-git"])
            .output()
            .unwrap()
            .status
            .success()
    );
    // invalid folder value rejected
    let before = fs::read_to_string(proj.join("papyrus.config.yml")).unwrap();
    let out = bin()
        .args([
            "--dir",
            proj.to_str().unwrap(),
            "config",
            "set",
            "backlog_directory",
            "../escape",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));
    let after = fs::read_to_string(proj.join("papyrus.config.yml")).unwrap();
    assert_eq!(before, after);
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn relocate_multi_segment_folder_accepted() {
    let d = tmpdir("multiseg");
    let proj = d.join("proj");
    fs::create_dir_all(&proj).unwrap();
    assert!(
        bin()
            .args(["--dir", proj.to_str().unwrap(), "init", "Proj", "--no-git"])
            .output()
            .unwrap()
            .status
            .success()
    );
    fs::write(proj.join("backlog/extra.txt"), "x").unwrap();
    let out = bin()
        .args([
            "--dir",
            proj.to_str().unwrap(),
            "config",
            "set",
            "backlog_directory",
            "a/b/c",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(proj.join("a/b/c/extra.txt").exists());
    assert!(
        !proj.join("backlog").exists()
            || fs::read_dir(proj.join("backlog")).unwrap().next().is_none()
    );
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn relocate_nested_destination_conflict_exits_four_no_changes() {
    let d = tmpdir("nestedconflict");
    let proj = d.join("proj");
    fs::create_dir_all(&proj).unwrap();
    assert!(
        bin()
            .args(["--dir", proj.to_str().unwrap(), "init", "Proj", "--no-git"])
            .output()
            .unwrap()
            .status
            .success()
    );
    fs::create_dir_all(proj.join("backlog/sub")).unwrap();
    fs::write(proj.join("backlog/sub/f.txt"), "x").unwrap();
    fs::create_dir_all(proj.join("archive/sub")).unwrap();
    fs::write(proj.join("archive/sub/f.txt"), "y").unwrap();

    let out = bin()
        .args([
            "--dir",
            proj.to_str().unwrap(),
            "config",
            "set",
            "backlog_directory",
            "archive",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));
    assert!(proj.join("backlog/sub/f.txt").exists());
    assert_eq!(
        fs::read_to_string(proj.join("archive/sub/f.txt")).unwrap(),
        "y"
    );
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn relocate_partial_failure_rolls_back_and_restores_dirs() {
    let d = tmpdir("partial");
    let proj = d.join("proj");
    fs::create_dir_all(&proj).unwrap();
    assert!(
        bin()
            .args(["--dir", proj.to_str().unwrap(), "init", "Proj", "--no-git"])
            .output()
            .unwrap()
            .status
            .success()
    );
    fs::write(proj.join("backlog/extra.txt"), "x").unwrap();
    fs::create_dir_all(proj.join("backlog/sub")).unwrap();
    fs::write(proj.join("backlog/sub/f.txt"), "y").unwrap();

    let before = fs::read_to_string(proj.join("papyrus.config.yml")).unwrap();

    let out = bin()
        .env("PAPYRUS_FORCE_PARTIAL_FAILURE", "1")
        .args([
            "--dir",
            proj.to_str().unwrap(),
            "config",
            "set",
            "backlog_directory",
            "archive",
        ])
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(4),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // both folders remain after rollback
    assert!(proj.join("backlog").exists(), "source folder restored");
    assert!(proj.join("archive").exists(), "destination folder restored");

    // source content restored
    assert!(proj.join("backlog/extra.txt").exists());
    assert!(proj.join("backlog/sub/f.txt").exists());

    // destination does not retain partially moved entries
    assert!(!proj.join("archive/extra.txt").exists());
    assert!(!proj.join("archive/sub/f.txt").exists());

    // config unchanged on failure
    let after = fs::read_to_string(proj.join("papyrus.config.yml")).unwrap();
    assert_eq!(before, after);
    assert!(after.contains("backlog_directory: backlog"));

    let _ = fs::remove_dir_all(&d);
}

#[test]
fn relocate_cross_device_fallback_reports_non_atomic() {
    let d = tmpdir("crossdevice");
    let proj = d.join("proj");
    fs::create_dir_all(&proj).unwrap();
    assert!(
        bin()
            .args(["--dir", proj.to_str().unwrap(), "init", "Proj", "--no-git"])
            .output()
            .unwrap()
            .status
            .success()
    );
    fs::write(proj.join("backlog/extra.txt"), "x").unwrap();
    fs::create_dir_all(proj.join("backlog/sub")).unwrap();
    fs::write(proj.join("backlog/sub/f.txt"), "y").unwrap();

    let out = bin()
        .env("PAPYRUS_FORCE_CROSS_DEVICE", "1")
        .args([
            "--dir",
            proj.to_str().unwrap(),
            "config",
            "set",
            "backlog_directory",
            "archive",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains("non-atomic"),
        "output did not mention non-atomic: {}",
        combined
    );

    assert!(proj.join("archive/extra.txt").exists());
    assert!(proj.join("archive/sub/f.txt").exists());
    assert!(
        !proj.join("backlog").exists()
            || fs::read_dir(proj.join("backlog")).unwrap().next().is_none()
    );

    // config persisted after success
    let cfg = fs::read_to_string(proj.join("papyrus.config.yml")).unwrap();
    assert!(cfg.contains("backlog_directory: archive"));

    let _ = fs::remove_dir_all(&d);
}
