---
id: TASK-111
title: Write upgrade notes and align the README with dynamic document commands
status: Done
assignee:
  - tech-writer
created_date: '2026-11-13T21:26:00Z'
updated_date: '2026-11-14T19:27:00Z'
labels:
  - docs
milestone: MS-018
dependencies:
  - TASK-103
  - TASK-104
  - TASK-106
  - TASK-107
  - TASK-109
  - TASK-110
documentation:
  - docs/release-1.0-upgrade-notes.md
  - README.md
priority: high
type: docs
ordinal: 41100
---


## Description
Documentation-only migration and reference alignment for the config-defined document-type release. `README.md` still advertises the pre-registry command inventory, the `nfreq` alias, `deploy` as a seeded type, and `deprecated/` storage, while the release notes cover only the Backlog.md import. Users upgrading therefore have no statement that static document commands are gone and no path for the documents sitting in `deprecated/`. This task publishes the migration steps and makes the README describe the command and configuration contract that TASK-105 to TASK-114 shipped.

**Why now:** REQ-007.001 to REQ-007.004 make the document command surface configuration, REQ-005.007 replaces deprecate/restore with archive and no inverse, and ADR-0016/0019/0021/0022/0023/0024 fix dispatch, validation, lifecycle, layout, and list-valued configuration. TASK-115 already corrected `docs/business-rules/`; the README and the release notes are the remaining stale user-facing documentation.

## Scope (in)

- `docs/release-1.0-upgrade-notes.md`: new document-type migration section plus an updated "Applies to" line.
- `README.md`: the `## Upgrading to Release 1.0` pointer and the two embedded command-reference blocks — `## Man Page (papyrus(1))` (`DESCRIPTION`, `COMMANDS`, `STATUSES`, `FILES`, `EXAMPLES`) and `## Configuration Man Page (papyrus-config(5))` (`Paths`, `Documents`, `EXAMPLE`).

## Scope (out)

- New man-page sources under `docs/man/`, any generator, `scripts/build-deb.sh`, `packaging/deb/`, or any NFR-003.003 compliance claim. ADR-0021 stays Proposed and the owner decision is deferred.
- `docs/requirements/**`, `docs/business-rules/**`, `docs/architecture/**`, `docs/specifications/**`, source code and tests.
- Browser, search, and init behaviour beyond the command/config wording the README needs (TASK-113 and TASK-114 own that behaviour).
- Documentation index regeneration; no file is added, so `docs/index.html` does not change.

## Authoritative sources

- REQ-007.001 — three artifact categories, document main/custom statuses, `status` command, no restore.
- REQ-007.002 — `document_types` keyed by slug, whole-map semantics, fields, defaults, fail-closed validation and repair surfaces.
- REQ-007.003 — baseline verbs, removed static commands and `nfreq`, generated help, NFR-003.003 risk.
- REQ-007.004 — config-defined generic document types.
- REQ-005.007 — `archived/` location, archive as the only archive path, no document restore, inert `deprecated/`.
- ADR-0016 (dispatch), ADR-0019 (validation), ADR-0022 (lifecycle and archive location), ADR-0023 (layout), ADR-0024 (list-valued keys), ADR-0021 (man-page strategy, Proposed).
- BR-001 (command model), BR-016 (configuration keys and precedence), BR-009/BR-010 (req/nfr extras), plus the TASK-115 rule corpus for wording.

## Dependencies

TASK-108, TASK-109, TASK-110, TASK-111, TASK-112, TASK-114, and TASK-115 are completed and are the shipped behaviour this task documents. TASK-117 verifies the resulting claims end to end and in the package.

## Assumptions

- Assumption: the document-type change ships in the same release as the Backlog.md import, so the migration content is added to the existing `docs/release-1.0-upgrade-notes.md` rather than a new file.
- Assumption: the README `<pre>` blocks are the in-repo command reference only. Editing them adds no man page, no `docs/man/` source, no generator, and no package content; ADR-0021's "README cannot remain a second copy" point is not actioned while that ADR is Proposed.
- Assumption: `document_types` is authoritative. The eleven REQ-007.002 entries are documented as the defaults `papyrus init` writes, not as a fixed command inventory.
- Assumption: the ten per-type command blocks collapse into one `<document-slug>` family section; type-specific detail survives only where a type adds something beyond the baseline (`create-group` and `--group` for req/nfr; `supersede` and no `status` for ui-mockup).
- Assumption: no new feature specification is written. The deliverable is user-facing documentation; the canonical contract stays in REQ-007.x, BR-016, and ADR-0021, and the ticket plus this plan is the implementation record.

