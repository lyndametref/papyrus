# Document Lifecycle State and Archive Location

## Status

Accepted (2026-11-13) — accepted by explicit user decision of 2026-11-13; amended 2026-11-14 by owner decision (decision 3 revision note and decision 12) without status change; amended 2026-11-15 by owner decision (milestone lifecycle) without status change.

## Date

2026-11-13 (status: Proposed, 2026-11-13; status: Accepted, 2026-11-13)

## Context

REQ-05.007 (rewritten in this change set) replaces the document lifecycle `active -> deprecated` with a **main status** `active` or `archived`, stores the state in the front-matter `status` field, moves an archived file into an `archived/` subfolder of the type's configured directory, removes every restore command for documents, and states that a `deprecated/` folder left by an earlier version is not read as archived and is not migrated. REQ-07.001 adds **custom statuses**: each document type may declare additional statuses in `document_types.<slug>.statuses`, optionally with a `default_status`; a custom status is a front-matter value only — no folder per status, no file move, no effect on listing, archiving, or deletion — and `papyrus <slug> status <id> <value>` sets `active` or a declared custom status but never archives. REQ-07.001 also fixes the milestone (`open -> closed`) and release (`planned -> released`) lifecycles, and REQ-05.010 now requires `status` in milestone, release, and front-matter document records. REQ-04.002 keeps decisions out of it: decisions have no archive command or archive state and use the five Nygard statuses.

Current code and ADRs contradict the new state:

- `src/record/descriptor.rs` resolves a `deprecated` directory (`resolve_deprecated_directory`) for document types, and the browser and record layer read it.
- `src/record/store.rs` exposes deprecate/restore moves; every document module has `Deprecate`/`Restore` verbs; `src/browser` has deprecate/restore handlers.
- ADR-0010 decision 2 lists `<doc_directory>/<type directory>/deprecated/` as the idle location, and decision 7 states "archive, restore, deprecate, restore are moves, not rewrites" with restore as a first-class operation. Decision 5's allocation rule already scans the deprecated location.
- ADR-0012's field map gives milestones only `release`, releases no additional keys, and document types no keys; `status` is absent for all three.
- ADR-0012 decision 6 declares the front-matter key registry the only metadata extension point (a registry entry plus a section-retirement rule), which is the mechanism available for adding `status`.
- `nfr`/task/milestone records keep their own `status` semantics: tasks configurable (REQ-01.002), milestones `open|closed`, releases `planned|released`; only documents lose restore.

Forces: one identity and one file name across state changes (REQ-01.005); no data loss on failure; the on-disk location must remain derivable from configuration (ADR-0010 decision 2); writes must stay on the single path (`record::store`, ADR-0013 decision 4); archive must not delete; legacy `deprecated/` content must not be silently presented as archived; and the state value must be visible in the same `RecordView` projection used by CLI, JSON, and browser (ADR-0013 decision 5).

## Decision

1. **`status` is a registry key for three record families, with two different state models.** ADR-0012 decision 6 is used: the key registry gains `status` for
   - milestone: fixed states `open | closed`;
   - release: fixed states `planned | released`;
   - every front-matter-backed document type (built-in or config-defined): **main status** `active | archived`, plus the type's **custom statuses** declared in configuration.

   For a document type the registry entry is `main = [active, archived]`, `custom = <declared statuses other than the two main ones, in configured order>`, `default = <default_status or active>`. `active` and `archived` are always present and cannot be removed, renamed, or replaced; a custom value may not reuse either. A built-in document type with no configured `statuses` therefore has exactly the two main states, and a config-defined type without `statuses` behaves the same way.

   Key order is unchanged: general keys first, then `status`, then the type's other keys. UI mockups keep their status in HTML metadata, gain no front-matter key, and declare no custom statuses (REQ-05.008). Decisions keep `status` in the document core (ADR-0012 decision 12).

2. **Archive is one operation on one record: patch `status`, then relocate.** `record::store::archive(id, descriptor)`:
   1. builds the patched text with `record::patch` (`SetFrontMatter("status", "archived")`, plus the UI-mockup HTML metadata write and `superseded_by` back-reference where the descriptor requires it);
   2. writes that text to the destination path `<active dir>/archived/<file name>` using the ADR-0006 atomic write sequence (temporary file, `sync_all`, rename);
   3. removes the source file only after the destination write succeeded.

   If step 2 fails the source is untouched and the command fails. If step 3 fails, both copies exist; the command reports a runtime error naming both paths, and the existing duplicate detection makes the record unreadable-for-write until a human removes one. No path in this sequence truncates or rewrites the source in place, so an interruption never loses content.

