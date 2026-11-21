---
id: TASK-291
title: "Restore a TECH body edit path and repair TECH-077 after the description-only edit"
created_date: '2026-11-21T06:11:39Z'
updated_date: '2026-11-21T06:11:39Z'
labels: []
references:
  - TECH-077
  - TECH-068
  - TECH-069
  - TECH-007
status: ToDo
assignee: []
milestone: null
dependencies: []
ordinal: 41626
documentation: []
type: chore
---

## Description
## Context

`papyrus tech-spec edit` exposes only `--description` and `--editor`.
`--description` patches the `## Description` H2 block; every other H2 section of
a TECH record is body content the CLI cannot address. The configured editor is
`nano`, which requires a TTY.

## Observed

- Rewriting TECH-077 through `--description` with the corrected full text
  appended the text instead of replacing the body: the record now holds a
  duplicated section set.
- The architect-requested corrections cannot be applied to their target
  sections with the current CLI:
  - TECH-077 section 6: comma syntax and collection reset limited to
    `statuses`; `slots` and `api_aliases` are declarative file data and not
    `config set` or `config unset` targets.
  - TECH-068 section 4.1: current metadata order with references, dependencies,
    and dependents disclosures, each only when non-empty; section 4.2.3 and the
    test plan: mockup `specs` and `refs` editable, `references` and `labels`
    read-only.
  - TECH-069 section 4 row B-02: fixed column order `ID, Title, Status, family
    slots, Created, Updated, Actions`.
  - TECH-007 sections 4.6, 4.12, 4.13, 4c, and 5.5: three-category sidebar
    label, current metadata order with three disclosures and the
    required-empty rule, mockup fixed metadata edit contract, and list-valued
    repeated form fields that accumulate.

## Required

1. Restore `docs/specifications/technical/TECH-077.md` from version control
   (the committed state), removing the duplicated content.
2. Apply the four corrections through a real body-edit path.
3. Give `papyrus tech-spec edit` a non-interactive whole-body edit option (for
   example `--body -` reading the full body from standard input) so an agent
   without a TTY can rewrite a section set without duplication.
4. Leave the four sections `draft` until architect conformance is re-run.

## Acceptance Criteria

- [ ] TECH-077 contains no duplicated section set and matches the committed body plus the section 6 correction.
- [ ] TECH-068, TECH-069, and TECH-007 carry the requested corrections in their target sections.
- [ ] `papyrus tech-spec edit` can replace a whole TECH body without an interactive editor.
