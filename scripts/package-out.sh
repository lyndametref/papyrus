#!/usr/bin/env bash
# Shared out-directory and version policy for the packaging scripts.
#
# Sourced by scripts/build-package.sh and scripts/build-deb.sh. Callers set
# ROOT to the absolute repository root before using the functions.

# System directories a package output may never target.
PACKAGE_SYSTEM_DIRS=(
    /bin /boot /dev /etc /lib /lib64 /proc /root /sbin /sys /usr /var
)

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
package_validate_out() {
    local out
    out="$(package_canonical_out "$1")" || return 2
    local root_real
    root_real="$(package_canonical_out "$ROOT")"
    local tmp_real
    tmp_real="$(package_canonical_out "$ROOT/.tmp")"

    if [[ "$out" == "/" ]]; then
        echo "error: refusing to use the filesystem root as --out" >&2
        return 2
    fi
    if [[ -n "${HOME:-}" ]]; then
        local home_real
        home_real="$(package_canonical_out "$HOME")"
        if [[ "$out" == "$home_real" ]]; then
            echo "error: refusing to use HOME as --out" >&2
            return 2
        fi
    fi
    local system
    for system in "${PACKAGE_SYSTEM_DIRS[@]}"; do
        if [[ "$out" == "$system" || "$out" == "$system/"* ]]; then
            echo "error: refusing to use the system directory $system as --out" >&2
            return 2
        fi
    done
    if [[ "$out" == "$root_real" ]]; then
        echo "error: --out must not be the repository root" >&2
        return 2
    fi
    if [[ "$root_real" == "$out"/* ]]; then
        echo "error: --out must not be a parent of the repository root" >&2
        return 2
    fi
    if [[ "$out" == "$root_real/"* && "$out" != "$tmp_real/"* ]]; then
        echo "error: --out inside the repository is allowed only under the project-local .tmp directory" >&2
        return 2
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
