---
id: TASK-226
title: "Amend REQ-05.001 to REQ-05.014 for seed-declared fields, grouped req and nfr, and the HTML mockup model"
created_date: '2026-11-20T17:38:12Z'
updated_date: '2026-11-21T15:50:53Z'
labels: []
references:
  - ADR-036
  - ADR-039
  - REQ-05.003
  - REQ-05.008
  - REQ-05.010
  - REQ-07.004
  - REQ-07.001
  - BR-009
  - BR-010
  - BR-011
  - BR-012
  - BR-013
  - BR-014
  - BR-017
  - BR-018
  - BR-019
  - BR-020
status: Done
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
  - docs/requirements/business/REQ-05.002.md
  - docs/requirements/business/REQ-05.004.md
  - docs/requirements/business/REQ-05.007.md
  - docs/requirements/business/REQ-05.009.md
  - docs/requirements/business/REQ-05.011.md
  - docs/requirements/business/REQ-05.012.md
  - docs/requirements/business/REQ-05.013.md
  - docs/requirements/business/REQ-05.014.md
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
## Refinement inputs (TASK-290)

### Requirement file map (current section numbers)
- REQ-05.001 (§5.1) — docs/requirements/business/REQ-05.001.md — Manage Business Requirements — light amendment — req GroupedMarkdown
- REQ-05.002 (§5.2) — docs/requirements/business/REQ-05.002.md — Manage Non-Functional Requirements — light amendment — nfr GroupedMarkdown
- REQ-05.003 (§5.3) — docs/requirements/business/REQ-05.003.md — Manage Glossary Terms — light amendment — title replaces --term
- REQ-05.004 (§5.4) — docs/requirements/business/REQ-05.004.md — Manage UI Specifications — light amendment — ui-spec SingleMarkdown
- REQ-05.005 (§5.5) — docs/requirements/business/REQ-05.005.md — Manage Business Rules — light amendment — rule fields
- REQ-05.006 (§5.6) — docs/requirements/business/REQ-05.006.md — Manage Audit Records — light amendment — audit fields
- REQ-05.007 (§5.7) — docs/requirements/business/REQ-05.007.md — Manage Document Archive State and Deletion — light amendment — archive/delete unchanged
- REQ-05.008 (§5.8) — docs/requirements/business/REQ-05.008.md — Manage UI Mockups — major rewrite — ui-mockup SingleHtmlMockup
- REQ-05.009 (§5.9) — docs/requirements/business/REQ-05.009.md — Manage Technical Specifications — light amendment — tech-spec SingleMarkdown
- REQ-05.010 (§5.10) — docs/requirements/business/REQ-05.010.md — Record Metadata in YAML Front Matter — major rewrite — reserved general front-matter keys
- REQ-05.011 (§5.11) — docs/requirements/business/REQ-05.011.md — Maintain Record Metadata Through All Commands — light amendment — status behavior unchanged
- REQ-05.012 (§5.12) — docs/requirements/business/REQ-05.012.md — Manage Use Cases — light amendment — use-case SingleMarkdown
- REQ-05.013 (§5.13) — docs/requirements/business/REQ-05.013.md — Manage Test Cases — light amendment — test-case SingleMarkdown
- REQ-05.014 (§5.14) — docs/requirements/business/REQ-05.014.md — Manage Test Reports — light amendment — test-report SingleMarkdown

### Refinement assumptions / definition of done
Nine per-type document modules are being deleted, but their behavior must remain expressible as seed data. The scope list is the complete expected field/behavior inventory; any field not listed must be found in the current module and reported rather than dropped. Only the fourteen listed REQ files change; the user verifies each rewrite. Done means all acceptance criteria pass and no source or test file changes (MS-031 AC-10).
## Acceptance Criteria
- [ ] All fourteen REQ-05 records are amended: light amendments to 05.001, 05.002, 05.004, 05.005, 05.006, 05.007, 05.009, 05.011, 05.012, 05.013, and 05.014; major rewrites of 05.008 and 05.010; and the 05.003 title and --term change (MS-031 AC-4).
- [ ] REQ-05.001 and REQ-05.002 describe the GroupedMarkdown shape: plain-ID group, dotted-ID members, story and description member fields, required group slot, same-type active group reuse, cross-type and archived-group rejection, hidden references, immutable membership, and no cascade on archive or delete (MS-031 AC-4, RE-010 D7).
- [ ] Each built-in document type's field set is expressed in REQ-05 exactly as listed in the scope, including glossary definition, rule rationale and applicability, rule requirements, audit audit_type and content, and ui-mockup fixed metadata with no status verb (MS-031 AC-4).
- [ ] REQ-05.010 reserves id, title, status, labels, references, created_date, updated_date, and group, makes additional keys config-declared, and states that model, title, content, and id_shape are not config keys (MS-031 AC-4, RE-010 D14).
- [ ] REQ-05.007 and REQ-05.011 keep archive or delete and status behavior unchanged, no changed record cites archived REQ-06.003, Acceptance Criteria and Traceability are updated, and the user has verified every rewrite (MS-031 AC-4, AC-8).
## Comments
- **requirements-engineer**: Amended REQ-05.001 to REQ-05.014 for the seed-declared field model. Grouped req and nfr: plain group ID plus dotted member IDs, seed fields story and description, required same-type active group, engine-owned immutable group link, merged group-then-member list order, hidden references, no cascade. Glossary term is the record title with a required definition field and --term removed. Business rule description plus create-only rationale and applicability plus a requirements list field; audit audit_type stored under front matter key type plus required content; tech-spec, ui-spec, use-case, test-case and test-report expressed as seed fields. Major rewrites: UI mockup as SingleHtmlMockup with fixed pending, approved and archived metadata, supersede verb and no status verb; front matter with registry-owned general keys id, title, status, labels, references, created_date, updated_date and group and descriptor frontmatter_key document keys. Archive and delete plus status behaviour unchanged. Acceptance criteria and traceability updated in every record citing RE-010, ADR-036, ADR-039 and related ADRs; no archived REQ-06.003 cited. papyrus check stays at the 3 pre-existing task frontmatter-conflict findings.
- **orchestrator**: Acceptance met: REQ-05.001..014 all amended to the seed-declared field model, grouped req and nfr, title contract and front-matter ownership; no new check findings; open questions recorded (group link location, glossary identity, mockup metadata key).
- **orchestrator**: Reopened for rework: REQ-05.008 conflicts with the ADR-039 clarification on mockup fixed metadata (refs versus editable references).
- **requirements-engineer**: Corrected REQ-05.008 to ADR-039 decision 21 as clarified on 2026-11-20 and to the implemented metadata keys. Description: the fixed metadata list now names status, task, specs, refs, references and superseded_by, states that specs and refs are the two list-valued keys the edit form writes, and that status, task, references and superseded_by are read only. The create, view and edit paragraph now names specs and refs as the two editable list controls and references as read only. Acceptance criteria: the metadata key criterion now requires exactly the six keys with specs and refs written back to the HTML metadata comment; the edit form criterion now offers the title plus specs and refs only, with status, task, references and superseded_by read only. Traceability: the ADR-039 line now records that the 2026-11-20 clarification governs the fixed metadata keys. No other rule, test case, other requirement, BR, TECH or ADR changed. papyrus check reports only the 3 pre existing task frontmatter conflict findings, and papyrus req view REQ-05.008 parses. Out of scope observation: REQ-06.014 carries the same stale references wording and was not changed here.
- **orchestrator**: Rework complete: REQ-05.008 fixed metadata aligned with ADR-039 clarification (specs and refs editable; references, status, task, superseded_by read-only).
