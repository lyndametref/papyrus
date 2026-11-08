# YAML Front Matter as the Single Record Metadata Source

## Status

Accepted — approved by explicit user confirmation on 2026-11-08

## Date

2026-11-08 (status: Accepted, 2026-11-08)

## Context

REQ-05.010 (front matter on every managed Markdown record), REQ-05.011 (metadata maintained by every command, lazy per-record migration on first read and first write), REQ-06.003 (browser page describing record types and lifecycles), REQ-06.004 (CLI and browser read front matter and write it back), REQ-01.005 (two ID spaces with translation), and NFR-04.003 (task and milestone interoperability with Backlog.md) are recorded. The behavior contract is the requirement files; this ADR fixes how record metadata is represented and where it lives.

Current implementation shape (verified in `src/` and the corpus):

- Every record module (`requirement`, `task`, `milestone`, `release`, `decision`, `glossary`, `rule`, `ui_spec`, `ui_mockup`, `audit`, `tech_spec`) owns its own heading parser, `## ` section parser, renderer, and `fs::write` call. Metadata such as task `status`, `assignee`, `milestone`, `labels`, `dependencies`, milestone `release`, and audit `type` is stored as `## ` sections of the Markdown body.
- `browser::records::parse_doc` treats line 1 of every file as the `# <ID> <title>` heading and knows nothing about front matter. Its `apply_*_edit` functions rebuild the file as `# heading` + preamble + `## ` sections, so a leading front-matter block would be destroyed (and is currently misparsed as the heading).
- The corpus starts in three shapes: (a) no Markdown record under `docs/` carries front matter today (144 `.md` files, zero leading `---` blocks); (b) `backlog/tasks/*.md` already carry the complete Backlog.md key set; (c) `backlog/milestones/*.md` carry only `id` and `title`.
- `yaml_serde` 0.10 is already a runtime dependency (config load and init), so YAML parsing needs no new crate.
- Accepted ADR bodies are immutable ("Body frozen per ADR lifecycle"), and ADR files live under `docs/architecture/adr/`; decision records are therefore a managed record type whose body cannot be re-rendered from a model.

The forces are: one metadata source per field (no double storage), lossless preservation of everything a command does not change, the same values in CLI and browser, Backlog.md round-trip for tasks and milestones, a single visible migration instead of an indefinite mixed corpus, and no new runtime dependency or second storage format.

## Decision

1. **A YAML front-matter block is mandatory for every Markdown record created or migrated by Papyrus.** The block starts at byte 0 of the file, opens with a line containing exactly `---`, and closes with a line containing exactly `---` followed by a blank line before the body. Nothing precedes it. Records created by Papyrus always carry it; existing records gain it on first read or first write (decision 9), so a record that has never been read or written may still lack it, and the corpus converges record by record.

