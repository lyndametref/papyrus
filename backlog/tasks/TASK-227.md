---
id: TASK-227
title: "Amend REQ-06.014, REQ-06.019, REQ-06.004, REQ-01.002, REQ-01.005, and REQ-01.008"
created_date: '2026-11-20T17:38:40Z'
updated_date: '2026-11-20T21:00:46Z'
labels: []
references:
  - ADR-036
  - ADR-039
  - REQ-06.014
  - REQ-06.019
  - REQ-06.004
  - REQ-01.002
  - REQ-01.005
  - REQ-01.008
  - REQ-07.001
  - REQ-07.002
  - REQ-07.003
  - REQ-07.004
  - BR-016
status: Done
assignee:
  - requirements-engineer
milestone: MS-031
dependencies:
  - TASK-223
  - TASK-224
ordinal: 110
documentation:
  - docs/requirements/business/REQ-06.014.md
  - docs/requirements/business/REQ-06.019.md
  - docs/requirements/business/REQ-06.004.md
  - docs/requirements/business/REQ-01.002.md
  - docs/requirements/business/REQ-01.005.md
  - docs/requirements/business/REQ-01.008.md
type: requirements
---

## Description
Context: browser presentation, reference visibility, ID shape, and diagnostics all depend on the new config surface.

Scope:
- REQ-06.014: config-defined document types render through one descriptor-driven surface; GroupedMarkdown renders group and member records with role fields; SingleHtmlMockup renders its fixed metadata; presentation facts (labels, headings, field order, help) come from the seed.
- REQ-06.019: reference visibility is seed data per type and role; the discriminator is model plus seed reference_visibility; hidden means no reference list in that type's browser list view.
- REQ-06.004: CLI and browser views read front-matter metadata through the field map and each field's frontmatter_key.
- REQ-01.002: config surface gains nested group and member blocks, a fields sequence, selector paths, a minimal init file, config set materializing only the target entry, config list showing effective values with key selectors, and collection unset resetting to seed.
- REQ-01.005: record ID shape is seed-only and not a config key; prefix and padding still govern IDs; member records use dotted IDs under the group's prefix.
- REQ-01.008: check diagnostics cover unknown keys, unknown enums, reserved keys, duplicate field keys, and unresolvable selector paths.
REQ-06.003 is archived; do not cite or restore it in any changed requirement.
## Refinement inputs (TASK-290)

### Requirement file map (current section numbers)
- REQ-06.014 (§6.14) — docs/requirements/business/REQ-06.014.md — Display Config-Defined Document Types in the Browser — amend — browser surface
- REQ-06.019 (§6.19) — docs/requirements/business/REQ-06.019.md — Do not Show the Reference List in Document-Type Browser List Views — amend — reference visibility
- REQ-06.004 (§6.4) — docs/requirements/business/REQ-06.004.md — Use Front-Matter Metadata in CLI and Browser Views — amend — front-matter field map
- REQ-01.002 (§1.2) — docs/requirements/business/REQ-01.002.md — Read and Edit Project Configuration — amend — nested config and selectors
- REQ-01.005 (§1.5) — docs/requirements/business/REQ-01.005.md — Assign Stable Record IDs — amend — seed-only ID shape and dotted member IDs
- REQ-01.008 (§1.8) — docs/requirements/business/REQ-01.008.md — Run Project Diagnostics (`papyrus check`) — amend — diagnostics coverage

### Refinement assumptions / definition of done
REQ-06.003 is archived and must not be cited or restored. Only the six listed REQ files change; the user verifies each amendment. Done means all acceptance criteria pass and no source or test file changes (MS-031 AC-10).
## Acceptance Criteria
- [ ] All six listed REQ records are amended, and REQ-06.003 is neither cited nor restored anywhere (MS-031 AC-4).
- [ ] REQ-06.014 describes the one descriptor-driven surface, GroupedMarkdown group and member role rendering, SingleHtmlMockup fixed metadata, and presentation facts from the seed (MS-031 AC-4, RE-010 D10).
- [ ] REQ-06.019 makes reference visibility seed data per type and role, with model plus seed reference_visibility as the discriminator and hidden meaning no reference list in that type's browser list view (MS-031 AC-4, RE-010 D8).
- [ ] REQ-06.004 routes CLI and browser metadata reads through the field map and each field's frontmatter_key; REQ-01.002 captures nested config, selector paths, minimal init, config set target-entry materialization, config list effective values with selectors, and collection unset reset (MS-031 AC-4).
- [ ] REQ-01.005 makes ID shape seed-only and not a config key while preserving prefix and padding and dotted member IDs; REQ-01.008 covers unknown keys and enums, reserved keys, duplicate field keys, and unresolvable selector paths; Acceptance Criteria and Traceability are updated and the user has verified all six (MS-031 AC-4, AC-8).
## Comments
- **requirements-engineer**: Amended six records: REQ-06.014 (27 AC), REQ-06.019 (12 AC, title and story updated), REQ-06.004 (19 AC), REQ-01.002 (32 AC), REQ-01.005 (12 AC), REQ-01.008 (37 AC). Traced to RE-010, ADR-036, ADR-037, ADR-038, ADR-039, ADR-0019. No archived REQ-06.003 citation. papyrus check unchanged at 3 baseline findings. Status kept In-Progress.
- **orchestrator**: Acceptance met: REQ-06.014, 06.019, 06.004, 01.002, 01.005 and 01.008 amended for the descriptor-driven browser surface, seed reference visibility, field-map metadata and expanded config diagnostics; no new check findings; REQ-06.003 untouched.
