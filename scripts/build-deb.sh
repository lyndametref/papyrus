#!/usr/bin/env bash
set -euo pipefail

# Debian assembly step for the Papyrus package.
#
# Usage:
#   scripts/build-deb.sh [--version <version>] [--out <dir>]
#
# Version precedence: --version, then PAPYRUS_DEB_VERSION, then the Cargo.toml
# package version. Output directory precedence: --out, then PAPYRUS_DEB_OUT,
# then the project-local .tmp/deb directory. The shared out policy in
# scripts/package-out.sh rejects unsafe paths before anything is created.
#
# Run directly with no version to derive the version from Cargo.toml and write
# the .deb to the project root, as before. The version and output options are
# the interface used by scripts/build-package.sh; they never edit a tracked
# file.

usage() {
    cat <<'EOF'
Usage: scripts/build-deb.sh [--version <version>] [--out <dir>]

Assemble the Papyrus Debian package from the release binary.

  --version <version>  Package version. Defaults to the Cargo.toml package
                       version (or PAPYRUS_DEB_VERSION when set).
  --out <dir>          Directory that receives the .deb. Defaults to the
                       project-local .tmp/deb directory (or PAPYRUS_DEB_OUT
                       when set).
  -h, --help           Print this help and exit.
EOF
}

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# shellcheck source=scripts/package-out.sh
source "${ROOT}/scripts/package-out.sh"

VERSION=""
OUT_DIR=""

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
OUT_DIR="$(package_validate_out "$OUT_DIR")" || exit $?
mkdir -p -- "$OUT_DIR"

ARCH="amd64"
PKG="papyrus_${VERSION}_${ARCH}"
STAGE="${ROOT}/target/deb/${PKG}"

# Ensure release binary exists.
if [[ ! -x "${ROOT}/target/release/papyrus" ]]; then
    cargo build --release
fi

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
