#!/usr/bin/env bash
# Shared out-directory and version policy for the packaging scripts.
#
# Usage: sourced by scripts/build-package.sh and scripts/build-deb.sh. Callers
# set ROOT to the absolute repository root before using the functions.
#
# `--out` policy (ADR-043 decision 7, amended 2026-11-23; the guard runs on the
# canonicalised path):
#
#   Permitted  <repo>/.tmp/<path>                 scratch; removed before the
#                                                 work ends
#              <repo>/builds/<segment>/...        publish location; git-ignored,
#                                                 never staged or committed
#   Refused    <repo>                             the repository root
#              <repo>/builds                      builds/ with no segment after it
#              any other path inside <repo>       with exit 2
#              /                                  the filesystem root
#              /bin /boot /dev /etc /lib ...      system directories
#              $HOME and its subtree              except the two permitted roots
#              any parent of <repo>               with exit 2
#
# Diagnostics go to stderr and every refusal exits 2.

# System directories a package output may never target.
PACKAGE_SYSTEM_DIRS=(
    /bin /boot /dev /etc /lib /lib64 /proc /root /sbin /sys /usr /var
)

# The permitted in-repository roots, relative to the repository root, with the
# rule each one follows.
package_out_policy() {
    cat <<'EOF'
In-repository --out policy (ADR-043 decision 7, amended 2026-11-23):
  permitted  <repo>/.tmp/<path>            scratch, removed before the work ends
  permitted  <repo>/builds/<segment>/...   publish location, git-ignored
  refused    <repo>                        the repository root
  refused    <repo>/builds                 builds/ with no segment after it
  refused    any other path inside <repo>  exit code 2
Outside the repository the filesystem root, system directories, $HOME and its
subtree, the repository root, and any parent of the repository root are refused
with exit code 2.
EOF
}

# Validate a version string as a package version. The value must have three
# numeric components and may carry one build/metadata suffix, for example
# 1.2.3, 1.2.3-rc.1, or 1.2.3+build.7.
package_validate_version() {
    local version="$1"
    if [[ ! "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+([-+~][0-9A-Za-z][0-9A-Za-z.+~-]*)?$ ]]; then
        echo "error: invalid version: $version (expected e.g. 1.2.3 or 1.2.3-rc.1)" >&2
        return 2
    fi
}

# Canonicalise an output path without creating it.
package_canonical_out() {
    realpath -m -- "$1"
}

# Refuse unsafe output paths before any directory is created.
#
# Returns the canonical path on stdout when accepted; every refusal writes one
# diagnostic naming the rule to stderr and returns 2.
package_validate_out() {
    local out
    out="$(package_canonical_out "$1")" || return 2
    local root_real
    root_real="$(package_canonical_out "$ROOT")"
    local tmp_real
    tmp_real="$(package_canonical_out "$ROOT/.tmp")"
    local builds_real
    builds_real="$(package_canonical_out "$ROOT/builds")"

    if [[ "$out" == "/" ]]; then
        echo "error: refusing to use the filesystem root as --out" >&2
        return 2
    fi
    local system
    for system in "${PACKAGE_SYSTEM_DIRS[@]}"; do
        if [[ "$out" == "$system" || "$out" == "$system/"* ]]; then
            echo "error: refusing to use the system directory $system as --out" >&2
            return 2
        fi
    done
    if [[ "$out" == "$root_real" ]]; then
        echo "error: --out must not be the repository root ($root_real); permitted in-repository roots are .tmp/<path> and builds/<segment>/..." >&2
        return 2
    fi
    if [[ "$root_real" == "$out"/* ]]; then
        echo "error: --out must not be a parent of the repository root" >&2
        return 2
    fi

    # In-repository: only the scratch and publish roots are permitted, each
    # strictly below its root so the root itself is refused.
    local permitted="false"
    if [[ "$out" == "$tmp_real/"* ]]; then
        permitted="true"
    elif [[ "$out" == "$builds_real/"* && -n "${out#"$builds_real/"}" ]]; then
        permitted="true"
    fi
    if [[ "$out" == "$root_real/"* && "$permitted" != "true" ]]; then
        echo "error: refusing --out inside the repository: $out" >&2
        echo "error: permitted in-repository roots are $root_real/.tmp/<path> (scratch) and $root_real/builds/<segment>/... (publish); $root_real and $root_real/builds itself are refused" >&2
        return 2
    fi

    if [[ -n "${HOME:-}" ]]; then
        local home_real
        home_real="$(package_canonical_out "$HOME")"
        # Refuse HOME and its whole subtree, but keep both permitted
        # in-repository roots usable even when the repository lives under HOME.
        if [[ ( "$out" == "$home_real" || "$out" == "$home_real/"* ) \
            && "$out" != "$tmp_real" && "$out" != "$tmp_real/"* \
            && "$out" != "$builds_real" && "$out" != "$builds_real/"* ]]; then
            echo "error: refusing to use HOME or a subdirectory of HOME as --out" >&2
            return 2
        fi
    fi
    printf '%s\n' "$out"
}

# Validate an output path and prepare it: it must be absent, empty, or the
# caller must pass --force, in which case its contents are removed.
package_prepare_out() {
    local requested="$1"
    local force="${2:-false}"
    local out
    out="$(package_validate_out "$requested")" || return $?

    if [[ -e "$out" ]]; then
        if [[ ! -d "$out" ]]; then
            echo "error: --out exists and is not a directory: $out" >&2
            return 2
        fi
        if [[ -n "$(find "$out" -mindepth 1 -maxdepth 1 -print -quit)" ]]; then
            if [[ "$force" != "true" ]]; then
                echo "error: --out is not empty: $out (pass --force to replace its contents)" >&2
                return 2
            fi
            find "$out" -mindepth 1 -maxdepth 1 -exec rm -rf -- {} +
        fi
    else
        mkdir -p -- "$out"
    fi
    printf '%s\n' "$out"
}
