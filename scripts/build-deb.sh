#!/usr/bin/env bash
set -euo pipefail

# Debian assembly step for the Papyrus package.
#
# Usage:
#   scripts/build-deb.sh [--version <version>] [--out <dir>] [--force]
#   scripts/build-deb.sh --check-assets-only
#
# Every assembly run rebuilds the release binary with
# `cargo build --release --locked` (AUD-153 L-4), so a direct run can never
# package a stale binary, and it applies the shared output-emptiness policy:
# the --out directory must be absent or empty unless --force removes its
# contents (AUD-153 L-4).
#
# Version precedence: --version, then PAPYRUS_DEB_VERSION, then the Cargo.toml
# package version. Output directory precedence: --out, then PAPYRUS_DEB_OUT,
# then the project-local .tmp/deb directory. The shared out policy in
# scripts/package-out.sh rejects unsafe paths before anything is created: an
# in-repository --out must be strictly below .tmp/ or strictly below
# builds/<segment>/...; the repository root, builds/ itself, every other
# in-repository path, the filesystem root, system directories, $HOME and its
# subtree, and any parent of the repository root are refused with exit 2.
#
# The version and output options are the interface used by
# scripts/build-package.sh; they never edit a tracked file.

usage() {
    cat <<'EOF'
Usage: scripts/build-deb.sh [--version <version>] [--out <dir>] [--force]

Assemble the Papyrus Debian package from a fresh release binary.

  --version <version>  Package version. Defaults to the Cargo.toml package
                       version (or PAPYRUS_DEB_VERSION when set).
  --out <dir>          Directory that receives the .deb. Defaults to the
                       project-local .tmp/deb directory (or PAPYRUS_DEB_OUT
                       when set). It must be absent or empty.
  --force              Remove the contents of a non-empty --out directory.
  --check-assets-only  Verify vendor asset versions and exit without building.
  -h, --help           Print this help and exit.

Every assembly rebuilds with `cargo build --release --locked`, so a stale
target/release/papyrus is never packaged.

--out policy (ADR-043 decision 7, amended 2026-11-23):
  permitted  <repo>/.tmp/<path>            scratch, removed before the work ends
  permitted  <repo>/builds/<segment>/...   publish location, git-ignored
  refused    <repo>                        the repository root
  refused    <repo>/builds                 builds/ with no segment after it
  refused    any other path inside <repo>  exit code 2
  refused    /, system directories, $HOME and its subtree, and any parent of
             the repository root                     exit code 2
EOF
}

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

