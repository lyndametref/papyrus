---
id: TECH-065
title: "Release 1.0 Native Layout, Import, Relocation, and Single ID Space Verification and Packaging"
created_date: '2026-11-16T17:46:53Z'
updated_date: '2026-11-18T22:50:56Z'
labels: []
references:
  - ADR-0001
  - ADR-0003
  - ADR-0004
  - ADR-0005
  - ADR-0006
  - ADR-0010
  - ADR-0011
  - ADR-0012
  - ADR-0013
  - MOCK-0002
  - NFR-03.002
  - NFR-04.001
  - NFR-04.002
  - NFR-04.004
  - NFR-05.001
  - NFR-06.001
  - NFR-06.002
  - REQ-01.002
  - REQ-01.003
  - REQ-01.004
  - REQ-01.005
  - REQ-05.007
  - REQ-05.010
  - REQ-05.011
  - REQ-06.002
  - REQ-06.004
  - REQ-06.008
  - REQ-06.013
  - TASK-1
  - TASK-100
  - TASK-102
  - TASK-103
  - TASK-4
  - TASK-73
  - TASK-83
  - TASK-88
  - TASK-89
  - TASK-90
  - TASK-92
  - TASK-97
  - TASK-98
  - TASK-99
  - ADR-034
  - REQ-06.018
  - RE-005
status: draft
---
# TECH-065 Release 1.0 Native Layout, Import, Relocation, and Single ID Space Verification and Packaging

> **Revised (TASK-203, 2026-11-18):** the browser configuration-view clauses — the relocation scope bullet, section 7 item 4, and the `browser view equivalence` part of the T-98-08 verification row — are retired per ADR-034 and REQ-06.018 (RE-005): the browser serves no configuration view and configuration editing is CLI-only. The CLI-only `config set` relocation and installed-binary verification clauses (REQ-01.002) remain current. REQ-06.008, which the retired clauses cite, is archived; the retired text is retained below as history. See TECH-080.

## Description
## Purpose

Deliver ADR-0013 **stage 8**: prove end to end that the combined properties the Backlog.md renunciation depends on actually hold, then produce and verify the installable release package. The individual implementation tasks tested their own behaviour; this ticket tests the whole: adoption idempotency and byte identity, no Backlog-shaped write anywhere, one Papyrus identity on every surface, import fidelity against a real sample store, relocation that never loses a folder, and search that still indexes whole persisted text now that front matter is part of it. It closes the ADR-0013 stage 8 gate on the release that claims NFR-04.004 and produces the amd64 Debian package required by NFR-03.002.

This is a verification and packaging ticket. Product behaviour must not change; only a minimal defect fix is permitted when a verification step proves a defect, and such a fix is reported separately in the ticket notes.

## Scope

### In scope

- An isolated sample Backlog.md project fixture, generated for the run under `./.tmp/`, covering active and archived tasks and milestones, `docs`, `drafts`, `decisions`, `config.yml`, `.locks`, every body marker family, an unknown section, and free text.
- Adoption idempotency and byte identity: repeated adoption and no-op or equal-value writes leave the file byte-, mode-, and timestamp-identical.
- The no-Backlog-shape proof: behavioural checks over the exercised lifecycle plus the mechanical `tests/no_drift.rs` checks.
- Invisibility of an unimported Backlog.md store on every surface, and full readability after `papyrus import backlog`.
- Single-ID proof for CLI text, CLI `--json`, browser JSON, browser HTML, and browser URLs, including rejection of a Backlog ID as an unknown record.
- Import fidelity, excluded scope, and the documented upgrade path, verified against the sample fixture only.
- Relocation atomicity and conflict behaviour, re-verified through the installed binary; **retired (TASK-203, 2026-11-18, ADR-034, REQ-06.018):** the check that the browser configuration view reaches the same `config set` path — the browser serves no configuration view, so only the CLI `config set` relocation path is re-verified.
- Search indexing whole persisted text including front matter, and correspondence with the documented search behaviour.
- Rust quality gates on the pinned toolchain, and the isolated Debian Forky amd64 package build, install, smoke, uninstall, and purge.
- One new automated integration test file, `tests/release_native_layout_smoke.rs`, for the parts that can be asserted against the built binary.
- Verification evidence in `docs/packaging.md` and in the TASK-98 ticket notes.

