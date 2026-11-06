# Feature Specification — CLI Milestone Packaging and Verification

**Ticket:** TASK-62
**Milestone:** m-4 (CLI)
**Status:** Refined (execution-ready)
**UI mockups:** none new — no UI change in this ticket; the shipped browser interface keeps MOCK-0001 (approved), MOCK-0002 (approved), and MOCK-0003 (pending approval, TASK-15) as its baseline. No new mockup is produced.

## Purpose

Package the current Papyrus feature set (TASK-1 foundation + TASK-2 init/config + TASK-3 req/nfreq + TASK-5 search + TASK-6 browser + TASK-8 milestones/releases + TASK-9 tasks + TASK-10 backlog search + TASK-11 backlog browser + TASK-12 packaging + TASK-13 decision management + TASK-14 decision search + TASK-15 decision browser + TASK-16 packaging + TASK-30 glossary management + TASK-34 UI specifications + TASK-38 UI mockups + TASK-43 business rules + TASK-47 audit records + TASK-55 technical specifications + TASK-61 README fix) as an installable amd64 Debian package, and verify in an isolated Debian Forky environment that the packaged binary installs, uninstalls, and provides the six new record-management command families end to end.

This is the last task of milestone m-4 and closes its "each increment is packaged and usable" goal for the CLI record types.

This is a packaging and verification task: no new product behavior is added. Work is limited to building, packaging, verifying, documenting, and adding one milestone-level integration test.

## Scope

### In scope

- Rebuild `target/release/papyrus` and `papyrus_<version>_amd64.deb` from the current source with the pinned Rust 1.99.1 toolchain, using `scripts/build-deb.sh` unchanged.
- Verify package metadata: `Package: papyrus`, `Architecture: amd64`, `Version` equal to the Cargo package version, `Depends: libc6 (>= 2.34)`, GPL-3.0-or-later copyright.
- Verify package contents contain only the binary and the copyright file — in particular, no runtime asset files (`src/browser/app.css`, `app.js`, templates) are packaged, because they are compiled into the binary.
- Install and uninstall the package in an isolated Debian Forky environment without modifying the host.
- Smoke-test the installed binary:
  - CLI baseline: `--help`, `--version`, `init`.
  - Management lifecycles for the six m-4 record types (glossary, UI specification, UI mockup, business rule, audit record, technical specification): create/list/view/edit/deprecate/restore where documented, with expected exit codes and persisted files, preserving IDs.
  - Regression of already-shipped search over requirement/task/milestone/release/decision records: quoted phrase, `--type`, `--limit`, `--json`, archival visibility.
  - Regression of already-shipped browser over the req/nfr/task/milestone/release/decision categories: startup with `--no-open` on a free port, record view, edit round-trip, JSON API, embedded assets, 320 px markup, no external references.
