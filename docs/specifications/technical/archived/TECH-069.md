---
id: TECH-069
title: "Three-Category Handler Set Release Verification and Packaging"
created_date: '2026-11-16T17:46:54Z'
updated_date: '2026-11-22T15:46:06Z'
labels: []
references:
  - ADR-0003
  - ADR-0010
  - ADR-0013
  - ADR-0016
  - ADR-0017
  - ADR-0019
  - ADR-0020
  - ADR-0021
  - ADR-0022
  - ADR-0023
  - ADR-0024
  - ADR-0027
  - ADR-0036
  - ADR-0037
  - ADR-0038
  - ADR-0039
  - BR-001
  - BR-016
  - NFR-003.002
  - NFR-003.003
  - NFR-004.002
  - NFR-005.001
  - NFR-007.001
  - NFR-007.003
  - REQ-001.004
  - REQ-005.007
  - REQ-005.010
  - REQ-005.011
  - REQ-006.014
  - REQ-006.018
  - REQ-006.019
  - REQ-007.001
  - REQ-007.002
  - REQ-007.003
  - REQ-007.004
  - ADR-043
status: archived
---
# TECH-069 Three-Category Handler Set Release Verification and Packaging

## Purpose

Prove the descriptor-driven refactor at its single release boundary and publish exactly one final package. The refactor is one deliverable with no intermediate release: the backlog, decision, and document handlers ship together, and the document handlers are one descriptor-driven surface over one `DocumentModel` and one engine. RC packages published at every verified milestone that carries the packaging gate are verification builds, not release artifacts; each is kept in `builds/<version>/` and marked when superseded, and exactly one final release artifact exists.

Verification covers the three-category handler set across the CLI, search, the
browser, an upgrade from a pre-refactor configuration, and the installed Debian
package. It also proves the no-drift rules that keep per-type document modules,
static document commands, the superseded content model, document entries in the presentation
table, and a process-global descriptor cache from returning.

This is a verification and packaging step. A defect found while verifying is
recorded with a disposition (follow-up, accepted risk, or release blocker); it
is not fixed here.

## Scope

### In scope

- The full Rust suite, with the three-category handler suites and the document
  model matrix named in the evidence.
- A CLI matrix for model-derived verbs and options, generated help, `--json`,
  and the documented exit codes `0/1/2/3/4`.
- A browser matrix for a user-defined type and the three document models: list,
  detail, edit, conflict, not-found, read-only status, validated status filter,
  archived/unknown/disabled equivalence, JSON API, and assets.
- One upgrade scenario from a pre-release configuration containing `req`,
  `deploy`, and a `deprecated/` folder.
- Debian package build, metadata and content inspection, isolated install,
  offline run of the installed binary against a project whose document type
  comes from configuration only, uninstall, purge, and host-state protection.
- Automated cross-surface, upgrade, and no-drift checks.
- Findings and dispositions, and evidence in the packaging record.

### Out of scope

- Changing source, user-facing documentation, requirements, business rules,
  architecture, or the packaging script. A defect is a recorded finding.
- Re-testing every per-feature contract; the existing suites remain
  authoritative and this step adds the cross-surface and upgrade evidence.
- New UI mockups or design work.
- Man-page sources, a man-page generator, or a new man-page claim. ADR-0021
  remains proposed and the scope decision is deferred.
- Debian releases other than the supported release and architectures other
  than amd64.
- Real-GUI runs in Firefox, Edge, and Safari; browser verification is HTTP,
  markup, asset, and console level.
- A version bump. The package version is supplied to the component build
  script by the versioning step.

## Behavior

### 1. Isolation, fixtures, and repository protection

1. Every scratch project, fixture, isolated dpkg root, and staged package tree
   is created under `./.tmp/` and removed when the step ends, including on
   failure. No system temporary directory is used.
2. The upgrade fixture is a fresh project created with `papyrus init --no-git`
   and then overwritten with a pre-refactor `papyrus.config.yml` containing a
   `req` entry, a `deploy` entry, and documents under
   `docs/requirements/business/deprecated/` and `docs/deployments/`.
