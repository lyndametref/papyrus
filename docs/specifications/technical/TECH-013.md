---
id: TECH-013
title: "Backlog Discovery Milestone Packaging and Verification"
created_date: '2026-11-16T17:46:44Z'
updated_date: '2026-11-21T16:27:08Z'
labels: []
references:
  - ADR-0001
  - ADR-0002
  - ADR-0003
  - ADR-0004
  - ADR-0005
  - ADR-0006
  - ADR-0036
  - ADR-0037
  - ADR-0038
  - ADR-0039
  - BR-004
  - BR-005
  - BR-006
  - BR-007
  - BR-015
  - BR-016
  - NFR-01.002
  - NFR-01.003
  - NFR-02.002
  - NFR-03.002
  - NFR-04.001
  - NFR-04.002
  - NFR-05.001
  - NFR-06.001
  - NFR-06.002
  - NFR-007.001
  - NFR-007.002
  - NFR-007.003
  - REQ-01.003
  - REQ-02
  - REQ-02.001
  - REQ-02.002
  - REQ-02.003
  - REQ-02.004
  - REQ-03
  - REQ-03.001
  - REQ-03.002
  - REQ-05.001
  - REQ-05.002
  - REQ-05.007
  - REQ-06
  - REQ-06.001
  - REQ-06.002
  - REQ-06.014
  - REQ-06.019
  - REQ-07.001
  - REQ-07.002
  - REQ-07.003
  - REQ-07.004
status: verified
---
# TECH-013 Backlog Discovery Milestone Packaging and Verification

## Purpose

Verify the backlog discovery capability as one milestone verification of the descriptor-driven refactor, and record the evidence. The verification covers the installed binary end to end: the bespoke backlog records (task, milestone, release), `search` over backlog records, and the offline local browser over the backlog categories.

This is a verification step. No product behaviour is added and no product code is changed here. The backlog and decision surfaces stay bespoke; the document surface is descriptor-driven. Both are exercised in the same run.

Every milestone verification builds a throwaway verification binary and a throwaway verification package. The package is installed in an isolated root, the installed binary is exercised, and both are deleted. Only the single release boundary produces a retained release package; no milestone verification publishes, promotes, or keeps one, and no intermediate release or release candidate exists.

## Scope

### In scope

- Build `target/release/papyrus` and one verification package from the current source with the pinned Rust 1.99.1 toolchain, using the packaging script unchanged to produce the package in the project root, then moving the package under `./.tmp/`.
- Run the verification gates of section 2 and the installed-binary checks of sections 3 and 4.
- Verify package metadata: `Package: papyrus`, `Architecture: amd64`, `Version` equal to the Cargo package version, `Depends: libc6 (>= 2.34)`, GPL-3.0-or-later copyright, and the documented asset Version binding.
- Verify the package inventory contains only the binary and the copyright file. No source file, no per-type document module, no runtime asset file, no template, no configuration file, and no man page is packaged; browser assets are embedded through the static asset table and document types are seed descriptors.
- Install and uninstall the verification package in an isolated Debian Forky environment without modifying the host, then delete the isolated roots and the package.
- Smoke-test the installed binary:
  - CLI baseline: `--help`, `--version`, `init`, and the task, milestone, and release lifecycles.
  - Search over task, milestone, and release records: quoted phrase, `--type`, `--limit`, `--json`, archived visibility.
  - Browser over the backlog categories: startup with `--no-open` on a free port, all categories (`req`, `nfr`, `task`, `milestone`, `release`), record view, edit round-trip, conflict and not-found pages, JSON API, embedded assets and their cache classes, the shared shell and list layout, and the CSP response classes.
- Verify offline operation: search and browser work with no internet access, using only loopback and locally served assets.
- Verify the browser configuration keys (`browser_host`, `browser_port`, `browser_open`) behave as documented in the installed binary.
- Add one milestone-level integration test under `tests/` covering task, milestone, release, search, and the browser against the built binary, without duplicating the existing contract coverage in `tests/browser_cli.rs` or `tests/search_backlog_cli.rs`.
- Record the verification evidence in `docs/packaging.md`, including the gate results, the measurements, the findings, and the statement that the verification build was discarded.

### Out of scope

