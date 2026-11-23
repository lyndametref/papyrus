---
id: TASK-289
title: "Verify MS-037 and confirm the single release is ready"
created_date: '2026-11-20T17:44:23Z'
updated_date: '2026-11-23T14:31:28Z'
labels: []
references:
  - REQ-07.004
  - NFR-007.001
  - NFR-007.002
  - NFR-007.003
  - TECH-069
  - MS-037
  - ADR-043
  - TECH-008
  - AUD-150
  - MS-036
status: Done
assignee:
  - quality-gatekeeper
milestone: MS-037
dependencies:
  - TASK-288
ordinal: 100
documentation:
  - docs/benchmarks/MS-036-results.md
  - docs/audits/AUD-150.md
  - decisions/ADR-043-component-build-script-as-the-packaging-entry-point.md
  - scripts/build-package.sh
  - scripts/build-deb.sh
  - docs/packaging.md
  - docs/release-1.0-upgrade-notes.md
type: milestone-verification
---

## Description
## Context
Milestone boundary verification for MS-037 and the only release boundary (MS-037 AC-7). Dependency TASK-288 (final package) must be Done. This task verifies; it does not implement or repair.

## Scope
- Verify every MS-037 acceptance criterion AC-1 through AC-7 against the committed verified tree, with a command or artifact per criterion.
- Cross-milestone gates: full suite; clippy and fmt; `no_drift`; document-type no-drift; descriptor-leak guard; no new crates; zero document `TypedHandler` variants; no document slug dispatch outside the seed; record, JSON, and status-code equivalence against the MS-032 fixtures; semantic HTML equivalence; NFR-007.001 and NFR-007.002 measured on the recorded host with the hardware deviation; NFR-007.003 cache classes; binary at or under 12 MiB; no open blocking audit finding.
- Carried items to verify:
  - MS-036 AC-8 records `Connection: close` as the preserved behaviour (planner correction recorded 2026-11-23; verify the wording or record the escalation).
  - ADR-043 decision 21 exception for MS-035 and MS-036 is recorded in the 2026-11-23 amendment; verify it or record the architectural escalation.
  - The duplicate AUD-145 and AUD-148 records are resolved, and `papyrus check` reports no duplicate-id finding for them.
  - The hardware deviation (6 physical and 12 logical cores, 15.3 GiB versus the 4 vCPU and 8 GiB reference) is present in the release report.
  - Production LOC is recorded against the 28,000 internal target as informational only, not an acceptance criterion.
