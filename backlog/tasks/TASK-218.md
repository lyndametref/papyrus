---
id: TASK-218
title: "Write ADR-A: config-declared document field schema and one descriptor model"
created_date: '2026-11-20T17:36:36Z'
updated_date: '2026-11-20T18:18:07Z'
labels: []
references:
  - ADR-0017
  - ADR-0018
  - ADR-0019
  - REQ-07.002
  - REQ-07.004
  - TECH-070
  - TECH-077
  - REQ-07.001
  - REQ-07.003
  - TECH-068
  - TECH-071
  - TECH-072
status: ToDo
assignee:
  - architect
milestone: MS-031
dependencies:
  - TASK-217
ordinal: 20
documentation:
  - decisions/ADR-036-config-declared-document-field-schema.md
  - decisions/ADR-0017-artifact-category-registry-and-type-descriptors.md
  - decisions/ADR-0018-generic-document-content-model.md
type: architecture
---

## Description
Context: ADR-0017 built the artifact category registry on type descriptors with a shared ContentModel (ADR-0018). The refactor needs one descriptor model whose per-type fields are declared in configuration, and ADR-0018's seven-key ContentModel cannot express role-specific field sets or an HTML model.

Scope: author ADR-036, "Config-declared document field schema and one descriptor model".
- Supersede ADR-0017 decisions 1-4, 7, 9, 10 and constraints 1, 4.
- Supersede ADR-0018 in full: decisions 1, 3, 5, 6, 7, 9, 10 and constraints 2, 5, 6.
- Own the DocumentModel enum with variants SingleMarkdown, GroupedMarkdown, SingleHtmlMockup, replacing ContentModel. SingleMarkdown: one record, plain ID, baseline verbs plus status, one field set. GroupedMarkdown: group record with plain ID plus member records with dotted IDs, baseline plus status plus create-group, role field sets, group link owned by the engine. SingleHtmlMockup: one record, plain ID, baseline plus supersede, no status verb, fixed pending/approved/archived metadata.
- Own role field sets, the seed table location and contents, and the field vocabulary: key, heading, frontmatter_key, option, label, kind (text|multiline|list), required, verbs (default [create, edit]), help; no order or enabled; sequence position is order for body and CLI options.
- Own default-fill for scalars and collection-replace for fields, group.fields, member.fields, statuses; prefix and directory are never filled; absent slug stays absent.
- Own the seed-only restriction: GroupedMarkdown and SingleHtmlMockup are seed-only; only req/nfr are grouped; only ui-mockup is HTML; user-defined types default to SingleMarkdown.
- State that the engine emits only PatchOps and never writes record bodies directly.
- Record reserved field keys id, title, status, labels, references, created_date, updated_date, group; heading and/or frontmatter_key required; field keys unique per role.

## Refinement inputs (TASK-290)

### ADR inventory and supersession targets (`decisions/`)
Existing records: ADR-0001 to ADR-0029, ADR-032, ADR-033, ADR-034, ADR-0035; there is no ADR-0030 or ADR-0031 record. Front-matter `id` is four-digit; filenames for ADR-032 to ADR-0035 are three-digit (`ADR-035-...`). `ADR-0035` is the managed id; `ADR-035` is an alias used by RE-010 and parts of the corpus.
Supersession inputs:
- `decisions/ADR-0017-artifact-category-registry-and-type-descriptors.md` — decisions 1-4, 7, 9, 10 and constraints 1, 4.
- `decisions/ADR-0018-generic-document-content-model.md` — in full.
ADR-0019 is read for validation context and is not superseded here. ADR-0016 d4 is amended by ADR-037, not by this record.
Output path: `decisions/ADR-036-config-declared-document-field-schema.md`.

### Refinement assumptions / definition of done
The source of truth is RE-010 D1-D14 (accepted, not reopenable) and MS-031 AC-2/AC-3. No decision is renumbered, softened, split, or reopened. Done means every acceptance criterion passes, the record is accepted and user-verified, and no source, test, or packaging file changes (MS-031 AC-10).

## Acceptance Criteria
- [ ] ADR-036 exists at `decisions/ADR-036-config-declared-document-field-schema.md` with status accepted (MS-031 AC-2A).
- [ ] ADR-036 explicitly supersedes ADR-0017 decisions 1-4, 7, 9, 10 and constraints 1, 4, and supersedes ADR-0018 in full, with no other ADR-0017 decision superseded (MS-031 AC-2A).
- [ ] ADR-036 defines DocumentModel (SingleMarkdown, GroupedMarkdown, SingleHtmlMockup) replacing ContentModel and states the field vocabulary, reserved field keys, default-fill and collection-replace rules, and seed-only restrictions listed in the scope (MS-031 AC-2A, RE-010 D5, D6, D14).
- [ ] ADR-036 states that the engine emits only PatchOps and never writes record bodies, and that title is fixed and non-configurable per RE-010 D13 (MS-031 AC-2A).
- [ ] ADR-036 records the affected requirement and TECH inputs as references and leaves every requirement, TECH, source, test, and packaging file unchanged (MS-031 AC-10).
