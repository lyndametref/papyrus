---
id: TASK-7
title: Package and verify the search/browser requirements milestone
status: Refined
assignee:
  - '@orchestrator'
created_date: '2026-10-31T22:25:00Z'
updated_date: '2026-11-03T21:00:00Z'
labels:
  - packaging
  - verification
milestone: m-1
dependencies:
  - TASK-6
references:
  - docs/requirements/business/REQ-06.001.md
  - docs/requirements/business/REQ-06.002.md
  - docs/requirements/non-functional/NFR-01.003.md
  - docs/requirements/non-functional/NFR-03.002.md
  - docs/requirements/non-functional/NFR-05.001.md
  - docs/business-rules/BR-004-browser-command-behavior.md
  - docs/business-rules/BR-015-search-command-behavior.md
  - docs/specifications/search-browser-milestone-packaging-and-verification.md
  - docs/specifications/search-requirements.md
  - docs/specifications/browser-requirements.md
  - docs/architecture/adr/ADR-0003-supported-debian-release-target-forky-only.md
  - docs/packaging.md
ordinal: 7000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Package and verify the search/browser requirements milestone (m-1): rebuild the amd64 .deb from the post-TASK-6 source, verify its metadata, licensing, and contents, install and uninstall it in an isolated Debian Forky environment, and smoke-test the installed binary to prove that both `search` and the offline local browser work over requirement content. Add one milestone-level integration test and update docs/packaging.md with the Forky-only supported release, procedures, checklist, and observed results. No new product behavior is added.

Supported release: Debian Forky only (amd64), per ADR-0003 and NFR-03.002. Debian Trixie and any other release are not claimed or verified.

Scope (in):
- Rebuild target/release/papyrus and papyrus_<version>_amd64.deb via scripts/build-deb.sh (unchanged) with pinned Rust 1.99.1; cargo build --release must run first because the script reuses an existing binary.
- Verify package metadata (Package, Architecture: amd64, Version == Cargo.toml, Depends, GPL-3.0-or-later copyright) and contents (only /usr/bin/papyrus plus the copyright file; no asset files).
- Install and uninstall in an isolated Debian Forky environment using fakeroot dpkg --instdir/--admindir under ./.tmp; never modify the host.
- Installed-binary smoke tests: --help, --version, init, req/nfreq lifecycle, search (quoted phrase, --type doc, --limit, JSON, deprecation visibility), browser (--no-open on a free port, both categories, view, edit round-trip, JSON API, assets, 320 px markup, no external references), offline operation, and browser config keys.
- Re-run cargo build --release, cargo test, cargo clippy --all-targets -- -D warnings, cargo fmt --check with Rust 1.99.1.
- Add tests/search_browser_smoke.rs covering search and browser together end-to-end against the built binary, without duplicating the 41 tests in tests/browser_cli.rs.
- Update docs/packaging.md: Forky-only claim, isolated install/uninstall steps, extended smoke-test checklist, observed results (pass/fail, environment, date with timezone).
- Record verification evidence in the ticket notes.

Scope (out):
- No new product features, commands, options, record types, or UI changes (TASK-8 onward).
- No verification or support claim for Debian Trixie or any release other than Forky.
- No real-GUI browser rendering checks (markup/asset-level only; NFR-05.001 remains covered by TASK-6 and MOCK-0001).
- No re-run of the runtime dependency-license audit (no new runtime dependencies).
- No changes to requirements, business rules, ADRs, or packaging scripts; product code changes only as a minimal bug fix if verification exposes a defect, reported in the notes.
- No new UI mockups.

