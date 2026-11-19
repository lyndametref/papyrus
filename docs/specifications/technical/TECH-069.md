---
id: TECH-069
title: "Config-Defined Document Types Release Verification and Packaging"
created_date: '2026-11-16T17:46:54Z'
updated_date: '2026-11-18T22:26:56Z'
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
  - BR-001
  - BR-007
  - BR-016
  - MOCK-0015
  - NFR-03.002
  - NFR-03.003
  - NFR-04.002
  - NFR-05.001
  - REQ-001
  - REQ-01.004
  - REQ-05.007
  - REQ-05.010
  - REQ-05.011
  - REQ-06.014
  - REQ-07.001
  - REQ-07.002
  - REQ-07.003
  - REQ-07.004
  - REQ-900
  - TASK-105
  - TASK-111
  - TASK-115
  - TASK-116
  - TASK-117
  - TASK-122
  - ADR-034
  - REQ-06.018
  - RE-005
status: draft
---
# TECH-069 Config-Defined Document Types Release Verification and Packaging

> **Revised (TASK-201, 2026-11-18):** the `/lifecycle` expectations in matrix rows B-14 and B-15, the T-117-03 lifecycle check, and the `/lifecycle` descriptor-count assumption are retired per ADR-034 and REQ-06.018; `GET /lifecycle` returns `404` and the disabled-type check no longer depends on a lifecycle page. See TECH-080.

## Description
## Purpose

Close the m-17 release gate. Twelve implementation tasks each tested their own
slice; this ticket proves the combined document-type change works across the
CLI, search, the browser, an upgrade from a pre-change configuration, and the
installed Debian package. It also proves the no-drift rules that keep
`deprecated/`, static per-type document commands, the `nfreq` alias, and
slug-literal dispatch from returning.

This is a verification and packaging ticket. It does not change product
behaviour. A defect found while verifying is recorded with a disposition
(follow-up, accepted risk, or release blocker); it is not fixed here, because
the implementation tasks are complete and none may be silently reopened.

Canonical traceability remains in REQ-07.001 to REQ-07.004, REQ-05.007,
REQ-05.010, REQ-05.011, BR-001, BR-016, and ADR-0016 to ADR-0024. This
specification defines only what must be verified, how, and how evidence is
recorded.

## Scope

### In scope

- The full Rust suite, with the document-type suites named explicitly in
  section 2 and a recorded test count.
- A CLI matrix for the generated command family, generated help, `--json`, and
  the documented exit codes `0/1/2/3/4` (section 3).
- A browser matrix for a config-defined type: list, detail, edit, read-only
  status, validated status filter, archived/unknown equivalence,
  disabled-type invisibility, the retired `/lifecycle` route returning `404`, JSON API, and assets (section 4).
- One upgrade scenario from a pre-change configuration containing `req`,
  `deploy`, and a `deprecated/` folder, executed literally from the Release 1.0
  upgrade notes (section 5).
- Debian package build, metadata and content inspection, isolated install,
  offline run of the installed binary against a project whose new command comes
  from configuration only, uninstall, purge, and host-state protection
  (section 6).
- Two new automated test files: `tests/release_document_types_smoke.rs` for the
  cross-surface and upgrade assertions, and `tests/document_type_no_drift.rs`
  for the mechanical no-drift rules (section 7).
- Verification and findings for the five carried gaps G-1 to G-5 (section 8).
- Evidence in `docs/packaging.md` and in the TASK-117 ticket notes (section 9).

### Out of scope

- Changing `src/`, `README.md`, `docs/release-1.0-upgrade-notes.md`,
  `docs/requirements/**`, `docs/business-rules/**`, `docs/architecture/**`, or
  `scripts/build-deb.sh`. A defect is a recorded finding, not an edit made
  here.
- Re-testing every per-feature contract. `tests/browser_config_types_cli.rs`,
  `tests/search_config_types_cli.rs`, `tests/config_list_keys_cli.rs`,
  `tests/cli_dispatch.rs`, and the unit modules remain authoritative; this
  ticket runs them and adds the cross-surface and upgrade evidence on top.
- New UI mockups or design work.
- Man-page sources, a man-page generator, or any new NFR-03.003 claim.
  ADR-0021 remains Proposed and the scope decision is deferred.
- Debian releases other than Forky and architectures other than amd64
  (ADR-0003, NFR-03.002).
