---
id: TECH-001
title: "Rust CLI and Debian Package Baseline"
created_date: '2026-11-16T17:46:42Z'
updated_date: '2026-11-16T17:46:59Z'
labels: []
references:
  - ADR-0001
  - ADR-0002
  - ADR-0003
  - ADR-0005
  - ADR-0025
  - ADR-0027
  - BR-001
  - NFR-001.001
  - NFR-001.003
  - NFR-003.001
  - NFR-003.002
  - NFR-005.001
  - NFR-006.001
  - NFR-006.002
  - REQ-006.017
  - TASK-1
  - TASK-143
  - TASK-146
  - TASK-148
  - TASK-2
  - TASK-22
status: archived
---
# TECH-001 Rust CLI and Debian Package Baseline

## Description
## Purpose

Establish the installable executable foundation for Papyrus: a Rust command-line binary that builds and runs with documented help and version output, packaged as an amd64 Debian `.deb` for Debian Trixie and Forky, licensed under GPL-3.0-or-later, with a runtime dependency-license audit. All later Papyrus capabilities depend on this baseline (TASK-2 … TASK-22).

## Scope

### In scope

- Create the Papyrus Rust crate (single crate, binary + library target) using Cargo, pinned to the Rust 1.99.1 toolchain (ADR-0001).
- Implement the minimal CLI surface: `papyrus --help` / `papyrus -h` and `papyrus --version`, plus the documented global option declarations needed for help rendering, exiting with status 0 on success (BR-001, README `papyrus(1)`).
- Declare GPL-3.0-or-later licensing: `LICENSE` file and crate/package metadata (`Cargo.toml` license field and `.deb` copyright/license metadata).
- Build an amd64 `.deb` package (`papyrus_<version>_amd64.deb`) using `dpkg-deb`, with control metadata (Package, Version, Architecture, Maintainer, Description, Section, Priority, and dependency declarations).
- Document the build, packaging, install, uninstall, and smoke-test procedure.
- Audit runtime dependency licenses and record the result.

### Out of scope

- Subcommands (`init`, `config`, `task`, …) and their behavior — TASK-2 onward.
- Global option runtime behavior beyond help/version rendering (e.g. `--dir`, `--json`, `--config`, `--no-color` semantics) — TASK-2.
- Multi-crate workspace or internal module architecture beyond a minimal `src/main.rs` + `src/lib.rs` split.
- Additional architectures (arm64, …) or non-Debian distribution formats — ADR-0002.
- Advertising support for Debian releases beyond Trixie and Forky — open question NFR-003.002 / ADR-0002 remains unresolved.

## Behavior

1. **Build:** `cargo build --release` (with the pinned toolchain) succeeds and produces an executable named `papyrus`.
2. **Help:** `papyrus --help` and `papyrus -h` print the documented usage/help text and exit 0.
3. **Version:** `papyrus --version` prints `papyrus <version>` (Cargo package version) and exits 0.
4. **Invalid usage:** unknown flags/arguments print usage to stderr and exit with status 2 (usage error), per BR-001 exit-status contract.
5. **Packaging:** the documented packaging command produces `papyrus_<version>_amd64.deb` with `Architecture: amd64`.
6. **Install:** `dpkg -i papyrus_<version>_amd64.deb` succeeds on Debian Trixie and Forky; the `papyrus` binary is on `PATH`.
7. **Uninstall:** `dpkg -r papyrus` (or `dpkg --purge`) removes the binary cleanly.
8. **Smoke test (installed):** `papyrus --help` and `papyrus --version` succeed on the installed binary.
9. **Licensing:** distributed artifacts declare GPL-3.0-or-later; runtime dependencies are open source and their licenses permit linking in a GPLv3-or-later application.

## Packaging details

- Package name: `papyrus`.
- Architecture: `amd64`.
- Supported suites declared: Debian Trixie and Forky only. Later Debian releases are not advertised until the NFR-003.002 open question is resolved (ADR-0002).
- Forward compatibility: the binary is built against a conservative glibc baseline so it is expected to run on later Debian releases, but this is not a supported/advertised claim.
- Control fields follow Debian policy; maintainer and homepage values are recorded in packaging metadata.

## Licensing and dependency audit

- License: GPL-3.0-or-later (SPDX `GPL-3.0-or-later`).
- Audit scope: **runtime dependencies only** (crates linked into the distributed binary). Dev/build-time dependencies are out of scope for this task; if a build-time dependency's license is later found incompatible, it is tracked separately.
- Audit output: a documented list of every runtime dependency with its license (SPDX identifier) and a compatibility verdict (compatible / incompatible). Stored with the packaging notes.
- Any incompatible runtime dependency blocks completion of this task.

## Assumptions

- Assumption: Rust 1.99.1 (via rustup) is available in the build environment, per ADR-0001.
- Assumption: `dpkg-deb` is available for packaging, per ADR-0002.
- Assumption: the Cargo package version is the single source of truth for the `.deb` version and the `--version` output.
- Assumption: Node.js/Python, if used, remain build-time/auxiliary tooling only and are never runtime dependencies (ADR-0001, ADR-0002).
- Assumption: minimal single-crate layout (`src/main.rs` + `src/lib.rs`); any move to a workspace or module boundaries beyond this requires a new ADR and is out of scope.
- Assumption: man-page installation in the `.deb` is optional and may be deferred; help text is the required documentation surface for this task.
- Assumption: package metadata values not specified by requirements (maintainer, section, priority) follow Debian policy defaults and are recorded in packaging files.

