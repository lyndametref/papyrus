#!/bin/sh
# Parse REQ/NFR Markdown, validate records, then render the HTML template.
# Usage:
#   generate_index.sh [requirements-or-corpus-dir [output.html]]
#
# If the given directory contains business/ or non-functional/ subdirectories,
# both corpus indexes are generated in their respective folders. Otherwise the
# directory itself is treated as one corpus (business or non-functional).

set -eu

base=${1:-docs/requirements}
output=${2:-}
[ -d "$base" ] || { echo "error: requirements directory not found: $base" >&2; exit 1; }
fail() { echo "error: $*" >&2; exit 1; }

base_abs=$(CDPATH= cd "$base" && pwd) || fail "requirements directory not found: $base"

# Root mode: generate one index per corpus subdirectory.
if [ -d "$base_abs/business" ] || [ -d "$base_abs/non-functional" ]; then
	if [ -d "$base_abs/business" ]; then
		if [ -n "$output" ]; then :; fi
		"$0" "$base_abs/business" "$base_abs/business/index.html"
	fi
	if [ -d "$base_abs/non-functional" ]; then
		"$0" "$base_abs/non-functional" "$base_abs/non-functional/index.html"
	fi
	exit 0
fi

output=${output:-$base_abs/index.html}

# Determine corpus kind from its contents (or directory name when empty).
has_req=0; has_nfr=0
for f in "$base_abs"/REQ-[0-9][0-9].md "$base_abs"/REQ-[0-9][0-9].[0-9][0-9][0-9].md; do [ -f "$f" ] && has_req=1; done
for f in "$base_abs"/NFR-[0-9][0-9].md "$base_abs"/NFR-[0-9][0-9].[0-9][0-9][0-9].md; do [ -f "$f" ] && has_nfr=1; done
[ "$has_req" -eq 0 ] || [ "$has_nfr" -eq 0 ] || fail "mixed REQ and NFR files in $base_abs"
if [ "$has_req" -eq 1 ]; then
	kind=business
	prefix=REQ
elif [ "$has_nfr" -eq 1 ]; then
	kind=nfr
	prefix=NFR
else
	case "$(basename "$base_abs")" in
		*non*|*nfr*|*NFR*) kind=nfr; prefix=NFR ;;
		*) kind=business; prefix=REQ ;;
	esac
fi

if [ "$kind" = "business" ]; then
	page_title="Business Requirements"
	page_heading="Business Requirements"
else
	page_title="Non-Functional Requirements"
	page_heading="Non-Functional Requirements"
fi