- Real-GUI runs in Firefox, Edge, and Safari; browser verification is HTTP,
  markup, and asset level (NFR-05.001 limitation recorded in section 8.6).
- A version bump. The package carries the `Cargo.toml` version.

## Behavior

### 1. Isolation, fixtures, and repository protection

1. Every scratch project, fixture, isolated dpkg root, and staged package tree
   is created under `./.tmp/task117/` and removed when the task ends, including
   on failure. No system temporary directory is used.
2. The upgrade fixture is a fresh project created with `papyrus init --no-git`
   and then overwritten with a pre-change `papyrus.config.yml` containing a
   `req` entry, a `deploy` entry, and documents under
   `docs/requirements/business/deprecated/` and `docs/deployments/` (section 5).
3. The repository's own `backlog/` store is never used as a fixture and is
   never written. `git status --porcelain` before and after records only the
   expected TASK-117 artifacts.
4. The fixture configuration and document trees are hashed before and after
   each read-only step; a step that mutates a fixture it should only read
   invalidates the run and must be repeated.
5. No requirement, business rule, ADR, README, or upgrade-notes file is edited.
   A stale statement is a finding for its owning task or the release owner.

### 2. Test suites

1. The full suite runs on the pinned toolchain: `cargo test`, recording the
   unit and integration counts and the aggregate result.
2. The document-type suites must pass and are named in the evidence:
   - `tests/cli_dispatch.rs` — generated help, per-slug help, unknown and
     disabled exit `2`, invocation parsing.
   - `src/registry.rs` unit tests — descriptors, status models, slug lookup,
     collision guards.
   - `src/config.rs` unit tests — configuration merge, validation, flattened
     keys.
   - `src/document.rs` unit tests — generic lifecycle, status, archive,
     relocation, delete, typed extensions.
   - `tests/browser_config_types_cli.rs` — the T-114 matrix.
   - `tests/search_config_types_cli.rs` — the T-113 matrix.
   - `tests/config_list_keys_cli.rs` — list-valued `statuses` and repair.
   - `tests/cli.rs` — the eleven init defaults.
   - `tests/no_drift.rs` — the existing ADR-0013 mechanical checks.
3. Quality gates: `cargo build --release`, `cargo clippy --all-targets -- -D
   warnings`, and `cargo fmt --check` all pass. The release binary is used for
   the package and installed-binary checks.
4. The two new test files are part of the same `cargo test` run; the evidence
   names them and their test names.

### 3. CLI matrix

Run every row against `target/release/papyrus` and, in section 6, against the
installed binary. A user-added type named `note` is declared in
`papyrus.config.yml` with `prefix: NOTE`, `directory: notes/`, and
`statuses: [active, archived, review, approved]`, `default_status: active`.

| # | Check | Expected |
|---|---|---|
| C-01 | `papyrus --help` | exit `0`; lists the eleven default slugs and the built-in non-document commands; includes `note` once declared |
| C-02 | `papyrus note --help` | exit `0`; lists `create`, `list`, `view`, `edit`, `archive`, `delete`, `status` |
| C-03 | `papyrus req --help` / `papyrus nfr --help` | exit `0`; baseline verbs plus `create-group` and `--group` |
| C-04 | `papyrus ui-mockup --help` | exit `0`; baseline plus `supersede`; no `status` |
| C-05 | `papyrus nfreq --help` | exit `2` |
| C-06 | `papyrus unknown-slug --help` | exit `2` |
| C-07 | `papyrus note ...` with `document_types.note.enabled: false` | exit `2` for every verb |
| C-08 | `papyrus deploy --help` before declaration | exit `2` |
| C-09 | `note create "First" --description ... --label ... --reference ...` | exit `0`; prints `NOTE-001`; writes `docs/notes/NOTE-001.md` |
| C-10 | `note list` / `note view NOTE-001` | exit `0`; ID-ordered; view shows persisted metadata and body |
| C-11 | `note edit NOTE-001 --title ...` | exit `0`; only the supplied field changes; ID/path unchanged |
| C-12 | `note status NOTE-001 review` | exit `0`; front-matter status is `review`; file location unchanged |
| C-13 | `note status NOTE-001 archived` | exit `4`; stderr names `archive`; file unchanged |
| C-14 | `note archive NOTE-001` | exit `0`; moves to `docs/notes/archived/NOTE-001.md`; absent from `list`; prior `review` value discarded |
| C-15 | `note delete NOTE-002` without `--force` | prompt; declining changes nothing; accepting archives then removes |
| C-16 | `note delete NOTE-003 --force` | exit `0`; no prompt; file gone |
| C-17 | `note view NOTE-999` | exit `3` |
| C-18 | `note create` with an empty title, control character, or path separator | exit `4`; nothing written |
| C-19 | `note create` in an unwritable directory | exit `1` |
| C-20 | `--json` on create, list, view, edit, status, archive, delete | one JSON value on stdout; documented shape; errors also JSON |
| C-21 | `papyrus search <title>` / `--type doc` / `--json` | finds `NOTE-*` as `doc`; archived/disabled records hidden |
| C-22 | A file under `docs/notes/deprecated/NOTE-900.md` | not listed, viewed, searched, or browsed; inert |
| C-23 | Manual recovery: move the file to `docs/notes/` and set `status: active` | visible in `list` and `view`; no restore command exists |
| C-24 | Invalid configuration (missing `archived` from `statuses`) | every validating command exits `4` naming `document_types.<slug>.<field>`; `config get`/`config list` still read |
| C-25 | Repair via `config set`/`config unset` | valid merged configuration writes; invalid merged configuration exits `4` and leaves the file byte-identical |
| C-26 | `init --force` on the broken project | rewrites the eleven defaults; commands work again |

