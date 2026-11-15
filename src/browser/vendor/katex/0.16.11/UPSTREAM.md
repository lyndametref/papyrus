# KaTeX 0.16.11 — upstream record

| Field | Value |
|---|---|
| Package | KaTeX (`katex`) |
| Version | 0.16.11 |
| Download URL | <https://registry.npmjs.org/katex/-/katex-0.16.11.tgz> |
| Archive SHA-256 | `8c5494c5f7d8e8bf73ff3fe64d4bd09a340e4878b7eec9bbd9ef8d8f3e24193f` |
| Retrieval date (UTC) | 2026-11-15 |
| Modification | No vendored file was modified. Every file is byte-identical to the archive entry listed below. |

Upstream's `package/dist/` directory is flattened onto this version root so
that `katex.min.css` resolves its relative `url(fonts/…)` references to
`/assets/katex/fonts/…`. Flattening a directory does not change file bytes.

## Vendored files and their archive paths

| Vendored file | Archive path |
|---|---|
| `katex.min.js` | `package/dist/katex.min.js` |
| `katex.min.css` | `package/dist/katex.min.css` |
| `LICENSE` | `package/LICENSE` |
| `fonts/KaTeX_AMS-Regular.woff2` | `package/dist/fonts/KaTeX_AMS-Regular.woff2` |
| `fonts/KaTeX_Caligraphic-Bold.woff2` | `package/dist/fonts/KaTeX_Caligraphic-Bold.woff2` |
| `fonts/KaTeX_Caligraphic-Regular.woff2` | `package/dist/fonts/KaTeX_Caligraphic-Regular.woff2` |
| `fonts/KaTeX_Fraktur-Bold.woff2` | `package/dist/fonts/KaTeX_Fraktur-Bold.woff2` |
| `fonts/KaTeX_Fraktur-Regular.woff2` | `package/dist/fonts/KaTeX_Fraktur-Regular.woff2` |
| `fonts/KaTeX_Main-Bold.woff2` | `package/dist/fonts/KaTeX_Main-Bold.woff2` |
| `fonts/KaTeX_Main-BoldItalic.woff2` | `package/dist/fonts/KaTeX_Main-BoldItalic.woff2` |
| `fonts/KaTeX_Main-Italic.woff2` | `package/dist/fonts/KaTeX_Main-Italic.woff2` |
| `fonts/KaTeX_Main-Regular.woff2` | `package/dist/fonts/KaTeX_Main-Regular.woff2` |
| `fonts/KaTeX_Math-BoldItalic.woff2` | `package/dist/fonts/KaTeX_Math-BoldItalic.woff2` |
| `fonts/KaTeX_Math-Italic.woff2` | `package/dist/fonts/KaTeX_Math-Italic.woff2` |
| `fonts/KaTeX_SansSerif-Bold.woff2` | `package/dist/fonts/KaTeX_SansSerif-Bold.woff2` |
| `fonts/KaTeX_SansSerif-Italic.woff2` | `package/dist/fonts/KaTeX_SansSerif-Italic.woff2` |
| `fonts/KaTeX_SansSerif-Regular.woff2` | `package/dist/fonts/KaTeX_SansSerif-Regular.woff2` |
| `fonts/KaTeX_Script-Regular.woff2` | `package/dist/fonts/KaTeX_Script-Regular.woff2` |
| `fonts/KaTeX_Size1-Regular.woff2` | `package/dist/fonts/KaTeX_Size1-Regular.woff2` |
| `fonts/KaTeX_Size2-Regular.woff2` | `package/dist/fonts/KaTeX_Size2-Regular.woff2` |
| `fonts/KaTeX_Size3-Regular.woff2` | `package/dist/fonts/KaTeX_Size3-Regular.woff2` |
| `fonts/KaTeX_Size4-Regular.woff2` | `package/dist/fonts/KaTeX_Size4-Regular.woff2` |
| `fonts/KaTeX_Typewriter-Regular.woff2` | `package/dist/fonts/KaTeX_Typewriter-Regular.woff2` |

The archive also contains `.woff` and `.ttf` forms, `katex.css`, `katex.js`,
`katex.mjs`, `dist/contrib/**`, `src/**`, `README.md`, and `package.json`.
Those files are deliberately not vendored and are not served.

## Font licence

The archive's `LICENSE` covers the code (MIT). The font files carry their own
SIL OFL-1.1 metadata; `fonts/OFL.txt` is the official OFL 1.1 text retrieved
on 2026-11-15 from <https://openfontlicense.org/documents/OFL.txt> because the
0.16.11 archive ships no OFL text. The per-font copyright and Reserved Font
Name statements are read from each font's own `name` table; they are recorded
in the package notice file.
