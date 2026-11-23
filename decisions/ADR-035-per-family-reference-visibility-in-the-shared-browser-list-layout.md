---
id: ADR-035
title: "Per-Family Reference Visibility in the Shared Browser List Layout"
created_date: '2026-11-19T06:40:21Z'
updated_date: '2026-11-20T20:09:24Z'
labels: []
references:
  - REQ-006.019
  - REQ-006.016
  - REQ-006.004
  - REQ-006.012
  - REQ-006.014
  - NFR-001.003
  - ADR-0028
  - ADR-0020
  - ADR-0033
  - RE-006
  - MOCK-0016
  - TECH-068
  - TECH-074
  - TECH-076
  - TECH-045
superseded_by: null
---
# ADR-035 Per-Family Reference Visibility in the Shared Browser List Layout

## Status
accepted
## Decision
1. Reference visibility in shared browser list views is declared seed data per document type and, for a grouped type, per role. The descriptor's `reference_visibility` resolves to `Visible` or `Hidden` for the record's role; the shared list renders a row's reference list only when that value is `Visible`. It is not a universal list rule and not a content-model rule.

2. ADR-036 decision 29 supplies the seed values: `req` and `nfr` are `Hidden` for both roles, every other seed type is `Visible`, and a user-defined type defaults to `Visible`.

3. No slug match, browser-side presentation override, per-family template, or per-type code is introduced. The descriptor's seed value is the only source, and the list renderer reads it for the row's type and role.

4. Labels remain rendered in the Title cell for every family, including document types; only references change.

5. The change is presentation-only: `Row.references` may still carry the `RecordView` projection, but the shared list renderer omits the reference block when the resolved value is `Hidden`.

6. No front matter, record file, `RecordView`, JSON API, CLI list or view output, or single-document view changes. The single-document view keeps its collapsible References section (ADR-0033), and CLI list and view keep references (REQ-006.004).

7. The shared list contract is otherwise unchanged: fixed structural columns, family slots, grouped rows, action placement, status filter, page width, CSP, no-JavaScript behaviour, and no network access.

8. This ADR amends ADR-0028 decision 2 and downstream constraint 11, and ADR-0020 decision 5, for list-row reference rendering; it does not supersede those records, and every other rule in them stands.

### Amendment (2026-11-20) — ADR-039

- **Amended:** the discriminator only. The rule keyed on `ContentModel::Generic` is replaced by the descriptor's seed `reference_visibility`, resolved per type and, for `GroupedMarkdown`, per role. ADR-0036 decision 29 supplies the seed values.
- **Not changed:** labels, the single-document References disclosure, CLI list and view, JSON, record files and front matter, the presentation-only scope, the shared list contract, and this record's accepted status.

### Amendment (2026-11-20) — descriptor model cross-check

- **Replacement discriminator.** The rule keyed on `ContentModel::Generic` is replaced by the pair (`DocumentModel`, seed `reference_visibility`). The model determines which roles the type has and therefore how many visibility values the descriptor carries: one for `SingleMarkdown` and `SingleHtmlMockup`, and one for the group role and one for the member role of `GroupedMarkdown`. The seed value for the row's type and role then decides `Visible` or `Hidden`, and the shared list renders the row's reference list only for `Visible`.
- **History.** The Context sentence that equates config-defined types with `ContentModel::Generic` and the Consequences bullet that scopes the rule to `ContentModel::Generic` describe the pre-refactor state; they are superseded by decision 1 as amended here and by ADR-039 decision 8. The rest of the record is read with that replacement.
- **Not changed:** decisions 2-8, labels for every family, the single-document References disclosure, CLI list and view, JSON, record files and front matter, the presentation-only scope, and this record's accepted status.
## Context
REQ-006.019 removes the front-matter reference list from browser list rows for config-defined document types while keeping it in the single-document view and in CLI list and view output. REQ-006.016 still requires one shared list layout for every artifact family, and REQ-006.004 requires CLI and browser metadata parity. ADR-0028 decision 2 and downstream constraint 11 currently make the Title cell render labels and references for every family. ADR-0020 decision 5 repeats that for generic document lists. The implementation follows that contract: ListPlan::for_descriptor copies record refs into Row.references and the list renderer emits a reference list for every row. Forces: compact document-type rows, one shared layout with no bespoke template, no CLI or detail-view regression, no record or front-matter change, no new dependency or asset, and offline operation under NFR-001.003. Config-defined document types are exactly the descriptors whose content model is ContentModel::Generic (ADR-0018); every built-in and typed family uses Typed or HtmlMockup and stays unchanged.
## Consequences
Positive: document-type list rows stay compact without a second layout or a per-type branch; the scope follows the registry content model, so a newly configured document type gets the behaviour with no code change; CLI parity, detail views, and stored records are untouched; no asset, dependency, template, or network change is introduced. Negative: REQ-006.004 list parity is now qualified for generic document types by REQ-006.019, so TECH-045, TECH-068, TECH-074, and TECH-076 must state the qualification or they contradict the requirement; tests that assert references in every Title cell must split generic from non-generic families. Risks and mitigations: an over-broad rule would hide references for a family that must keep them, so the rule keys on ContentModel::Generic only and a contract test asserts references remain for every Typed and HtmlMockup family; a renderer that still emits an empty reference list is avoided because the block is omitted, not merely emptied; scope creep into the single-document view or CLI is excluded because neither is in the list rendering path and both are covered by regression tests.

**Amendment (2026-11-20) — descriptor model cross-check.** The risk bullet above that scopes the rule to `ContentModel::Generic` is superseded: the discriminator is the model plus the seed reference visibility (decision 1 as amended). The mitigation survives in that form — a contract test asserts that references are rendered for every family whose resolved value is `Visible` and omitted for `Hidden`, for both grouped roles.