set --
group_count=0
leaf_count=0
for g in "$base_abs"/$prefix-[0-9][0-9].md; do
	[ -f "$g" ] || continue
	set -- "$@" "$g"
	group_count=$((group_count + 1))
	stem=${g##*/}; stem=${stem%.md}
	for a in "$base_abs/$stem".[0-9][0-9][0-9].md; do [ -f "$a" ] || continue; set -- "$@" "$a"; leaf_count=$((leaf_count + 1)); done
done

disk_ids=$(for f in "$base_abs"/$prefix-[0-9][0-9].[0-9][0-9][0-9].md; do [ -f "$f" ] || continue; b=${f##*/}; printf '%s\n' "${b%.md}"; done | sort)
script_dir=$(CDPATH= cd "$(dirname "$0")" && pwd)
template="$script_dir/../templates/index.html"
renderer="$script_dir/../../html-template-migration/scripts/render_html_template.sh"
[ -f "$template" ] || fail "requirements HTML template not found: $template"
[ -f "$renderer" ] || fail "shared HTML template renderer not found: $renderer"

data_tmp="$base_abs/.index.data.$$"
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
function reset_body() { title = ""; sec = ""; story = ""; crit = ""; desc = ""; rati = ""; veri = ""; ac = "" }
function flush_file(   id, rest, gid, i, obj) {
	if (title == "") return
	if (!match(title, /^(REQ|NFR)-[0-9][0-9](\.[0-9][0-9][0-9])?([ \t]+|$)/)) { print "bad title: " title > "/dev/stderr"; exit 1 }
	id = trim(substr(title, 1, RLENGTH))
	rest = trim(substr(title, RLENGTH + 1))
	if (index(id, ".")) {
		gid = substr(id, 1, 6)
		i = gi[gid]
		if (i == 0) { print "group " gid " not seen before " id > "/dev/stderr"; exit 1 }
		if (id ~ /^REQ-/) {
			obj = "{\"id\":\"" jesc(id) "\",\"title\":\"" jesc(rest) "\",\"criticality\":\"" jesc(trim(crit)) "\",\"userStory\":\"" jesc(trim(story)) "\",\"description\":\"" jesc(trim(desc)) "\",\"criteria\":\"" jesc(trim(ac)) "\"}"
		} else {
			obj = "{\"id\":\"" jesc(id) "\",\"title\":\"" jesc(rest) "\",\"description\":\"" jesc(trim(desc)) "\",\"rationale\":\"" jesc(trim(rati)) "\",\"verification\":\"" jesc(trim(veri)) "\"}"
		}
		gchild[i] = gchild[i] (gchild[i] == "" ? "" : ",") obj
		nleaf++
	} else {
		if (gi[id]) { print "duplicate group ID " id > "/dev/stderr"; exit 1 }
		gi[id] = ++ngroup; gids[ngroup] = id; gtitle[ngroup] = rest
		gstory[ngroup] = trim(story); gdesc[ngroup] = trim(desc); gchild[ngroup] = ""
	}
	title = ""
}
function group_json(i) {
	return "{\"id\":\"" jesc(gids[i]) "\",\"title\":\"" jesc(gtitle[i]) "\",\"userStory\":\"" jesc(gstory[i]) "\",\"description\":\"" jesc(gdesc[i]) "\",\"children\":[" gchild[i] "]}"
}
function build_payload(   out, i, first) {
	out = "{\"groups\":["
	first = 1
	for (i = 1; i <= ngroup; i++) { out = out (first ? "" : ",") group_json(i); first = 0 }
	return out "]}"
}
BEGIN { ngroup = 0; nleaf = 0; prevfile = ""; done = 0 }

FILENAME == "-" {
	if (prevfile != "") flush_file()
	payload = build_payload()
	print payload
	done = 1
	next
}

{
	if (FILENAME != prevfile) { if (prevfile != "") { flush_file(); reset_body() }; prevfile = FILENAME }
	if ($0 ~ /^# /) { title = substr($0, 3); next }
	if ($0 ~ /^## /) { sec = substr($0, 4); next }
	if (sec == "Group") next
	if (sec == "Acceptance Criteria") { if ($0 ~ /^- /) ac = ac "\n" substr($0, 3); else ac = ac "\n" $0; next }
	if (sec == "User Story") story = story "\n" $0
	else if (sec == "Criticality") crit = crit "\n" $0
	else if (sec == "Description") desc = desc "\n" $0
	else if (sec == "Rationale") rati = rati "\n" $0
	else if (sec == "Verification Method") veri = veri "\n" $0
}
' "$@" - > "$data_tmp" <<'DATA_END'
__RENDER_DATA__
DATA_END

n_json_ids=$(grep -o '"id":"' "$data_tmp" | wc -l | tr -d ' ')
expected_ids=$((group_count + leaf_count))
[ "$n_json_ids" -eq "$expected_ids" ] || fail "expected $expected_ids id entries, found $n_json_ids"
for id in $disk_ids; do grep -q "\"$id\"" "$data_tmp" || fail "id $id missing from parsed data"; done

"$renderer" "$template" "$output" "$data_tmp" "$page_title" "$page_heading"
echo "wrote $output: $leaf_count requirements ($group_count groups)"
