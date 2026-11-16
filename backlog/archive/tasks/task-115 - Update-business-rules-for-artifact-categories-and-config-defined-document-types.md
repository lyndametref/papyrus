---
id: TASK-115
title: >-
  Update business rules for artifact categories and config-defined document
  types
status: Done
assignee:
  - business-analyst
created_date: '2026-11-13T21:26:00Z'
updated_date: '2026-11-14T18:39:00Z'
labels:
  - docs
  - requirements
milestone: m-17
dependencies:
  - TASK-108
  - TASK-109
  - TASK-110
  - TASK-111
  - TASK-112
documentation:
  - docs/requirements/business/REQ-07.001.md
  - docs/business-rules/BR-016-configuration-keys-and-precedence.md
  - docs/requirements/business/REQ-04.002.md
  - docs/requirements/business/REQ-07.002.md
  - docs/requirements/business/REQ-07.003.md
  - docs/requirements/business/REQ-07.004.md
  - docs/requirements/business/REQ-05.007.md
  - docs/architecture/adr/ADR-0023-consolidated-record-layout-and-field-map.md
  - >-
    docs/architecture/adr/ADR-0019-fail-closed-document-type-configuration-validation.md
  - docs/business-rules/BR-010-nfreq-command-behavior.md
priority: high
ordinal: 41090
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Documentation-only alignment of the business-rule corpus with the artifact-category and config-defined document-type model.

Why: after TASK-105 to TASK-114 the shipped CLI derives one command family per enabled `document_types` slug (REQ-07.003), validates the type map fail-closed (REQ-07.002, ADR-0019), supports list-valued `statuses` (ADR-0024), and stores document state as `active`/`archived` plus custom front-matter statuses (REQ-07.001, REQ-05.007, ADR-0022/0023). The rules under `docs/business-rules/` still describe a fixed per-type command set with `deprecate`/`restore`, the `nfreq` alias, `deploy` as a seeded type, and a type map without slugs, statuses, or validation. They contradict the requirements, the ADRs, and the code.

In scope: 14 rule files in `docs/business-rules/` — BR-001, BR-007, BR-008, BR-009, BR-010, BR-011, BR-012, BR-013, BR-014, BR-016, BR-017, BR-018, BR-019, BR-020 — plus the regenerated `docs/business-rules/index.html`.

Out of scope: `docs/requirements/**` (never edited), `docs/architecture/**`, `README.md` and man pages (separate README alignment work), `docs/domain-model/**` and `docs/specifications/**`, BR-002 to BR-006, BR-015, and all source code.

Authoritative sources: REQ-07.001 to REQ-07.004, REQ-04.002, REQ-05.007, REQ-05.001, REQ-05.002, REQ-05.008; ADR-0016, ADR-0017, ADR-0018, ADR-0019, ADR-0022, ADR-0023, ADR-0024; `src/registry.rs` as a read-only cross-check of the shipped verb and status model.

Dependencies: the five prerequisites are tracked in the Backlog dependency graph and are completed (TASK-108, TASK-109, TASK-110, TASK-111, TASK-112).

Convention change: the changed rules stop mirroring `README.md` verbatim. They state the contract in prose grounded in the requirements and ADRs, and drop `README.md` line-range citations that no longer match the shipped command set. BR-015 and BR-002 to BR-006 keep their current excerpts.

BR-010 is converted, not dropped: it becomes the `nfr` rule so `papyrus nfr` keeps a rule page and REQ-05.002 traceability stays substantively correct. Rule numbering stays stable; BR-011 to BR-020 are not renumbered.

Assumptions:
- Assumption: `README.md` and the man pages still describe the pre-registry command set when this task runs, so changed rules cite requirements and ADRs rather than README excerpts; re-syncing the excerpts belongs to the README alignment work.
- Assumption: these rule files are documentation here; no front-matter adoption (`id`, dates, `status`) is performed by hand, and no rule is created or deleted.
- Assumption: `src/registry.rs` reflects the shipped model after TASK-105 to TASK-114; a disagreement between it and the requirements is reported in the task notes, not fixed in code.
- Assumption: requirements and ADRs are authoritative and are not edited; conflicts outside `docs/business-rules/` are follow-ups.
- Assumption: business-rule numbering is stable, so BR-010 keeps its number as the `nfr` rule and no gap or renumbering is introduced.
- Assumption: the eleven default document types listed in REQ-07.002 are the complete seeded set, and `deploy` is not one of them.

