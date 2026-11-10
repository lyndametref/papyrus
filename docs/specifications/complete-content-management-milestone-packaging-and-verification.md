# Feature Specification — Complete Content-Management Milestone Packaging and Verification

**Ticket:** TASK-20
**Milestone:** m-12 (Final Cross-Type Integration)
**Status:** Refined (execution-ready)
**UI mockups:** none new — no UI change in this ticket; MOCK-0012 (approved 2026-11-10) remains the layout reference for the shipped browser interface, reusing the approved baseline MOCK-0001 and the approved patterns MOCK-0010 and MOCK-0011. No new mockup is produced.

## Purpose

Package the complete Papyrus feature set delivered by milestones m-0 through m-12 as one final installable amd64 Debian increment, and verify in an isolated Debian Forky environment that the packaged binary installs, uninstalls, and provides every supported record family, unified search across all documented record types, and the offline local browser over all Papyrus content, including the cross-type inventory, metadata projection, managed-artifact lifecycle and agent-skills pages, interactive init walkthrough, ID padding validation, lazy front-matter migration, and Backlog.md round-trip interoperability. This is the last task of milestone m-12 and closes its "ship the final installable increment" goal.

This is a packaging and verification task: no new product behavior is added. Work is limited to building, packaging, verifying, documenting, and adding one milestone-level integration test.

## Scope

### In scope