### Out of scope

- Importing this repository's own backlog store. The store stays in the Backlog.md layout after this ticket (deferred by the project owner; TASK-100 is archived). No path under the repository `backlog/` is read as an import source or written by verification.
- Changing requirements, business rules, ADRs, packaging scripts, or Debian control metadata.
- Authoring the Release 1.0 upgrade notes and release summary (TASK-99) and closing the release (TASK-73).
- Version bumping. The package carries the `Cargo.toml` version unchanged (open question OQ-2).
- Debian releases other than Forky, and architectures other than amd64 (ADR-0003, NFR-03.002).
- Real-GUI browser runs in Firefox, Edge, or Safari; verification is markup, asset, and HTTP level (OQ-1).
- New UI mockups or design work.
- Repeating per-feature contract coverage already authoritative in `tests/*_cli.rs`, `tests/*_smoke.rs`, `tests/no_drift.rs`, `tests/config_relocation_cli.rs`, and `tests/import_backlog_cli.rs`.
- New runtime dependencies; the TASK-1/TASK-4 dependency-license audit remains the reference (NFR-06.002).

## Behavior

### 1. Isolated fixture and isolation rules

1. The sample Backlog.md project is created for the run, either by `tests/release_native_layout_smoke.rs` or by a scripted manual procedure, under `./.tmp/task98/source/` (project convention: no system temporary directories). It is removed when the task ends, including on failure.
2. The fixture contains, at minimum: `backlog/tasks/task-1 … task-5.md` (including one record with `dependencies` on another and one with a `milestone` reference), `backlog/archive/tasks/`, `backlog/milestones/m-0 … m-2.md`, `backlog/archive/milestones/`, `backlog/docs/`, `backlog/drafts/`, `backlog/decisions/`, `backlog/config.yml`, and `backlog/.locks/`.
3. Fixture task bodies exercise `SECTION:DESCRIPTION`, `AC` with `#N` ordinals and both checkbox states, `DOD`, `SECTION:PLAN`, `SECTION:NOTES`, `COMMENTS`, one unknown section, and free text outside any marker block. Milestone bodies carry one `## Description`.
4. Front matter on fixture records includes `title`, `status` (including one value that does not match the target vocabulary), `assignee`, quoted ISO-8601 timestamps, `labels`, `milestone`, `dependencies`, `references`, `documentation`, `type`, `ordinal`, and one unknown key.
5. The upgrade scenario is reproduced inside one isolated project: `papyrus init` creates the target, the fixture store is copied to its configured `backlog/` directory, and the import is run with `--from` pointing at that same isolated project. No repository `backlog/` content participates.
6. Before and after the run the ticket records `git status --porcelain backlog/` and a recursive content hash of the repository `backlog/`; both must be unchanged, and no file under `backlog/` may be added, removed, or rewritten.
7. The source tree hash is recorded before the import and compared afterwards; the source must be byte-identical, and `--dry-run` must write nothing anywhere.

### 2. Adoption idempotency and byte identity

1. A record that already carries front matter is read twice and written with a patch whose values equal the current ones; the file's bytes, mode, and modification time are unchanged, and `updated_date` is not refreshed.
2. Adoption read of a legacy-shaped record without front matter (document record type, task, and milestone) is byte-identical: the front-matter block is added and every remaining body byte is unchanged; the file name and ID are unchanged by adoption.
3. The first write to a legacy record retires the legacy title heading and metadata sections per ADR-0012 decision 4 (the documented TASK-89 behaviour), so the written file is native front-matter-only; every subsequent read and equal-value write of that record is byte-identical, with mode and modification time unchanged and `updated_date` not refreshed.
4. An equal-value write to a record that already carries native front matter (never legacy-shaped) is byte-identical in the same sense.
5. A read that cannot adopt safely (read-only file or directory) reports the error, leaves the file byte-identical, and keeps the record readable from its existing text.
6. The same checks are repeated with the installed binary from the built package.