Follow-ups recorded, not fixed here: stale `active -> deprecated` wording and BR-010 references in `docs/domain-model/**` and in completed milestone specifications, and the README/man-page realignment.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 BR-001 states the config-derived command model: each enabled document_types entry has a papyrus <slug> command; the baseline verbs are create, list, view, edit, archive, delete; the baseline is a lower bound, not an exhaustive verb set; front-matter document types add status <id> <value> and UI mockups do not; papyrus --help lists every enabled slug; an unknown or disabled slug is a usage error; the static per-type command inventory and the nfreq name are stated as removed.
- [x] #2 BR-009, BR-010, BR-011, BR-012, BR-013, BR-014, BR-017, BR-018, BR-019, BR-020 each state for their type: it is an ordinary document_types.<slug> entry that can be renamed, removed, or given a different prefix and directory under doc_directory; it has the baseline verbs; req and nfr additionally declare create-group and --group on create, and no other type in this list declares an extension verb; records use the document status model with main statuses active/archived and optional configured custom statuses; archive is the only way to archive and there is no deprecate or restore; the type-specific content fields described today are retained.
- [x] #3 BR-010 is converted to the nfr rule: the file is renamed to BR-010-nfr-command-behavior.md with git mv, keeps the heading # BR-010: Papyrus nfr Command Behavior, and covers papyrus nfr only, with no nfreq command, alias, or lifecycle claim; BR-011 to BR-020 keep their current numbers and no rule is renumbered.
- [x] #4 BR-016 documents document_types keyed by slug, where the key is the command name and no separate document-slug key exists; per-entry prefix, directory, enabled, optional statuses and default_status, with active/archived always declared and default_status never archived; the eleven seeded defaults req, nfr, glossary, rule, ui-spec, ui-mockup, audit, tech-spec, use-case, test-case, test-report and no deploy; whole-map semantics including removal versus enabled:false; list-valued statuses written by config set from a comma-separated value and stored as a YAML flow sequence; slug character rules and case-insensitive rejection of built-in command names; prefix uniqueness across all entries and directory uniqueness among enabled entries; fail-closed validation that names the offending key; and the two repair surfaces config get|set|unset|list and init --force. The existing precedence order is unchanged.
- [x] #5 BR-008 contains no decision archive command line, no archive description, and no statement that a decision can be archived or carries an archive state; its status list is exactly proposed, accepted, rejected, deprecated, superseded; its rationale cites REQ-04.002 under its current title Update and Supersede Decisions.
- [x] #6 Every rule changed by this task separates the document main statuses active and archived from custom front-matter statuses, states that archiving sets archived, does not retain the previous custom value, and has no inverse command, keeps decision statuses distinct from document statuses, and keeps the milestone open -> closed and release planned -> released lifecycles fixed and non-configurable. BR-007 is updated to state planned -> released and to stop claiming active -> released.
- [x] #7 No rule under docs/business-rules/ retains deprecate or restore for a document type, the nfreq command or alias, deploy in the default document_types set, deprecated/ as the document inactive location, or a static per-type command table. Task and milestone restore wording, the decision status deprecated, and historical ADR text are unaffected.
- [x] #8 docs/business-rules/index.html is regenerated from the rule files with the business-rules index script; the script reports the same number of rules as there are BR-*.md files, the page lists BR-001 to BR-020 with BR-010 titled Papyrus nfr Command Behavior, and no Markdown index is created or edited.
- [x] #9 Cross-references inside the corpus resolve: every changed rule keeps at least one REQ- reference in its Rationale, every BR-, REQ-, and ADR- identifier cited in a changed rule resolves to an existing file, and the canonical format (# BR-XXX: Title, Description, Rationale, Applicability) holds for every file. Dangling references outside the corpus are recorded as follow-ups, not edited here.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 All acceptance criteria verified, with the forbidden-string grep output, the identifier check result, and the index rule count recorded in the task implementation notes.
- [x] #2 The task diff touches only docs/business-rules/ rule files and the regenerated index.html; nothing under docs/requirements/, docs/architecture/, docs/domain-model/, docs/specifications/, README.md, or src/ changes.
- [x] #3 Every changed rule passes the canonical-format check and its Rationale cites at least one existing requirement.
- [x] #4 docs/business-rules/index.html is newer than every rule file and was produced by the index script, not edited by hand.
- [x] #5 Follow-ups are recorded in a task comment: domain-model and completed-specification staleness, and the README/man-page realignment.
<!-- DOD:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Baseline inventory: read the 14 target rules; grep docs/business-rules for deprecate, restore, nfreq, deploy, deprecated/, and active -> and record the affected lines. Cross-check the expected verbs and statuses against src/registry.rs and REQ-07.001/REQ-07.003.
2. Rewrite rule text in this order: BR-001 (categories, document families, baseline verbs, status verb, --help inventory, removed static commands), BR-016 (keys, eleven defaults, list-valued statuses, whole-map semantics, validation, repair surfaces), BR-007 (release lifecycle), BR-008 (decision archive removal), BR-009 and BR-010 (req and nfr families with create-group/--group), BR-011 to BR-014 and BR-017 to BR-020 (baseline-only types).
3. git mv docs/business-rules/BR-010-nfreq-command-behavior.md docs/business-rules/BR-010-nfr-command-behavior.md, then rewrite the body for papyrus nfr.
4. Cross-reference check: extract every BR-\\d{3}, REQ-\\d{2}(\\.\\d{3})?, and ADR-\\d{4} from the changed rules, assert each target file exists, and re-run the forbidden-string grep requiring empty output.
5. Regenerate the index with the project skill script (.opencode/skills/business-rules-index-visualization/scripts/generate_business_rules_index.sh), confirm the reported count matches the 20 BR files, and inspect the BR-010 entry title.
6. Record follow-ups in a task comment and report the changed BR ids and requirement references.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
TASK-115 implementation evidence.

