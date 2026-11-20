---
id: TASK-226
title: "Amend REQ-05.001 to REQ-05.014 for seed-declared fields, grouped req and nfr, and the HTML mockup model"
created_date: '2026-11-20T17:38:12Z'
updated_date: '2026-11-20T17:38:33Z'
labels: []
references:
  - ADR-036
  - ADR-039
  - REQ-05.003
  - REQ-05.008
  - REQ-05.010
  - REQ-07.004
status: ToDo
assignee:
  - requirements-engineer
milestone: MS-031
dependencies:
  - TASK-223
  - TASK-224
ordinal: 100
documentation:
  - docs/requirements/business/REQ-05.001.md
  - docs/requirements/business/REQ-05.003.md
  - docs/requirements/business/REQ-05.005.md
  - docs/requirements/business/REQ-05.006.md
  - docs/requirements/business/REQ-05.008.md
  - docs/requirements/business/REQ-05.010.md
type: requirements
---

## Description
Context: nine per-type document modules are deleted. Every per-type behavior they carry must remain expressible as seed data and must stay required by REQ-05.

Scope: light amendments to 05.001, 05.002, 05.004, 05.005, 05.006, 05.007, 05.009, 05.011, 05.012, 05.013, 05.014; major rewrites of REQ-05.008 (UI mockups) and REQ-05.010 (front matter); REQ-05.003 removes --term.
- req (05.001) and nfr (05.002): GroupedMarkdown; group record with plain ID plus member records with dotted IDs; member fields story and description; slot group required; existing active group of the same type reused; cross-type and archived-group rejection; reference_visibility hidden.
- glossary (05.003): title replaces --term; field definition with heading "## Definition", required.
- rule (05.005): description required; rationale required with verbs [create]; applicability required with verbs [create]; requirements field with heading "## Requirements", kind list, option requirement.
- audit (05.006): audit_type field with heading "## Audit Type", frontmatter_key type, option audit-type; content field with heading "## Content", required; title arrives as the positional title.
- ui-mockup (05.008): SingleHtmlMockup; one record with plain ID; supersede verb; no status verb; fixed pending, approved, archived metadata; title contract in the HTML metadata; HTML file remains Papyrus-owned.
- tech-spec (05.009), ui-spec (05.004), use-case (05.012), test-case (05.013), test-report (05.014): SingleMarkdown with their existing fields expressed in the seed; no behavior loss.
- 05.010: general front-matter keys id, title, status, labels, references, created_date, updated_date, group are reserved; additional keys are config-declared; model, title, content, id_shape are not config keys.
- 05.007 and 05.011: archive/delete and status behavior unchanged; group archive or delete cascades to nothing.