3. **`archive` is idempotent in effect, accepts an active record whatever its status value, and discards the custom value.** Archiving an already-archived record is a no-op success (no write, no `updated_date` change). Archiving a record that carries a custom status is the ordinary case: the patch sets `status` to `archived`, the custom value is not copied anywhere, and no separate field records it (REQ-05.007, REQ-07.001). A record stored in the active folder while its `status` reads `archived` is not an archive-time error: it is auto-relocated to `archived/` on read per decision 12; `archive` on such a record is a no-op success once relocation has occurred.

   *Revision (2026-11-14, owner decision): the archived-in-active-folder case was previously a validation error refusing writes and reporting; it is now auto-relocated on read (decision 12). The original wording is superseded by this revision; the ADR remains Accepted.*

4. **`status <id> <value>` changes the front-matter value only, and never archives.** `document::status`:
   - accepts `active` or a custom status declared for that type, and rejects any other value, including `archived`, with a validation error (exit 4) that names `archive` as the only way to archive;
   - rejects an archived record with a validation error (exit 4) naming the record as archived, changing nothing and moving nothing;
   - patches `status` and refreshes `updated_date` in one write through `record::store::write_atomic`; it performs no move, creates no folder, and leaves the file in the active directory;
   - is not offered on a type without a front-matter status (UI mockups), which keep their lifecycle in HTML metadata (REQ-05.008).

   Custom statuses therefore never enter the archive logic: an active record with a custom status is indistinguishable from an `active` record for storage, listing, archiving, and deletion.

5. **There is no restore for documents.** `record::store` loses its document restore operation, every document descriptor loses the `restore` verb, and no CLI, browser, or API path can move an archived document back. Only tasks keep archive/restore, because REQ-01.005 requires task IDs to survive that cycle.

   *Revision (2026-11-15, owner decision): milestones no longer keep archive/restore. The milestone lifecycle is fixed to `open -> closed` by `papyrus milestone close <id>`; the archive and restore verbs and the milestone archive location are removed. Only tasks keep archive/restore, because REQ-01.005 requires task IDs to survive that cycle. `close` writes `status` and `updated_date` in place and does not move the record.*

6. **`archived/` is the only recognized inactive location.** Directory resolution gains `resolve_archived_directory`; `resolve_deprecated_directory` and every read alias for `deprecated/` are deleted. A `deprecated/` folder is therefore invisible to listing, view, search, and the browser, exactly as REQ-05.007 states, and is never migrated, moved, or rewritten. Its files remain on disk until a human moves them. Orphaned `deprecated/` folders stay inert: no inventory, listing, or migration command is provided (decision 12).

7. **Delete is archive-then-unlink.** `delete` on an active document performs decision 2's archive, then removes the archived file; on an already-archived document it removes the file directly. Confirmation and `--force` are CLI concerns (REQ-05.007), not store concerns. Allocation stays `max(number present) + 1` over active and archived locations and never recomputes, so a gap is left where a file was removed. Reuse of the highest deleted number when the highest-numbered record is deleted is accepted behavior and is documented; no persisted high-water mark is introduced (decision 12; ADR-0010 decision 5 unchanged).

8. **Supersession is a lifecycle transition expressed the same way.** `ui-mockup supersede <old> <new>` sets the old mockup's HTML status to `archived`, records the back-reference, and moves it to `archived/`, using decision 2's sequence. It does not introduce a second archive mechanism.

9. **Reading a non-conforming `status` is tolerant and non-destructive.** A front-matter `status` outside the descriptor's declared vocabulary — a value removed from `statuses` after records used it, or a hand-edited string — is preserved verbatim, the record is flagged in list/view diagnostics, and a write that would normalize it is refused until the value or the configuration is fixed. A record in that state is still an ordinary active record for storage and for `archive`; only `status` writes are refused. Papyrus never silently rewrites a state value on read and never adds an undeclared value to the vocabulary.

10. **Visibility follows location, and the status value is a filter, not a visibility switch.** List, search, and browser index only the active directory; the `archived/` directory is never indexed and is not resolvable by ID for viewing (REQ-05.007, REQ-06.014). A custom status does not hide a record: it remains listed and is selectable through the browser's status filter, whose options are `active` plus the declared custom statuses. `--json` reports `status` for every reachable record, so an API consumer sees the value it is reading.

