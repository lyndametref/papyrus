---
id: TASK-309
title: "Create the Papyrus binary packaging script (build-package.sh)"
created_date: '2026-11-22T15:23:44Z'
updated_date: '2026-11-22T16:46:56Z'
labels: []
references:
  - TECH-069
  - TECH-024
  - TASK-288
  - MS-035
  - ADR-043
status: ToDo
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
