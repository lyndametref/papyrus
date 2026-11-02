#!/usr/bin/env bash
set -euo pipefail

# Resolve project root (script directory's parent).
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

VERSION="$(awk -F '\"' '/^version[[:space:]]*=/{print $2; exit}' Cargo.toml)"
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

# Render control file with the Cargo version.
sed "s/^Version: .*/Version: ${VERSION}/" \
    "${ROOT}/packaging/deb/DEBIAN/control" > "${STAGE}/DEBIAN/control"
chmod 0644 "${STAGE}/DEBIAN/control"

cp "${ROOT}/packaging/deb/usr/share/doc/papyrus/copyright" \
    "${STAGE}/usr/share/doc/papyrus/copyright"
chmod 0644 "${STAGE}/usr/share/doc/papyrus/copyright"

# Build the .deb in the project root.
dpkg-deb --build --root-owner-group "$STAGE" "${ROOT}/${PKG}.deb"

echo "Built ${ROOT}/${PKG}.deb"
