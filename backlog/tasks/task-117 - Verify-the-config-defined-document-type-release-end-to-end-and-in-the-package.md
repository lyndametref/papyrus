---
id: TASK-117
title: Verify the config-defined document type release end to end and in the package
status: Done
assignee:
  - rust-dev
created_date: '2026-11-13T21:27:00Z'
updated_date: '2026-11-14T20:02:00Z'
labels:
  - rust
  - browser
  - verification
milestone: m-17
dependencies:
  - TASK-110
  - TASK-112
  - TASK-113
  - TASK-114
  - TASK-116
documentation:
  - docs/packaging.md
  - docs/requirements/business/REQ-07.003.md
  - >-
    docs/specifications/config-defined-document-types-verification-and-packaging.md
  - docs/release-1.0-upgrade-notes.md
priority: high
ordinal: 41110
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Verification and packaging gate for m-17. Twelve completed tasks (TASK-105 to TASK-116) each tested their own slice; this task proves the combined document-type change works across the CLI, search, the browser, an upgrade from a pre-change configuration, and the installed Debian package, and that the no-drift rules hold. **Verification only: do not change product behaviour, source, requirements, business rules, ADRs, README, or upgrade notes.** A defect is recorded with a disposition (follow-up, accepted risk, or release blocker), not fixed here.

**Canonical plan:** `docs/specifications/config-defined-document-types-verification-and-packaging.md`. Requirements: REQ-07.001–07.004, REQ-05.007, REQ-05.010, REQ-05.011, REQ-06.014, REQ-01.004, NFR-03.002/03.003/04.002/05.001. Rules: BR-001, BR-016. ADRs: 0016, 0017, 0019, 0020, 0021 (Proposed), 0022, 0023, 0024.

## Scope (in)

- Full `cargo test` plus the named document-type suites; record unit/integration counts.
- CLI matrix for the generated command family, generated help, `--json`, and exit codes 0/1/2/3/4.
- Browser matrix for one config-defined type: list, detail, edit, read-only status, validated filter, archived/unknown equivalence, disabled invisibility, lifecycle page, API, assets.
- One upgrade scenario from a pre-change config with `req`, `deploy`, and a `deprecated/` folder, executed from the Release 1.0 upgrade notes.
- Debian package build, metadata/content inspection, isolated install, offline run of a configuration-added command, uninstall, purge, host-state protection.
- Two new test files: `tests/release_document_types_smoke.rs` (cross-surface, upgrade) and `tests/document_type_no_drift.rs` (mechanical checks).
- Verification and dispositions for carried gaps G-1 to G-5; recording of limitations G-6.
- Evidence in `docs/packaging.md` and ticket notes.

## Scope (out)

- Any product/source edit; any edit to `README.md`, `docs/release-1.0-upgrade-notes.md`, `docs/requirements/**`, `docs/business-rules/**`, `docs/architecture/**`, or `scripts/build-deb.sh`.
- Re-testing every per-feature contract already authoritative in `tests/browser_config_types_cli.rs`, `tests/search_config_types_cli.rs`, `tests/config_list_keys_cli.rs`, `tests/cli_dispatch.rs`, and unit modules.
- New UI mockups; man-page sources or a generator; any NFR-03.003 claim for config-defined slugs (ADR-0021 stays Proposed).
- Debian releases other than Forky, architectures other than amd64, real-GUI browser runs, and a version bump.

## CLI matrix (run against `target/release/papyrus`, then the installed binary)

Fixture: user-added `note` type (`prefix: NOTE`, `directory: notes/`, `statuses: [active, archived, review, approved]`, `default_status: active`).