11. **Clause-level supersession, explicitly scoped.** On acceptance this ADR supersedes:
    - ADR-0010 decision 2's `<type directory>/deprecated/` idle location and decision 7's deprecate/restore move rule, for document types only;
    - ADR-0012's field-map rows for milestone, release, and front-matter document types, which gain `status`.

    ADR-0010's layout, identity, allocation and configuration-driven path decisions, and ADR-0012's metadata, patch, adoption and serialization decisions, remain in force. Because both ADRs are `Accepted` and their bodies are frozen, a consolidation ADR that restates the layout and field-map contract without the deprecated clauses is **outstanding** and is flagged in the ADR corpus audit; until it exists, this ADR is the authoritative delta and the old clauses must not be implemented.

12. **Read-path auto-relocate and settled defaults.** Owner decisions of 2026-11-14, recorded here so the ADR stays Accepted:
    - **Auto-relocate on read.** When a read path (list, view, search, browser resolution, or any ID lookup) encounters a file in the active directory whose `status` reads `archived`, Papyrus auto-relocates that file to `archived/` using decision 2's sequence: write the destination via the ADR-0006 atomic write, then unlink the source. On success the read returns the archived record. On failure (destination write or source unlink fails) the source remains intact and the read returns a read-only record together with a validation outcome describing the failure; no source write occurs and no partial state is left. This is the only permitted write during a read path. It replaces the prior "refuse writes and report" behavior for the archived-in-active-folder case.
    - **Deleted IDs.** Allocation stays `max(number present) + 1` over active and archived locations with no recomputation; reuse of the highest deleted number when the highest-numbered record is deleted is accepted behavior and is documented. No persisted high-water mark is introduced (ADR-0010 decision 5 unchanged).
    - **Orphaned `deprecated/` folders.** They stay inert: not read, migrated, inventoried, or reported. No inventory command is provided; a human moves the files. Consistent with decision 6.

## Consequences

**Positive:**

- One lifecycle operation (`archive`) replaces deprecate and restore for ten document types, removing an entire class of state-transition bugs and roughly half of each document module's command surface.
- The state is in front matter and the location follows only from the main status, so a record's lifecycle is visible in the same projection used everywhere (REQ-06.004).
- Custom statuses are configuration data with no storage consequence, so a type can gain a vocabulary without a folder, a migration, or a new lifecycle.
- The archive sequence never destroys content: the destination is written before the source is removed.
- The `deprecated/` decision is explicit and testable: the directory is not read, so an upgrade cannot half-migrate a corpus.

**Negative:**

- Existing projects with documents in `deprecated/` lose access to them through Papyrus until a human moves the files and edits their metadata; the release notes must say so (REQ-05.007 already accepts this).
- Removing restore is a user-visible capability loss; it is a deliberate product decision, not an implementation gap.
- Archive now performs a content change, so a record's bytes and `updated_date` change on archive; any test or user expectation of "move only" must change.
- Two operations (write + unlink) are not atomic together; the residual duplicate state is handled by reporting, not by a transaction.
- Reading a record whose `status` is `archived` but whose file is still in the active folder now writes during the read (auto-relocate); callers must tolerate a validation outcome instead of a hard error on failure.
- A custom status is lost on archive and cannot be restored; a record that was `in-review` before archiving reads `archived` afterwards with no history. This is the owner's decision (no retained field), not an omission.
- `status` writes are refused on a record whose stored value is no longer declared, so a configuration change can leave records needing a manual fix.

**Risks and mitigations:**

- *Data loss between write and unlink*: impossible by ordering; a test kills the process between steps and asserts the source still exists.
- *Residual duplicates*: detected by the existing duplicate rule, reported with both paths, writes refused; documented in the upgrade notes.
- *Legacy `deprecated/` visibility regression*: the requirement accepts it; the upgrade note names the folder and the manual recovery steps.
- *Status/key drift*: the key registry is the only place `status` is declared; one test asserts the three state models and that decisions and mockups are excluded.
- *`archive` mistaken for a custom-status transition*: `status` rejects `archived` and names `archive`, `archive` discards the custom value; two tests cover both directions.
- *Location/status disagreement*: a file whose `status` is `archived` but which sits in the active folder is auto-relocated on read (decision 12); a read-path write occurs and must be tested for failure handling (source intact, read-only record plus validation outcome on failure).
- *Read-path write*: auto-relocate performs a write during a read; tests kill the process between destination write and source unlink and assert the source survives and the read returns the validation outcome.
- *Status vocabulary shrink*: removing a value from `statuses` leaves records that read but cannot be written; the validator cannot detect it (it only sees configuration), so the diagnostics and the release note carry it.

## Alternatives Considered

