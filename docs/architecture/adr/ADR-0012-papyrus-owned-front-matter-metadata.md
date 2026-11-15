---
id: ADR-0012
title: "Papyrus-Owned Front Matter as the Single Record Metadata Source"
created_date: '2026-11-11T00:00:00Z'
updated_date: '2026-11-15T15:47:30Z'
labels: []
references: []
---
# Papyrus-Owned Front Matter as the Single Record Metadata Source

## Status

Superseded by ADR-0023 on 2026-11-13 — ADR-0023 restates the front-matter field map with `status` for milestone, release, and front-matter document types and the document `status` exemption from the create/update-option rule, which this ADR's field map and option rule do not carry. Body frozen; the metadata, patch, adoption, serialization, heading, and scoping decisions are restated in ADR-0023.

## Date

2026-11-11 (status: Accepted, 2026-11-11; status: Superseded, 2026-11-13)

## Context

REQ-05.010 requires every managed Markdown record to begin with a YAML front-matter block and fixes its field map; REQ-05.011 requires every command and every browser save to maintain it, and requires lazy per-record adoption of records that do not yet carry a block. REQ-01.005 now defines a single Papyrus ID space, and NFR-04.004 renounces Backlog.md compatibility. REQ-06.004 requires CLI and browser to read the same front-matter values.

ADR-0007 (Accepted) established front matter as the single metadata source and the patch-based writer, which remain correct. It also fixed three things that the new requirements remove: the task key set was declared to be "the complete Backlog.md key set", serialization had to stay Backlog-compatible for round trips, and the `id` and reference values of tasks and milestones were declared to be Backlog.md storage IDs. It also assumed NFR-04.003 interoperability and a legacy-to-Backlog body conversion. Those clauses contradict NFR-04.004 and REQ-01.005 and cannot be amended in place because an Accepted ADR body is frozen; this ADR supersedes ADR-0007 and restates the metadata contract without them.

Forces: one metadata source per field; lossless preservation of everything a command does not change; identical values in CLI and browser; a single ID space; no second parser or serializer; no new dependency (`yaml_serde` 0.10 is already used); frozen ADR bodies; and a corpus that still contains records without front matter, which must converge without a bulk rewrite.

## Decision

1. **A YAML front-matter block is mandatory for every Markdown record created or adopted by Papyrus.** The block starts at byte 0, opens with a line containing exactly `---`, and closes with a line containing exactly `---` followed by a blank line before the body. Nothing precedes it. Records created by Papyrus always carry it; existing records gain it on first read or first write (decision 9), so a record never read or written may still lack it, and the corpus converges record by record.

2. **Field map.** Every record type carries the general keys in this canonical order:

   `id`, `title`, `created_date`, `updated_date`, `labels`, `references`

   Type-specific keys follow the general keys:

   | Record type | Additional front-matter keys | Source |
   |---|---|---|
   | Task | `status`, `assignee` (YAML list), `milestone`, `dependencies`, `ordinal`, `documentation`, `type` | REQ-05.010 |
   | Milestone | `release` | REQ-05.010, REQ-03.002 |
   | Release | none (general keys only; REQ-03.002 defines no stored release state) | REQ-05.010 |
   | Decision | `superseded_by` (optional) | REQ-05.010 |
   | Audit record | `type` | REQ-05.010 |
   | Business requirement, NFR, glossary, UI spec, technical spec, business rule, use case, test case, test report | none | REQ-05.010 |
   | UI mockup | not applicable — HTML, excluded (decision 11) | REQ-05.008, REQ-05.010 |

   Additional project keys are tolerated and preserved (`priority` on tasks is the observed example). There is **no Backlog.md key-set contract**: Papyrus neither requires Backlog.md's key set nor validates against it, and a key present in an existing file is preserved whether or not Papyrus understands it.