- C-01 `--help`: exit 0; eleven default slugs + built-ins; `note` once declared.
- C-02 `note --help`: exit 0; `create`, `list`, `view`, `edit`, `archive`, `delete`, `status`.
- C-03 `req/nfr --help`: baseline + `create-group`/`--group`; C-04 `ui-mockup --help`: baseline + `supersede`, no `status`.
- C-05 `nfreq --help`, C-06 unknown slug, C-07 disabled slug, C-08 undeclared `deploy`: exit 2.
- C-09..C-12 create (`NOTE-001`), list/view, edit, `status review`: exit 0; only supplied fields change; status stays in place.
- C-13 `status ... archived`: exit 4, names `archive`, file unchanged.
- C-14 archive: moves to `docs/notes/archived/NOTE-001.md`, drops `review`; hidden from list.
- C-15/C-16 delete without `--force` (prompt; decline = no change) and with `--force` (no prompt).
- C-17 unknown ID: exit 3. C-18 empty title/control char/path separator: exit 4, no write. C-19 unwritable dir: exit 1.
- C-20 `--json` on create/list/view/edit/status/archive/delete: one JSON value; errors JSON too.
- C-21 search by title, `--type doc`, `--json`: finds `NOTE-*` as `doc`; archived/disabled hidden.
- C-22 file under `docs/notes/deprecated/`: inert for list/view/search/browser.
- C-23 manual recovery (move to active dir, `status: active`) makes it visible; no restore command exists.
- C-24 invalid config (missing `archived`): validating commands exit 4 naming `document_types.<slug>.<field>`; `config get`/`config list` still read.
- C-25 `config set`/`unset` repair validates before write; invalid merge exits 4 and leaves the file byte-identical.
- C-26 `init --force` rewrites the eleven defaults and commands work again.

`--dir`, `--config`, and `--json` apply exactly as REQ-01.004 defines.

## Browser matrix (HTTP only; `browser --no-open --port <free>`)

- B-01 startup line exactly `Serving http://127.0.0.1:<port>/`.
- B-02 `/?type=note`: 200; `Notes` under `Documents`; shared `Status/ID/Title/Created/Updated` columns; rows match CLI.
- B-03 detail 200, status read-only; B-04 edit form has no `status`/`status_change` control.
- B-05 edit POST 303, persists via CLI, unknown section/key preserved; B-06 identical no-op save byte-identical and no `updated_date` refresh; B-07 stale version 409, file unchanged; B-08 `status=archived` POST 422, file unchanged.
- B-09 filter offers exactly `active`, `review`, `approved` in order, no `archived`; B-10 `?status=review`/`approved` filter correctly.
- B-11 `?status=archived`, `bogus`, `Review`, empty, duplicate → 400 with error page, no list.
- B-12 no `status` parameter lists custom-status records too.
- B-13 archived: omitted; detail/edit/POST/API 404; identical to unknown-ID 404.
- B-14 disabled type: no sidebar/list/detail/API; `/lifecycle` still shows `Notes` with `enabled: false`.
- B-15 lifecycle shows `Backlog`, `Decisions`, `Documents`, one section per registry descriptor (17 with `note`), custom statuses/prefix/folder/enabled, no hardcoded count.
- B-16 `/api/docs/note` shape: `id`, `kind`, `title`, `status`, `labels`, `references`, `version`, `path`, `metadata`.
- B-17 assets 200, non-empty, correct type, `Cache-Control: no-store`; B-18 no external refs, CSP present.
- B-19 inside `unshare -rn` with loopback up: outbound TCP fails; list/detail/edit/API/assets succeed.

## Upgrade scenario (scripted + repeated with the installed binary)

1. `papyrus init --no-git` under `./.tmp/task117/upgrade/`; overwrite config with pre-change `document_types.req` and `document_types.deploy`; add `docs/requirements/business/deprecated/REQ-900.md`, a `docs/deployments/` document, and an active `REQ-001`.
2. `--help` lists `req` and `deploy`; no `nfreq`.
3. `papyrus nfreq --help`, `req deprecate`, `req restore` exit 2.
4. `REQ-900` is invisible to list/search; `deprecated/` is inert.
5. Follow the documented recovery: move the file to the active directory and set `status: active`; list/view work with the same ID.
6. Break config (remove `archived` from a `statuses` list); commands exit 4 naming the key; `config get`/`list` read; repair via `config set` or documented direct edit.
7. Declare `deploy` by editing `papyrus.config.yml`; `deploy create "Release 1"` prints `DEP-001`; `deploy list` shows it; `--help` lists `deploy`.
8. End state: no `nfreq`, no static command assumption, no `deprecated/` read, one working dynamic command, fixture hash stable outside intentional edits.