`--json` is accepted before or after the subcommand exactly as REQ-01.004
defines for other commands, and `--dir`/`--config` apply to every row.

### 4. Browser matrix

Start `papyrus --dir <fixture> browser --no-open --port <free>` and use HTTP
only. The `note` fixture carries records with `active`, `review`, and
`approved` statuses.

| # | Check | Expected |
|---|---|---|
| B-01 | Startup line | exactly `Serving http://127.0.0.1:<port>/` on stdout |
| B-02 | `/?type=note` | `200`; `Notes` under the `Documents` sidebar group; shared `Status / ID / Title / Created / Updated` columns; count and rows match the CLI |
| B-03 | `/doc/NOTE-001` | `200`; status shown read-only; no status control; metadata and body match the file |
| B-04 | `/doc/NOTE-001/edit` | `200`; no `name="status"` or `name="status_change"`; labels/references fields present |
| B-05 | Edit POST | `303` to `/doc/NOTE-001`; changes persist and are visible through the CLI; unknown section and key preserved |
| B-06 | Identical no-op save | file byte-identical; `updated_date` not refreshed |
| B-07 | Stale version | `409`; file unchanged |
| B-08 | Status POST with `status=archived` | `422`; file unchanged |
| B-09 | Filter values | exactly `active`, `review`, `approved` in that order; no `archived`; `Show all` omits `status` |
| B-10 | `?status=review` / `?status=approved` | only matching records; empty state when none |
| B-11 | `?status=archived`, `?status=bogus`, `?status=Review`, `?status=`, duplicate `status` | `400` with the standard error page and no record list |
| B-12 | No `status` parameter | every active-directory record listed, including custom statuses |
| B-13 | Archived record | omitted from list; detail, edit, POST, and API return `404`; the 404 page is identical to an unknown ID |
| B-14 | `document_types.note.enabled: false` | no sidebar entry, no list, no detail, no API (`404`); `GET /lifecycle` returns `404` (retired per ADR-034), so no page lists `Notes` |
| B-15 | `/lifecycle` (retired) | `GET /lifecycle` and `GET /?type=lifecycle` return `404` per ADR-034 and REQ-06.018; the former Backlog/Decisions/Documents grouping and descriptor count are retired history, not a served surface |
| B-16 | `/api/docs/note` | `200`; array rows with `id`, `kind`, `title`, `status`, `labels`, `references`, `version`, `path`, `metadata` |
| B-17 | `/assets/app.css`, `/assets/app.js` | `200`, non-empty, correct content type, `Cache-Control: no-store` |
| B-18 | Served HTML/CSS/JS | no external references; HTML carries the CSP |
| B-19 | Offline (`unshare -rn`, loopback up) | outbound TCP fails; list, detail, edit, API, and assets succeed over loopback |

### 5. Upgrade scenario