Assumptions:
- Assumption: the verification host is Debian Forky amd64 (at refinement: forky/sid, glibc 2.43, dpkg 1.23.11, rustc/cargo 1.99.1, fakeroot and unshare -rn available), so the isolated dpkg install exercises Forky glibc and dpkg.
- Assumption: fakeroot dpkg --instdir/--admindir is acceptable isolation; a Forky container/chroot/VM may substitute with the same checks.
- Assumption: scratch directories live under ./.tmp/ and are removed before the task ends.
- Assumption: the package version stays 0.1.0 (no m-1 version bump is defined; release versioning belongs to release orchestration).
- Assumption: scripts/build-deb.sh stays unchanged and does not force a rebuild; the repository currently holds a stale target/release/papyrus and papyrus_0.1.0_amd64.deb built before TASK-5/TASK-6, so packaging without a rebuild would ship a binary without search and browser.
- Assumption: the new integration test runs the built binary; installed-package checks are the recorded manual checklist.
- Assumption: the offline check uses unshare -rn with loopback up; if unavailable, the documented fallback evidence is recorded as a limitation.

Dependencies: TASK-6 (Done) and the whole m-1 chain TASK-1..TASK-6. No dependency on TASK-8+.

Open questions (non-blocking, each with a default in the specification):
- OQ-1: real Firefox/Edge/Safari rendering is not verified in this headless task; default is markup/asset-level verification plus the TASK-6 evidence.
- OQ-2: container-level isolation is optional; default is the fakeroot isolated dpkg install on the Forky build host.
- OQ-3: the milestone test lives in a new tests/search_browser_smoke.rs rather than extending tests/milestone_smoke.rs.
- OQ-4: keep version 0.1.0.