## Open questions

None. The ADR-0021 scope decision remains with the project owner; this task documents the gap and does not resolve it.

## Acceptance Criteria
- [x] `docs/release-1.0-upgrade-notes.md` contains a document-type migration section stating that the static per-type document commands are removed, that `nfreq` no longer exists and `nfr` is the command, that `req` and `nfr` work only because they are default `document_types` entries, that `deploy` is no longer seeded and must be declared before `papyrus deploy` works, and that `deprecate` and `restore` no longer exist for documents.
- [x] The migration section explains whole-map `document_types` semantics — when the key is present exactly its entries exist and no default is re-added; removing an entry stops managing its files without deleting them; `enabled: false` removes the command and browser view while keeping the entry, prefix reservation, and files — and documents `statuses`, `default_status`, the fixed `active`/`archived` values, the `status` command, list-valued `statuses` syntax, and both repair surfaces (`config get|set|unset|list` and `init --force`).
- [x] The migration section documents that archived documents move to the `archived/` subfolder of the type's configured directory, that `archive` is the only way to reach `archived`, that documents have no `restore`, that a custom status is discarded on archive, and that a legacy `deprecated/` folder is inert — not read, migrated, or inventoried — with manual recovery steps (move the file into the type folder and set front-matter `status: active`).
- [x] README's `papyrus(1)` `COMMANDS` section contains no per-type document command blocks (req, nfreq, glossary, ui-spec, ui-mockup, tech-spec, audit, use-case, test-case, test-report, rule, including the duplicated raw-HTML block); one `<document-slug>` entry states that the command name is an enabled `document_types` slug, lists the baseline verbs (`create`, `list`, `view`, `edit`, `archive`, `delete`), documents `status <id> <value>` for front-matter-backed types, lists the type-specific additions (`create-group` and `--group` for req/nfr; `supersede` and no `status` for ui-mockup), and marks the eleven entries as `papyrus init` defaults rather than a fixed command list.
- [x] README's `papyrus(1)` `DESCRIPTION`, `STATUSES`, `FILES`, and `EXAMPLES` and the `papyrus-config(5)` `Paths`, `Documents`, and `EXAMPLE` sections describe the new model: documents use main statuses `active`/`archived` plus optional custom statuses; storage is `<doc_directory>/<document_types.<slug>.directory>/` and its `archived/` subfolder; no document `deprecated/`, document `deprecate`, document `restore`, or `nfreq` text remains except the inert-folder migration wording; `deploy` is absent from the documented defaults and shown as addable; and the config manpage documents `document_types` fields, validation, repair, and list-valued `statuses` as REQ-007.002 and BR-016 specify.
- [x] README states that config-defined slugs have no man page pending the ADR-0021 owner decision and that NFR-003.003 is not claimed for them; the upgrade notes state the same and record the gap as a release risk.
- [x] The task changes only `README.md` and `docs/release-1.0-upgrade-notes.md`: no `docs/man/` source, generator, packaging file, requirement, business rule, ADR, specification, or source file is added or modified, no NFR-003.003 compliance is claimed anywhere, and the README AI-assistance notice is intact.

## Definition of Done
- [x] #1 The document-type migration section is present in `docs/release-1.0-upgrade-notes.md` and its subsections cover removal, migration commands, whole-map `document_types`, statuses, archive and `archived/`, legacy `deprecated/`, repair, man-page risk, and upgrade verification.
- [x] #2 `README.md` contains one `<document-slug>` command family instead of per-type document command blocks, and no stale `nfreq`, document `deprecate`/`restore`, seeded `deploy`, or `/deprecated/` text remains outside the inert-folder migration wording.
- [x] #3 The config manpage in `README.md` documents `document_types` as a whole map with fields, statuses, `default_status`, validation, repair surfaces, and list-valued syntax, with `deploy` shown only as an optional user-defined entry.
- [x] #4 Every command and configuration claim changed by this task was spot-checked against the built binary on a throwaway project under `./.tmp/`, and that scratch project was removed.
- [x] #5 Relative links in both files resolve, both files end with their AI-assistance notice, and `git status` shows only `README.md` and `docs/release-1.0-upgrade-notes.md` as changed.


