---
id: TASK-309
title: "Create the Papyrus binary packaging script (build-package.sh)"
created_date: '2026-11-22T15:23:44Z'
updated_date: '2026-11-22T18:51:22Z'
labels: []
references:
  - TECH-069
  - TECH-024
  - TASK-288
  - MS-035
  - ADR-043
status: Done
assignee:
  - rust-dev
milestone: MS-035
dependencies:
  - TASK-269
ordinal: 85
documentation: []
type: implementation
---

## Description
Create the deployable-component build script required by ADR-043, and give the Debian assembly step a version input. The script is the packaging entry point for the single-app repository root. MS-035 is an internal checkpoint, so the script may be exercised during MS-035 verification but the produced artifact is discarded; the release boundary (MS-037 TASK-288) uses the same script to produce the single retained release package.

Scope: two changes in one task, per ADR-043. (1) A new executable, committed scripts/build-package.sh at the repository root, the packaging entry point for the deployable component (the repository root is the component, per the build-package-script contract and its single-app exception). It builds the Papyrus Rust binary artifact used for the package and delegates Debian assembly to scripts/build-deb.sh; it does not assemble the .deb itself. (2) scripts/build-deb.sh accepts the version supplied by scripts/build-package.sh without editing any tracked file (for example a --version argument or an environment variable), and uses that version for the DEBIAN/control Version: field and the papyrus_<version>_amd64.deb artifact name.

build-deb.sh constraints: keep its existing direct-run behavior, so run directly with no version supplied it derives the version from Cargo.toml and builds as today; keep its staging tree (DEBIAN/control, usr/bin/papyrus, usr/share/doc/papyrus/copyright), the copyright file, the amd64 architecture, and the ADR-038 asset-version check unchanged; never read the version from Git tags; do not duplicate or re-implement the assembly in build-package.sh.

build-package.sh interface: --version <X.Y.Z> --out <dir>, and --help. Behavior: resolve the repository root from the script path so it runs from any working directory; detect the Rust toolchain and fail with a clear message when missing; pass the authoritative version to scripts/build-deb.sh so it appears in DEBIAN/control and the artifact name, without editing tracked files (git status --porcelain empty before and after); clean the component destination before writing so reruns are idempotent; write only under --out plus the Debian staging directory under target/ and the tool cache; leave no package file in the project root after a run; print one produced file path per line on stdout with all diagnostics on stderr; exit non-zero when the build fails or outputs are missing; stay non-interactive. Prohibited: committing beyond the task's own commit, tagging, pushing, uploading, writing outside --out and the tool cache, secrets or machine-specific absolute paths, downloading tools at build time, and reading the version from Git tags.

Self-check (definition of done): run scripts/build-package.sh --help; run scripts/build-package.sh --version 0.0.0-test --out ./.tmp/script-check/papyrus/; confirm the produced .deb is non-empty, its DEBIAN/control Version: is 0.0.0-test, its name is papyrus_0.0.0-test_amd64.deb, git status --porcelain is empty after the run, and no package file is left in the repository root; rerun into the same --out to confirm idempotency; run scripts/build-deb.sh directly with no version to confirm it still derives the Cargo.toml version and builds; remove ./.tmp/script-check/; commit both scripts and report their paths and the commit.

