# Feature Specification — Technical Specifications Milestone Packaging and Verification

**Ticket:** TASK-59
**Milestone:** m-11 (Technical Specifications)
**Status:** Refined (execution-ready)
**UI mockups:** none new — no UI change in this ticket; the shipped browser interface keeps the approved baseline MOCK-0001…MOCK-0008 and the approved MOCK-0009, which covers the Technical Specifications category.

## Purpose

Package the current Papyrus feature set (all work through TASK-58, including TASK-55 technical-specification management, TASK-56 technical-specification search, TASK-57 technical-specification browser access, and the TASK-58 design review) as an installable amd64 Debian package, and verify in an isolated Debian Forky environment that the packaged binary installs, uninstalls, and provides technical-specification creation, listing, viewing, editing, deprecation, restoration, search, and offline local browser access end to end.

This is the last task of milestone m-11 and closes its "each increment is packaged and usable" goal for technical specifications. It follows the pattern of the completed milestone-packaging tasks TASK-33 (glossary), TASK-37 (UI specifications), TASK-42 (UI mockups), TASK-46 (business rules), and TASK-50 (audit records).

This is a packaging and verification task: no new product behavior is added. Work is limited to building, packaging, verifying, documenting, and adding one milestone-level integration test.

## Scope

### In scope

- Rebuild `target/release/papyrus` and `papyrus_<version>_amd64.deb` from the current source with the pinned Rust 1.99.1 toolchain, using `scripts/build-deb.sh` unchanged.
- Verify package metadata: `Package: papyrus`, `Architecture: amd64`, `Version` equal to the Cargo package version, `Depends: libc6 (>= 2.34)`, GPL-3.0-or-later copyright.
- Verify package contents contain only the binary and the copyright file — in particular, no runtime asset files (`src/browser/app.css`, `app.js`, templates) are packaged, because they are compiled into the binary.
- Install and uninstall the package in an isolated Debian Forky environment without modifying the host.
- Smoke-test the installed binary:
  - CLI baseline: `--help`, `--version`, `init`, and the technical-specification lifecycle (create, list, view, edit, deprecate, restore).
  - Search over technical specifications: quoted phrase, split words, `--type doc`, other type filters, `--limit`, `--json`, empty result, deprecation visibility.
  - Browser over the Technical Specifications category (`?type=tech-spec`): start with `--no-open` on a free port, list, record view, edit round-trip, field-preserving write, JSON API (`/api/tech-specs`, `/api/tech-spec`, `/api/doc/<id>`), embedded assets, 320 px markup, no external references; deprecated specifications are hidden and unreachable (404 on view, edit, POST, and API).