## Implementation Notes
1. Capture the shipped surface (read-only). Read `src/registry.rs` (`DOCUMENT_BASELINE_VERBS`, descriptor templates, `StatusModel`, slug templates) and `src/cli/builtin.rs` (`BUILTIN_COMMANDS`). In a scratch project under `./.tmp/`, run `papyrus --help`, `papyrus req --help`, `papyrus nfr --help`, `papyrus ui-mockup --help`, and `papyrus deploy --help` (expect exit 2) and record the printed inventory. Document only what the binary prints; delete the scratch project afterwards.

2. Upgrade notes — `docs/release-1.0-upgrade-notes.md`.
   - Update the `Applies to:` line so the document covers both the Backlog.md import and the document-type change.
   - Insert `## Document types are now configuration-defined` between `## Verify the import` and `## Other user-visible changes in Release 1.0`, with these subsections:
     - `### Who must act` — static document commands and `nfreq` are gone; documents under `deprecated/` are invisible until moved by hand.
     - `### Command migration` — table mapping `papyrus req deprecate` → `papyrus req archive`, `papyrus req restore` → no equivalent, `papyrus nfreq …` → `papyrus nfr …`, `papyrus deploy …` → declare a `deploy` entry first; unknown and disabled slugs exit 2.
     - `### document_types is a whole map` — when the key exists exactly its entries exist and no default is re-added; removing an entry stops managing its files without deleting them; `enabled: false` keeps the entry, prefix, and files but removes the command and browser view; list the eleven defaults `papyrus init` writes (no `deploy`).
     - `### Statuses` — fixed main statuses `active`/`archived`; custom statuses are front-matter values only; `default_status`; `papyrus <slug> status <id> <value>` sets `active` or a custom status and rejects `archived` (exit 4, names `archive`); `archive` discards the previous custom value.
     - `### Archive location and no restore` — `archived/` under the type directory, hidden from lists/search/browser, no restore command, `delete` archives first and `--force` skips confirmation.
     - `### Legacy deprecated/ folders` — inert: not read, migrated, or inventoried; manual recovery steps (move the file into the type directory, set `status: active` in front matter, then verify with `list`/`view`).
     - `### Repairing an invalid configuration` — fail-closed validation; the two repair surfaces `papyrus config get|set|unset|list` and `papyrus init --force`; list-valued `statuses` syntax, comma caveat, and refusal without writing.
     - `### Man pages and NFR-003.003` — config-defined slugs have no man page pending the ADR-0021 owner decision; NFR-003.003 is not claimed for them and the gap is a release risk.
     - `### Verify the upgrade` — numbered checks: no `nfreq`, old commands fail, archive moves to `archived/`, status rejects `archived`, repaired config runs every command, legacy folder stays inert until moved.

3. README — `## Upgrading to Release 1.0`: add a second paragraph for the document-type migration, linking `docs/release-1.0-upgrade-notes.md#document-types-are-now-configuration-defined`.

4. README — `## Man Page (papyrus(1))`.
   - Add a scope note before the `<pre>` block: the section documents built-in commands and the document family generically; config-defined slugs have no man page pending ADR-0021 and NFR-003.003 is not claimed for them.
   - `DESCRIPTION`: replace the "archive/restore and deprecate/restore cycles" sentence with the shipped ID-stability statement (IDs survive status changes and archive; documents have no restore).
   - `COMMANDS`: delete the per-type document blocks from the `req` heading through the last document block (`audit`, including the duplicate raw-HTML `use-case`/`test-case`/`test-report` block); insert one `<document-slug>` section that states the command name is an enabled `document_types` slug, that `papyrus --help` lists enabled slugs and `papyrus <slug> --help` lists that type's verbs, the baseline verbs (`create`, `list`, `view`, `edit`, `archive`, `delete`), the `status <id> <value>` verb for front-matter-backed types, the extras (`create-group` and required `--group` for req/nfr; `supersede` and no `status` for ui-mockup), and a table of the eleven init defaults explicitly labelled as configuration defaults. Keep every non-document command section unchanged.
   - `STATUSES` → `Documents`: main statuses `active`/`archived` plus optional configured custom statuses, `active` → `archived` only through `archive`, no deprecate.
   - `FILES`: replace each per-type document entry and every `<doc_directory>/<document_types.<slug>.directory>/deprecated/` entry with `<doc_directory>/<document_types.<slug>.directory>/` and `.../archived/`; add one line that a legacy `deprecated/` folder is inert.
   - `EXAMPLES`: replace the deprecate/restore example with `papyrus req archive REQ-003` and `papyrus req status REQ-003 review`; add an example that declares `document_types.deploy` with `config set` and then runs `papyrus deploy list`.

