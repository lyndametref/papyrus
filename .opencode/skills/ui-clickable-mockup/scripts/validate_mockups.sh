#!/bin/sh
# Validate single-file mockup review packages in a mockups directory.
# Usage: validate_mockups.sh [mockups-dir]
# A valid package is ONE self-contained HTML per ticket containing:
#   1. a review landing (ticket + one button per platform),
#   2. one clickable prototype per platform (current feature, happy path only).
# No design deck is allowed.
# Also checks unique ids, registry rows, platform coverage, and link targets.
set -eu

dir=${1:-docs/specifications/ui/mockups}
registry="$dir/README.md"
[ -d "$dir" ] || { echo "error: mockups directory not found: $dir" >&2; exit 1; }
[ -f "$registry" ] || { echo "error: registry not found: $registry" >&2; exit 1; }
[ -f "$dir/wireframe.css" ] || { echo "error: shared stylesheet missing: $dir/wireframe.css" >&2; exit 1; }

errors=0
err() { echo "error: $*" >&2; errors=$((errors + 1)); }

files=""
for f in "$dir"/MOCK-*; do
  [ -e "$f" ] || continue
  b=${f##*/}
  case "$b" in
    MOCK-[0-9][0-9][0-9][0-9]-*.html) files="$files $f" ;;
    MOCK-[0-9][0-9][0-9][0-9]-*) err "unsupported artifact (one .html review file per ticket): $b" ;;
    *) err "non-conforming filename: $b" ;;
  esac
done
[ -n "$files" ] || { echo "error: no MOCK-*.html files in $dir" >&2; exit 1; }

ids=""
for f in $files; do
  b=${f##*/}
  id=${b#MOCK-}
  ids="$ids ${id%%-*}"
done

dups=$(printf '%s\n' $ids | sort | uniq -d)
[ -z "$dups" ] || err "duplicate MOCK ids across files:$dups"

for f in $files; do
  b=${f##*/}
  id=${b#MOCK-}
  id=${id%%-*}

  grep -q 'data-mock-id="MOCK-'"$id"'"' "$f" \
    || err "$b: data-mock-id missing or does not match the file id"
  mid_count=$(grep -o 'data-mock-id="MOCK-[0-9][0-9][0-9][0-9]"' "$f" | sort -u | wc -l)
  [ "$mid_count" -eq 1 ] \
    || err "$b: a file must carry exactly one MOCK id (found $mid_count)"
  grep -q 'href="wireframe.css"' "$f" \
    || err "$b: missing <link> to wireframe.css"
  grep -q 'data-review="index"' "$f" \
    || err "$b: missing review landing (data-review=\"index\")"
  if grep -q 'data-review="deck"' "$f"; then
    err "$b: design deck is not allowed (landing + prototypes only)"
  fi

  platforms=$(grep -o '<[a-z][^>]*wf-proto[^>]*>' "$f" \
    | sed -n 's/.*data-platform="\([^"]*\)".*/\1/p')
  [ -n "$platforms" ] || err "$b: no platform prototype (.wf-proto with data-platform)"

  screens=$(grep -o '<section[^>]*wf-screen[^>]*>' "$f" || true)
  bad=$(printf '%s\n' "$screens" | grep '^<section' | grep -v 'data-screen=' || true)
  [ -z "$bad" ] || err "$b: .wf-screen section without data-screen"
  bad=$(printf '%s\n' "$screens" | grep '^<section' | grep -v 'data-platform=' || true)
  [ -z "$bad" ] || err "$b: .wf-screen section without data-platform"

  for p in $platforms; do
    count=$(printf '%s\n' "$screens" | grep -c "data-platform=\"$p\"" || true)
    [ "$count" -ge 1 ] || err "$b: platform '$p' has no .wf-screen view"
    grep -q "href=\"#$p" "$f" \
      || err "$b: landing/chrome has no link to platform '$p'"
  done

  views=$(grep -o 'href="#[^"]*/[^"]*"' "$f" | sed 's/^href="#//; s/"$//' | sort -u || true)
  for v in $views; do
    sid=${v#*/}
    grep -q "data-screen=\"$sid\"" "$f" \
      || err "$b: link '#$v' targets a missing data-screen"
  done

  gotos=$(grep -o 'data-goto="[^"]*"' "$f" | sed 's/^data-goto="//; s/"$//' | sort -u || true)
  for g in $gotos; do
    grep -q "data-screen=\"$g\"" "$f" \
      || err "$b: data-goto=\"$g\" targets a missing data-screen"
  done

  if grep -qE '(src|href)="https?://' "$f"; then err "$b: external resource reference found"; fi
  if grep -qE '(src|href)="//' "$f"; then err "$b: protocol-relative external reference found"; fi
  if grep -q 'TODO' "$f"; then err "$b: unresolved TODO marker"; fi

  row=$(grep "^| *MOCK-$id " "$registry" || true)
  if [ -z "$row" ]; then
    err "$b: missing registry row"
  else
    for p in $platforms; do
      printf '%s' "$row" | grep -q "$p" \
        || err "$b: registry row does not list platform '$p'"
    done
    case "$row" in
      *"| draft |"*|*"| reviewed |"*|*"| approved |"*) ;;
      *) err "$b: registry status must be draft, reviewed, or approved" ;;
    esac
  fi
done

reg_ids=$(sed -n 's/^| *MOCK-\([0-9][0-9][0-9][0-9]\) .*/\1/p' "$registry" | sort)
for id in $ids; do
  printf '%s\n' "$reg_ids" | grep -qx "$id" \
    || err "registry row without file: MOCK-$id"
done
for id in $reg_ids; do
  printf '%s\n' $ids | grep -qx "$id" \
    || { echo "warning: registry row without mockup file: MOCK-$id" >&2; }
done

if [ "$errors" -gt 0 ]; then
  echo "mockup validation FAILED: $errors error(s)" >&2
  exit 1
fi
echo "mockup validation OK: $(printf '%s\n' $ids | grep -c .) review file(s)"
