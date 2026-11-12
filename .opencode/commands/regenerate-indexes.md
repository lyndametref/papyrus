---
description: Regenerate all documentation index.html files (ADR, business glossary, business rules, requirements) from their Markdown sources.
---

Regenerate every documentation index in this project from its Markdown sources, then report the result.

Run the generators (each is skipped if its corpus directory is absent):

!`set -e
SK="$HOME/.config/opencode/skills"
ok=0; fail=0
run() { if [ -d "$1" ]; then "$2" && ok=$((ok+1)) || fail=$((fail+1)); else echo "skip: $1 (not present)"; fi; }
run docs/architecture/adr "$SK/adr-index-visualization/scripts/generate_adr_index.sh"
run docs/domain-model/business-glossary "$SK/business-glossary-index-visualization/scripts/generate_business_glossary_index.sh"
run docs/business-rules "$SK/business-rules-index-visualization/scripts/generate_business_rules_index.sh"
run docs/requirements "$SK/requirements-index-visualization/scripts/generate_index.sh"
echo "done: $ok succeeded, $fail failed"`

For each generator that failed, read its stderr and fix the reported cause, then rerun. Never hand-edit any `index.html`.