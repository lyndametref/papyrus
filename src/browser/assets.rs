//! The embedded browser asset table.
//!
//! Every byte served under `/assets/` comes from this table. There is no
//! runtime asset directory, no path join, and no filesystem access: an asset
//! path is matched byte-exactly against the entries below.
//!
//! The contract is `docs/architecture/adr/ADR-0027-vendored-embedded-browser-assets-and-packaging-notices.md`.

/// One embedded file, addressed by its exact request path.
pub struct Asset {
    /// The exact request path, including the leading `/`.
    pub path: &'static str,
    /// The response `Content-Type` for this file.
    pub mime: &'static str,
    /// The file contents, embedded at compile time.
    pub bytes: &'static [u8],
}

const CSS: &str = "text/css; charset=utf-8";
const JS: &str = "text/javascript; charset=utf-8";
const WOFF2: &str = "font/woff2";

/// Every embedded browser asset, one entry per served file.
pub static ASSETS: &[Asset] = &[
    Asset {
        path: "/assets/app.css",
        mime: CSS,
        bytes: include_bytes!("app.css"),
    },
    Asset {
        path: "/assets/app.js",
        mime: JS,
        bytes: include_bytes!("app.js"),
    },
    Asset {
        path: "/assets/nav-state.js",
        mime: JS,
        bytes: include_bytes!("nav-state.js"),
    },
    Asset {
        path: "/assets/katex/katex.min.css",
        mime: CSS,
        bytes: include_bytes!("vendor/katex/0.16.11/katex.min.css"),
    },
    Asset {
        path: "/assets/katex/katex.min.js",
        mime: JS,
        bytes: include_bytes!("vendor/katex/0.16.11/katex.min.js"),
    },
    Asset {
        path: "/assets/katex/fonts/KaTeX_AMS-Regular.woff2",
        mime: WOFF2,
        bytes: include_bytes!("vendor/katex/0.16.11/fonts/KaTeX_AMS-Regular.woff2"),
    },
    Asset {
        path: "/assets/katex/fonts/KaTeX_Caligraphic-Bold.woff2",
        mime: WOFF2,
        bytes: include_bytes!("vendor/katex/0.16.11/fonts/KaTeX_Caligraphic-Bold.woff2"),
    },
    Asset {
        path: "/assets/katex/fonts/KaTeX_Caligraphic-Regular.woff2",
        mime: WOFF2,
        bytes: include_bytes!("vendor/katex/0.16.11/fonts/KaTeX_Caligraphic-Regular.woff2"),
    },
    Asset {
        path: "/assets/katex/fonts/KaTeX_Fraktur-Bold.woff2",
        mime: WOFF2,
        bytes: include_bytes!("vendor/katex/0.16.11/fonts/KaTeX_Fraktur-Bold.woff2"),
    },
    Asset {
        path: "/assets/katex/fonts/KaTeX_Fraktur-Regular.woff2",
        mime: WOFF2,
        bytes: include_bytes!("vendor/katex/0.16.11/fonts/KaTeX_Fraktur-Regular.woff2"),
    },
    Asset {
        path: "/assets/katex/fonts/KaTeX_Main-Bold.woff2",
        mime: WOFF2,
        bytes: include_bytes!("vendor/katex/0.16.11/fonts/KaTeX_Main-Bold.woff2"),
    },
    Asset {
        path: "/assets/katex/fonts/KaTeX_Main-BoldItalic.woff2",
        mime: WOFF2,
        bytes: include_bytes!("vendor/katex/0.16.11/fonts/KaTeX_Main-BoldItalic.woff2"),
    },
    Asset {
        path: "/assets/katex/fonts/KaTeX_Main-Italic.woff2",
        mime: WOFF2,
        bytes: include_bytes!("vendor/katex/0.16.11/fonts/KaTeX_Main-Italic.woff2"),
    },
    Asset {
        path: "/assets/katex/fonts/KaTeX_Main-Regular.woff2",
        mime: WOFF2,
        bytes: include_bytes!("vendor/katex/0.16.11/fonts/KaTeX_Main-Regular.woff2"),
    },
    Asset {
        path: "/assets/katex/fonts/KaTeX_Math-BoldItalic.woff2",
        mime: WOFF2,
        bytes: include_bytes!("vendor/katex/0.16.11/fonts/KaTeX_Math-BoldItalic.woff2"),
    },
    Asset {
        path: "/assets/katex/fonts/KaTeX_Math-Italic.woff2",
        mime: WOFF2,
        bytes: include_bytes!("vendor/katex/0.16.11/fonts/KaTeX_Math-Italic.woff2"),
    },
    Asset {
        path: "/assets/katex/fonts/KaTeX_SansSerif-Bold.woff2",
        mime: WOFF2,
        bytes: include_bytes!("vendor/katex/0.16.11/fonts/KaTeX_SansSerif-Bold.woff2"),
    },
    Asset {
        path: "/assets/katex/fonts/KaTeX_SansSerif-Italic.woff2",
        mime: WOFF2,
        bytes: include_bytes!("vendor/katex/0.16.11/fonts/KaTeX_SansSerif-Italic.woff2"),
    },
    Asset {
        path: "/assets/katex/fonts/KaTeX_SansSerif-Regular.woff2",
        mime: WOFF2,
        bytes: include_bytes!("vendor/katex/0.16.11/fonts/KaTeX_SansSerif-Regular.woff2"),
    },
    Asset {
        path: "/assets/katex/fonts/KaTeX_Script-Regular.woff2",
        mime: WOFF2,
        bytes: include_bytes!("vendor/katex/0.16.11/fonts/KaTeX_Script-Regular.woff2"),
    },
    Asset {
        path: "/assets/katex/fonts/KaTeX_Size1-Regular.woff2",
        mime: WOFF2,
        bytes: include_bytes!("vendor/katex/0.16.11/fonts/KaTeX_Size1-Regular.woff2"),
    },
    Asset {
        path: "/assets/katex/fonts/KaTeX_Size2-Regular.woff2",
        mime: WOFF2,
        bytes: include_bytes!("vendor/katex/0.16.11/fonts/KaTeX_Size2-Regular.woff2"),
    },
    Asset {
        path: "/assets/katex/fonts/KaTeX_Size3-Regular.woff2",
        mime: WOFF2,
        bytes: include_bytes!("vendor/katex/0.16.11/fonts/KaTeX_Size3-Regular.woff2"),
    },
    Asset {
        path: "/assets/katex/fonts/KaTeX_Size4-Regular.woff2",
        mime: WOFF2,
        bytes: include_bytes!("vendor/katex/0.16.11/fonts/KaTeX_Size4-Regular.woff2"),
    },
    Asset {
        path: "/assets/katex/fonts/KaTeX_Typewriter-Regular.woff2",
        mime: WOFF2,
        bytes: include_bytes!("vendor/katex/0.16.11/fonts/KaTeX_Typewriter-Regular.woff2"),
    },
];