Execute the Release 1.0 upgrade notes (`docs/release-1.0-upgrade-notes.md`,
"Document types are now configuration-defined") literally against one isolated
project. The scenario is scripted in `tests/release_document_types_smoke.rs`
and repeated by hand with the installed binary in section 6.

1. Create `./.tmp/task117/upgrade/` with `papyrus init --no-git`, then write a
   pre-change configuration containing `document_types.req` and
   `document_types.deploy` and no `statuses` extensions. Create
   `docs/requirements/business/deprecated/REQ-900.md` and a `docs/deployments/`
   document, plus one active `REQ-001`.
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
   `config list` still read; repair it with `config set` (or the documented
   direct edit) and confirm commands work again.
7. Declare the `deploy` type by editing `papyrus.config.yml` as the notes say.
   `papyrus deploy create "Release 1"` prints `DEP-001`, `papyrus deploy list`
   shows it, and `papyrus --help` lists `deploy`.
8. Confirm the final state: no `nfreq`, no static per-type command assumption,
   no `deprecated/` read, one working dynamic command, and a byte-identical
   source fixture outside the files the scenario intentionally edits.

### 6. Debian package, install, offline run, and removal

1. `cargo build --release` first, then `./target/release/papyrus --help` must
   list the enabled slugs before packaging. `./scripts/build-deb.sh` runs
   unchanged and produces `papyrus_<version>_amd64.deb`; size and SHA-256 are
   recorded.
2. `dpkg-deb -I` records `Package`, `Version`, `Architecture`, `Depends`,
   `Section`, `Priority`, and the maintainer; the packaged copyright declares
   `GPL-3.0-or-later`. `dpkg-deb -c` lists only `/usr/bin/papyrus` (0755) and
   `/usr/share/doc/papyrus/copyright` (0644) plus directories.
3. Install in an isolated dpkg root at `./.tmp/task117/{inst,adm}` with
   `fakeroot dpkg --instdir --admindir --force-depends`. Record the host
   `dpkg -l papyrus` state before and after; it must be unchanged.
4. The installed binary at `<inst>/usr/bin/papyrus` is `root:root` 0755,
   resolves on the isolated `PATH`, and prints the package version.
5. With the installed binary, create a project, declare `note` by editing
   `papyrus.config.yml` only, and run `papyrus note create`, `list`, `view`,
   `status`, `archive`, `delete`, and `search`. `papyrus --help` lists `note`.
   No rebuild and no installed file outside `/usr/bin/papyrus` participates.
6. `papyrus --help` and `papyrus note --help` are run inside `unshare -rn`
   with loopback up; an outbound TCP connection fails while both commands
   succeed. If the namespace is unavailable, record the limitation and use
   loopback/no-external-reference evidence instead.
7. No man page is expected or asserted for `note` or any config-defined slug.
   The NFR-03.003 gap and ADR-0021 status are recorded as a release risk
   (section 8.6).
8. `dpkg -r` removes the binary; `dpkg --purge` removes the admin entry and
   leaves no papyrus file. The isolated roots and staged tree are deleted.

### 7. No-drift checks

A new `tests/document_type_no_drift.rs` must fail the suite when any of the
following reappears. It scans production lines (excluding `#[cfg(test)]` items)
and the named documentation files.

| # | Pattern | Allowed locations |
|---|---|---|
| D-01 | literal `deprecated/` in `src/` | none |
| D-02 | `nfreq` as a command in `src/` | the registry collision test's rejection fixture only |
| D-03 | document `deprecate` or `restore` command variants in `src/` | none |
| D-04 | a per-type document command variant or a static document-type command list in `src/cli/` | none; `src/registry.rs` descriptors are the only type model |
| D-05 | a document slug literal used to select behaviour by `match` or `if` in `src/cli/dispatch.rs`, `src/cli/tree.rs`, `src/document.rs`, `src/search.rs`, or `src/browser/` | the type's own content module (`req.rs`, `nfr.rs`, …) and `src/registry.rs` |
| D-06 | `papyrus nfreq`, a per-type document command block, or a `deprecated/` mention in `README.md`, `docs/release-1.0-upgrade-notes.md`, or `docs/business-rules/*.md` | the removal/migration wording, the inert-folder recovery section, and the legacy-folder note |
| D-07 | a static document-type command inventory in the two config-type specifications (`search-` and `browser-config-defined-document-types.md`) | none |

