#!/bin/sh
# Create missing documentation indexes without overwriting existing files.
# Usage: initialize_docs_indexes.sh PROJECT_ROOT

set -eu

if [ "$#" -ne 1 ]; then
	echo "usage: initialize_docs_indexes.sh PROJECT_ROOT" >&2
	exit 2
fi

project_root=$(CDPATH= cd "$1" && pwd) || { echo "error: project root not found: $1" >&2; exit 1; }
docs="$project_root/docs"
adr_dir="$docs/architecture/adr"
requirements_dir="$docs/requirements"
rules_dir="$docs/business-rules"
glossary_dir="$docs/domain-model/business-glossary"

fail() { echo "error: $*" >&2; exit 1; }

for directory in "$docs" "$docs/architecture" "$adr_dir" "$requirements_dir" \
	"$requirements_dir/business" "$requirements_dir/non-functional" \
	"$rules_dir" "$docs/domain-model" "$glossary_dir"; do
	[ ! -L "$directory" ] || fail "unexpected symlink in documentation path: $directory"
	[ -d "$directory" ] || fail "required documentation directory is missing: $directory (approve/create the base folder structure first)"
done

validate_index_path() {
	path=$1
	[ ! -L "$path" ] || fail "index path is a symlink: $path"
	if [ -e "$path" ] && [ ! -f "$path" ]; then fail "index path exists but is not a regular file: $path"; fi
}

for index_path in \
	"$docs/index.html" \
	"$adr_dir/index.html" \
	"$requirements_dir/business/index.html" \
	"$requirements_dir/non-functional/index.html" \
	"$rules_dir/index.html" \
	"$glossary_dir/index.html"; do
	validate_index_path "$index_path"
done

script_dir=$(CDPATH= cd "$(dirname "$0")" && pwd)
skill_root=$(CDPATH= cd "$script_dir/.." && pwd)
renderer="$skill_root/../html-template-migration/scripts/render_html_template.sh"
adr_generator="$skill_root/../adr-index-visualization/scripts/generate_adr_index.sh"
requirements_generator="$skill_root/../requirements-index-visualization/scripts/generate_index.sh"
rules_generator="$skill_root/../business-rules-index-visualization/scripts/generate_business_rules_index.sh"
glossary_generator="$skill_root/../business-glossary-index-visualization/scripts/generate_business_glossary_index.sh"
docs_template="$skill_root/templates/docs-index.html"

for file in "$renderer" "$adr_generator" "$requirements_generator" "$rules_generator" "$glossary_generator" "$docs_template"; do
	[ -f "$file" ] || fail "required template or generator is missing: $file"
done

data_tmp="$docs/.initialize-index-data.$$"
trap 'rm -f "$data_tmp"' EXIT HUP INT TERM

index_exists() {
	path=$1
	if [ -L "$path" ]; then fail "index path is a symlink: $path"; fi
	if [ -e "$path" ]; then
		[ -f "$path" ] || fail "index path exists but is not a regular file: $path"
		echo "preserved existing $path"
		return 0
	fi
	return 1
}

render_empty() {
	target=$1
	template=$2
	payload=$3
	title=$4
	heading=$5
	printf '%s\n' "$payload" > "$data_tmp"
	"$renderer" "$template" "$target" "$data_tmp" "$title" "$heading"
	echo "created empty $target"
}

create_index() {
	target=$1
	template=$2
	payload=$3
	title=$4
	heading=$5
	if index_exists "$target"; then return; fi
	render_empty "$target" "$template" "$payload" "$title" "$heading"
}

create_corpus_index() {
	target=$1
	template=$2
	payload=$3
	title=$4
	heading=$5
	source_dir=$6
	has_source=$7
	generator=$8
	if index_exists "$target"; then return; fi
	if [ "$has_source" -eq 1 ]; then
		"$generator" "$source_dir" "$target"
	else
		render_empty "$target" "$template" "$payload" "$title" "$heading"
	fi
}

has_adr=0
for file in "$adr_dir"/ADR-*.md; do
	[ -f "$file" ] || continue
	[ "${file##*/}" = "ADR-INDEX.md" ] && continue
	has_adr=1
done

has_business=0
for file in "$requirements_dir"/business/REQ-*.md; do [ -f "$file" ] || continue; has_business=1; done

has_nfr=0
for file in "$requirements_dir"/non-functional/NFR-*.md; do [ -f "$file" ] || continue; has_nfr=1; done

has_rules=0
for file in "$rules_dir"/BR-*.md; do [ -f "$file" ] || continue; has_rules=1; done

has_glossary=0
for file in "$glossary_dir"/*.md; do [ -f "$file" ] || continue; has_glossary=1; done

create_corpus_index \
	"$adr_dir/index.html" \
	"$skill_root/../adr-index-visualization/templates/index.html" \
	'{"adrs":[]}' \
	"Architecture Decision Records" "Architecture Decision Records" \
	"$adr_dir" "$has_adr" "$adr_generator"

create_corpus_index \
	"$requirements_dir/business/index.html" \
	"$skill_root/../requirements-index-visualization/templates/index.html" \
	'{"groups":[]}' \
	"Business Requirements" "Business Requirements" \
	"$requirements_dir/business" "$has_business" "$requirements_generator"

create_corpus_index \
	"$requirements_dir/non-functional/index.html" \
	"$skill_root/../requirements-index-visualization/templates/index.html" \
	'{"groups":[]}' \
	"Non-Functional Requirements" "Non-Functional Requirements" \
	"$requirements_dir/non-functional" "$has_nfr" "$requirements_generator"

create_corpus_index \
	"$rules_dir/index.html" \
	"$skill_root/../business-rules-index-visualization/templates/index.html" \
	'{"rules":[]}' \
	"Business Rules" "Business Rules" \
	"$rules_dir" "$has_rules" "$rules_generator"

create_corpus_index \
	"$glossary_dir/index.html" \
	"$skill_root/../business-glossary-index-visualization/templates/index.html" \
	'{"terms":[]}' \
	"Business Glossary" "Business Glossary" \
	"$glossary_dir" "$has_glossary" "$glossary_generator"

create_index \
	"$docs/index.html" "$docs_template" \
	'{"indexes":[{"title":"Architecture Decision Records","href":"architecture/adr/index.html","description":"Architecture decisions, status, and rationale."},{"title":"Business Requirements","href":"requirements/business/index.html","description":"Business requirements."},{"title":"Non-Functional Requirements","href":"requirements/non-functional/index.html","description":"Non-functional requirements."},{"title":"Business Rules","href":"business-rules/index.html","description":"Validated business rules and applicability."},{"title":"Business Glossary","href":"domain-model/business-glossary/index.html","description":"Canonical business terms and definitions."}]}' \
	"Project Documentation" "Project Documentation"
