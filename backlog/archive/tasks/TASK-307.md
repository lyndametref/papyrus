---
id: TASK-307
title: "Create the Rust build-package script for the Papyrus binary"
created_date: '2026-11-22T12:31:25Z'
updated_date: '2026-11-22T13:01:28Z'
labels: []
references:
  - TECH-069
status: ToDo
assignee:
  - rust-dev
milestone: MS-034
dependencies:
  - TASK-254
ordinal: 85
documentation: []
type: implementation
---

## Description
Context: MS-034 and later milestones publish an RC package at milestone verification, but no scripts/build-package.sh exists; only scripts/build-deb.sh is present.

Scope: create one executable, committed scripts/build-package.sh at the repository root per the build-package-script contract. Interface: --version <version> --out <dir>, and --help. Behavior: resolve the repository root from the script path, detect the required toolchain with a clear failure, inject the version without editing tracked files, clean the destination component directory before writing, write only under --out plus the Cargo cache, print one produced file path per line on stdout with diagnostics on stderr, exit non-zero when the build fails or outputs are missing, and stay non-interactive. Prohibited: committing, tagging, pushing, uploading, writing outside --out and the tool cache, secrets or machine-specific absolute paths, downloading tools, reading the version from Git tags. Self-check with --version 0.0.0-test --out ./.tmp/script-check/papyrus/, then remove ./.tmp/script-check/.

Out of scope: building or publishing any package, changing scripts/build-deb.sh, and editing TECH-069 or milestone text.

## Acceptance Criteria
- [ ] The build-package script exists at the path fixed by the build-package-script contract and is mode 755, starting with the required bash shebang and set -euo pipefail.
- [ ] --help prints usage and exits 0; missing or unknown arguments produce a clear error on stderr and a non-zero exit.
- [ ] --version and --out are honored: the build runs from any working directory, writes only under the output directory plus the Cargo cache, and prints one produced file path per line on stdout.
- [ ] The self-check run with version 0.0.0-test produces a non-empty artifact; a rerun is idempotent, no tracked file is modified, and the temporary check directory is removed.
## Comments
- **planner**: Planner note: ADR-042 and the MS-034 internal-checkpoint ruling removed the RC-package requirement from TASK-261 (packaging waived per MS-032 AUD-138 and MS-033 AUD-140). This task's premise and MS-034 AC-8 still state the old RC gate; they are flagged as an open decision to the orchestrator. Do not treat this task as a verification packaging gate unless the orchestrator re-scopes it.
- **planner**: Archive reason: Premise invalid: build-deb.sh is the single build script per the TASK-235 architect ruling; MS-037 AC-6 corrected; no release gate requires a build-package script.
