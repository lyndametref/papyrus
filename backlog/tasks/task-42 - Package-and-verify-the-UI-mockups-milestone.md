---
id: TASK-42
title: Package and verify the UI mockups milestone
status: Done
assignee: []
created_date: '2026-11-04T19:20:00Z'
updated_date: '2026-11-07T07:06:00Z'
labels:
  - packaging
milestone: m-7
dependencies:
  - TASK-38
  - TASK-39
  - TASK-40
references:
  - docs/specifications/ui-mockups-management.md
  - docs/specifications/search-ui-mockups.md
  - docs/specifications/browser-ui-mockups.md
  - docs/specifications/ui/browser-ui-mockups-ui.md
  - docs/specifications/cli-milestone-packaging-and-verification.md
  - docs/packaging.md
  - docs/requirements/business/REQ-05.007.md
  - docs/requirements/business/REQ-05.008.md
  - docs/requirements/business/REQ-05.009.md
  - docs/requirements/business/REQ-06.001.md
  - docs/requirements/business/REQ-06.002.md
  - docs/requirements/non-functional/NFR-01.002.md
  - docs/requirements/non-functional/NFR-01.003.md
  - docs/requirements/non-functional/NFR-03.002.md
  - docs/requirements/non-functional/NFR-04.001.md
  - docs/requirements/non-functional/NFR-04.002.md
  - docs/requirements/non-functional/NFR-05.001.md
  - docs/requirements/non-functional/NFR-06.001.md
  - docs/requirements/non-functional/NFR-06.002.md
  - docs/business-rules/BR-004-browser-command-behavior.md
  - docs/business-rules/BR-015-search-command-behavior.md
  - docs/business-rules/BR-016-configuration-keys-and-precedence.md
  - docs/architecture/adr/ADR-0001-implementation-language-rust.md
  - docs/architecture/adr/ADR-0003-supported-debian-release-target-forky-only.md
  - docs/architecture/adr/ADR-0004-repository-root-single-crate-layout.md
  - >-
    docs/architecture/adr/ADR-0005-embedded-dependency-free-http-server-and-server-rendered-ui.md
  - >-
    docs/architecture/adr/ADR-0006-optimistic-concurrency-and-atomic-record-writes.md
documentation:
  - docs/specifications/ui-mockups-management.md
  - docs/specifications/search-ui-mockups.md
  - docs/specifications/browser-ui-mockups.md
  - docs/specifications/ui/browser-ui-mockups-ui.md
  - docs/specifications/cli-milestone-packaging-and-verification.md
  - docs/packaging.md
priority: medium
type: chore
ordinal: 25400
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Package and verify the UI mockups milestone (m-7) in an installable increment: rebuild the amd64 Debian package from the post-TASK-41 source, verify its metadata and GPL-3.0-or-later licensing, install and uninstall it in an isolated Debian Forky environment, and run smoke tests proving the packaged binary can manage UI mockups, search UI mockups, and serve the offline local browser UI-mockup category end to end. Add one milestone-level integration test and update docs/packaging.md with the Forky-only supported release, procedures, checklist, and observed results.