/// Resolve an exact asset request path.
///
/// Matching is a linear scan for byte-exact, case-sensitive equality. It is
/// not a prefix match and performs no decoding or path handling.
pub fn lookup(path: &str) -> Option<&'static Asset> {
    ASSETS.iter().find(|asset| asset.path == path)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::ASSETS;

    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];

    fn sha256(data: &[u8]) -> [u8; 32] {
        let mut state: [u32; 8] = [
            0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
            0x5be0cd19,
        ];
        let bit_len = (data.len() as u64).wrapping_mul(8);
        let mut message = data.to_vec();
        message.push(0x80);
        while message.len() % 64 != 56 {
            message.push(0);
        }
        message.extend_from_slice(&bit_len.to_be_bytes());
        for chunk in message.as_chunks::<64>().0 {
            let mut words = [0u32; 64];
            for (index, word) in words.iter_mut().take(16).enumerate() {
                let start = index * 4;
                *word = u32::from_be_bytes([
                    chunk[start],
                    chunk[start + 1],
                    chunk[start + 2],
                    chunk[start + 3],
                ]);
            }
            for index in 16..64 {
                let s0 = words[index - 15].rotate_right(7)
                    ^ words[index - 15].rotate_right(18)
                    ^ (words[index - 15] >> 3);
                let s1 = words[index - 2].rotate_right(17)
                    ^ words[index - 2].rotate_right(19)
                    ^ (words[index - 2] >> 10);
                words[index] = words[index - 16]
                    .wrapping_add(s0)
                    .wrapping_add(words[index - 7])
                    .wrapping_add(s1);
            }
            let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = state;
            for index in 0..64 {
                let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
                let ch = (e & f) ^ ((!e) & g);
                let temp1 = h
                    .wrapping_add(s1)
                    .wrapping_add(ch)
                    .wrapping_add(K[index])
                    .wrapping_add(words[index]);
                let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
                let maj = (a & b) ^ (a & c) ^ (b & c);
                let temp2 = s0.wrapping_add(maj);
                h = g;
                g = f;
                f = e;
                e = d.wrapping_add(temp1);
                d = c;
                c = b;
                b = a;
                a = temp1.wrapping_add(temp2);
            }
            state[0] = state[0].wrapping_add(a);
            state[1] = state[1].wrapping_add(b);
            state[2] = state[2].wrapping_add(c);
            state[3] = state[3].wrapping_add(d);
            state[4] = state[4].wrapping_add(e);
            state[5] = state[5].wrapping_add(f);
            state[6] = state[6].wrapping_add(g);
            state[7] = state[7].wrapping_add(h);
        }
        let mut digest = [0u8; 32];
        for (index, word) in state.iter().enumerate() {
            digest[index * 4..index * 4 + 4].copy_from_slice(&word.to_be_bytes());
        }
        digest
    }

    fn hex(digest: &[u8; 32]) -> String {
        digest.iter().map(|byte| format!("{byte:02x}")).collect()
    }

    struct ManifestRow {
        asset_path: String,
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
            let (bytes_index, sha_index) = match fields.len() {
                7 => (5, 6),
                3 => (1, 2),
                width => panic!("unexpected manifest row width {width}: {line}"),
            };
            rows.push(ManifestRow {
                asset_path: asset_path.to_string(),
                bytes: fields[bytes_index]
                    .parse()
                    .unwrap_or_else(|_| panic!("invalid byte count in: {line}")),
                sha256: fields[sha_index].trim_matches('`').to_string(),
            });
        }
        rows
    }

    #[test]
    fn sha256_matches_standard_vectors() {
        assert_eq!(
            hex(&sha256(b"")),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            hex(&sha256(b"abc")),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
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
                hex(&sha256(asset.bytes)),
                "sha256 mismatch for {}",
                asset.path
            );
        }

        for row in &rows {
            assert!(
                ASSETS.iter().any(|asset| asset.path == row.asset_path),
                "manifest row for an asset that is not embedded: {}",
                row.asset_path
            );
        }
    }
}