3. **`id` and every reference value are Papyrus IDs.** This covers `id`, `milestone`, `dependencies`, `references` and `superseded_by` (REQ-01.005). No storage ID, no tool-specific form, and no translation at any boundary. The `id` value and the file name are consistent per ADR-0010; a disagreement is a validation error that refuses writes.

4. **Front matter is authoritative and is the only metadata store.** Values are never mirrored into a `## ` section. These body sections are retired as metadata carriers: task `## Status`, `## Assignee`, `## Milestone`, `## Labels`, `## Dependencies`; milestone `## Release`; audit `## Audit Type`; requirement `## Group` is **not** affected (it is content, not front matter). New files never write the retired sections; a write to an existing file moves their values into front matter and removes the section, so no field is stored twice. Content sections are retained unchanged: `## Description`, `## User Story`, `## Group`, `## Acceptance Criteria`, `## Comments`, `## Implementation Notes`, `## Definition of Done`, `## Notes`, `## Definition`, `## Rationale`, `## Applicability`, `## Requirements`, `## View`, `## Content`, `## Decision`, `## Context`, `## Consequences`.

5. **Heading policy.**

   - Tasks and milestones: no `#` heading. Their bodies use the record's section layout.
   - Every other Markdown record type: a `# <ID> <title>` body title line is generated by the writer from the front-matter `id` and `title`, so the two cannot diverge; readers take both values from front matter. The heading is retained because accepted ADR bodies are frozen, and because existing readers, search excerpts, and documentation indexes rely on it.
   - Frozen ADR bodies keep their existing heading and sections; the added front-matter block is the only change made to an existing decision file.

6. **One shared front-matter implementation owns parsing and serialization** (`src/record/frontmatter`, layout in ADR-0013): detection and separation of the block, a parse result that keeps key order, raw scalar text and unknown keys, typed accessors backed by one key registry, and a patch operation that rewrites only changed keys while preserving unknown keys, key order and comments. No record module and no browser code may hand-format a `---` block, re-serialize an untouched block, or split front matter with ad-hoc string parsing.

7. **Serialization is Papyrus-documented and Backlog-independent.** Quoting and key order are fixed by Papyrus and covered by its own golden tests; they are no longer required to match Backlog.md, and no round-trip against Backlog.md is a gate. Parsing and serialization use the existing `yaml_serde` dependency; full re-serialization that reorders or requotes foreign keys remains prohibited because it would defeat "preserve what the operation does not change".

8. **Timestamps.** `created_date` and `updated_date` are quoted ISO-8601 UTC strings at second precision, `'YYYY-MM-DDTHH:MM:SSZ'`. `created_date` is written once at creation and never changed later; `updated_date` is refreshed by every write path (create, edit, archive, restore, deprecate, supersede, browser save). A save that changes nothing does not refresh it because no write happens. One clock value is read per write (ADR-0013). A small in-project UTC formatter is used; no new dependency.

9. **Existing records are adopted lazily, per record, on first read and first write.** There is no bulk pass and no migration command; the corpus converges record by record. Adoption:
   - **adds the block only** — it never re-renders, re-shapes, renames or relocates a record, and it never converts a task or milestone body;
   - **may retire legacy metadata sections** on a write, after moving their values into front matter (decision 4), so no field is stored twice;
   - **is idempotent** — a record that already carries front matter is a byte-identical no-op on both read and write;
   - **is scoped to descriptor-claimed files** — task, milestone, release, decision, business requirement, non-functional requirement, glossary term, UI specification, technical specification, business rule, use case, test case, test report, audit record, deployment record. Prose Markdown that no descriptor claims (`docs/specifications/`, `docs/domain-model/`, `docs/business-processes/`, `README.md`, workflow docs) and UI mockups/HTML are excluded;
   - **is metadata-only for frozen records** — an Accepted ADR gains the block and nothing else;
   - **derives missing values best-effort** — `created_date` from the record's own `## Date` section where the type has one, otherwise from the file modification time; `updated_date` from the adoption time; both documented as approximate, never recomputed later;
   - **must not partially write** — if the patch cannot be applied (read-only media, permissions), the reader reports the error and leaves the file byte-identical; two concurrent first readers serialize through the atomic write contract and the second becomes a no-op;
   - **is reviewable** — a dry run and a per-file inventory (path, action, resulting keys) remain available as a reporting and verification aid, not as a prerequisite step;
   - **no longer converts anything to the Backlog.md layout** — the legacy task/milestone body conversion of ADR-0007/ADR-0008 and the lazy Backlog-layout relocation are removed (NFR-04.004, REQ-05.011). A Backlog.md record enters Papyrus only through the one-way import (ADR-0011).

