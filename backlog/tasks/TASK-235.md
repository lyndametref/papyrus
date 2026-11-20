---
id: TASK-235
title: "Update TECH-008, TECH-013, TECH-024, and TECH-027 packaging and verification sections"
created_date: '2026-11-20T17:40:19Z'
updated_date: '2026-11-20T18:15:43Z'
labels: []
references:
  - TECH-008
  - TECH-013
  - TECH-024
  - TECH-027
  - NFR-007.001
  - NFR-007.002
  - NFR-007.003
status: ToDo
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
