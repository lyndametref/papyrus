//! The embedded browser asset table.
//!
//! Every byte served under `/assets/` comes from this table. There is no
//! runtime asset directory, no path join, and no filesystem access: an asset
//! path is matched byte-exactly against the entries below.
//!
//! Vendor files carry their packaged asset version in the URL, a literal
//! compile-time path that mirrors the embedding path. First-party files stay
//! unversioned and are revalidated by a strong SHA-256 `ETag`.
//!
//! The contract is `docs/architecture/adr/ADR-0027-vendored-embedded-browser-assets-and-packaging-notices.md`
//! and `docs/architecture/adr/ADR-038-cacheable-immutable-browser-assets.md`.

use std::collections::BTreeMap;

use crate::browser::sha256;

/// How an embedded asset is cached (ADR-038 decision 5).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssetKind {
    /// A vendored file served from a versioned, immutable URL.
    Vendor,
    /// A first-party file served unversioned with a strong `ETag`.
    FirstParty,
}

/// One embedded file, addressed by its exact request path.
pub struct Asset {
    /// The exact request path, including the leading `/`.
    pub path: &'static str,
    /// The response `Content-Type` for this file.
    pub mime: &'static str,
    /// The file contents, embedded at compile time.
    pub bytes: &'static [u8],
    /// The embedded file path relative to this module. The version-binding
    /// test uses it to prove the URL version and the embedded directory agree.
    pub embedded: &'static str,
    /// The cache class produced by this entry.
    pub kind: AssetKind,
}

const CSS: &str = "text/css; charset=utf-8";
const JS: &str = "text/javascript; charset=utf-8";
const WOFF2: &str = "font/woff2";

macro_rules! first_party_asset {
    ($path:literal, $mime:expr, $file:literal) => {
        Asset {
            path: $path,
            mime: $mime,
            bytes: include_bytes!($file),
            embedded: $file,
            kind: AssetKind::FirstParty,
        }
    };
}

macro_rules! vendor_asset {
    ($mime:expr, $package:literal, $version:literal, $file:literal) => {
        Asset {
            path: concat!("/assets/", $package, "/", $version, "/", $file),
            mime: $mime,
            bytes: include_bytes!(concat!("vendor/", $package, "/", $version, "/", $file)),
            embedded: concat!("vendor/", $package, "/", $version, "/", $file),
            kind: AssetKind::Vendor,
        }
    };
}

/// Every embedded browser asset, one entry per served file.
pub static ASSETS: &[Asset] = &[
    first_party_asset!("/assets/app.css", CSS, "app.css"),
    first_party_asset!("/assets/app.js", JS, "app.js"),
    first_party_asset!("/assets/nav-state.js", JS, "nav-state.js"),
    vendor_asset!(CSS, "katex", "0.16.11", "katex.min.css"),
    vendor_asset!(JS, "katex", "0.16.11", "katex.min.js"),
    vendor_asset!(WOFF2, "katex", "0.16.11", "fonts/KaTeX_AMS-Regular.woff2"),
    vendor_asset!(
        WOFF2,
        "katex",
        "0.16.11",
        "fonts/KaTeX_Caligraphic-Bold.woff2"
    ),
    vendor_asset!(
        WOFF2,
        "katex",
        "0.16.11",
        "fonts/KaTeX_Caligraphic-Regular.woff2"
    ),
    vendor_asset!(WOFF2, "katex", "0.16.11", "fonts/KaTeX_Fraktur-Bold.woff2"),
    vendor_asset!(
        WOFF2,
        "katex",
        "0.16.11",
        "fonts/KaTeX_Fraktur-Regular.woff2"
    ),
    vendor_asset!(WOFF2, "katex", "0.16.11", "fonts/KaTeX_Main-Bold.woff2"),
    vendor_asset!(
        WOFF2,
        "katex",
        "0.16.11",
        "fonts/KaTeX_Main-BoldItalic.woff2"
    ),
    vendor_asset!(WOFF2, "katex", "0.16.11", "fonts/KaTeX_Main-Italic.woff2"),
    vendor_asset!(WOFF2, "katex", "0.16.11", "fonts/KaTeX_Main-Regular.woff2"),
    vendor_asset!(
        WOFF2,
        "katex",
        "0.16.11",
        "fonts/KaTeX_Math-BoldItalic.woff2"
    ),
    vendor_asset!(WOFF2, "katex", "0.16.11", "fonts/KaTeX_Math-Italic.woff2"),
    vendor_asset!(
        WOFF2,
        "katex",
        "0.16.11",
        "fonts/KaTeX_SansSerif-Bold.woff2"
    ),
    vendor_asset!(
        WOFF2,
        "katex",
        "0.16.11",
        "fonts/KaTeX_SansSerif-Italic.woff2"
    ),
    vendor_asset!(
        WOFF2,
        "katex",
        "0.16.11",
        "fonts/KaTeX_SansSerif-Regular.woff2"
    ),
    vendor_asset!(
        WOFF2,
        "katex",
        "0.16.11",
        "fonts/KaTeX_Script-Regular.woff2"
    ),
    vendor_asset!(WOFF2, "katex", "0.16.11", "fonts/KaTeX_Size1-Regular.woff2"),
    vendor_asset!(WOFF2, "katex", "0.16.11", "fonts/KaTeX_Size2-Regular.woff2"),
    vendor_asset!(WOFF2, "katex", "0.16.11", "fonts/KaTeX_Size3-Regular.woff2"),
    vendor_asset!(WOFF2, "katex", "0.16.11", "fonts/KaTeX_Size4-Regular.woff2"),
    vendor_asset!(
        WOFF2,
        "katex",
        "0.16.11",
        "fonts/KaTeX_Typewriter-Regular.woff2"
    ),
    vendor_asset!(JS, "mermaid", "11.17.2", "mermaid.min.js"),
];

