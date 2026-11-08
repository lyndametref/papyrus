# Shared Record Envelope and Unified Write Path for CLI and Browser

## Status

Accepted — approved by explicit user confirmation on 2026-11-08

## Date

2026-11-08 (status: Accepted, 2026-11-08)

## Context

ADR-0007 makes YAML front matter the single metadata store, requires field-preserving patches, and requires lazy per-record migration on first read and first write that adds front matter to each managed Markdown record and converts legacy task/milestone bodies, with idempotent no-ops thereafter. ADR-0008 makes task and milestone storage Backlog.md-shaped and introduces a translation layer between Papyrus display IDs and Backlog.md storage IDs. Both requirements land on a codebase with duplicated record handling:

- Fourteen modules each implement their own heading parser, `## ` section parser, renderer, and writer: `requirement`, `task`, `milestone`, `release`, `decision`, `glossary`, `rule`, `ui_spec`, `ui_mockup`, `audit`, `tech_spec`, plus the browser's `records` and `render`, and `search` reads the same files again.
- Two independent parsers already disagree in risk profile: `browser::records::parse_doc` + `apply_*_edit` (section-preserving, version-checked, atomic) and the CLI modules' `parse_*` + `render_*` + `fs::write` (full re-render, non-atomic, no version token).
- ADR-0006 (Accepted) fixed optimistic concurrency and atomic writes for browser edits and explicitly deferred the CLI: "Reconciling the CLI with this contract (atomic writes, optimistic concurrency, section preservation) is deferred to a follow-up task and is not part of this ADR."
- REQ-05.011 requires every writing command, CLI and browser, to preserve untouched fields and requires lazy per-record migration on first read and first write; REQ-06.004 requires the browser to write metadata back into front matter and preserve the rest of the file. Re-rendering from a parsed model cannot satisfy either: the browser's `apply_*_edit` functions rebuild the file from `# heading` + preamble + sections, which would discard a front-matter block entirely.
- REQ-01.005 requires the display/storage ID translation to happen at the storage boundary and to be consistent in both surfaces.
- ADR-0005 fixes the module boundaries inside the single crate and forbids new dependencies; any restructuring must stay inside that baseline.

The alternative to a shared implementation is to add front-matter handling to each of the fourteen modules and to the browser, plus one migration script per family. That guarantees the drift ADR-0006 already lists as a risk and makes "CLI and browser show the same values" and "migration changes nothing but the block" impossible to guarantee.

## Decision

1. **Introduce one shared record module inside the single crate** (ADR-0004, ADR-0005), planned as `src/record/`:
   - `record::frontmatter` — detect, parse, patch, and serialize the leading YAML block; owns the key registry (general keys plus per-type keys) from ADR-0007.
   - `record::document` — the lossless envelope: front matter plus ordered body units (optional heading line, preamble lines, `## ` sections with their raw bodies). Section text and unknown content are kept as raw lines, never re-rendered.
   - `record::patch` — the only mutation API: a list of `PatchOp` values (`SetFrontMatter(key, value)`, `ClearFrontMatter(key)`, `SetSection(heading, value)`, `ClearSection(heading)`, `SetHeadingTitle(title)`). Applying a patch returns the new text; applying an empty patch or a patch whose values equal the current values returns byte-identical text.
   - `record::identity` — the only ID code: parse, normalize, and translate between the Papyrus display ID and the Backlog.md storage ID (ADR-0008 decision 4), validate reference values, and allocate the next storage number from the records present in both active and archived locations.
   - `record::store` — record location, safe path resolution (`Context::ensure_within`), read, atomic write (the `create_new` + `sync_all` + same-directory rename + cleanup sequence from ADR-0006), the version token, and translation of incoming display IDs to storage paths and of stored storage IDs to display IDs on the way out.
