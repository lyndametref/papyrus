#!/bin/sh
# Parse ADR Markdown, validate records, then render the skill's HTML template.
# Usage: generate_adr_index.sh [adr-dir [output.html]]

set -eu

base=${1:-docs/architecture/adr}
output=${2:-$base/index.html}
[ -d "$base" ] || { echo "error: ADR directory not found: $base" >&2; exit 1; }

fail() { echo "error: $*" >&2; exit 1; }

for f in "$base"/ADR-*.md; do
	[ -f "$f" ] || continue
	b=${f##*/}
	[ "$b" = "ADR-INDEX.md" ] && continue
	case $b in
		ADR-[0-9][0-9][0-9][0-9]-*.md) ;;
		*) fail "bad ADR filename: $b (expected ADR-NNNN-kebab-case-title.md)" ;;
	esac
done

set --
adr_count=0
for f in "$base"/ADR-[0-9][0-9][0-9][0-9]-*.md; do
	[ -f "$f" ] || continue
	set -- "$@" "$f"
	adr_count=$((adr_count + 1))
done
[ "$adr_count" -gt 0 ] || fail "no ADR files found in $base"

disk_ids=$(
	for f in "$base"/ADR-[0-9][0-9][0-9][0-9]-*.md; do
		[ -f "$f" ] || continue
		b=${f##*/}
		printf '%.8s\n' "$b"
	done | sort
)
dupes=$(printf '%s\n' "$disk_ids" | uniq -d)
[ -z "$dupes" ] || fail "duplicate ADR id(s): $(printf '%s' "$dupes" | tr '\n' ' ')"

script_dir=$(CDPATH= cd "$(dirname "$0")" && pwd)
template="$script_dir/../templates/index.html"
renderer="$script_dir/../../html-template-migration/scripts/render_html_template.sh"
[ -f "$template" ] || fail "ADR HTML template not found: $template"
[ -f "$renderer" ] || fail "shared HTML template renderer not found: $renderer"
page_title="Architecture Decision Records"
page_heading="Architecture Decision Records"
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
function trim(s) {
	sub(/^[ \t\r\n]+/, "", s)
	sub(/[ \t\r\n]+$/, "", s)
	return s
}
function reset_adr() { title = ""; status = ""; ns = 0; cur = "" }
function flush_adr(   b, id, st, i, secs, first, obj) {
	if (title == "") { print "missing title heading in " prevfile > "/dev/stderr"; exit 1 }
	b = prevfile
	sub(/^.*\//, "", b)
	if (!match(b, /^ADR-[0-9][0-9][0-9][0-9]-/)) { print "bad filename: " b > "/dev/stderr"; exit 1 }
	id = substr(b, 1, 8)
	st = trim(status)
	if (st == "") { print "missing Status section in " b > "/dev/stderr"; exit 1 }
	if (!match(st, /^(Proposed|Accepted|Refused|Deprecated|Superseded)([ \t]|$)/)) {
		print "invalid status in " b ": " st > "/dev/stderr"; exit 1
	}
	secs = ""; first = 1
	for (i = 1; i <= ns; i++) {
		secs = secs (first ? "" : ",") "{\"heading\":\"" jesc(trim(shead[i])) "\",\"content\":\"" jesc(trim(sbody[i])) "\"}"
		first = 0
	}
	obj = "{\"id\":\"" jesc(id) "\",\"title\":\"" jesc(title) "\",\"status\":\"" jesc(st) "\",\"sections\":[" secs "]}"
	nadr++
	adrs[nadr] = obj
}
function build_payload(   out, i) {
	out = "{\"adrs\":["
	for (i = 1; i <= nadr; i++) out = out (i > 1 ? "," : "") adrs[i]
	return out "]}"
}
BEGIN { nadr = 0; ns = 0; prevfile = ""; done = 0 }

FILENAME == "-" {
	if (prevfile != "") flush_adr()
	payload = build_payload()
	print payload
	done = 1
	next
}

{
	if (FILENAME != prevfile) {
		if (prevfile != "") { flush_adr(); reset_adr() }
		prevfile = FILENAME
	}
	if ($0 ~ /^#[ \t]/) { title = trim(substr($0, 2)); cur = ""; next }
	if ($0 ~ /^##[ \t]/) {
		h = trim(substr($0, 3))
		if (h == "Status") { cur = "Status"; next }
		ns++
		shead[ns] = h
		sbody[ns] = ""
		cur = "sec"
		next
	}
	if (cur == "Status") { status = status "\n" $0; next }
	if (cur == "sec") { sbody[ns] = sbody[ns] "\n" $0; next }
}
' "$@" - > "$data_tmp" <<'DATA_END'
__RENDER_DATA__
DATA_END

if grep -q '"id":""' "$data_tmp"; then fail "empty ADR id in parsed data"; fi
n_json_ids=$(grep -o '"id":"' "$data_tmp" | wc -l | tr -d ' ')
[ "$n_json_ids" -eq "$adr_count" ] || fail "expected $adr_count id entries, found $n_json_ids"
for id in $disk_ids; do grep -q "\"$id\"" "$data_tmp" || fail "id $id missing from parsed data"; done

"$renderer" "$template" "$output" "$data_tmp" "$page_title" "$page_heading"
if [ "$adr_count" -eq 1 ]; then noun="decision record"; else noun="decision records"; fi
echo "wrote $output: $adr_count $noun"
