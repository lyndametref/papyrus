#!/bin/sh
# Fill the standard typed slots in a standalone HTML template.
# Usage: render_html_template.sh TEMPLATE OUTPUT DATA_JSON PAGE_TITLE PAGE_HEADING

set -eu

if [ "$#" -ne 5 ]; then
	echo "usage: render_html_template.sh TEMPLATE OUTPUT DATA_JSON PAGE_TITLE PAGE_HEADING" >&2
	exit 2
fi

template=$1
output=$2
data_json=$3
page_title=$4
page_heading=$5
[ -f "$template" ] || { echo "error: HTML template not found: $template" >&2; exit 1; }
[ -f "$data_json" ] || { echo "error: JSON data file not found: $data_json" >&2; exit 1; }

case $output in
	*/*) output_dir=${output%/*}; output_name=${output##*/}; [ -n "$output_dir" ] || output_dir=/ ;;
	*) output_dir=.; output_name=$output ;;
esac
[ -n "$output_name" ] || { echo "error: output path must name a file" >&2; exit 1; }
mkdir -p "$output_dir"
temporary="$output_dir/.$output_name.tmp.$$"
[ ! -e "$temporary" ] || { echo "error: temporary output already exists: $temporary" >&2; exit 1; }
trap 'rm -f "$temporary"' EXIT HUP INT TERM

HTML_PAGE_TITLE=$page_title HTML_PAGE_HEADING=$page_heading JSON_DATA_PATH=$data_json \
awk '
function fail(message) {
	print "error: " message > "/dev/stderr"
	failed = 1
	exit 1
}
function html_escape(s,   out, i, c) {
	out = ""
	for (i = 1; i <= length(s); i++) {
		c = substr(s, i, 1)
		if (c == "&") out = out "&amp;"
		else if (c == "<") out = out "&lt;"
		else if (c == ">") out = out "&gt;"
		else if (c == "\"") out = out "&quot;"
		else if (c == "\047") out = out "&#39;"
		else out = out c
	}
	return out
}
function script_escape(s,   out, i, c) {
	out = ""
	for (i = 1; i <= length(s); i++) {
		c = substr(s, i, 1)
		if (c == "<") out = out "\\u003c"
		else if (c == ">") out = out "\\u003e"
		else if (c == "&") out = out "\\u0026"
		else out = out c
	}
	return out
}
function render_line(s,   result, token, value, position, length_token) {
	result = ""
	while (match(s, /\{\{[^}]*\}\}/)) {
		position = RSTART
		length_token = RLENGTH
		result = result substr(s, 1, position - 1)
		token = substr(s, position, length_token)
		if (token == "{{HTML:PAGE_TITLE}}") value = title
		else if (token == "{{HTML:PAGE_HEADING}}") value = heading
		else if (token == "{{JSON:DATA}}") value = json
		else fail("unrecognized placeholder: " token)
		slots[token]++
		result = result value
		s = substr(s, position + length_token)
	}
	return result s
}
BEGIN {
	data_path = ENVIRON["JSON_DATA_PATH"]
	if ((getline payload < data_path) <= 0) fail("JSON data file is empty or unreadable: " data_path)
	if ((getline extra < data_path) > 0) fail("JSON data must be a single line")
	close(data_path)
	if (payload !~ /^\{.*\}$/) fail("JSON data must be a JSON object on one line")
	title = html_escape(ENVIRON["HTML_PAGE_TITLE"])
	heading = html_escape(ENVIRON["HTML_PAGE_HEADING"])
	json = script_escape(payload)
}
{
	print render_line($0)
}
END {
	if (failed) exit 1
	if (!slots["{{HTML:PAGE_TITLE}}"] || !slots["{{HTML:PAGE_HEADING}}"] || !slots["{{JSON:DATA}}"])
		fail("template is missing one or more required placeholders")
}
' "$template" > "$temporary"

mv -f "$temporary" "$output"
trap - EXIT HUP INT TERM