## Package, install, offline, removal

1. `cargo build --release`; assert `--help` lists enabled slugs; run `./scripts/build-deb.sh` unchanged; record package size and SHA-256.
2. `dpkg-deb -I` metadata and `dpkg-deb -c` contents: only `/usr/bin/papyrus` 0755 and `/usr/share/doc/papyrus/copyright` 0644 plus dirs; copyright `GPL-3.0-or-later`.
3. Isolated install under `./.tmp/task117/{inst,adm}` with `fakeroot dpkg --instdir --admindir --force-depends`; host `dpkg -l papyrus` unchanged before/after.
4. Installed binary `root:root` 0755, on isolated `PATH`, prints version.
5. With the installed binary: create a project, declare `note` by editing config only, run create/list/view/status/archive/delete/search; `--help` lists `note`. No rebuild, no other installed file.
6. Inside `unshare -rn` (loopback up), outbound TCP fails while `papyrus --help` and `papyrus note --help` succeed.
7. Do not assert a man page for config-defined slugs; record the NFR-03.003 risk/ADR-0021 status.
8. `dpkg -r` removes the binary; `dpkg --purge` removes the admin entry and leaves no papyrus file; delete isolated roots.

## No-drift checks (new `tests/document_type_no_drift.rs`)

- D-01 literal `deprecated/` in `src/` — none allowed.
- D-02 `nfreq` command in `src/` — only the registry collision test fixture.
- D-03 document `deprecate`/`restore` variants in `src/` — none.
- D-04 static per-type document command variant/list in `src/cli/` — none; registry descriptors only.
- D-05 slug-literal behaviour dispatch in `src/cli/dispatch.rs`, `src/cli/tree.rs`, `src/document.rs`, `src/search.rs`, `src/browser/` — allowed only in the type's own content module and `src/registry.rs`.
- D-06 `papyrus nfreq`, per-type document command block, or `deprecated/` mention in `README.md`, `docs/release-1.0-upgrade-notes.md`, `docs/business-rules/*.md` — only removal/migration and inert-folder wording.
- D-07 static document-type inventory in the two config-type specifications — none.
- Historical ADRs (ADR-0010/0022/0023), historical specifications, `docs/packaging.md` evidence, and `backlog/` records are excluded.

## Carried gaps to verify or explicitly accept

- **G-1** `config set` cannot bootstrap a new type (TASK-116 note 2): single field write fails validation and nothing is written; mapping values refused. Verify the documented direct-edit workaround and that the error names the missing field. Default: accepted limitation; follow-up only if the error is unclear. Not release-blocking.
- **G-2** Upgrade notes line 148 says task IDs allocate from "active, archived, and deprecated locations". Verify allocation reads only active + archived and record the wording discrepancy; do not edit the notes. Default: docs finding; not release-blocking.
- **G-3** README Milestones says `active -> archived`, but `src/registry.rs` `MILESTONE_STATES` is `["open","closed"]` and REQ-05.010 fixes `open`/`closed`. Verify shipped behaviour; record stale wording. Default: README finding; not release-blocking for this gate.
- **G-4** `config list` omits `document_types.<slug>.default_status` (`src/config.rs` flattened). Verify `config get`, `config set/unset` validation, and the documented repair path. Default: release-blocking only if an invalid `default_status` cannot be inspected or repaired; otherwise follow-up for TASK-111's owner.
- **G-5** BR-007 documents `papyrus release close` as the only `planned -> released` transition, but `src/release.rs` has no `Close` variant and the command exits 2; browser text also differs. Default: release-blocking for the m-17 gate unless a follow-up code task is opened or the owner accepts amending BR-007. Record only.
- **G-6** Limitations: no real-GUI browser run; Forky amd64 only; no man page for config-defined slugs (NFR-03.003 gap); packaging script does not force a rebuild (mitigate by building first and asserting `--help`).

