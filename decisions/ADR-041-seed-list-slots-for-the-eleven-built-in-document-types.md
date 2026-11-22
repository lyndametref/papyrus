---
id: ADR-041
title: "Seed list slots for the eleven built-in document types"
created_date: '2026-11-22T08:55:18Z'
updated_date: '2026-11-22T09:42:39Z'
labels: []
references:
  - RE-010
  - ADR-0028
  - ADR-0036
  - ADR-039
  - REQ-07.002
  - REQ-07.004
  - TECH-070
  - TECH-077
superseded_by: null
---
# ADR-041 Seed list slots for the eleven built-in document types

## Status
accepted
## Decision
### Scope

1. This ADR owns the seed `slots` of the eleven seed document types of ADR-036 decision 29: `req`, `nfr`, `glossary`, `rule`, `ui-spec`, `ui-mockup`, `audit`, `tech-spec`, `use-case`, `test-case`, `test-report`. `slots` is already seed data (ADR-036 decision 30); this ADR fixes its value per seed type. No field, field set, model, role, status set, reference visibility, or other presentation fact changes.

2. The declared sets reproduce today's browser output exactly and are frozen for the refactor: `src/browser/routes.rs::PRESENTATION` (`list_slots`) is the source of every value below, and the characterization corpus (`tests/fixtures/characterization/corpus/expected/http/list-*`) is the evidence. No slot is added, removed, or relabelled relative to that rendering.

3. Browser list columns are unchanged. The column order stays `id`, `title`, `status`, family slot(s) in declaration order, `created`, `updated`, `actions` (ADR-0028 decision 2 as amended by ADR-039 decision 5), and the slot cells keep their current values and `(none)` fallback.

### Seed slot sets

4. A seed entry declares exactly these slots, in this order, as `(key, label)` pairs:

| Slug | `slots` |
|---|---|
| `req` | `group` / `Group` |
| `nfr` | `group` / `Group` |
| `glossary` | none — `slots: []` |
| `rule` | `applicability` / `Applicability` |
| `ui-spec` | none — `slots: []` |
| `ui-mockup` | `task` / `Task`, `specs` / `Specs` |
| `audit` | `audit_type` / `Audit type` |
| `tech-spec` | none — `slots: []` |
| `use-case` | none — `slots: []` |
| `test-case` | none — `slots: []` |
| `test-report` | none — `slots: []` |

5. "None" means the seed entry declares an empty `slots` collection (`slots: []`), not an absent one: a present collection replaces the inherited one wholesale and an absent collection would inherit (ADR-036 decision 32). A user-defined type has no seed entry and no slots; this ADR does not change that default.

6. `req` and `nfr` are the only `GroupedMarkdown` seed types (ADR-036 decision 31). Their `group` slot is the engine-owned group link: the member's reserved `group` front-matter key, written and maintained only by the engine (ADR-036 decision 9). The slot is read-only, renders the existing group ID value, and is never a field, a CLI option, a form control, or a writable value; membership remains immutable.

### A slot key is not a field key

7. A slot key selects a list column. It is never a field key, a `frontmatter_key`, an option, a form-post key, or a field-map key, even when the two names coincide. A slot key may equal a declared field key (the existing `audit_type` and `applicability`) or a fixed-metadata key (`task` and `specs` for `SingleHtmlMockup`, ADR-039 decision 21); the coincidence grants the slot no field behaviour, and a field grants no column.

8. This ADR amends ADR-039 decision 6 for the `group` key only. The prohibited slot keys are the structural keys (`id`, `title`, `status`, `created`, `updated`, `actions`) and the reserved field keys of ADR-036 decision 23 (`id`, `title`, `status`, `labels`, `references`, `created_date`, `updated_date`, `group`), except that `group` is a permitted slot key for a `GroupedMarkdown` seed type, where it renders the engine-owned group link of decision 6 above. `group` remains reserved and prohibited as a field key and a `frontmatter_key` everywhere; every other reserved field key remains prohibited as a slot key, and a slot key may not repeat.

### Amendments to ADR-039

9. This ADR amends ADR-039 decisions 5, 6, and 25 to the extent set out in decisions 2, 4, and 8 above: a document descriptor's slots come from its seed entry and the seed sets are the table of decision 4, so the clauses "the seed declares none" and "currently none" no longer hold. Dated notes record this in ADR-039. Every other decision, downstream constraint, and clarification of ADR-039 stands unchanged.

10. This ADR supersedes no ADR. ADR-036, ADR-0028, and ADR-039 keep their accepted status.
## Consequences
**Positive:**

- MS-033 is unblocked: TASK-249 can seed `slots` from the table of decision 4, and its slot acceptance criterion needs no behaviour choice.
- The seed reproduces today's rendering exactly, so the characterization corpus keeps passing and the browser list columns are unchanged.
- The contradiction between ADR-039 and the code is closed without rewriting a frozen decision: the seed is the single slot source, and ADR-039 keeps its accepted status.
- The reserved-key rule survives with one narrow, named exception: `group` is a slot for grouped seed types and still can never be a field.

**Negative:**

- The seeded slot sets become architectural: changing a slot, its label, or its order now requires amending this ADR, not a cosmetic seed edit.
- ADR-039 carries a dated note that a reader must follow to reach the current slot sets.
- A slot key and a field key can share a name (`audit_type`, `applicability`), so a reviewer must keep the distinction of decision 7 in mind.

**Risks and mitigations:**

- *Slot drift from the corpus*: MS-033's characterization tests replay `expected/http/list-*`; the seed sets of decision 4 are asserted by those pages.
- *`group` re-enters as a field*: ADR-036 decision 23 keeps `group` reserved and prohibited for fields and `frontmatter_key`; only the slot position is permitted.
- *A later type gains a `group` slot*: decision 8 limits the exception to `GroupedMarkdown` seed types, and only `req` and `nfr` are grouped (ADR-036 decision 31).