## Open questions

- OQ-1: The "later Debian releases" boundary (NFR-003.002) is unresolved; this task advertises only Trixie and Forky. Resolution is tracked against NFR-003.002 and ADR-0002.
- OQ-2: The exact interpretation of "open source" and "linking" for dev/build-time dependencies (NFR-006.002) remains unresolved; this task audits runtime dependencies only.

## Traceability

| Artifact | Reference |
|---|---|
| NFR | NFR-001.001, NFR-003.001, NFR-003.002, NFR-006.001, NFR-006.002 |
| Business rule | BR-001 (papyrus command behavior) |
| ADR | ADR-0001 (Rust), ADR-0003 (Debian Forky only; supersedes ADR-0002) |
| Ticket | TASK-1 |
| UI mockups | none |
path: docs/specifications/rust-cli-and-debian-package-baseline.md

## Bundled browser assets and third-party notices (TASK-143)

Contract source: ADR-0027 (Accepted) for the asset model and the notices, ADR-0025 (Accepted) for the KaTeX file set, ADR-0005 as amended by ADR-0027 decision 7. This section narrows two earlier statements in this specification: the asset sentence in [Packaging details](#packaging-details) ("assets are compiled into the binary") and the licence statement in [Licensing and dependency audit](#licensing-and-dependency-audit) (GPL-3.0-or-later as the package's only declared licence). Both remain true for first-party code; bundled third-party assets add the sections below. Wording that only *describes* bundled fonts in the browse UIs is not revised here — that is the specification-revision task.

### 1. Static asset table and routing

- `src/browser/assets.rs` exposes `pub struct Asset { pub path: &'static str, pub mime: &'static str, pub bytes: &'static [u8] }`, `pub static ASSETS: &[Asset]`, and `pub fn lookup(path: &str) -> Option<&'static Asset>`.
- One `Asset` per served file; `path` is the exact request path, including its leading `/`. Bytes come from `include_bytes!`/`include_str!` at compile time; nothing is read from disk at runtime.
- `lookup` is a linear scan for byte-exact, case-sensitive equality. It performs no prefix matching, no directory index, no path join, no percent-decoding, and no filesystem access.
- Routing: every request path that starts with `/assets/` is resolved only by the table — a hit with `GET` or `HEAD` is `200` with the asset's `mime` and bytes, a hit with any other method is `405`, and a miss is `404` for every method. All other paths route exactly as before. The `/assets/` prefix is a routing gate, not part of the lookup semantics: `/assets/`, `/assets/app.css/`, and `/assets/nope` are misses.
- `HEAD` sends the `GET` status and headers, including `Content-Length` equal to the body length, and no body.
- The MIME set is exactly `text/css; charset=utf-8` for stylesheets, `text/javascript; charset=utf-8` for scripts, and `font/woff2` for WOFF2 font files. `browser::http::Response` gains `bytes(status: u16, content_type: &'static str, body: &'static [u8])`.
- `Cache-Control: no-store` and the other ADR-0005 response headers stay global in the response writer. No `ETag`, no `Accept-Encoding` negotiation, no compression, no range request, and no asset-specific cache header is added. A `405` for an asset and a `404` for an unknown path reuse the existing error rendering and its current `Allow` header unchanged.

### 2. Served paths added by TASK-143

`/assets/app.css` and `/assets/app.js` move from the hard-coded route blocks onto the table with unchanged paths, bytes, and MIME types. TASK-143 adds:

| Path | Bytes | MIME |
|---|---|---|
| `/assets/katex/katex.min.css` | 23,335 | `text/css; charset=utf-8` |
| `/assets/katex/katex.min.js` | 275,414 | `text/javascript; charset=utf-8` |
| `/assets/katex/fonts/<name>.woff2` (20 files) | 259,792 total | `font/woff2` |
| `/assets/mermaid/mermaid.min.js` | 3,572,661 | `text/javascript; charset=utf-8` |

The 20 WOFF2 files are `KaTeX_AMS-Regular`, `KaTeX_Caligraphic-Bold`, `KaTeX_Caligraphic-Regular`, `KaTeX_Fraktur-Bold`, `KaTeX_Fraktur-Regular`, `KaTeX_Main-Bold`, `KaTeX_Main-BoldItalic`, `KaTeX_Main-Italic`, `KaTeX_Main-Regular`, `KaTeX_Math-BoldItalic`, `KaTeX_Math-Italic`, `KaTeX_SansSerif-Bold`, `KaTeX_SansSerif-Italic`, `KaTeX_SansSerif-Regular`, `KaTeX_Script-Regular`, `KaTeX_Size1-Regular`, `KaTeX_Size2-Regular`, `KaTeX_Size3-Regular`, `KaTeX_Size4-Regular`, `KaTeX_Typewriter-Regular`, each with the `.woff2` extension. The `.woff` and `.ttf` forms of the same families are not vendored and are not served: the supported browsers (NFR-005.001) always select the `woff2` source, so the other two sources in each `@font-face` block are inert.

The Mermaid row is Mermaid 11.17.2, MIT, SHA-256 `581ed7d74bd9048d0e3a91363927d72ef22942d7722546b27f7cc29e35390eb8`, landed by TASK-146. The served KaTeX total is 558,541 B (CSS 23,335 + JS 275,414 + 20 WOFF2 259,792).

### 3. Vendor layout

- `src/browser/vendor/<package>/<version>/` holds the unmodified upstream files, `LICENSE`, and `UPSTREAM.md`.
- KaTeX 0.16.11 keeps upstream's `dist/` contents flattened onto the version root, so `katex.min.css`, `katex.min.js`, and `fonts/` sit side by side. This is what makes the stylesheet's relative `url(fonts/…)` values resolve against `/assets/katex/katex.min.css` to `/assets/katex/fonts/…`. Flattening the directory does not alter a file's bytes, so it is not a vendoring patch; the upstream path (`package/dist/…`) is recorded in `UPSTREAM.md`.
- `UPSTREAM.md` records: package name, version, the exact upstream URL the archive was downloaded from, the path of each vendored file inside that archive, the archive's SHA-256, the retrieval date (UTC, `YYYY-MM-DD`), and a statement that no file was modified.
- `LICENSE`, `UPSTREAM.md`, and `MANIFEST.md` are not entries in `ASSETS` and are never served.

### 4. Digest manifest

`src/browser/vendor/MANIFEST.md` is the auditable record of every embedded byte. It contains two Markdown tables with fixed column order, one row per `ASSETS` entry, rows sorted by `path`, `bytes` written as decimal integers, and `sha256` written as 64 lowercase hexadecimal characters:

- Vendored assets: `path | package | version | licence | upstream | bytes | sha256`.
- First-party assets: `path | bytes | sha256`.

The test that keeps it honest is a unit test in `src/browser/assets.rs` that:

1. parses both tables from `include_str!("vendor/MANIFEST.md")`;
2. asserts that the set of manifest paths equals the set of `ASSETS[].path` exactly, with no duplicates and no missing row;
3. recomputes SHA-256 over each `Asset.bytes` and compares it with the recorded value and the recorded byte length;
4. uses a `#[cfg(test)]` SHA-256 implementation in the same module (FIPS 180-4), because no digest crate may be added, and self-checks it against the standard vectors for the empty input and for `abc`.

A digest mismatch, a missing row, an extra row, or a stale byte count fails the build.

### 5. Package notices

`packaging/deb/usr/share/doc/papyrus/copyright` stays the only documentation file in the package and stays in Debian machine-readable format. It gains, after the existing `Files: *` block, one section per bundled asset family:

- **KaTeX code** (`katex.min.js`, `katex.min.css`) — `License: MIT`, the upstream copyright line, and the full MIT text verbatim from `src/browser/vendor/katex/0.16.11/LICENSE`.
- **KaTeX fonts** (`fonts/*.woff2`) — `License: OFL-1.1`, the font attribution recorded by the licence review, and the full SIL Open Font License 1.1 text verbatim.
- **Mermaid** (`mermaid.min.js`) — a standalone `License: MIT-Mermaid` section with its own short name, the Knut Sveidqvist copyright line, and the full MIT text verbatim from `src/browser/vendor/mermaid/11.17.2/LICENSE`. It does not reuse the KaTeX `MIT` paragraph, which embeds the Khan Academy copyright.

The copyright file stays the only documentation file in the package.

First-party assets (`app.css`, `app.js`, templates) stay covered by the existing `Files: *` GPL-3.0-or-later block and get no extra section. A later bundled package adds its own section when it lands. `scripts/build-deb.sh` is not modified: it still copies exactly one copyright file, so its shape and the two-entry package content are preserved.

The licence inventory is a release gate (NFR-006.002): an asset whose licence cannot be shown compatible with GPL-3.0-or-later does not ship, and an incomplete notice blocks the release rather than being deferred.

### 6. Specification port (TASK-148)

The browser-UI and packaging-verification pages that once described an asset-free package and a system-font-only UI are revised to the landed contract. `docs/specifications/browser-shell-ui.md` is the normative shell/list/width/navigation page; `docs/specifications/browser-requirements.md` §9.7 is the single normative home of the two CSP strings; `docs/specifications/browser-math-and-diagram-rendering.md` records the math and diagram rendering contract. This section remains the source for the asset-table, vendor-layout, digest, and notice clauses.

### Contract traceability

| Artifact | Reference |
|---|---|
| ADR | ADR-0027 (asset table, vendor layout, manifest, notices), ADR-0025 (KaTeX file set and font serving), ADR-0005 (as amended, header and CSP baseline) |
| Requirement | NFR-001.003, NFR-003.001, NFR-005.001, NFR-006.001, NFR-006.002, REQ-006.017 |
| Ticket | TASK-143 |