## Evidence

- Add `## TASK-117 — Config-defined document types release verification and packaging` to `docs/packaging.md`: date/timezone, environment, isolation, test counts, package name/size/SHA-256, quality gates, install/offline/removal table, findings G-1..G-5 with severity/owner/disposition, limitations G-6.
- Ticket notes: matrix results, upgrade transcript, no-drift result, exact front-matter-only search query, package SHA-256, installed version, and an explicit `release-ready: yes|no` line with blockers.
- Remove all `./.tmp/task117/` scratch, including on failure; keep the built `.deb` and evidence.
- Final `git status` lists only the new spec, the two new tests, `docs/packaging.md`, and TASK-117 metadata.

## Assumptions

- Assumption: verification host is Debian Forky amd64 with Rust 1.99.1, `dpkg` 1.23.11, `fakeroot`, and `unshare`.
- Assumption: all m-17 implementation tasks are Done and are the shipped behaviour; this task does not reopen them.
- Assumption: product behaviour is unchanged; defects are recorded, not fixed.
- Assumption: existing per-feature suites stay authoritative; the new smoke test adds cross-surface and upgrade coverage only.
- Assumption: the fixture type is `note` (`NOTE`, `notes/`), matching the T-113/T-114 suites; no product type is added.
- Assumption: historical ADRs, historical specifications, `docs/packaging.md` evidence, and `backlog/` records are excluded from the documentation no-drift scan.
- Assumption: config-defined slugs have no man page; the package check asserts absence, not presence.
- Assumption: the five carried gaps are findings from TASK-115/TASK-116, not new scope.

## Dependencies

