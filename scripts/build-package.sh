#!/usr/bin/env bash
set -euo pipefail

# Packaging entry point for the Papyrus deployable component.
#
# Usage:
#   scripts/build-package.sh --version <version> --out <dir>
#   scripts/build-package.sh --help
#
# Builds the release binary, delegates the Debian assembly to
# scripts/build-deb.sh, and writes the package only under --out.
# Diagnostics go to stderr; produced file paths go to stdout, one per line.

usage() {
    cat <<'EOF'
Usage: scripts/build-package.sh --version <version> --out <dir>
       scripts/build-package.sh --help

Build the Papyrus release binary and its Debian package.

  --version <version>  Package version, for example 0.2.0, 0.2.0-rc.1, or
                       0.0.0-test. Authoritative for the control Version:
                       field and the papyrus_<version>_amd64.deb name.
  --out <dir>          Directory that receives the deliverable. Its contents
                       are removed before writing, so reruns are idempotent.
  -h, --help           Print this help and exit.

Requires cargo, rustc, and dpkg-deb on PATH.
EOF
}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

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
if [[ ! "$VERSION" =~ ^[0-9][0-9A-Za-z.+~-]*$ ]]; then
    echo "error: invalid version: $VERSION (expected e.g. 1.2.3 or 1.2.3-rc.1)" >&2
    exit 2
fi

for tool in cargo rustc dpkg-deb; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "error: required tool not found on PATH: $tool" >&2
        exit 1
    fi
done

mkdir -p "$OUT_DIR"
OUT_DIR="$(cd "$OUT_DIR" && pwd)"

if [[ "$OUT_DIR" == "/" ]]; then
    echo "error: refusing to use the filesystem root as --out" >&2
    exit 2
fi
if [[ "$OUT_DIR" == "$ROOT" ]]; then
    echo "error: --out must not be the repository root" >&2
    exit 2
fi
if [[ "$ROOT" == "$OUT_DIR"/* ]]; then
    echo "error: --out must not be a parent of the repository root" >&2
    exit 2
fi

# Clean the destination component directory so reruns are idempotent.
find "$OUT_DIR" -mindepth 1 -maxdepth 1 -exec rm -rf -- {} +

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
