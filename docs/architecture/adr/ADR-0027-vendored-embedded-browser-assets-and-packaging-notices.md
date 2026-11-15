# Vendored Embedded Browser Assets and Packaging Notices

## Status

Proposed (2026-11-15) — awaiting owner validation. Do not implement before acceptance.

## Date

2026-11-15

## Context

ADR-0025 and ADR-0026 require third-party browser assets (KaTeX with its font files, Mermaid) inside the single binary that ADR-0002/ADR-0003 package as a `.deb`. The current asset model does not cover them:

- `src/browser/mod.rs` embeds exactly two assets (`include_str!("app.css")`, `include_str!("app.js")`) and serves them from two hard-coded `if path == "/assets/app.css"` blocks.
- `browser::http::Response::text` only carries `&str`; there is no binary body and no font MIME type.
- ADR-0005 permits embedded assets and forbids remote ones, forbids compression, and fixes `Cache-Control: no-store`.
- The package contains exactly three entries under `usr/share/doc/papyrus/`: the `copyright` file. `scripts/build-deb.sh` copies one copyright file and one control file.
- Several existing packaging specifications and tests assert that **no** served CSS contains `url(`, and that no page references a web font. KaTeX CSS contains 60 `url(fonts/…)` references and 20 `@font-face` blocks, so those criteria must change.

Forces: single binary and no runtime asset directory (ADR-0002, ADR-0005); no new Rust crate and no Node/npm build step (ADR-0001, ADR-0004); offline determinism; GPLv3-or-later compatibility for every bundled licence (NFR-06.002, NFR-06.001); an auditable supply chain for files that become Papyrus bugs when they become Papyrus CVEs; and Debian's requirement that `usr/share/doc/<pkg>/copyright` carry every licence and copyright notice of the distributed files.

The total new weight is ≈4.0 MiB (Mermaid 3,572,661 B; KaTeX 275,414 B + 23,335 B + ≈259,792 B of woff2). The current binary is 4,197,544 B.

## Decision

1. **One static asset table, exact-match routing, binary-safe bodies.** A new module `src/browser/assets.rs` exposes:

   ```rust
   pub struct Asset {
       pub path: &'static str,       // "/assets/katex/katex.min.css"
       pub mime: &'static str,       // "text/css; charset=utf-8"
       pub bytes: &'static [u8],     // include_bytes! / include_str!.as_bytes()
   }
   pub static ASSETS: &[Asset] = &[ /* … */ ];
   pub fn lookup(path: &str) -> Option<&'static Asset>;
   ```

   `browser::http` gains `Response::bytes(status, content_type, &'static [u8])`. `lookup` is a linear scan over exact strings; a miss is `404` for every method, a hit with a method other than `GET`/`HEAD` is `405` (ADR-0005 semantics preserved). `HEAD` sends `Content-Length` and no body. No prefix matching, no directory expansion, no path join, no percent-decoded filesystem access.

2. **Vendored assets are committed upstream artifacts, not build products.** Layout: `src/browser/vendor/<package>/<version>/` containing the unmodified upstream files, `LICENSE`, and `UPSTREAM.md` (package, version, upstream URL, retrieval date). Redirect/registry URLs are recorded, not fetched at build time. Consequence: `cargo build` remains the entire build and a checkout builds identically offline.

3. **No vendoring patches by default.** Files are served byte-identical to upstream so a recorded digest is meaningful; the KaTeX font references work unmodified because `/assets/katex/katex.min.css` resolves `fonts/…` to `/assets/katex/fonts/…`. If a future asset genuinely needs a patch, the patch is a documented `.patch` file next to the asset and the digest is recorded **after** patching, with the patch listed in the notice.

4. **Integrity and update procedure.** `src/browser/vendor/MANIFEST.md` lists, per asset: package, version, licence, upstream URL, byte size, and SHA-256. A unit test recomputes the digest of every embedded asset and fails on a mismatch. Updating an asset is a task that updates the version directory, the manifest, the notices, and re-runs the browser test suite. There is no automatic updater, no lockfile fetch, and no network in the build.

