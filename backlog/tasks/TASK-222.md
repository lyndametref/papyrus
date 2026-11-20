---
id: TASK-222
title: "Amend ADR-0019, ADR-0016, ADR-0012, ADR-0023, ADR-0035, ADR-0028, ADR-0024, ADR-0025, and ADR-0034 for the new model"
created_date: '2026-11-20T17:37:01Z'
updated_date: '2026-11-20T18:19:12Z'
labels: []
references:
  - ADR-0012
  - ADR-0016
  - ADR-0019
  - ADR-0023
  - ADR-0024
  - ADR-0025
  - ADR-0028
  - ADR-0034
  - ADR-0035
  - ADR-036
  - ADR-037
  - ADR-038
  - ADR-039
  - ADR-0005
  - ADR-0006
  - ADR-0020
  - ADR-0027
  - ADR-0026
  - REQ-07.004
  - TECH-077
status: ToDo
assignee:
  - architect
milestone: MS-031
dependencies:
  - TASK-218
  - TASK-219
  - TASK-220
  - TASK-221
ordinal: 60
documentation:
  - decisions/ADR-0019-fail-closed-document-type-configuration-validation.md
  - decisions/ADR-0016-runtime-config-derived-command-dispatch.md
  - decisions/ADR-0012-papyrus-owned-front-matter-metadata.md
  - decisions/ADR-0023-consolidated-record-layout-and-field-map.md
  - decisions/ADR-0024-list-valued-configuration-keys.md
  - decisions/ADR-0025-bundled-katex-client-side-math-rendering.md
  - decisions/ADR-0028-browser-shell-shared-list-layout-content-width-and-collapsible-navigation.md
  - decisions/ADR-035-per-family-reference-visibility-in-the-shared-browser-list-layout.md
  - decisions/ADR-034-remove-configuration-managed-artifacts-and-agent-skills-sections-from-the-browser.md
type: architecture
---

## Description
Context: ADR-036 to ADR-039 replace the descriptor and content-model decisions, but nine other ADRs state invariants that the new model changes. Leaving them unamended produces contradictory architecture.

Scope: amend in place, preserving decision numbering and recording each amendment as a dated note.
- ADR-0019: d7 key on the content model instead of the ui-mockup slug; d8 entry-set wholesale versus collection replace; d10 selector paths and adding fields file-edit-only; d11 new invariants (reserved field keys, seed-only models, unique field keys per role); d13 indexed field diagnostics.
- ADR-0016: d4 and d7 help title arguments are engine-generated; about lines and type order are frozen; per-option help prose and order may regenerate.
- ADR-0012 d2 and d6 plus ADR-0023 d5 and constraint 4: config-declared front-matter keys are a second extension point; general front-matter keys stay protected; heading ownership is stated.
- ADR-0035: replacement discriminator is model plus seed reference_visibility.
- ADR-0028: generic edit renderer; reserved field keys versus slots.
- ADR-0024: nested mapping sequences are not list-valued keys.
- ADR-0025: prose-field selection for the math pre-pass.
- ADR-0034: note that ADR-0020 d9 is already retired and must not be reintroduced as a lifecycle page.

Do not renumber decisions. Do not modify ADR-0017 or ADR-0018 here; their supersession belongs to ADR-036.
## Refinement inputs (TASK-290)

### ADR inventory (`decisions/`)
Existing records: ADR-0001 to ADR-0029, ADR-032, ADR-033, ADR-034, ADR-0035. No ADR-0030 or ADR-0031 record exists. Front-matter `id` is four-digit. Filenames for ADR-032 to ADR-0035 are three-digit (`ADR-035-...`); all other ADR filenames are four-digit.
ID convention: the managed record id is `ADR-0035`; RE-010 and parts of the corpus also write `ADR-035`. Use `ADR-0035` in `--reference` and the existing three-digit filename in `--documentation`.
Supersession targets (MS-031 AC-2, RE-010 D1):
- `decisions/ADR-0017-artifact-category-registry-and-type-descriptors.md` decisions 1-4, 7, 9, 10 and constraints 1, 4 (partial supersession; ADR-036).
- `decisions/ADR-0018-generic-document-content-model.md` in full (ADR-036).
Amendment targets (MS-031 AC-3):
- ADR-0019 d7/d8/d10/d11/d13: `decisions/ADR-0019-fail-closed-document-type-configuration-validation.md`
- ADR-0016 d4/d7: `decisions/ADR-0016-runtime-config-derived-command-dispatch.md`
- ADR-0012 d2/d6: `decisions/ADR-0012-papyrus-owned-front-matter-metadata.md`
- ADR-0023 d5 and constraint 4: `decisions/ADR-0023-consolidated-record-layout-and-field-map.md`
- ADR-0035 reference-visibility discriminator: `decisions/ADR-035-per-family-reference-visibility-in-the-shared-browser-list-layout.md`
- ADR-0028 generic edit renderer and reserved field keys vs slots: `decisions/ADR-0028-browser-shell-shared-list-layout-content-width-and-collapsible-navigation.md`
- ADR-0024 nested mapping sequences: `decisions/ADR-0024-list-valued-configuration-keys.md`
- ADR-0025 prose-field selection: `decisions/ADR-0025-bundled-katex-client-side-math-rendering.md`
- ADR-0034 note that ADR-0020 d9 is already retired: `decisions/ADR-034-remove-configuration-managed-artifacts-and-agent-skills-sections-from-the-browser.md`
New ADRs authored in this milestone: ADR-036 (TASK-218), ADR-037 (TASK-219), ADR-038 (TASK-220), ADR-039 (TASK-221).

### Refinement assumptions / definition of done
The source of truth is RE-010 D1-D14 (accepted, not reopenable) and MS-031 AC-2/AC-3. No decision is renumbered, softened, split, or reopened. Done means every acceptance criterion passes, the record is accepted and user-verified, and no source, test, or packaging file changes (MS-031 AC-10).
## Acceptance Criteria
- [ ] All nine target ADR files are amended in place with decision numbering preserved and each amendment recorded as a dated note (MS-031 AC-3).
- [ ] ADR-0019 d7, d8, d10, d11, and d13; ADR-0016 d4 and d7; ADR-0012 d2 and d6; ADR-0023 d5 and constraint 4; ADR-0028; ADR-0024; ADR-0025; and ADR-0034 contain every amendment listed in the task scope (MS-031 AC-3).
- [ ] ADR-0035's reference-visibility discriminator is replaced by model plus seed reference_visibility, using the existing file `decisions/ADR-035-per-family-reference-visibility-in-the-shared-browser-list-layout.md` (MS-031 AC-3, RE-010 D8).
- [ ] ADR-0034 notes that ADR-0020 d9 is already retired and that a lifecycle page must not be reintroduced (MS-031 AC-3).
- [ ] ADR-0017 and ADR-0018 are not modified by this task; their supersession remains in ADR-036 (MS-031 AC-2A, AC-3).
- [ ] A cross-check finds no remaining contradictory statement among the nine amended ADRs, and no source, test, or packaging file is changed (MS-031 AC-3, AC-10).
