---
id: TASK-227
title: "Amend REQ-06.014, REQ-06.019, REQ-06.004, REQ-01.002, REQ-01.005, and REQ-01.008"
created_date: '2026-11-20T17:38:40Z'
updated_date: '2026-11-20T17:38:40Z'
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
status: ToDo
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