- New product behaviour, commands, options, record types, or UI changes.
- A retained release package, a version bump, a tag, a release candidate, or release sign-off. The release boundary owns the one retained package and the release evidence (TECH-069 section 6).
- Verification on a Debian release other than Forky, and support claims beyond Forky.
- Real-GUI rendering checks in Firefox, Edge, or Safari; verification is HTTP, markup, asset, and console level.
- Re-running the runtime dependency-license audit; no runtime dependency is added.
- Changes to requirements, business rules, ADRs, packaging scripts, or packaging metadata.
- New UI mockups; the shipped browser interface is unchanged.
- Fixing a defect found during verification. The defect is recorded with expected and observed values and a disposition.

## Behavior

### 1. Build and verification package

1. `cargo build --release` succeeds with the pinned Rust 1.99.1 toolchain (`rust-toolchain.toml`) and produces `target/release/papyrus`.
2. `target/release/papyrus --help` lists `task`, `milestone`, `release`, `search`, `browser`, every enabled document slug, and the built-in non-document commands. A stale binary that lacks a required command is never packaged; the release build runs first.
3. `./scripts/build-deb.sh` runs after the release build and produces the verification package `papyrus_<version>_amd64.deb` in the project root, where `<version>` is read from `Cargo.toml`. The verification moves the package into a staging directory under `./.tmp/`; the staging directory and the package are deleted when the verification ends, including on failure.
4. The verification package is never a release artifact. It is not published, tagged, version-bumped, promoted, or copied outside `./.tmp/`; no release candidate is produced from it. Only the release boundary builds the one retained package, once, with its size and SHA-256 recorded.
5. `dpkg-deb -I papyrus_<version>_amd64.deb` reports `Package: papyrus`, `Version: <Cargo version>`, `Architecture: amd64`, `Depends: libc6 (>= 2.34)`, `Section: utils`, `Priority: optional`, a `Maintainer` value, a `Homepage` value, and a `Description` naming Papyrus as a project-level backlog and documentation manager.
6. The packaged copyright file (`/usr/share/doc/papyrus/copyright`) is in Debian copyright format and declares `GPL-3.0-or-later`, matching the `license` field in `Cargo.toml`, with one notice section per bundled asset family.
7. Package inventory: `dpkg-deb -c` lists exactly `/usr/bin/papyrus` (0755) and `/usr/share/doc/papyrus/copyright` (0644), plus their directories. It lists no `.rs` source file and no per-type document module, no `.css`, `.js`, or `.html` file, no template, no configuration file, and no man page.
8. Asset Version packaging check: the packaging step enumerates the version directories of `src/browser/vendor/<package>/<version>/` and reads the `version` column of `src/browser/vendor/MANIFEST.md`. It fails the build when a package has no version directory, more than one, a manifest row with a different version, or a copyright section citing a different version. The asset Version is unrelated to the Papyrus package version and appears in the versioned asset URL.
9. Binary size gate: `stat -c %s target/release/papyrus` is at most 12,582,912 bytes (12 MiB). The measured size is recorded. The binary embeds the whole asset table and requires no runtime asset directory.
10. `target/release/papyrus --version` and the installed binary's `--version` both print `papyrus <version>`, equal to the package Version.
11. No verification package survives the step: the staging directory, the isolated dpkg root, the isolated admin directory, and the fixture projects are removed before the evidence is finalised.

### 2. Verification gates

Every milestone verification runs the same gates on the pinned toolchain. A failing gate blocks the verification; the verification build is never retained or handed forward as a release.