Historical ADRs (`docs/architecture/adr/ADR-0010`, ADR-0022, ADR-0023),
historical specifications, `docs/packaging.md` evidence, `backlog/` records,
and archived tasks are excluded: they record past behaviour and must not be
rewritten. The test's allowlist is the table above, not a broad directory
exclusion, so a new stale claim in live code or user-facing documentation
fails the suite.

### 8. Carried gaps to verify or explicitly accept

Each gap is verified, given a severity, and given a disposition. A gap is
**release-blocking** when a shipped command or documented contract is false and
no workaround exists; the release gate then reports “not ready” until the owner
accepts the risk or a follow-up task resolves it. TASK-117 does not fix any
gap.

#### G-1 — `config set` cannot bootstrap a new document type (TASK-116 note 2)

- Check: on the default configuration, `config set
  document_types.note.prefix NOTE` exits `4` because the merged entry still
  lacks `directory`; the file is byte-identical. A mapping value is refused.
  The documented direct edit then makes `papyrus note` work.
- Pass condition: the documented workaround is accurate and the error names
  the missing field. Default disposition: accepted documented limitation with
  a usability follow-up for TASK-111's owner if the error does not name the
  field. Not release-blocking.

#### G-2 — Upgrade notes line 148 mentions deprecated locations for task IDs

- Check: `docs/release-1.0-upgrade-notes.md` says task IDs start from the
  highest number in “active, archived, and deprecated locations”. Run an
  upgrade fixture with an archived task and confirm allocation reads only the
  active and archived stores; no `deprecated/` task store is read.
- Pass condition: allocation matches the code and REQ-05.010 while the wording
  is wrong. Default disposition: documentation finding for the upgrade-notes
  owner; not release-blocking because no task data is lost or misallocated.

#### G-3 — README milestone status contradicts the registry

- Check: `README.md` (`STATUSES` → `Milestones`) says `active -> archived`;
  `src/registry.rs` `MILESTONE_STATES` is `["open", "closed"]` and REQ-05.010
  fixes `open`/`closed`. Exercise `papyrus milestone archive` and inspect the
  stored front matter to determine the shipped behaviour.
- Pass condition: the shipped lifecycle is identified and the stale README
  wording is recorded. Default disposition: README finding for its owner; not
  release-blocking for the document-type gate.

#### G-4 — `config list` omits `document_types.<slug>.default_status`

- Check: `config get document_types.note.default_status` returns the value,
  `config list` does not print it, and `config set`/`config unset` still
  validate it. Test the documented repair path on an invalid
  `default_status`.
- Pass condition: the omission and its repair impact are recorded. Default
  disposition: release-blocking only if an invalid `default_status` cannot be
  inspected or repaired through the documented surfaces; otherwise a
  follow-up for TASK-111's owner.
- Resolution (TASK-122): closed. `config list` now prints
  `document_types.<slug>.default_status` immediately after that slug's
  `statuses` line (or after `enabled` when no status list is declared) and
  omits the key when the entry declares none, so the value the validator
  rejects is inspectable before repair. Verified on the G-4 reproduction in
  `tests/release_document_types_smoke.rs`: `config list` prints
  `document_types.note.default_status: archived`, `config get` returns
  `archived`, and `config set document_types.note.default_status active` exits
  `0` and restores a valid configuration. JSON carries the same value through
  `DocTypeEntry::default_status`; validation, `config get`, and `config set`
  semantics are unchanged.

#### G-5 — Release `close` is documented but not implemented (TASK-115 note 4)

- Check: BR-007 states `papyrus release close <id>` is the only
  `planned -> released` transition. `src/release.rs` `ReleaseCmd` has no
  `Close` variant and `papyrus release close REL-001` exits `2`. The browser
  still renders a release status transition the CLI does not provide.
- Pass condition: the mismatch is reproduced and recorded. Default
  disposition: release-blocking for the m-17 release gate unless a follow-up
  code task is opened or the owner accepts amending BR-007. TASK-117 records
  the finding only.

#### G-6 — Verification limitations

- No real-GUI browser run (NFR-05.001 limitation).
- Debian Forky amd64 only (ADR-0003, NFR-03.002).
- No man page for config-defined slugs (ADR-0021 Proposed, NFR-03.003 gap).
- The packaging script does not force a rebuild; mitigation is building first
  and asserting `--help` before packaging.
