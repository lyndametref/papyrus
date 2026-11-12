#!/bin/sh
# Print the next free MOCK-XXXX id based on existing MOCK-*.html files.
# Usage: next_mock_id.sh [mockups-dir]
set -eu
dir=${1:-docs/specifications/ui/mockups}
mkdir -p "$dir"
max=0
for f in "$dir"/MOCK-*.*; do
  [ -e "$f" ] || continue
  b=${f##*/}
  case "$b" in
    MOCK-[0-9][0-9][0-9][0-9]-*) ;;
    *) continue ;;
  esac
  # extract the four digits after MOCK-
  n=$(printf '%s' "$b" | sed -n 's/^MOCK-\([0-9][0-9][0-9][0-9]\)-.*/\1/p')
  [ -z "$n" ] && continue
  n=$(printf '%s' "$n" | sed 's/^0*//')
  [ -z "$n" ] && n=0
  if [ "$n" -gt "$max" ]; then max=$n; fi
done
printf 'MOCK-%04d\n' $((max + 1))