/// Resolve an exact asset request path.
///
/// Matching is a linear scan for byte-exact, case-sensitive equality. It is
/// not a prefix match and performs no decoding or path handling.
pub fn lookup(path: &str) -> Option<&'static Asset> {
    ASSETS.iter().find(|asset| asset.path == path)
}

/// Compute the strong `ETag` for every first-party asset once at startup.
///
/// The value is a double quote, the SHA-256 of the exact embedded bytes as 64
/// lowercase hexadecimal characters, and a double quote. The caller holds the
/// map for the process lifetime, so validation never re-hashes a request body
/// and a reload cannot change a validator.
pub fn first_party_etags() -> BTreeMap<&'static str, String> {
    ASSETS
        .iter()
        .filter(|asset| asset.kind == AssetKind::FirstParty)
        .map(|asset| {
            let digest = sha256::digest(asset.bytes);
            (asset.path, format!("\"{}\"", sha256::hex(&digest)))
        })
        .collect()
}

/// The strong `ETag` for a first-party asset from the startup map.
///
/// Vendor assets have no validator and return `None`.
pub fn etag<'a>(
    etags: &'a BTreeMap<&'static str, String>,
    asset: &'static Asset,
) -> Option<&'a str> {
    etags.get(asset.path).map(String::as_str)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{ASSETS, AssetKind, etag, first_party_etags, lookup};
    use crate::browser::sha256;

    struct ManifestRow {
        asset_path: String,
        package: String,
        version: String,
        bytes: usize,
        sha256: String,
    }

    fn manifest_rows() -> Vec<ManifestRow> {
        let text = include_str!("vendor/MANIFEST.md");
        let mut rows = Vec::new();
        for line in text.lines() {
            let line = line.trim();
            if !line.starts_with('|') {
                continue;
            }
            let fields: Vec<&str> = line.trim_matches('|').split('|').map(str::trim).collect();
            let asset_path = fields[0].trim_matches('`');
            if !asset_path.starts_with("/assets/") {
                continue;
            }
            let (package, version, bytes_index, sha_index) = match fields.len() {
                7 => (fields[1].to_string(), fields[2].to_string(), 5, 6),
                3 => (String::new(), String::new(), 1, 2),
                width => panic!("unexpected manifest row width {width}: {line}"),
            };
            rows.push(ManifestRow {
                asset_path: asset_path.to_string(),
                package,
                version,
                bytes: fields[bytes_index]
                    .parse()
                    .unwrap_or_else(|_| panic!("invalid byte count in: {line}")),
                sha256: fields[sha_index].trim_matches('`').to_string(),
            });
        }
        rows
    }

    /// Split a served vendor path into `(package, version, file)`.
    fn vendor_path_parts(path: &str) -> Option<(&str, &str, &str)> {
        let rest = path.strip_prefix("/assets/")?;
        let mut parts = rest.splitn(3, '/');
        let package = parts.next()?;
        let version = parts.next()?;
        let file = parts.next()?;
        if package.is_empty() || version.is_empty() || file.is_empty() {
            return None;
        }
        Some((package, version, file))
    }

    /// Split an embedded path into `(package, version, file)`.
    fn embedded_path_parts(path: &str) -> Option<(&str, &str, &str)> {
        let rest = path.strip_prefix("vendor/")?;
        let mut parts = rest.splitn(3, '/');
        let package = parts.next()?;
        let version = parts.next()?;
        let file = parts.next()?;
        if package.is_empty() || version.is_empty() || file.is_empty() {
            return None;
        }
        Some((package, version, file))
    }

    #[test]
    fn manifest_covers_every_asset_with_matching_bytes_and_digest() {
        let rows = manifest_rows();
        assert_eq!(rows.len(), ASSETS.len(), "manifest row count");

        let mut seen: BTreeSet<&str> = BTreeSet::new();
        for row in &rows {
            assert!(
                seen.insert(row.asset_path.as_str()),
                "duplicate manifest row for {}",
                row.asset_path
            );
        }

        for asset in ASSETS {
            let row = rows
                .iter()
                .find(|row| row.asset_path == asset.path)
                .unwrap_or_else(|| panic!("missing manifest row for {}", asset.path));
            assert_eq!(
                row.bytes,
                asset.bytes.len(),
                "byte count mismatch for {}",
                asset.path
            );
            assert_eq!(
                row.sha256,
                sha256::hex(&sha256::digest(asset.bytes)),
                "sha256 mismatch for {}",
                asset.path
            );

            // Version binding: the served path version, the manifest
            // `version` column, and the embedded directory are one value.
            match asset.kind {
                AssetKind::Vendor => {
                    let (package, version, file) = vendor_path_parts(asset.path)
                        .unwrap_or_else(|| panic!("vendor path is not versioned: {}", asset.path));
                    let (embedded_package, embedded_version, embedded_file) =
                        embedded_path_parts(asset.embedded).unwrap_or_else(|| {
                            panic!("embedded vendor path is not versioned: {}", asset.embedded)
                        });
                    assert_eq!(package, embedded_package, "package {}", asset.path);
                    assert_eq!(version, embedded_version, "version {}", asset.path);
                    assert_eq!(file, embedded_file, "file {}", asset.path);
                    assert_eq!(
                        row.version, version,
                        "manifest version for {} is not the served version",
                        asset.path
                    );
                    assert_eq!(
                        row.package.to_ascii_lowercase(),
                        package,
                        "manifest package for {} is not the served package",
                        asset.path
                    );
                }
                AssetKind::FirstParty => {
                    assert!(
                        vendor_path_parts(asset.path).is_none(),
                        "first-party path gained a version segment: {}",
                        asset.path
                    );
                    assert_eq!(row.package, "", "first-party row package {}", asset.path);
                    assert_eq!(row.version, "", "first-party row version {}", asset.path);
                }
            }
        }

        for row in &rows {
            assert!(
                ASSETS.iter().any(|asset| asset.path == row.asset_path),
                "manifest row for an asset that is not embedded: {}",
                row.asset_path
            );
        }
    }

    #[test]
    fn first_party_etags_are_strong_sha256_and_stable() {
        let etags = first_party_etags();
        let mut seen = 0;
        for asset in ASSETS {
            match asset.kind {
                AssetKind::Vendor => {
                    assert!(etag(&etags, asset).is_none(), "vendor {}", asset.path)
                }
                AssetKind::FirstParty => {
                    seen += 1;
                    let value = etag(&etags, asset).expect("first-party etag");
                    let expected = format!("\"{}\"", sha256::hex(&sha256::digest(asset.bytes)));
                    assert_eq!(value, expected, "etag for {}", asset.path);
                    assert_eq!(value.len(), 66);
                    assert!(value.starts_with('"') && value.ends_with('"'));
                    assert!(
                        value[1..65]
                            .bytes()
                            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase()),
                        "etag is not lowercase hex: {value}"
                    );
                    let again = etag(&etags, asset).expect("first-party etag");
                    assert!(
                        std::ptr::eq(value, again),
                        "etag for {} is recomputed per call",
                        asset.path
                    );
                }
            }
        }
        assert_eq!(seen, 3, "exactly three first-party assets");
    }

    #[test]
    fn unversioned_vendor_paths_are_absent() {
        for path in [
            "/assets/katex/katex.min.css",
            "/assets/katex/katex.min.js",
            "/assets/katex/fonts/KaTeX_Main-Regular.woff2",
            "/assets/mermaid/mermaid.min.js",
        ] {
            assert!(lookup(path).is_none(), "unversioned alias {path}");
        }
    }
}