5. README — `## Configuration Man Page (papyrus-config(5))`.
   - `Paths`: `doc_directory` text refers to `archived/`, not `deprecated/`.
   - `Documents`: rewrite the `document_types` entry — keyed by slug and slug is the command name; fields `prefix`, `directory`, `enabled`, `statuses`, `default_status`; whole-map semantics; eleven defaults without `deploy`; fixed `active`/`archived` plus custom statuses; list-valued `statuses` syntax with comma caveat; validation rules (slug charset, reserved built-in names, unique prefix including disabled entries, relative in-project directory unique among enabled entries, repeated key rejected); repair surfaces. Delete the `nfreq` sentence and the `deploy` default line.
   - `EXAMPLE`: remove `deploy` from the default map; add `statuses`/`default_status` to one sample entry; show `deploy` as a commented optional user-defined entry; replace the `deprecated/` comment with the `archived/` location.

6. Consistency pass: grep both files for `nfreq`, `deprecate`, `restore` (documents only), `/deprecated/`, and a seeded `deploy`; confirm every relative link resolves, both AI-assistance notices are intact, and `git status` lists only the two files. Record the verified claims in the ticket notes for TASK-117.


## Comments
author: @task-refiner
created: 2026-11-14T18:45:00Z

---

Refined: scope in/out, 7 testable ACs, 5 DoD items, implementation plan (upgrade-notes section structure and README section list). No feature specification added; canonical contract stays in REQ-007.x, BR-016, ADR-0021. Assumption recorded: migration content extends docs/release-1.0-upgrade-notes.md.

---

author: @tech-writer
created: 2026-11-14T19:11:00Z

---

config set cannot bootstrap a new document type in one write (verified against src/commands.rs config_set and src/config.rs validate): a single document_types.<new-slug>.<field> write leaves the other required field empty, so the merged configuration fails validation and nothing is written. A mapping value is also refused because config set accepts YAML scalars only. The docs therefore send new types to a direct papyrus.config.yml edit and keep config set for existing entries. TASK-117 should verify this end to end and decide whether a code change is needed.

---

author: @tech-writer
created: 2026-11-14T19:11:00Z

---

Pre-existing upgrade-notes claim in docs/release-1.0-upgrade-notes.md (What changes > Task IDs are reallocated): task IDs are said to start from the highest number in "active, archived, and deprecated locations". Task allocation now has no deprecated location. TASK-116 did not change this (outside its document-type scope); TASK-117 should confirm task ID allocation wording/code and reconcile.

---

author: @tech-writer
created: 2026-11-14T19:11:00Z

---

README STATUSES > Milestones says active -> archived, but src/registry.rs MILESTONE_STATES is ["open", "closed"]. TASK-116 left the pre-existing text untouched; TASK-117 should verify which is authoritative and reconcile the README or the registry.

---

author: @tech-writer
created: 2026-11-14T19:11:00Z

---

config list omits document_types.<slug>.default_status: src/config.rs Config::flattened prints prefix, directory, enabled, and statuses but never default_status. The README no longer claims config list prints every effective key; TASK-117 should verify and decide whether config list or the docs need a fix.

---

author: @orchestrator
created: 2026-11-14T19:24:00Z

---

Owner approved widening tech-writer edit permission to README.md and docs/release-1.0-upgrade-notes.md (tooling-only commit, separate from the documentation diff). Product diff remains README + upgrade notes.

---

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Release 1.0 upgrade notes now explain the configuration-defined document model (migration table, whole-map document_types, eleven defaults without deploy, statuses/default_status, archive/archived/ and no document restore, inert deprecated/ recovery, repair surfaces, NFR-003.003 risk, verification steps). README lists the dynamic <document-slug> family instead of per-type blocks, documents document_types as the source of slugs/statuses, updates STATUSES/FILES/EXAMPLES and papyrus-config(5), and links the notes. No man-page source, generator, packaging change, or NFR-003.003 claim added. Evidence: claims spot-checked against the binary; stale-term sweep and link/anchor checks clean; cargo test 773 passed.