3. The repository's own backlog store is never used as a fixture and is never
   written. `git status --porcelain` before and after records only the expected
   verification artifacts.
4. The fixture configuration and document trees are hashed before and after
   each read-only step; a step that mutates a fixture it should only read
   invalidates the run and is repeated.
5. No requirement, business rule, ADR, README, or upgrade-notes file is edited.
   A stale statement is a finding for its owning surface or the release owner.

### 2. Test suites

1. The full suite runs on the pinned toolchain: `cargo test`, recording the
   unit and integration counts and the aggregate result.
2. The three-category handler suites must pass and are named in the evidence:
   - backlog: the task board, milestone, release, and decision list/view suites;
   - decision: the decision record suite;
   - document: the CLI, search, browser, and model matrix suites for
     `SingleMarkdown`, `GroupedMarkdown`, and `SingleHtmlMockup`.
3. The document model matrix covers one seed type per model plus a
   user-defined `SingleMarkdown` type: create, list, view, edit, archive,
   delete, `status` or `supersede`, grouped roles, and the fixed HTML
   metadata.
4. Configuration, help golden, migration, native-layout, no-drift, search, and
   browser suites run unchanged.
5. Quality gates: `cargo build --release`, `cargo clippy --all-targets -- -D
   warnings`, and `cargo fmt --check` all pass. The release binary is used for
   the package and installed-binary checks.

### 3. CLI matrix

Run every row against the release binary and, in section 6, against the
installed binary. A user-added type named `note` is declared in
`papyrus.config.yml` with `prefix: NOTE`, `directory: notes/`, and
`statuses: [active, archived, review, approved]`, `default_status: active`.

| # | Check | Expected |
|---|---|---|
| C-01 | `papyrus --help` | exit `0`; lists every enabled document slug and the built-in non-document commands; includes `note` once declared |
| C-02 | `papyrus note --help` | exit `0`; lists `create`, `list`, `view`, `edit`, `archive`, `delete`, `status` |
| C-03 | `papyrus req --help` / `papyrus nfr --help` | exit `0`; baseline verbs plus `create-group` and `--group` |
| C-04 | `papyrus ui-mockup --help` | exit `0`; baseline plus `supersede`; no `status` |
| C-05 | `papyrus nfreq --help` | exit `2` |
| C-06 | `papyrus unknown-slug --help` | exit `2` |
| C-07 | `papyrus note ...` with the entry disabled | exit `2` for every verb |
| C-08 | `papyrus deploy --help` before declaration | exit `2` |
| C-09 | `note create "First" --description ... --label ... --reference ...` | exit `0`; prints `NOTE-001`; writes the record under the configured directory |
| C-10 | `note list` / `note view NOTE-001` | exit `0`; ID-ordered; view shows the field map and persisted metadata |
| C-11 | `note edit NOTE-001 --title ...` | exit `0`; only the supplied field changes; ID/path unchanged |
| C-12 | `note status NOTE-001 review` | exit `0`; front-matter status is `review`; file location unchanged |
| C-13 | `note status NOTE-001 archived` | exit `4`; stderr names `archive`; file unchanged |
| C-14 | `note archive NOTE-001` | exit `0`; moves to the `archived/` subfolder; absent from `list`; prior custom status discarded |
| C-15 | `note delete NOTE-002` without `--force` | prompt; declining changes nothing; accepting archives then removes |
| C-16 | `note delete NOTE-003 --force` | exit `0`; no prompt; file gone |
| C-17 | `note view NOTE-999` | exit `3` |
| C-18 | `note create` with an empty title, control character, or path separator | exit `4`; nothing written |
| C-19 | `note create` in an unwritable directory | exit `1` |
| C-20 | `--json` on create, list, view, edit, status, archive, delete | one JSON value on stdout; documented shape; errors also JSON |
| C-21 | `papyrus search <title>` / `--type doc` / `--json` | finds `NOTE-*` as `doc`; archived/disabled records hidden |
| C-22 | A file under the configured `deprecated/` folder | not listed, viewed, searched, or browsed; inert |
| C-23 | Manual recovery: move the file to the active directory and set `status: active` | visible in `list` and `view`; no restore command exists |
| C-24 | Invalid configuration (missing `archived` from `statuses`) | every validating command exits `4` naming `document_types.<slug>.<path>`; `config get`/`config list` still read |
| C-25 | Repair via `config set`/`config unset` | valid merged configuration writes; invalid merged configuration exits `4` and leaves the file byte-identical |
| C-26 | `init --force` on the broken project | rewrites the eleven minimal seed entries; commands work again |

