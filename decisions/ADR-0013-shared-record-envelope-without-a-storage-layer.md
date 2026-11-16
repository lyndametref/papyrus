---
id: ADR-0013
title: "Shared Record Envelope and Single Write Path Without a Storage Layer"
created_date: '2026-11-11T00:00:00Z'
updated_date: '2026-11-16T21:08:36Z'
labels: []
references: []
---
# Shared Record Envelope and Single Write Path Without a Storage Layer

## Status
accepted
## Date

2026-11-11 (status: Accepted, 2026-11-11)

## Context

ADR-0012 makes Papyrus-owned YAML front matter the single metadata source and requires patch-based, field-preserving writes and lazy front-matter adoption. ADR-0010 makes the on-disk layout Papyrus-native with one ID space. ADR-0011 adds a one-way import that must produce ordinary Papyrus records. ADR-0006 fixes optimistic concurrency and atomic writes for browser edits.

ADR-0009 (Accepted) established the right structure — one shared record module, one patch API, one writer for CLI and browser, one metadata projection, and a staged delivery — but it is entangled with the storage-identity layer that NFR-04.004 and REQ-01.005 remove: `record::identity` is defined as a two-way display/storage translator, descriptors declare a storage ID pattern and directory, the store translates IDs on the way in and out, Stage 2 exists only to build that translation, and Stage 4 exists only to move records into the Backlog.md layout. Leaving ADR-0009 Accepted would keep a contradicting instruction in force, and its body is frozen, so this ADR supersedes it and restates the envelope with the storage layer removed.

The starting codebase already contains the shared module (`src/record/{frontmatter,document,patch,store,identity,descriptor,migrate,view}.rs`) plus fourteen legacy per-type modules and two browser write paths. The no-drift rule of ADR-0009 decision 7 is only partly enforced: `src/task.rs` and `src/milestone.rs` still resolve three directory locations each and still allocate numbers through the storage-offset identity code, and `src/record/migrate.rs` still contains the legacy-to-Backlog body conversion and relocation.

Forces: one writer so preservation is testable once; CLI/browser parity; single ID space; atomic writes and version tokens; staged delivery on a live corpus; no new dependencies (ADR-0005); and no second parser for the import.

## Decision

1. **One shared record module inside the single crate** (ADR-0004, ADR-0005), `src/record/`:
   - `record::frontmatter` — detect, parse, patch, and serialize the leading YAML block; owns the key registry (general keys plus per-type keys) from ADR-0012.
   - `record::document` — lossless envelope: front matter plus ordered body units (optional heading line, preamble lines, `## ` sections with raw bodies). Section text and unknown content stay as raw lines, never re-rendered.
   - `record::patch` — the only mutation API: `SetFrontMatter`, `ClearFrontMatter`, `SetSection`, `ClearSection`, `SetHeadingTitle`, plus explicit section-move operations for adoption. Applying an empty patch, or a patch whose values equal the current ones, returns byte-identical text.
   - `record::identity` — **Papyrus identity only**: parse and format an ID under a configured prefix and `id_padding`, validate a reference value, and allocate the next number of a family from the records present in every active, archived and deprecated location (ADR-0010 decision 5). It has no storage prefix, no offset, and no translation function.
   - `record::store` — record location from configuration, safe path resolution (`Context::ensure_within`), read, atomic write (`create_new` + `sync_all` + same-directory rename + cleanup from ADR-0006), version token, archive/restore and deprecate/restore moves.
   - `record::relocate` — the single implementation of a configured folder move (ADR-0010 decision 8): plan, pre-check, move, rollback, used by `config set` and therefore by the browser configuration view.
   - `record::descriptor` — the record-type registry.
   - `src/import/` — the Backlog.md source adapter (enumerate, map, plan, report). It may read and interpret Backlog.md files, but it must express every write as a `record::store` call and every target file as a `record::document`/`record::patch` result. It may not format front matter or write files itself.

2. **Every record module becomes a descriptor, not a serializer.** A descriptor declares: its configured ID prefix key, its configured directory key, its native file-name form, its extra front-matter keys, its known body sections and their order, and its validation rules. No module keeps a `render_*` function that assembles a whole file. The only exceptions are UI mockups (HTML, outside front-matter scope per ADR-0012) and the import's body-section mapping, which must be expressed as patch/section operations.

3. **The browser uses the same reader, patcher and identity.** `browser::records::parse_doc` and the `apply_*_edit` functions are replaced by `record::document` + `record::patch` + `record::identity`. Metadata edits (title, status, milestone, assignee, labels, dependencies, references, release, audit type, `superseded_by`) become front-matter patches; free-text edits remain section patches. Title edits patch front matter and, where the type keeps one, the heading line, in one write. Browser routes, form values and links carry the Papyrus ID; no other ID form is accepted.

