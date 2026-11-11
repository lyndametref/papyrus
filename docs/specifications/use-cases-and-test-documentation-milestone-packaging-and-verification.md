# Feature Specification — Use Cases and Test Documentation Milestone Packaging and Verification

**Ticket:** TASK-82
**Milestone:** m-15 (Use Cases and Test Documentation)
**Status:** Refined (execution-ready)
**UI mockups:** none new — no UI change in this ticket; the shipped browser interface keeps the approved baseline MOCK-0001…MOCK-0012 and the approved MOCK-0013 (TASK-79), which covers the Use Cases, Test Cases, and Test Reports categories.

## Purpose

Package the current Papyrus feature set (all work through TASK-81, including TASK-78 use-case/test-case/test-report management, TASK-80 search, and TASK-81 browser access) as an installable amd64 Debian package, and verify in an isolated Debian Forky environment that the packaged binary installs, uninstalls, and provides use-case, test-case, and test-report creation, listing, viewing, editing, deprecation, restoration, search, and offline local browser access end to end.

This is the last task of milestone m-15 and closes its "each increment is packaged and usable" goal for use cases and test documentation. It follows the pattern of the completed milestone-packaging tasks TASK-20 (complete content management), TASK-59 (technical specifications), and the earlier milestone-packaging tasks.

This is a packaging and verification task: no new product behavior is added. Work is limited to building, packaging, verifying, documenting, and adding one milestone-level integration test.

## Scope

### In scope

- Rebuild `target/release/papyrus` and `papyrus_<version>_amd64.deb` from the current source with the pinned Rust 1.99.1 toolchain, using `scripts/build-deb.sh` unchanged.
- Verify package metadata: `Package: papyrus`, `Architecture: amd64`, `Version` equal to the Cargo package version, `Depends: libc6 (>= 2.34)`, GPL-3.0-or-later copyright.
- Verify package contents contain only the binary and the copyright file — in particular, no runtime asset files (`src/browser/app.css`, `app.js`, templates) are packaged, because they are compiled into the binary.
- Install and uninstall the package in an isolated Debian Forky environment without modifying the host.
- Smoke-test the installed binary:
  - CLI baseline: `--help`, `--version`, `init`, and the use-case/test-case/test-report lifecycle (create, list, view, edit, deprecate, restore).
  - Search over use cases, test cases, and test reports: quoted phrase, split words, `--type doc`, other type filters, `--limit`, `--json`, empty result, deprecation visibility.
  - Browser over the Use Cases, Test Cases, and Test Reports categories (`?type=use-case`, `?type=test-case`, `?type=test-report`): start with `--no-open` on a free port, list, record view, edit round-trip, field-preserving write, JSON API (`/api/use-cases`, `/api/use-case`, `/api/test-cases`, `/api/test-case`, `/api/test-reports`, `/api/test-report`, `/api/doc/<id>`), embedded assets, 320 px markup, no external references; deprecated records are hidden and unreachable (404 on view, edit, POST, and API).