`--json` is accepted before or after the subcommand exactly as REQ-001.004
defines, and `--dir`/`--config` apply to every row.

### 4. Browser matrix

Start `papyrus --dir <fixture> browser --no-open --port <free>` and use HTTP
only. The `note` fixture carries records with `active`, `review`, and
`approved` statuses.

| # | Check | Expected |
|---|---|---|
| B-01 | Startup line | exactly `Serving http://127.0.0.1:<port>/` on stdout |
| B-02 | `/?type=note` | `200`; `Notes` under the `Documents` sidebar group; element order `ID, Title, Status, family slots, Created, Updated, Actions` (the `note` row has no family slots); count and rows match the CLI |
| B-03 | `/doc/NOTE-001` | `200`; status read-only; no status control; field sections match the descriptor |
| B-04 | `/doc/NOTE-001/edit` | `200`; no `name="status"`; field controls, labels, and references present |
| B-05 | Edit POST | `303` to `/doc/NOTE-001`; changes persist and are visible through the CLI; unknown section and key preserved |
| B-06 | Identical no-op save | file byte-identical; `updated_date` not refreshed |
| B-07 | Stale version | `409`; file unchanged; submitted values shown |
| B-08 | Status POST with `status=archived` | `422`; file unchanged |
| B-09 | Filter values | exactly `active`, `review`, `approved` in that order; no `archived`; `Show all` omits `status` |
| B-10 | `?status=review` / `?status=approved` | only matching records; empty state when none |
| B-11 | `?status=archived`, `?status=bogus`, `?status=Review`, `?status=`, duplicate `status` | `400` with the standard error page and no record list |
| B-12 | No `status` parameter | every active-directory record listed, including custom statuses |
| B-13 | Archived record | omitted from list; detail, edit, POST, and API return `404`; the page is identical to an unknown ID |
| B-14 | Disabled type | no sidebar entry, no list, no detail, no API (`404`); retired `/lifecycle` and `/config` also `404` |
| B-15 | Grouped type | group and member rows render in merged order; group view lists members; member view links the group; no membership form control |
| B-16 | `/api/docs/note` | `200`; array rows with `id`, `kind`, `title`, `status`, `labels`, `references`, `version`, `path`, `metadata` |
| B-17 | `/assets/app.css`, `/assets/app.js` | `200`, non-empty, correct content type, `Cache-Control: no-cache`, `ETag`; a matching `If-None-Match` returns `304` |
| B-18 | Versioned vendor asset | `200` with `Cache-Control: public, max-age=31536000, immutable`; an unversioned vendor path returns `404` |
| B-19 | Served HTML/CSS/JS | no external references; HTML and JSON carry `Cache-Control: no-store` and the HTML CSP |
| B-20 | Offline (`unshare -rn`, loopback up) | outbound TCP fails; list, detail, edit, API, and assets succeed over loopback |

### 5. Upgrade scenario

Execute the upgrade notes literally against one isolated project. The scenario
is scripted and repeated by hand with the installed binary in section 6.

1. Create `./.tmp/upgrade/` with `papyrus init --no-git`, then write a
   pre-refactor configuration containing `document_types.req` and
   `document_types.deploy` and no field sequences. Create
   `docs/requirements/business/deprecated/REQ-900.md`, a `docs/deployments/`
   document, and one active `REQ-001`.