10. **`id`, `created_date` and `updated_date` are exempt from the "every field is a create/update option" rule.** Every other front-matter key of a type is exposed as a create option and an update option. `ordinal` is an opaque non-negative integer with a create option, an update option, and a default derived from the current maximum. A key a command never changes must still be preserved, which is the patch contract in decision 6.

11. **UI mockups are excluded from front matter.** REQ-05.008 fixes mockup metadata as HTML comments inside the HTML file, and an unwrapped `---` block at the top of an HTML file is invalid markup. Mockup HTML files carry no front matter and keep REQ-05.008 as their single metadata source.

12. **Decision `status` and decision `type` (`adr`/`bdr`) stay in the document core.** ADR status carries free-text rationale and cannot move without editing frozen bodies; `type` is already derived from the ID prefix. The key registry remains extensible: moving either later is a registry entry plus a section-retirement rule.

13. **Backfilling front matter is a metadata-only change and is permitted on frozen records**, including Accepted ADRs, because it does not modify the governed body. `id` is backfilled from the existing record identity (front matter, heading, or file name) so REQ-01.005 stability holds. Adoption under decision 9 runs on this rule.

14. **Decision front matter is additive to `## Date` and `## References`.** Those sections remain part of the body and are neither replaced nor moved; `created_date` does not duplicate the `## Date` line, and neither section becomes an editable metadata source. New decisions write `created_date` and `## Date` from one clock value.

15. **Non-record Markdown is out of scope.** A file is a record only when a descriptor claims its directory and filename/ID pattern. Documentation that is not a managed record type never gains front matter implicitly.

## Consequences

**Positive:**

- One metadata source serves CLI, browser, search, and the import; REQ-06.004's "same values" becomes structural.
- Field-preserving writes are expressible as a patch, which is the only way to satisfy "preserve untouched front matter" and it is shared by CLI, browser, adoption, and import.
- The contract no longer depends on another tool's key set or serialization, so Papyrus can evolve its metadata without a compatibility gate.
- The ID value in front matter is the only identity, which removes a whole class of translation bugs and documentation ambiguity.
- Adoption remains cheap and observable: per record, idempotent, and reviewable through the inventory.

**Negative:**

- The corpus temporarily contains records with and without front matter; readers keep the legacy fallback until convergence.
- A task or milestone that still carries legacy `## Status`-style sections loses them on its next write (values move to front matter); the file's shape changes once.
- Removing Backlog.md serialization compatibility means Papyrus can no longer guarantee that another tool reads its front matter; that is the accepted product direction (NFR-04.004).
- Adoption writes on read, so read-only media or missing permissions surface as errors on read.

**Risks and mitigations:**

- *Front matter destroyed by a writer that re-renders the file*: the patch contract, the single writer (ADR-0013), and a byte-identity test on no-op edits.
- *Unknown keys dropped*: the parse result keeps unknown keys and order; a patch never rebuilds the block.
- *Duplicate storage of a field*: a retired section is removed only when its values moved to front matter; a write that cannot move a value keeps the section and reports.
- *Timestamp drift or non-UTC values*: one formatter, one clock value per write, frozen `created_date`.
- *Adoption damaging a body*: adoption is front-matter-only; frozen ADR bodies are byte-identical apart from the block; legacy task/milestone sections are retired through explicit patch operations, never by re-rendering.
- *ID/file-name divergence*: flagged invalid, reads from front matter, writes refused (ADR-0010 decision 4).
- *Approximate `created_date` mistaken for truth*: documented as best-effort and never recomputed.