### 3. No Backlog-shaped write

1. After the exercised flows (create, edit, adopt, archive, restore, deprecate, restore, import), every written record path matches the native layout: `<ID>.md`, decisions `<ID>-<kebab-slug>.md`, documents `<ID>.md`.
2. No written file has a Backlog-shaped stem (`task-<n>`, `m-<n>`), no task or milestone regains a legacy `## Status`-style metadata section, and no file gains a storage-ID key, storage path, offset-derived number, or lower-case storage prefix.
3. Imported records are ordinary native records: their front-matter `id`, file name, CLI output, and browser surfaces all carry the allocated Papyrus ID, and no source ID is persisted.
4. The eight mechanical checks of `tests/no_drift.rs` pass in the same `cargo test` run, and the evidence records that they were part of the passing suite.

### 4. Invisibility before import and readability after

1. With an unimported Backlog.md store inside an initialised project, `task list`, `task view`, `milestone list`, `search`, their `--json` forms, the browser backlog view, the browser JSON API, and the record views report no tasks and no milestones; no Backlog ID resolves.
2. Records of other families in the same project remain visible, proving the invisibility is specific to unrecognized Backlog stems, not a general failure.
3. After `papyrus import backlog --from <isolated project>`, every in-scope source record is readable through CLI and browser with its allocated Papyrus ID; archived source records land in the configured archive directories.
4. `docs`, `drafts`, and `decisions` are reported as excluded, are not read as records, and are not imported; `config.yml` and `.locks` are not written.
5. The source store is byte-identical after the import and after the subsequent CLI and browser checks.

### 5. Single ID space on every surface

1. After import, `task list`, `task view <ID>`, `milestone list`, `search`, and `import`-produced record views contain only Papyrus IDs; no `task-<n>` or `m-<n>` form appears.
2. `--json` output for list, view, and search carries the Papyrus ID in `id` and in `metadata.id`; no `storage_id`, `display_id`, `storage_path`, or equivalent field is present.
3. Browser HTML list and record pages, `/api/tasks`, `/api/milestones`, `/api/doc/<ID>`, and every rendered link use the Papyrus ID; a rendered link never contains a Backlog stem.
4. A Backlog ID passed as input (`task view task-1`, `milestone view m-0`) exits 3 as an unknown record; no surface accepts or translates it.
5. The checks assert on captured output rather than on a manual reading, so the test fails if a non-Papyrus ID form reappears.

### 6. Import fidelity, excluded scope, and upgrade path

1. Allocation: per family, imported IDs are consecutive from 1 in ascending source-number order, unique, with milestones in the `MS-` space starting at 1 and no source number preserved.
2. Field mapping: title, assignee list, valid quoted timestamps, labels, references, documentation, type, ordinal, and the unknown key are preserved; status adopts the configured spelling on a normalized match and is preserved verbatim with a warning otherwise; `milestone` and `dependencies` are translated through the plan with warnings for unresolvable entries; no milestone `release` value is written.
3. Body mapping: `## Description`, `## Acceptance Criteria` with `#N` ordinals dropped and checkbox states kept, `## Definition of Done`, `## Implementation Notes`, `## Notes`, `## Comments`; marker comments removed; unknown sections and free text appended verbatim; milestones keep a single `## Description`.
4. Destinations: active source records under the configured active directories, archived source records under the configured archive directories, and no overwrite, merge, or renumbering of an existing record.
5. Excluded scope: `docs`, `drafts`, and `decisions` are counted and reported as excluded, and no record from them exists afterwards.
6. Upgrade path: the documented one-time import procedure is executed literally against the isolated project — `init`, dry run, real import with a saved plan/report, then normal CLI and browser use. The observed outcome matches the documented claims: no automatic import, IDs and milestone numbers reallocated, source store kept, and a second run refused without `--allow-existing` and duplicating with it.
7. Any divergence between the documented procedure and observed behaviour is recorded as a finding in the ticket notes and reported to the TASK-99/TASK-73 owner; this ticket does not edit those documents.