2. `papyrus --help` lists `req` and `deploy`; it does not list `nfreq`.
3. `papyrus nfreq --help` exits `2`; `papyrus req deprecate REQ-001` and
   `papyrus req restore REQ-001` exit `2`.
4. `papyrus req list` does not show `REQ-900`; `search` does not find its
   marker. The `deprecated/` folder is inert.
5. Follow the documented recovery: move `REQ-900.md` into
   `docs/requirements/business/` and set `status: active`. `req list` and
   `req view REQ-900` now work; the ID is unchanged.
6. Break the configuration as the notes describe (remove `archived` from a
   `statuses` list). Every command exits `4` naming the key; `config get` and
   `config list` still read; repair it with `config set` or the documented
   direct edit and confirm commands work again.
7. Declare the `deploy` type by editing `papyrus.config.yml` as the notes say.
   `papyrus deploy create "Release 1"` prints `DEP-001`, `papyrus deploy list`
   shows it, and `papyrus --help` lists `deploy`.
8. Confirm the final state: no `nfreq`, no static per-type document command,
   no `deprecated/` read, one working dynamic command, and a byte-identical
   source fixture outside the files the scenario intentionally edits.

### 6. Debian package, install, offline run, and removal

1. `cargo build --release` first, then `./target/release/papyrus --help` must
   list the enabled slugs before packaging.
   `./scripts/build-package.sh --version <version> --out builds/<version>/.build/<component>`
   produces `papyrus_<version>_amd64.deb` by invoking `scripts/build-deb.sh`
   for the Debian assembly; packaging assembles the component archive,
   `MANIFEST.txt`, and the `.sha256` file under the git-ignored
   `builds/<version>/`; size and SHA-256 are recorded.
2. The same script produces both package modes: an RC package at every verified
   milestone that carries the packaging gate unless that milestone's packaging
   is explicitly waived by the user, and the final package at this release
   boundary. `--version` is authoritative for the control `Version:` field and
   the artifact name and is injected without editing a tracked file. An RC
   package is kept in `builds/<version>/`, is never overwritten, and is marked
   when superseded; exactly one final release artifact is kept.
3. `dpkg-deb -I` records `Package`, `Version`, `Architecture`, `Depends`,
   `Section`, `Priority`, and the maintainer; the packaged copyright declares
   `GPL-3.0-or-later`. `dpkg-deb -c` lists only `/usr/bin/papyrus` (0755) and
   `/usr/share/doc/papyrus/copyright` (0644) plus directories.
4. The Debian assembly step enumerates the vendored version directories and
   reads the manifest `version` column. It fails the build when a package has no
   version directory, more than one, a manifest row with a different version,
   or a copyright section citing a different version. The Papyrus package
   version and the asset version remain unrelated.
5. Install from a transient copy of the package under `./.tmp/` in an isolated
   dpkg root with `fakeroot dpkg --instdir --admindir --force-depends`. Record
   the host `dpkg -l papyrus` state before and after; it must be unchanged.
6. The installed binary is `root:root` 0755, resolves on the isolated `PATH`,
   and prints the package version.
7. With the installed binary, create a project, declare `note` by editing
   `papyrus.config.yml` only, and run `papyrus note create`, `list`, `view`,
   `status`, `archive`, `delete`, and `search`. `papyrus --help` lists `note`.
   No rebuild and no installed file outside `/usr/bin/papyrus` participates.
8. `papyrus --help` and `papyrus note --help` are run inside `unshare -rn`
   with loopback up; an outbound TCP connection fails while both commands
   succeed. If the namespace is unavailable, record the limitation and use
   loopback/no-external-reference evidence instead.
9. No man page is expected or asserted for `note` or any config-defined slug.
   The man-page gap and ADR-0021 status are recorded as a release risk.
10. `dpkg -r` removes the binary; `dpkg --purge` removes the admin entry and
    leaves no papyrus file. The isolated roots, the staged tree, and the
    transient package copy are deleted; the package stays in
    `builds/<version>/`.

