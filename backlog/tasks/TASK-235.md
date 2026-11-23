---
id: TASK-235
title: "Update TECH-008, TECH-013, TECH-024, and TECH-027 packaging and verification sections"
created_date: '2026-11-20T17:40:19Z'
updated_date: '2026-11-21T16:27:31Z'
labels: []
references:
  - TECH-008
  - TECH-013
  - TECH-024
  - TECH-027
  - NFR-007.001
  - NFR-007.002
  - NFR-007.003
status: Done
assignee:
  - tech-writer
milestone: MS-031
dependencies:
  - TASK-232
ordinal: 190
documentation:
  - docs/specifications/technical/TECH-008.md
  - docs/specifications/technical/TECH-013.md
  - docs/specifications/technical/TECH-024.md
  - docs/specifications/technical/TECH-027.md
type: docs
---

## Description
Context: these four sections still describe per-milestone packaging and verification with per-type document code. The refactor has one release boundary and no intermediate release or RC.

Scope:
- Replace milestone-level packaging steps with verification builds that are discarded; only the final milestone produces a retained release package.
- State the gates that every milestone verification runs: full suite, clippy, no_drift, no new crates, zero document TypedHandler variants, no document slug dispatch outside the seed, strict record/JSON/status-code equivalence, semantic HTML equivalence, and the NFR-007.001 to NFR-007.003 measurements.
- Keep the binary size gate at 12 MiB and record the production LOC target of 28,000 as an internal target, explicitly not an acceptance criterion.
- Remove references to per-type document modules from the packaging inventory.
Do not reference TASK, MS, or REL IDs in the body.
## Refinement inputs (TASK-290)

### TECH section map (real paths)
- TECH-008 — `docs/specifications/technical/TECH-008.md` — §1 build and package, §2 isolated install/uninstall, §3 installed-binary smoke tests, §4 automated integration test, §5 documentation and evidence.
- TECH-013 — `docs/specifications/technical/TECH-013.md` — §1 build and package, §2 isolated install/uninstall, §3 installed-binary smoke tests, §4 automated integration test, §5 documentation and evidence.
- TECH-024 — `docs/specifications/technical/TECH-024.md` — §1 build and package, §2 isolated install/uninstall, §3 installed-binary smoke tests, §4 automated integration test, §5 documentation and evidence.
- TECH-027 — `docs/specifications/technical/TECH-027.md` — §1 build and package, §2 isolated install/uninstall, §3 installed-binary smoke tests, §4 automated integration test, §5 documentation and evidence.
- Read-only inputs: NFR-007.001 to NFR-007.003 and the TASK-232 benchmark protocol.