### 7. Relocation atomicity and conflict behaviour

1. `config set backlog_directory <new>` moves every file and subfolder of the source folder, including unmanaged files, and removes the source folder when empty.
2. A destination conflict, an unwritable destination, or a destination outside the project exits 4, moves nothing, leaves both trees intact, and persists no configuration value; the configuration file is byte-identical.
3. A multi-segment destination is accepted and resolved relative to the project directory; every resolved path stays inside the project.
4. **Retired (TASK-203, 2026-11-18, ADR-034, REQ-06.018):** the browser configuration view reaches the same relocation path: the same move performed through the view produces the same on-disk result and configuration value as the CLI, and the view never moves a folder itself. No browser configuration view exists; items 1-3 and 5 remain current for the CLI `config set` path.
5. Success and conflict are re-verified through the installed binary; `tests/config_relocation_cli.rs` remains authoritative for the exhaustive case matrix.

### 8. Search over whole persisted text (CLI) and browser record exposure

1. `papyrus search` matches case-insensitive substrings of the whole persisted record text, including front matter, so a query matching only a metadata value (for example a label, an assignee, or a title stored in front matter) returns the record in the CLI text and `--json` output.
2. Search is a CLI surface only: the browser exposes no search endpoint, page, or query parameter. The browser half of this check is the record view and the JSON API returning the record that the CLI query found, under its Papyrus ID.
3. Quoted phrases, `--type` filters, `--limit`, `--json`, empty results, and deprecated/archived visibility behave as documented in `search-complete-unified.md` and the per-type search specifications.
4. A search run writes no file: the project hash is unchanged afterwards.
5. The evidence records the exact front-matter-only query used, so the claim is reproducible.

### 9. Rust quality gates

1. `cargo build --release` succeeds on the pinned toolchain (`rust-toolchain.toml`, Rust 1.99.1 at refinement).
2. `cargo test` passes the whole suite, including `tests/no_drift.rs`, `tests/config_relocation_cli.rs`, `tests/import_backlog_cli.rs`, and the new `tests/release_native_layout_smoke.rs`; the resulting test count is recorded.
3. `cargo clippy --all-targets -- -D warnings` reports no warning, and `cargo fmt --check` reports no diff.
4. `target/release/papyrus --help` lists the documented command set including `import` and `browser`; `--version` prints the Cargo package version.

### 10. Debian package, install, and installed smoke

1. `./scripts/build-deb.sh` is run unchanged after `cargo build --release` and produces `papyrus_<version>_amd64.deb` in the project root; the package size and SHA-256 are recorded.
2. `dpkg-deb -I` reports `Package: papyrus`, `Version: <Cargo version>`, `Architecture: amd64`, `Depends: libc6 (>= 2.34)`, `Section: utils`, `Priority: optional`, and a maintainer value; the packaged copyright declares `GPL-3.0-or-later`.
3. `dpkg-deb -c` lists only `/usr/bin/papyrus` (0755) and `/usr/share/doc/papyrus/copyright` (0644) plus their directories; no asset, template, configuration, or man file is packaged.
4. Install runs in an isolated dpkg root under `./.tmp/task98/{inst,adm}` using `fakeroot dpkg --instdir --admindir --force-depends`, so the host package database and filesystem are never modified; the host `dpkg -l papyrus` state is recorded before and after and is unchanged.
5. The installed `/usr/bin/papyrus` is `root:root` 0755, resolves on the isolated `PATH`, and prints the package version.
6. With the installed binary: `init` an isolated project, reproduce the invisibility and import scenario of sections 4–6, then serve the browser and check the backlog board (per-milestone sections and status columns), a task record view, a board status change persisted to the record, the JSON API, and both embedded assets over loopback.
7. Offline check: inside `unshare -rn` with loopback up, an outbound TCP connection fails while browser list, record view, status change, API, and assets succeed; if the namespace is unavailable, the limitation is recorded and the loopback/no-external-reference evidence is used instead.
8. `dpkg -r` removes the binary from the isolated `PATH`; `dpkg --purge` removes the admin entry and leaves no papyrus file; the isolated roots are then deleted.