### 7. No-drift checks

The suite fails when any of the following reappears. It scans production lines
(excluding `#[cfg(test)]` items) and the named documentation files.

| # | Pattern | Allowed locations |
|---|---|---|
| D-01 | literal `deprecated/` in `src/` | none |
| D-02 | `nfreq` as a command in `src/` | the registry collision test's rejection fixture only |
| D-03 | document `deprecate` or `restore` command variants in `src/` | none |
| D-04 | a per-type document command variant, a static document-type command list, or a per-type document module in `src/` | none; the seed descriptors are the only type model |
| D-05 | a document slug literal used to select behaviour. **Scope:** any `match` or `if` on a document slug literal anywhere in `src/`, except the `SEED_DOCUMENT_TYPES` table in the config module; tests and documentation are exempt. | **Allowed location:** the `SEED_DOCUMENT_TYPES` table in `src/config.rs` only. |
| D-06 | the superseded content model, `document_detail.html`, `document_edit.html`, or a document entry in the static presentation table | none |
| D-07 | a process-global descriptor map or `Box::leak` for a document slug | none |
| D-08 | `papyrus nfreq`, a per-type document command block, or a `deprecated/` mention in `README.md`, the upgrade notes, or `docs/business-rules/*.md` | the removal/migration wording, the inert-folder recovery section, and the legacy-folder note |

Historical ADRs, historical specifications, packaging evidence, backlog
records, and archived tasks are excluded: they record past behaviour and must
not be rewritten. The test's allowlist is the table above, not a broad
directory exclusion.

### 8. Findings and dispositions

Each finding is verified, given a severity, and given a disposition. A finding
is **release-blocking** when a shipped command or documented contract is false
and no workaround exists; the release gate then reports `release-ready: no`
until the owner accepts the risk or a follow-up task resolves it.

- A defect in a shipped command, a record format, JSON output, a status code, or
  the package is recorded with its expected and observed values.
- A stale README, upgrade-note, or business-rule statement is a documentation
  finding for its owning surface.
- A verification limitation is recorded rather than claimed away: no real-GUI
  run, the supported Debian release and amd64 only, no man page for a
  config-defined slug, and the packaging script not forcing a rebuild.
- This step fixes no finding. A release-blocking finding is reported to the
  release owner with the gap, owner, and requested decision.

### 9. Evidence and repository protection

1. The packaging record gains a section with: date and timezone, environment
   (distribution, glibc, `rustc`/`cargo`, clippy, `dpkg`, `fakeroot`,
   `unshare`), isolation method, test counts, package name, size and SHA-256,
   quality-gate table, install/offline/removal table, findings with
   dispositions, and recorded limitations.
2. The verification notes carry the same summary plus the CLI and browser
   matrix tables, the upgrade-scenario transcript, the no-drift run result, and
   every command whose output is asserted.
3. The notes state release readiness explicitly: `release-ready: yes` only when
   every check is verified and no release-blocking finding is open; otherwise
   `release-ready: no` with the blocking finding IDs and owners.
4. The evidence records the exact search query, the package SHA-256, the test
   count, and the installed-binary version.
5. Every scratch directory, isolated dpkg root, staged package tree, and
   fixture under `./.tmp/` is removed before the step ends, including on
   failure; the package under `builds/<version>/` and the written evidence
   remain.
6. `git status` at the end lists only the verification artifacts and no product
   source, requirement, business rule, ADR, README, or upgrade-notes change.

## Inputs and outputs