| Gate | Pass condition |
|---|---|
| Full suite | `cargo test` passes, including the no-drift suites, the document model matrix, the backlog suites, and the equivalence replay. Unit and integration counts are recorded as evidence; a count is not itself a gate. |
| Lints and format | `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` are clean. |
| No drift | The no-drift suites pass: no literal `deprecated/` in `src/`, no `nfreq` command, no document `deprecate` or `restore` variant, no static per-type document command list, no per-type document module, no document entry in the static presentation table, and no process-global descriptor map. |
| No new crates | `Cargo.toml` and `Cargo.lock` add no crate and no dependency edge beyond the existing set. Dev-dependencies and build-dependencies are included: any new crate or dependency edge in either file fails the gate. |
| Zero document `TypedHandler` variants | No document browser path selects behaviour through a `TypedHandler` variant. Backlog and decision handlers keep their bespoke variants; a source scan over the document path finds none. |
| No document slug dispatch outside the seed | No `match` or `if` on a document slug literal selects behaviour anywhere in `src/`, except the `SEED_DOCUMENT_TYPES` table in `src/config.rs`. The seed table is the only place that names document slugs; tests and documentation are exempt. |
| Strict record, JSON, and status-code equivalence | Against the captured fixture corpus: record-file bytes are identical, JSON payloads are identical including documented key order, and every HTTP status code is identical. |
| Semantic HTML equivalence | Rendered document HTML is equivalent by selector, text, navigation, form field, and the absence of per-type wrappers. The backlog and decision surfaces stay byte-compatible with the captured output. Byte equality is not used for document pages. |
| NFR-007.001 to NFR-007.003 measurements | The inventory p95 is under 250 ms and the view and edit p95 under 100 ms at 1,000 records on the reference hardware; the asset cache-class headers and the repeat-visit transfer match the caching contract. |
| Binary size | The built binary is at most 12 MiB (12,582,912 bytes). |
| Production LOC | Reported at every verification as an informational measurement against the 28,000-line internal target, and reported at the release boundary. It is never a gate, acceptance criterion, or blocker: a size above it is a recorded observation, not a failed gate. |

1. The fixture corpus covers one type per `DocumentModel`, a grouped type with several members, an HTML mockup in each fixed metadata state, a user-defined `SingleMarkdown` type, records with resolved and unresolved references, a record at the archived boundary, and a record at a zero-padding ID boundary. Backlog fixtures cover a release, a milestone assigned to it, a task assigned to the milestone, an archived task, and an archived milestone.
2. The equivalence comparison is strict only where the contract is byte-exact: record files, JSON, and status codes. Rendered document HTML is compared semantically; backlog and decision pages are compared byte-for-byte with the captured output.
3. Expected deltas are recorded as expected deltas, never as equivalence failures: the uniform title command shape, the removal of the term option, rewritten help prose, the asset cache headers, and latency changes.
4. A behaviour change that is not on the expected-delta list is a gate failure until it is recorded with a disposition and the owning surface is named.
5. Measurement method for NFR-007.001 and NFR-007.002: the documented benchmark harness, the 1,000-record corpus, 10 warm-up requests, then 100 measured sequential requests per route, p50 and p95 as observed by the server, excluding the first cold-start request and any request that triggers a configuration reload. The harness command, corpus, hardware, and p50 and p95 values are recorded.
6. Measurement method for NFR-007.003: the cache-class header matrix on the installed binary plus the repeat-visit transfer measurement; the second load of a page with a diagram and a page with math transfers no vendor bytes.
7. The binary size gate and the production LOC figure are measured on the release build. A size above 12 MiB blocks the verification. The production LOC is reported at every verification as information and at the release boundary; a production size above 28,000 lines is recorded and reported, does not fail a gate, and is not an acceptance criterion.

### 3. Isolated install and uninstall

1. Verification runs on Debian Forky amd64 using an isolated dpkg database and install root, so the host's package database and filesystem are never modified: `fakeroot dpkg --instdir=<root> --admindir=<adm> --force-depends -i <staging>/papyrus_<version>_amd64.deb`. A container, chroot, or virtual machine on Forky may be used instead with the same checks.
2. The isolated root and admin directories are created under `./.tmp/` and are removed after the run, including on failure.
3. Before and after the run, the host reports papyrus as not installed (`dpkg -l papyrus` has no installed entry).
4. `dpkg -i` in the isolated root succeeds with exit 0.
5. After install, `<root>/usr/bin/papyrus` exists with mode 0755 and owner `root:root`.
6. With `<root>/usr/bin` first on `PATH`, `papyrus --version` resolves to the installed binary and prints `papyrus <version>`.
7. `dpkg -r papyrus` in the isolated root removes the binary; `papyrus` no longer resolves on that `PATH`.
8. `dpkg --purge papyrus` in the isolated root removes the remaining files and the admin entry; the root contains no papyrus files afterwards.

### 4. Installed-binary smoke tests

All commands run against the installed binary with `--dir <project>` pointing at a scratch project under `./.tmp/`. Exit statuses follow the existing contract: 0 success, 1 IO or boundary, 2 usage, 3 not found, 4 validation.

