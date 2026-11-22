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

fn build_package_forced(out: &Path) -> Output {
    Command::new(root().join("scripts/build-package.sh"))
        .args(["--version", "1.2.3", "--out"])
        .arg(out)
        .arg("--force")
        .output()
        .expect("run build-package.sh --force")
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
fn h2_home_subdirectory_is_refused_even_with_force() {
    let home = std::env::var("HOME").expect("HOME");
    let target = PathBuf::from(home).join(format!("papyrus-guard-home-{}", std::process::id()));
    let _ = fs::remove_dir_all(&target);
    fs::create_dir_all(&target).expect("create HOME subdirectory");
    let marker = target.join("keep.txt");
    fs::write(&marker, b"do not clean").expect("write marker");
    let output = build_package_forced(&target);
    assert_eq!(
        output.status.code(),
        Some(2),
        "a HOME subdirectory must be refused: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("HOME"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read(&marker).expect("marker retained"),
        b"do not clean",
        "--force must not clean a refused HOME subdirectory"
    );
    let _ = fs::remove_dir_all(&target);
}

#[test]
fn h2_force_refuses_an_unsafe_non_empty_path_before_cleaning() {
    let target = root()
        .join("target")
        .join(format!("package-guard-force-evil-{}", std::process::id()));
    let _ = fs::remove_dir_all(&target);
    fs::create_dir_all(&target).expect("create unsafe workspace");
    let marker = target.join("keep.txt");
    fs::write(&marker, b"do not clean").expect("write marker");
    let output = build_package_forced(&target);
    assert_eq!(
        output.status.code(),
        Some(2),
        "an unsafe path must be refused even with --force: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("project-local .tmp"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read(&marker).expect("marker retained"),
        b"do not clean",
        "--force must validate the path before cleaning"
    );
    let _ = fs::remove_dir_all(&target);
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

// AUD-148 F2: the asset-version guard runs on the normal assembly path, so
// each mismatch class is rejected before any assembly work starts.

fn asset_tree(name: &str) -> PathBuf {
    let tree = root()
        .join(".tmp")
        .join(format!("asset-guard-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tree);
    fs::create_dir_all(tree.join("scripts")).expect("create script directory");
    for script in ["build-deb.sh", "package-out.sh"] {
        fs::copy(
            root().join("scripts").join(script),
            tree.join("scripts").join(script),
        )
        .expect("copy packaging script");
    }
    tree
}

fn write_asset_version_tree(
    tree: &Path,
    version_dirs: &[&str],
    manifest_version: &str,
    copyright_version: &str,
) {
    let vendor = tree.join("src/browser/vendor");
    for version in version_dirs {
        fs::create_dir_all(vendor.join("mermaid").join(version)).expect("create vendor version");
    }
    fs::write(
        vendor.join("MANIFEST.md"),
        format!(
            "| path | package | version | licence |\n|---|---|---|---|\n| `/assets/mermaid/{manifest_version}/mermaid.min.js` | mermaid | {manifest_version} | MIT |\n"
        ),
    )
    .expect("write manifest");
    let copyright = tree.join("packaging/deb/usr/share/doc/papyrus");
    fs::create_dir_all(&copyright).expect("create copyright directory");
    fs::write(
        copyright.join("copyright"),
        format!("Files: src/browser/vendor/mermaid/{copyright_version}/mermaid.min.js\n"),
    )
    .expect("write copyright");
}

fn run_normal_deb_build(tree: &Path) -> (std::process::Output, PathBuf) {
    let out = tree.join("out");
    let output = Command::new(tree.join("scripts/build-deb.sh"))
        .args(["--version", "1.2.3", "--out"])
        .arg(&out)
        .output()
        .expect("run normal build-deb.sh");
    (output, out)
}

fn assert_guard_rejected(
    name: &str,
    version_dirs: &[&str],
    manifest: &str,
    copyright: &str,
    expected: &str,
) {
    let tree = asset_tree(name);
    write_asset_version_tree(&tree, version_dirs, manifest, copyright);
    let (output, out) = run_normal_deb_build(&tree);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        output.status.code(),
        Some(1),
        "{name}: a normal build must reject the mismatch, stderr: {stderr}"
    );
    assert!(
        stderr.contains("asset version check failed"),
        "{name}: {stderr}"
    );
    assert!(
        stderr.contains(expected),
        "{name}: expected {expected:?} in {stderr}"
    );
    assert!(
        !out.exists(),
        "{name}: the guard runs before any assembly output is created"
    );
    let _ = fs::remove_dir_all(&tree);
}

#[test]
fn f2_vendor_version_directory_mismatch_fails_a_normal_build() {
    let tree = asset_tree("vendor");
    write_asset_version_tree(&tree, &["11.17.2", "11.18.0"], "11.17.2", "11.17.2");
    let (output, out) = run_normal_deb_build(&tree);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(
        stderr.contains("more than one version directory"),
        "{stderr}"
    );
    assert!(!out.exists(), "the guard runs before assembly");
    let _ = fs::remove_dir_all(&tree);
}

#[test]
fn f2_manifest_version_mismatch_fails_a_normal_build() {
    assert_guard_rejected(
        "manifest",
        &["11.17.2"],
        "11.17.1",
        "11.17.2",
        "manifest version '11.17.1'",
    );
}

#[test]
fn f2_copyright_version_mismatch_fails_a_normal_build() {
    assert_guard_rejected(
        "copyright",
        &["11.17.2"],
        "11.17.2",
        "11.17.1",
        "copyright version '11.17.1'",
    );
}