| Input / output | Description |
|---|---|
| Source tree | The descriptor-driven three-category implementation: one document engine, one document surface, seed-derived descriptors, and the serving snapshot. |
| Toolchain | The pinned Rust toolchain; `dpkg-deb`, `dpkg`, `fakeroot`, `unshare`/`ip`, `curl`; the supported Debian release and amd64. |
| Packaging entry point | `scripts/build-package.sh --version <version> --out <dir>`; it invokes `scripts/build-deb.sh` for the Debian assembly, the package layout, and the asset version check. |
| Cross-surface test file | Cross-surface, upgrade, and installed-style checks against the built binary. |
| No-drift test file | The mechanical scanner of section 7. |
| `papyrus_<version>_amd64.deb` | The package produced by `scripts/build-package.sh --version <version>` through `scripts/build-deb.sh`, assembled under `builds/<version>/`; the RC package is kept there and the final release artifact is built the same way; size and SHA-256 recorded. |
| Isolated dpkg root | Under `./.tmp/`; removed after the run. |
| Packaging record | Verification evidence, findings, and release-readiness statement. |

## Validation and error behavior

| Condition | Consequence |
|---|---|
| A required suite fails | Blocked; the failing test, command, and output are recorded before continuing. |
| A CLI matrix row disagrees with the model-derived command contract | Finding; record expected vs observed. |
| A browser matrix row disagrees with the document surface contract | Finding; record route, status, and body evidence. |
| The upgrade scenario cannot reach a working dynamic command | Release-blocking finding; the documented migration is wrong. |
| A `deprecated/` file is read, listed, searched, or browsed | Release-blocking; the inert-folder contract fails. |
| A no-drift rule fires | Blocked until the offending line is recorded and attributed; this step does not edit it. |
| The package metadata, contents, or install differs from section 6 | Blocked; record the exact diff. |
| The RC package is overwritten, rebuilt in place, deleted, or staged, committed, tagged, pushed, or uploaded | Blocked; a package version is immutable and is not a release artifact. |
| More than one final release artifact exists | Blocked; exactly one final release artifact is kept. |
| The working tree is dirty after the packaging run | Blocked; the version is injected without editing a tracked file. |
| The host package database changes | Blocked; isolation failed. |
| The offline run needs the network | Offline claim not verified; record and use the loopback evidence. |
| A release-blocking finding remains open | `release-ready: no`; name the finding, owner, and requested decision. |
| A requirement, rule, or ADR conflict is found | Record an open question and escalate; do not edit requirements, rules, or ADRs. |

## Assumptions

- Assumption: the verification host matches the supported Debian release and
  amd64, with the pinned toolchain, `dpkg`, and `fakeroot`/`unshare` available.
- Assumption: packaging runs the component build script
  `scripts/build-package.sh --version <version> --out builds/<version>/.build/<component>`,
  which invokes `scripts/build-deb.sh` for the Debian assembly; `--version` is
  authoritative and is injected without editing a tracked file, and
  `build-deb.sh` keeps its `Cargo.toml`-derived default only when run directly.
- Assumption: all three category handlers and the descriptor engine are the
  shipped behaviour; this step verifies that behaviour and does not reopen its
  implementation.
- Assumption: product behaviour is unchanged. A defect is a recorded finding,
  not a fix in this step.
- Assumption: existing suites remain authoritative for per-feature contract
  coverage; the cross-surface test adds upgrade and installed-binary
  assertions only.
- Assumption: the user-added type is `note` because the CLI and browser
  matrices use it; no new product type is introduced.
- Assumption: historical ADRs, historical specifications, packaging evidence,
  and backlog records are excluded from the no-drift documentation scan because
  they record past behaviour.
- Assumption: config-defined slugs have no man page and the man-page
  requirement is not claimed for them; the package check asserts absence, not
  presence.

## Open questions

Non-blocking; each has a default.

- **OQ-1 (default: release owner decides).** Who accepts a release-blocking
  finding? If one is confirmed, the gate reports `release-ready: no` and names
  the finding. Only the project owner may accept the risk in writing;
  otherwise a follow-up task must be opened before release sign-off.
- **OQ-2 (default: record, do not fix).** Should a trivially wrong README or
  upgrade-notes line be fixed inside this step? Default: no. The step is
  verification-only; the finding is attributed to the owning surface or the
  release owner.
- **OQ-3 (default: HTTP/markup-level browser verification).** Is a real GUI run
  required? Default: no, consistent with prior release verification and the
  recorded browser limitation.