#### 4.1 CLI baseline and backlog lifecycles

1. `papyrus --help` exits 0 and lists `task`, `milestone`, `release`, `search`, and `browser`.
2. `papyrus --version` exits 0 and prints `papyrus <version>`.
3. `papyrus --dir <project> init <name> --no-git` creates `papyrus.config.yml` and the configured directory structure.
4. Release lifecycle: `release create <title>` prints a generated release ID; `list`, `view`, and `edit` succeed, and the edited title persists.
5. Milestone lifecycle: `milestone create <title> --release <release-id>` prints a generated milestone ID; `list`, `view`, and `edit` (including assignment and removal) succeed; IDs are preserved.
6. Task lifecycle: `task create <title> --milestone <milestone-id> --ac <criterion>` prints a generated task ID; `list`, `view`, and `edit --status` succeed; `archive` then `restore` preserve the ID, and `list` reflects both states.
7. Filtering and linking work from the installed binary: `task list --milestone <milestone-id>`, `task list --release <release-id>`, the milestone view shows its release assignment, and the task view shows its milestone assignment.

#### 4.2 Search over backlog records

1. A quoted phrase present contiguously in a task, milestone, or release body is found: `papyrus search "<phrase>"` prints `<id> <type> <title>` lines in ascending ID order, including the `task`, `milestone`, and `release` types.
2. The same words separated in the source are not a phrase match.
3. `--type task`, `--type milestone`, and `--type release` each restrict results to that type; `--type doc` returns document records only.
4. `papyrus search <term> --limit 1` prints at most one result line; a limit above the match count returns every match.
5. `papyrus --json search <term>` prints a JSON array of `{id, type, title, path}` objects; no matches prints `[]` and exits 0.
6. An archived task or milestone is absent from search results and reappears after `restore`; a release has no archived state and is always searched.
7. Usage errors (missing query, unknown `--type`, `--limit 0`) exit 2; an empty or control-character query exits 4; a symlinked record directory escaping the project exits 1.

#### 4.3 Browser over the backlog categories

1. `papyrus --dir <project> browser --no-open --port <free-port>` prints exactly one startup line `Serving http://127.0.0.1:<free-port>/` and serves until stopped. The port is obtained by binding `127.0.0.1:0`, reading the port, and dropping the listener, with a retry if the bind fails.
2. `--no-open` suppresses the platform opener; the process still serves.
3. `GET /` and `GET /?type=req` render the business requirements category; `GET /?type=nfr` renders the non-functional category; `GET /?type=task`, `?type=milestone`, and `?type=release` each render exactly their own category. The selected navigation entry carries `aria-current="page"`, and the navigation groups are the three categories.
4. `GET /doc/<id>` renders the record view for a task, a milestone, and a release, including ID, title, the backlog-specific fields, and the reference disclosure. For a document record, the view is built from the descriptor field map.
5. Edit round-trip: `GET /doc/<id>/edit` yields the form with the hidden Version token; `POST /doc/<id>` with the documented fields returns `303` with `Location: /doc/<id>`; the follow-up `GET` shows the new value, and the matching CLI `view` shows the same persisted file.
6. `POST` with a stale Version token returns `409` and writes nothing; an undeclared key returns `422` and writes nothing; an unknown, archived, or disabled ID returns the one not-found page with `404`.
7. `GET /api/docs/<slug>` for a document category, the documented backlog JSON index routes, and `GET /api/doc/<id>` return the documented payloads.
8. Asset cache classes: `GET /assets/app.css` and `GET /assets/app.js` return `200` with the documented MIME type, `Cache-Control: no-cache`, and one strong `ETag` of the form `"<64 lowercase hex characters>"`; a `GET` or `HEAD` whose `If-None-Match` is that exact tag returns a bodyless `304` carrying the same `ETag` and `Cache-Control`, no `Content-Type`, no `Content-Length`, and no CSP; a stale, weak, or unparseable tag returns the full `200`. `/assets/nav-state.js` follows the same class.
9. Asset cache classes for vendored files: a versioned path such as `/assets/katex/0.16.11/katex.min.css`, a KaTeX font under `/assets/katex/0.16.11/fonts/`, and `/assets/mermaid/11.17.2/mermaid.min.js` return `200` with the documented MIME type and `Cache-Control: public, max-age=31536000, immutable`, no `ETag`, no `Last-Modified`, and no `Vary`; a conditional request returns the full `200`, never `304`; the unversioned vendor paths return `404` for `GET`, `HEAD`, and `POST`.
10. Every HTML and JSON response carries `Cache-Control: no-store`, no validator, and never `304`. No response of any class carries `Content-Encoding` or `Vary`.
11. Pages reference versioned vendor URLs: a math page contains `/assets/katex/0.16.11/katex.min.css` and `/assets/katex/0.16.11/katex.min.js`; a diagram page contains `/assets/mermaid/11.17.2/mermaid.min.js`; no served page references an unversioned vendor path.
12. Served HTML, CSS, and JS contain no `http://`, `https://`, protocol-relative `//`, remote `@import`, remote `url(...)`, `<img>`, or CDN reference. Every `url(` in every served stylesheet resolves to an entry of the asset table, and KaTeX's font references resolve under the versioned directory.
13. Every `text/html` response carries exactly one of the two fixed CSP strings: the strict string when the pre-pass did not set `has_diagram`, and the diagram string only when it did. `/assets/` and non-HTML responses carry no CSP, and `script-src` never gains `'unsafe-inline'`, `'unsafe-eval'`, a nonce, or a hash.
14. 320 px markup: every page carries the viewport meta, the navigation toggle with `aria-expanded` and `aria-controls`, and no page-level horizontal scrolling at 320, 480, and 720 CSS px.
15. Asset manifest verification: the manifest test asserts that the manifest row set equals the asset table paths with no duplicate or missing row and recomputes SHA-256 and byte length for every entry; the packaged copyright shows the licence block plus one section per bundled family; `dpkg-deb -c` still lists exactly the binary and the copyright file.
16. An empty backlog category renders its empty state instead of failing; an archived task or milestone is not listed and is unreachable (`404`); a disabled document type has no navigation entry, list, detail, or API route.

