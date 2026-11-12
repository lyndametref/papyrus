---
name: business-glossary-index-visualization
description: 'Regenerate docs/domain-model/business-glossary/index.html — a searchable detail view of business glossary terms — from one-Markdown-file-per-term glossary files. Use after any glossary term file change or when asked to refresh the glossary index.'
argument-hint: 'Run after glossary changes, e.g. "regenerate the glossary index" or "refresh index.html".'
user-invocable: true
disable-model-invocation: false
---

# Business Glossary Index Visualization

## Outcome

`docs/domain-model/business-glossary/index.html` is the only generated glossary index. It is regenerated from the current one-file-per-term glossary Markdown files. Markdown files are source inputs; no Markdown index is required or generated.

## When To Use

- After creating, updating, renaming, or removing a glossary term file.
- On explicit request to regenerate or refresh the glossary index.

## Prerequisites

- This skill bundles its parser at `scripts/generate_business_glossary_index.sh` (POSIX sh + awk) and its presentation at `templates/index.html`.
- Rendering uses the shared POSIX sh/awk renderer from the `html-template-migration` skill.
- The target project has `docs/domain-model/business-glossary/` with one or more `<kebab-case-term>.md` files.
- Each term file uses the canonical `# Term` heading and a `## Definition` section.

If the bundled script is missing, stop and report it; do not create a replacement silently.

## Procedure

1. From the target project root, run:

   ```
   ~/.config/opencode/skills/business-glossary-index-visualization/scripts/generate_business_glossary_index.sh
   ```

    It defaults to `docs/domain-model/business-glossary`. Pass a directory to override; an optional second argument selects a different output path:

   ```
    ~/.config/opencode/skills/business-glossary-index-visualization/scripts/generate_business_glossary_index.sh path/to/business-glossary path/to/index.html
   ```

2. On success, report `wrote <output>: N business glossary terms`.
3. On failure, inspect stderr and fix the reported source issue before retrying. The script removes its temporary data file and leaves the destination unchanged.
4. Never hand-edit `index.html`; regenerate it from the term files.

## What The Script Does

- Reads and validates each glossary term file heading and required `## Definition` section.
- Rejects malformed filenames, duplicate term slugs, and missing required content.
- Builds a JSON payload and renders it through `templates/index.html`, which provides a filterable term list and full Definition (and Aliases when present).
- Does not read, create, or update a Markdown index.

## Failure Modes

| Message | Cause | Fix |
|---|---|---|
| `business glossary directory not found: <dir>` | Directory is missing | Run from project root or pass the directory explicitly |
| `no glossary term files found in <dir>` | No `*.md` term files are present | Add canonical glossary term files before generating the index |
| `bad glossary filename: <file>` | Filename does not match `<kebab-case-term>.md` | Rename the file to the canonical format |
| `bad title` / `missing Definition` | Term heading or required section is malformed or empty | Correct the glossary source file |
| `missing value` / `unused value` / `unresolved placeholder` | Template and payload do not match | Report the details; do not edit generated HTML by hand |
| `expected N id entries` / `id ... missing from parsed data` | Parser or source problem | Check glossary filenames and rerun |

## Completion Checks

- Script exits 0 and reports the number of glossary terms written.
- The reported count equals the number of valid term files.
- `<glossary-dir>/index.html` exists and is newer than its source term files.
- The generated page includes a link back to the main documentation index (`docs/index.html`).
- No Markdown index was created or updated.