Scope (in):
- Rebuild target/release/papyrus and papyrus_<version>_amd64.deb via scripts/build-deb.sh (unchanged); cargo build --release must run first (stale-binary guard via AC #2).
- Verify package metadata and contents (binary and copyright only; assets compiled into the binary).
- Isolated install/uninstall on Debian Forky via fakeroot dpkg --instdir/--admindir under ./.tmp; host package database and filesystem unchanged.
- Installed-binary smoke tests: UI mockup lifecycle (create/list/view/edit/deprecate/restore/supersede), search (quoted phrase, --type doc, --limit, --json, deprecated hidden), browser (?type=ui-mockup, view, edit round-trip, JSON API, assets, 320 px markup, no external references).
- Offline operation, browser configuration keys, embedded assets/no runtime file dependency.
- Add tests/ui_mockups_smoke.rs (milestone-level end-to-end test) and update docs/packaging.md; record verification evidence in ticket notes.

Scope (out):
- No new product behavior, commands, options, or UI changes; feature behavior delivered by TASK-38 (management), TASK-39 (search), TASK-40 (browser), TASK-41 (design review, MOCK-0006 approved).
- No verification on Debian releases other than Forky (ADR-0003, NFR-03.002); no real-GUI browser checks (markup/asset-level only); no dependency-license audit re-run; no changes to requirements, business rules, ADRs, packaging scripts, or UI; product code changes only as a minimal bug fix found during verification.

Milestone m-7 scope vs packaging: this ticket packages and verifies the m-7 UI-mockups increment only; it does not re-deliver management, search, browser, or design-review behavior.

Assumptions:
- Assumption: the verification host is Debian Forky amd64 (forky/sid, glibc 2.43, dpkg 1.23.11), rustc/cargo 1.99.1, fakeroot and unshare -rn available.
- Assumption: isolation via fakeroot dpkg --instdir/--admindir is acceptable (open question OQ-2).
- Assumption: scratch directories live under ./.tmp/ and are removed before the task ends.
- Assumption: the package version stays 0.1.0 (open question OQ-4).
- Assumption: scripts/build-deb.sh is used unchanged and does not force a rebuild; cargo build --release must run first.
- Assumption: no new runtime dependency was added; the TASK-1/TASK-4 runtime dependency license audit remains valid (NFR-06.002).
- Assumption: no UI change ships in this ticket; MOCK-0001 through MOCK-0006 approved baselines remain the layout references and no new mockup is produced.

Open questions (non-blocking, each with a default in the specification):
- OQ-1 (default: markup/asset-level verification only; no real-GUI browser run) - NFR-05.001 verification method.
- OQ-2 (default: fakeroot dpkg --instdir/--admindir on the Forky build host) - container/VM isolation.
- OQ-3 (default: new tests/ui_mockups_smoke.rs) - milestone test file location.
- OQ-4 (default: keep version 0.1.0).

Specification: docs/specifications/cli-milestone-packaging-and-verification.md
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The amd64 Debian package built from the current source installs on Debian Forky (amd64) - the only supported release per ADR-0003 and NFR-03.002 - and the installed binary provides UI mockup management, search, and browser (ui-mockup category); Debian Trixie and other releases are not claimed or verified.
- [x] #2 cargo build --release succeeds with the pinned Rust 1.99.1 toolchain and target/release/papyrus --help lists ui-mockup (guards against packaging a stale binary).
- [x] #3 cargo test, cargo clippy --all-targets -- -D warnings, and cargo fmt --check pass with Rust 1.99.1, and the test counts are recorded.
- [x] #4 scripts/build-deb.sh (unchanged) produces papyrus_<version>_amd64.deb with Version equal to the Cargo package version and Architecture: amd64.
- [x] #5 dpkg-deb metadata reports Package: papyrus, Architecture: amd64, Version equal to Cargo.toml, Depends: libc6 (>= 2.34), and a copyright file declaring GPL-3.0-or-later.
- [x] #6 The package contains only /usr/bin/papyrus (0755) and /usr/share/doc/papyrus/copyright, with no asset, template, or configuration files, and the installed binary serves /assets/app.css and /assets/app.js from assets compiled into the binary.
- [x] #7 In an isolated Debian Forky environment, dpkg -i succeeds, /usr/bin/papyrus exists as root:root 0755, papyrus resolves on PATH, and the host package database and filesystem are unchanged.
- [x] #8 In the same isolated environment, dpkg -r (or dpkg --purge) papyrus removes the binary and papyrus no longer resolves on PATH.
- [x] #9 The installed binary smoke test passes: papyrus --help exits 0 and lists ui-mockup, search, and browser, and papyrus --version exits 0 printing papyrus <version> matching the package version.
- [x] #10 The installed binary initializes a project and exercises the UI mockup lifecycle (create/list/view/edit/deprecate/restore/supersede) with expected exit codes and persisted files, preserving IDs.
- [x] #11 Installed-binary search smoke tests pass over UI mockups: a quoted phrase matches contiguously, --type doc includes UI mockups, --limit caps results, --json returns the {id,type,title,path} array (and [] when empty), and deprecated mockups are hidden until restored.
- [x] #12 Installed-binary browser smoke tests pass with --no-open on a free port: the startup line is printed, the ui-mockup category renders with the selected sidebar entry marked aria-current, a mockup view renders, an edit round-trip returns 303 and persists through both the UI and the CLI, the JSON API responds (/api/ui-mockups, /api/doc/<id>), both embedded assets are served, 320 px markup hooks are present, no served page/stylesheet/script references an external origin, and deprecated mockups are not listed or reachable (404).
- [x] #13 Offline operation is verified with no internet access (isolated network namespace, or the documented fallback evidence): search and browser view/edit checks pass using only loopback and locally served assets.
- [x] #14 Browser configuration keys work in the installed binary: config list/get/set/unset expose browser_host, browser_port, and browser_open, a configured port is used when --port is absent, --port overrides it, defaults are restored on unset, and invalid values exit 4 without changing the file.
- [x] #15 A milestone-level integration test in tests/ui_mockups_smoke.rs exercises the UI mockup lifecycle, search, and browser together end-to-end against the built binary, passes under cargo test, and does not duplicate the contract coverage in tests/ui_mockup_cli.rs, tests/search_cli.rs, or tests/browser_cli.rs.
- [x] #16 docs/packaging.md states Debian Forky only (amd64), explicitly disclaims Trixie and other releases, and records the isolated install/uninstall procedure, the extended smoke-test checklist, and observed results with environment and date including timezone.
- [x] #17 Verification evidence (environment, date/timezone, package file and size, quality-gate results with test counts, smoke-test outcomes, deviations) is recorded in the TASK-42 ticket notes.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 All acceptance criteria verified and recorded in the ticket notes and docs/packaging.md.
- [x] #2 cargo build --release, cargo test, cargo clippy --all-targets -- -D warnings, and cargo fmt --check pass with Rust 1.99.1.
- [x] #3 The .deb metadata and contents match the specification, and the package installs and uninstalls cleanly in an isolated Debian Forky environment with the host untouched.
- [x] #4 The installed-binary smoke checklist passes (UI mockup lifecycle, search, browser, offline operation, config keys, embedded assets).
- [x] #5 The new tests/ui_mockups_smoke.rs passes and the full existing test suite remains green.
- [x] #6 docs/packaging.md is updated with the Forky-only claim, procedures, checklist, observed results (date with timezone), and any recorded limitation.
- [x] #7 No requirements, business rules, ADRs, packaging scripts, or UI were changed; any product bug fix is recorded separately in the notes.
<!-- DOD:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
## Verification evidence (2026-11-07 UTC)

**Environment:** Debian GNU/Linux forky/sid amd64, glibc 2.43-7, dpkg 1.23.11, `rustc`/`cargo` 1.99.1, clippy 0.1.99, `fakeroot` 2.1.4, `unshare -rn` with `ip link set lo up`.

**Isolation:** `fakeroot dpkg --instdir=./.tmp/task42/inst --admindir=./.tmp/task42/adm --force-depends`. Host `dpkg` status was `install ok installed 0.1.0` before and after the run and was not modified. Scratch directories were removed after verification.

**Package:** `papyrus_0.1.0_amd64.deb`, 886836 bytes, SHA-256 `1294fd25ab5fe0081d6970d52670e97159ac17e368adff6776fc62bacf50c216`.

**Metadata:** `Package: papyrus`, `Version: 0.1.0` (Cargo.toml), `Architecture: amd64`, `Depends: libc6 (>= 2.34)`, `Section: utils`, `Priority: optional`, copyright `GPL-3.0-or-later`.

**Contents:** `/usr/bin/papyrus` (0755, 2771616 bytes) and `/usr/share/doc/papyrus/copyright` (0644, 865 bytes) only; no asset, template, configuration, or man files. Assets are embedded in the binary.

**Quality gates:** `cargo build` PASS; `cargo build --release` PASS; `target/release/papyrus --help` lists `ui-mockup`, `search`, `browser` PASS; `cargo test` PASS — 292 tests (54 unit + 238 integration, including `tests/ui_mockups_smoke.rs` 2); `cargo clippy --all-targets -- -D warnings` PASS — 0 warnings; `cargo fmt --check` PASS.

**Install/uninstall:** isolated `dpkg -i` exit 0; `/usr/bin/papyrus` root:root 0755; `papyrus` resolves on the isolated PATH; `dpkg -r` removes the binary and it no longer resolves on that PATH; `dpkg --purge` removes the binary and admin entry; host package database/filesystem unchanged.

**Installed-binary smoke:** 68 CLI/search/config checks PASS — `--help`/`--version`; `init`; UI mockup create/list/view/edit/deprecate/restore/supersede with persisted files and preserved IDs; exit codes 3/4 for unknown/blocked cases; search quoted phrase, split words, `--type doc`, `--limit`, `--json`, empty `[]`, deprecated hidden until restored; config list/get/set/unset for `browser_host`, `browser_port`, `browser_open`, invalid values exit 4 with unchanged file. 57 browser checks PASS — startup line, `?type=ui-mockup` with `aria-current`, view, edit round-trip `303` + `Location` and persistence through UI and CLI, `/api/ui-mockups`, `/api/doc/<id>`, `/assets/app.css`, `/assets/app.js`, 320 px hooks, no external references, deprecated `404`. 12 offline checks PASS inside `unshare -rn` — outbound TCP blocked, search/list/view/assets over loopback. Browser config checks PASS — configured port honored, `--port` overrides, defaults restored, `browser_open=false` serves without opening.

**New test:** `tests/ui_mockups_smoke.rs` (2 tests) exercises the lifecycle, search, and browser together against the built binary and does not duplicate the `ui_mockup_cli`/`search_cli`/`browser_cli` contract coverage.

**Deviations/gaps:** OQ-1 no real-GUI browser run (markup/asset-level only); Forky only, Trixie and other releases not claimed/verified; host already had `papyrus 0.1.0` installed from prior work (unchanged); `scripts/build-deb.sh` unchanged and the stale-binary guard was executed manually. No product bug fix was required; no requirements, business rules, ADRs, packaging scripts, or UI were changed.

Offline check also covered the browser edit round-trip (303 + persisted title) inside `unshare -rn`.
<!-- SECTION:NOTES:END -->

## Comments

<!-- COMMENTS:BEGIN -->
created: 2026-11-07T07:06:00Z

---

Validated 2026-11-07 (UTC): quality-gatekeeper Pass with findings (17/17 AC, 7/7 DoD); cargo test 292 passed; isolated Forky install/uninstall + installed-binary smoke verified; user-approved for Done.

---
<!-- COMMENTS:END -->