- Verify offline operation: search and browser work with no internet access, using only loopback and locally served assets.
- Verify the browser configuration keys (`browser_host`, `browser_port`, `browser_open`) behave as documented in the installed binary.
- Re-run the Rust quality gates: `cargo build --release`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check` with Rust 1.99.1.
- Add one milestone-level integration test file `tests/use_cases_test_cases_test_reports_smoke.rs` covering the three-type lifecycle, search, and browser together against the built binary, without duplicating the existing `tests/use_cases_test_cases_and_test_reports_cli.rs` (if present), `tests/search_cli.rs`, or `tests/browser_cli.rs` contract coverage.
- Update `docs/packaging.md` with the verified supported release, install/uninstall steps, the extended smoke-test checklist, and observed results (pass/fail, environment, date/timezone).
- Record verification evidence in the TASK-82 ticket notes.

### Out of scope

- New product features, commands, options, record types, or UI changes; use-case/test-case/test-report behavior is delivered by TASK-78 (management), TASK-80 (search), TASK-81 (browser), and TASK-79 (design review, MOCK-0013 approved).
- A `supersede` or `delete` command for use cases, test cases, or test reports — BR-018/BR-019/BR-020 define create/list/view/edit/deprecate/restore only. The lifecycle smoke covers those commands only and claims no supersede step.
- A dedicated `--type use-case`, `--type test-case`, or `--type test-report` search filter — the documented `--type` vocabulary is `task`, `milestone`, `release`, `decision`, `doc`; all three types are searched through `--type doc` (and default search).
- Structured references between use cases/test cases/test reports and tasks or milestones — BR-018/BR-019/BR-020 define no reference fields beyond the general `references` list; such IDs remain literal text in the title, description, or `references`.
- Verification on Debian Trixie or any Debian release other than Forky (ADR-0003, NFR-03.002).
- Real-GUI rendering checks in Firefox, Edge, or Safari; only markup- and asset-level verification is performed.
- Re-running the runtime dependency-license audit (no new runtime dependencies; TASK-1/TASK-4 audit remains valid).
- Changes to requirements (`docs/requirements/`), business rules (`docs/business-rules/`), ADRs (`docs/architecture/adr/`), or packaging scripts (`scripts/build-deb.sh`, `packaging/deb/**`).
- New UI mockups; the shipped browser interface is unchanged and MOCK-0001…MOCK-0013 stay authoritative.
- Product code changes, except a minimal bug fix when a verification step exposes a defect; any such fix is reported in the ticket notes and kept separate from the packaging work.
- Final cross-type completeness verification — TASK-20 (m-12) already covers the complete content-management milestone; TASK-82 covers the m-15 increment only.

**Milestone m-15 versus packaging:** this ticket packages and verifies the m-15 use-cases-and-test-documentation increment only; it does not re-deliver management, search, browser, or design-review behavior.

## Behavior

### 1. Build and package

1. `cargo build --release` succeeds with the pinned Rust 1.99.1 toolchain (`rust-toolchain.toml`) and produces `target/release/papyrus`.
2. `cargo test`, `cargo clippy --all-targets -- -D warnings`, and `cargo fmt --check` pass before packaging.
3. The freshly built binary exposes the three subcommands: `target/release/papyrus --help` lists `use-case`, `test-case`, and `test-report` (alongside `init`, `config`, `req`, `nfreq`, `task`, `milestone`, `release`, `decision`, `glossary`, `ui-spec`, `ui-mockup`, `rule`, `audit`, `tech-spec`, `search`, and `browser`).
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
   A container, chroot, or VM on Forky may be used instead; the same checks apply.
2. The isolated root and admin directories are created under `./.tmp/` in the workspace (project convention: no system temporary directories) and are removed after the run, including on failure.
3. Before and after the run, the host reports papyrus as not installed (`dpkg -l papyrus` has no installed entry), or its pre-existing status is recorded as unchanged, proving the host was untouched.
4. `dpkg -i` in the isolated root succeeds, exit 0.
5. After install, `<root>/usr/bin/papyrus` exists with mode `0755` and owner `root:root`.
6. With `<root>/usr/bin` first on `PATH`, `papyrus --version` resolves to the installed binary and prints `papyrus <version>`.
7. `dpkg -r papyrus` (isolated) removes the binary; `papyrus` no longer resolves on that `PATH`.
8. `dpkg --purge papyrus` (isolated) removes the remaining files and the admin entry; `<root>` contains no papyrus files afterwards.

### 3. Installed-binary smoke tests

All commands run against the installed binary with `--dir <project>` pointing at a scratch project under `./.tmp/`. Exit statuses follow the existing contract: 0 success, 1 IO/boundary, 2 usage, 3 not found, 4 validation.

#### 3.1 CLI baseline and use-case/test-case/test-report lifecycle

1. `papyrus --help` exits 0 and lists `use-case`, `test-case`, `test-report`, `search`, and `browser` among the subcommands.
2. `papyrus --version` exits 0 and prints `papyrus <version>`.
3. `papyrus --dir <project> init <name> --no-git` creates `papyrus.config.yml` and the configured directory structure, including the three type directories and their `deprecated/` subfolders.
4. Use-case, test-case, and test-report lifecycle (BR-018/BR-019/BR-020, REQ-05.012/013/014, REQ-05.007):
   - `papyrus use-case create <title> --description <text>`, `papyrus test-case create <title> --description <text>`, and `papyrus test-report create <title> --description <text>` print `UC-001`, `TC-001`, and `TR-001` (or the next free ID), and the files are written as UTF-8 Markdown `<doc_directory>/<type-dir>/<ID>.md` with a `# <ID> <title>` heading, a `## Description` section, and a single trailing newline. Optional `--applicability <text>` adds a `## Applicability` section.
   - `use-case list` / `test-case list` / `test-report list` list active records of that type in ID ascending order as `<ID> <title>`; deprecated records are hidden.
   - `use-case view <id>` (and the test-case/test-report equivalents) print the full persisted text, including the heading and `## Description`; viewing a deprecated record by ID is allowed; an unknown ID exits 3.
   - `use-case edit <id> --title <new>` and `--description <new>` (and the test-case/test-report equivalents) update only the supplied field and preserve the other; `--applicability` replaces or clears the applicability section; the ID, file name, and folder are unchanged.
   - `use-case deprecate <id>` (and equivalents) moves the file into `deprecated/`; `list` no longer shows it; an already-deprecated record exits 4; an unknown ID exits 3.
   - `use-case restore <id>` (and equivalents) moves it back to the active folder with the same ID and unchanged content; an already-active record exits 4; an unknown ID exits 3.
   - IDs are preserved across deprecate/restore cycles and are never reused across the active and `deprecated/` folders.
   - Missing `<title>` or `--description` exits 2; sanitization failures (empty values, control characters, path separators) exit 4 and write nothing.
   - When `document_types.use-case.enabled`, `document_types.test-case.enabled`, or `document_types.test-report.enabled` is `false`, `create` and `list` report a validation error (exit 4) and no records are created or shown (BR-016).
   - `--editor` behavior remains covered by the CLI contract tests; it is environment-dependent and is not part of the isolated-install smoke run.

#### 3.2 Search (use cases, test cases, and test reports)

1. A quoted phrase present contiguously in a use-case, test-case, or test-report title or description is found: `papyrus search "<phrase>"` prints `<ID> doc <title>` lines in ascending ID order.
2. The same phrase split into separate words is not treated as a phrase match.
3. `--type doc` restricts results to document records and includes all three types; all three carry the `doc` type value. Other `--type` values (`task`, `milestone`, `release`, `decision`) do not return them.
4. `papyrus search <term> --limit 1` prints at most one result line; a limit larger than the match count returns all matches.
5. `papyrus --json search <term>` prints a JSON array of `{id, type, title, path}` objects; a search with no matches prints `[]` and exits 0.
6. Deprecated records of all three types are absent from search results; restored records reappear.
7. Usage errors (missing query, unknown `--type`, `--limit 0`) exit 2; an empty or control-character query exits 4. A `--type use-case`, `--type test-case`, or `--type test-report` value is not part of the documented vocabulary and exits 2.
8. A missing or empty type directory yields an empty result with exit 0 and creates nothing.

#### 3.3 Browser (Use Cases, Test Cases, and Test Reports categories)

1. `papyrus --dir <project> browser --no-open --port <free-port>` prints exactly one startup line `Serving http://127.0.0.1:<free-port>/` and serves until killed. `<free-port>` is obtained by binding `127.0.0.1:0`, reading the port, and dropping the listener, with a retry if the bind fails.
2. `--no-open` suppresses the platform opener.
3. `GET /?type=use-case`, `GET /?type=test-case`, and `GET /?type=test-report` render their categories. The sidebar lists Business requirements, Business Rules, Audit Records, Technical Specifications, Use Cases, Test Cases, Test Reports, Non-functional requirements, Backlog, Decisions, Glossary, UI Specifications, UI Mockups, Managed artifacts & lifecycles, and Agent skills & workflows, and the selected entry carries `aria-current="page"` with the non-colour cue. The default category stays `req`; any other `?type=` value, including `all`, is `400`.
4. The list renders one `h2` per category (`Use Cases`, `Test Cases`, `Test Reports`) with the record count and one row per active record in ascending ID order, each row linking `ID + title` to `/doc/{id}` with the first paragraph of the description as a preview and a metadata line (created date, updated date, labels, references). Empty and long-title states render; non-matching `.md` files, non-`.md` files, and `deprecated/` are never listed or counted.
5. `GET /doc/<ID>` renders the view: record ID (monospace), record form `Use case`/`Test case`/`Test report`, category `Use Cases`/`Test Cases`/`Test Reports`, the title as `h1`, and the description as body text with `white-space: pre-wrap` and no "Description" heading; an empty description shows `(none)`. The applicability, when present, renders as a section headed "Applicability" with `white-space: pre-wrap`. The breadcrumb is the category plural then the ID, and the page offers exactly one action, `Edit`; the raw file is never served or embedded.
6. `GET /doc/<ID>/edit` yields the form matching MOCK-0013: required title input, required multi-line description textarea, optional multi-line applicability textarea, front-matter metadata block (ID, created date, updated date read-only; labels and references tags), hidden `version` input plus the visible non-interactive annotation `Hidden field: version (optimistic-concurrency token)`, and `Save changes`/`Cancel`. The pristine, dirty, validation-error (422), conflict (409), and 320 CSS-pixel states are present, and the form works with JavaScript disabled.
7. Edit round-trip: `POST /doc/<ID>` with `version`, `title`, `description`, and optionally `applicability`, `labels`, and `references` returns `303` with `Location: /doc/<ID>`; the follow-up `GET` shows the new values, and `papyrus use-case view` / `papyrus test-case view` / `papyrus test-report view <ID>` shows the same persisted values.
8. Field-preserving write: a record seeded with a preamble and an editor-added unknown section is edited through the browser; a byte comparison of the pre-edit and saved files shows only the `# <ID> <title>` heading, the `## Description` body, the `## Applicability` body (when edited), and the front-matter `title`/`labels`/`references` keys changed. The preamble and every other section and line are content-identical; the file is never regenerated from a parsed record or the CLI serializer.
9. Validation: empty or over-long title, control characters, empty or over-long description, a description or applicability line starting with `## `, invalid labels, invalid references, and duplicate references return `422` with the documented messages, preserve the submitted values, and write nothing. A missing or malformed `version` returns `400`; a stale `version` returns `409` with the documented message and writes nothing.
10. Deprecated records of all three types are not listed or counted and return `404` on `GET /doc/{id}`, `GET /doc/{id}/edit`, `POST /doc/{id}`, `GET /api/doc/{id}`, and `POST /api/doc/{id}`. Every HTML `404` for an ID of one of the three types (deprecated or unknown) renders the identical category 404 page: status `404`, heading `Record not found`, help text "Deprecated <type> are not listed and cannot be opened. The same page is used for an unknown address.", and exactly one recovery link "Back to <type plural>" to `/?type=<category>`. JSON `404` responses keep the baseline JSON error object. No browser control changes deprecation state.
11. JSON API:
    - `GET /api/use-cases` returns `200 application/json` with an ascending array of `{"id":"UC-001","title":"..."}`; the alias `/api/use-case` returns the same response. `GET /api/test-cases` (alias `/api/test-case`) and `GET /api/test-reports` (alias `/api/test-report`) behave identically for their types.
    - `GET /api/doc/<ID>` returns the full record shape with `id`, `kind`/`form` (`use-case`/`test-case`/`test-report`), `title`, `description`, `applicability`, `labels`, `references`, `created_date`, `updated_date`, `version`, `path`, and `metadata` (the TASK-67 projection). `description` and `applicability` may be `null` when absent or empty; `labels` and `references` are arrays; `path` is project-relative with `/` separators.
    - `POST /api/doc/{id}` accepts JSON with `version` and optional `title`/`description`/`applicability`/`labels`/`references`, leaves omitted members unchanged, rejects a null or empty description, returns `400` for wrong types, and `422` for invalid values.
12. `GET /assets/app.css` returns `200 text/css` and `GET /assets/app.js` returns `200 text/javascript`, both with `Cache-Control: no-store`; the bodies are non-empty.
13. 320 px markup: every page carries `<meta name="viewport" content="width=device-width, initial-scale=1">`, the document-type sidebar toggle is present with `aria-expanded` and `aria-controls`, and the served CSS/JS use no forbidden features from the TASK-6 browser baseline.
14. No external references: served HTML, CSS, and JS contain no `http://`, `https://`, protocol-relative `//` remote reference, remote `@import`, remote `url(...)`, `<img>`, web font, or CDN reference; HTML responses carry the documented Content-Security-Policy.
15. A missing or empty type directory, and a configuration with `document_types.use-case.enabled`/`test-case.enabled`/`test-report.enabled` set to `false`, render the category empty state instead of failing (no server error), and the disabled sidebar entry is omitted.

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

1. A new file `tests/use_cases_test_cases_test_reports_smoke.rs` provides a milestone-level end-to-end test against the built binary (`CARGO_BIN_EXE_papyrus`), using temporary project directories under `./.tmp/`, a free loopback port, and the same startup-line wait and child-kill-on-drop pattern as `tests/browser_cli.rs`.
2. Test 1 — `milestone_use_cases_test_cases_test_reports_flow`: initialize a project; create `UC-001`, `TC-001`, and `TR-001`, with at least one record carrying a unique multi-word phrase in the description; assert the lifecycle (create/list/view/edit by title and by description without losing the other field/deprecate/restore with ID preservation and the documented exit codes) for all three types; exercise search (contiguous quoted phrase, split words no match, `--type doc`, other `--type` filters, `--limit 1`, `--json` shape and empty `[]`, deprecated hidden until restored); seed a preamble and an unknown section, then start `browser --no-open` on a free port and assert the startup line, `/?type=use-case`, `/?type=test-case`, and `/?type=test-report` with the sidebar `aria-current="page"` and position after Technical Specifications, the list count/rows/preview and deprecated exclusion, the record view (form, category, title, description body without a "Description" heading, applicability section, single `Edit`), the edit round-trip (`303` then updated view and matching CLI `view`), the field-preserving byte comparison of untouched sections, the JSON API (`/api/use-cases`, `/api/use-case`, `/api/test-cases`, `/api/test-case`, `/api/test-reports`, `/api/test-report`, `/api/doc/<ID>`), both assets, the 320 px markup hooks, the absence of external references, and the deprecated `404` on view, edit, POST, and API (including the identical 404 page text and recovery link).
3. Test 2 — `installed_layout_has_no_runtime_asset_dependency`: run the binary with its working directory set to an empty directory and the project directory containing no asset files; assert `search` (including a record of one of the three types) and `browser` still work and served assets are non-empty.
4. The new tests must not duplicate the TASK-78 contract coverage in the CLI tests, the TASK-80 coverage in `tests/search_cli.rs`, or the TASK-81 coverage in `tests/browser_cli.rs` (request limits, complete error-status matrix, XSS, symlink escape, conflict matrix, accessibility audit). Those remain authoritative there.
5. Tests require no internet access, bind only loopback, and leave no child process running.
6. `cargo test` passes with the new tests. Baseline at refinement: the m-15 test suite (unit + `cli` + `milestone_cli` + `milestone_smoke` + `req_cli` + `task_cli` + `decision_cli` + `glossary_cli` + `ui_spec_cli` + `ui_mockup_cli` + `business_rules_cli` + `audit_records_cli` + `technical_specifications_cli` + `search_cli` + `search_backlog_cli` + `browser_cli` + smoke files) all passing; the exact count is recorded during verification.

### 5. Documentation and evidence

1. `docs/packaging.md` states the supported release as Debian Forky only (amd64), per ADR-0003 and NFR-03.002, and explicitly states that Trixie and other releases are not claimed or verified.
2. `docs/packaging.md` contains the build/package commands, the isolated install and uninstall procedure (scratch directories under `./.tmp/`, removed afterwards), and the extended smoke-test checklist covering sections 3.1–3.6.
3. `docs/packaging.md` records observed results as a pass/fail table with the environment (Debian Forky version, glibc, `rustc`/`cargo` 1.99.1, `dpkg` version, isolation method, whether `unshare -rn` was available) and the date with timezone.
4. The TASK-82 ticket notes record the same evidence: environment, date/timezone, package file name and size, quality-gate outcomes with test counts, smoke-test outcomes, and any deviation or bug fix found.
5. The runtime dependency-license audit from TASK-1/TASK-4 is referenced as unchanged; no re-run is required because no runtime dependency was added.

## Inputs and outputs

| Input / output | Description |
|---|---|
| Source tree | Post-TASK-81 state: use-case/test-case/test-report management (TASK-78), search (TASK-80), browser (TASK-81), design review (TASK-79, MOCK-0013 approved), plus all earlier features. |
| Toolchain | Rust 1.99.1 via `rust-toolchain.toml`; `dpkg-deb`, `fakeroot`; Debian Forky amd64. |
| `scripts/build-deb.sh` | Unchanged packaging script; reads the version from `Cargo.toml`. |
| `papyrus_<version>_amd64.deb` | Built package in the project root. |
| Isolated root/admin | `./.tmp/` directories for `dpkg --instdir/--admindir`; removed after the run. |
| `tests/use_cases_test_cases_test_reports_smoke.rs` | New milestone-level integration test. |
| `docs/packaging.md` | Updated procedures, checklist, and observed results. |
| Ticket notes | Verification evidence for TASK-82. |

## Validation and error behavior

| Condition | Consequence |
|---|---|
| `cargo build --release`, `cargo test`, clippy, or `fmt --check` fails | Blocked; fix before packaging. |
| `target/release/papyrus --help` lacks `use-case`/`test-case`/`test-report`/`search`/`browser` (stale binary packaged) | Blocked; rebuild before `build-deb.sh`. |
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
| T-01 | Pinned 1.99.1 release build succeeds and exposes use-case/test-case/test-report/search/browser | `cargo build --release`; `--help` subcommand list |
| T-02 | Quality gates pass | `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`; recorded test counts |
| T-03 | `.deb` builds with the Cargo version and amd64 architecture | `./scripts/build-deb.sh`; `dpkg-deb -I` |
| T-04 | Metadata: `Package: papyrus`, `Version`, `Architecture: amd64`, `Depends: libc6 (>= 2.34)`, GPL-3.0-or-later copyright | `dpkg-deb -I`; copyright file inspection |
| T-05 | Package contains only the binary and copyright; assets are embedded | `dpkg-deb -c`; asset requests from the installed binary |
| T-06 | Installs and uninstalls in an isolated Debian Forky environment, host untouched | `fakeroot dpkg --instdir/--admindir -i`, `-r`, `--purge`; host `dpkg -l` before/after |
| T-07 | Installed binary is on `PATH` at `/usr/bin/papyrus` and runs | `PATH`-prefixed `papyrus --version` |
| T-08 | Baseline CLI and use-case/test-case/test-report lifecycle | Installed-binary smoke commands and exit codes |
| T-09 | Search over use cases, test cases, and test reports | Installed-binary search commands |
| T-10 | Browser: startup with `--no-open`, three categories, list, view, edit round-trip, field-preserving write | Installed-binary browser commands plus HTTP requests |
| T-11 | Browser: JSON API and embedded assets served from the binary | `GET /api/*`, `GET /assets/*` |
| T-12 | 320 px markup and no external references | Markup/asset assertions on served content |
| T-13 | Deprecated records hidden from list, search, and browser; `404` on view, edit, POST, and API | Deprecate/restore fixtures plus HTTP assertions |
| T-14 | Offline operation with no internet access | `unshare -rn` run with a failing outbound connection, then search + browser smoke |
| T-15 | Browser configuration keys work in the installed binary | `config list/get/set/unset` and startup-line assertions |
| T-16 | Milestone-level integration test added without duplicating lifecycle/search/browser contract tests | `tests/use_cases_test_cases_test_reports_smoke.rs`; `cargo test` |
| T-17 | Documentation states Forky only and records procedures and results with date/timezone | `docs/packaging.md` review |
| T-18 | Evidence recorded in the ticket | TASK-82 notes review |

## Assumptions

- Assumption: the verification host is Debian Forky amd64. At refinement the build host reports `Debian GNU/Linux forky/sid`, with `fakeroot`, `dpkg-deb`, `ip`, and `unshare -rn` available, and `rustc`/`cargo` 1.99.1 pinned by `rust-toolchain.toml`. The isolated `dpkg --instdir/--admindir` install therefore runs against Forky's glibc and dpkg, satisfying the NFR-03.002 verification method without a container.
- Assumption: isolation via `fakeroot dpkg --instdir/--admindir` is acceptable; a Forky container, chroot, or VM may substitute with the same checks.
- Assumption: scratch directories live under `./.tmp/` and are removed before the task ends (project convention forbids system temporary directories).
- Assumption: the package version stays `0.1.0`; no version bump is defined for m-15 and release versioning belongs to release orchestration.
- Assumption: `scripts/build-deb.sh` is used unchanged and does not force a rebuild; `cargo build --release` must run first. The repository may contain a stale `target/release/papyrus` and `papyrus_0.1.0_amd64.deb` built before TASK-81, so packaging without a rebuild would ship a binary lacking browser support for the three types. AC #2 guards this.
- Assumption: `use-case`, `test-case`, `test-report`, `search`, and `browser` require no new runtime dependency; the TASK-1/TASK-4 dependency-license audit remains valid (NFR-06.002).
- Assumption: no UI change ships in this ticket, so MOCK-0001…MOCK-0013 remain the approved layout references and no new mockup is produced. MOCK-0013 is `approved` in the mockup registry (user approval recorded 2026-11-11 UTC, TASK-79).
- Assumption: NFR-05.001 browser-version support was verified at markup/feature-subset level by TASK-6/TASK-81 and is not re-verified in a real GUI browser here.
- Assumption: the new integration test runs the built binary, not the installed package; the installed-package checks are the manual/scripted checklist in sections 2–3, recorded in `docs/packaging.md`.
- Assumption: the offline check is environment-dependent; `unshare -rn` is available at refinement, with the documented fallback if not.
- Assumption: use cases, test cases, and test reports support `restore` after `deprecate` (BR-018/BR-019/BR-020); smoke tests assert restoration and ID preservation.
- Assumption: `--description` and `--applicability` are single-line sanitized CLI inputs and `--editor` is the CLI route for multi-line bodies; the browser otherwise allows multi-line values. `--editor` is not exercised in the isolated install because it depends on the environment; the CLI contract tests remain its coverage.
- Assumption: the browser requires both title and description; an on-disk record with an absent or empty description is viewable as `(none)` but must be completed before it can be saved from the browser.
- Assumption: deprecated records of all three types are excluded from list, search, and browser per REQ-05.007/BR-018/BR-019/BR-020; the smoke test asserts exclusion at all three surfaces.
- Assumption: the dedicated packaging specification authored with this refinement (`docs/specifications/use-cases-and-test-documentation-milestone-packaging-and-verification.md`) is the canonical home for this ticket.

## Open questions

None of these blocks implementation; each has a specified default. OQ-1 is recommended for user confirmation before release.

- **OQ-1 (recommended default: markup/asset-level verification only)** — NFR-05.001's verification method asks for checks in Firefox/Edge/Safari 100+. This headless packaging task verifies the served markup, asset set, viewport hooks, and absence of forbidden features, but does not drive a real GUI browser. Default: record the limitation in `docs/packaging.md` and rely on TASK-6/TASK-81 markup-level verification plus the approved MOCK-0001…MOCK-0013; schedule a manual browser pass outside this ticket if the user requires one. Non-blocking.
- **OQ-2 (recommended default: fakeroot `dpkg --instdir/--admindir` on the Forky build host)** — Whether a full Forky container/VM is mandatory. Default: use the fakeroot isolated install, which exercises Forky's glibc and dpkg and never touches the host; if the user requires container-level isolation, the same checklist applies unchanged. Non-blocking.
- **OQ-3 (recommended default: a new `tests/use_cases_test_cases_test_reports_smoke.rs`)** — Whether the milestone test belongs in an existing smoke file or a new file. Default: a new file keeps each milestone's smoke coverage separate and avoids growing an already consolidated file. Non-blocking.
- **OQ-4 (recommended default: keep version 0.1.0)** — No m-15 release version is defined in the requirements. Default: package the current Cargo version unchanged; a bump belongs to release orchestration. Non-blocking.
- **OQ-5 (recommended default: re-verify the lifecycle in this milestone package)** — TASK-78 verified the use-case/test-case/test-report management lifecycle at the CLI level. Default: repeat the lifecycle smoke here against the installed m-15 package because this ticket must prove the packaged binary provides the complete increment, and the repeat is small. The contract detail stays in the CLI contract tests. Non-blocking.
- **OQ-6 (open item, requirements wording)** — REQ-05.012/013/014 state that use cases, test cases, and test reports "are managed with the `req` command", which conflicts with BR-009 (which defines `req` as business-requirements only) and with BR-018/BR-019/BR-020 (which define the dedicated `use-case`, `test-case`, and `test-report` commands). This packaging task verifies the dedicated commands only; it does not edit requirements. The requirements owner may correct the wording separately (see TASK-78 OQ-4). Non-blocking for packaging.

## Traceability

| Artifact | Reference |
|---|---|
| Requirement | REQ-05.007, REQ-05.010, REQ-05.012, REQ-05.013, REQ-05.014, REQ-06.001, REQ-06.002, REQ-06.004 |
| Non-functional | NFR-01.002, NFR-01.003, NFR-02.002, NFR-03.002, NFR-04.001, NFR-04.002, NFR-05.001, NFR-06.001, NFR-06.002 |
| Business rule | BR-004 (browser command behavior), BR-015 (search command behavior), BR-016 (configuration keys and precedence), BR-018 (use-case document type behavior), BR-019 (test-case document type behavior), BR-020 (test-report document type behavior) |
| ADR | ADR-0001 (Rust), ADR-0003 (Debian Forky only; supersedes ADR-0002), ADR-0004 (repository-root single crate), ADR-0005 (embedded HTTP server and server-rendered UI), ADR-0006 (optimistic concurrency and atomic writes), ADR-0007 (YAML front matter), ADR-0009 (front-matter options) |
| Ticket | TASK-82 |
| UI mockups | none new; MOCK-0001…MOCK-0012 are the approved baseline; MOCK-0013 (approved, TASK-79) covers the Use Cases, Test Cases, and Test Reports categories |
| Related specification | `docs/specifications/use-cases-test-cases-and-test-reports-management.md` (TASK-78) |
| Related specification | `docs/specifications/search-use-cases-test-cases-and-test-reports.md` (TASK-80) |
| Related specification | `docs/specifications/browser-use-cases-test-cases-and-test-reports.md` (TASK-81) |
| Related specification | `docs/specifications/ui/browser-use-cases-test-cases-and-test-reports-ui.md` (TASK-79) |
| Related specification | `docs/specifications/technical-specifications-milestone-packaging-and-verification.md` (TASK-59 pattern) |
| Related specification | `docs/specifications/complete-content-management-milestone-packaging-and-verification.md` (TASK-20 pattern) |
| Related specification | `docs/specifications/cli-milestone-packaging-and-verification.md` (TASK-62 m-4 precedent) |
| Packaging documentation | `docs/packaging.md` |
| Domain model | `docs/domain-model/business-glossary/` (use case, test case, test report terms if present) |
path: docs/specifications/use-cases-and-test-documentation-milestone-packaging-and-verification.md