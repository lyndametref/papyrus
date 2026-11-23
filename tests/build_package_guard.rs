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
    Command::new("bash")
        .arg(root().join("scripts/build-package.sh"))
        .args(["--version", "1.2.3", "--out"])
        .arg(out)
        .output()
        .expect("run build-package.sh")
}

fn build_package_forced(out: &Path) -> Output {
    Command::new("bash")
        .arg(root().join("scripts/build-package.sh"))
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

/// Run the shared `package_validate_out` function directly, so a permitted
/// path can be checked without a release build.
fn validate_out(out: &Path) -> Output {
    let script = root().join("scripts/package-out.sh");
    Command::new("bash")
        .arg("-c")
        .arg("set -euo pipefail; source \"$1\"; ROOT=\"$2\"; package_validate_out \"$3\"")
        .arg("--")
        .arg(&script)
        .arg(root())
        .arg(out)
        .output()
        .expect("run package_validate_out")
}

fn prepare_out(out: &Path) -> Output {
    let script = root().join("scripts/package-out.sh");
    Command::new("bash")
        .arg("-c")
        .arg("set -euo pipefail; source \"$1\"; ROOT=\"$2\"; package_prepare_out \"$3\" false")
        .arg("--")
        .arg(&script)
        .arg(root())
        .arg(out)
        .output()
        .expect("run package_prepare_out")
}

fn assert_in_repo_refusal_message(output: &Output) {
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(".tmp"), "{stderr}");
    assert!(stderr.contains("builds"), "{stderr}");
}

#[test]
fn h2_out_inside_the_repository_is_refused_before_creation() {
    let target = root()
        .join("target")
        .join(format!("package-guard-evil-{}", std::process::id()));
    let _ = fs::remove_dir_all(&target);
    let output = assert_refused(&target, "repository target");
    assert_in_repo_refusal_message(&output);
    assert!(
        !target.exists(),
        "the refused out directory must not be created"
    );
}

// ADR-043 decision 7 as amended 2026-11-23 -------------------------------
#[test]
fn adr043_tmp_and_builds_roots_are_permitted_in_repository_targets() {
    let pid = std::process::id();
    let tmp = root()
        .join(".tmp")
        .join(format!("package-guard-permitted-tmp-{pid}"));
    let version = format!("0.0.0-guard-{pid}");
    let builds = root()
        .join("builds")
        .join(&version)
        .join(".build")
        .join("papyrus");

    for (out, label, leaf) in [
        (&tmp, "tmp", format!("package-guard-permitted-tmp-{pid}")),
        (&builds, "builds", "papyrus".to_string()),
    ] {
        let _ = fs::remove_dir_all(root().join("builds").join(&version));
        let output = validate_out(out);
        assert!(
            output.status.success(),
            "{label}: a permitted root must be accepted, stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let accepted = String::from_utf8_lossy(&output.stdout).trim().to_string();
        assert!(
            accepted.ends_with(&leaf),
            "{label}: expected the canonical path to end in {leaf}, got {accepted}"
        );
        assert!(
            !out.exists(),
            "{label}: validation must not create the directory"
        );

        let output = prepare_out(out);
        assert!(
            output.status.success(),
            "{label}: a permitted root must be prepared, stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(out.is_dir(), "{label}: the permitted root is created");
    }
    let _ = fs::remove_dir_all(&tmp);
    let _ = fs::remove_dir_all(root().join("builds").join(&version));
}

#[test]
fn adr043_repository_root_and_builds_itself_are_refused() {
    let root_output = build_package(&root());
    assert_eq!(root_output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&root_output.stderr);
    assert!(stderr.contains("repository root"), "{stderr}");
    assert!(root().exists(), "the repository root must not be removed");

    let builds = root().join("builds");
    let existed = builds.exists();
    let output = build_package(&builds);
    assert_eq!(
        output.status.code(),
        Some(2),
        "builds/ itself must be refused: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("builds"), "{stderr}");
    assert!(stderr.contains(".tmp"), "{stderr}");
    assert_eq!(
        builds.exists(),
        existed,
        "the guard must not create or remove builds/ itself"
    );
}

#[test]
fn adr043_canonicalised_in_repository_paths_are_refused() {
    let pid = std::process::id();
    let escaped = root()
        .join("builds")
        .join("..")
        .join(format!("package-guard-escape-{pid}"));
    // `realpath -m` folds `builds/..` back to the repository root, so the path
    // is neither permitted root and must be refused without creation.
    let output = build_package(&escaped);
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !root().join(format!("package-guard-escape-{pid}")).exists(),
        "a canonicalised escape must not be created"
    );

    let builds_dot = root().join("builds").join(".");
    let output = build_package(&builds_dot);
    assert_eq!(
        output.status.code(),
        Some(2),
        "builds/. canonicalises to builds/ itself and must be refused: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn adr043_usage_text_states_both_permitted_roots_and_each_refusal() {
    for script in ["package-out.sh", "build-package.sh", "build-deb.sh"] {
        let text = fs::read_to_string(root().join("scripts").join(script))
            .unwrap_or_else(|error| panic!("{script}: {error}"));
        for needle in [
            ".tmp/",
            "builds/<segment>",
            "<repo>",
            "repository root",
            "system directories",
            "HOME",
            "exit code 2",
        ] {
            assert!(
                text.contains(needle),
                "{script} usage text must state {needle:?}"
            );
        }
    }
}

#[test]
fn adr043_parent_of_the_repository_root_is_refused() {
    let parent = root().parent().expect("repository parent").to_path_buf();
    let output = build_package(&parent);
    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("parent of the repository root"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(parent.exists(), "a refused parent must not be removed");
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
    assert_in_repo_refusal_message(&output);
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
    let output = Command::new("bash")
        .arg(root().join("scripts/build-deb.sh"))
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
    let output = Command::new("bash")
        .arg(root().join("scripts/build-package.sh"))
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
    let output = Command::new("bash")
        .arg(tree.join("scripts/build-deb.sh"))
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