#### 4.4 Offline operation

1. The smoke run is repeated inside an isolated network namespace (`unshare -rn`, loopback brought up with `ip link set lo up`).
2. Inside the namespace, an outbound TCP connection to a public address fails, proving the namespace is disconnected.
3. With no network, `search` and the browser checks of sections 4.2 and 4.3 pass over loopback; the browser binds the IP literal `127.0.0.1`.
4. If `unshare -rn` is unavailable, the offline claim rests on the loopback-only bind, the absence of external references in served content, and the absence of an outbound-connection code path; the limitation is recorded.

#### 4.5 Browser configuration keys

1. `config list` includes `browser_host`, `browser_port`, and `browser_open` with effective values.
2. `config get` prints the effective value for each key.
3. A configured port is used when `browser` starts without `--port`; `--port` still overrides it.
4. A configured `browser_open: false` suppresses opening without `--no-open`; removing the key restores the default.
5. An invalid port or an invalid boolean exits 4 and leaves the configuration file unchanged.

#### 4.6 Embedded assets

1. The installed binary serves `/assets/app.css` and `/assets/app.js` from a project directory that contains no asset files and from a working directory that contains no source tree.
2. The package contains no asset, template, or documentation file beyond the copyright file.
3. The installed binary needs only `/usr/bin/papyrus` and the project directory.

### 5. Automated integration test

1. A milestone-level test file under `tests/` drives the built binary (`CARGO_BIN_EXE_papyrus`) with temporary project directories, a free loopback port, and the same startup-line wait and child-kill-on-drop pattern as `tests/browser_cli.rs`.
2. The test initialises a project, creates one release, one milestone assigned to it, and one task assigned to the milestone with a unique multi-word phrase; asserts the three lifecycles, the quoted-phrase search across the three types, `--type task|milestone|release`, `--limit 1`, the `--json` shape, and archived visibility; then starts `browser --no-open` on a free port and asserts the startup line, the five category pages, a record view for each backlog type, the edit round-trip (303 then the updated view and the matching CLI view), the JSON API, the asset cache classes, the 320 px hooks, and the absence of external references.
3. A second test runs the binary with its working directory set to an empty directory and a project directory containing no asset files, and asserts that search and the browser still work with non-empty served assets.
4. The tests do not duplicate the protocol coverage in `tests/browser_cli.rs` or the backlog search coverage in `tests/search_backlog_cli.rs`: request limits, the error-status matrix, the concurrency cap, XSS, symlink escape, Version conflict, and the accessibility audit remain authoritative there.
5. The tests need no internet access, bind only loopback, and leave no child process running. `cargo test` passes with the new tests.
6. The tests assert the strict equivalence expectations they can observe: persisted bytes through the CLI, JSON shape, and HTTP status codes. Rendered HTML is asserted semantically.