TASK-105 to TASK-116 are completed (Backlog graph); TASK-117 is the only remaining m-17 task. No new dependency is added.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The full Rust suite passes with the named document-type suites (cli_dispatch, registry, config, document, browser_config_types_cli, search_config_types_cli, config_list_keys_cli, cli, no_drift) and unit/integration counts are recorded; the new `tests/release_document_types_smoke.rs` drives a user-added `note` type through create, list, view, edit, `status`, archive, and delete via the CLI with exit 0 and persisted state.
- [x] #2 The CLI matrix (C-01..C-26) is executed and recorded: `--help` lists enabled slugs; `<slug> --help` lists baseline verbs plus type extensions; unknown, disabled, `nfreq`, and undeclared `deploy` exit 2; `--json` is accepted on every document verb; exit codes 0/1/2/3/4 are each observed; `status ... archived` exits 4 and names `archive`; delete prompts without `--force` and skips it with `--force`.
- [x] #3 The browser matrix (B-01..B-19) is executed over HTTP: `/?type=note` list, `/doc/NOTE-001` detail, edit round trip with 303, read-only status, filter offering exactly active/review/approved plus 400 rejection of archived/bogus/wrong-case/empty/duplicate, archived/unknown identical 404, disabled invisibility with the lifecycle page still listing the entry, `/api/docs/note` shape, assets, and offline loopback.
- [x] #4 The upgrade scenario starts from a pre-change configuration with `req`, `deploy`, and a `deprecated/` folder; follows the documented repair and migration steps; and ends with no `nfreq`, no static command assumption, no `deprecated/` read, and a working `papyrus deploy` command from configuration alone, with the fixture unchanged outside intentional edits.
- [x] #5 The Debian package is built from a fresh release build, its metadata/contents and SHA-256 are recorded, it installs in an isolated dpkg root, the installed binary runs `papyrus note ...` and `papyrus --help` offline with `note` listed, no man page is asserted for config-defined slugs, the NFR-03.003 risk is recorded, and uninstall/purge leave the host unchanged.
- [x] #6 `tests/document_type_no_drift.rs` fails the suite on any D-01..D-07 pattern (literal `deprecated/` in `src/`, `nfreq` command, document `deprecate`/`restore` variants, static per-type command list, slug-literal behaviour dispatch outside the type module/registry, stale per-type inventory or `deprecated/` mention in README/upgrade notes/business rules, static inventory in the config-type specs), with the documented historical exclusions.
- [x] #7 Each carried gap G-1 to G-5 is reproduced or refuted with command output, assigned a severity and an owner, and given a disposition (accepted limitation, follow-up, or release blocker); G-6 limitations are recorded; no gap is fixed by editing source or documents in this task.
- [x] #8 `docs/packaging.md` contains a TASK-117 section with date/timezone, environment, isolation, test counts, quality gates, package name/size/SHA-256, install/offline/removal results, findings, and limitations; the ticket notes carry the matrices, upgrade transcript, no-drift result, and an explicit `release-ready: yes|no` statement with any blockers.
- [x] #9 The change set is limited to the new specification, the two new test files, `docs/packaging.md`, and TASK-117 metadata: no `src/`, requirement, business-rule, ADR, README, upgrade-notes, or packaging-script edit; all `./.tmp/task117/` scratch is removed; the final `git status` shows only the expected paths.
- [x] #10 `cargo build --release`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, and `cargo fmt --check` all pass, and the release gate result (ready or not ready, with blocking gap IDs) is reported to the release owner.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Every acceptance criterion is checked with its command output, exit status, HTTP status, or file assertion recorded in the ticket notes; no criterion is checked on inference alone.
- [x] #2 The full `cargo test` run, clippy, fmt, and release build pass on the pinned toolchain, and the test counts plus the two new test files are named in the evidence.
- [x] #3 The package is built, inspected, installed in an isolated root, exercised offline with a configuration-added command, removed and purged, and its size/SHA-256 and the unchanged host state are recorded.
- [x] #4 All five carried gaps have a recorded disposition and owner, the G-6 limitations are recorded, and the `release-ready` statement names every blocker or states that none is open.
- [x] #5 `docs/packaging.md` and the ticket notes contain the evidence in the specified structure, including the exact search query, package SHA-256, installed version, and the no-drift result.
- [x] #6 No product source or governed document is edited; the diff is limited to the spec, the two tests, `docs/packaging.md`, and the ticket; `./.tmp/task117/` is gone and the built `.deb` remains.
<!-- DOD:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Baseline and environment: record host (Debian Forky amd64, glibc, rustc/cargo 1.99.1, clippy, dpkg, fakeroot, unshare), `git status --porcelain`, and the current `papyrus --help`. Read `docs/release-1.0-upgrade-notes.md`, `docs/packaging.md`, and the two config-type specifications. Create `./.tmp/task117/` for all scratch.

2. Suites: run `cargo build --release`, then `cargo test` (record unit and integration counts), `cargo test --test cli_dispatch --test browser_config_types_cli --test search_config_types_cli --test config_list_keys_cli --test no_drift`, `cargo clippy --all-targets -- -D warnings`, and `cargo fmt --check`. Record any failure before proceeding.

3. Write `tests/release_document_types_smoke.rs`: create a `note` project from a written config (prefix NOTE, directory notes/, statuses active/archived/review/approved, default active); assert the CLI lifecycle and exit matrix (C-09..C-20); assert the upgrade scenario (pre-change config with req + deploy + deprecated folder, documented repair/recovery, working deploy); assert a front-matter-only search query and archived/disabled invisibility. Use a self-removing `Scratch` guard under `./.tmp/task117/`.

4. Write `tests/document_type_no_drift.rs`: production-line scanner (reuse the `#[cfg(test)]` skip pattern from `tests/no_drift.rs`) for D-01..D-05; documentation scanner for D-06..D-07 with the historical exclusions in section 7 of the spec. Add a scanner self-test proving a planted violation fails and an allowlisted line passes.

