---
name: business-terminology-consistency-check
description: 'Check docs and backlog for non-canonical business terms, conflicting term variants, and missing glossary alignment. Use when validating wording consistency before or after glossary changes.'
argument-hint: 'Examples: check term consistency for intervention; audit terminology in docs and backlog'
user-invocable: true
disable-model-invocation: false
---

# Business Terminology Consistency Check

## Outcome
Detect inconsistent business terminology and propose precise replacements aligned with the canonical business glossary.

## When To Use
- After adding or updating a business glossary entry
- Before renaming or merging glossary terms
- During requirement/domain refactors that touch business wording
- Before release documentation reviews

## Canonical Source
- Canonical business glossary directory: `docs/domain-model/business-glossary/`
- Each term is defined in its own file: `docs/domain-model/business-glossary/<kebab-case-term>.md`

## Scope
- Include:
  - `docs/requirements/`
  - `docs/domain-model/`
  - `docs/architecture/`
  - `docs/specs/`
  - `backlog/`
- Exclude by default:
  - `.ai/`
  - `.github/skills/`
  - agent examples and skill internals

## Procedure
1. Collect canonical terms.
- Read glossary terms and definitions from the canonical source.
- Build preferred labels and allowed aliases.

2. Scan for variants.
- Find case variants, plural variants, legacy names, and near-synonyms.
- Detect terms used without glossary entry when they appear repeatedly.

3. Classify findings.
- `conflict`: same term used with different meanings.
- `variant`: different labels for same meaning.
- `orphan`: repeated term not present in glossary.
- `drift`: wording diverges from current glossary definition.

4. Propose fixes.
- Provide file-level replacements that preserve requirement intent.
- Prefer minimal wording edits over broad rewrites.
- Do not apply edits unless explicitly requested.

5. Validate after apply (if requested).
- Re-scan target files.
- Confirm conflicts and variants are resolved.

## Completion Checks
- All conflicts and variants are listed with evidence.
- Proposed canonical replacement is provided for each issue.
- High-risk changes (could alter requirement intent) are explicitly flagged.

## Output Template
- Canonical term set reviewed: <count>
- Files scanned: <count>
- Findings:
  - conflict: <n>
  - variant: <n>
  - orphan: <n>
  - drift: <n>
- Proposed replacements: <list>
- High-risk items requiring approval: <list>