## Downstream constraints

1. Remove every rule that treats the Backlog.md key set, Backlog.md quoting, or a Backlog.md round trip as a contract: no key-set validation against `backlog/config.yml`, no compatibility tests as an acceptance gate, no "Backlog-compatible" wording in help, man page, or specifications.
2. `id`, `milestone`, `dependencies`, `references`, `superseded_by` are Papyrus IDs everywhere: parsing, serialization, validation, display, JSON, links. Tests must assert that no surface prints or accepts a storage ID.
3. Adoption must remain front-matter-only: delete the legacy-to-Backlog body conversion path and the lazy Backlog-layout relocation together with their tests and fixtures (ADR-0013 stage plan).
4. New or changed record types are added by a descriptor plus a key-registry entry, never by a new parser or serializer.
5. `docs/specifications/migrate-document-record-types-to-front-matter-lazily-on-first-read-and-first-write.md`, `docs/specifications/migrate-tasks-and-milestones-to-the-backlog-md-layout.md`, `docs/specifications/expose-front-matter-metadata-options-on-all-record-commands.md`, and `docs/specifications/render-one-metadata-projection-in-cli-and-browser-views.md` must be revised where they state Backlog.md storage IDs, offsets, or body conversion.

## Alternatives Considered

| Option | Outcome | Reason |
|---|---|---|
| Papyrus-owned front matter, Papyrus IDs, patch writer, lazy front-matter-only adoption (this ADR) | Selected | Meets REQ-05.010/05.011, REQ-01.005 and NFR-04.004 together, and keeps the proven patch architecture. |
| Keep ADR-0007 as-is and only change code | Rejected | An Accepted ADR that mandates Backlog.md storage IDs and serialization would still contradict the requirements; the ADR corpus must not contain accepted decisions the product has renounced. |
| Remove front matter and keep `## ` sections as metadata | Rejected | Two sources that diverge; contradicts REQ-05.010 and the browser metadata contract. |
| Front matter as a read-only cache regenerated from the body | Rejected | Hand-edited values would be lost; contradicts "front matter is authoritative". |
| Bulk one-time adoption of the whole corpus | Rejected by product decision 2026-11-08 | Large single diff, needs a clean tree, and a later-upgraded project needs the same pass anyway. |
| TOML/JSON metadata block | Rejected | Not what REQ-05.010 asks for; needs new parsing and alienates Markdown tooling. |
| Keep the legacy task/milestone body conversion as a Papyrus-native cleanup | Rejected | It exists only to reach the Backlog.md shape, which is renounced; it is also the only path that rewrites a task body. |

## References

- `docs/requirements/business/REQ-05.010.md` — field map and record format.
- `docs/requirements/business/REQ-05.011.md` — metadata maintenance and lazy adoption; Backlog clauses removed.
- `docs/requirements/business/REQ-01.005.md` — single Papyrus ID space.
- `docs/requirements/business/REQ-06.004.md` — front-matter metadata in CLI and browser.
- `docs/requirements/non-functional/NFR-04.004.md` — compatibility renounced; import is the only bridge.
- `docs/architecture/adr/ADR-0007-yaml-front-matter-as-single-record-metadata-source.md` — superseded by this ADR.
- `docs/architecture/adr/ADR-0010-papyrus-native-on-disk-record-layout.md` — file layout and identity.
- `docs/architecture/adr/ADR-0011-one-way-backlog-md-import.md` — how Backlog.md records enter Papyrus.
- `docs/architecture/adr/ADR-0013-shared-record-envelope-without-a-storage-layer.md` — module and write-path structure.
- Implementation: `src/record/frontmatter.rs`, `src/record/patch.rs`, `src/record/migrate.rs`, `src/record/document.rs`.