Specification: docs/specifications/search-browser-milestone-packaging-and-verification.md
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The amd64 Debian package built from the current source installs on Debian Forky (amd64) - the only supported release per ADR-0003 and NFR-03.002 - and the installed binary provides both search over requirement content and the local browser over requirement content; Debian Trixie and any other release are not claimed or verified.
- [ ] #2 cargo build --release succeeds with the pinned Rust 1.99.1 toolchain and target/release/papyrus --help lists both search and browser (guards against packaging a stale binary).
- [ ] #3 cargo test, cargo clippy --all-targets -- -D warnings, and cargo fmt --check pass with Rust 1.99.1, and the test counts are recorded.
- [ ] #4 scripts/build-deb.sh (unchanged) produces papyrus_<version>_amd64.deb with Version equal to the Cargo package version and Architecture: amd64.
- [ ] #5 dpkg-deb metadata reports Package: papyrus, Architecture: amd64, Version equal to Cargo.toml, Depends: libc6 (>= 2.34), and a copyright file declaring GPL-3.0-or-later.
- [ ] #6 The package contains only /usr/bin/papyrus (0755) and /usr/share/doc/papyrus/copyright, with no asset, template, or configuration files, and the installed binary serves /assets/app.css and /assets/app.js from assets compiled into the binary (no runtime asset file dependency).
- [ ] #7 In an isolated Debian Forky environment (fakeroot dpkg --instdir/--admindir under ./.tmp, or a Forky container/chroot/VM), dpkg -i succeeds, /usr/bin/papyrus exists as root:root 0755, papyrus resolves on PATH, and the host package database and filesystem are unchanged.
- [ ] #8 In the same isolated environment, dpkg -r (or dpkg --purge) papyrus removes the binary and papyrus no longer resolves on PATH.
- [ ] #9 The installed binary smoke test passes: papyrus --help exits 0 and lists search and browser, and papyrus --version exits 0 printing papyrus <version> matching the package version.
- [ ] #10 The installed binary initializes a project and exercises req and nfreq create-group, create, list, view, edit, deprecate, and restore with expected exit codes and persisted files, preserving IDs across deprecate/restore.
- [ ] #11 Installed-binary search smoke tests pass: a quoted phrase matches contiguously, --type doc restricts to requirement records, --limit caps results, --json returns the {id,type,title,path} array (and [] when empty), and a deprecated record is hidden until restored.
- [ ] #12 Installed-binary browser smoke tests pass with --no-open on a free port: the startup line is printed, both requirement categories render one at a time with the selected sidebar entry marked aria-current, a record view renders, an edit round-trip returns 303 and persists through both the UI and the CLI, the JSON API responds, both embedded assets are served, 320 px markup hooks are present, and no served page, stylesheet, or script references an external origin.
- [ ] #13 Offline operation is verified with no internet access (isolated network namespace, or the documented fallback evidence): search and the browser view/edit checks pass using only loopback and locally served assets.
- [ ] #14 Browser configuration keys work in the installed binary: config list/get/set/unset expose browser_host, browser_port, and browser_open, a configured port is used when --port is absent, --port overrides it, defaults are restored on unset, and invalid values exit 4 without changing the file.
- [ ] #15 A milestone-level integration test in tests/search_browser_smoke.rs exercises search and browser together end-to-end against the built binary, passes under cargo test, and does not duplicate the contract coverage in tests/browser_cli.rs.
- [ ] #16 docs/packaging.md states Debian Forky only (amd64), explicitly disclaims Trixie and other releases, and records the isolated install/uninstall procedure, the extended smoke-test checklist, and observed results with environment and date including timezone.
- [ ] #17 Verification evidence (environment, date/timezone, package file and size, quality-gate results with test counts, smoke-test outcomes, deviations) is recorded in the TASK-7 ticket notes.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 All acceptance criteria are verified and recorded in the ticket notes and docs/packaging.md.
- [ ] #2 cargo build --release, cargo test, cargo clippy --all-targets -- -D warnings, and cargo fmt --check pass with Rust 1.99.1.
- [ ] #3 The .deb metadata and contents match the specification, and the package installs and uninstalls cleanly in an isolated Debian Forky environment with the host untouched.
- [ ] #4 The installed-binary smoke checklist passes: baseline CLI, init, req/nfreq lifecycle, search, browser (categories, view, edit round-trip, API, assets, 320 px markup, no external references), offline operation, config keys, and embedded-asset independence.
- [ ] #5 The new tests/search_browser_smoke.rs passes and the full existing test suite remains green.
- [ ] #6 docs/packaging.md is updated with the Forky-only claim, procedures, checklist, observed results (date with timezone), and any recorded limitation.
- [ ] #7 No requirements, business rules, ADRs, packaging scripts, or UI were changed; any product bug fix is recorded separately in the notes.
<!-- DOD:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Preflight: confirm Debian Forky amd64, rustc/cargo 1.99.1 via rust-toolchain.toml, dpkg-deb, fakeroot, and unshare -rn availability; record versions and date (UTC) for the evidence log.
2. Quality gates: cargo build --release, cargo test (record counts), cargo clippy --all-targets -- -D warnings, cargo fmt --check. Confirm target/release/papyrus --help lists search and browser.
3. Package: run scripts/build-deb.sh unchanged; verify dpkg-deb -I (Package, Version == Cargo.toml, Architecture amd64, Depends libc6 (>= 2.34), copyright GPL-3.0-or-later) and dpkg-deb -c (only /usr/bin/papyrus 0755 plus copyright; no asset files). Record file name and size.
4. Isolated install: create ./.tmp/inst and ./.tmp/adm, initialize an empty dpkg database, install with fakeroot dpkg --instdir/--admindir -i; assert /usr/bin/papyrus root:root 0755, PATH resolution, and that host dpkg -l papyrus is unchanged. Verify dpkg -r and --purge remove the binary and admin entry, then remove ./.tmp scratch.
5. Installed-binary smoke: run sections 3.1-3.3 of the specification against the installed binary in a ./.tmp project (help/version, init, req/nfreq lifecycle, search quoted phrase/--type doc/--limit/--json/deprecation, browser --no-open on a free port, both categories, view, edit round-trip, JSON API, assets, 320 px markup, no external references).
6. Offline check: repeat the search and browser smoke inside unshare -rn with lo up, first proving an outbound TCP connection fails; record the fallback evidence if the namespace is unavailable.
7. Config keys: config list/get/set/unset for browser_host/browser_port/browser_open, configured port honored without --port, --port override, defaults restored, invalid values exit 4.
8. Embedded assets: run the installed binary from an empty working directory against a project with no asset files; assert search and browser still work and assets are non-empty.
9. Tests: add tests/search_browser_smoke.rs with the consolidated search+browser flow and the no-runtime-asset-dependency test, using CARGO_BIN_EXE_papyrus, temp project dirs, a free loopback port, startup-line wait, and child kill on drop; avoid duplicating tests/browser_cli.rs contract checks. Run cargo test and cargo fmt/clippy again.
10. Documentation: update docs/packaging.md with the Forky-only supported release and Trixie disclaimer, prerequisites, build/package commands, isolated install/uninstall using ./.tmp, the extended smoke-test checklist, and an observed-results table (environment, date/timezone, pass/fail, limitations).
11. Evidence and handoff: record environment, date/timezone, package name and size, gate results with test counts, smoke outcomes, and deviations in the ticket notes; report any bug fix separately.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
## Refinement baseline (recorded 2026-11-03, UTC - not the final verification evidence)