- A real-GUI or non-Forky run is not claimed; the limitation is recorded in
  `docs/packaging.md` and the ticket notes.

### 9. Evidence and repository protection

1. `docs/packaging.md` gains a `## TASK-117 — Config-defined document types
   release verification and packaging` section with: date and timezone,
   environment (Debian Forky amd64, glibc, `rustc`/`cargo`, clippy, `dpkg`,
   `fakeroot`, `unshare`), isolation method, test counts, package name, size
   and SHA-256, quality-gate table, install/offline/removal table, findings
   G-1 to G-5 with dispositions, and recorded limitations G-6.
2. The ticket notes carry the same summary plus the CLI and browser matrix
   tables, the upgrade-scenario transcript, the no-drift run result, and every
   command whose output is asserted.
3. The notes state release readiness explicitly: `release-ready: yes` only
   when every AC is checked and no release-blocking gap is open;
   otherwise `release-ready: no` with the blocking gap IDs and owners.
4. The evidence records the exact front-matter-only search query, the package
   SHA-256, the test count, and the installed-binary version.
5. Every scratch directory, isolated dpkg root, staged package tree, and
   fixture under `./.tmp/task117/` is removed before the task ends, including
   on failure; the built `.deb` and the written evidence remain.
6. `git status` at the end lists only the new specification, the two new test
   files, `docs/packaging.md`, and the TASK-117 ticket metadata. No product
   source, requirement, business rule, ADR, README, or upgrade-notes file is
   changed.

## Inputs and outputs

| Input / output | Description |
|---|---|
| Source tree | Post-TASK-105 to TASK-116 state: registry-derived CLI, search, and browser; registry validation; list-valued config; upgrade notes and README. |
| Toolchain | Rust 1.99.1 via `rust-toolchain.toml`; `dpkg-deb`, `dpkg`, `fakeroot`, `unshare`/`ip`, `curl`; Debian Forky amd64. |
| `tests/release_document_types_smoke.rs` | New cross-surface, upgrade, and installed-style checks against the built binary. |
| `tests/document_type_no_drift.rs` | New mechanical no-drift scanner (section 7). |
| `papyrus_<version>_amd64.deb` | Built package in the project root; size and SHA-256 recorded. |
| Isolated dpkg root | `./.tmp/task117/{inst,adm}`; removed after the run. |
| `docs/packaging.md` | TASK-117 evidence section. |
| Ticket notes | Verification evidence, findings, and release-readiness statement. |

## Validation and error behavior

| Condition | Consequence |
|---|---|
| A required suite fails | Blocked; the failing test, command, and output are recorded before continuing. |
| A CLI matrix row disagrees with REQ-07.003 or the upgrade notes | Finding; record expected vs observed. |
| A browser matrix row disagrees with the T-114 contract | Finding; record route, status, and body evidence. |
| The upgrade scenario cannot reach a working dynamic command | Release-blocking finding; the documented migration is wrong. |
| A `deprecated/` file is read, listed, searched, or browsed | Release-blocking; the inert-folder contract fails. |
| A no-drift rule fires | Blocked until the offending line is recorded and attributed; TASK-117 does not edit it. |
| The package metadata, contents, or install differs from section 6 | Blocked; record the exact diff. |
| The host package database changes | Blocked; isolation failed. |
| The offline run needs the network | Offline claim not verified; record and use the loopback evidence. |
| A release-blocking gap remains open | `release-ready: no`; name the gap, owner, and requested decision. |
| A requirement, rule, or ADR conflict is found | Record an open question and escalate; do not edit requirements, rules, or ADRs. |

## Acceptance criteria mapping