5. **Licence notices ship in the package.** `packaging/deb/usr/share/doc/papyrus/copyright` gains one section per bundled asset (name, version, licence identifier, full licence text or a `debian/copyright`-style reference to the on-disk text), in addition to the existing Papyrus GPL-3.0-or-later statement. The package keeps exactly one documentation file so `scripts/build-deb.sh` and the packaging verification tests keep their shape. The licence inventory is a release gate: an asset whose licence cannot be shown compatible with GPLv3-or-later (NFR-06.002) does not ship.

   Current inventory to be recorded: KaTeX 0.16.11 — code MIT, fonts SIL OFL-1.1; Mermaid 11.x — MIT.

6. **Weight budget and load discipline.** The release check asserts the release binary stays within **12 MiB** (current 4.2 MiB + ≈4.0 MiB of assets + headroom). Assets are emitted only on pages that need them: KaTeX CSS/JS/fonts only where math was found, Mermaid only where a Mermaid fence was found. No asset is referenced from a page that does not use it.

7. **ADR-0005's asset clause is amended, not replaced.** The transport, routing, templating, escaping, and security model of ADR-0005 remain in force. This ADR supersedes only these parts of it: "Embed CSS and JavaScript with `include_str!`" widens to the binary-safe asset table; "no web font" widens to bundled same-origin fonts; "the `.deb` contains only `/usr/bin/papyrus` and the copyright file" stays true but the copyright file now carries third-party sections. On acceptance, ADR-0005's Status line gains an amendment note pointing here; its frozen body is not edited.

8. **`Cache-Control: no-store` is retained.** No new cache semantics, no `ETag`, no compression, no range requests. A page with diagrams therefore re-sends the Mermaid bundle on each navigation. This is accepted for a loopback single-user interface; serving vendor assets with a long-lived cache is a separate decision and would need its own ADR because it changes ADR-0005's header contract.

9. **No source maps, no minified references to remote origins.** `.map` files are not embedded. A test asserts that no served asset is referenced from a page outside `/assets/`, that no served CSS contains `@import`, and that the only `url(` values in served CSS are same-origin font paths.

10. **Existing offline-reference tests are retargeted, not deleted.** The `t24/t40l/t25l/t26k/t57k/t81l`-style "no `url(`, no web font" assertions scanned `app.css`; they keep scanning `app.css` and gain a sibling assertion that every `url(` in every served stylesheet resolves to an entry of the asset table.

## Consequences

**Positive:**

- One mechanism replaces two hard-coded asset routes, and adding an asset stops being a code change in the router.
- Binary bodies, fonts, and MIME types are supported without a dependency or a static-file server.
- The supply chain is explicit: versioned directories, upstream URLs, digests, licences, and a documented update task.
- The `.deb` stays a two-file package with one copyright document that satisfies Debian's notice requirement.
- The build stays `cargo build`, offline and deterministic.

**Negative:**

- ≈4.0 MiB is embedded unconditionally in the binary even though most pages use none of it; only the *transfer* is conditional. The binary grows to ≈8.2 MiB and the `.deb` to ≈2.8 MiB.
- Vendored pre-built JavaScript cannot be reviewed line by line by this project; trust rests on the upstream project, the pinned version, and the digest.
- Every asset update is a manual, multi-file task (version directory, manifest, notices, tests).
- `no-store` on a 3.4 MiB bundle is a real user-visible cost on diagram-heavy navigation.
- The notice file becomes the place where a licence mistake ships to users; it needs review, not just editing.

**Risks and mitigations:**

- *Digest drift or a silently edited vendored file*: the manifest digest test fails the build.
- *A bundled licence is incompatible or a notice is missing*: the release gate blocks; the inventory is short (two packages) and reviewed per update.
- *Package-content test breaks*: the packaging test asserts the two-file docs set explicitly and is updated in the same change as `build-deb.sh`.
- *Binary size regression*: 12 MiB budget asserted in the release check.
- *A vendored bundle references a remote origin and is only caught in production*: CSP `default-src 'none'` blocks it at runtime, and the served-asset reference test catches it at build time.

