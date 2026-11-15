# Consolidated Record Layout and Front-Matter Field Map

## Status

Accepted (2026-11-13) — accepted by explicit user decision of 2026-11-13; amended 2026-11-15 by owner decision (decision 2 release archive location) without status change; amended again 2026-11-15 by owner decision (decision 2 milestone archive location, decision 3, decision 4 legacy alias qualification, downstream constraint 2, and the decision 9 supersession bullet) without status change.

## Date

2026-11-13 (status: Proposed, 2026-11-13; status: Accepted, 2026-11-13)

## Context

Two accepted ADRs still state the on-disk layout and the front-matter field map with clauses the product has since removed:

- **ADR-0010** (native record layout) lists `<doc_directory>/<type directory>/deprecated/` as the idle location for document records (decision 2) and defines "archive, restore, deprecate, restore are moves" (decision 7), with allocation scanning deprecated locations (decision 5).
- **ADR-0012** (front-matter metadata) maps milestones to `release` only, releases to no additional key, and document types to no additional key (decision 2), so it cannot express the `status` field that REQ-05.010 now requires for milestones, releases, and front-matter documents (decision 10's option rule is likewise silent on the document `status` exemption).

REQ-05.007 replaced the document lifecycle `active -> deprecated` with `active -> archived`, moved the inactive location to `archived/`, removed every document restore command, and made `deprecated/` unread and unmigrated. REQ-05.010/REQ-05.011 added `status` to milestone, release, and front-matter document records and exempted the document `status` from the create/update-option rule. REQ-07.001 added per-type custom statuses that are front-matter values only. REQ-07.002/REQ-07.003 made document types configuration-defined. ADR-0022 recorded the lifecycle delta (status model, archive operation, no restore) but explicitly left a consolidation ADR outstanding: two accepted ADRs cannot keep contradicting the requirements, and an implementer reading ADR-0010 or ADR-0012 alone would build a `deprecated/` folder and restore commands.

The owner decided on 2026-11-13 to consolidate rather than patch further: one ADR restates the layout and the field map as they are now in force, and the two obsolete records become Superseded. Their still-valid decisions are restated here and remain authoritative through this ADR; their bodies stay frozen and are not edited.

Forces: one identity, one file, one name (ADR-0010 decision 1); configuration-driven paths with project-directory containment (NFR-04.002); atomic, non-destructive moves (ADR-0006); a single writer and patch API (ADR-0013); one metadata source per field (ADR-0012); adoption without a bulk migration (REQ-05.011); fixed milestone/release/decision lifecycles and configurable task/document status vocabularies (REQ-07.001); and no clause left in the corpus that an implementer could follow into a defect.

## Decision

1. **This ADR is the single authoritative statement of the on-disk layout and the front-matter field map.** It supersedes ADR-0010 and ADR-0012. ADR-0022 remains Accepted and in force for the lifecycle *operations* (status semantics, archive sequence, `status` command, delete, no restore); this ADR consolidates the layout and metadata contract that ADR-0022 amended. Nothing here overrides ADR-0011 (import), ADR-0006 (atomic write and version token), or ADR-0013 (envelope, patch, single writer).

2. **Layout: configuration-driven active and archive locations.**

   | Record family | Active location | Archive location |
   |---|---|---|
   | Task | `<backlog_directory>/tasks/` | `<backlog_directory>/archive/tasks/` |
   | Milestone | `<backlog_directory>/milestones/` | none — never moved by a milestone command; its state is the front-matter status (`open` or `closed`) |
   | Release | `<backlog_directory>/releases/` | none — the record is never moved; its state is the front-matter status |
   | Decision (ADR, BDR) | `<decision_directory>/` | none — the record is never moved; its state is the `## Status` line, which keeps it in the ADR index |
   | Document, any configured type (`req`, `nfr`, `glossary`, `rule`, `ui-spec`, `ui-mockup`, `audit`, `tech-spec`, `use-case`, `test-case`, `test-report`, and every user-added slug) | `<doc_directory>/<type directory>/` | `<doc_directory>/<type directory>/archived/` |

   `backlog_directory`, `decision_directory`, and `doc_directory` are relative to the project directory; a document type's `directory` is relative to `doc_directory`. Descriptors name configuration keys, never literal paths. Every resolved path passes `Context::ensure_within` before any read or write (NFR-04.002), including after symlink resolution. UI mockups are HTML records under their configured `ui-mockup` directory; they use the same active/`archived/` layout and keep their state in HTML metadata (decision 5).

   A release has `ArchivePolicy::None`: `release close` rewrites `status` from `planned` to `released` in place through the single write path, and no `archive/releases/` path is resolved by any code path or descriptor.

   *Revision (2026-11-15, owner decision): the Release row previously listed `<backlog_directory>/archive/releases/` as its archive location. Releases never move — the descriptor carries `ArchivePolicy::None` and the record's state is its front-matter `status`, so the row and the sentence above now record the delivered contract.*

   A milestone likewise has no archive location and no milestone command moves the record: `papyrus milestone close <id>` rewrites `status` from `open` to `closed` and refreshes `updated_date` in place through the single write path (REQ-03.001, ADR-0022 decision 5), and the milestone descriptor carries `ArchivePolicy::None`. The legacy `<backlog_directory>/archive/milestones/` folder is a **read/import location only**: the milestone resolver still reads it, and the one-way Backlog.md import writes archived source milestones there (ADR-0011). No milestone command archives, restores, migrates, or relocates a record into or out of that folder, and no milestone command creates it; the import is the only writer that creates archived milestone records there. A milestone file under that folder remains readable legacy data: this ADR introduces no migration, relocation, or deletion of it, and no command archives or restores a milestone.

   *Revision (2026-11-15, owner decision): the Milestone row previously listed `<backlog_directory>/archive/milestones/` as its archive location. Milestones never move — `papyrus milestone close <id>` writes `status` and `updated_date` in place, and the milestone archive and restore verbs were removed (ADR-0022 decision 5). The row and the paragraph above now record the delivered contract, including the legacy read/import exception for `<backlog_directory>/archive/milestones/`.*

3. **`archived/` is the only inactive location for document records.**
   - A `deprecated/` folder is **not** read, listed, searched, indexed by the browser, migrated, moved, or renamed. Its files stay exactly where they are until a human acts outside Papyrus. No code path may resolve it, and no descriptor may name it.
   - No Papyrus command returns an archived document to `active`; there is no document restore. Only tasks keep archive/restore, because REQ-01.005 requires task IDs to survive that cycle. Milestones have no archive or restore: their lifecycle is fixed `open -> closed` and `papyrus milestone close <id>` closes them in place (REQ-03.001, ADR-0022 decision 5).
   - Archive and delete follow ADR-0022: archive patches `status` to `archived` and writes the destination before removing the source; delete archives an active record first and then removes the file; decisions have no archive or restore operation (REQ-04.002).

   *Revision (2026-11-15, owner decision): this decision previously said that tasks and milestones keep archive/restore. Only tasks do, because REQ-01.005 requires task IDs to survive that cycle; milestones are closed in place and have no archive or restore verb (ADR-0022 decision 5). ADR-0017 decision 2 records the milestone verb set.*

4. **ID allocation, file names, resolution, relocation, and import are unchanged from ADR-0010 except for the allocation scope.**
   - One number space per family (tasks, milestones, releases, each decision prefix, each document prefix). Allocation is `max(number present in the **active and archived** locations of that family) + 1`, minimum 1. **`deprecated/` locations are excluded** because they are unmanaged and unread; a number already used by a file that a human left in `deprecated/` can therefore be allocated again, which is accepted as the consequence of not reading that folder.
   - File names: `<ID>.md`, with `<ID>` carrying the configured prefix and padding as a minimum; decisions keep `<ID>-<kebab-case-slug>.md` and the slug is cosmetic and never regenerated; title edits never rename a file. Slug-named document records keep their names (for example `BR-016-<slug>.md`).
   - Resolution is read-tolerant and never used to write: a stem `<PREFIX>-<digits>` with optional `-<slug>`/` - <slug>`, prefix compared case-insensitively, leading zeros optional; front-matter `id` is authoritative and a disagreement with the stem marks the record invalid and refuses writes. Backlog.md storage stems are not recognized (ADR-0011 is the only bridge). Legacy archive aliases `<backlog_directory>/tasks/archived/` and `<backlog_directory>/milestones/archived/` remain readable as archived tasks and milestones and are never written by a command. For milestones, `<backlog_directory>/archive/milestones/` is additionally a read/import location: the resolver reads it and the one-way import writes archived source milestones there, but no milestone command archives, restores, migrates, or relocates a record into or out of it and no milestone command creates it (decision 2).
   - A configured folder change during `config set` moves every file and subfolder through the single relocation routine with pre-check, rollback, and containment (REQ-01.002, ADR-0013); it is never lazy.
   - `papyrus.config.yml` is the only source of prefixes, padding, roots, directories, and status vocabularies; `backlog/config.yml` is not read. A status value outside the configured vocabulary is preserved verbatim and reported, never normalized.
   - Import writes native records only; a project still in the Backlog.md layout must import before the layout switchover, and the release that ships the layout ships the import and its upgrade note.

   *Revision (2026-11-15, owner decision): the legacy-alias sentence previously said that the milestone legacy alias and the milestone archive folder are never written. The alias is read-only, but `<backlog_directory>/archive/milestones/` is also a read/import location that the one-way import writes to; the sentence now states that exception and repeats that no milestone command archives, restores, migrates, or relocates a record into or out of it (decision 2).*

5. **Field map.** Every managed Markdown record carries the general keys in canonical order `id`, `title`, `created_date`, `updated_date`, `labels`, `references`, followed by its type-specific keys.

   | Record type | Additional front-matter keys |
   |---|---|
   | Task | `status`, `assignee` (YAML list), `milestone`, `dependencies`, `ordinal`, `documentation`, `type` — unchanged |
   | Milestone | `release`, `status` (`open` or `closed`) |
   | Release | `status` (`planned` or `released`) |
   | Decision | `superseded_by` (optional); `status` and `type` stay in the document core, not front matter |
   | Audit record | `type`, `status` (`active`, `archived`, or a custom status declared for the audit type) |
   | Business requirement, non-functional requirement, glossary term, UI specification, technical specification, business rule, use case, test case, test report | `status` (`active`, `archived`, or a custom status declared for that document type) |
   | Document type added by configuration | `status` only |
   | UI mockup | none — HTML record, no front-matter block; state in HTML metadata |

   Document `status` has two layers: the **main status** `active` or `archived`, which alone determines storage location, and the **custom statuses** declared in `document_types.<slug>.statuses`, which are front-matter values only and never move a file. `active` and `archived` are always declared and cannot be removed, renamed, or replaced; a new record starts at the type's `default_status` or at `active`. UI mockup state is `pending -> approved -> archived` in HTML metadata and declares no custom statuses.

   Additional project keys are tolerated and preserved (for example `priority` on tasks). There is no Backlog.md key-set contract.

6. **`status` is exempt from the create/update-option rule.** ADR-0012 decision 10's rule stands: `id`, `created_date`, and `updated_date` are exempt. For front-matter documents, `status` is additionally exempt because it is lifecycle- and configuration-owned: no create or edit option sets it, and it changes only through `papyrus <slug> status` and `archive` (ADR-0022). Task `status` keeps its documented option and configured vocabulary; milestone, release, and decision `status` keep the behaviour of their own requirements and commands (close operations, decision edit), and none of them is affected by the document exemption. A key a command never changes is still preserved by the patch contract.

7. **Timestamps and adoption are restated with the current write-path list.** `created_date` is written once at creation and never changed; `updated_date` is refreshed by every write path — create, edit, `status`, archive, delete, supersede, milestone close, release close, and browser save — and not refreshed by a no-op write. Timestamps are quoted ISO-8601 UTC at second precision (`'YYYY-MM-DDTHH:MM:SSZ'`), one clock value per write, in-project formatter, no new dependency. Adoption of a record without a front-matter block remains lazy and per record on first read and first write, adds the block only, may retire legacy metadata sections on a write, is idempotent, is scoped to descriptor-claimed files, is metadata-only for frozen records, derives missing dates best-effort, must not partially write, and offers a dry run and a per-file inventory. No path converts a record to a Backlog.md shape.

8. **Unchanged metadata rules carried forward from ADR-0012.** `id` and every reference value are Papyrus IDs (no storage ID, no translation); front matter is the only metadata store and is never mirrored into a `## ` section; `record::frontmatter` owns parsing, patching, and serialization with unknown keys and order preserved; serialization is Papyrus-documented and Backlog-independent using `yaml_serde`; the heading policy is unchanged (tasks and milestones have no `#` heading, every other Markdown record type has a generated `# <ID> <title>`, frozen ADR bodies gain only the front-matter block); decisions keep `## Date` and `## References` additively and never move `status`/`type` into front matter; UI mockups are excluded from front matter; backfilling front matter on frozen records is permitted as a metadata-only change; Markdown that no descriptor claims is out of scope.

9. **What this ADR replaces, relative to ADR-0010 and ADR-0012.**
   - Replaced from ADR-0010: decision 2's table (the `deprecated/` idle column, the "Inactive location" for decisions, and the `deploy`-as-default example are replaced by decision 2 above), decision 5's allocation scope (deprecated locations excluded), and decision 7 (deprecate/restore for documents replaced by archive-only; archive/restore restated unchanged for tasks, and milestones restated as close-only with no archive or restore).
   - Replaced from ADR-0012: decision 2's field map (superseded by decision 5 above, which adds `status` and the document rows) and decision 8's write-path list and decision 10's exemption list (both restated in decisions 7 and 6).
   - Remaining in force from ADR-0010 through this restatement: one identity per record and file (decision 1); configuration-driven paths and containment (decision 2's principle); file naming and non-renaming (decision 3); read-tolerant resolution and authoritative front-matter `id` (decision 4); numbering immutability apart from the allocation scope (decision 5); no shape conversion (decision 6); atomic, non-destructive moves (decision 7's mechanism); whole-folder relocation during `config set` (decision 8); configuration as the only source of paths and statuses (decision 9); import writes native records only (decision 10); the switchover rule requiring import in the same release and an upgrade note (decision 11).
   - Remaining in force from ADR-0012 through this restatement: decisions 1, 3, 4, 5, 6, 7, 9, 11, 12, 13, 14, 15 and the corrected decisions 8 and 10 above.
   - ADR-0022's lifecycle-operation decisions are unaffected and remain the reference for archive, `status`, delete, and the no-restore rule.

## Consequences

**Positive:**

- One document states the layout and the field map, so an implementer cannot follow a frozen ADR into building `deprecated/`, a document restore, or a status-less field map.
- The corpus no longer contains two Accepted ADRs that contradict REQ-05.007, REQ-05.010, and REQ-07.001.
- Allocation's exclusion of `deprecated/` is explicit, which makes the upgrade behaviour (a legacy folder is inert) predictable rather than accidental.
- The in-force/replaced split is recorded, so no valid decision is lost by superseding the two ADRs.

**Negative:**

- A historical reader must follow two links (ADR-0010/0012 → this ADR) to see the current rule; the superseded records keep their original text by design.
- Restating layout and metadata in a third document adds a maintenance point: a future layout change must amend this ADR, not the two superseded ones.
- Excluding `deprecated/` from allocation means a number held by an orphan file can be reused; a record moved back by hand could then collide, which is detected at read time but not prevented.
- The consolidated field map duplicates requirement-derived knowledge (keys and status vocabularies); a requirement change must be reflected here.

**Risks and mitigations:**

- *Divergence between this ADR and ADR-0022*: this ADR states layout and metadata only; every lifecycle operation is delegated to ADR-0022, and a test/assertion of the field map derives from the registry (ADR-0017), not from prose.
- *A reader treating ADR-0010/0012 as current*: both status lines now point to this ADR, and the index shows them as Superseded.
- *`deprecated/` reappearing in code*: the no-drift check in ADR-0013 is extended to fail on any descriptor, resolver, route, or documentation reference to `deprecated/`.
- *Field map drift*: the registry's key table is asserted in one test against this ADR's table; adding a key requires updating both.

## Alternatives Considered

| Option | Outcome | Reason |
|---|---|---|
| Consolidate into a new ADR that supersedes ADR-0010 and ADR-0012 (this ADR) | Selected | One authoritative statement, no clause-level ambiguity, and the frozen bodies stay as history. |
| Amend ADR-0010 and ADR-0012 in place | Rejected | Their bodies are frozen once Accepted; editing them would break the ADR lifecycle rule. |
| Leave ADR-0010/0012 Accepted and rely on ADR-0022 as the delta | Rejected | Two accepted ADRs would keep mandating `deprecated/`, restore, and a status-less field map; an implementer could follow either. |
| Write two separate consolidation ADRs (layout, metadata) | Rejected | The two contracts are read together and share the same requirement changes; splitting them doubles the cross-reference surface. |
| Delete ADR-0010/0012 | Rejected | The corpus keeps superseded decisions as history; deleting loses the rationale for path/identity and patch/adoption choices. |

## Open Questions

- Should a `deprecated/` folder be reported by a read-only inventory command so a user can find orphaned files after the upgrade? Current requirement: no; the folder is inert.
- Should ADR-0022's "consolidation ADR outstanding" sentence be annotated now that this ADR exists? Its body is frozen; the index shows the resolution and no edit is made.

## Downstream constraints

1. `deprecated/` must not appear in any descriptor, resolver, browser route, search path, documentation, or test fixture after this ADR, except in the historical ADR bodies and in the upgrade note that tells users the folder is inert.
2. Document restore must not exist in any surface; only tasks keep their archive/restore pair, and milestones are closed in place with no archive or restore.
3. Allocation scans active and archived locations only, and excludes `deprecated/`.
4. The field map in decision 5 is authoritative; adding or removing a key requires a registry change (ADR-0012 decision 6 as carried forward) and an update to this ADR.
5. `status` is never a create or edit option for a front-matter document; only `status` and `archive` write it.
6. ADR-0022 remains the reference for lifecycle operations and must not be contradicted by an implementation of this ADR.

## References

- `docs/requirements/business/REQ-05.007.md` — archive state, `archived/`, delete, no restore, `deprecated/` inert.
- `docs/requirements/business/REQ-05.010.md`, `REQ-05.011.md` — field map, `status`, option exemption, adoption.
- `docs/requirements/business/REQ-05.008.md` — UI mockup HTML metadata.
- `docs/requirements/business/REQ-07.001.md`, `REQ-07.002.md` — main/custom document statuses and their configuration.
- `docs/requirements/business/REQ-01.002.md`, `REQ-01.005.md` — folder relocation, IDs.
- `docs/requirements/business/REQ-04.002.md` — decisions have no archive state.
- `docs/architecture/adr/ADR-0006-optimistic-concurrency-and-atomic-record-writes.md` — atomic write.
- `docs/architecture/adr/ADR-0010-papyrus-native-on-disk-record-layout.md` — superseded by this ADR; layout, identity, formatting rationale retained as history.
- `docs/architecture/adr/ADR-0011-one-way-backlog-md-import.md` — the only bridge from Backlog.md.
- `docs/architecture/adr/ADR-0012-papyrus-owned-front-matter-metadata.md` — superseded by this ADR; metadata rationale retained as history.
- `docs/architecture/adr/ADR-0013-shared-record-envelope-without-a-storage-layer.md` — envelope, patch, single writer.
- `docs/architecture/adr/ADR-0017-artifact-category-registry-and-type-descriptors.md` — status models and descriptors.
- `docs/architecture/adr/ADR-0022-document-lifecycle-state-and-archive-location.md` — lifecycle operations, in force.
- Implementation: `src/record/descriptor.rs`, `src/record/store.rs`, `src/record/frontmatter.rs`, `src/record/identity.rs`, `src/record/migrate.rs`, `src/registry.rs`.