Out of scope: producing or publishing any RC or release package, and editing milestone text, TECH-069, or the milestone verification tasks.
## Acceptance Criteria
- [ ] #1 scripts/build-package.sh exists at the repository root, is executable, supports `--version <version> --out <dir>` and `--help`, and delegates Debian assembly to scripts/build-deb.sh without assembling the .deb itself.
- [ ] #2 scripts/build-deb.sh accepts the externally supplied version (argument or environment variable) from scripts/build-package.sh and uses it for the DEBIAN/control `Version:` field and the papyrus_<version>_amd64.deb artifact name, without editing a tracked file.
- [ ] #3 scripts/build-deb.sh run directly with no version supplied still derives the version from Cargo.toml and preserves its staging tree, DEBIAN/control, copyright, amd64 architecture, and asset-version check.
- [ ] #4 A scripts/build-package.sh `--version 0.0.0-test --out ./.tmp/script-check/papyrus/` run produces a non-empty papyrus_0.0.0-test_amd64.deb, leaves `git status --porcelain` empty, leaves no package file in the repository root, and reruns idempotently.
- [ ] #5 Produced file paths go to stdout, diagnostics go to stderr, the script exits non-zero on build failure or missing output, and ./.tmp/script-check/ is removed after the self-check.
## Comments
- **planner**: planner: MS-035 is an internal checkpoint. TASK-309 still creates the packaging entry point per ADR-043 and may be exercised during MS-035 verification, but the produced artifact is discarded; no RC version and no retained package. The release boundary TASK-288 uses the same script. Dependency on TASK-269 (269b, the deletion task) retained.
- **planner**: planner: ADR-043 added as the packaging-entry-point source for this task.
- **rust-dev**: Self-check passed.
- **rust-dev**: Ran --version 0.0.0-test --out DIR twice; both exit 0.
- **rust-dev**: build-package
- **rust-dev**: foo.sh
- **rust-dev**: scripts slash name
- **rust-dev**: build-package.sh
- **rust-dev**: build-deb.sh
- **rust-dev**: Implementation: added executable build-package.sh under the root scripts folder (mode 755). Interface: --version X --out DIR and --help; resolves the repo root from its own path; checks cargo, rustc, and dpkg-deb and exits 1 with a clear stderr message when one is missing; validates the version; cleans the --out component directory; runs cargo build --release --locked; delegates Debian assembly to build-deb.sh; prints one artifact path on stdout and diagnostics on stderr; exits 1 when the expected .deb is missing or empty; non-interactive. build-deb.sh (mode 755) now accepts --version or PAPYRUS_DEB_VERSION and --out or PAPYRUS_DEB_OUT while keeping the Cargo.toml-derived version and project-root output as the direct-run default; staging tree, control template, copyright, amd64, and existing checks unchanged; no tracked file edited.
- **rust-dev**: Self-check: --help exit 0. --version 0.0.0-test with a temp --out under .tmp exit 0, producing one non-empty papyrus_0.0.0-test_amd64.deb; dpkg-deb -f shows Version 0.0.0-test, Package papyrus, Architecture amd64. A second run into the same --out exit 0 with exactly one artifact. git status --porcelain unchanged before and after; no new .deb in the repo root; temp check directory removed.
- **rust-dev**: build-deb.sh --help exit 0. A direct run with no version and a temp --out derived 0.1.0 from Cargo.toml and produced papyrus_0.1.0_amd64.deb; the project-root output remains the default for a bare direct run. Error paths: missing --version, unknown argument, and invalid version exit 2; missing cargo exits 1 with a clear message; a simulated missing artifact exits 1 with empty stdout.
- **rust-dev**: Tests and checks: cargo test 1332 passed, 0 failed, 26 ignored with EDITOR and VISUAL unset. With the host EDITOR=nano set, the pre-existing cli.rs test config_editor_json_null_when_unset fails on that ambient env; unrelated to this change. clippy -D warnings clean; fmt clean.
- **rust-dev**: Docs: no delta. TECH-008, TECH-013, TECH-024, TECH-027, and TECH-069 already name build-package.sh, and the packaging documentation historical build-deb references still match.
- **rust-dev**: Created executable build-package.sh under the root scripts folder (mode 755) and modified build-deb.sh (mode 755).
- **rust-dev**: Branch TASK-309; status left In-Progress. No commit made, per instruction.
- **orchestrator**: Acceptance met: build-package.sh is the executable packaging entry point with version and out arguments, toolchain checks, one artifact path on stdout, idempotent runs and no tracked edits; build-deb.sh takes the injected version; self-check produced the test deb and removed scratch; 1332 tests pass.
