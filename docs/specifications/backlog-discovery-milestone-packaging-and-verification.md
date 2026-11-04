# Feature Specification — Backlog Discovery Milestone Packaging and Verification

**Ticket:** TASK-12
**Milestone:** m-2 (Tasks, Milestones, Releases, and Discovery)
**Status:** Refined (execution-ready)
**UI mockups:** none new — no UI change in this ticket; the shipped browser interface keeps MOCK-0001 (approved) as its baseline and MOCK-0002 (approved) as the layout reference for backlog records

## Purpose

Package the current Papyrus feature set (TASK-1 foundation + TASK-2 init/config + TASK-3 req/nfreq + TASK-5 search + TASK-6 browser + TASK-8 milestones/releases + TASK-9 tasks + TASK-10 backlog search + TASK-11 backlog browser) as an installable amd64 Debian package, and verify in an isolated Debian Forky environment that the packaged binary installs, uninstalls, and provides task, milestone, release, search (over task/milestone/release records), and offline local browser access (including the backlog categories) end to end. This is the last task of milestone m-2 and closes its "each increment is packaged and usable" goal for backlog discovery.

This is a packaging and verification task: no new product behavior is added. Work is limited to building, packaging, verifying, documenting, and adding one milestone-level integration test.

## Scope

### In scope

- Rebuild `target/release/papyrus` and `papyrus_<version>_amd64.deb` from the current source with the pinned Rust 1.99.1 toolchain, using `scripts/build-deb.sh` unchanged.
- Verify package metadata: `Package: papyrus`, `Architecture: amd64`, `Version` equal to the Cargo package version, `Depends: libc6 (>= 2.34)`, GPL-3.0-or-later copyright.
- Verify package contents contain only the binary and the copyright file — in particular, no runtime asset files (`src/browser/app.css`, `app.js`, templates) are packaged, because they are compiled into the binary.
- Install and uninstall the package in an isolated Debian Forky environment without modifying the host.
- Smoke-test the installed binary:
  - CLI baseline: `--help`, `--version`, `init`, and the task/milestone/release lifecycles (create, list, view, edit, and where documented archive/restore).
  - Search over task, milestone, and release records: quoted phrase, `--type`, `--limit`, `--json`, deprecation/archival visibility.
  - Browser over the backlog categories: start with `--no-open` on a free port, all five categories (`req`, `nfr`, `task`, `milestone`, `release`), record view, edit round-trip, JSON API, embedded assets, 320 px markup, no external references.
