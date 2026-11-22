//! AUD-145 H2 / L4: the packaging out-directory guard refuses unsafe targets
//! before creating or removing anything.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn workspace(name: &str) -> PathBuf {
    let path = root()
        .join(".tmp")
        .join(format!("package-guard-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("create guard workspace");
    path
}

fn build_package(out: &Path) -> Output {
    Command::new(root().join("scripts/build-package.sh"))
        .args(["--version", "1.2.3", "--out"])
        .arg(out)
        .output()
        .expect("run build-package.sh")
}

fn assert_refused(out: &Path, label: &str) -> Output {
    let output = build_package(out);
    assert_eq!(
        output.status.code(),
        Some(2),
        "{label}: expected refusal, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

#[test]
fn h2_out_inside_the_repository_is_refused_before_creation() {
    let target = root()
        .join("target")
        .join(format!("package-guard-evil-{}", std::process::id()));
    let _ = fs::remove_dir_all(&target);
    let output = assert_refused(&target, "repository target");
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("project-local .tmp"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !target.exists(),
        "the refused out directory must not be created"
    );
}

#[test]
fn h2_home_system_and_root_out_paths_are_refused() {
    let home = std::env::var("HOME").expect("HOME");
    for (path, label) in [
        (PathBuf::from(home), "HOME"),
        (PathBuf::from("/usr/local/papyrus-guard"), "system"),
        (PathBuf::from("/"), "root"),
    ] {
        assert_refused(&path, label);
    }
}

#[test]
fn h2_non_empty_out_requires_force_and_keeps_existing_files() {
    let workspace = workspace("non-empty");
    let marker = workspace.join("keep.txt");
    fs::write(&marker, b"do not delete").expect("write marker");
    let output = assert_refused(&workspace, "non-empty");
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("--force"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read(&marker).expect("marker kept"),
        b"do not delete",
        "a refused out directory keeps its contents"
    );
    let _ = fs::remove_dir_all(&workspace);
}

#[test]
fn h2_force_cleans_only_after_validation() {
    let workspace = workspace("force");
    let marker = workspace.join("old.txt");
    fs::write(&marker, b"old").expect("write marker");
    let script = root().join("scripts/package-out.sh");
    let output = Command::new("bash")
        .arg("-c")
        .arg("set -euo pipefail; source \"$1\"; ROOT=\"$2\"; package_prepare_out \"$3\" true")
        .arg("--")
        .arg(&script)
        .arg(root())
        .arg(&workspace)
        .output()
        .expect("run guard");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!marker.exists(), "--force removes stale contents");
    let _ = fs::remove_dir_all(&workspace);
}

#[test]
fn l4_deb_assembly_shares_the_repository_out_policy() {
    let output = Command::new(root().join("scripts/build-deb.sh"))
        .args(["--version", "1.2.3", "--out"])
        .arg(root())
        .output()
        .expect("run build-deb.sh");
    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("--out must not be the repository root"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn l4_version_regex_rejects_a_non_three_component_version() {
    let workspace = workspace("version");
    let out = workspace.join("out");
    let output = Command::new(root().join("scripts/build-package.sh"))
        .args(["--version", "1.2", "--out"])
        .arg(&out)
        .output()
        .expect("run build-package.sh");
    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("invalid version"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let _ = fs::remove_dir_all(&workspace);
}