## Alternatives Considered

| Option | Outcome | Reason |
|---|---|---|
| Static asset table with `include_bytes!`, vendored versioned files (this ADR) | Selected | Keeps the single binary and `cargo build`, adds binary bodies and fonts, and makes the supply chain auditable. |
| Ship assets in `/usr/share/papyrus/assets` and read them at runtime | Rejected | Breaks single-binary packaging (ADR-0002/ADR-0005), adds a runtime path and a traversal surface, and makes "installed but unreadable" a new failure mode. |
| A Rust asset-embedding crate (`include_dir`, `rust-embed`) | Rejected | A new dependency for something 40 lines of `include_bytes!` already do; ADR-0005's no-new-dependency rule holds. |
| Base64-inline the assets into the HTML | Rejected | +33 % size, no caching, template blow-up, CSP `style-src`/`data:` complications. |
| Rebuild the assets from source with npm in the build | Rejected | Introduces a Node toolchain and network access into `cargo build`; violates ADR-0001/ADR-0004 and offline determinism. |
| Pre-compress the embedded assets and serve `Content-Encoding: gzip` | Rejected for now | ADR-0005 rejects compression handling; it would add `Accept-Encoding` negotiation and a compressed-body path to the hand-written HTTP server. A later ADR may revisit it as the single biggest size win (≈1.5 MiB instead of 4 MiB transferred). |
| Separate `THIRD-PARTY-NOTICES` file in the package | Rejected | Debian requires the licence inventory in `copyright` anyway; two files add a packaging step and a place for notices to go stale. |

## Open Questions

- **Approve the 12 MiB binary budget** (current 4.2 MiB)? Recommended: yes; a tighter budget forces choosing between Mermaid and a single-file diagram subset.
- **Should vendor assets get a long-lived cache header?** Current decision: no, keep `no-store`. Recommended follow-up only if diagram-heavy navigation proves painful.
- **Is a pinned Mermaid minor version enough, or is a documented "review on security advisory" cadence required?** Recommended: a release-checklist item, not an automated mechanism.

## Downstream constraints

1. No module may serve a file from disk for the browser; every byte comes from `ASSETS`.
2. Asset paths are exact strings; no prefix matching, directory listing, range request, or path join is added.
3. Every embedded asset has a manifest entry with digest and licence, and the digest test passes.
4. The package contains no runtime asset directory; the docs set stays one copyright file.
5. Vendored files are byte-identical to upstream unless a documented `.patch` accompanies them.
6. `Cargo.toml` gains no dependency; the build gains no toolchain step and no network access.
7. Existing "no remote reference" tests keep passing, retargeted as described in decision 10; specification wording about "no web font" is revised by the business analyst / tech writer.

## References

- `docs/requirements/non-functional/NFR-01.003.md`, `NFR-03.001.md`, `NFR-05.001.md`, `NFR-06.001.md`, `NFR-06.002.md`.
- `docs/requirements/business/REQ-06.017.md` — bundled, offline rendering assets.
- `docs/architecture/adr/ADR-0001-implementation-language-rust.md`, `ADR-0004-repository-root-single-crate-layout.md`.
- `docs/architecture/adr/ADR-0002-distribution-format-amd64-deb-packages.md`, `ADR-0003-supported-debian-release-target-forky-only.md` — one binary per package.
- `docs/architecture/adr/ADR-0005-embedded-dependency-free-http-server-and-server-rendered-ui.md` — asset, transport, CSP, and header baseline (partially amended here).
- `docs/architecture/adr/ADR-0025-bundled-katex-client-side-math-rendering.md`, `ADR-0026-mermaid-only-offline-diagram-rendering.md` — the consumers of these assets.
- Implementation: `src/browser/assets.rs` (new), `src/browser/vendor/`, `src/browser/http.rs`, `src/browser/mod.rs`, `scripts/build-deb.sh`, `packaging/deb/usr/share/doc/papyrus/copyright`.
- Tests: `tests/browser_cli.rs` (offline asset scans, CSP, packaging), `tests/search_browser_smoke.rs`.