| Check | Verifies | How it is verified |
|---|---|---|
| T-117-01 | Full suite plus the named document-type suites pass; a user-added type completes the CLI lifecycle | `cargo test` with recorded counts; `tests/release_document_types_smoke.rs` runs create/list/view/edit/status/archive/delete through the binary |
| T-117-02 | CLI help, `--json`, and exit codes `0/1/2/3/4` | Section 3 matrix, captured stdout/stderr and exit status |
| T-117-03 | Browser list/detail/edit, read-only status, filter, archived/unknown, disabled, removed-surface `404` for `/lifecycle` and `/skills`, API, assets | Section 4 matrix over HTTP |
| T-117-04 | Upgrade from a pre-change configuration follows the documented steps and ends with a working dynamic command | Section 5 scripted scenario plus the installed-binary repeat |
| T-117-05 | Package builds, installs, runs a configuration-added command offline, and removes cleanly | Section 6 in an isolated dpkg root |
| T-117-06 | No-drift rules fail on the forbidden patterns | `tests/document_type_no_drift.rs` in the passing suite |
| T-117-07 | G-1 to G-5 are verified and dispositioned; G-6 limitations are recorded | Section 8 evidence with severity and owner |
| T-117-08 | Evidence recorded and release readiness stated | `docs/packaging.md` section, ticket notes, `release-ready` line |
| T-117-09 | Repository protected and scratch removed | `git status`, diff inspection, `./.tmp/task117/` absence |

## Assumptions

- Assumption: the verification host is Debian Forky amd64 with the pinned Rust
  1.99.1 toolchain, `dpkg` 1.23.11, and `fakeroot`/`unshare` available; this
  matches the previous release-verification evidence in `docs/packaging.md`.
- Assumption: all m-17 implementation tasks (TASK-105 to TASK-116) are Done
  and are the shipped behaviour; this ticket verifies that behaviour and does
  not reopen its implementation.
- Assumption: product behaviour is unchanged. A defect is a recorded finding,
  not a fix in this ticket; fixing requires a follow-up task or an explicit
  owner decision.
- Assumption: existing suites remain authoritative for per-feature contract
  coverage; the new smoke test adds cross-surface and upgrade assertions only.
- Assumption: the user-added type is `note` (prefix `NOTE`, directory `notes/`)
  because the T-113/T-114 suites use it; no new product type is introduced.
- Assumption: historical ADRs, historical specifications, `docs/packaging.md`
  evidence, and `backlog/` records are excluded from the no-drift documentation
  scan because they record past behaviour.
- Assumption: config-defined slugs have no man page and NFR-03.003 is not
  claimed for them; the package check asserts absence, not presence.
- Assumption: the five carried gaps are findings from completed tasks
  (TASK-115/TASK-116) and are not new scope; TASK-117 verifies and reports
  them without editing their source documents.
- Assumption: the former `/lifecycle` page counted every registry descriptor
  (default 16, 17 with `note`). The page is retired per ADR-034 and `GET /lifecycle`
  returns `404`, so no page asserts a descriptor count (TASK-201).

## Open questions

Non-blocking; each has a default.

- **OQ-1 (default: release owner decides) — who accepts a release-blocking
  gap?** If G-4 or G-5 is confirmed release-blocking, TASK-117 reports
  `release-ready: no` and names the gap. The default is that only the project
  owner may accept the risk in writing; otherwise a follow-up task must be
  opened before release sign-off.
- **OQ-2 (default: record, do not fix) — should a trivially wrong README or
  upgrade-notes line be fixed inside TASK-117?** Default: no. The task is
  verification-only and the user asked not to implement; the finding is
  attributed to the owning task or release owner.
- **OQ-3 (default: HTTP/markup-level browser verification) — is a real GUI
  run required?** Default: no, consistent with prior release verification and
  NFR-05.001's recorded limitation.

## Traceability

| Artifact | Reference |
|---|---|
| Requirements | REQ-01.004, REQ-05.007, REQ-05.010, REQ-05.011, REQ-06.014, REQ-07.001, REQ-07.002, REQ-07.003, REQ-07.004 |
| Non-functional | NFR-03.002, NFR-03.003, NFR-04.002, NFR-05.001 |
| Business rules | BR-001, BR-016 |
| ADRs | ADR-0003, ADR-0016, ADR-0017, ADR-0019, ADR-0020, ADR-0021, ADR-0022, ADR-0023, ADR-0024 |
| Feature specifications verified | `search-config-defined-document-types.md`, `browser-config-defined-document-types.md`, `refactor-record-layer-for-document-archive-and-status.md`, `project-initialization-and-configuration.md` |
| User-facing documentation verified | `docs/release-1.0-upgrade-notes.md`, `README.md`, `docs/business-rules/*.md` |
| Packaging documentation | `docs/packaging.md` |
| Ticket | TASK-117; prerequisites TASK-105 to TASK-116 |
| UI mockups | none new; MOCK-0015 is the browser reference |