- Rewrote 14 rules: BR-001, BR-007, BR-008, BR-009, BR-010, BR-011, BR-012, BR-013, BR-014, BR-016, BR-017, BR-018, BR-019, BR-020.

- Renamed BR-010-nfreq-command-behavior.md to BR-010-nfr-command-behavior.md; H1 is # BR-010: Papyrus nfr Command Behavior.

- Index: generate_business_rules_index.sh reported "wrote docs/business-rules/index.html: 20 business rules"; index mtime is newer than every rule; BR-010 entry titled Papyrus nfr Command Behavior.

- Canonical format: 20/20 BR files have # BR-XXX: Title plus Description, Rationale, Applicability; every Rationale contains a REQ- reference.

- Identifier check: every extracted BR-, REQ-, and ADR- id resolves to an existing file.

- Forbidden-string grep on the changed rules: "papyrus nfreq" none; "deploy:" none; "deprecated/" none; "active ->" none; word deprecate none; restore only in BR-005 task lines 28 and 127 (allowed). The nfreq word remains only in BR-001 as the required removal statement; deploy remains only in BR-016 as the required not-seeded statement.
<!-- SECTION:NOTES:END -->

## Comments

<!-- COMMENTS:BEGIN -->
created: 2026-11-13T21:59:00Z

---

Scope addition from TASK-105 refinement Q2: TASK-108 removes the `papyrus decision archive` CLI surface; this task removes the matching BR-008 documentation so the rule and the command set agree with REQ-04.002.

---

created: 2026-11-14T18:16:00Z

---

Refinement 2026-11-14: AC#3 changed from retiring BR-010 to converting it into the nfr rule (user decision). Rationale: papyrus nfr remains a real command family with create-group/--group, so retiring BR-010 would leave it with no rule page and break REQ-05.002 traceability. Consequence: AC#2 now includes BR-010; the file is renamed to BR-010-nfr-command-behavior.md; no renumbering. Scope also adds BR-007 (stale active -> released lifecycle) so AC#6 holds; BR-002 to BR-006 and BR-015 stay unchanged.

---

created: 2026-11-14T18:22:00Z

---

Follow-ups (not fixed here): (1) docs/domain-model/** still describes active -> deprecated and may cite BR-010 as nfreq; (2) completed milestone specifications may carry the pre-registry command set; (3) README.md and man pages still document static per-type commands, deprecate/restore, nfreq, and a seeded deploy entry, to be realigned in TASK-116. No requirements, architecture, domain-model, or code files were edited. The environment rejected git mv, so the BR-010 rename was performed with mv and remains an unstaged delete+add in git.

---

author: @business-analyst
created: 2026-11-14T18:34:00Z

---

Quality-review follow-up (business-analyst). BR-007 now states the release verbs `create`, `list`, `view`, `edit`, and `close`, with `papyrus release close <id>` as the only `planned -> released` transition, matching REQ-03.002. BR-016 fail-closed wording now excludes the repair surfaces (`config get|set|unset|list` and `init --force`), consistent with its own Repair surfaces section. `docs/business-rules/index.html` regenerated (20 rules). Evidence: forbidden strings (deprecate, deprecated/, active ->, nfreq, deploy) absent except the allowed decision status `deprecated`, the removal statements for `nfreq`, and the explicit `deploy` non-seeded note; all BR/REQ/ADR identifiers in the corpus resolve. Code gap (not edited): `src/release.rs` `ReleaseCmd` has no `Close` variant, `src/registry.rs` has no release transition handler, and `src/browser/render.rs` still prints `papyrus release status <id> released`; needs a follow-up code task. Also stale and out of scope: `docs/specifications/release-lifecycle.md` still says `active -> released` and keeps the transition as an open question.

---

author: @business-analyst
created: 2026-11-14T18:34:00Z

---

Rename status update: `git mv` is blocked by the environment (permission.rejected), so the BR-010 rename is recorded as a staged delete of `BR-010-nfreq-command-behavior.md` plus a staged add of `BR-010-nfr-command-behavior.md` (`git status`: `D` + `A`). Git cannot pair them as a rename at the default threshold because the file was fully rewritten (2% similarity per `git diff --cached -M1% --summary`); both paths are present in the index, so the pair is committed together and the old path carries no content.

---
<!-- COMMENTS:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
14 business rules rewritten to the config-defined model: BR-001 dynamic document family + baseline verbs + status; BR-009/010 and BR-011-014/017-020 each document their type as an ordinary document_types entry with prefix/folder, baseline family, and extensions only where declared; BR-010 converted from nfreq to nfr; BR-016 documents slug-keyed document_types, 11 defaults without deploy, whole-map, list-valued statuses, validation and repair surfaces; BR-008 removes decision archive; BR-007 documents release close as the only planned->released transition. Index regenerated (20 rules), identifiers resolve, forbidden strings absent. Follow-ups recorded: release close code gap, README/man pages (TASK-116), domain-model/spec staleness.
<!-- SECTION:FINAL_SUMMARY:END -->
