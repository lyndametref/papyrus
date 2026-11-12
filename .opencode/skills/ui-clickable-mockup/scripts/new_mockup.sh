#!/bin/sh
# Create a new single-file mockup review package skeleton.
# One file per ticket: review landing + one prototype per platform.
# Usage: new_mockup.sh [mockups-dir] [slug] [name] [platform...]
set -eu

dir=${1:-docs/specifications/ui/mockups}
slug=${2:-new-mockup}
name=${3:-New mockup}
shift 3 2>/dev/null || true
platforms="$*"
[ -n "$platforms" ] || platforms="web"

case "$slug" in
  *[!a-z0-9-]*) echo "error: slug must be kebab-case (a-z, 0-9, -): $slug" >&2; exit 1 ;;
esac

script_dir=$(CDPATH= cd "$(dirname "$0")" && pwd)
template="$script_dir/../templates/mockup.html"
[ -f "$template" ] || { echo "error: template not found: $template" >&2; exit 1; }

"$script_dir/ensure_wireframe_css.sh" "$dir" >/dev/null
id=$("$script_dir/next_mock_id.sh" "$dir")
out="$dir/$id-$slug.html"

chrome=""
cards=""
blocks=""

for platform in $platforms; do
  label=$(printf '%s' "$platform" | sed 's/^./\U&/')
  case "$platform" in
    android|ios) frame="wf-frame--mobile" ;;
    tablet|ipad) frame="wf-frame--tablet" ;;
    *) frame="wf-frame--desktop" ;;
  esac

  chrome="$chrome<a class=\"wf-chrome-btn\" href=\"#$platform\">$label</a>
  "
  cards="$cards<article class=\"wf-box wf-card\">
        <span class=\"wf-badge\">$label · 1 view</span>
        <h3>$label prototype</h3>
        <p>Click through the $platform view and interaction to test the usability of the current feature.</p>
        <a class=\"wf-button\" href=\"#$platform\">Open $platform prototype</a>
      </article>
      "
  blocks="$blocks<div class=\"wf-proto\" data-platform=\"$platform\">
  <nav class=\"wf-proto-nav\" aria-label=\"$label views (review navigation)\">
    <span class=\"wf-chrome-label\">$label views</span>
    <a class=\"wf-chrome-btn wf-view-link\" href=\"#$platform/main\">Main</a>
  </nav>
  <div class=\"wf-frame $frame\">
    <section class=\"wf-screen is-active\" data-platform=\"$platform\" data-screen=\"main\">
      <div class=\"wf-container\">
        <p class=\"wf-eyebrow\">$label · Main</p>
        <h2 class=\"wf-title\">Main view</h2>
        <!-- Draw only the current feature's happy path here. Add data-goto on elements that navigate. -->
      </div>
    </section>
  </div>
</div>
"
done

awk \
  -v id="$id" -v name="$name" \
  -v chrome="$chrome" -v cards="$cards" -v blocks="$blocks" '
  function esc(s) { gsub(/\\/, "\\\\", s); gsub(/&/, "\\&", s); return s }
  {
    if ($0 ~ /^[[:space:]]*\{\{PLATFORM_CARDS\}\}[[:space:]]*$/) { print cards; next }
    if ($0 ~ /^[[:space:]]*\{\{PLATFORM_BLOCKS\}\}[[:space:]]*$/) { print blocks; next }
    line = $0
    gsub(/\{\{MOCK_ID\}\}/, id, line)
    gsub(/\{\{MOCK_NAME\}\}/, esc(name), line)
    gsub(/\{\{TICKET_ID\}\}/, "TODO-TASK", line)
    gsub(/\{\{TICKET_TITLE\}\}/, "TODO ticket title", line)
    gsub(/\{\{SOURCES\}\}/, "TODO: REQ ids, specifications, ticket", line)
    gsub(/\{\{PLATFORM_CHROME\}\}/, esc(chrome), line)
    print line
  }
' "$template" > "$out"

echo "$out"