4. **One write path for both surfaces.** All CLI writes use `record::store::write_atomic`; no module calls `fs::write` on a record file. The browser keeps the ADR-0006 contract: a stale or malformed version token is `409`/`400` before any write. The CLI does not gain version tokens but does gain atomicity and section preservation, discharging the ADR-0006 deferred follow-up without changing ADR-0006. A write that changes no byte does not happen, so `updated_date` is not refreshed by a no-op.

5. **One projection for both views, with one identity.** A single `RecordView` exposes the Papyrus ID, the ordered `metadata` map as stored (whose `id` is the same Papyrus ID), and the typed document fields. CLI `list`/`view` text and `--json`, browser list/detail JSON, and browser HTML render from it. There is no storage-ID field and no second identity anywhere; any existing `--json` field that exposed a storage ID is removed as a documented breaking change, and `metadata.id` carries the Papyrus ID.

6. **The front-matter key registry is the only metadata extension point.** Adding a key to a record type is a registry entry plus a section-retirement rule, not a new serializer.

7. **Strict no-drift rule.** From this ADR onward no record parsing, metadata formatting, ID formatting, path resolution, or record writing may be added outside `src/record/`. Existing module-local code is removed as each type migrates. A grep-based check is part of review: no `fs::write` on a record path, no `format!("---` outside `src/record/`, no ID formatting outside `record::identity`, and no remaining `to_storage_id`/`to_display_id`/offset/storage-prefix symbols.

8. **Delivery is staged; the corpus stays readable at every stage.** Each stage is independently mergeable and testable.
   - **Stage 1 — envelope.** `record::frontmatter` + `document` + `patch` + `store` read both legacy (no front matter) and new shapes. Largely present.
   - **Stage 2 — native identity and location.** Replace the translation identity with Papyrus-only identity and allocation; descriptors resolve directories from configuration; tolerant stem resolution (`<PREFIX>-<digits>[-slug]`); read aliases for `tasks/archived/` and `milestones/archived/`; delete storage prefixes, offsets, `to_storage_id`, `to_display_id`, the `backlog/config.yml` reader, and slug-based identity. Remove the milestone `+1` offset and the `m-0` base from allocation and tests.
   - **Stage 3 — document front-matter adoption.** Lazy per-record adoption for document record types on first read and first write, built only on `record::patch`, with dry run and per-file inventory (ADR-0012 decision 9). Requires Stage 1 only.
   - **Stage 4 — task and milestone adoption.** Front-matter-only adoption for tasks and milestones, including retirement of legacy `## Status`-style sections on write. The legacy-to-Backlog body conversion, the Backlog-layout relocation, and the "byte-identical Backlog body" guarantee are deleted with their tests, because the target shape no longer exists (NFR-04.004). Requires Stages 1 and 2.
   - **Stage 5 — relocation.** `record::relocate` and its integration into `config set` and the browser configuration view (REQ-01.002, REQ-06.008): whole-folder move, pre-flight conflict detection, rollback, containment.
   - **Stage 6 — import.** `papyrus import backlog` (ADR-0011) on top of Stages 1, 2, 4 and 5.
   - **Stage 7 — removal and enforcement.** Delete the legacy per-type parsers/writers and the browser write path; enforce the decision 7 checks.
   - **Stage 8 — verification.** Adoption idempotency and byte identity; no Backlog-shaped write anywhere; single-ID surfaces in CLI text, `--json`, browser JSON, browser HTML and URLs; import fidelity and excluded scope; relocation atomicity and conflict behaviour; search still indexing whole persisted text (now including front matter) as documented.
   A stage may not be reordered past its prerequisites; Stages 3 and 4 may run in parallel; Stage 6 must not ship before Stage 2 (otherwise imported records would carry translated identities), and **Stage 2 must not ship without Stage 6 in the same release** whenever an upgrade path from a Backlog.md-layout store exists, because Stage 2 makes those records invisible (ADR-0010 decision 11).

9. **One clock per write.** Every write path obtains one UTC timestamp and uses it for `updated_date` (and `created_date` at creation); no module calls the clock separately for the same write.

10. **Deletions are part of the decision, not cleanup.** The following are removed rather than left dormant: the storage-identity translation and its constants, the `backlog/config.yml` layering, the legacy-to-Backlog body conversion and relocation, the Backlog.md round-trip verification suites, storage-ID diagnostics and JSON fields, and any documentation that declares Backlog.md compatibility.

## Consequences

**Positive:**

- "Preserve everything the operation does not change" is one tested property instead of fourteen untested behaviours, shared by CLI, browser, adoption and import.
- CLI and browser cannot disagree about metadata or identity because there is one reader, one identity function and one projection.
- The concrete removal list makes the renunciation enforceable: any surviving storage-ID code path fails review mechanically.
- The relocation routine is written once and serves both REQ-01.002 and REQ-06.008.
- Imported records are ordinary records from the moment they are written, because the import cannot bypass the writer.

**Negative:**