2. **Every record module becomes a descriptor, not a serializer.** A record type declares: its display ID pattern and directory, its storage ID pattern and directory (where they differ), its extra front-matter keys, its known body sections and their order, and its field validation rules. Creation and editing build patch operations through the shared module. No module keeps a `render_*` function that assembles a whole file, except UI mockups (HTML, out of front-matter scope per ADR-0007) and the lazy per-record migration, which must itself be expressed as patches through `record::patch`.
3. **The browser uses the same reader, patcher, and identity.** `browser::records::parse_doc` and the ten `apply_*_edit` functions are replaced by `record::document` + `record::patch` + `record::identity`. Metadata edits (title, status, milestone, assignee, labels, dependencies, release, audit type, `superseded_by`) become front-matter patch operations; free-text edits remain section patches. Title edits patch the front matter and, where the record type keeps one, the heading line, in the same write. Browser routes, form values, and links carry the display ID; the store translates it before touching the filesystem, and a storage ID in a URL is rejected like any unknown ID.
4. **One write path for both surfaces.** All CLI writes use `record::store::write_atomic`; the CLI no longer calls `fs::write` on a record file. The browser keeps the ADR-0006 optimistic concurrency contract: a stale or malformed version token is `409`/`400` before any write. The CLI does not gain version tokens (there is no client to return a conflict to), but it does gain atomicity and section preservation, which discharges the ADR-0006 deferred follow-up without changing ADR-0006's decision. A write that changes no byte does not happen at all, so `updated_date` is not refreshed by a no-op (ADR-0007 decision 7).
5. **One projection for both views, with an explicit identity field.** A single `RecordView` type exposes the record identity (the display ID), the raw metadata (`metadata` as an ordered map of the front-matter keys as stored, where `id` is the storage value), and the typed document fields. CLI `list`/`view` text and `--json`, browser list/detail JSON, and browser HTML all render from it, so REQ-06.004's "same metadata values" is structural rather than maintained by hand. The identity field is the display ID and is the only value presented as the record's ID; the storage ID appears only inside `metadata` and in labelled diagnostics. `--json` gains a `metadata` object; the browser JSON record and form gain the same object plus `created_date`/`updated_date`. The `metadata` object is a stable public contract (resolved 2026-11-08): documented in `papyrus(1)` and the browser specifications; changes are additive only.
6. **The front-matter key registry is the only metadata extension point.** Adding a key to a record type (for example decision `status` if the product decision later moves it, or a release state) is a registry entry plus a section-retirement rule; it must not require a new serializer.
7. **Strict no-drift rule.** From the implementation of this ADR onward, no new record parsing, metadata formatting, ID translation, or record writing may be added outside `src/record/`. Existing module-local code is removed as each type migrates. Temporary dual implementations during the migration must be marked and tracked, and the migration is not complete until `grep` finds no `fs::write` on record paths, no `format!("---` outside the shared module, and no ID-formatting outside `record::identity`.
8. **Delivery is staged so the corpus stays readable and the migration has exactly one implementation.**
   - Stage 1 — `record::frontmatter` + `record::document` + `record::patch` + `record::store`, able to read both the legacy (no front matter) and new shapes, with the descriptor set for the record types.
   - Stage 2 — `record::identity` for task and milestone IDs, including the storage-number allocator and conflict detection (ADR-0008 decision 9).
   - Stage 3 — lazy migration of document record types (requirements, NFRs, specifications, glossary, rules, decisions, audits, releases) on first read and first write, built only on `record::patch`, with a dry run and a per-file inventory available as a reporting and verification aid (ADR-0007 decision 9). Stages 1 and 2 are prerequisites; Stage 2 is not needed for document types but the shared reader/patcher is.
   - Stage 4 — lazy migration of tasks and milestones to the Backlog.md layout and storage IDs on first read and first write, which must follow Stage 2 because their front matter carries storage IDs and reference values (ADR-0008 decision 8). This is the only stage permitted to change a task or milestone body, and only for records still in the legacy Papyrus shape (ADR-0008 decision 8); records already in Backlog.md shape stay byte-identical.
   - Stage 5 — removal of the legacy per-module parsers/writers and enforcement of the no-drift checks in decision 7.
   - Stage 6 (verification) — a dedicated verification task for lazy migration correctness and idempotency: first read and first write of a managed record adds front matter; a second read and second write are byte-identical no-ops; legacy Papyrus-shape task/milestone files convert on first access with unknown sections and free text preserved verbatim and a before/after diff reported; Backlog.md-shape files keep a byte-identical body apart from the front-matter block; migration covers only descriptor-claimed files; unread records may remain unmigrated; read-path write failures (read-only media, missing permission) leave the original file byte-identical and are reported.
   The migration cannot run before Stages 1 and 2. A migration implemented independently of `record::patch` is prohibited: it would be a second writer whose byte-identity guarantees nobody tests.