5. Browser matrix: start the built binary with `browser --no-open --port <free>` in the `note` project and run B-01..B-18 with `curl`/the test HTTP helper; record status codes and key body assertions. Run B-19 inside `unshare -rn` (loopback up), confirming an outbound TCP failure.

6. Upgrade transcript: execute section 5 literally by hand in `./.tmp/task117/upgrade/` (or replay the smoke test commands), capturing each command and result. Compare the observed steps with the upgrade-notes text; record G-1, G-2, and any wording mismatch.

7. Gap verification: reproduce G-3 (milestone archive front matter vs README/registry), G-4 (`config get`/`list`/`set`/`unset` with default_status), and G-5 (`papyrus release close` exit 2; inspect `src/release.rs` and the browser release text). Record severity, owner, and disposition for each.

8. Package: `cargo build --release`; assert `--help` lists the enabled slugs; `./scripts/build-deb.sh`; record size/SHA-256; `dpkg-deb -I` and `-c`; isolated `fakeroot dpkg` install under `./.tmp/task117/{inst,adm}`; verify owner/mode/version; declare `note` by editing config only and run create/list/view/status/archive/delete/search plus `--help`; run `--help` and `note --help` inside `unshare -rn`; `dpkg -r`, `dpkg --purge`; record host state before/after.

9. Evidence and cleanup: append the TASK-117 section to `docs/packaging.md` with the required structure; write the ticket notes with matrices, transcript, no-drift result, findings, and `release-ready`; remove `./.tmp/task117/`; confirm `git status` shows only the spec, the two tests, `docs/packaging.md`, and TASK-117 metadata; report to the release owner.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
## TASK-117 verification evidence (2026-11-14 20:53 CET / UTC+01:00)

Release readiness: **release-ready: no** — blocking gap **G-5** (release `close` documented but not implemented). All other gaps are recorded follow-ups or accepted limitations.

### Suites and gates
- `cargo test`: 789 passed, 0 failed, 22 ignored (42 binaries; 299 library unit tests).
- Named suites: `cli_dispatch` 9, `browser_config_types_cli` 19, `search_config_types_cli` 13, `config_list_keys_cli` 9, `cli` 24, `no_drift` 10, new `release_document_types_smoke` 8, new `document_type_no_drift` 8.
- `cargo build --release` PASS; `cargo clippy --all-targets -- -D warnings` PASS (0 warnings); `cargo fmt --check` PASS.
- New tests: `t117_01_generated_help_and_dispatch_matrix` .. `t117_08_browser_matrix_over_http`; `d01_no_deprecated_folder_literal_in_src` .. `d07_no_static_inventory_in_config_type_specifications` plus the scanner self-test.

### CLI matrix C-01..C-26 (release binary, `note` fixture NOTE/notes/ statuses active,archived,review,approved)
- C-01 help exit 0: eleven defaults + note; no deploy/nfreq. C-02 note help lists create/list/view/edit/archive/delete/status. C-03 req/nfr help: create-group and create --group. C-04 ui-mockup help: supersede, no status.
- C-05 nfreq, C-06 unknown slug, C-07 disabled note, C-08 undeclared deploy: exit 2.
- C-09 create exit 0 prints NOTE-001 and writes docs/notes/NOTE-001.md. C-10 list/view exit 0, ID order, metadata/body match. C-11 edit exit 0, only title changed. C-12 status review exit 0, status: review in place.
- C-13 status archived exit 4, stderr names archive, SHA-256 unchanged. C-14 archive exit 0, moves to docs/notes/archived/NOTE-001.md, review discarded, hidden from list.
- C-15 delete prompt: `n` leaves the file, `y` removes it. C-16 delete --force exit 0, no prompt, file gone. C-17 unknown ID exit 3.
- C-18 empty/newline/CR/path-separator titles exit 4 with no write (see F-117-1 for TAB/0x01). C-19 unwritable directory exit 1 (`Permission denied (os error 13)`).
- C-20 --json on create/list/view/edit/status/archive/delete: one JSON value each; error JSON `{"error":{"code":3,...}}` on stderr for unknown view.
- C-21 search by title, --type doc, --json: NOTE-* as doc; archived/disabled hidden. C-22 deprecated/ file not listed/viewed/searched. C-23 manual move + status active makes NOTE-900 visible; no restore command (exit 2).
- C-24 broken statuses exit 4 naming `document_types.note.statuses`; config get/list read. C-25 invalid set exit 4 with byte-identical file (same SHA-256); valid set repairs. C-26 init --force rewrites eleven defaults; req works, undeclared note exit 2.

