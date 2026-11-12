#!/bin/sh
# Parse business-glossary Markdown, validate terms, then render the HTML template.
# Usage: generate_business_glossary_index.sh [glossary-dir [output.html]]

set -eu

base=${1:-docs/domain-model/business-glossary}
output=${2:-$base/index.html}
[ -d "$base" ] || { echo "error: business glossary directory not found: $base" >&2; exit 1; }
fail() { echo "error: $*" >&2; exit 1; }

set --
term_count=0
for f in "$base"/*.md; do
	[ -f "$f" ] || continue
	b=${f##*/}
	slug=${b%.md}
	case "$slug" in ''|-*|*-|*--*|*[!a-z0-9-]*) fail "bad glossary filename: $b (expected <kebab-case-term>.md)" ;; esac
	set -- "$@" "$f"
	term_count=$((term_count + 1))
done
[ "$term_count" -gt 0 ] || fail "no glossary term files found in $base"

disk_ids=$(for f in "$base"/*.md; do [ -f "$f" ] || continue; b=${f##*/}; printf '%s\n' "${b%.md}"; done | sort)
dupes=$(printf '%s\n' "$disk_ids" | uniq -d)
[ -z "$dupes" ] || fail "duplicate glossary term slug(s): $(printf '%s' "$dupes" | tr '\n' ' ')"

script_dir=$(CDPATH= cd "$(dirname "$0")" && pwd)
template="$script_dir/../templates/index.html"
renderer="$script_dir/../../html-template-migration/scripts/render_html_template.sh"
[ -f "$template" ] || fail "business-glossary HTML template not found: $template"
[ -f "$renderer" ] || fail "shared HTML template renderer not found: $renderer"
page_title="Business Glossary"
page_heading="Business Glossary"
data_tmp="$base/.index.data.$$"
trap 'rm -f "$data_tmp"' EXIT HUP INT TERM

awk '
function jesc(s,   out, i, c, n) {
	out = ""; n = length(s)
	for (i = 1; i <= n; i++) {
		c = substr(s, i, 1)
		if (c == "\\" || c == "\"") out = out "\\" c
		else if (c == "\n") out = out "\\n"
		else if (c == "\r") out = out "\\r"
		else if (c == "\t") out = out "\\t"
		else out = out c
	}
	return out
}
function trim(s) { sub(/^[ \t\r\n]+/, "", s); sub(/[ \t\r\n]+$/, "", s); return s }
function basename(s) { sub(/^.*\//, "", s); return s }
function reset_term() { title = ""; section = ""; definition = ""; aliases = "" }
function error(message) { print "error: " message > "/dev/stderr"; exit 1 }
function flush_term(   b, slug, name, obj) {
	b = basename(previous_file)
	if (title == "") error("missing title heading in " b)
	name = trim(title)
	if (name == "") error("missing term title in " b)
	if (trim(definition) == "") error("missing Definition section in " b)
	slug = substr(b, 1, length(b) - 3)
	obj = "{\"id\":\"" jesc(slug) "\",\"term\":\"" jesc(name) "\",\"definition\":\"" jesc(trim(definition)) "\",\"aliases\":\"" jesc(trim(aliases)) "\"}"
	if (seen[slug]++) error("duplicate glossary term " slug)
	terms[++term_total] = obj
}
function build_payload(   out, i) {
	out = "{\"terms\":["
	for (i = 1; i <= term_total; i++) out = out (i > 1 ? "," : "") terms[i]
	return out "]}"
}
BEGIN { term_total = 0; previous_file = "" }

FILENAME == "-" {
	if (previous_file != "") flush_term()
	payload = build_payload()
	print payload
	next
}

{
	if (FILENAME != previous_file) {
		if (previous_file != "") flush_term()
		previous_file = FILENAME
		reset_term()
	}
	if ($0 ~ /^# /) { title = substr($0, 3); next }
	if ($0 ~ /^## /) { section = substr($0, 4); next }
	if (section == "Definition") definition = definition "\n" $0
	else if (section == "Aliases") aliases = aliases "\n" $0
}
' "$@" - > "$data_tmp" <<'DATA_END'
__RENDER_DATA__
DATA_END

if grep -q '"id":""' "$data_tmp"; then fail "empty glossary term slug in parsed data"; fi
n_json_ids=$(grep -o '"id":"' "$data_tmp" | wc -l | tr -d ' ')
[ "$n_json_ids" -eq "$term_count" ] || fail "expected $term_count id entries, found $n_json_ids"
for id in $disk_ids; do grep -q "\"$id\"" "$data_tmp" || fail "id $id missing from parsed data"; done

"$renderer" "$template" "$output" "$data_tmp" "$page_title" "$page_heading"
echo "wrote $output: $term_count business glossary terms"