9. **One clock per write.** Every write path obtains a single UTC timestamp value and uses it for `updated_date` (and `created_date` at creation); no module calls the clock separately for the same write.

## Consequences

**Positive:**

- "Preserve everything the operation does not change" becomes one tested property instead of fourteen untested behaviours.
- CLI and browser cannot disagree about a record's metadata or its ID because there is one reader, one identity function, and one projection.
- REQ-05.011's per-field options become a loop over the key registry instead of fourteen hand-written CLI surfaces.
- The ADR-0006 deferred gap (CLI non-atomic writes, full re-render, lost unknown sections) is closed as a side effect of the requirement, not as separate scope.
- Backlog.md round-trip tests can target one patcher, and the migration reuses it, so migration fidelity is the same property as edit fidelity.
- The ID translation is testable as one pure function rather than per command.

**Negative:**

- This is the largest refactor in the change: every record type's read and write path is touched, and the migration cannot be delivered in one small task.
- The staged order creates a delivery constraint: nothing user-visible can ship before Stage 1 and 2 exist, so the first merge is large and low on visible value.
- The shared envelope must model enough of each type's structure (heading or not, section order, markers) without becoming a general Markdown engine; there is a real risk of over-generalizing.
- Error messages, validation text, and JSON shapes currently produced per module must be preserved exactly; the refactor will surface hidden per-module quirks as test failures.
- Identity becomes a concept every surface must respect (display ID out, storage path in); a single bypass reintroduces the confusion the previous draft created.
- Search still reads raw file text independently and will now also match front-matter values, including storage IDs; that behaviour must be documented rather than fixed by filtering.

**Risks and mitigations:**

- *Refactor regression*: migrate one record type at a time with its existing tests unchanged, plus new patch tests (no-op byte identity, unknown-key preservation, unknown-section preservation).
- *Over-generalization*: the envelope models lines and sections only; it does not parse Markdown semantics. Type-specific structure stays in descriptors.
- *Partial migration with two writers*: the completion check in decision 7 is mechanical and can be enforced in review; the migration is required to use `record::patch` (decision 8).
- *Migration ordered before identity*: task and milestone front matter would encode display IDs, silently breaking the Backlog.md round trip; decision 8 forbids it, and the round-trip test in ADR-0008 decision 10 catches it.
- *Migration growing into a body rewriter*: only Stage 4 may change a task/milestone body, only for legacy-shaped files, and only through explicit body-mapping ops; document record migration and every normal edit stay front-matter/section patches. A test asserts that migrating an already-storage-shaped corpus is byte-identical.
- *ID translation bypassed by a new surface*: `record::identity` is the only ID code under the no-drift rule; a test asserts that CLI and browser output never contains a storage ID in the identity position.
- *JSON API change*: adding `metadata` to the browser JSON record is an additive change, but the identity field keeps its meaning (`id` = display ID); the specification pages for each browser surface must be updated by `business-analyst`/`tech-writer`, and JSON consumers in tests must be checked. If a consumer needs the storage ID, it reads `metadata.id`; no second top-level field is introduced without a decision. Ownership note (settled 2026-11-08): the specification documents to revise for the display/storage ID split (`papyrus(1)` ID sections plus `docs/specifications/browser-*.md` showing IDs) are owned by `business-analyst`/`tech-writer` after this ADR is accepted; not blocking the design.
- *Write atomicity on unusual filesystems*: unchanged from ADR-0006; same-directory rename assumption still holds.
- *Lazy migration on the read path*: a first read may trigger a write (front-matter insertion or legacy body conversion). On read-only media or without write permission the migration must fail without partial writes and must leave the original file byte-identical; the reader reports the error rather than a migrated record. Two concurrent readers of the same unmigrated record serialize through the atomic write contract; the second observes the already-migrated file and becomes a no-op. A reader that loaded pre-migration bytes is not silently upgraded. These implications are also recorded in ADR-0007.
- *Performance*: per-request re-reads stay; the shared reader should parse once per record instead of the current duplicate parsing in `browser::records`, so the change is not expected to regress. ID translation is arithmetic, not I/O.