VERSION=""
OUT_DIR=""
CHECK_ASSETS_ONLY=0
FORCE="false"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --version)
            [[ $# -ge 2 ]] || { echo "error: --version requires a value" >&2; exit 2; }
            VERSION="$2"
            shift 2
            ;;
        --out)
            [[ $# -ge 2 ]] || { echo "error: --out requires a value" >&2; exit 2; }
            OUT_DIR="$2"
            shift 2
            ;;
        --force)
            FORCE="true"
            shift
            ;;
        --check-assets-only)
            CHECK_ASSETS_ONLY=1
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "error: unknown argument: $1" >&2
            usage >&2
            exit 2
            ;;
    esac
done

cd "$ROOT"

# Verify the packaged asset versions before assembling anything. The asset
# version is the vendor directory name, the manifest `version` column, and the
# version each copyright section cites; it is independent of the Papyrus
# package version (ADR-038 decisions 29 and 30).
check_asset_versions() {
    local vendor_root="src/browser/vendor"
    local manifest="${vendor_root}/MANIFEST.md"
    local copyright="packaging/deb/usr/share/doc/papyrus/copyright"
    if [[ ! -d "$vendor_root" ]]; then
        echo "error: vendor asset tree not found: ${vendor_root}" >&2
        return 1
    fi
    if [[ ! -f "$manifest" ]]; then
        echo "error: asset manifest not found: ${manifest}" >&2
        return 1
    fi
    if [[ ! -f "$copyright" ]]; then
        echo "error: copyright file not found: ${copyright}" >&2
        return 1
    fi

    local failed=0
    local package_dir package
    for package_dir in "$vendor_root"/*/; do
        [[ -d "$package_dir" ]] || continue
        package="$(basename "$package_dir")"

        local -a versions=()
        local entry
        for entry in "$package_dir"*/; do
            [[ -d "$entry" ]] || continue
            versions+=("$(basename "$entry")")
        done
        if (( ${#versions[@]} == 0 )); then
            echo "error: vendor package '$package' has no version directory under ${package_dir}" >&2
            failed=1
            continue
        fi
        if (( ${#versions[@]} > 1 )); then
            echo "error: vendor package '$package' has more than one version directory: ${versions[*]}" >&2
            failed=1
            continue
        fi
        local version="${versions[0]}"

        local -a row_versions=()
        local row_version
        while IFS= read -r row_version; do
            if [[ -n "$row_version" ]]; then
                row_versions+=("$row_version")
            fi
        done < <(awk -F'|' -v prefix="/assets/${package}/" '
            /^[[:space:]]*\|/ {
                path = $2
                gsub(/^[[:space:]]+|[[:space:]]+$/, "", path)
                gsub(/`/, "", path)
                if (index(path, prefix) == 1) {
                    value = $4
                    gsub(/^[[:space:]]+|[[:space:]]+$/, "", value)
                    print value
                }
            }' "$manifest")
        if (( ${#row_versions[@]} == 0 )); then
            echo "error: manifest has no version row for vendor package '$package'" >&2
            failed=1
        else
            for row_version in "${row_versions[@]}"; do
                if [[ "$row_version" != "$version" ]]; then
                    echo "error: manifest version '$row_version' for vendor package '$package' does not match directory version '$version'" >&2
                    failed=1
                fi
            done
        fi

        local -a cited_versions=()
        local cited_version
        while IFS= read -r cited_version; do
            if [[ -n "$cited_version" ]]; then
                cited_versions+=("$cited_version")
            fi
        done < <(grep -oE "src/browser/vendor/${package}/[^/[:space:]]+" "$copyright" 2>/dev/null | sed -E "s#.*/${package}/([^/]+).*#\1#" | sort -u || true)
        if (( ${#cited_versions[@]} == 0 )); then
            echo "error: copyright cites no version for vendor package '$package'" >&2
            failed=1
        else
            for cited_version in "${cited_versions[@]}"; do
                if [[ "$cited_version" != "$version" ]]; then
                    echo "error: copyright version '$cited_version' for vendor package '$package' does not match directory version '$version'" >&2
                    failed=1
                fi
            done
        fi
    done

    if (( failed != 0 )); then
        echo "error: asset version check failed; refusing to build the package" >&2
        return 1
    fi
    return 0
}

# The guard runs on every assembly path, not only with the opt-in flag
# (AUD-148 F2), so a normal package build rejects any version mismatch.
check_asset_versions || exit 1

if (( CHECK_ASSETS_ONLY )); then
    exit 0
fi

# shellcheck source=scripts/package-out.sh
source "${ROOT}/scripts/package-out.sh"

if [[ -z "$VERSION" ]]; then
    VERSION="${PAPYRUS_DEB_VERSION:-}"
fi
if [[ -z "$VERSION" ]]; then
    VERSION="$(awk -F '"' '/^version[[:space:]]*=/{print $2; exit}' Cargo.toml)"
fi
if [[ -z "$VERSION" ]]; then
    echo "error: cannot determine the package version from Cargo.toml" >&2
    exit 1
fi
package_validate_version "$VERSION" || exit 2

if [[ -z "$OUT_DIR" ]]; then
    OUT_DIR="${PAPYRUS_DEB_OUT:-$ROOT/.tmp/deb}"
fi
# AUD-153 L-4: a direct run obeys the same output-emptiness policy as the
# packaging entry point; an existing non-empty directory needs --force.
OUT_DIR="$(package_prepare_out "$OUT_DIR" "$FORCE")" || exit $?

ARCH="amd64"
PKG="papyrus_${VERSION}_${ARCH}"
STAGE="${ROOT}/target/deb/${PKG}"

# AUD-153 L-4: rebuild before staging on every assembly path. A stale
# target/release/papyrus is never packaged, and the locked build keeps the
# dependency set fixed.
cargo build --release --locked >&2

rm -rf "$STAGE"
mkdir -p "${STAGE}/DEBIAN" "${STAGE}/usr/bin" "${STAGE}/usr/share/doc/papyrus"

cp "${ROOT}/target/release/papyrus" "${STAGE}/usr/bin/papyrus"
chmod 0755 "${STAGE}/usr/bin/papyrus"

# Render control file with the effective package version.
sed "s/^Version: .*/Version: ${VERSION}/" \
    "${ROOT}/packaging/deb/DEBIAN/control" > "${STAGE}/DEBIAN/control"
chmod 0644 "${STAGE}/DEBIAN/control"

cp "${ROOT}/packaging/deb/usr/share/doc/papyrus/copyright" \
    "${STAGE}/usr/share/doc/papyrus/copyright"
chmod 0644 "${STAGE}/usr/share/doc/papyrus/copyright"

dpkg-deb --build --root-owner-group "$STAGE" "${OUT_DIR}/${PKG}.deb"

echo "Built ${OUT_DIR}/${PKG}.deb"