### Refinement assumptions / definition of done
The refactor has one release boundary; verification builds are discarded and only the final milestone retains a package. No TASK/MS/REL id appears in a body. Done means all acceptance criteria pass and no source or test file changes (MS-031 AC-7, AC-10).
## Acceptance Criteria
- [ ] All four TECH files replace milestone-level packaging steps with discarded verification builds and reserve the retained release package for the final milestone (MS-031 AC-7).
- [ ] All four state the verification gates: full suite, clippy, no_drift, no new crates, zero document TypedHandler variants, no document slug dispatch outside the seed, strict record, JSON, and status-code equivalence, semantic HTML equivalence, and NFR-007.001 to NFR-007.003 measurements (MS-031 AC-7, AC-9).
- [ ] The 12 MiB binary size gate is retained and the 28,000 production LOC figure is recorded as an internal target explicitly not an acceptance criterion (MS-031 AC-7).
- [ ] Per-type document modules are removed from the packaging inventory; no TASK, MS, or REL ids appear in any body, and the user has verified the four updates (MS-031 AC-7, AC-8).
## Comments
- **tech-writer**: Updated TECH-008, TECH-013, TECH-024, and TECH-027 to the single-release-boundary model; all four are status draft and each was verified with papyrus tech-spec view to contain exactly one front matter block, one heading, and one section set. Changes: the packaging section is now Build and verification package, the verification package is staged under .tmp and discarded, and only the final milestone produces a retained release package with one artifact and recorded size and SHA-256; a shared verification gate table now covers the full suite, clippy and fmt, no_drift, no new crates, zero document TypedHandler variants, no document slug dispatch outside the seed, strict record, JSON, and status-code equivalence, semantic HTML equivalence, the NFR-007.001 to NFR-007.003 measurements, the 12 MiB binary gate, and the 28,000-line production LOC figure recorded as an internal target that is explicitly not an acceptance criterion; the package inventory is exactly the binary and the copyright file with no source file, no per-type document module, no asset, template, configuration, or man page; asset header assertions were retargeted to the ADR-038 cache classes; the ADR-038 asset Version packaging check was added; and every TASK reference was removed from front matter and bodies, with ADR-0036, ADR-037, ADR-038, ADR-039, NFR-007.001, NFR-007.002, NFR-007.003, and REQ-007.001 to REQ-007.004 added and only requirement, NFR, rule, glossary, and decision references retained. papyrus check reports only the three pre-existing archived-task frontmatter conflicts and no new findings. No source, test, requirement, business rule, ADR, or packaging change was made and papyrus.config.yml is byte-identical. Architect conformance requested.
- **architect**: Architect conformance review, 2026-11-21 UTC. Verdicts: TECH-008 draft; TECH-013 draft; TECH-024 draft; TECH-027 draft. None is verified; no TECH text was changed by the architect. Conformant in all four: one release boundary and discarded verification builds; the full gate set (full suite, clippy and fmt, no_drift, no new crates, zero document TypedHandler variants, no document slug dispatch outside the seed, strict record, JSON, and status-code equivalence, semantic HTML equivalence, NFR-007.001 to NFR-007.003 measurements); the 12 MiB binary gate; 28,000 LOC as an internal target explicitly not an acceptance criterion; the inventory is the binary plus copyright with no per-type document module; ADR-038's three cache classes (immutable vendor, revalidated app, no-store HTML and JSON). Required corrections (same in TECH-008, TECH-013, TECH-024, TECH-027): (1) Staging path: the build-deb script under scripts is unchanged and writes papyrus_<version>_amd64.deb to the project root, not a .tmp directory; in section 1.3 replace the claim that the script writes into a staging directory under the .tmp directory with: produces papyrus_<version>_amd64.deb in the project root, and the verification moves it into a staging directory under the .tmp directory; in the scope bullet replace the package staged under the .tmp directory with: then moving the package under the .tmp directory; in T-03 replace the build-deb script into the .tmp directory; dpkg-deb -I with the build-deb script; move the package into the .tmp directory; dpkg-deb -I. (2) Slug-dispatch scan: the four-path list lets a slug literal in the src registry module or another production file pass; replace the four-path list with anywhere in src except the SEED_DOCUMENT_TYPES table in the config module; keep tests and documentation are exempt; this matches MS-037 AC-3 and ADR-036 decision 28. (3) Minor: in the Purpose, Both are installed in an isolated root is false for the binary; replace with: The package is installed in an isolated root, the installed binary is exercised, and both are deleted. Rulings: (1) Packaging script: use the existing build-deb script under scripts for verification and the release boundary; do not add a build-package script under scripts; ADR-038 decision 30 and TECH-069 section 6 name the build-deb script, and MS-037 AC-6's name is a milestone-text inconsistency for the milestone owner to correct. (2) No new crates: include dev-dependencies and build-dependencies; any new crate or dependency edge in Cargo.toml or Cargo.lock fails the gate. (3) LOC: report production LOC at every verification as an informational measurement and at the release boundary; it is never a gate, acceptance criterion, or blocker. Evidence: reviewed the four TECH files, ADR-036 to ADR-039 and the amendments to ADR-0005, ADR-0006, ADR-0016, ADR-0020, ADR-0027, ADR-0028, ADR-033, and ADR-035; MS-036 and MS-037; NFR-007.001 to NFR-007.003; the build-deb script under scripts. No source, test, requirement, rule, ADR, packaging, or TECH text was changed.
- **tech-writer**: Applied the architect corrections to TECH-008, TECH-013, TECH-024, and TECH-027; all four remain draft. Edits per file: section 1.3 now states that build-deb writes papyrus_<version>_amd64.deb to the project root and the verification moves it into the tmp staging directory and deletes it afterwards; the scope bullet says the script produces the package in the project root and the verification then moves it under tmp; T-03 now reads build-deb, move the package into tmp, then dpkg-deb -I; the Purpose says the package is installed in the isolated root, the installed binary is exercised, and both are deleted; the slug-dispatch gate now scans anywhere in src except the SEED_DOCUMENT_TYPES table in the config module, with tests and documentation exempt; the no-new-crates gate includes dev-dependencies and build-dependencies and fails on any new crate or dependency edge in Cargo.toml or Cargo.lock; the production LOC is reported at every verification as information and never a gate, acceptance criterion, or blocker; the packaging assumption names build-deb as the single script for verification and release and states no separate build-package script exists; the verification record and T-16 include LOC. Each section parsed with papyrus tech-spec view. No TASK, MS, or REL ids in bodies; statuses stay draft. Architect conformance requested.
- **architect**: Architect re-review, 2026-11-21 UTC. Verdicts: TECH-008 verified; TECH-013 verified; TECH-024 verified; TECH-027 verified. The three required corrections are applied in all four: the build-deb script produces papyrus_<version>_amd64.deb in the project root and the verification moves it into .tmp and deletes it, consistent in the scope bullet, section 1.3, T-03, and the inputs table; the slug-dispatch gate scans anywhere in src except the SEED_DOCUMENT_TYPES table in the src config module, with tests and documentation exempt; the Purpose states that the package is installed in the isolated root, the installed binary is exercised, and both are deleted. The three rulings are recorded and reflected: the existing build-deb script serves the milestone verification and the release boundary and no build-package script exists; the no-new-crates gate includes dev-dependencies and build-dependencies and fails on any new crate or dependency edge in Cargo.toml or Cargo.lock; production LOC is reported at every verification and at the release boundary as information and is never a gate, acceptance criterion, or blocker. All four retain the nine gates, the 12 MiB binary gate, an inventory limited to the binary and the copyright file, the ADR-038 cache classes, ADR-036 decision 28 as the seed definition, no TASK, MS, or REL ids in bodies, and only REQ, NFR, BR, GLO, and ADR references. papyrus check reports only the three pre-existing archived-task frontmatter conflicts. No source, test, requirement, rule, decision, packaging, or TECH text was changed.
- **orchestrator**: Acceptance met: TECH-008, TECH-013, TECH-024 and TECH-027 updated and architect-verified for discarded verification builds, the nine gates, 12 MiB binary gate, informational LOC and packaging inventory without per-type modules.