## Resolved open questions (user decisions 2026-11-08)

1. Resolved (user decision 2026-11-08): CLI `--json` `metadata` is a stable public contract, documented in `papyrus(1)` and the browser specifications; changes are additive only. Not blocking.

## Alternatives Considered

| Option | Outcome | Reason |
|---|---|---|
| One shared record module with a patch-based writer and identity function (this ADR) | Selected | Only design that makes field preservation, CLI/browser parity, and one ID translation testable once; stays inside ADR-0004/0005 boundaries. |
| Add front-matter and ID code to each of the fourteen modules | Rejected | Guarantees drift, repeats fourteen times the logic that must match Backlog.md byte-for-byte. |
| Keep the browser serializer and give the CLI a separate one | Rejected | Two parsers already disagree; REQ-06.004 requires identical values and identical preservation. |
| Lazy per-record migration on first read and first write (this ADR, revised 2026-11-08) | Selected | Matches the revised REQ-05.011; the corpus converges record by record; readers keep the legacy fallback until convergence; idempotent. Stages 1 and 2 remain prerequisites; Stage 6 verifies correctness and idempotency. |
| One bulk migration stage instead of lazy per-record migration | Rejected by product decision 2026-11-08 | ADR-0007 decision 9 no longer requires a bulk pass; lazy migration achieves the same end state without a dedicated command and keeps readers robust to both shapes. |
| A standalone migration script separate from the shared writer | Rejected | Becomes a second writer whose body-preservation and byte-identity behaviour is untested and diverges from normal edits. |
| Adopt a Markdown/AST library | Rejected | New dependency and a rendering model that would not preserve unknown content byte-identically; ADR-0005 forbids new dependencies. |
| Move records into a database or an index cache | Rejected | Violates NFR-04.001 text-file persistence and the no-parallel-store constraint; an ID mapping table is the same mistake at smaller scale. |
| Full re-render from a typed model on every write | Rejected | Cannot preserve unknown keys, foreign front matter, or ADR-frozen bodies. |

## References

- `docs/requirements/business/REQ-05.011.md` — every writing command preserves untouched metadata; lazy per-record migration.
- `docs/requirements/business/REQ-06.004.md` — CLI and browser read front matter, browser writes it back.
- `docs/requirements/business/REQ-01.005.md` — display/storage ID translation at the storage boundary.
- `docs/requirements/non-functional/NFR-04.001.md` — text-file persistence.
- `docs/requirements/non-functional/NFR-04.002.md` — project-directory boundary.
- `docs/architecture/adr/ADR-0005-embedded-dependency-free-http-server-and-server-rendered-ui.md` — module boundaries and no-new-dependency baseline.
- `docs/architecture/adr/ADR-0006-optimistic-concurrency-and-atomic-record-writes.md` — atomic write and version token contract; deferred CLI alignment.
- `docs/architecture/adr/ADR-0007-yaml-front-matter-as-single-record-metadata-source.md` — front-matter key registry, patch requirement, lazy per-record migration.
- `docs/architecture/adr/ADR-0008-adopt-backlog-md-on-disk-layout-for-tasks-and-milestones.md` — task/milestone shape, two ID spaces, conflict handling.
- Implementation: `src/browser/records.rs` (`parse_doc`, `apply_*_edit`, `write_atomic`, `version_token`), `src/browser/mod.rs` (`form_post`, `api_post`), per-type CLI modules under `src/`, `src/context.rs` (`ensure_within`).
- Tests: `tests/browser_cli.rs`, module unit tests in `src/browser/records.rs`.