### 11. Evidence and repository protection

1. `docs/packaging.md` gains a TASK-98 section recording: environment (Debian Forky amd64, glibc, `rustc`/`cargo`, clippy, `dpkg`, `fakeroot`, `unshare` availability), date with timezone, isolation method, test count, package name/size/SHA-256, quality-gate table, install/uninstall/smoke table, and recorded limitations.
2. The ticket notes carry the same evidence summary plus the fixture hash checks, the front-matter-only search query, the relocation outcomes, and every finding raised against another ticket.
3. The section states that Debian Forky amd64 is supported and verified, and that Trixie and other releases are not claimed or verified (ADR-0003, NFR-03.002).
4. Recorded limitations reuse the established pattern: no real-GUI browser run (OQ-1), packaging script does not force a rebuild (mitigated by building first and asserting `--help`), and any environment that could not be exercised.
5. The repository backlog store is confirmed untouched (section 1.6) and this is stated in the evidence.
6. Every scratch directory, isolated dpkg root, staged package tree, and temporary fixture created under `./.tmp/` is removed before the task ends, including on failure; the built `.deb` and the recorded evidence remain.

## Inputs and outputs

| Input / output | Description |
|---|---|
| Source tree | Post-TASK-97/TASK-103 state: one shared record reader, patcher, identity, and writer; native layout; one-way import; relocation; per-milestone board. |
| Toolchain | Rust 1.99.1 via `rust-toolchain.toml`; `dpkg-deb`, `fakeroot`, `dpkg`, `unshare`/`ip`; Debian Forky amd64. |
| Isolated sample fixture | Backlog.md-layout project generated under `./.tmp/task98/` for import, invisibility, adoption, and fidelity checks. |
| `scripts/build-deb.sh` | Unchanged packaging script; reads the version from `Cargo.toml`. |
| `papyrus_<version>_amd64.deb` | Built package in the project root, with recorded size and SHA-256. |
| Isolated root/admin | `./.tmp/task98/{inst,adm}` for `dpkg --instdir/--admindir`; removed after the run. |
| `tests/release_native_layout_smoke.rs` | New milestone-level integration test for the assertable portion. |
| `docs/packaging.md` | Updated with the TASK-98 procedure and observed results. |
| Ticket notes | Verification evidence and findings for TASK-98. |

## Validation and error behavior

| Condition | Consequence |
|---|---|
| Repository `backlog/` content changes, or is used as an import source | Blocked; the run is invalid and the fixture approach must be corrected. |
| Fixture source tree is not byte-identical after the run | Blocked; import wrote to its source. |
| Any surface shows a Backlog ID, or accepts one as input | Blocked; single-ID claim fails. |
| Any written file is Backlog-shaped, or a storage-ID key appears | Blocked; no-drift claim fails. |
| A repeated adoption, or an equal-value write to a native front-matter record (including a record after its first-write retirement), changes a byte, mode, or timestamp | Blocked; idempotency claim fails. |
| Import fidelity mismatch against fixture expectations | Blocked; record the record, field, and expected/actual values. |
| Relocation moves a partial plan, or persists configuration on failure | Blocked. |
| Front-matter-only search query returns nothing | Blocked; search claim fails. |
| Any quality gate fails | Blocked; fix or report before packaging. |
| `dpkg-deb -I` metadata mismatch, or package contents exceed the two expected files | Blocked. |
| Isolated install fails, or the host package state changes | Blocked; isolation is mandatory. |
| Installed-binary import or browser smoke fails | Blocked; record the command and output. |
| Outbound connection succeeds in the offline namespace | Offline claim not verified; investigate before completion. |
| Product defect found | Fix minimally, cover it with a test, and report it separately in the notes; no scope expansion. |
| Requirement, specification, or ADR conflict found | Record as an open question and escalate; do not edit requirements, business rules, or ADRs here. |
| Documented upgrade steps disagree with observed behaviour | Record a finding for TASK-99/TASK-73; do not edit the upgrade notes here. |