| Option | Outcome | Reason |
|---|---|---|
| `status` in front matter (main + custom) plus `archived/`, archive = write-destination-then-unlink, no restore (this ADR) | Selected | Matches REQ-05.007/REQ-05.010/REQ-07.001, keeps one file and one ID, and never loses content on failure. |
| Keep `deprecated/` as the folder name and change only the command verbs | Rejected | Contradicts REQ-05.007's `archived/` location and its explicit "deprecated/ is not read as archived". |
| Rename the file into `archived/` first and patch `status` afterwards | Rejected | A failure between the two leaves an `status: active` record in the archive folder, which decision 3 would then have to special-case. |
| Store the state only in front matter and leave files in place | Rejected | REQ-05.007 requires the move; listing would need a per-file state filter and archive would become O(scan). |
| Give every custom status its own folder | Rejected | REQ-07.001 states custom statuses are front-matter values only, with no folder per status and no move. |
| Keep the pre-archive custom value in a second field | Rejected by REQ-07.001 | The requirement states the previous value is not retained anywhere, and a second field would need its own lifecycle. |
| Model custom statuses as a second configurable lifecycle for documents | Rejected | REQ-07.001 keeps the main statuses fixed and makes only the custom vocabulary configurable; a configurable lifecycle would make archive/delete behavior depend on configuration. |
| Keep restore for documents as a hidden command | Rejected | REQ-05.007 states no Papyrus command returns an archived document to `active`. |
| Migrate `deprecated/` automatically on first read | Rejected | REQ-05.007 explicitly forbids migration; a bulk move would also rewrite a corpus without review. |

## Open Questions

- Should a configuration change that removes a declared custom status be detected and reported as an inventory of affected records (read-only check), or left to the first failed write?
- Should `archived/` be excluded from `git` via `.gitignore` or kept under version control? Current behaviour: kept, like every other record.

## Downstream constraints

1. `deprecated/` must not appear in any descriptor, resolver, browser route, search path, or documentation after this ADR.
2. Document descriptors have no `restore` verb; `record::store` exposes `archive` and `delete` only for documents.
3. Milestone and release descriptors declare `status` with their fixed state lists; document descriptors declare the main statuses plus the validated custom list and default; decisions and UI mockups must not.
4. `archive` must use `record::patch` and the ADR-0006 atomic write; no module may move a record file with `fs::rename` alone.
5. ADR-0010 and ADR-0012 must not be implemented where they contradict decisions 1, 2, 6, and 8 until the consolidation ADR lands.
6. Only `archive` writes `archived`, only `supersede` writes it for mockups, and `status` rejects it; no other command may produce the value.
7. `status` performs no move, creates no folder, and is refused on an archived record.
8. Auto-relocate on read uses decision 2's sequence and is the only permitted write during a read path; on failure the source stays intact and the read returns a read-only record with a validation outcome.
9. No command inventories or migrates orphaned `deprecated/` folders.

## References

- `docs/requirements/business/REQ-05.007.md` — archive state, location, delete, no restore, `deprecated/` not read.
- `docs/requirements/business/REQ-05.010.md`, `REQ-05.011.md` — `status` in front matter and metadata maintenance.
- `docs/requirements/business/REQ-05.008.md` — UI mockup HTML metadata and supersession.
- `docs/requirements/business/REQ-04.002.md` — decisions have no archive state.
- `docs/requirements/business/REQ-07.001.md` — main status, custom statuses, `status` command, fixed lifecycles per category.
- `docs/requirements/business/REQ-07.002.md` — `statuses`/`default_status` configuration and validation.
- `docs/requirements/business/REQ-01.005.md` — ID stability across archive.
- `docs/architecture/adr/ADR-0006-optimistic-concurrency-and-atomic-record-writes.md` — atomic write sequence.
- `docs/architecture/adr/ADR-0010-papyrus-native-on-disk-record-layout.md` — layout, allocation, identity; deprecated clauses superseded here.
- `docs/architecture/adr/ADR-0012-papyrus-owned-front-matter-metadata.md` — key registry and field map; `status` rows superseded here.
- `docs/architecture/adr/ADR-0013-shared-record-envelope-without-a-storage-layer.md` — patch API and single writer.
- `docs/architecture/adr/ADR-0017-artifact-category-registry-and-type-descriptors.md` — `archive_policy` and `status_model` fields.
- `docs/architecture/adr/ADR-0019-fail-closed-document-type-configuration-validation.md` — status-list validation.
- Implementation: `src/record/store.rs`, `src/record/descriptor.rs`, `src/record/patch.rs`, `src/record/migrate.rs`, `src/*.rs` (deprecate/restore removal), `src/browser/records.rs`.