### Browser matrix B-01..B-19 (HTTP)
- B-01 exactly `Serving http://127.0.0.1:<port>/`. B-02 200, Notes under Documents, Status/ID/Title/Created/Updated order, rows match CLI. B-03 detail 200 read-only status. B-04 edit form has no status/status_change controls.
- B-05 POST 303 to /doc/NOTE-003, persists through CLI, unknown key/section preserved. B-06 identical save byte-identical (SHA-256 unchanged). B-07 stale version 409, unchanged. B-08 status=archived 422, unchanged.
- B-09 filter exactly active,review,approved; no archived. B-10 review/approved filter correctly. B-11 archived/bogus/Review/empty/duplicate each 400 with error page and no list. B-12 no status lists all active records.
- B-13 archived omitted; detail/edit/POST/API 404; 404 page identical to unknown NOTE-999; deprecated NOTE-900 also 404. B-14 after restart disabled: list/detail/edit/API 404, no sidebar entry, /lifecycle still shows Notes enabled: false.
- B-15 lifecycle Backlog/Decisions/Documents with 17 enabled sections and Notes custom statuses/prefix/folder. B-16 /api/docs/note carries id,kind,title,status,labels,references,version,path,metadata. B-17 app.css/app.js 200 non-empty correct type Cache-Control no-store. B-18 CSP default-src none present, no external http(s) refs.
- B-19 `unshare -rn` loopback up: outbound TCP to 1.1.1.1:80 fails (exit 1); list/detail/edit/API/assets all 200 over loopback.

### Upgrade transcript (pre-change config: req + deploy, deprecated/REQ-900, active REQ-001, docs/deployments/DEP-001)
1. `--help` lists req and deploy, not nfreq. 2. `nfreq --help`, `req deprecate`, `req restore` exit 2. 3. `req list` omits REQ-900; search marker empty; `deploy list` shows DEP-001.
4. Move REQ-900.md to docs/requirements/business/ and set status: active; `req list`/`req view REQ-900` work with the same ID. 5. Break statuses -> exit 4 naming document_types.req.statuses; `config get`/`list` read; `config set` repairs.
6. Remove deploy -> `deploy --help` exit 2; declare deploy by editing papyrus.config.yml only -> `deploy create "Release 1"` prints DEP-001, list shows it, help lists deploy. Fixture byte-identical outside intentional edits.
- Front-matter-only search query: `fm-only-label` -> `NOTE-001 doc Wallet note`; project hash unchanged.

### Package, install, offline, removal
- Package `papyrus_0.1.0_amd64.deb`, size 1,284,160 bytes, SHA-256 `6028cc9e79f112d03a0c2e2b1a3d423997920ec6232f723a48ac45516494486d`.
- `dpkg-deb -I`: papyrus 0.1.0 amd64, libc6 (>= 2.34), utils/optional, Papyrus Maintainers. `-c`: only /usr/bin/papyrus 0755 and /usr/share/doc/papyrus/copyright 0644 (GPL-3.0-or-later) plus dirs.
- Isolated `fakeroot dpkg --instdir/--admindir` install exit 0; installed file root:root 0755/0644; installed binary prints `papyrus 0.1.0`.
- Installed binary: declare note by editing config only; `--help` lists note; create/list/view/status/archive/delete/search pass. No rebuild, no other installed file.
- Offline `unshare -rn` (loopback up): outbound TCP fails; `--help`, `note --help`, `note list`, browser list/detail/edit/API/assets succeed.
- No man page asserted for config-defined slugs (NFR-03.003 gap; ADR-0021 Proposed). `dpkg -r` removes both files; `--purge` removes the admin entry; host `dpkg -l papyrus` unchanged before/after.
- No-drift D-01..D-07 PASS including scanner self-test and historical exclusions.