- RC and package gate per MS-037 AC-7: after every quality and code check passes, determine the RC version with `next-version` in RC mode and publish the RC package for that version under `builds/<version>/` with the same `scripts/build-package.sh` command, recording version, commit SHA, script paths and commands, archives, paths, sizes, and sha256; or record an explicit user waiver with scope, reason, and timestamp with timezone. The output path is permitted by the ADR-043 decision 7 rule implemented in TASK-283.
- Confirm exactly one release artifact exists and no intermediate release or RC is retained; the RC is discardable.
- Record the release report (the project's `release-reports` document under `docs/release-reports/`) with the NFR-007.001 and NFR-007.002 hardware deviation caveat for the owner.

## Out of scope
- Do not repair a failing criterion; report the command and artifact to the orchestrator.
- Do not edit MS-036, ADR-043, requirements, rules, specifications, mockups, or source.

## Definition of done
Every MS-037 acceptance criterion and cross-milestone gate has a recorded pass verdict with a reproducible command or artifact; the carried items are verified or escalated; the RC version and package evidence, or a recorded user waiver, are present; the release report contains the hardware deviation caveat; exactly one release artifact remains; the task is Done only then, and the release is handed to the orchestrator for Phase 6.

## Acceptance Criteria
- [ ] #1 After all quality and code checks pass, the milestone RC version is determined with next-version in RC mode and recorded in this verification task.
- [ ] #2 The RC package is published with packaging from the committed verified tree into the git-ignored builds directory for that version, using each deployable component's build-package script with the version and output arguments fixed in the definition of done, and the package evidence is recorded in this task: version, commit SHA, script paths and commands, component archives, paths, sizes, and sha256; an explicit user waiver with scope, reason, and timestamp with timezone is the only alternative.
- [ ] #3 This task is not Done until the RC version is recorded and the package evidence or the recorded user waiver is present.
- [ ] #4 The RC package does not replace the final release artifact: TASK-288 remains the single final release package produced with the release build script, and both the RC evidence and the final package evidence are recorded before the release leaves this task.
- [ ] #5 Every MS-037 AC-1 through AC-7 has a recorded pass or fail verdict with a command or artifact.
- [ ] #6 The full suite, clippy, fmt, no-drift, document-type no-drift, descriptor-leak guard, and no-new-crate gates pass and are named.
- [ ] #7 Zero document TypedHandler variants and no document slug dispatch outside the seed are verified by the guard suite.
- [ ] #8 Record, JSON, and status-code equivalence and semantic HTML equivalence pass against the MS-032 corpus.
- [ ] #9 NFR-007.001 and NFR-007.002 results are presented with the hardware deviation and are not stated as NFR satisfaction; NFR-007.003 cache classes pass; the binary is at most 12,582,912 bytes.
- [ ] #10 MS-036 AC-8 records `Connection: close` as the preserved behaviour; the ADR-043 decision 21 exception is recorded in the 2026-11-23 amendment; the duplicate AUD-145 and AUD-148 records are resolved and papyrus check is clean for them.
- [ ] #11 The release report contains the NFR-007.001 and NFR-007.002 hardware deviation caveat and records production LOC against 28,000 as informational only.
- [ ] #12 Exactly one release artifact exists after the checks; no intermediate release or RC is retained; any RC produced is recorded as discardable.
- [ ] #13 The task is Done only when the carried items and the RC evidence or waiver are present, and the release is handed to the orchestrator for Phase 6.

## Comments
- **task-refiner**: Reproduced at refinement: the RC command output directory is refused by package-out, so the RC gate is blocked on the TASK-288 escalation.
- **planner**: planner resolution: the TASK-288 output-path escalation is cleared; ADR-043 decision 7 as amended 2026-11-23 permits the versioned builds path and TASK-283 implements and tests the guard. MS-036 AC-8 wording was corrected by the planner on 2026-11-23.
- **rust-dev**: RC EVIDENCE (rust-dev): version 1.0.0-rc.1 built from the committed verified tree at HEAD 440b9fe; command: scripts build-package.sh --version 1.0.0-rc.1 --out builds 1.0.0-rc.1 .build papyrus; artifact: builds 1.0.0-rc.1 .build papyrus papyrus_1.0.0-rc.1_amd64.deb; size 2431408 bytes; sha256 00a950ab78cec75c7ca0329e9cfedd8abf48aa936e4c85d3719552f3e55ac319; dpkg-deb -I reports Version 1.0.0-rc.1 and Architecture amd64; inventory is exactly two files, usr bin papyrus mode 0755 and usr share doc papyrus copyright mode 0644 plus directories; the packaged binary reports papyrus 1.0.0 (Cargo version) and is byte-identical, sha256 430a6e538fcdc9ccc3b59d7c4b8ab9b53fc0410016b5b6224777ce9b0671e2ba, to the final package binary; the RC differs from the final package only in the Debian Version field and the artifact name; the RC is discardable and is not a release artifact. FINAL PACKAGE: builds 1.0.0 .build papyrus papyrus_1.0.0_amd64.deb unchanged, 2431328 bytes, sha256 e59150d011582fa57f8e9bc13e7e36460019d0886132c188bd8a4e20ac3312b6 (owner shorthand e59150d0...12b3 matched on the e59150d0 prefix; the full digest ends 3312b6); exactly one release artifact remains for 1.0.0. FIXES: F6 removed the deferred debug print eprintln DEBUG closed at src browser mod.rs line 1497; no debug-log mechanism exists in the browser module, so the line was deleted; F8 corrected docs packaging.md artifact paths to builds version .build component papyrus version amd64.deb and recorded release commit cceed99 (merged at 440b9fe) for the retained 1.0.0 package. CHECKS: env -u EDITOR -u VISUAL cargo test --locked 1443 passed, 0 failed, 27 ignored across 70 suites; cargo clippy --all-targets -- -D warnings clean; cargo fmt --check clean; cargo audit unavailable in the toolchain and this task changes no dependency. Working tree left uncommitted as instructed; task status remains In-Progress.
- **code-analyzer**: code-analyzer: REL-001 code audit reissue AUD-155: code delta passes with findings; blocking: the retained 1.0.0 package predates HEAD 103604e and contains the removed DEBUG print (fresh HEAD build d519577b), so rebuild the package from HEAD and update the recorded sha256.
- **rust-dev**: REL-001 security reissue closure (rust-dev): L-2 config set and config unset now read the existing configuration through the budgeted reader read_raw_config via the new read_config_overlay helper (src commands.rs), so the 1048576 byte size limit and the anchor and alias rejection apply before any parse; unit test config_set_and_unset_reject_an_unbudgeted_configuration_without_writing covers an oversized file and an aliased file for both commands and asserts the file bytes are unchanged; installed-binary smoke: the oversized file exits 4 with 'configuration exceeds the 1048576 byte YAML limit' and its digest is unchanged, the aliased file exits 4 with 'configuration must not contain YAML anchors or aliases' and its digest is unchanged. L-3 the debug request log now sanitizes the decoded path through log_path (printable ASCII only, 200 character cap) in src browser mod.rs; unit test log_path_sanitizes_decoded_control_characters_and_caps_length plus integration test debug_log_sanitizes_decoded_control_characters_in_the_path, where a percent 0A path is rejected 400 at the request boundary and the single stderr line '- - 400' carries no control character. L-4 the README browser_host configuration-key entry now states that a non-loopback bind is unauthenticated with read and modify scope, mirroring the browser command section. L-5 builds 1.0.0-rc.1 SUPERSEDED.md marks that candidate a discarded verification build, not a release artifact; the artifact itself is kept. I-1 TECH-007 section 1 item 9 now quotes the ADR-044 warning text 'warning: binding to non-loopback address <host> without authentication; anyone who can reach this port can read and modify every record'; the section structure and the status are unchanged. PACKAGE REBUILT from this tree at HEAD 103604e plus the uncommitted fixes above, with no commit made per instruction: builds 1.0.0 .build papyrus papyrus_1.0.0_amd64.deb, 2429284 bytes, sha256 b2d747ac017a65659b5147300775d423b95961c3b611bfbc733df09c57ddf43c, replacing e59150d0...3312b6; dpkg-deb -I reports Package papyrus, Version 1.0.0, Architecture amd64, Section utils, Priority optional, Depends libc6 (>= 2.34) and the copyright file declares GPL-3.0-or-later; inventory is exactly two files, usr bin papyrus mode 0755 owner root root size 8285368 bytes and usr share doc papyrus copyright mode 0644 size 8000 bytes, plus directories; the packaged binary sha256 66a733f15b33b4d0506260b5a83b39998c56a087d9e4b198c402883c6d7459ab is byte identical to the release build target papyrus and reports papyrus 1.0.0; the binary carries the new non-loopback warning string and no Rust DEBUG literal; the isolated fakeroot dpkg install with instdir and admindir under .tmp succeeded, the installed binary is root root 755, resolves on the isolated PATH and runs. CHECKS: env -u EDITOR -u VISUAL cargo test --locked 1446 passed, 0 failed, 27 ignored across 70 suites; cargo clippy --all-targets --locked -- -D warnings clean; cargo fmt --all --check clean; binary 8285368 bytes, below the 12582912 byte ceiling; exactly one 1.0.0 release artifact remains and the 1.0.0-rc.1 candidate is marked discarded. OPEN ITEM: docs packaging.md line 69 still records the retained package as built from release commit cceed99, but it now includes the post-cceed99 fixes, so the orchestrator should update or accept that provenance sentence. NOTE: a concurrent code-analyzer session created docs audits AUD-155 and moved AUD-152 to docs audits archived; those changes are not mine. Task status left In-Progress.
- **rust-dev**: Documentation correction in packaging.md line 69 (docs folder): retained 1.0.0 package commit changed from cceed99 to 38736ae (TASK-289 security reissue rebuild), recording SHA-256 b2d747ac017a65659b5147300775d423b95961c3b611bfbc733df09c57ddf43c and 2429284 bytes. Artifact hash verified on disk; nothing rebuilt. papyrus check reports no new findings (same 3 pre-existing archive frontmatter-conflict findings); the document relative link resolves.
- **quality-gatekeeper**: Quality audit AUD-156 issued: REL-001 ready for release approval; 0 blockers, 1 medium record gap (AUD-155 reissue), 2 low follow-ups.
- **orchestrator**: Release verification PASS (AUD-156): MS-037 AC-1 to AC-7 met; 1446 tests pass; final package b2d747ac 2,429,284 B reproduced from 38736ae; RC 1.0.0-rc.1 built and discarded; release report RELR-001 issued; security reissue AUD-157 and code artifact closure recorded.
