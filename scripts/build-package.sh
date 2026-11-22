#!/usr/bin/env bash
set -euo pipefail

# Packaging entry point for the Papyrus deployable component.
#
# Usage:
#   scripts/build-package.sh --version <version> --out <dir> [--force]
#   scripts/build-package.sh --help
#
# Builds the release binary, delegates the Debian assembly to
# scripts/build-deb.sh, and writes the package only under --out.
# Diagnostics go to stderr; produced file paths go to stdout, one per line.

usage() {
    cat <<'EOF'
Usage: scripts/build-package.sh --version <version> --out <dir> [--force]
       scripts/build-package.sh --help

Build the Papyrus release binary and its Debian package.

  --version <version>  Package version, for example 0.2.0, 0.2.0-rc.1, or
                       0.0.0-test. Authoritative for the control Version:
                       field and the papyrus_<version>_amd64.deb name.
  --out <dir>          Directory that receives the deliverable. It must be
                       absent or empty; inside the repository only a path under
                       the project-local .tmp directory is accepted.
  --force              Remove the contents of a non-empty --out directory.
  -h, --help           Print this help and exit.

Requires cargo, rustc, and dpkg-deb on PATH.
EOF
}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
# shellcheck source=scripts/package-out.sh
source "${SCRIPT_DIR}/package-out.sh"

VERSION=""
OUT_DIR=""
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

if [[ -z "$VERSION" ]]; then
    echo "error: --version is required" >&2
    usage >&2
    exit 2
fi
if [[ -z "$OUT_DIR" ]]; then
    echo "error: --out is required" >&2
    usage >&2
    exit 2
fi
package_validate_version "$VERSION" || exit 2

for tool in cargo rustc dpkg-deb; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "error: required tool not found on PATH: $tool" >&2
        exit 1
    fi
done

# Validate the destination before creating or cleaning anything.
OUT_DIR="$(package_prepare_out "$OUT_DIR" "$FORCE")" || exit $?

echo "Building papyrus ${VERSION}..." >&2
cd "$ROOT"
cargo build --release --locked >&2

echo "Assembling the Debian package..." >&2
"${SCRIPT_DIR}/build-deb.sh" --version "$VERSION" --out "$OUT_DIR" >&2

ARTIFACT="${OUT_DIR}/papyrus_${VERSION}_amd64.deb"
if [[ ! -s "$ARTIFACT" ]]; then
    echo "error: expected artifact is missing or empty: $ARTIFACT" >&2
    exit 1
fi

printf '%s\n' "$ARTIFACT"