- This remains the largest refactor in the change: every record type's read and write path is touched, and the work is spread over several tickets.
- Stage ordering constrains planning: no user-visible import before Stage 2, no import at all before Stage 6.
- The shared envelope must model each type's structure (heading or not, section order, markers) without becoming a Markdown engine.
- Removing `--json` storage-ID fields is a breaking API change for any external consumer.
- Search behaviour changes again: front matter is indexed, so metadata values become searchable.

**Risks and mitigations:**

- *Refactor regression*: migrate one record type at a time with its existing tests unchanged plus new patch tests (no-op byte identity, unknown-key preservation, unknown-section preservation).
- *Two writers during the transition*: the decision 7 checks are mechanical; the import is explicitly bound to `record::store`.
- *Stage 2 incomplete while Stage 4 runs*: Stage 4 must not start before Stage 2 lands, or task front matter could still carry a translated identity.
- *Identity bypass in a new surface*: `record::identity` is the only ID code; tests assert CLI/browser output and browser URLs never contain a non-Papyrus ID.
- *Relocation data loss*: pre-flight plan, rollback, and tests for destination conflict, permission failure, and cross-device fallback.
- *Adoption on the read path*: read-only media and concurrency implications are unchanged from ADR-0007/ADR-0012 and must be tested (error reported, file left byte-identical, concurrent second reader a no-op).
- *Search regression*: indexing is whole-file text; the specification must state that metadata values are searchable.

## Downstream constraints

1. No new module may parse, format, locate, or write a record outside `src/record/`; `src/import/` may only read Backlog.md sources and build a plan.
2. `record::identity` exposes Papyrus ID parse/format/validate/allocate only. Any patch that adds a translation function is rejected.
3. Descriptors name configuration keys, not paths; the configured roots are the only source of locations.
4. Descriptor and configuration must stay in agreement: an enabled type without a prefix or directory is a load-time validation error (this replaces the ADR-0008 layered-config validation).
5. `record::store` remains the only record writer, and the ADR-0006 version-token contract is unchanged for browser writes.
6. `record::relocate` is the only folder-move implementation and is invoked by `config set`; the browser may not move folders.
7. Existing `--json` and browser JSON consumers must be re-tested after the storage-ID fields are removed; specification updates for `papyrus(1)` and `docs/specifications/browser-*.md` are owned by `business-analyst`/`tech-writer`.
8. Stage order is a planning constraint: Stage 2 before Stages 4 and 6; Stage 1 before everything; verification (Stage 8) is a gate on the release that claims NFR-04.004.

## Alternatives Considered

| Option | Outcome | Reason |
|---|---|---|
| Shared envelope, single writer, Papyrus-only identity, staged removal (this ADR) | Selected | Keeps the structure that made preservation testable and drops only the storage layer that the requirements remove. |
| Keep ADR-0009 in force and treat the translation as dead code | Rejected | The ADR would still mandate translation, Stage 2 and Stage 4 in the Backlog.md layout; accepted decisions must not contradict requirements. |
| Add front-matter and identity code to each of the fourteen modules | Rejected | Guarantees drift across CLI, browser, adoption and import. |
| Keep the browser serializer and give the CLI a separate one | Rejected | Two writers already disagree; REQ-06.004 requires identical values and identical preservation. |
| A standalone import script outside the crate | Rejected | A second writer with untested preservation; the wrapper-script form is allowed only as a thin delegate (ADR-0011). |
| A Markdown/AST library | Rejected | New dependency, and re-rendering would not preserve unknown content byte-identically (ADR-0005). |
| Store records in a database or index cache | Rejected | Violates NFR-04.001 text-file persistence and would be a parallel store. |
| Delete only the two conflicting ADRs' decisions and leave the code | Rejected | Leaves accepted decisions unenforceable and the code contradicting the requirements. |

## References

- `docs/architecture/adr/ADR-0009-shared-record-envelope-and-unified-write-path.md` — superseded by this ADR.
- `docs/architecture/adr/ADR-0010-papyrus-native-on-disk-record-layout.md` — layout, identity, allocation, relocation.
- `docs/architecture/adr/ADR-0011-one-way-backlog-md-import.md` — import contract built on this writer.
- `docs/architecture/adr/ADR-0012-papyrus-owned-front-matter-metadata.md` — metadata contract.
- `docs/architecture/adr/ADR-0006-optimistic-concurrency-and-atomic-record-writes.md` — atomic writes and version tokens.
- `docs/architecture/adr/ADR-0005-embedded-dependency-free-http-server-and-server-rendered-ui.md` — module boundaries and no new dependencies.
- `docs/requirements/business/REQ-05.011.md`, `REQ-06.004.md`, `REQ-01.005.md`, `REQ-01.002.md`, `REQ-06.008.md`.
- `docs/requirements/non-functional/NFR-04.004.md`, `NFR-04.001.md`, `NFR-04.002.md`.
- Implementation: `src/record/*`, `src/task.rs`, `src/milestone.rs`, `src/browser/records.rs`, `src/commands.rs`, `src/init.rs`.