### Findings / dispositions
- G-1 Low, TASK-111 owner: `config set` cannot bootstrap a type; error names missing `document_types.newtype.directory`, file unchanged; direct edit works. Accepted limitation.
- G-2 Low, upgrade-notes owner: line 148 says active+archived+deprecated but allocator reads active+archived only; planted deprecated TASK-999 did not change TASK-005 allocation. Documentation finding.
- G-3 Low, README owner: README says milestones active->archived; registry is open/closed and shipped milestones carry no status; archive moves the file. Stale wording.
- G-4 Medium, TASK-111 owner: `config list` omits document_types.<slug>.default_status; `config get` returns it and `config set` repairs an invalid value. Follow-up, not blocking.
- G-5 High, release owner/BR-007 owner: `papyrus release close REL-001` exits 2 because `ReleaseCmd` has no Close variant; browser lifecycle text names a non-existent `papyrus release status <id> released`. Planned->released is unreachable. **Release-blocking.**
- F-117-1 Low, REQ-01.004/TASK-110 owner: `sanitize_name` rejects NUL/CR/LF/path separators but accepts TAB 0x09 and 0x01 in titles. Follow-up input-validation hardening; not blocking.
- G-6 limitations recorded: no real-GUI run; Forky amd64 only; no man page for config-defined slugs; packaging script does not force a rebuild (build first and assert --help); `dpkg -r` removes both files so `--purge` reports already removed.
<!-- SECTION:NOTES:END -->

## Comments

<!-- COMMENTS:BEGIN -->
author: @task-refiner
created: 2026-11-14T19:36:00Z

---

Refined for execution. Added `docs/specifications/config-defined-document-types-verification-and-packaging.md` (verification matrices, upgrade scenario, package procedure, no-drift rules, gap dispositions). Rewrote the description with scope in/out, CLI C-01..C-26 and browser B-01..B-19 matrices, the upgrade/package/no-drift procedure, and the five carried gaps G-1..G-5 plus limitations G-6. Replaced the 7 loose ACs with 10 testable ACs and added 6 DoD items and a 9-step plan. No UI design needed; MOCK-0015 remains the browser reference.

Assumptions recorded: Debian Forky amd64 with Rust 1.99.1; no product/document edits in this task; historical ADRs/specs and backlog records excluded from the documentation no-drift scan; `note` is the fixture type.

Open questions (non-blocking, defaults stated): who accepts a release-blocking gap (default: project owner in writing); whether a trivially stale README/upgrade-notes line may be fixed here (default: no, record only); whether a real-GUI browser run is required (default: no). Execution-ready: yes. Release readiness may still be `no` if G-4 or G-5 is confirmed release-blocking.

---
<!-- COMMENTS:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
End-to-end verification complete: 789 tests pass (0 failed, 22 ignored) including new release_document_types_smoke (8) and document_type_no_drift (8); CLI matrix C-01..C-26 and browser matrix B-01..B-19 executed; upgrade scenario from a pre-change config with req/deploy/deprecated works; Debian package built (1,284,160 bytes, SHA-256 6028cc9e...4486d), installed in isolation, run offline, removed; no man page for config-defined slugs and NFR-03.003 recorded as a gap. Evidence in docs/packaging.md and ticket notes. release-ready: no — G-5 (papyrus release close missing, BR-007 documents it) is release-blocking; G-1/G-2/G-3/F-117-1 low, G-4 medium, G-6 limitations recorded.
<!-- SECTION:FINAL_SUMMARY:END -->