- Verify offline operation: search and browser work with no internet access, using only loopback and locally served assets.
- Verify the browser configuration keys (`browser_host`, `browser_port`, `browser_open`) behave as documented in the installed binary.
- Re-run the Rust quality gates: `cargo build --release`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check` with Rust 1.99.1.
- Add one milestone-level integration test file under `tests/` covering task/milestone/release search and browser together against the built binary, without duplicating the existing `tests/browser_cli.rs` / `tests/search_backlog_cli.rs` contract coverage.
- Update `docs/packaging.md` with the verified supported release, install/uninstall steps, the extended smoke-test checklist, and observed results (pass/fail, environment, date/timezone).
- Record verification evidence in the TASK-12 ticket notes.

### Out of scope

- New product features, commands, options, record types, or UI changes (TASK-13 onward).
- Verification on Debian Trixie or any Debian release other than Forky (ADR-0003, NFR-03.002).
- Advertising support for any release beyond Forky.
- Real-GUI rendering checks in Firefox, Edge, or Safari; only markup- and asset-level verification is performed (see OQ-1).
- Re-running the runtime dependency-license audit (no new runtime dependencies; TASK-1/TASK-4 audit remains valid).
- Changes to requirements (`docs/requirements/`), business rules (`docs/business-rules/`), ADRs (`docs/architecture/adr/`), or packaging scripts (`scripts/build-deb.sh`, `packaging/deb/**`).
- New UI mockups; the shipped browser interface is unchanged and MOCK-0001/MOCK-0002 stay authoritative.
- Product code changes, except a minimal bug fix when a verification step exposes a defect; any such fix is reported in the ticket notes and kept separate from the packaging work.

## Behavior

### 1. Build and package

1. `cargo build --release` succeeds with the pinned Rust 1.99.1 toolchain (`rust-toolchain.toml`) and produces `target/release/papyrus`.
2. `cargo test`, `cargo clippy --all-targets -- -D warnings`, and `cargo fmt --check` pass before packaging.
3. The freshly built binary exposes the backlog subcommands: `target/release/papyrus --help` lists `task`, `milestone`, `release`, `search`, and `browser` (alongside `init`, `config`, `req`, `nfreq`).
4. `./scripts/build-deb.sh` produces `papyrus_<version>_amd64.deb` in the project root, where `<version>` is read from `Cargo.toml` (0.1.0 at refinement time). The script must be run after `cargo build --release`, because it reuses an existing `target/release/papyrus` instead of rebuilding it.
5. `dpkg-deb -I papyrus_<version>_amd64.deb` reports:
   - `Package: papyrus`
   - `Version: <Cargo version>`
   - `Architecture: amd64`
   - `Depends: libc6 (>= 2.34)`
   - `Section: utils`, `Priority: optional`, a `Maintainer` value, and a `Homepage` value
   - a `Description` naming Papyrus as a project-level backlog and documentation manager.
6. The packaged copyright file (`/usr/share/doc/papyrus/copyright`) is in Debian copyright format and declares `GPL-3.0-or-later`, matching `Cargo.toml`'s `license` field (NFR-06.001).
7. `dpkg-deb -c` lists exactly `/usr/bin/papyrus` (mode `0755`) and `/usr/share/doc/papyrus/copyright`, plus their directories. It lists no `.css`, `.js`, or `.html` file, no configuration file, and no man page.
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

#### 3.1 CLI baseline regression (task, milestone, release)

1. `papyrus --help` exits 0 and lists `task`, `milestone`, `release`, `search`, and `browser` among the subcommands.
2. `papyrus --version` exits 0 and prints `papyrus <version>`.
3. `papyrus --dir <project> init <name> --no-git` creates `papyrus.config.yml` and the configured directory structure.
4. Release lifecycle: `papyrus release create <title>` prints `REL-001` (or the next free ID); `release list`, `release view REL-001`, and `release edit REL-001 --title <new>` succeed with exit 0; the edited title persists in `release view`.
5. Milestone lifecycle: `papyrus milestone create <title> [--release REL-001]` prints `MS-001`; `milestone list`, `milestone view MS-001`, `milestone edit MS-001 --title <new>` (and `--release`/`--no-release`) succeed; IDs are preserved.
6. Task lifecycle: `papyrus task create <title> [--milestone MS-001] [--ac <criterion>]` prints `TASK-001`; `task list`, `task view TASK-001`, `task edit TASK-001 --status <status>` succeed; `task archive TASK-001` then `task restore TASK-001` preserve the ID; `task list` reflects archived/restored state.
7. Filtering and linking work from the installed binary: `task list --milestone MS-001`, `task list --release REL-001`, `milestone list` show the created records; `milestone view MS-001` shows its release assignment and `task view TASK-001` shows its milestone assignment.

#### 3.2 Search (task, milestone, release records)

1. A quoted phrase present contiguously in a task/milestone/release body is found: `papyrus search "<phrase>"` prints `<id> <type> <title>` lines in ascending ID order, including `task`, `milestone`, and `release` types.
2. The same phrase split into separate words is not treated as a phrase match (no result for a query that only matches across word boundaries).
3. `--type task`, `--type milestone`, `--type release` each restrict results to the corresponding record type; `--type doc` returns requirement records only.
4. `papyrus search <term> --limit 1` prints at most one result line; a limit larger than the match count returns all matches.
5. `papyrus --json search <term>` prints a JSON array of `{id, type, title, path}` objects; a search with no matches prints `[]` and exits 0.
6. Archived task/milestone records are absent from search results and reappear after `restore`; release records (no archived state) are always searched.
7. Usage errors (missing query, unknown `--type`, `--limit 0`) exit 2; an empty or control-character query exits 4; a symlinked record directory escaping the project exits 1.

#### 3.3 Browser (backlog categories)

1. `papyrus --dir <project> browser --no-open --port <free-port>` prints exactly one startup line `Serving http://127.0.0.1:<free-port>/` and serves until killed. `<free-port>` is obtained by binding `127.0.0.1:0`, reading the port, and dropping the listener, with a retry if the bind fails.
2. `--no-open` suppresses the platform opener: no `xdg-open` invocation occurs and the process still serves.
3. `GET /` and `GET /?type=req` render the business-requirements category; `GET /?type=nfr` renders the non-functional category; `GET /?type=task`, `?type=milestone`, `?type=release` each render exactly their own category and nothing else. The selected sidebar entry carries `aria-current="page"`; the sidebar's three entries are Business requirements, Non-functional requirements, Backlog.
4. `GET /doc/<id>` renders the record view for a task, milestone, and release, including ID, form, category, title, and their backlog-specific fields (status/assignee/milestone for tasks; release for milestones; description for all).
5. Edit round-trip: `GET /doc/<id>/edit` yields the form with its hidden `version` token; `POST /doc/<id>` with the documented form fields returns `303` with `Location: /doc/<id>`; the follow-up `GET` shows the new title, and `papyrus task|milestone|release view <id>` shows the same persisted file.
6. `GET /api/requirements?type=task|milestone|release` returns the documented JSON index for one category; `GET /api/doc/<id>` returns the documented record object.
7. `GET /assets/app.css` returns `200 text/css` and `GET /assets/app.js` returns `200 text/javascript`, both with `Cache-Control: no-store`; the bodies are non-empty.
8. 320 px markup: every page carries `<meta name="viewport" content="width=device-width, initial-scale=1">`, the document-type sidebar toggle is present with `aria-expanded` and `aria-controls`, and the served CSS/JS use no forbidden features from the TASK-6 browser baseline.
9. No external references: served HTML, CSS, and JS contain no `http://`, `https://`, protocol-relative `//` remote reference, remote `@import`, remote `url(...)`, `<img>`, web font, or CDN reference; HTML responses carry the documented Content-Security-Policy.
10. A missing or empty task/milestone/release directory renders its empty state instead of failing (no server error); archived tasks/milestones are not listed or reachable (404).

#### 3.4 Offline operation

1. The smoke run is repeated inside an isolated network namespace (`unshare -rn`, loopback brought up with `ip link set lo up`) so no external network is reachable.
2. Inside that namespace, an outbound TCP connection to a public address fails, proving the namespace is disconnected.
3. With no network available, `search` and the browser checks of sections 3.2 and 3.3 pass over loopback; the browser binds an IP literal (`127.0.0.1`) so no name resolution is required.
4. If `unshare -rn` is unavailable in the verification environment, the offline claim is supported instead by: loopback-only bind, no external references in served content, and no outbound-connection code path — and the limitation is recorded explicitly in `docs/packaging.md` and the ticket notes.

#### 3.5 Browser configuration keys

1. `config list` includes `browser_host`, `browser_port`, and `browser_open` after the `document_types.*` entries, with effective values.
2. `config get browser_host|browser_port|browser_open` prints the effective value.
3. `config set browser_port <free-port>` then starting `browser` without `--port` uses that port in the startup line; `--port` still overrides it.
4. `config set browser_open false` suppresses opening without `--no-open`; `config unset browser_open` restores the default `true`.
5. `config set browser_port 0`, `70000`, or `config set browser_open maybe` exits 4 and leaves the configuration file unchanged.
6. `config unset browser_port` restores the default `6420`.

#### 3.6 Embedded assets and no runtime file dependency

1. The installed binary serves `/assets/app.css` and `/assets/app.js` from a project directory that contains no asset files and from a working directory that contains no source tree, proving the assets are compiled in.
2. The package contains no asset, template, or documentation files beyond the copyright file (section 1.7).
3. The installed binary runs with only `/usr/bin/papyrus` and the project directory.

### 4. Automated integration test

1. A new file `tests/backlog_discovery_smoke.rs` provides a milestone-level end-to-end test against the built binary (`CARGO_BIN_EXE_papyrus`), using temporary project directories, a free loopback port, and the same startup-line wait and child-kill-on-drop pattern as `tests/browser_cli.rs`.
2. Test 1 — `milestone_backlog_discovery_flow`: initialize a project, create one release, one milestone assigned to it, and one task assigned to the milestone with a unique multi-word phrase; assert the task/milestone/release lifecycles (create/list/view/edit), quoted-phrase search across all three types, `--type task|milestone|release`, `--limit 1`, `--json` shape, archived-task/milestone visibility; then start `browser --no-open` on a free port and assert the startup line, all five category pages, a record view for each backlog type, the form edit round-trip (303 then updated view and matching CLI `view`), the JSON API, both assets, the 320 px markup hooks, and the absence of external references.
3. Test 2 — `installed_layout_has_no_runtime_asset_dependency`: run the binary with its working directory set to an empty directory and the project directory containing no asset files; assert `search` and `browser` still work and served assets are non-empty.
4. The new tests must not duplicate the TASK-6/TASK-11 contract coverage in `tests/browser_cli.rs` (request limits, error-status matrix, concurrency cap, XSS, symlink escape, version conflict, accessibility audit) or the TASK-10 coverage in `tests/search_backlog_cli.rs`. Those remain authoritative there.
5. Tests require no internet access, bind only loopback, and leave no child process running.
6. `cargo test` passes with the new tests. Baseline at refinement: the m-2 test suite (unit + `cli` + `milestone_cli` + `milestone_smoke` + `req_cli` + `task_cli` + `search_cli` + `search_backlog_cli` + `browser_cli` + `search_browser_smoke`) all passing.

### 5. Documentation and evidence

1. `docs/packaging.md` states the supported release as Debian Forky only (amd64), per ADR-0003 and NFR-03.002, and explicitly states that Trixie and other releases are not claimed or verified.
2. `docs/packaging.md` contains the build/package commands, the isolated install and uninstall procedure (scratch directories under `./.tmp/`, removed afterwards), and the extended smoke-test checklist covering sections 3.1–3.6.
3. `docs/packaging.md` records observed results as a pass/fail table with the environment (Debian Forky version, glibc, `rustc`/`cargo` 1.99.1, `dpkg` version, isolation method, whether `unshare -rn` was available) and the date with timezone (UTC).
4. The TASK-12 ticket notes record the same evidence: environment, date/timezone, package file name and size, quality-gate outcomes with test counts, smoke-test outcomes, and any deviation or bug fix found.
5. The runtime dependency-license audit from TASK-1/TASK-4 is referenced as unchanged; no re-run is required because no runtime dependency was added.

## Inputs and outputs

| Input / output | Description |
|---|---|
| Source tree | Post-TASK-11 state: `task`, `milestone`, `release`, `search` (incl. backlog), `browser` (incl. backlog), init/config, req/nfreq. |
| Toolchain | Rust 1.99.1 via `rust-toolchain.toml`; `dpkg-deb`, `fakeroot`; Debian Forky amd64. |
| `scripts/build-deb.sh` | Unchanged packaging script; reads the version from `Cargo.toml`. |
| `papyrus_<version>_amd64.deb` | Built package in the project root. |
| Isolated root/admin | `./.tmp/` directories for `dpkg --instdir/--admindir`; removed after the run. |
| `tests/backlog_discovery_smoke.rs` | New milestone-level integration test. |
| `docs/packaging.md` | Updated procedures, checklist, and observed results. |
| Ticket notes | Verification evidence for TASK-12. |

## Validation and error behavior

| Condition | Consequence |
|---|---|
| `cargo build --release`, `cargo test`, clippy, or `fmt --check` fails | Blocked; fix before packaging. |
| `target/release/papyrus --help` lacks `task`/`milestone`/`release`/`search`/`browser` (stale binary packaged) | Blocked; rebuild before `build-deb.sh`. |
| `dpkg-deb -I` metadata mismatch (name, version, architecture, depends, copyright) | Blocked. |
| Package contains asset/config/man files | Blocked. |
| `dpkg -i` fails in the isolated Forky root | Blocked. |
| Host package database or filesystem modified | Blocked; isolation is mandatory. |
| Installed binary missing from `PATH` after install, or still present after purge | Blocked. |
| Any smoke check of sections 3.1–3.6 fails | Blocked; record the failing command and output. |
| Outbound connection succeeds inside the offline namespace | Offline claim not verified; investigate before completion. |
| Served content references an external origin, or an asset request returns 404/empty | Blocked. |
| `cargo test` fails, including the new test | Blocked. |
| A defect in product code is found | Fix minimally, record it in the notes, keep the fix separate from packaging changes. |
| A failure is caused by a requirement/spec conflict (not by the package) | Record as an open question and escalate; do not change requirements in this ticket. |

## Acceptance criteria mapping

| Check | Verifies | How it is verified |
|---|---|---|
| T-01 | Pinned 1.99.1 release build succeeds and exposes task/milestone/release/search/browser | `cargo build --release`; `--help` subcommand list |
| T-02 | Quality gates pass | `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`; recorded test counts |
| T-03 | `.deb` builds with the Cargo version and amd64 architecture | `./scripts/build-deb.sh`; `dpkg-deb -I` |
| T-04 | Metadata: `Package: papyrus`, `Version`, `Architecture: amd64`, `Depends: libc6 (>= 2.34)`, GPL-3.0-or-later copyright | `dpkg-deb -I`; copyright file inspection |
| T-05 | Package contains only the binary and copyright; assets are embedded | `dpkg-deb -c`; asset requests from the installed binary |
| T-06 | Installs and uninstalls in an isolated Debian Forky environment, host untouched | `fakeroot dpkg --instdir/--admindir -i`, `-r`, `--purge`; host `dpkg -l` before/after |
| T-07 | Installed binary is on `PATH` at `/usr/bin/papyrus` and runs | `PATH`-prefixed `papyrus --version` |
| T-08 | Baseline CLI, `init`, and task/milestone/release lifecycles | Installed-binary smoke commands and exit codes |
| T-09 | Search over task/milestone/release records | Installed-binary search commands |
| T-10 | Browser: startup with `--no-open`, all five categories, view, edit round-trip | Installed-binary browser commands plus HTTP requests |
| T-11 | Browser: JSON API and embedded assets served from the binary | `GET /api/*`, `GET /assets/*` |
| T-12 | 320 px markup and no external references | Markup/asset assertions on served content |
| T-13 | Offline operation with no internet access | `unshare -rn` run with a failing outbound connection, then search + browser smoke |
| T-14 | Browser configuration keys work in the installed binary | `config list/get/set/unset` and startup-line assertions |
| T-15 | Milestone-level integration test added without duplicating browser/search contract tests | `tests/backlog_discovery_smoke.rs`; `cargo test` |
| T-16 | Documentation states Forky only and records procedures and results with date/timezone | `docs/packaging.md` review |
| T-17 | Evidence recorded in the ticket | TASK-12 notes review |

## Assumptions

- Assumption: the verification host is Debian Forky amd64. At refinement it reports `Debian GNU/Linux forky/sid`, glibc 2.43, `dpkg` 1.23.11, with `fakeroot`, `dpkg-deb`, `ip`, and `unshare -rn` available, and `rustc`/`cargo` 1.99.1 pinned by `rust-toolchain.toml`. The isolated `dpkg --instdir/--admindir` install therefore runs against Forky's glibc and dpkg, satisfying the NFR-03.002 verification method without a container.
- Assumption: isolation via `fakeroot dpkg --instdir/--admindir` is acceptable; a Forky container, chroot, or VM may substitute with the same checks (OQ-2).
- Assumption: scratch directories live under `./.tmp/` and are removed before the task ends (project convention forbids system temporary directories).
- Assumption: the package version stays `0.1.0`; no version bump is defined for m-2 and release versioning belongs to release orchestration (OQ-4).
- Assumption: `scripts/build-deb.sh` is used unchanged and does not force a rebuild; `cargo build --release` must run first. The repository may contain a stale `target/release/papyrus` and `papyrus_0.1.0_amd64.deb` built before TASK-8/TASK-9/TASK-10/TASK-11, so packaging without a rebuild would ship a binary lacking the backlog features. AC #2 guards this.
- Assumption: `task`, `milestone`, `release`, `search`, and `browser` require no new runtime dependency; the TASK-1/TASK-4 dependency-license audit remains valid (NFR-06.002).
- Assumption: no UI change ships in this ticket, so MOCK-0001 and MOCK-0002 remain the approved layout references and no new mockup is produced.
- Assumption: NFR-05.001 browser-version support was verified at markup/feature-subset level by TASK-6/TASK-11 and is not re-verified in a real GUI browser here (OQ-1).
- Assumption: the new integration test runs the built binary, not the installed package; the installed-package checks are the manual/scripted checklist in sections 2–3, recorded in `docs/packaging.md`.
- Assumption: the offline check is environment-dependent; `unshare -rn` is available at refinement, with the documented fallback if not.

## Open questions

None of these blocks implementation; each has a specified default. OQ-1 is recommended for user confirmation before release.

- **OQ-1 (recommended default: markup/asset-level verification only)** — NFR-05.001's verification method asks for checks in Firefox/Edge/Safari 100+. This headless packaging task verifies the served markup, asset set, viewport hooks, and absence of forbidden features, but does not drive a real GUI browser. Default: record the limitation in `docs/packaging.md` and rely on TASK-6/TASK-11 markup-level verification plus the approved MOCK-0001/MOCK-0002; schedule a manual browser pass outside this ticket if the user requires one. Non-blocking.
- **OQ-2 (recommended default: fakeroot `dpkg --instdir/--admindir` on the Forky build host)** — Whether a full Forky container/VM is mandatory. Default: use the fakeroot isolated install, which exercises Forky's glibc and dpkg and never touches the host; if the user requires container-level isolation, the same checklist applies unchanged. Non-blocking.
- **OQ-3 (recommended default: a new `tests/backlog_discovery_smoke.rs`)** — Whether the milestone test belongs in the existing `tests/milestone_smoke.rs` or a new file. Default: a new file keeps each milestone's smoke coverage separate and avoids growing an already consolidated TASK-4 file. Non-blocking.
- **OQ-4 (recommended default: keep version 0.1.0)** — No m-2 release version is defined in the requirements. Default: package the current Cargo version unchanged; a bump belongs to release orchestration. Non-blocking.

## Traceability

| Artifact | Reference |
|---|---|
| Requirement | REQ-02, REQ-02.001, REQ-02.002, REQ-02.003, REQ-02.004, REQ-03, REQ-03.001, REQ-03.002, REQ-06, REQ-06.001, REQ-06.002, REQ-05.001, REQ-05.002, REQ-05.007, REQ-01.003 |
| Non-functional | NFR-01.002, NFR-01.003, NFR-02.002, NFR-03.002, NFR-04.001, NFR-04.002, NFR-05.001, NFR-06.001, NFR-06.002 |
| Business rule | BR-004 (browser command behavior), BR-005 (task command behavior), BR-006 (milestone command behavior), BR-007 (release command behavior), BR-015 (search command behavior), BR-016 (configuration keys and precedence) |
| ADR | ADR-0001 (Rust), ADR-0003 (Debian Forky only; supersedes ADR-0002), ADR-0004 (repository-root single crate), ADR-0005 (embedded HTTP server and server-rendered UI), ADR-0006 (optimistic concurrency and atomic writes) |
| Ticket | TASK-12 |
| UI mockups | none new; MOCK-0001 and MOCK-0002 remain the approved layout references for the unchanged browser interface |
| Related specification | `docs/specifications/task-management.md` (TASK-9) |
| Related specification | `docs/specifications/milestone-and-release-management.md` (TASK-8) |
| Related specification | `docs/specifications/search-tasks-milestones-and-releases.md` (TASK-10) |
| Related specification | `docs/specifications/browser-tasks-milestones-and-releases.md` (TASK-11) |
| Related specification | `docs/specifications/search-requirements.md` (TASK-5) |
| Related specification | `docs/specifications/browser-requirements.md` (TASK-6) |
| Related specification | `docs/specifications/requirements-management-milestone-packaging-and-verification.md` (TASK-4 precedent) |
| Related specification | `docs/specifications/search-browser-milestone-packaging-and-verification.md` (TASK-7 precedent) |
| Related specification | `docs/specifications/rust-cli-and-debian-package-baseline.md` (TASK-1) |
| Related specification | `docs/specifications/project-initialization-and-configuration.md` (TASK-2) |
| Related specification | `docs/specifications/business-and-non-functional-requirements-management.md` (TASK-3) |
| Packaging documentation | `docs/packaging.md` |