### 6. Documentation and evidence

1. `docs/packaging.md` states the supported release as Debian Forky only (amd64) and states that other releases are not claimed or verified.
2. `docs/packaging.md` contains the build and package commands, the isolated install and uninstall procedure, and the smoke-test checklist covering sections 4.1 to 4.6, with the staging paths under `./.tmp/`.
3. The verification record states the date with timezone, the environment (distribution, glibc, Rust and Cargo, clippy, `dpkg`, `fakeroot`, `unshare`), the isolation method, the gate table, the test counts, the verification package name and size, the binary size, the production LOC as an informational measurement, and the NFR-007.001 to NFR-007.003 measurements with p50 and p95.
4. The record states that the verification build was discarded, names any expected delta, and lists every finding with its expected and observed values and a disposition.
5. The record states release readiness for this increment only as an input to the release boundary; it does not claim a release.
6. The runtime dependency-license audit is referenced as unchanged; no runtime dependency was added.
7. A defect in product code, a stale documentation statement, or a verification limitation is recorded with a disposition and attributed to its owning surface; this step fixes none of them.

## Inputs and outputs

| Input or output | Description |
|---|---|
| Source tree | The bespoke backlog records, the descriptor-driven document engine, search, and the browser. |
| Toolchain | Rust 1.99.1 via `rust-toolchain.toml`; `dpkg-deb`, `dpkg`, `fakeroot`, `unshare` and `ip`; Debian Forky amd64. |
| Packaging script | `scripts/build-deb.sh`, used unchanged for the milestone verification and the single release boundary; reads the version from `Cargo.toml`. |
| Verification package | `papyrus_<version>_amd64.deb` produced in the project root and moved under `./.tmp/`; installed, exercised, and deleted. |
| Isolated root and admin | `./.tmp/` directories for `dpkg --instdir` and `--admindir`; removed after the run. |
| Fixture corpus and harness | The captured equivalence fixtures and the benchmark harness, reused unchanged. |
| Integration test | The milestone-level backlog test under `tests/`. |
| Verification record | The gate table, measurements, findings, and discarded-build statement in `docs/packaging.md`. |

## Validation and error behavior

| Condition | Consequence |
|---|---|
| `cargo build --release`, the full suite, clippy, or `fmt --check` fails | Blocked; fix before packaging. |
| A verification gate of section 2 fails | Blocked; record the gate, command, and output. |
| A new crate or dependency appears in `Cargo.toml` or `Cargo.lock` | Blocked; the no-new-crates gate fails. |
| A document `TypedHandler` variant or document slug dispatch outside the seed exists | Blocked; record the file and line. |
| A record, JSON, or status-code comparison differs | Blocked; capture the fixture, expected value, and observed value. |
| A document HTML comparison differs semantically, or a backlog page differs byte-wise | Blocked; record the diverging selector, text, or field. |
| The binary exceeds 12 MiB (12,582,912 bytes) | Blocked; the binary size gate fails. |
| Production LOC exceeds 28,000 lines | Recorded and reported as an informational measurement; not a gate failure and not an acceptance criterion. |
| The inventory or view and edit p95 exceeds its target | Gate fails; record the measurement and escalate for a decision. |
| The asset cache classes or the repeat-visit transfer differ | Gate fails; record the path and the observed headers or bytes. |
| `target/release/papyrus --help` lacks a required command (stale binary packaged) | Blocked; rebuild before packaging. |
| `dpkg-deb -I` metadata mismatch, including the asset Version binding | Blocked. |
| The package inventory lists a source file, a per-type document module, an asset, a template, a configuration file, or a man page | Blocked. |
| `dpkg -i` fails in the isolated root, or the host state changes | Blocked; isolation is mandatory. |
| The installed binary is missing from `PATH` after install, or remains after purge | Blocked. |
| Any smoke check fails | Blocked; record the failing command and output. |
| Outbound traffic succeeds inside the offline namespace | Offline claim not verified; investigate and record the limitation. |
| A verification package or an intermediate release candidate is retained | Blocked; a milestone verification keeps no package. |
| A defect or a requirement, rule, or ADR conflict is found | Record with a disposition; do not edit requirements, rules, ADRs, or product code here. |