2. **Field map.** Every record type carries the general keys, in this canonical order:

   `id`, `title`, `created_date`, `updated_date`, `labels`, `references`

   Type-specific keys are added after the general keys:

   | Record type | Additional front-matter keys | Source requirement |
   |---|---|---|
   | Task | `status`, `assignee` (YAML list), `milestone`, `dependencies`, `ordinal`, `documentation`, `type` — the complete Backlog.md key set | REQ-05.010, NFR-04.003 |
   | Milestone | `release` (additive over Backlog.md's `id` and `title`) | REQ-05.010, NFR-04.003 |
   | Release | none (general keys only; REQ-03.002 defines no stored release state) | REQ-05.010 |
   | Decision | `superseded_by` (optional) | REQ-05.010 |
   | Audit record | `type` | REQ-05.010 |
   | Business requirement, NFR, glossary, UI spec, tech spec, business rule | none | REQ-05.010 |
   | UI mockup | not applicable — HTML, excluded (decision 11) | REQ-05.008, REQ-05.010 |

   For tasks and milestones, the `id` value is the Backlog.md storage ID (`TASK-18`, `m-0`), not the Papyrus display ID (`TASK-018`, `MS-001`); the same applies to reference-valued keys `milestone` and `dependencies`. ADR-0008 owns that mapping.

3. **Front matter is authoritative and is the only metadata store.** Values are never mirrored into a `## ` section. The following body sections are retired as metadata carriers: task `## Status`, `## Assignee`, `## Milestone`, `## Labels`, `## Dependencies`; milestone `## Release`; audit `## Audit Type`. New files never write them; a write to an existing file removes them after moving their values into front matter. Body sections that express document content are retained unchanged: `## Description`, `## User Story`, `## Group`, `## Acceptance Criteria`, `## Comments`, `## Implementation Notes`, `## Definition`, `## Rationale`, `## Applicability`, `## Requirements` (business-rule traceability, not general references), `## View`, `## Content`, `## Decision`, `## Context`, `## Consequences`.

4. **The `# <ID> <title>` heading policy differs by record family (product-confirmed).**
   - Tasks and milestones: no `#` heading. The Backlog.md body has none and NFR-04.003 requires the Backlog.md body shape. ADR-0008 owns the exact body layout.
   - All other Markdown record types: the `# <ID> <title>` line is retained as the body title line because accepted ADR bodies are frozen and cannot lose it, because existing readers, search excerpts, and the documentation indexes rely on it, and because a human opening the file benefits from it. It is *not* an independent metadata source: readers take `id` and `title` from front matter; every writer regenerates the heading from the front-matter values so the two cannot diverge. For record types whose `id` is a Papyrus ID, the heading shows the Papyrus ID (ADR-0008 decision 4).
   - Frozen ADR bodies keep their existing heading and sections; the added block is the only change made to an existing decision file.

5. **One shared front-matter implementation owns all parsing and serialization.** A `src/record/frontmatter` module (layout fixed by ADR-0009) provides:
   - detection of the leading block and separation of `{ raw_front_matter, body }`;
   - a parse result that keeps key order, raw scalar text, and unknown keys;
   - typed accessors for the general keys and per-type extras declared in one key registry;
   - serialization that emits the canonical key order and Backlog-compatible quoting, and a **patch** operation that rewrites only the changed keys while preserving unknown keys, key order, and comments.
   No record module and no browser code may hand-format a `---` block, re-serialize an untouched block, or parse front matter with ad-hoc string splitting.

6. **YAML handling uses the existing `yaml_serde` dependency.** Reading parses the block with `yaml_serde`; writing serializes only scalar/list values under Papyrus control and patches them into the existing block text. A full document re-serialization that would reorder or requote foreign keys is prohibited because it breaks NFR-04.003 round trips.

7. **Timestamp format.** `created_date` and `updated_date` are quoted ISO-8601 UTC strings at second precision: `'YYYY-MM-DDTHH:MM:SSZ'` (matching observed Backlog.md values). `created_date` is written once at creation and never changed by a later command. `updated_date` is refreshed by every write path (create, edit, archive, restore, deprecate, supersede, browser save). A save that changes nothing does not refresh it because no write happens. A small in-project UTC formatter is used; no new time-formatting dependency is added (ADR-0001 toolchain, ADR-0005 dependency baseline).

8. **Reading is front-matter-first with fallbacks.** Parsing order for a record field: front-matter key → legacy `## ` section → heading → default. This keeps unmigrated files readable during the change and makes Backlog.md files readable immediately. For tasks and milestones, ID resolution accepts both ID spaces and normalizes before comparison (ADR-0008 decision 4).

9. **Existing records are migrated lazily, per record, on first read and first write (product decision, revised 2026-11-08).** There is no bulk pass and no dedicated migration command: the first read or first write of a managed Markdown record adds the block to that file if it lacks one, and the corpus converges record by record. Migration is idempotent: once a record carries front matter, neither a later read nor a later write re-runs it.
   - **Scope.** Only files claimed by a record descriptor are migrated (decision 15). The managed record types are: task, milestone, release, decision, business requirement, non-functional requirement, glossary term, UI specification, technical specification, business rule, audit record. Prose Markdown that is not a managed record (`docs/specifications/`, `docs/domain-model/`, `docs/business-processes/`, `README.md`, workflow docs) and UI mockups/HTML are excluded.
   - **Bodies are untouched.** A frozen ADR body keeps its heading, sections, and prose; only the block is inserted. Task and milestone files already in the Backlog.md storage shape keep their bodies and every marker comment byte-identical; records whose front matter is already complete are not rewritten at all. A task or milestone still in the legacy Papyrus shape (heading plus `## ` metadata sections) is converted to the storage shape on its first read or first write, which is a body change beyond the block; ADR-0008 decision 8 owns that case, and it is the only migration path allowed to alter a task/milestone body.
   - **Metadata-only and idempotent.** Migration is expressed as a front-matter-only patch through the shared patcher (ADR-0009); it never re-renders a body. A record that already carries front matter is a byte-identical no-op on both read and write, and this is part of the acceptance test. Lazy migration on a read path must not corrupt or partially write the file if the patch cannot be applied; the read-path write implications are recorded in Consequences.
   - **Values with no stored source.** `created_date` is taken from the record's own date section where the type has one (decision `## Date`), otherwise from the file's modification time; `updated_date` is the migration time for every file whose block is created or extended. This is best-effort provenance, not a historical claim, and is documented as such.
   - **Ordering relative to the shared envelope (ADR-0009).** Lazy migration runs only after the shared reader/patcher and the task/milestone ID translation exist, because the migration must not contain a second parser and because task/milestone blocks carry storage IDs. Task and milestone conversion additionally requires Stage 2 identity (ADR-0009 decision 8); document record types need only Stage 1. There is no within-pass ordering because there is no pass.
   - **Reviewable.** A dry run and a per-file inventory (path, action, resulting keys) remain available as a reporting and verification aid over the descriptor-claimed corpus, but they are not a prerequisite bulk step; the corpus converges record by record as records are read or written. The inventory is the evidence that the descriptor set covers the intended corpus and nothing more. No migration command is required for convergence.

10. **`id`, `created_date`, and `updated_date` are exempt from the "every field is a create/update option" rule of REQ-05.011.** `id` is generated and stable (REQ-01.005), the timestamps are automatic; every other front-matter key for a type is exposed as a create option and an update option on that type's commands. `ordinal` is an opaque non-negative integer with a create option, an update option, and a default derived from the current maximum. A key that a command never changes still has to be *preserved*, which is the patch contract in 5.

11. **UI mockups are excluded from front matter (product decision).** REQ-05.008 fixes mockup metadata as HTML comments (`mockup-id`, `status`, `task`, `specs`, `refs`) inside the HTML file, and REQ-05.010 scopes front matter to Markdown. An unwrapped `---` block at the top of an HTML file is invalid markup and would be rendered as text; wrapping it in an HTML comment creates a format no generic YAML reader accepts. Mockup HTML files therefore carry no front matter and keep REQ-05.008 as their single metadata source. This exclusion is final for this ADR, not provisional.

12. **Decision `status` and decision `type` (`adr`/`bdr`) stay in the document core (product decision: no change).** ADR status is a lifecycle value with free-text rationale ("Accepted — approved by explicit user confirmation on 2026-11-04 (TASK-28)") that cannot be moved without editing frozen bodies, and `type` is already derived from the ID prefix. Deprecation state stays expressed by directory placement and mockup status stays in HTML comments. The key registry in 5 remains extensible: moving either later is a registry entry plus a section-retirement rule, not a redesign.

13. **Backfilling front matter is a metadata-only change and is permitted on frozen records.** Adding a block above an existing record body — including an Accepted ADR — does not modify the body it governs. It therefore does not require superseding the ADR and does not violate the ADR immutability rule, which freezes Context, Decision, and Consequences. `id` is backfilled from the existing record ID (heading or filename) so stability under REQ-01.005 holds; for tasks and milestones the backfilled `id` is the storage ID (ADR-0008). Lazy migration of decision 9 is performed under this rule.

14. **Decision `created_date` and `references` are additive to the existing sections (product decision).** For decisions, `## Date` and `## References` remain part of the body and are neither replaced nor moved: front matter for decisions is additive. `created_date` does not replace or duplicate the ADR `## Date` line, and neither section becomes an editable metadata source. New decisions write `created_date` and `## Date` from one clock value so they agree; existing files keep the frozen prose. Readers use front matter first and fall back to the section.

15. **Non-record Markdown is out of scope for front matter.** A file is a record only when a record descriptor claims its directory and filename/ID pattern. Documentation that is not a managed record type (`docs/index.html` sources, prose specifications unless a descriptor claims them, domain-model prose, business-process descriptions, workflow documentation, `README.md`) is not migrated and never gains front matter implicitly. The migration inventory from decision 9 is the evidence that the descriptor set covers the intended corpus and nothing more.

## Consequences

**Positive:**

- One metadata source: CLI list/view, browser list/detail, and search read the same keys, satisfying REQ-06.004 without a synchronization layer.
- Field-preserving writes are expressible as a patch, which is the only way to satisfy "preserve untouched front matter" and Backlog.md round-trip in the same writer.
- No new dependency: `yaml_serde` is already used.
- `created_date`/`updated_date` become real record data instead of file mtime or nothing, removing the current `SystemTime`-based task sort/list approximations.
- Migration is per record and idempotent, so the corpus converges record by record as records are read or written; readers still need the legacy fallback for unread records, and the fallback can be deleted only after convergence is verified.
- Provenance of approximated values (`created_date` from file mtime) is recorded once per record, at that record's first migration, instead of being silently invented on every first edit.

**Negative:**

- Every record module's parser, renderer, and writer is affected; this is a cross-cutting change, not a per-type feature.
- The heading exception (decision 4) leaves `title` present twice for document records; it is bounded, writer-enforced, and justified by frozen ADR bodies.
- YAML quoting must match Backlog.md closely enough for its reader; hand-rolled serialization of changed keys has to be tested against Backlog.md, not only against our own parser.
- Body sections disappear from files that users may have hand-edited; a user who wrote `## Status` manually loses that section on the next Papyrus write (its value moves to front matter).
- Migration now happens on the read and write paths: a read can trigger a write (adding front matter, or converting a legacy task/milestone body), which has implications for read-only filesystems, file permissions, atomicity on the read path, and concurrency (see Risks). There is no single reviewable corpus-wide diff; convergence is observed per record.
- `created_date` for pre-existing documents is approximate (file mtime) and will be visibly earlier or later than the real creation date in some files.
- Tasks already carrying a complete Backlog.md block are intentionally not rewritten, so the migration is not uniform across task and document files.

**Risks and mitigations:**

- *Divergent quoting/order breaking Backlog.md*: round-trip tests in both directions are the acceptance gate for NFR-04.003 (ADR-0008 defines the contract).
- *Front matter destroyed by a writer that re-renders the file*: the patch contract plus a test that an untouched file byte-compares equal after a no-op edit; ADR-0009 makes the shared writer the only write path.
- *Unknown/foreign keys dropped*: the parse result keeps unknown keys and their order; a patch never rebuilds the block.
- *Timestamp drift or non-UTC values*: single formatter, single clock call per write, frozen `created_date`.
- *Lazy migration damaging a body*: migration is patcher-only, idempotent, with byte-identity tests on task/milestone bodies already in Backlog.md shape, before/after diffs for converted legacy bodies, and an idempotency test; no body regeneration path exists. A read-path migration that cannot be applied safely must leave the file untouched and report the error rather than partially write.
- *Lazy migration touching the wrong files*: the descriptor set (decision 15) keeps non-record documentation out; the dry-run inventory is the evidence that the descriptor set covers the intended corpus and nothing more. A file is migrated only when a managed record type claims it.
- *Read-path writes and read-only media*: a first read may need to write front matter (or convert a legacy body). On a read-only filesystem, or when the process lacks write permission, the migration cannot complete; the reader must surface the error and must not report a migrated record. Callers that only intend to read should be prepared for a write failure. Atomicity on the read path follows ADR-0006 (create_new + sync_all + same-directory rename); a failed migration leaves the original file byte-identical. Concurrency: two processes reading the same unmigrated record concurrently may both attempt to migrate it; the atomic write plus version/lock contract serializes them, and the second writer observes the already-migrated file and becomes a no-op. An in-flight reader that loaded the pre-migration bytes is not silently upgraded.
- *Search behavior change*: front matter becomes part of indexed text, so searches can now match metadata values; this is consistent with "searchable text is whole persisted record text" and must be stated in the search specification.
- *Approximate `created_date` mistaken for truth*: documented as best-effort in the specification and never recomputed by later writes.

## Open questions (product decisions, not decided here)

1. Resolved (user decision 2026-11-08): migration covers only files claimed by a Papyrus managed record type — task, milestone, release, decision, business requirement, non-functional requirement, glossary term, UI specification, technical specification, business rule, audit record. Prose Markdown that is not a managed record (`docs/specifications/`, `docs/domain-model/`, `docs/business-processes/`, `README.md`, workflow docs) and UI mockups/HTML are out of scope. Migration mode is now lazy per record on first read and first write (user decision 2026-11-08, revised); there is no dedicated `papyrus migrate` subcommand. Non-blocking for the architecture.

## Alternatives Considered

| Option | Outcome | Reason |
|---|---|---|
| YAML front matter as the only metadata store, parsed by one shared module (this ADR) | Selected | Meets REQ-05.010/05.011/06.004, is what Backlog.md already does, and needs no new dependency. |
| Keep metadata in `## ` sections and mirror it into front matter | Rejected | Two sources that can diverge; violates "metadata is not duplicated". |
| Front matter as a read-only cache regenerated from the body on every write | Rejected | A hand-edited or Backlog-edited front matter value would be lost; breaks round-trip. |
| TOML or JSON metadata block | Rejected | Not Backlog.md-compatible; NFR-04.003 fails. |
| One metadata block per record type with a per-type parser | Rejected | Repeats the current duplication and guarantees drift between CLI and browser. |
| Remove all `#` headings and rely on front matter only | Rejected | Impossible for frozen ADR bodies and costly for existing docs/indexers; the heading is retained as a writer-generated body title line (4). |
| Lazy upgrade on first read and first write (this ADR, revised 2026-11-08) | Selected | Corpus converges record by record as records are read or written; readers keep the legacy fallback only until convergence; idempotent, no dedicated command, and no single large diff. Satisfies REQ-05.011 as revised. |
| One-time bulk migration of the whole corpus (earlier product decision) | Rejected by product decision 2026-11-08 | Requires a clean git tree and produces a large single diff; a later-upgraded project still needs the same pass. Lazy per-record migration achieves the same end state without a dedicated command and keeps readers robust to both shapes. |
| Migrate only metadata-bearing files, leave read-only docs alone | Rejected | Creates an unpredictable corpus where "has front matter" does not mean "is a record"; descriptor scope (15) is the explicit boundary instead. |
| YAML front matter in HTML mockups, wrapped in an HTML comment | Rejected by product decision | Non-standard format that neither generic YAML tools nor browsers handle; REQ-05.008 HTML metadata stays authoritative. |

## References

- `docs/requirements/business/REQ-05.010.md` — Record Metadata in YAML Front Matter.
- `docs/requirements/business/REQ-05.011.md` — Maintain Record Metadata Through All Commands.
- `docs/requirements/business/REQ-06.003.md` — Describe Managed Artifacts and Their Lifecycles in the Browser.
- `docs/requirements/business/REQ-06.004.md` — Use Front-Matter Metadata in CLI and Browser Views.
- `docs/requirements/business/REQ-01.005.md` — Assign Stable Record IDs (two ID spaces, translation).
- `docs/requirements/non-functional/NFR-04.003.md` — Backlog.md task and milestone interoperability.
- `docs/requirements/business/REQ-05.008.md` — UI mockup metadata in HTML content.
- `docs/architecture/adr/ADR-0006-optimistic-concurrency-and-atomic-record-writes.md` — existing write contract and its deferred CLI alignment.
- `docs/architecture/adr/ADR-0008-adopt-backlog-md-on-disk-layout-for-tasks-and-milestones.md` — Backlog.md layout, storage IDs, and ID translation.
- `docs/architecture/adr/ADR-0009-shared-record-envelope-and-unified-write-path.md` — single parse/patch/write implementation and migration staging.
- Implementation: `src/browser/records.rs` (`parse_doc`, `apply_*_edit`), `src/task.rs` (`parse_task`, `render_task`), `src/milestone.rs`, `src/decision.rs`, `src/audit.rs`, `src/rule.rs`, `src/requirement.rs`; configuration parsing already using `yaml_serde` in `src/commands.rs` and `src/init.rs`.
- Corpus evidence: `backlog/tasks/task-18 - ....md` (complete Backlog.md block), `backlog/milestones/m-0 - ....md` (`id`, `title` only), `docs/**/*.md` (no front matter before migration).
