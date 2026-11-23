---
id: TASK-288
title: "Produce the single release package and verify its contents"
created_date: '2026-11-20T17:44:23Z'
updated_date: '2026-11-23T06:32:49Z'
labels: []
references:
  - TECH-069
  - TECH-027
  - ADR-0027
  - ADR-038
  - NFR-007.003
  - ADR-043
  - TECH-008
  - MS-037
  - AUD-146
  - REL-001
status: ToDo
assignee:
  - rust-dev
milestone: MS-037
dependencies:
  - TASK-287
ordinal: 90
documentation:
  - scripts/build-deb.sh
  - packaging/deb
  - scripts/build-package.sh
  - scripts/package-out.sh
  - tests/build_package_guard.rs
  - src/browser/vendor/MANIFEST.md
  - packaging/deb/usr/share/doc/papyrus/copyright
type: packaging
---

## Description
## Context
Owner decision 2 allows one release only; earlier builds exist for verification and must not be shipped. The packaging entry point is `scripts/build-package.sh` (ADR-043), delegating Debian assembly to `scripts/build-deb.sh`. Current tree state at refinement:
- `scripts/build-package.sh` runs `cargo build --release --locked`, calls `scripts/build-deb.sh --version <v> --out <dir>`, and prints the `.deb` path.
- `scripts/build-deb.sh` verifies vendor asset versions (`src/browser/vendor/*/<version>`, `src/browser/vendor/MANIFEST.md`, `packaging/deb/usr/share/doc/papyrus/copyright`), stages only `/usr/bin/papyrus` and `/usr/share/doc/papyrus/copyright`, and supports `--version`, `--out`, and `--check-assets-only`.
- `scripts/package-out.sh` permits an in-repository `--out` strictly below `.tmp/` (scratch) or strictly below `builds/` with at least one segment after it (publish location, git-ignored, never committed), per ADR-043 decision 7 as amended 2026-11-23. TASK-283 implements and tests this rule; `--out builds/<version>/.build/<component>` is the intended publish path.
- A tracked `papyrus_0.1.0_amd64.deb` sits in the repository root (2,393,760 bytes) and a stale verification build `builds/0.2.0-rc.1/` (`.deb`, `.sha256`, `MANIFEST.txt`, built 2026-11-19 with `scripts/build-deb.sh`) exists; `builds/` is git-ignored.

## Scope
- Produce exactly one final release package with `scripts/build-package.sh --version <final> --out builds/<version>/.build/<component>`, the path permitted by the ADR-043 decision 7 rule recorded in TASK-283. Project-local `.tmp/` remains available for isolated install staging.
- Verify the contents: `.deb` metadata (`dpkg-deb -I`: Package, Version, Architecture amd64, Depends libc6 >= 2.34, Section, Priority, Maintainer, Homepage, Description); inventory (`dpkg-deb -c`: only `/usr/bin/papyrus` 0755 and `/usr/share/doc/papyrus/copyright` 0644 plus directories; no `.rs`, no per-type module, no `.css`, `.js`, or `.html`, no template, no configuration file, no man page); the asset-version check passing; and the packaged binary stripped, with no debug symbols, equal to `target/release/papyrus`.
- Verify installation in an isolated `fakeroot dpkg --instdir` and `--admindir` root under `./.tmp/`, `papyrus --version` equal to the package version, `papyrus config list`, and a create and edit round trip from the installed binary; remove the isolated root afterwards.
- Record in the task: version, commit SHA, script paths and exact commands, artifact path, size, sha256, and a manifest.
- Remove the tracked repository-root `papyrus_0.1.0_amd64.deb` from the tree and discard or explicitly mark the stale `builds/0.2.0-rc.1/` verification build, so exactly one release artifact remains.

## Out of scope
- No source, test, requirement, rule, ADR, specification, mockup, or documentation change except the packaging-artifact removal above.
- No tag, push, upload, or deploy.
- No man page in the package (TECH-008 section 1.7; ADR-0021 proposed). The stale task text that says to include the man pages is corrected by this refinement: the package ships no man page.

## Assumptions
- `<final>` is supplied by the release versioning step; the script accepts `<x.y.z>` and `<x.y.z-rc.N>`.
- The asset version is independent of the package version and is checked by `build-deb.sh` on every assembly path.
- Any RC under `builds/` is a discardable verification build, not a release artifact (MS-037 AC-6 and AC-7).

## Definition of done
The single final `.deb` is produced and retained, its metadata, inventory, and checksums are recorded, the installed-binary checks pass, the stale root `.deb` and old RC build are removed, and exactly one release artifact remains.

## Acceptance Criteria
- [ ] Exactly one final .deb is produced by the build-package script with the final version and retained; no second release artifact exists.
- [ ] The output path is `builds/<version>/.build/<component>`, strictly below the git-ignored `builds/` root and accepted by the packaging guard, and the exact command is recorded.
- [ ] dpkg-deb -I shows the supplied version, amd64, the libc dependency, section, priority, maintainer, homepage, and the Papyrus description.
- [ ] dpkg-deb -c lists only the packaged binary (0755) and the packaged copyright file (0644) plus directories; it lists no Rust source, per-type document module, CSS, JS, HTML, template, configuration file, or man page.
- [ ] The asset-version check passes on every assembly path, and no injected package version appears in an asset URL.
- [ ] The packaged binary is stripped, has no debug symbols, equals the release build, and prints papyrus with the final version.
- [ ] The installed binary in an isolated dpkg root under the project-local temp directory prints the package version, papyrus config list succeeds, and a create and edit round trip succeeds; the isolated root is removed afterwards.
- [ ] The artifact path, size, and sha256 are recorded, with the version, commit SHA, script paths, and exact commands.
- [ ] The tracked repository-root papyrus_0.1.0_amd64.deb is removed from the tree and the stale builds verification build is discarded or explicitly marked superseded.
- [ ] The working tree is clean of packaging changes after the run except the intended artifact-removal commit and the retained artifact under the git-ignored builds directory.

## Comments
- **task-refiner**: Reproduced at refinement: the build-package script with a verify version and an out directory under the repository builds path exits 2 with the project-local temp directory error, confirming the ADR-043 decision 7 conflict.
- **planner**: planner resolution: ADR-043 decision 7 as amended 2026-11-23 permits an in-repository --out strictly below the builds directory with a segment after it; TASK-283 implements and tests the guard. The output-path escalation and open question are cleared.