## Verification matrix

| Check | Verifies | How it is verified |
|---|---|---|
| T-01 | Release build succeeds and exposes the backlog commands, search, and browser | `cargo build --release`; `--help` output |
| T-02 | Every verification gate of section 2 passes | Gate table output recorded in the verification record |
| T-03 | Verification package builds with the Cargo version and amd64 architecture | `./scripts/build-deb.sh`; move the package into `./.tmp/`; `dpkg-deb -I` |
| T-04 | Metadata, copyright, and the asset Version binding | `dpkg-deb -I`; copyright inspection; version-directory and manifest check |
| T-05 | Inventory holds only the binary and the copyright file; no per-type document module | `dpkg-deb -c`; asset requests from the installed binary |
| T-06 | Installs and uninstalls in an isolated Forky root, host untouched | Isolated `dpkg -i`, `-r`, `--purge`; host `dpkg -l` before and after |
| T-07 | Installed binary resolves on `PATH` and prints the package version | `PATH`-prefixed `papyrus --version` |
| T-08 | Task, milestone, and release lifecycles work from the installed binary | Installed-binary commands and exit codes |
| T-09 | Search over task, milestone, and release records | Installed-binary search commands |
| T-10 | Browser categories, view, edit round-trip, conflict and not-found pages | Installed-binary browser commands plus HTTP requests |
| T-11 | JSON API and embedded assets | `GET /api/*`, `GET /assets/*` |
| T-12 | Asset cache classes, CSP classes, layout, and the manifest | Header and markup assertions; manifest test; binary size |
| T-13 | Offline operation | Isolated network namespace run with a failing outbound connection |
| T-14 | Browser configuration keys | `config list`, `get`, `set`, `unset` and startup-line assertions |
| T-15 | Milestone-level integration test without duplicate contract coverage | The new test file; `cargo test` |
| T-16 | Verification record complete, including expected deltas, LOC, and discarded build | `docs/packaging.md` review |

## Assumptions

- Assumption: the verification host is Debian Forky amd64 with the pinned toolchain, `fakeroot`, `dpkg-deb`, `ip`, and `unshare -rn` available.
- Assumption: isolated install via `fakeroot dpkg --instdir/--admindir` is sufficient; a Forky container, chroot, or virtual machine may substitute with the same checks.
- Assumption: scratch directories live under `./.tmp/` and are removed before the step ends.
- Assumption: the package Version is the `Cargo.toml` version; a version bump belongs to the release boundary.
- Assumption: `scripts/build-deb.sh` is the packaging script for both the milestone verification and the single release boundary, is used unchanged, and does not force a rebuild; no separate build-package script exists, and the release build runs first.
- Assumption: the production LOC figure is reported at every verification as information and at the release boundary; it never blocks verification or release.
- Assumption: the backlog commands, search, and the browser add no runtime dependency, so the dependency-license audit remains valid.
- Assumption: no UI change ships, so the approved mockups remain the layout reference and no new mockup is produced.
- Assumption: browser-version support is verified at markup, feature-subset, and HTTP level, not in a real GUI browser.
- Assumption: backlog and decision records stay bespoke, so their pages are compared with the captured output rather than through the document field map.
- Assumption: the offline check is environment-dependent; the documented fallback applies when `unshare -rn` is unavailable.
- Assumption: the retained release package and the release evidence are produced only at the single release boundary.

## Open questions

None of these blocks verification; each has a default.

- **OQ-1 (default: markup, asset, and HTTP level)** — Is a real-GUI browser run required? Default: no. The verification checks served markup, assets, headers, and viewport hooks, and records the limitation.
- **OQ-2 (default: isolated dpkg root on the build host)** — Is a full Forky container required? Default: no. The isolated install exercises Forky's glibc and dpkg and never touches the host.
- **OQ-3 (default: record, do not fix)** — Should a defect or a stale statement be fixed inside the verification step? Default: no. It is recorded with a disposition and attributed to its owning surface.