- Verify offline operation: search and browser work with no internet access, using only loopback and locally served assets.
- Verify the browser configuration keys (`browser_host`, `browser_port`, `browser_open`) behave as documented in the installed binary.
- Re-run the Rust quality gates: `cargo build --release`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check` with Rust 1.99.1.
- Add one milestone-level integration test file under `tests/` covering the six management lifecycles together against the built binary, without duplicating the existing contract coverage in `tests/glossary_cli.rs`, `tests/ui_spec_cli.rs`, `tests/ui_mockup_cli.rs`, `tests/business_rules_cli.rs` (rule), `tests/audit_records_cli.rs`, `tests/technical_specifications_cli.rs`.
- Update `docs/packaging.md` with the verified supported release, install/uninstall steps, the extended smoke-test checklist, and observed results (pass/fail, environment, date/timezone).
- Record verification evidence in the TASK-62 ticket notes.

### Out of scope

- New product features, commands, options, record types, or UI changes.
- Search integration for the six new record types — those are delivered by TASK-31/TASK-35/TASK-39/TASK-44/TASK-48/TASK-56 (still To Do at refinement time) and verified in their own milestone packaging tasks (TASK-33/TASK-37/TASK-42/TASK-46/TASK-50/TASK-59). TASK-62 does NOT claim or verify search over glossary/UI-spec/UI-mockup/business-rule/audit/tech-spec records.
- Browser listing, viewing, and editing of the six new record types — those are delivered by TASK-23/TASK-24/TASK-25/TASK-26/TASK-40/TASK-57 and verified in their own milestone packaging tasks. TASK-62 does NOT claim or verify browser categories for the new types.
- Verification on Debian Trixie or any Debian release other than Forky (ADR-0003, NFR-03.002).
- Advertising support for any release beyond Forky.
- Real-GUI rendering checks in Firefox, Edge, or Safari; only markup- and asset-level verification is performed (see OQ-1).
- Re-running the runtime dependency-license audit (no new runtime dependencies; TASK-1/TASK-4 audit remains valid).
- Changes to requirements (`docs/requirements/`), business rules (`docs/business-rules/`), ADRs (`docs/architecture/adr/`), or packaging scripts (`scripts/build-deb.sh`, `packaging/deb/**`).
- New UI mockups; the shipped browser interface is unchanged.
- Product code changes, except a minimal bug fix when a verification step exposes a defect; any such fix is reported in the ticket notes and kept separate from the packaging work.

## Behavior

### 1. Build and package

1. `cargo build --release` succeeds with the pinned Rust 1.99.1 toolchain (`rust-toolchain.toml`) and produces `target/release/papyrus`.
2. `cargo test`, `cargo clippy --all-targets -- -D warnings`, and `cargo fmt --check` pass before packaging.
3. The freshly built binary exposes the m-4 subcommands: `target/release/papyrus --help` lists `glossary`, `ui-spec`, `ui-mockup`, `rule`, `audit`, `tech-spec` (alongside `init`, `config`, `req`, `nfreq`, `task`, `milestone`, `release`, `decision`, `search`, and `browser`).
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

1. Verification runs on Debian Forky amd64 using an isolated dpkg database and install root, so the host's package database and filesystem are never modified:
   `fakeroot dpkg --instdir=<root> --admindir=<adm> --force-depends -i papyrus_<version>_amd64.deb`.
2. The isolated root and admin directories are created under `./.tmp/` in the workspace (project convention: no system temporary directories) and are removed after the run, including on failure.
3. Before and after the run, the host reports papyrus as not installed (`dpkg -l papyrus` has no installed entry), proving the host was untouched.
4. `dpkg -i` in the isolated root succeeds, exit 0.
5. After install, `<root>/usr/bin/papyrus` exists with mode `0755` and owner `root:root`.
6. With `<root>/usr/bin` first on `PATH`, `papyrus --version` resolves to the installed binary and prints `papyrus <version>`.
7. `dpkg -r papyrus` (isolated) removes the binary; `papyrus` no longer resolves on that `PATH`.
8. `dpkg --purge papyrus` (isolated) removes the remaining files and the admin entry; `<root>` contains no papyrus files afterwards.

### 3. Installed-binary smoke tests

All commands run against the installed binary with `--dir <project>` pointing at a scratch project under `./.tmp/`. Exit statuses follow the existing contract: 0 success, 1 IO/boundary, 2 usage, 3 not found, 4 validation.

#### 3.1 CLI baseline regression

1. `papyrus --help` exits 0 and lists `glossary`, `ui-spec`, `ui-mockup`, `rule`, `audit`, `tech-spec`, `decision`, `task`, `milestone`, `release`, `search`, and `browser` among the subcommands.
2. `papyrus --version` exits 0 and prints `papyrus <version>`.
3. `papyrus --dir <project> init <name> --no-git` creates `papyrus.config.yml` and the configured directory structure.

#### 3.2 Management lifecycles for the six m-4 record types

For each of glossary, ui-spec, ui-mockup, rule, audit, tech-spec, run the documented lifecycle against the installed binary:

- Create prints the generated ID (for example `GLO-001`, `UI-001`, `MOCK-0001`, `RUL-001`, `AUD-001`, `TECH-001`) and the ID is preserved across list/view/edit/deprecate/restore.
- `list` shows active records only, in ID order; deprecated records are hidden.
- `view <id>` prints the full persisted text; viewing an unknown ID exits 3; viewing a deprecated record by ID is allowed where documented.
- `edit <id>` with supplied fields updates only those fields; omitted fields retain previous values; unknown ID exits 3; unsanitizable values exit 4.
- `deprecate <id>` moves the file into `deprecated/`; deprecating an already-deprecated or unknown ID exits 4/3.
- `restore <id>` returns the file to the active folder with the same ID (where the command is documented; note ui-mockup restoration updates the status field in the HTML).
- Disabled types (`document_types.<type>.enabled = false`) are rejected by `create` and hidden from `list` (BR-016).

Command shapes (from BR-011..BR-017 and the management specs):

- glossary: `create --term <term> --definition <text>`; `edit <id> [--term] [--definition] [--editor]`; `deprecate`/`restore`.
- ui-spec: `create <title> --view <view> --description <text>`; `edit <id> [--view] [--description] [--editor]`; `deprecate`/`restore`.
- ui-mockup: `create <title> --task <task> --spec <id>... [--ref <doc>...]`; `edit <id> [--title] [--task] [--spec...] [--ref...] [--status] [--superseded-by] [--editor]`; `deprecate`/`restore`; supersession rules per BR-012/REQ-05.008.
- rule: `create <title> --description <text> --rationale <text> --applicability <text> [--requirement <id>...]`; `edit <id> [--title] [-d] [--requirement...] [--no-requirement...] [--editor]`; `deprecate`/`restore`.
- audit: `create --title <title> --content <text> [--audit-type <type>]` (default type `security`); `edit <id> [--title] [--content] [--audit-type] [--editor]`; `deprecate`/`restore`.
- tech-spec: `create <title> --description <text>`; `edit <id> [--title] [--description] [--editor]`; `deprecate`/`restore`.

#### 3.3 Search regression (already-shipped types)

1. A quoted phrase present contiguously in a decision record is found: `papyrus search "<phrase>"` prints `<id> <type> <title>` lines in ascending ID order, including `decision` type alongside `task`, `milestone`, `release`, and `doc`.
2. The same phrase split into separate words is not treated as a phrase match.
3. `--type decision|task|milestone|release|doc` restricts results to that type.
4. `papyrus search <term> --limit 1` prints at most one result line.
5. `papyrus --json search <term>` prints a JSON array of `{id, type, title, path}` objects; a search with no matches prints `[]` and exits 0.
6. Archived decision records are absent from search results; release and requirement records follow their own documented visibility rules.
7. Usage errors (missing query, unknown `--type`, `--limit 0`) exit 2; an empty or control-character query exits 4; a symlinked record directory escaping the project exits 1.

Search over the six new record types is out of scope (deferred to their milestones) and is not asserted here.

#### 3.4 Browser regression (already-shipped categories)

1. `papyrus --dir <project> browser --no-open --port <free-port>` prints exactly one startup line `Serving http://127.0.0.1:<free-port>/` and serves until killed. `<free-port>` is obtained by binding `127.0.0.1:0`, reading the port, and dropping the listener, with a retry if the bind fails.
2. `--no-open` suppresses the platform opener.
3. `GET /` and `GET /?type=req` render business requirements; `GET /?type=nfr` renders non-functional; `GET /?type=task`, `?type=milestone`, `?type=release`, `?type=decision` each render exactly their own category. The selected sidebar entry carries `aria-current="page"`; the sidebar entries are Business requirements, Non-functional requirements, Backlog, Decisions.
4. `GET /doc/<id>` renders the record view for a task, milestone, release, and decision, including ID, form, category, title, and their type-specific fields.
5. Edit round-trip (task/milestone/release/decision): `GET /doc/<id>/edit` yields the form with its hidden `version` token; `POST /doc/<id>` with the documented form fields returns `303` with `Location: /doc/<id>`; the follow-up `GET` shows the new title, and the matching CLI `view` shows the same persisted file.
6. `GET /api/requirements?type=task|milestone|release|decision` returns the documented JSON index for one category; `GET /api/decisions` returns the documented summary array; `GET /api/doc/<id>` returns the documented record object.
7. `GET /assets/app.css` returns `200 text/css` and `GET /assets/app.js` returns `200 text/javascript`, both with `Cache-Control: no-store`; the bodies are non-empty.
8. 320 px markup: every page carries `<meta name="viewport" content="width=device-width, initial-scale=1">`, the document-type sidebar toggle is present with `aria-expanded` and `aria-controls`, and the served CSS/JS use no forbidden features from the TASK-6 browser baseline.
9. No external references: served HTML, CSS, and JS contain no `http://`, `https://`, protocol-relative `//` remote reference, remote `@import`, remote `url(...)`, `<img>`, web font, or CDN reference; HTML responses carry the documented Content-Security-Policy.
10. A missing or empty task/milestone/release/decision directory renders its empty state instead of failing (no server error); archived decisions and deprecated requirements are not listed or reachable (404).

Browser categories for the six new record types are out of scope (deferred to their milestones) and are not asserted here.

#### 3.5 Offline operation

1. The smoke run is repeated inside an isolated network namespace (`unshare -rn`, loopback brought up with `ip link set lo up`) so no external network is reachable.
2. Inside that namespace, an outbound TCP connection to a public address fails, proving the namespace is disconnected.
3. With no network available, search and the browser checks of sections 3.3 and 3.4 pass over loopback; the browser binds an IP literal (`127.0.0.1`) so no name resolution is required.
4. If `unshare -rn` is unavailable in the verification environment, the offline claim is supported instead by: loopback-only bind, no external references in served content, and no outbound-connection code path — and the limitation is recorded explicitly in `docs/packaging.md` and the ticket notes.

#### 3.6 Browser configuration keys

1. `config list` includes `browser_host`, `browser_port`, and `browser_open` after the `document_types.*` entries, with effective values.
2. `config get browser_host|browser_port|browser_open` prints the effective value.
3. `config set browser_port <free-port>` then starting `browser` without `--port` uses that port in the startup line; `--port` still overrides it.
4. `config set browser_open false` suppresses opening without `--no-open`; `config unset browser_open` restores the default `true`.
5. `config set browser_port 0`, `70000`, or `config set browser_open maybe` exits 4 and leaves the configuration file unchanged.
6. `config unset browser_port` restores the default `6420`.

#### 3.7 Embedded assets and no runtime file dependency

1. The installed binary serves `/assets/app.css` and `/assets/app.js` from a project directory that contains no asset files and from a working directory that contains no source tree, proving the assets are compiled in.
2. The package contains no asset, template, or documentation files beyond the copyright file (section 1.7).
3. The installed binary runs with only `/usr/bin/papyrus` and the project directory.

### 4. Automated integration test

1. A new file `tests/cli_milestone_smoke.rs` provides a milestone-level end-to-end test against the built binary (`CARGO_BIN_EXE_papyrus`), using temporary project directories, a free loopback port, and the same startup-line wait and child-kill-on-drop pattern as `tests/browser_cli.rs`.
2. Test 1 — `milestone_cli_records_flow`: initialize a project, create one record of each of the six m-4 types carrying a unique multi-word phrase where the record supports a body field, assert the management lifecycle (create/list/view/edit/deprecate/restore where documented) and ID preservation; then run the offline browser checks of section 3.4 only as regression over the already-shipped categories (no new categories are asserted).
3. Test 2 — `installed_layout_has_no_runtime_asset_dependency`: run the binary with its working directory set to an empty directory and the project directory containing no asset files; assert `search` (over a shipped type) and `browser` still work and served assets are non-empty.
4. The new tests must not duplicate the TASK-30/TASK-34/TASK-38/TASK-43/TASK-47/TASK-55 contract coverage in their respective `*_cli.rs` files (ID generation, sanitization, exit-code matrix, editor round-trip, supersession rules, disabled-type behavior). Those remain authoritative there.
5. Tests require no internet access, bind only loopback, and leave no child process running.
6. `cargo test` passes with the new tests. Baseline at refinement: the m-4 test suite (unit + `cli` + `milestone_cli` + `milestone_smoke` + `req_cli` + `task_cli` + `decision_cli` + `glossary_cli` + `ui_spec_cli` + `ui_mockup_cli` + `business_rules_cli` + `audit_records_cli` + `technical_specifications_cli` + `search_cli` + `search_backlog_cli` + `browser_cli` + `backlog_discovery_smoke` + `decision_records_smoke` + `search_browser_smoke`) all passing.

### 5. Documentation and evidence

1. `docs/packaging.md` states the supported release as Debian Forky only (amd64), per ADR-0003 and NFR-03.002, and explicitly states that Trixie and other releases are not claimed or verified.
2. `docs/packaging.md` contains the build/package commands, the isolated install and uninstall procedure (scratch directories under `./.tmp/`, removed afterwards), and the extended smoke-test checklist covering sections 3.1–3.7.
3. `docs/packaging.md` records observed results as a pass/fail table with the environment (Debian Forky version, glibc, `rustc`/`cargo` 1.99.1, `dpkg` version, isolation method, whether `unshare -rn` was available) and the date with timezone (UTC).
4. The TASK-62 ticket notes record the same evidence: environment, date/timezone, package file name and size, quality-gate outcomes with test counts, smoke-test outcomes, and any deviation or bug fix found.
5. The runtime dependency-license audit from TASK-1/TASK-4 is referenced as unchanged; no re-run is required because no runtime dependency was added.

## Inputs and outputs

| Input / output | Description |
|---|---|
| Source tree | Post-TASK-61 state: all m-4 management commands (glossary, ui-spec, ui-mockup, rule, audit, tech-spec) plus earlier milestones (req/nfreq, task/milestone/release, decision, search, browser, init/config). |
| Toolchain | Rust 1.99.1 via `rust-toolchain.toml`; `dpkg-deb`, `fakeroot`; Debian Forky amd64. |
| `scripts/build-deb.sh` | Unchanged packaging script; reads the version from `Cargo.toml`. |
| `papyrus_<version>_amd64.deb` | Built package in the project root. |
| Isolated root/admin | `./.tmp/` directories for `dpkg --instdir/--admindir`; removed after the run. |
| `tests/cli_milestone_smoke.rs` | New milestone-level integration test. |
| `docs/packaging.md` | Updated procedures, checklist, and observed results. |
| Ticket notes | Verification evidence for TASK-62. |

## Validation and error behavior

| Condition | Consequence |
|---|---|
| `cargo build --release`, `cargo test`, clippy, or `fmt --check` fails | Blocked; fix before packaging. |
| `target/release/papyrus --help` lacks `glossary`/`ui-spec`/`ui-mockup`/`rule`/`audit`/`tech-spec`/`decision`/`task`/`milestone`/`release`/`search`/`browser` (stale binary packaged) | Blocked; rebuild before `build-deb.sh`. |
| `dpkg-deb -I` metadata mismatch (name, version, architecture, depends, copyright) | Blocked. |
| Package contains asset/config/man files | Blocked. |
| `dpkg -i` fails in the isolated Forky root | Blocked. |
| Host package database or filesystem modified | Blocked; isolation is mandatory. |
| Installed binary missing from `PATH` after install, or still present after purge | Blocked. |
| Any smoke check of sections 3.1–3.7 fails | Blocked; record the failing command and output. |
| Outbound connection succeeds inside the offline namespace | Offline claim not verified; investigate before completion. |
| Served content references an external origin, or an asset request returns 404/empty | Blocked. |
| `cargo test` fails, including the new test | Blocked. |
| A defect in product code is found | Fix minimally, record it in the notes, keep the fix separate from packaging changes. |
| A failure is caused by a requirement/spec conflict (not by the package) | Record as an open question and escalate; do not change requirements in this ticket. |

## Acceptance criteria mapping

| Check | Verifies | How it is verified |
|---|---|---|
| T-01 | Pinned 1.99.1 release build succeeds and exposes all m-4 subcommands | `cargo build --release`; `--help` subcommand list |
| T-02 | Quality gates pass | `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`; recorded test counts |
| T-03 | `.deb` builds with the Cargo version and amd64 architecture | `./scripts/build-deb.sh`; `dpkg-deb -I` |
| T-04 | Metadata: `Package: papyrus`, `Version`, `Architecture: amd64`, `Depends: libc6 (>= 2.34)`, GPL-3.0-or-later copyright | `dpkg-deb -I`; copyright file inspection |
| T-05 | Package contains only the binary and copyright; assets are embedded | `dpkg-deb -c`; asset requests from the installed binary |
| T-06 | Installs and uninstalls in an isolated Debian Forky environment, host untouched | `fakeroot dpkg --instdir/--admindir -i`, `-r`, `--purge`; host `dpkg -l` before/after |
| T-07 | Installed binary is on `PATH` at `/usr/bin/papyrus` and runs | `PATH`-prefixed `papyrus --version` |
| T-08 | Baseline CLI, `init` | Installed-binary smoke commands and exit codes |
| T-09 | Management lifecycles for the six m-4 record types | Installed-binary smoke commands and exit codes |
| T-10 | Search regression over already-shipped types | Installed-binary search commands |
| T-11 | Browser regression over already-shipped categories | Installed-binary browser commands plus HTTP requests |
| T-12 | 320 px markup and no external references | Markup/asset assertions on served content |
| T-13 | Offline operation with no internet access | `unshare -rn` run with a failing outbound connection, then search + browser smoke |
| T-14 | Browser configuration keys work in the installed binary | `config list/get/set/unset` and startup-line assertions |
| T-15 | Embedded assets and no runtime file dependency | Asset requests from an empty working directory |
| T-16 | Milestone-level integration test added without duplicating management contract tests | `tests/cli_milestone_smoke.rs`; `cargo test` |
| T-17 | Documentation states Forky only and records procedures and results with date/timezone | `docs/packaging.md` review |
| T-18 | Evidence recorded in the ticket | TASK-62 notes review |

## Assumptions

- Assumption: the verification host is Debian Forky amd64. At refinement it reports `Debian GNU/Linux forky/sid`, glibc 2.43, `dpkg` 1.23.11, with `fakeroot`, `dpkg-deb`, `ip`, and `unshare -rn` available, and `rustc`/`cargo` 1.99.1 pinned by `rust-toolchain.toml`. The isolated `dpkg --instdir/--admindir` install therefore runs against Forky's glibc and dpkg, satisfying the NFR-03.002 verification method without a container.
- Assumption: isolation via `fakeroot dpkg --instdir/--admindir` is acceptable; a Forky container, chroot, or VM may substitute with the same checks (OQ-2).
- Assumption: scratch directories live under `./.tmp/` and are removed before the task ends (project convention forbids system temporary directories).
- Assumption: the package version stays `0.1.0`; no version bump is defined for m-4 and release versioning belongs to release orchestration.
- Assumption: `scripts/build-deb.sh` is used unchanged and does not force a rebuild; `cargo build --release` must run first. The repository may contain a stale `target/release/papyrus` and `papyrus_0.1.0_amd64.deb` built before TASK-30/TASK-34/TASK-38/TASK-43/TASK-47/TASK-55/TASK-61, so packaging without a rebuild would ship a binary lacking the m-4 command families. AC #2 guards this.
- Assumption: `glossary`, `ui-spec`, `ui-mockup`, `rule`, `audit`, `tech-spec`, `decision`, `task`, `milestone`, `release`, `search`, and `browser` require no new runtime dependency; the TASK-1/TASK-4 dependency-license audit remains valid (NFR-06.002).
- Assumption: no UI change ships in this ticket, so MOCK-0001, MOCK-0002, and MOCK-0003 remain the layout references and no new mockup is produced.
- Assumption: NFR-05.001 browser-version support was verified at markup/feature-subset level by TASK-6/TASK-11/TASK-15 and is not re-verified in a real GUI browser here (OQ-1).
- Assumption: the new integration test runs the built binary, not the installed package; the installed-package checks are the manual/scripted checklist in sections 2–3, recorded in `docs/packaging.md`.
- Assumption: the offline check is environment-dependent; `unshare -rn` is available at refinement, with the documented fallback if not.
- Assumption: the six m-4 record types have no browser or search coverage at m-4 time (search/browser for them are delivered by later milestones). TASK-62 therefore verifies only their management lifecycles and regresses already-shipped search/browser over req/task/milestone/release/decision. Search/browser smoke over the new types is explicitly out of scope and is verified by TASK-33/TASK-37/TASK-42/TASK-46/TASK-50/TASK-59.
- Assumption: disabled document types are rejected by `create` and hidden from `list` per BR-016; the smoke test asserts this at the CLI level for at least one of the six types.

## Open questions

None of these blocks implementation; each has a specified default. OQ-1 is recommended for user confirmation before release.

- **OQ-1 (recommended default: markup/asset-level verification only)** — NFR-05.001's verification method asks for checks in Firefox/Edge/Safari 100+. This headless packaging task verifies the served markup, asset set, viewport hooks, and absence of forbidden features, but does not drive a real GUI browser. Default: record the limitation in `docs/packaging.md` and rely on TASK-6/TASK-11/TASK-15 markup-level verification plus the approved MOCK-0001/MOCK-0002 and pending MOCK-0003; schedule a manual browser pass outside this ticket if the user requires one. Non-blocking.
- **OQ-2 (recommended default: fakeroot `dpkg --instdir/--admindir` on the Forky build host)** — Whether a full Forky container/VM is mandatory. Default: use the fakeroot isolated install, which exercises Forky's glibc and dpkg and never touches the host; if the user requires container-level isolation, the same checklist applies unchanged. Non-blocking.
- **OQ-3 (recommended default: a new `tests/cli_milestone_smoke.rs`)** — Whether the milestone test belongs in an existing smoke file or a new file. Default: a new file keeps each milestone's smoke coverage separate and avoids growing an already consolidated file. Non-blocking.
- **OQ-4 (recommended default: keep version 0.1.0)** — No m-4 release version is defined in the requirements. Default: package the current Cargo version unchanged; a bump belongs to release orchestration. Non-blocking.
- **OQ-5 (resolved by user choice during refinement)** — Smoke-test scope for the six new record types. Resolution: management-only (create/list/view/edit/deprecate/restore). Search and browser smoke over the new types are out of scope and deferred to their own milestone packaging tasks, because search (SearchType) and browser (Category) do not yet cover them at m-4 time. Non-blocking.

## Traceability

| Artifact | Reference |
|---|---|
| Requirement | REQ-05.003, REQ-05.004, REQ-05.005, REQ-05.006, REQ-05.007, REQ-05.008, REQ-05.009, REQ-06, REQ-06.001, REQ-06.002 |
| Non-functional | NFR-01.002, NFR-01.003, NFR-02.002, NFR-03.002, NFR-04.001, NFR-04.002, NFR-05.001, NFR-06.001, NFR-06.002 |
| Business rule | BR-004 (browser command behavior), BR-011 (glossary), BR-012 (ui-spec), BR-013 (rule), BR-014 (audit), BR-015 (search), BR-016 (configuration keys and precedence), BR-017 (tech-spec) |
| ADR | ADR-0001 (Rust), ADR-0003 (Debian Forky only; supersedes ADR-0002), ADR-0004 (repository-root single crate), ADR-0005 (embedded HTTP server and server-rendered UI), ADR-0006 (optimistic concurrency and atomic writes) |
| Ticket | TASK-62 |
| UI mockups | none new; MOCK-0001 and MOCK-0002 remain approved layout references; MOCK-0003 (pending approval, TASK-15) covers the decision category |
| Related specification | `docs/specifications/glossary-terms-management.md` (TASK-30) |
| Related specification | `docs/specifications/ui-specifications-management.md` (TASK-34) |
| Related specification | `docs/specifications/ui-mockups-management.md` (TASK-38) |
| Related specification | `docs/specifications/business-rules-management.md` (TASK-43) |
| Related specification | `docs/specifications/audit-records-management.md` (TASK-47) |
| Related specification | `docs/specifications/technical-specifications-management.md` (TASK-55) |
| Related specification | `docs/specifications/decision-records-milestone-packaging-and-verification.md` (TASK-16 precedent) |
| Related specification | `docs/specifications/backlog-discovery-milestone-packaging-and-verification.md` (TASK-12 precedent) |
| Related specification | `docs/specifications/search-browser-milestone-packaging-and-verification.md` (TASK-7 precedent) |
| Related specification | `docs/specifications/requirements-management-milestone-packaging-and-verification.md` (TASK-4 precedent) |
| Packaging documentation | `docs/packaging.md` |
path: string,