- Verify offline operation: search and browser work with no internet access, using only loopback and locally served assets.
- Verify the browser configuration keys (`browser_host`, `browser_port`, `browser_open`) behave as documented in the installed binary.
- Re-run the Rust quality gates: `cargo build --release`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check` with Rust 1.99.1.
- Add one milestone-level integration test file `tests/technical_specifications_smoke.rs` covering the technical-specification lifecycle, search, and browser together against the built binary, without duplicating the existing `tests/technical_specifications_cli.rs`, `tests/search_cli.rs`, or `tests/browser_cli.rs` contract coverage.
- Update `docs/packaging.md` with the verified supported release, install/uninstall steps, the extended smoke-test checklist, and observed results (pass/fail, environment, date/timezone).
- Record verification evidence in the TASK-59 ticket notes.

### Out of scope

- New product features, commands, options, record types, or UI changes; technical-specification behavior is delivered by TASK-55 (management), TASK-56 (search), TASK-57 (browser), and TASK-58 (design review, MOCK-0009 approved).
- A `supersede` or `delete` command for technical specifications — BR-017 defines create/list/view/edit/deprecate/restore only. The lifecycle smoke covers those commands only and claims no supersede step.
- A dedicated `--type tech-spec` search filter — the documented `--type` vocabulary is `task`, `milestone`, `release`, `decision`, `doc`; technical specifications are searched through `--type doc` (and default search).
- Structured references between technical specifications and non-functional requirements or tasks — BR-017 defines no reference fields; such IDs remain literal text in the title or description.
- Verification on Debian Trixie or any Debian release other than Forky (ADR-0003, NFR-03.002).
- Real-GUI rendering checks in Firefox, Edge, or Safari; only markup- and asset-level verification is performed (OQ-1).
- Re-running the runtime dependency-license audit (no new runtime dependencies; TASK-1/TASK-4 audit remains valid).
- Changes to requirements (`docs/requirements/`), business rules (`docs/business-rules/`), ADRs (`docs/architecture/adr/`), or packaging scripts (`scripts/build-deb.sh`, `packaging/deb/**`).
- New UI mockups; the shipped browser interface is unchanged and MOCK-0001…MOCK-0009 stay authoritative.
- Product code changes, except a minimal bug fix when a verification step exposes a defect; any such fix is reported in the ticket notes and kept separate from the packaging work.
- Final cross-type completeness verification — TASK-20 (m-12).

**Milestone m-11 versus packaging:** this ticket packages and verifies the m-11 technical-specifications increment only; it does not re-deliver management, search, browser, or design-review behavior.

## Behavior

### 1. Build and package

1. `cargo build --release` succeeds with the pinned Rust 1.99.1 toolchain (`rust-toolchain.toml`) and produces `target/release/papyrus`.
2. `cargo test`, `cargo clippy --all-targets -- -D warnings`, and `cargo fmt --check` pass before packaging.
3. The freshly built binary exposes the technical-specification subcommand: `target/release/papyrus --help` lists `tech-spec` (alongside `init`, `config`, `req`, `nfreq`, `task`, `milestone`, `release`, `decision`, `glossary`, `ui-spec`, `ui-mockup`, `rule`, `audit`, `search`, and `browser`).
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
   A container, chroot, or VM on Forky may be used instead; the same checks apply (OQ-2).
2. The isolated root and admin directories are created under `./.tmp/` in the workspace (project convention: no system temporary directories) and are removed after the run, including on failure.
3. Before and after the run, the host reports papyrus as not installed (`dpkg -l papyrus` has no installed entry), or its pre-existing status is recorded as unchanged, proving the host was untouched.
4. `dpkg -i` in the isolated root succeeds, exit 0.
5. After install, `<root>/usr/bin/papyrus` exists with mode `0755` and owner `root:root`.
6. With `<root>/usr/bin` first on `PATH`, `papyrus --version` resolves to the installed binary and prints `papyrus <version>`.
7. `dpkg -r papyrus` (isolated) removes the binary; `papyrus` no longer resolves on that `PATH`.
8. `dpkg --purge papyrus` (isolated) removes the remaining files and the admin entry; `<root>` contains no papyrus files afterwards.

### 3. Installed-binary smoke tests

All commands run against the installed binary with `--dir <project>` pointing at a scratch project under `./.tmp/`. Exit statuses follow the existing contract: 0 success, 1 IO/boundary, 2 usage, 3 not found, 4 validation.

#### 3.1 CLI baseline and technical-specification lifecycle

1. `papyrus --help` exits 0 and lists `tech-spec`, `search`, and `browser` among the subcommands.
2. `papyrus --version` exits 0 and prints `papyrus <version>`.
3. `papyrus --dir <project> init <name> --no-git` creates `papyrus.config.yml` and the configured directory structure, including the technical-specification directory and its `deprecated/` subfolder.
4. Technical-specification lifecycle (BR-017, REQ-05.009, REQ-05.007):
   - `papyrus tech-spec create <title> --description <text>` prints `TECH-001` (or the next free ID), and the file is written as UTF-8 Markdown `<doc_directory>/technical/TECH-001.md` with a `# TECH-001 <title>` heading, a `## Description` section, and a single trailing newline.
   - `tech-spec list` lists active specifications in ID ascending order as `TECH-001 <title>`; deprecated specifications are hidden.
   - `tech-spec view TECH-001` prints the full persisted text, including the heading and `## Description`; viewing a deprecated specification by ID is allowed; an unknown ID exits 3.
   - `tech-spec edit TECH-001 --title <new>` and `tech-spec edit TECH-001 --description <new>` update only the supplied field and preserve the other; the ID, file name, and folder are unchanged.
   - `tech-spec deprecate TECH-001` moves the file into `deprecated/`; `tech-spec list` no longer shows it; an already-deprecated specification exits 4; an unknown ID exits 3.
   - `tech-spec restore TECH-001` moves it back to the active folder with the same ID and unchanged content; an already-active specification exits 4; an unknown ID exits 3.
   - IDs are preserved across deprecate/restore cycles and are never reused across the active and `deprecated/` folders.
   - Missing `<title>` or `--description` exits 2; sanitization failures (empty values, control characters, path separators) exit 4 and write nothing.
   - When `document_types.tech-spec.enabled` is `false`, `create` and `list` report a validation error (exit 4) and no records are created or shown (BR-016).
   - `--editor` behavior remains covered by `tests/technical_specifications_cli.rs`; it is environment-dependent and is not part of the isolated-install smoke run.

#### 3.2 Search (technical specifications and existing records)

1. A quoted phrase present contiguously in a technical specification's title or description is found: `papyrus search "<phrase>"` prints `TECH-001 doc <title>` lines in ascending ID order.
2. The same phrase split into separate words is not treated as a phrase match.
3. `--type doc` restricts results to document records and includes technical specifications; technical specifications carry the `doc` type value. Other `--type` values (`task`, `milestone`, `release`, `decision`) do not return technical specifications.
4. `papyrus search <term> --limit 1` prints at most one result line; a limit larger than the match count returns all matches.
5. `papyrus --json search <term>` prints a JSON array of `{id, type, title, path}` objects; a search with no matches prints `[]` and exits 0.
6. Deprecated technical specifications are absent from search results; restored specifications reappear.
7. Usage errors (missing query, unknown `--type`, `--limit 0`) exit 2; an empty or control-character query exits 4. A `--type tech-spec` value is not part of the documented vocabulary and exits 2.
8. A missing or empty technical-specification directory yields an empty result with exit 0 and creates nothing.

#### 3.3 Browser (Technical Specifications category)

1. `papyrus --dir <project> browser --no-open --port <free-port>` prints exactly one startup line `Serving http://127.0.0.1:<free-port>/` and serves until killed. `<free-port>` is obtained by binding `127.0.0.1:0`, reading the port, and dropping the listener, with a retry if the bind fails.
2. `--no-open` suppresses the platform opener.
3. `GET /?type=tech-spec` renders the Technical Specifications category. The sidebar lists Business requirements, Business Rules, Audit Records, Technical Specifications, Non-functional requirements, Backlog, Decisions, Glossary, UI Specifications, and UI Mockups, and the selected Technical Specifications entry carries `aria-current="page"` with the non-colour cue. The default category stays `req`; any other `?type=` value, including `all`, is `400`.
4. The list renders one `h2` "Technical Specifications" with the record count and one row per active specification in ascending ID order, each row linking `ID + title` to `/doc/{id}` with the first paragraph of the description as a preview. Empty and long-title states render; non-matching `.md` files, non-`.md` files, and `deprecated/` are never listed or counted.
5. `GET /doc/TECH-001` renders the view: record ID (monospace), record form `Technical specification`, category `Technical Specifications`, the title as `h1`, and the description as body text with `white-space: pre-wrap` and no "Description" heading; an empty description shows `(none)`. The breadcrumb is `Technical Specifications` then the ID, and the page offers exactly one action, `Edit`; the raw file is never served or embedded.
6. `GET /doc/TECH-001/edit` yields the form matching MOCK-0009: required title input, required multi-line description textarea, hidden `version` input plus the visible non-interactive annotation `Hidden field: version (optimistic-concurrency token)`, and `Save changes`/`Cancel`. The pristine, dirty, validation-error (422), conflict (409), and 320 CSS-pixel states are present, and the form works with JavaScript disabled.
7. Edit round-trip: `POST /doc/TECH-001` with `version`, `title`, and `description` returns `303` with `Location: /doc/TECH-001`; the follow-up `GET` shows the new title and description, and `papyrus tech-spec view TECH-001` shows the same persisted values.
8. Field-preserving write: a technical specification seeded with a preamble and an editor-added unknown section is edited through the browser; a byte comparison of the pre-edit and saved files shows only the `# <ID> <title>` heading and the `## Description` body changed. The preamble and every other section and line are content-identical; the file is never regenerated from a parsed record or the CLI serializer.
9. Validation: empty or over-long title, control characters, empty or over-long description, and a description line starting with `## ` return `422` with the documented messages, preserve the submitted values, and write nothing. A missing or malformed `version` returns `400`; a stale `version` returns `409` with the documented message and writes nothing.
10. Deprecated technical specifications are not listed or counted and return `404` on `GET /doc/{id}`, `GET /doc/{id}/edit`, `POST /doc/{id}`, `GET /api/doc/{id}`, and `POST /api/doc/{id}`. Every HTML `404` for a technical-specification ID (deprecated or unknown) renders the identical Technical Specifications 404 page: status `404`, heading `Record not found`, help text "Deprecated technical specifications are not listed and cannot be opened. The same page is used for an unknown address.", and exactly one recovery link "Back to technical specifications" to `/?type=tech-spec`. JSON `404` responses keep the baseline JSON error object. No browser control changes deprecation state.
11. JSON API:
    - `GET /api/tech-specs` returns `200 application/json` with an ascending array of `{"id":"TECH-001","title":"..."}`; the alias `/api/tech-spec` returns the same response.
    - `GET /api/doc/TECH-001` returns `{"id","kind":"tech-spec","form":"tech-spec","title","description","version","path"}`, where `description` may be `null` when the section is absent or empty, and `path` is project-relative with `/` separators.
    - `POST /api/doc/{id}` accepts JSON with `version` and optional `title`/`description`, leaves omitted members unchanged, rejects a null or empty description, returns `400` for wrong types, and `422` for invalid values.
12. `GET /assets/app.css` returns `200 text/css` and `GET /assets/app.js` returns `200 text/javascript`, both with `Cache-Control: no-store`; the bodies are non-empty.
13. 320 px markup: every page carries `<meta name="viewport" content="width=device-width, initial-scale=1">`, the document-type sidebar toggle is present with `aria-expanded` and `aria-controls`, and the served CSS/JS use no forbidden features from the TASK-6 browser baseline.
14. No external references: served HTML, CSS, and JS contain no `http://`, `https://`, protocol-relative `//` remote reference, remote `@import`, remote `url(...)`, `<img>`, web font, or CDN reference; HTML responses carry the documented Content-Security-Policy.
15. A missing or empty technical-specification directory, and a configuration with `document_types.tech-spec.enabled: false`, render the category empty state instead of failing (no server error), and the disabled sidebar entry is omitted.

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

1. A new file `tests/technical_specifications_smoke.rs` provides a milestone-level end-to-end test against the built binary (`CARGO_BIN_EXE_papyrus`), using temporary project directories under `./.tmp/`, a free loopback port, and the same startup-line wait and child-kill-on-drop pattern as `tests/browser_cli.rs`.
2. Test 1 — `milestone_technical_specifications_flow`: initialize a project; create `TECH-001` and a second specification carrying a unique multi-word phrase in the description; assert the lifecycle (create/list/view/edit by title and by description without losing the other field/deprecate/restore with ID preservation and the documented exit codes); exercise search (contiguous quoted phrase, split words no match, `--type doc`, other `--type` filters, `--limit 1`, `--json` shape and empty `[]`, deprecated hidden until restored); seed a preamble and an unknown section, then start `browser --no-open` on a free port and assert the startup line, `/?type=tech-spec` with the sidebar `aria-current="page"` and position after Audit Records, the list count/rows/preview and deprecated exclusion, the record view (form, category, title, description body without a "Description" heading, single `Edit`), the edit round-trip (`303` then updated view and matching CLI `view`), the field-preserving byte comparison of untouched sections, the JSON API (`/api/tech-specs`, `/api/tech-spec`, `/api/doc/TECH-001`), both assets, the 320 px markup hooks, the absence of external references, and the deprecated `404` on view, edit, POST, and API (including the identical 404 page text and recovery link).
3. Test 2 — `installed_layout_has_no_runtime_asset_dependency`: run the binary with its working directory set to an empty directory and the project directory containing no asset files; assert `search` (including a technical specification) and `browser` still work and served assets are non-empty.
4. The new tests must not duplicate the TASK-55 contract coverage in `tests/technical_specifications_cli.rs`, the TASK-56 coverage in `tests/search_cli.rs`/`tests/search_backlog_cli.rs`, or the TASK-57 coverage in `tests/browser_cli.rs` (request limits, complete error-status matrix, XSS, symlink escape, conflict matrix, accessibility audit). Those remain authoritative there.
5. Tests require no internet access, bind only loopback, and leave no child process running.
6. `cargo test` passes with the new tests. Baseline at refinement: the m-11 test suite (unit + `cli` + `milestone_cli` + `milestone_smoke` + `req_cli` + `task_cli` + `decision_cli` + `glossary_cli` + `ui_spec_cli` + `ui_mockup_cli` + `business_rules_cli` + `audit_records_cli` + `technical_specifications_cli` + `search_cli` + `search_backlog_cli` + `browser_cli` + `backlog_discovery_smoke` + `decision_records_smoke` + `search_browser_smoke` + `cli_milestone_smoke` + `glossary_terms_smoke` + `ui_specifications_smoke` + `ui_mockups_smoke` + `business_rules_smoke` + `audit_records_smoke`) all passing; the exact count is recorded during verification.

### 5. Documentation and evidence

1. `docs/packaging.md` states the supported release as Debian Forky only (amd64), per ADR-0003 and NFR-03.002, and explicitly states that Trixie and other releases are not claimed or verified.
2. `docs/packaging.md` contains the build/package commands, the isolated install and uninstall procedure (scratch directories under `./.tmp/`, removed afterwards), and the extended smoke-test checklist covering sections 3.1–3.6.
3. `docs/packaging.md` records observed results as a pass/fail table with the environment (Debian Forky version, glibc, `rustc`/`cargo` 1.99.1, `dpkg` version, isolation method, whether `unshare -rn` was available) and the date with timezone.
4. The TASK-59 ticket notes record the same evidence: environment, date/timezone, package file name and size, quality-gate outcomes with test counts, smoke-test outcomes, and any deviation or bug fix found.
5. The runtime dependency-license audit from TASK-1/TASK-4 is referenced as unchanged; no re-run is required because no runtime dependency was added.

## Inputs and outputs

| Input / output | Description |
|---|---|
| Source tree | Post-TASK-58 state: technical-specification management (TASK-55), search (TASK-56), browser (TASK-57), design review (TASK-58, MOCK-0009 approved), plus all earlier features. |
| Toolchain | Rust 1.99.1 via `rust-toolchain.toml`; `dpkg-deb`, `fakeroot`; Debian Forky amd64. |
| `scripts/build-deb.sh` | Unchanged packaging script; reads the version from `Cargo.toml`. |
| `papyrus_<version>_amd64.deb` | Built package in the project root. |
| Isolated root/admin | `./.tmp/` directories for `dpkg --instdir/--admindir`; removed after the run. |
| `tests/technical_specifications_smoke.rs` | New milestone-level integration test. |
| `docs/packaging.md` | Updated procedures, checklist, and observed results. |
| Ticket notes | Verification evidence for TASK-59. |

## Validation and error behavior

| Condition | Consequence |
|---|---|
| `cargo build --release`, `cargo test`, clippy, or `fmt --check` fails | Blocked; fix before packaging. |
| `target/release/papyrus --help` lacks `tech-spec`/`search`/`browser` (stale binary packaged) | Blocked; rebuild before `build-deb.sh`. |
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
| T-01 | Pinned 1.99.1 release build succeeds and exposes tech-spec/search/browser | `cargo build --release`; `--help` subcommand list |
| T-02 | Quality gates pass | `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`; recorded test counts |
| T-03 | `.deb` builds with the Cargo version and amd64 architecture | `./scripts/build-deb.sh`; `dpkg-deb -I` |
| T-04 | Metadata: `Package: papyrus`, `Version`, `Architecture: amd64`, `Depends: libc6 (>= 2.34)`, GPL-3.0-or-later copyright | `dpkg-deb -I`; copyright file inspection |
| T-05 | Package contains only the binary and copyright; assets are embedded | `dpkg-deb -c`; asset requests from the installed binary |
| T-06 | Installs and uninstalls in an isolated Debian Forky environment, host untouched | `fakeroot dpkg --instdir/--admindir -i`, `-r`, `--purge`; host `dpkg -l` before/after |
| T-07 | Installed binary is on `PATH` at `/usr/bin/papyrus` and runs | `PATH`-prefixed `papyrus --version` |
| T-08 | Baseline CLI and technical-specification lifecycle | Installed-binary smoke commands and exit codes |
| T-09 | Search over technical specifications | Installed-binary search commands |
| T-10 | Browser: startup with `--no-open`, Technical Specifications category, list, view, edit round-trip, field-preserving write | Installed-binary browser commands plus HTTP requests |
| T-11 | Browser: JSON API and embedded assets served from the binary | `GET /api/*`, `GET /assets/*` |
| T-12 | 320 px markup and no external references | Markup/asset assertions on served content |
| T-13 | Deprecated technical specifications hidden from list, search, and browser; `404` on view, edit, POST, and API | Deprecate/restore fixtures plus HTTP assertions |
| T-14 | Offline operation with no internet access | `unshare -rn` run with a failing outbound connection, then search + browser smoke |
| T-15 | Browser configuration keys work in the installed binary | `config list/get/set/unset` and startup-line assertions |
| T-16 | Milestone-level integration test added without duplicating lifecycle/search/browser contract tests | `tests/technical_specifications_smoke.rs`; `cargo test` |
| T-17 | Documentation states Forky only and records procedures and results with date/timezone | `docs/packaging.md` review |
| T-18 | Evidence recorded in the ticket | TASK-59 notes review |

## Assumptions

- Assumption: the verification host is Debian Forky amd64. At refinement it reports `Debian GNU/Linux forky/sid`, glibc 2.43, `dpkg` 1.23.11, with `fakeroot`, `dpkg-deb`, `ip`, and `unshare -rn` available, and `rustc`/`cargo` 1.99.1 pinned by `rust-toolchain.toml`. The isolated `dpkg --instdir/--admindir` install therefore runs against Forky's glibc and dpkg, satisfying the NFR-03.002 verification method without a container.
- Assumption: isolation via `fakeroot dpkg --instdir/--admindir` is acceptable; a Forky container, chroot, or VM may substitute with the same checks (OQ-2).
- Assumption: scratch directories live under `./.tmp/` and are removed before the task ends (project convention forbids system temporary directories).
- Assumption: the package version stays `0.1.0`; no version bump is defined for m-11 and release versioning belongs to release orchestration (OQ-4).
- Assumption: `scripts/build-deb.sh` is used unchanged and does not force a rebuild; `cargo build --release` must run first. The repository may contain a stale `target/release/papyrus` and `papyrus_0.1.0_amd64.deb` built before TASK-57, so packaging without a rebuild would ship a binary lacking technical-specification browser support. AC #2 guards this.
- Assumption: `tech-spec`, `search`, and `browser` require no new runtime dependency; the TASK-1/TASK-4 dependency-license audit remains valid (NFR-06.002).
- Assumption: no UI change ships in this ticket, so MOCK-0001…MOCK-0009 remain the approved layout references and no new mockup is produced. MOCK-0009 is `approved` in the mockup registry (user approval recorded 2026-11-07 UTC, TASK-58).
- Assumption: NFR-05.001 browser-version support was verified at markup/feature-subset level by TASK-6/TASK-57 and is not re-verified in a real GUI browser here (OQ-1).
- Assumption: the new integration test runs the built binary, not the installed package; the installed-package checks are the manual/scripted checklist in sections 2–3, recorded in `docs/packaging.md`.
- Assumption: the offline check is environment-dependent; `unshare -rn` is available at refinement, with the documented fallback if not.
- Assumption: technical specifications support `restore` after `deprecate` (BR-017); smoke tests assert restoration and ID preservation.
- Assumption: `--description` is a single-line sanitized CLI input and `--editor` is the CLI route for multi-line bodies; the browser otherwise allows a multi-line description. `--editor` is not exercised in the isolated install because it depends on the environment; `tests/technical_specifications_cli.rs` remains its contract coverage.
- Assumption: the browser requires both title and description; an on-disk record with an absent or empty description is viewable as `(none)` but must be completed before it can be saved from the browser.
- Assumption: deprecated technical specifications are excluded from list, search, and browser per REQ-05.007/BR-017; the smoke test asserts exclusion at all three surfaces.
- Assumption: the dedicated packaging specification authored with this refinement (`docs/specifications/technical-specifications-milestone-packaging-and-verification.md`) is the canonical home for this ticket; the generic `docs/specifications/cli-milestone-packaging-and-verification.md` (TASK-62) remains the m-4 precedent and does not claim search/browser coverage for technical specifications.

## Open questions

None of these blocks implementation; each has a specified default. OQ-1 is recommended for user confirmation before release.

- **OQ-1 (recommended default: markup/asset-level verification only)** — NFR-05.001's verification method asks for checks in Firefox/Edge/Safari 100+. This headless packaging task verifies the served markup, asset set, viewport hooks, and absence of forbidden features, but does not drive a real GUI browser. Default: record the limitation in `docs/packaging.md` and rely on TASK-6/TASK-57 markup-level verification plus the approved MOCK-0001…MOCK-0009; schedule a manual browser pass outside this ticket if the user requires one. Non-blocking.
- **OQ-2 (recommended default: fakeroot `dpkg --instdir/--admindir` on the Forky build host)** — Whether a full Forky container/VM is mandatory. Default: use the fakeroot isolated install, which exercises Forky's glibc and dpkg and never touches the host; if the user requires container-level isolation, the same checklist applies unchanged. Non-blocking.
- **OQ-3 (recommended default: a new `tests/technical_specifications_smoke.rs`)** — Whether the milestone test belongs in an existing smoke file or a new file. Default: a new file keeps each milestone's smoke coverage separate and avoids growing an already consolidated file. Non-blocking.
- **OQ-4 (recommended default: keep version 0.1.0)** — No m-11 release version is defined in the requirements. Default: package the current Cargo version unchanged; a bump belongs to release orchestration. Non-blocking.
- **OQ-5 (recommended default: re-verify the lifecycle in this milestone package)** — TASK-62 (m-4 CLI packaging) already verified the technical-specification management lifecycle at the CLI level. Default: repeat the lifecycle smoke here against the installed m-11 package because this ticket must prove the packaged binary provides the complete technical-specification increment, and the repeat is small. The contract detail stays in `tests/technical_specifications_cli.rs`. Non-blocking.

## Traceability

| Artifact | Reference |
|---|---|
| Requirement | REQ-05.007, REQ-05.009, REQ-06.001, REQ-06.002 |
| Non-functional | NFR-01.002, NFR-01.003, NFR-02.002, NFR-03.002, NFR-04.001, NFR-04.002, NFR-05.001, NFR-06.001, NFR-06.002 |
| Business rule | BR-004 (browser command behavior), BR-015 (search command behavior), BR-016 (configuration keys and precedence), BR-017 (tech-spec command behavior) |
| ADR | ADR-0001 (Rust), ADR-0003 (Debian Forky only; supersedes ADR-0002), ADR-0004 (repository-root single crate), ADR-0005 (embedded HTTP server and server-rendered UI), ADR-0006 (optimistic concurrency and atomic writes) |
| Ticket | TASK-59 |
| UI mockups | none new; MOCK-0001…MOCK-0008 are the approved baseline; MOCK-0009 (approved, TASK-58) covers the Technical Specifications category |
| Related specification | `docs/specifications/technical-specifications-management.md` (TASK-55) |
| Related specification | `docs/specifications/search-technical-specifications.md` (TASK-56) |
| Related specification | `docs/specifications/browser-technical-specifications.md` (TASK-57) |
| Related specification | `docs/specifications/ui/browser-technical-specifications-ui.md` (TASK-58) |
| Related specification | `docs/specifications/glossary-terms-milestone-packaging-and-verification.md` (TASK-33 pattern) |
| Related specification | `docs/specifications/ui-specifications-milestone-packaging-and-verification.md` (TASK-37 pattern) |
| Related specification | `docs/specifications/cli-milestone-packaging-and-verification.md` (TASK-62 m-4 precedent) |
| Packaging documentation | `docs/packaging.md` |
| Domain model | `docs/domain-model/business-glossary/technical-specification.md` |