- Rebuild `target/release/papyrus` and `papyrus_<version>_amd64.deb` from the current source with the pinned Rust 1.99.1 toolchain, using `scripts/build-deb.sh` unchanged.
- Verify package metadata: `Package: papyrus`, `Architecture: amd64`, `Version` equal to the Cargo package version, `Depends: libc6 (>= 2.34)`, GPL-3.0-or-later copyright.
- Verify package contents contain only the binary and the copyright file — in particular, no runtime asset, template, configuration, or man files are packaged, because assets are compiled into the binary.
- Install and uninstall the package in an isolated Debian Forky environment without modifying the host.
- Smoke-test the installed binary across every supported record family: business requirements, non-functional requirements, tasks, milestones, releases, decisions, glossary terms, UI specifications, UI mockups, business rules, audit records, and technical specifications — create, list, view, edit (and deprecate/restore or archive/restore where documented), search, and browser view/edit.
- Smoke-test the cross-type capabilities delivered by m-12: unified search across all record types, the browser inventory hub, the final sidebar order, cross-type ID resolution, deprecated-record hiding, disabled-type gating, front-matter metadata projection, the managed artifacts and lifecycles page, the agent skills and workflows page, the interactive init walkthrough, ID padding validation (1–6), lazy front-matter migration on first read/write, and Backlog.md round-trip with no drift.
- Verify offline operation: search and browser work with no internet access, using only loopback and locally served assets.
- Verify the browser configuration keys (`browser_host`, `browser_port`, `browser_open`) behave as documented in the installed binary.
- Re-run the Rust quality gates: `cargo build --release`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check` with Rust 1.99.1.
- Add one milestone-level integration test file under `tests/` covering the cross-type flow against the built binary, without duplicating the existing per-type contract coverage.
- Update `docs/packaging.md` with the verified supported release, install/uninstall steps, the extended smoke-test checklist, and observed results (pass/fail, environment, date/timezone).
- Record verification evidence in the TASK-20 ticket notes.

### Out of scope

- New product features, commands, options, record types, or UI changes (any future work).
- Verification on Debian Trixie or any Debian release other than Forky (ADR-0003, NFR-03.002).
- Advertising support for any release beyond Forky.
- Real-GUI rendering checks in Firefox, Edge, or Safari; only markup- and asset-level verification is performed (see OQ-1).
- Re-running the runtime dependency-license audit (no new runtime dependencies; TASK-1/TASK-4 audit remains valid).
- Changes to requirements (`docs/requirements/`), business rules (`docs/business-rules/`), ADRs (`docs/architecture/adr/`), or packaging scripts (`scripts/build-deb.sh`, `packaging/deb/**`).
- New UI mockups; the shipped browser interface is unchanged and MOCK-0012 stays authoritative.
- Product code changes, except a minimal bug fix when a verification step exposes a defect; any such fix is reported in the ticket notes and kept separate from the packaging work.
- Re-verifying per-type contract coverage already authoritative in `tests/*_cli.rs` and `tests/*_smoke.rs`.

## Behavior

### 1. Build and package

1. `cargo build --release` succeeds with the pinned Rust 1.99.1 toolchain (`rust-toolchain.toml`) and produces `target/release/papyrus`.
2. `cargo test`, `cargo clippy --all-targets -- -D warnings`, and `cargo fmt --check` pass before packaging.
3. The freshly built binary exposes the full command set: `target/release/papyrus --help` lists at least `init`, `config`, `task`, `milestone`, `release`, `req`, `nfr`, `decision`, `glossary`, `ui-spec`, `ui-mockup`, `rule`, `audit`, `tech-spec`, `search`, and `browser`.
4. `./scripts/build-deb.sh` produces `papyrus_<version>_amd64.deb` in the project root, where `<version>` is read from `Cargo.toml` (0.1.0 at refinement time). The script must be run after `cargo build --release`, because it reuses an existing `target/release/papyrus` instead of rebuilding it.
5. `dpkg-deb -I papyrus_<version>_amd64.deb` reports:
   - `Package: papyrus`
   - `Version: <Cargo version>`
   - `Architecture: amd64`
   - `Depends: libc6 (>= 2.34)`
   - `Section: utils`, `Priority: optional`, a `Maintainer` value, and a `Homepage` value
   - a `Description` naming Papyrus as a project-level backlog and documentation manager.
6. The packaged copyright file (`/usr/share/doc/papyrus/copyright`) is in Debian copyright format and declares `GPL-3.0-or-later`, matching `Cargo.toml`'s `license` field (NFR-06.001).
7. `dpkg-deb -c` lists exactly `/usr/bin/papyrus` (mode `0755`) and `/usr/share/doc/papyrus/copyright`, plus their directories. It lists no `.css`, `.js`, `.html`, configuration, or man file.
8. `target/release/papyrus --version` and the installed binary's `--version` both print `papyrus <version>`, equal to the package version.

### 2. Isolated install and uninstall on Debian Forky

1. Verification runs on Debian Forky amd64 (at refinement the build host reports `forky/sid`, glibc 2.43, `dpkg` 1.23.11) using an isolated dpkg database and install root, so the host's package database and filesystem are never modified:
   `fakeroot dpkg --instdir=<root> --admindir=<adm> --force-depends -i papyrus_<version>_amd64.deb`.
   A container, chroot, or VM on Forky may be used instead; the same checks apply.
2. The isolated root and admin directories are created under `./.tmp/` in the workspace (project convention: no system temporary directories) and are removed after the run, including on failure.
3. Before and after the run, the host reports papyrus as not installed (`dpkg -l papyrus` has no installed entry), proving the host was untouched.
4. `dpkg -i` in the isolated root succeeds, exit 0.
5. After install, `<root>/usr/bin/papyrus` exists with mode `0755` and owner `root:root`.
6. With `<root>/usr/bin` first on `PATH`, `papyrus --version` resolves to the installed binary and prints `papyrus <version>`.
7. `dpkg -r papyrus` (isolated) removes the binary; `papyrus` no longer resolves on that `PATH`.
8. `dpkg --purge papyrus` (isolated) removes the remaining files and the admin entry; `<root>` contains no papyrus files afterwards.

### 3. Installed-binary smoke tests

All commands run against the installed binary with `--dir <project>` pointing at a scratch project under `./.tmp/`. Exit statuses follow the existing contract: 0 success, 1 IO/boundary, 2 usage, 3 not found, 4 validation.

#### 3.1 CLI baseline and init

1. `papyrus --help` exits 0 and lists every subcommand enumerated in section 1.3.
2. `papyrus --version` exits 0 and prints `papyrus <version>`.
3. `papyrus --dir <project> init <name> --no-git` creates `papyrus.config.yml` and the configured directory structure.
4. The interactive init walkthrough (TASK-75) runs to completion when invoked without `--no-git` in a prepared scratch directory: it activates the configured document types and writes the configuration, and the same project subsequently passes `papyrus config list`.
5. `papyrus --dir <project> config list` exposes `browser_host`, `browser_port`, and `browser_open` after the `document_types.*` entries, with effective values.

#### 3.2 Record-family lifecycle (every supported family)

For each family the installed binary exercises at least create → list → view → edit, with the documented ID form, and deprecate/restore or archive/restore where the family supports it. IDs are preserved across deprecate/restore and archive/restore.

| Family | Create | Lifecycle exercised | Notes |
|---|---|---|---|
| Business requirements | `req create-group` + `req create --group` | create/list/view/edit/deprecate/restore | `REQ-001`, `REQ-001.001` |
| Non-functional requirements | `nfr create-group` + `nfr create --group` | create/list/view/edit/deprecate/restore | `NFR-001`, `NFR-001.001` |
| Tasks | `task create` | create/list/view/edit/archive/restore | `TASK-001` |
| Milestones | `milestone create` | create/list/view/edit/archive/restore | `MS-001` |
| Releases | `release create` | create/list/view/edit | `REL-001` |
| Decisions | `decision create` | create/list/view/edit/deprecate/restore | `ADR-001` |
| Glossary terms | `glossary create` | create/list/view/edit/deprecate/restore | `GLO-001` |
| UI specifications | `ui-spec create` | create/list/view/edit/deprecate/restore | `UI-001` |
| UI mockups | `ui-mockup create` | create/list/view/edit/deprecate/restore | `MOCK-0001` |
| Business rules | `rule create` | create/list/view/edit/deprecate/restore | `RUL-001` |
| Audit records | `audit create` | create/list/view/edit/deprecate/restore | `AUD-001` |
| Technical specifications | `tech-spec create` | create/list/view/edit/deprecate/restore | `TECH-001` |

6. Edit changes only the supplied fields and preserves IDs; unknown IDs exit 3; validation failures exit 4; missing required arguments exit 2.
7. ID padding is constrained to 1–6: `config set id_padding 0` and `config set id_padding 7` exit 4 and leave the configuration file unchanged; valid values 1–6 are accepted and reflected in subsequent IDs.
8. The Backlog.md round-trip is verified in the installed binary: a task and milestone created by Papyrus remain readable after a Backlog.md-style edit of their front matter, and `papyrus` reports no drift (consistent with TASK-71); no legacy serializer is invoked.

#### 3.3 Unified search (all record types)

1. A quoted phrase present contiguously in a record body is found across every family: `papyrus search "<phrase>"` prints `<id> <type> <title>` lines in ascending ID order, with `doc` as the type for every document family and the family's own filter value otherwise.
2. The same phrase split into separate words is not treated as a phrase match.
3. `papyrus search <term> --type doc` returns all document-family records; `--type task`/`milestone`/`release`/`decision` return only their own store; an undocumented value such as `--type tech-spec` exits 2.
4. `papyrus search <term> --limit 1` prints at most one result line; a limit larger than the match count returns all matches.
5. `papyrus --json search <term>` prints a JSON array of `{id, type, title, path}` objects; a search with no matches prints `[]` and exits 0.
6. A deprecated document record is absent from search results and reappears after restore; an archived task or milestone is absent and reappears after restore; release records (no archived state) are all searched.
7. Disabled or absent document types contribute no results and no error; a missing store directory yields an empty success.
8. Usage errors (missing query, unknown `--type`, `--limit 0`) exit 2; an empty or control-character query exits 4; a symlinked store directory escaping the project exits 1.
9. `search` performs no writes: a filesystem snapshot before and after is identical.

#### 3.4 Browser — inventory, sidebar, and per-type views

1. `papyrus --dir <project> browser --no-open --port <free-port>` prints exactly one startup line `Serving http://127.0.0.1:<free-port>/` and serves until killed.
2. `--no-open` suppresses the platform opener: no `xdg-open` invocation occurs and the process still serves.
3. `GET /` renders the inventory hub with exactly one `h1` whose text is `Browser inventory`, one card per supported record type in the final sidebar order, and the active-record count for each type; disabled document types contribute no card and no error.
4. Each represented type's card links to its per-type list (`/?type=<category>`); following the link renders that category's list with `aria-current="page"` on the matching sidebar entry.
5. The sidebar on the inventory hub, on a per-type list, on the managed-artifacts page, and on the agent-skills page lists entries in the final order (Business requirements, Business rules, Audit records, Technical specifications, Non-functional requirements, Backlog, Decisions, Glossary, UI specifications, UI mockups, Managed artifacts & lifecycles, Agent skills & workflows), with the active entry marked `aria-current="page"` and the non-colour cue; gated entries are omitted when their type is disabled. On the inventory hub no document-type entry is active.
6. `GET /?type=req` (and every other documented `?type=` value) still renders its per-type list exactly as before this increment; `GET /` no longer renders the requirements list. `GET /?type=all` (and any other undocumented value) returns `400`.
7. `GET /doc/<id>` renders the record view for a record of every supported family, including ID, form, category, title, and body fields; a deprecated ID and an unknown ID return `404` with the per-type `404` page.
8. Cross-type ID resolution: an ID present in exactly one category resolves to that record; an ID present in two categories returns `500` with a generic message; a deprecated or unknown ID returns `404`. Deprecated records are never matched or counted.
9. Edit round-trip: `GET /doc/<id>/edit` yields the form with its hidden `version` token; `POST /doc/<id>` with the documented form fields returns `303` with `Location: /doc/<id>`; the follow-up `GET` shows the new title, and the CLI `view` shows the same persisted file.
10. `GET /api/<category>` (and the documented aliases) returns the documented JSON index for one category; `GET /api/doc/<id>` returns the documented record object.
11. `GET /assets/app.css` returns `200 text/css` and `GET /assets/app.js` returns `200 text/javascript`, both with `Cache-Control: no-store`; the bodies are non-empty.
12. 320 px markup: every page carries `<meta name="viewport" content="width=device-width, initial-scale=1">`, the document-type sidebar toggle is present with `aria-expanded` and `aria-controls`, and the served CSS/JS use no forbidden features from the TASK-6 browser baseline.
13. No external references: served HTML, CSS, and JS contain no `http://`, `https://`, protocol-relative remote reference, remote `@import`, remote `url(...)`, `<img>`, web font, or CDN reference; HTML responses carry the documented Content-Security-Policy.
14. A missing or empty store directory renders its empty state instead of failing (no server error).
15. `GET /lifecycle` (and `/?type=lifecycle`) renders the managed artifacts and lifecycles page; `GET /skills` (and `/?type=skills`) renders the agent skills and workflows page. Both render the shared shell and the sidebar in final order.
16. Front-matter metadata projection (REQ-06.004): list and detail views read `id`, `title`, `created_date`, `updated_date`, `labels`, `references`, and type-specific keys from the YAML front matter; `created_date` and `updated_date` are read-only in the browser; a save writes changed values to the front matter, refreshes `updated_date`, preserves every other front-matter key and the rest of the file, and never copies metadata into the body.

#### 3.5 Offline operation

1. The smoke run is repeated inside an isolated network namespace (`unshare -rn`, loopback brought up with `ip link set lo up`) so no external network is reachable.
2. Inside that namespace, an outbound TCP connection to a public address fails, proving the namespace is disconnected.
3. With no network available, search and the browser checks of sections 3.3 and 3.4 pass over loopback; the browser binds an IP literal (`127.0.0.1`) so no name resolution is required.
4. If `unshare -rn` is unavailable in the verification environment, the offline claim is supported instead by: loopback-only bind, no external references in served content, and no outbound-connection code path — and the limitation is recorded explicitly in `docs/packaging.md` and the ticket notes.

#### 3.6 Browser configuration keys

1. `config get browser_host|browser_port|browser_open` prints the effective value.
2. `config set browser_port <free-port>` then starting `browser` without `--port` uses that port in the startup line; `--port` still overrides it.
3. `config set browser_open false` suppresses opening without `--no-open`; `config unset browser_open` restores the default `true`.
4. `config set browser_port 0`, `70000`, or `config set browser_open maybe` exits 4 and leaves the configuration file unchanged.
5. `config unset browser_port` restores the default `6420`.

#### 3.7 Embedded assets and no runtime file dependency

1. The installed binary serves `/assets/app.css` and `/assets/app.js` from a project directory that contains no asset files and from a working directory that contains no source tree, proving the assets are compiled in.
2. The package contains no asset, template, or documentation files beyond the copyright file (section 1.7).
3. Removing or renaming nothing outside the package is required: the installed binary runs with only `/usr/bin/papyrus` and the project directory.

#### 3.8 Lazy migration and round-trip

1. A record written by an older serializer form is migrated to front matter on first read and remains searchable and viewable; a migration that cannot be applied safely leaves the file untouched and the record is still searched from its existing text.
2. After a Backlog.md-style edit of a task or milestone file, Papyrus reads and writes the record without drift and reports no conflict; the on-disk file remains valid Backlog.md content.

### 4. Automated integration test

1. A new file `tests/content_management_smoke.rs` provides a milestone-level end-to-end test against the built binary (`CARGO_BIN_EXE_papyrus`), using temporary project directories, a free loopback port, and the same startup-line wait and child-kill-on-drop pattern as `tests/browser_cli.rs`.
2. Test 1 — `milestone_content_management_flow`: initialize a project; create one record of every supported family carrying a shared unique multi-word phrase; assert the unified search (default, `--type doc`, `--type task`/`milestone`/`release`/`decision`, `--limit`, `--json`, quoted phrase, empty result); then start `browser --no-open` on a free port and assert the startup line, the inventory hub with one card per type and correct counts, the final sidebar order with `aria-current`, a record view for at least three families, an edit round-trip (303 then updated view and matching CLI `view`), the JSON API, both assets, the 320 px markup hooks, and the absence of external references.
3. Test 2 — `content_management_cross_type_resolution`: verify cross-type ID resolution (single match, multi-match → 500, deprecated → 404, unknown → 404) and deprecated hiding in counts.
4. Test 3 — `content_management_metadata_and_pages`: verify front-matter metadata projection on a list and detail view, the managed artifacts and lifecycles page, and the agent skills and workflows page render.
5. Test 4 — `installed_layout_has_no_runtime_asset_dependency`: run the binary with its working directory set to an empty directory and the project directory containing no asset files; assert `search` and `browser` still work and served assets are non-empty.
6. The new tests must not duplicate the per-type contract coverage in `tests/*_cli.rs` and `tests/*_smoke.rs` (request limits, error-status matrix, concurrency cap, XSS, symlink escape, version conflict, accessibility audit). Those remain authoritative there.
7. Tests require no internet access, bind only loopback, and leave no child process running.
8. `cargo test` passes with the new tests. Baseline at refinement: the full existing suite (including `tests/content_management_smoke.rs` once added) must remain green; record the resulting test count in the ticket notes.

### 5. Documentation and evidence

1. `docs/packaging.md` states the supported release as Debian Forky only (amd64), per ADR-0003 and NFR-03.002, and explicitly states that Trixie and other releases are not claimed or verified.
2. `docs/packaging.md` contains the build/package commands, the isolated install and uninstall procedure (scratch directories under `./.tmp/`, removed afterwards), and the extended smoke-test checklist covering sections 3.1–3.8.
3. `docs/packaging.md` records observed results as a pass/fail table with the environment (Debian Forky version, glibc, `rustc`/`cargo` 1.99.1, `dpkg` version, isolation method, whether `unshare -rn` was available) and the date with timezone (UTC).
4. The TASK-20 ticket notes record the same evidence: environment, date/timezone, package file name and size, quality-gate outcomes with test counts, smoke-test outcomes, and any deviation or bug fix found.
5. The runtime dependency-license audit from TASK-1/TASK-4 is referenced as unchanged; no re-run is required because no runtime dependency was added.

## Inputs and outputs

| Input / output | Description |
|---|---|
| Source tree | Post-m-12 state: all record families, unified search, complete browser access, metadata projection, managed artifacts and skills pages, init walkthrough, id padding, migration, round-trip. |
| Toolchain | Rust 1.99.1 via `rust-toolchain.toml`; `dpkg-deb`, `fakeroot`; Debian Forky amd64. |
| `scripts/build-deb.sh` | Unchanged packaging script; reads the version from `Cargo.toml`. |
| `papyrus_<version>_amd64.deb` | Built package in the project root. |
| Isolated root/admin | `./.tmp/` directories for `dpkg --instdir/--admindir`; removed after the run. |
| `tests/content_management_smoke.rs` | New milestone-level integration test. |
| `docs/packaging.md` | Updated procedures, checklist, and observed results. |
| Ticket notes | Verification evidence for TASK-20. |

## Validation and error behavior

| Condition | Consequence |
|---|---|
| `cargo build --release`, `cargo test`, clippy, or `fmt --check` fails | Blocked; fix before packaging. |
| `target/release/papyrus --help` lacks any subcommand enumerated in section 1.3 (stale binary packaged) | Blocked; rebuild before `build-deb.sh`. |
| `dpkg-deb -I` metadata mismatch (name, version, architecture, depends, copyright) | Blocked. |
| Package contains asset/config/man files | Blocked. |
| `dpkg -i` fails in the isolated Forky root | Blocked. |
| Host package database or filesystem modified | Blocked; isolation is mandatory. |
| Installed binary missing from `PATH` after install, or still present after purge | Blocked. |
| Any smoke check of sections 3.1–3.8 fails | Blocked; record the failing command and output. |
| Outbound connection succeeds inside the offline namespace | Offline claim not verified; investigate before completion. |
| Served content references an external origin, or an asset request returns 404/empty | Blocked. |
| `cargo test` fails, including the new test | Blocked. |
| A defect in product code is found | Fix minimally, record it in the notes, keep the fix separate from packaging changes. |
| A failure is caused by a requirement/spec conflict (not by the package) | Record as an open question and escalate; do not change requirements in this ticket. |

## Acceptance criteria mapping

| Check | Verifies | How it is verified |
|---|---|---|
| T-20-01 | Pinned 1.99.1 release build succeeds and exposes the full command set | `cargo build --release`; `--help` subcommand list |
| T-20-02 | Quality gates pass | `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`; recorded test counts |
| T-20-03 | `.deb` builds with the Cargo version and amd64 architecture | `./scripts/build-deb.sh`; `dpkg-deb -I` |
| T-20-04 | Metadata: `Package: papyrus`, `Version`, `Architecture: amd64`, `Depends: libc6 (>= 2.34)`, GPL-3.0-or-later copyright | `dpkg-deb -I`; copyright file inspection |
| T-20-05 | Package contains only the binary and copyright; assets are embedded | `dpkg-deb -c`; asset requests from the installed binary |
| T-20-06 | Installs and uninstalls in an isolated Debian Forky environment, host untouched | `fakeroot dpkg --instdir/--admindir -i`, `-r`, `--purge`; host `dpkg -l` before/after |
| T-20-07 | Installed binary is on `PATH` at `/usr/bin/papyrus` and runs | `PATH`-prefixed `papyrus --version` |
| T-20-08 | Baseline CLI, init, interactive walkthrough, and record-family lifecycle across all twelve families | Installed-binary smoke commands and exit codes |
| T-20-09 | ID padding constrained to 1–6 and Backlog.md round-trip without drift | Installed-binary config commands and round-trip check |
| T-20-10 | Unified search across all record types: quoted phrase, `--type`, `--limit`, `--json`, empty, deprecation/archival visibility, disabled types, no writes | Installed-binary search commands |
| T-20-11 | Browser: startup with `--no-open`, inventory hub, sidebar final order, `aria-current`, per-type lists, record views, edit round-trip | Installed-binary browser commands plus HTTP requests |
| T-20-12 | Browser: cross-type ID resolution (single, multi → 500, deprecated/unknown → 404), disabled types, empty states | Installed-binary browser commands plus HTTP requests |
| T-20-13 | Browser: JSON API and embedded assets served from the binary | `GET /api/*`, `GET /assets/*` |
| T-20-14 | 320 px markup and no external references | Markup/asset assertions on served content |
| T-20-15 | Front-matter metadata projection in CLI and browser | Installed-binary view/edit assertions |
| T-20-16 | Managed artifacts and lifecycles page and agent skills and workflows page render | Installed-binary browser assertions |
| T-20-17 | Lazy migration and Backlog.md round-trip in the installed binary | Installed-binary read/write assertions |
| T-20-18 | Offline operation with no internet access | `unshare -rn` run with a failing outbound connection, then search + browser smoke |
| T-20-19 | Browser configuration keys work in the installed binary | `config list/get/set/unset` and startup-line assertions |
| T-20-20 | Milestone-level integration test added without duplicating per-type contract tests | `tests/content_management_smoke.rs`; `cargo test` |
| T-20-21 | Documentation states Forky only and records procedures and results with date/timezone | `docs/packaging.md` review |
| T-20-22 | Evidence recorded in the ticket | TASK-20 notes review |

## Assumptions

- Assumption: the verification host is Debian Forky amd64. At refinement it reports `Debian GNU/Linux forky/sid`, glibc 2.43, `dpkg` 1.23.11, with `fakeroot`, `dpkg-deb`, `ip`, and `unshare -rn` available, and `rustc`/`cargo` 1.99.1 pinned by `rust-toolchain.toml`. The isolated `dpkg --instdir/--admindir` install therefore runs against Forky's glibc and dpkg, satisfying the NFR-03.002 verification method without a container.
- Assumption: isolation via `fakeroot dpkg --instdir/--admindir` is acceptable; a Forky container, chroot, or VM may substitute with the same checks (OQ-2).
- Assumption: scratch directories live under `./.tmp/` and are removed before the task ends (project convention forbids system temporary directories).
- Assumption: the package version stays `0.1.0`; no version bump is defined for m-12 and release versioning belongs to release orchestration (OQ-4). TASK-73 (Release 1.0) follows this ticket.
- Assumption: `scripts/build-deb.sh` is used unchanged and does not force a rebuild; `cargo build --release` must run first.
- Assumption: `search`, `browser`, and the new cross-type features require no new runtime dependency; the TASK-1/TASK-4 dependency-license audit remains valid (NFR-06.002).
- Assumption: no UI change ships in this ticket, so MOCK-0012 stays the approved layout reference and no new mockup is produced.
- Assumption: NFR-05.001 browser-version support was verified at markup/feature-subset level by the per-type increments and is not re-verified in a real GUI browser here (OQ-1).
- Assumption: the new integration test runs the built binary, not the installed package; the installed-package checks are the manual/scripted checklist in sections 2–3, recorded in `docs/packaging.md`.
- Assumption: the offline check is environment-dependent; `unshare -rn` is available at refinement, with the documented fallback if not.
- Assumption: per-type contract coverage remains authoritative in `tests/*_cli.rs` and `tests/*_smoke.rs`; this ticket adds only one milestone-level test that exercises the cross-type flow.
- Assumption: the full record-family lifecycle smoke is performed against the installed binary for all twelve families, but exhaustive per-command contract checks are not repeated here.

## Open questions

None of these blocks implementation; each has a specified default. OQ-1 is recommended for user confirmation before release.

- **OQ-1 (recommended default: markup/asset-level verification only)** — NFR-05.001's verification method asks for checks in Firefox/Edge/Safari 100+. This headless packaging task verifies the served markup, asset set, viewport hooks, and absence of forbidden features, but does not drive a real GUI browser. Default: record the limitation in `docs/packaging.md` and rely on the per-type increments' markup-level verification plus the approved MOCK-0012; schedule a manual browser pass outside this ticket if the user requires one. Non-blocking.
- **OQ-2 (recommended default: fakeroot `dpkg --instdir/--admindir` on the Forky build host)** — Whether a full Forky container/VM is mandatory. Default: use the fakeroot isolated install, which exercises Forky's glibc and dpkg and never touches the host; if the user requires container-level isolation, the same checklist applies unchanged. Non-blocking.
- **OQ-3 (recommended default: a new `tests/content_management_smoke.rs`)** — Whether the milestone test belongs in an existing smoke file or a new one. Default: a new file keeps each milestone's smoke coverage separate. Non-blocking.
- **OQ-4 (recommended default: keep version 0.1.0)** — No m-12 release version is defined in the requirements. Default: package the current Cargo version unchanged; a bump belongs to release orchestration (TASK-73). Non-blocking.
- **OQ-5 (recommended default: repeat the full record-family lifecycle smoke in this milestone package)** — End-to-end proof of the complete m-12 increment, even though earlier milestone packages verified subsets. Non-blocking.
- **OQ-6 (recommended default: verify lazy migration and round-trip at markup/CLI level only)** — Real Backlog.md interoperability edge cases are already covered by TASK-71; this ticket re-checks them in the installed binary without re-implementing. Non-blocking.

## Traceability

| Artifact | Reference |
|---|---|
| Requirement | REQ-01, REQ-01.003, REQ-01.006, REQ-01.007, REQ-02, REQ-02.001, REQ-02.003, REQ-03, REQ-03.001, REQ-03.002, REQ-04, REQ-04.001, REQ-04.002, REQ-05, REQ-05.001–REQ-05.011, REQ-06, REQ-06.001, REQ-06.002, REQ-06.004, REQ-06.005 |
| Non-functional | NFR-01.002, NFR-01.003, NFR-02.002, NFR-03.002, NFR-04.001, NFR-04.002, NFR-04.003, NFR-05.001, NFR-06.001, NFR-06.002 |
| Business rule | BR-004 (browser), BR-005 (task), BR-006 (milestone), BR-007 (release), BR-008 (decision), BR-009 (req), BR-010 (nfr), BR-011 (glossary), BR-012 (ui-spec), BR-013 (rule), BR-014 (audit), BR-015 (search), BR-016 (config keys), BR-017 (tech-spec) |
| ADR | ADR-0001 (Rust), ADR-0003 (Debian Forky only), ADR-0004 (repository-root single crate), ADR-0005 (embedded dependency-free HTTP server and server-rendered UI), ADR-0006 (optimistic concurrency and atomic writes), ADR-0007 (YAML front matter), ADR-0009 (shared record envelope) |
| Ticket | TASK-20 |
| UI mockups | none new; MOCK-0001 (approved baseline), MOCK-0010, MOCK-0011, MOCK-0012 (approved 2026-11-10) remain the layout references |
| Related specifications | `docs/specifications/search-complete-unified.md` (TASK-18), `docs/specifications/browser-complete-access.md` (TASK-19), `docs/specifications/technical-specifications-milestone-packaging-and-verification.md` (TASK-59), `docs/specifications/backlog-discovery-milestone-packaging-and-verification.md` (TASK-12), `docs/specifications/search-browser-milestone-packaging-and-verification.md` (TASK-7), `docs/specifications/cli-milestone-packaging-and-verification.md` (TASK-62), `docs/specifications/migrate-document-record-types-to-front-matter-lazily-on-first-read-and-first-write.md` (TASK-68), `docs/specifications/migrate-tasks-and-milestones-to-the-backlog-md-layout.md` (TASK-69), `docs/specifications/remove-legacy-record-serializers-and-enforce-the-no-drift-rule.md` (TASK-70), `docs/specifications/verify-backlog-md-round-trip-interoperability.md` (TASK-71), `docs/specifications/render-one-metadata-projection-in-cli-and-browser-views.md` (TASK-67), `docs/specifications/browser-managed-artifacts-and-lifecycles.md` (TASK-72), `docs/specifications/browser-agent-skills-and-workflows.md` (TASK-77), `docs/specifications/interactive-init-walkthrough-for-artifact-activation.md` (TASK-75), `docs/specifications/constrain-id-padding-to-1-6-in-configuration-validation.md` (TASK-74) |
| Packaging documentation | `docs/packaging.md` |
| Domain model | `docs/domain-model/business-glossary/` (record-type terms) |