Environment observed on the verification host:
- OS: Debian GNU/Linux forky/sid (VERSION_CODENAME=forky), amd64.
- glibc: 2.43-7. dpkg: 1.23.11. fakeroot, dpkg-deb, ip present.
- Rust: rustc 1.99.1 (58cf8f9c3 2026-10-13), cargo 1.99.1, clippy 0.1.99; rust-toolchain.toml pins 1.99.1.
- unshare -rn works; unshare -n does not (no root). In a new netns, lo must be brought up with `ip link set lo up` before binding 127.0.0.1.

Stale-artifact finding (must not be packaged):
- target/release/papyrus (2026-11-02 20:23) and papyrus_0.1.0_amd64.deb (2026-11-02 20:26) predate TASK-5/TASK-6. `target/release/papyrus --help` lists only init/config/req/nfreq; `search` is an unrecognized subcommand.
- scripts/build-deb.sh rebuilds only when target/release/papyrus is missing, so `cargo build --release` must run before packaging (AC #2 guards this).

Current package baseline (stale): Package papyrus, Version 0.1.0, Architecture amd64, Depends libc6 (>= 2.34), GPL-3.0-or-later copyright; contents /usr/bin/papyrus (0755) + /usr/share/doc/papyrus/copyright only (500560 bytes).

Test baseline: cargo test PASS - 131 tests (33 unit + 41 browser_cli + 20 cli + 5 milestone_smoke + 11 req_cli + 21 search_cli).

Refinement decisions:
- Supported release corrected to Debian Forky only (amd64) in AC #1; Trixie is not claimed or verified (ADR-0003, NFR-03.002, which is already amended to Forky only).
- AC #3 (original) rewritten as AC #12: browser verification is markup/asset-level (real-GUI rendering is out of scope; NFR-05.001 stays covered by TASK-6 and MOCK-0001).
- New milestone test: tests/search_browser_smoke.rs (2 tests), not an extension of tests/milestone_smoke.rs.
- No UI mockup needed; MOCK-0001 unchanged.

## Final verification evidence (2026-11-03, UTC)

Environment: Debian GNU/Linux forky/sid amd64; glibc 2.43-7; dpkg 1.23.11; rustc 1.99.1 (58cf8f9c3 2026-10-13); cargo 1.99.1; clippy 0.1.99; fakeroot; `unshare -rn` with loopback brought up via `ip link set lo up`. Date: 2026-11-03 (UTC). Branch: `task/TASK-7`; implementation commit: `5f7508a`.

Artifact: `papyrus_0.1.0_amd64.deb`, 624048 bytes, SHA-256 `4b2c9e8799016a1f98168f4c373100b3efd2f03e93036e648e2e756951dac697`. The binary inside the package (SHA-256 `bcdeaa99ee1d4aa8125e718a44622d576e4320cd0f461eb605c00bc0c8c0d923`) is byte-identical to the current `target/release/papyrus`.

Freshness: `cargo build --release` was run immediately before `scripts/build-deb.sh` (script unchanged), and `target/release/papyrus --help` listed both `search` and `browser` before packaging, so the stale pre-TASK-5/TASK-6 binary was not shipped.

Package metadata: `Package: papyrus`, `Version: 0.1.0` (= Cargo.toml), `Architecture: amd64`, `Depends: libc6 (>= 2.34)`, `Section: utils`, `Priority: optional`, `Maintainer`, `Homepage`, description naming Papyrus as a project-level backlog and documentation manager; copyright file declares `GPL-3.0-or-later`.
Package contents: `/usr/bin/papyrus` (0755, 1688240 bytes) and `/usr/share/doc/papyrus/copyright` (0644, 865 bytes) only. No asset, template, configuration, or man files.

Quality gates (Rust 1.99.1):
- `cargo build --release`: PASS.
- `cargo test`: PASS - 133 tests (33 unit + 41 browser_cli + 20 cli + 5 milestone_smoke + 11 req_cli + 2 search_browser_smoke + 21 search_cli), 0 failed, 0 ignored.
- `cargo clippy --all-targets -- -D warnings`: PASS, 0 warnings.
- `cargo fmt --check`: PASS.

Isolated install/uninstall (Debian Forky, host untouched):
- `fakeroot dpkg --instdir=.tmp/inst --admindir=.tmp/adm --force-depends -i papyrus_0.1.0_amd64.deb`: PASS (exit 0). dpkg warned that `/var/log/dpkg.log` is not writable and that libc6 is absent from the empty database (host glibc 2.43 satisfies the dependency); no host file or package database entry was created.
- Installed file `root:root` mode 0755; `papyrus` resolved to `.tmp/inst/usr/bin/papyrus` on PATH; `papyrus --version` printed `papyrus 0.1.0`.
- Host `dpkg-query -W papyrus` before and after: "no packages found matching papyrus".
- `dpkg -r papyrus` (isolated): PASS, binary removed, `papyrus` no longer on PATH.
- Fresh install then `dpkg --purge papyrus` (isolated): PASS, binary and admin entry removed.
- Scratch roots removed afterwards; `.tmp/` cleaned.

Installed-binary smoke (all PASS):
- 3.1 `--help` (lists search/browser), `--version`, `init`, req/nfreq create-group/create/list/view/edit/deprecate/restore with IDs preserved.
- 3.2 Search: contiguous quoted phrase, split words not a phrase, `--type doc`, `--type task` empty, `--limit` cap/oversize, JSON `{id,type,title,path}` array and `[]`, deprecated hidden then restored; usage exits 2 and validation exit 4.
- 3.3 Browser: exact startup line, both categories one at a time with `aria-current="page"`, record view, edit round-trip (303 + `Location` + updated UI and CLI view), JSON API index/record, both embedded assets (`no-store`, non-empty), 320 px markup hooks and narrow-viewport CSS, no external references, CSP present.
- 3.4 Offline: `unshare -rn` with loopback; outbound TCP to 1.1.1.1:443 failed; search, view, and assets passed over loopback.
- 3.5 Config: `browser_host`/`browser_port`/`browser_open` listed and gettable; configured port honored without `--port`; `--port` overrides; defaults restored on unset; invalid values (0, 70000, maybe) exit 4 with the file hash unchanged.
- 3.6 Embedded assets: search and browser worked from an empty working directory and an asset-free project; assets non-empty.

AC coverage: #1 (Forky-only install, search + browser), #2 (release build + `--help`), #3 (gates/test counts), #4 (script, version, amd64), #5 (metadata + GPL-3.0-or-later), #6 (contents + embedded assets), #7 (isolated install, PATH, host untouched), #8 (isolated remove/purge), #9 (help/version), #10 (init + req/nfreq lifecycle), #11 (search), #12 (browser), #13 (offline), #14 (config keys), #15 (`tests/search_browser_smoke.rs`, 2 tests, no duplication of browser contract tests), #16 (`docs/packaging.md`), #17 (this evidence).

Deviations / notes:
- `scripts/build-deb.sh` intentionally left unchanged (AC #4 and DoD #7 require packaging scripts unchanged). The stale-binary hazard is mitigated by the documented mandatory `cargo build --release` plus the pre-package `--help` assertion; both were executed.
- No product bug was found and no product code was changed.
- OQ-1 limitation recorded in `docs/packaging.md`: verification is markup/asset-level; no GUI Firefox/Edge/Safari run. NFR-05.001 remains covered by TASK-6 and MOCK-0001.
- Version stays 0.1.0 (OQ-4); Debian Trixie and other releases are not claimed or verified (ADR-0003, NFR-03.002).
<!-- SECTION:NOTES:END -->
