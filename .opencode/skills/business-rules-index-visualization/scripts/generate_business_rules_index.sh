#!/bin/sh
# Parse business-rule Markdown, validate records, then render the HTML template.
# Usage: generate_business_rules_index.sh [business-rules-dir [output.html]]

set -eu

base=${1:-docs/business-rules}
output=${2:-$base/index.html}
[ -d "$base" ] || { echo "error: business-rules directory not found: $base" >&2; exit 1; }
fail() { echo "error: $*" >&2; exit 1; }

set --
rule_count=0
for f in "$base"/BR-*.md; do
	[ -f "$f" ] || continue
	b=${f##*/}
	case "$b" in BR-[0-9][0-9][0-9]-*.md) ;; *) fail "bad business-rule filename: $b (expected BR-XXX-short-name.md)" ;; esac
	slug=${b#BR-[0-9][0-9][0-9]-}; slug=${slug%.md}
	case "$slug" in ''|-*|*-|*--*|*[!a-z0-9-]*) fail "bad business-rule filename: $b (expected BR-XXX-short-name.md)" ;; esac
	set -- "$@" "$f"
	rule_count=$((rule_count + 1))
done
[ "$rule_count" -gt 0 ] || fail "no business-rule files found in $base"

disk_ids=$(for f in "$base"/BR-[0-9][0-9][0-9]-*.md; do [ -f "$f" ] || continue; b=${f##*/}; printf '%.6s\n' "$b"; done | sort)
dupes=$(printf '%s\n' "$disk_ids" | uniq -d)
[ -z "$dupes" ] || fail "duplicate BR ID(s): $(printf '%s' "$dupes" | tr '\n' ' ')"

script_dir=$(CDPATH= cd "$(dirname "$0")" && pwd)
template="$script_dir/../templates/index.html"
renderer="$script_dir/../../html-template-migration/scripts/render_html_template.sh"
[ -f "$template" ] || fail "business-rules HTML template not found: $template"
[ -f "$renderer" ] || fail "shared HTML template renderer not found: $renderer"
page_title="Business Rules"
page_heading="Business Rules"
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
function reset_rule() { title = ""; section = ""; description = ""; rationale = ""; applicability = "" }
function error(message) { print "error: " message > "/dev/stderr"; exit 1 }
function flush_rule(   b, id, name, obj) {
	b = basename(previous_file)
	if (title == "") error("missing title heading in " b)
	if (!match(title, /^BR-[0-9][0-9][0-9]:[ \t]+/)) error("bad title in " b " (expected # BR-XXX: Title)")
	id = substr(title, 1, 6)
	if (id != substr(b, 1, 6)) error("title ID does not match filename in " b)
	name = trim(substr(title, RLENGTH + 1))
	if (name == "") error("missing title in " b)
	if (trim(description) == "") error("missing Description section in " b)
	if (trim(rationale) == "") error("missing Rationale section in " b)
	if (trim(applicability) == "") error("missing Applicability section in " b)
	obj = "{\"id\":\"" jesc(id) "\",\"title\":\"" jesc(name) "\",\"description\":\"" jesc(trim(description)) "\",\"rationale\":\"" jesc(trim(rationale)) "\",\"applicability\":\"" jesc(trim(applicability)) "\"}"
	if (seen[id]++) error("duplicate BR ID " id)
	rules[++rule_total] = obj
}
function build_payload(   out, i) {
	out = "{\"rules\":["
	for (i = 1; i <= rule_total; i++) out = out (i > 1 ? "," : "") rules[i]
	return out "]}"
}
BEGIN { rule_total = 0; previous_file = "" }

FILENAME == "-" {
	if (previous_file != "") flush_rule()
	payload = build_payload()
	print payload
	next
}

{
	if (FILENAME != previous_file) {
		if (previous_file != "") flush_rule()
		previous_file = FILENAME
		reset_rule()
	}
	if ($0 ~ /^# /) { title = substr($0, 3); next }
	if ($0 ~ /^## /) { section = substr($0, 4); next }
	if (section == "Description") description = description "\n" $0
	else if (section == "Rationale") rationale = rationale "\n" $0
	else if (section == "Applicability") applicability = applicability "\n" $0
}
' "$@" - > "$data_tmp" <<'DATA_END'
__RENDER_DATA__
DATA_END

if grep -q '"id":""' "$data_tmp"; then fail "empty business-rule ID in parsed data"; fi
n_json_ids=$(grep -o '"id":"' "$data_tmp" | wc -l | tr -d ' ')
[ "$n_json_ids" -eq "$rule_count" ] || fail "expected $rule_count id entries, found $n_json_ids"
for id in $disk_ids; do grep -q "\"$id\"" "$data_tmp" || fail "id $id missing from parsed data"; done

"$renderer" "$template" "$output" "$data_tmp" "$page_title" "$page_heading"
echo "wrote $output: $rule_count business rules"