## Acceptance criteria mapping

| Check | Verifies | How it is verified |
|---|---|---|
| T-98-01 | Isolated fixture complete and repository store untouched | Fixture inventory, `git status --porcelain backlog/`, recursive hash before/after |
| T-98-02 | Adoption read byte-identical; native equal-value writes byte-identical; first legacy write retires sections | Repeated adoption and equal-value patch writes with byte/mode/mtime comparison; first-write retirement of a legacy record asserted separately, with post-retirement identity |
| T-98-03 | No Backlog-shaped write anywhere | Written-path inspection over create/edit/adopt/archive/restore/import; `tests/no_drift.rs` in the passing suite |
| T-98-04 | Unimported store invisible, readable after import, source immutable | CLI and browser checks before/after import; source hash |
| T-98-05 | One Papyrus ID on every surface; Backlog ID rejected | Captured CLI text, `--json`, browser HTML, JSON API, URLs; `task view task-1` exits 3 |
| T-98-06 | Import fidelity, bodies, destinations, excluded scope | Fixture-expectation comparison of every in-scope record and the excluded counts |
| T-98-07 | Documented upgrade path executes as written | Literal procedure run on the isolated project; plan/report kept; findings recorded |
| T-98-08 | Relocation atomicity, conflict, containment, browser path | CLI `config set` success/conflict, hash comparisons, installed-binary re-check; **retired (TASK-203, 2026-11-18, ADR-034, REQ-06.018):** the former browser-path and browser view equivalence requirements, which the rewritten `t98_08` no longer tests because the browser serves no configuration view |
| T-98-09 | Search indexes front matter on the CLI; browser exposes the record | Front-matter-only query via `papyrus search` (text and `--json`); the same record rendered by the browser record view and the JSON API; project hash unchanged |
| T-98-10 | Rust quality gates pass | `cargo build --release`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check` |
| T-98-11 | Debian package metadata and contents | `./scripts/build-deb.sh`, `dpkg-deb -I`, `dpkg-deb -c`, copyright inspection |
| T-98-12 | Installed binary imports and serves the browser on Forky | Isolated `dpkg` install, CLI import, browser board/view/status change/API/assets, offline check, `-r`, `--purge` |
| T-98-13 | Evidence recorded; new smoke test added | `docs/packaging.md` section, ticket notes, `tests/release_native_layout_smoke.rs` in the passing suite |

## Assumptions

- Assumption: the verification host is Debian Forky amd64. At refinement it reports `forky/sid`, glibc 2.43, `dpkg` 1.23.11, with `fakeroot`, `dpkg-deb`, `ip`, and `unshare -rn` available, and Rust 1.99.1 pinned by `rust-toolchain.toml`; the isolated `dpkg --instdir/--admindir` install therefore satisfies NFR-03.002's verification method without a container (OQ-3).
- Assumption: an isolated sample Backlog.md project is the only import source and the repository's own store is not imported in this ticket; its one-time import is deferred out of Release 1.0 (project-owner decision; TASK-100 archived).
- Assumption: the upgrade-path check uses the release upgrade notes produced by TASK-99 when they exist at verification time; otherwise it verifies the same steps against the shipped command behaviour and records that the notes were not yet available. No dependency edge is added because TASK-99 is a later-numbered ticket.
- Assumption: the package version stays `0.1.0`, read from `Cargo.toml`; no version bump is defined for this ticket and release versioning belongs to release orchestration (OQ-2).
- Assumption: `scripts/build-deb.sh` is used unchanged and does not force a rebuild, so `cargo build --release` runs first and `--help` is asserted before packaging.
- Assumption: the exhaustive relocation and import case matrices stay authoritative in `tests/config_relocation_cli.rs` and `tests/import_backlog_cli.rs`; this ticket adds end-to-end and installed-binary evidence on top.
- Assumption: the new integration test runs the built binary (`CARGO_BIN_EXE_papyrus`), not the installed package; installed-package checks are the scripted checklist in sections 10 and are recorded in `docs/packaging.md`.
- Assumption: the whole-repository `backlog/` directory is a tracked input of this project that must remain in the Backlog.md layout, so any verification step that would write there is replaced by the isolated fixture.
- Assumption: no new runtime dependency and no new UI, so the dependency-license audit and MOCK-0002 remain valid.
- Assumption: product behaviour is unchanged; a defect found by verification is fixed minimally, covered by a test, and reported, not treated as new scope.

## Open questions

None of these blocks implementation; each has a specified default.

- **OQ-1 (recommended default: markup/asset/HTTP-level browser verification)** — NFR-05.001's verification method names Firefox, Edge, and Safari. This headless task verifies served markup, assets, HTTP responses, and the board status-change round trip, but does not drive a real GUI browser. Default: record the limitation and rely on the per-feature browser tests plus MOCK-0002; schedule a manual GUI pass outside this ticket if required. Non-blocking.
- **OQ-2 (recommended default: keep version 0.1.0)** — No Release 1.0 version number is defined in the requirements. Default: package the current Cargo version unchanged and let release closure (TASK-73) decide on a bump. Non-blocking; flag for the release owner.
- **OQ-3 (recommended default: `fakeroot dpkg --instdir/--admindir` on the Forky build host)** — Whether a full Forky container or VM is mandatory. Default: the isolated install exercises Forky's glibc and `dpkg` without touching the host; the same checklist applies unchanged if a container is required. Non-blocking.
- **OQ-4 (recommended default: upgrade notes not a blocker)** — Whether the upgrade-path verification waits for TASK-99. Default: verify the shipped behaviour and the documented steps if available, and record the ordering in the evidence. Non-blocking.

## Traceability

| Artifact | Reference |
|---|---|
| Requirement | REQ-01.002, REQ-01.003, REQ-01.004, REQ-01.005, REQ-05.007, REQ-05.010, REQ-05.011, REQ-06.002, REQ-06.004, REQ-06.008, REQ-06.013 |
| Non-functional | NFR-03.002, NFR-04.001, NFR-04.002, NFR-04.004, NFR-05.001, NFR-06.001, NFR-06.002 |
| ADR | ADR-0001 (Rust), ADR-0003 (Debian Forky only), ADR-0004 (single crate), ADR-0005 (embedded server, no new dependency), ADR-0006 (atomic writes and version tokens), ADR-0010 (native layout and single ID space), ADR-0011 (one-way import), ADR-0012 (Papyrus-owned front matter), ADR-0013 (stage 8 verification gate) |
| Feature specification | `docs/specifications/release-1.0-native-layout-verification-and-packaging.md` (this file) |
| Related specifications | `papyrus-native-record-layout-and-single-id-space.md`, `import-backlog-md-records.md`, `remove-legacy-record-serializers-and-enforce-the-no-drift-rule.md`, `shared-record-envelope-and-front-matter-patcher.md`, `migrate-document-record-types-to-front-matter-lazily-on-first-read-and-first-write.md`, `render-one-metadata-projection-in-cli-and-browser-views.md`, `search-complete-unified.md`, `browser-tasks-milestones-and-releases.md`, `browser-spec-revisions-task-101.md`, `browser-spec-revisions-task-102.md`, `project-initialization-and-configuration.md`, `rust-cli-and-debian-package-baseline.md` |
| Ticket | TASK-98; upstream TASK-83, TASK-88, TASK-89, TASK-90, TASK-92, TASK-97, TASK-103 |
| UI mockups | none new; MOCK-0002 remains the backlog board reference |
| Packaging documentation | `docs/packaging.md` |
| Existing authoritative tests | `tests/no_drift.rs`, `tests/config_relocation_cli.rs`, `tests/import_backlog_cli.rs`, `tests/browser_cli.rs`, `tests/search_cli.rs`, per-feature `tests/*_cli.rs` and `tests/*_smoke.rs` |
