# Papyrus Build, Packaging, and Installation

## Supported platforms

- Architecture: `amd64`.
- Supported Debian release: **Debian Forky only** (per ADR-0003, which supersedes
  ADR-0002, and NFR-03.002).
- Debian Trixie and any other Debian release are **not claimed or verified** as
  supported. Adding a release to the supported set requires an explicit verification
  pass and a follow-up ADR/requirement update (ADR-0003, NFR-03.002).
- The package declares `Depends: libc6 (>= 2.34)`; the verified environment is Forky
  with glibc 2.43.

## Prerequisites

- Rust 1.99.1 via rustup (pinned by `rust-toolchain.toml`), including `rustfmt` and
  `clippy`.
- Cargo 1.99.1.
- `dpkg-deb` and `dpkg` (Debian package tooling; verified with dpkg 1.23.11).
- `fakeroot` for the isolated install/uninstall check.
- `unshare` and `ip` for the offline verification inside a network namespace
  (`unshare -rn` must be available; no root required).
- `curl` and `jq` for the browser and JSON smoke checks.

## Build

```sh
cargo build --release
```

The binary is produced at `target/release/papyrus`.

Run the test suite:

```sh
cargo test
```

Lint and format:

```sh
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

## Run (uninstalled)

```sh
./target/release/papyrus --help
./target/release/papyrus -h
./target/release/papyrus --version
```

Expected:

- `--help` / `-h`: print help and exit `0`, listing `search` and `browser`.
- `--version`: print `papyrus <version>` and exit `0`.
- unknown flags/arguments: print usage to stderr and exit `2`.

## Packaging (Release 1.0, 2026-11-23)

`scripts/build-package.sh` is the packaging entry point. It builds the release binary and calls `scripts/build-deb.sh`, the Debian assembly step.

```sh
scripts/build-package.sh --version 1.0.0 --out builds/1.0.0/
```

- `--version` is authoritative for the `Version:` field in `DEBIAN/control` and the `papyrus_<version>_amd64.deb` artifact name. The version is injected while the package is staged; no tracked file is edited. Release 1.0 pins `Cargo.toml` to `1.0.0`, and the supplied `--version` matches it.
- The entry point runs `cargo build --release --locked`, validates the asset versions through the assembly step, and writes the package only under `--out`.
- `--out` must be absent or empty. With `--force`, the contents of a non-empty directory are replaced.
- Success prints the artifact path on standard output; diagnostics go to standard error.

### Output locations and the `--out` policy

- `builds/<version>/` is the only publish location. It is git-ignored and is never staged or committed.
- A release candidate is a discardable verification build in `builds/<version>/`; the release retains exactly one artifact in that directory.
- `./.tmp/` is scratch only and is removed before the work ends.
- The build scripts refuse, with exit `2`: the repository root; `builds/` without a version segment after it; any other path inside the repository; the filesystem root; system directories; `$HOME` and its subtree (except the two permitted in-repository roots); and any parent of the repository root.

### Assembly only

`scripts/build-deb.sh` can be run directly to assemble a package:

```sh
scripts/build-deb.sh [--version <version>] [--out <dir>] [--force]
```

- Version: `--version`, then `PAPYRUS_DEB_VERSION`, then the `Cargo.toml` version.
- Output: `--out`, then `PAPYRUS_DEB_OUT`, then the default `./.tmp/deb/`.
- Every assembly run rebuilds with `cargo build --release --locked` before staging, so a direct run can never package a stale `target/release/papyrus` (AUD-153 L-4).
- The output-emptiness policy also applies: `--out` must be absent or empty, and `--force` replaces a non-empty directory (AUD-153 L-4). The default `./.tmp/deb/` therefore needs `--force` on a second run.
- `--check-assets-only` verifies the asset versions and exits without assembling.

### Asset-version guard

The assembly step verifies every vendored package before staging:

- The vendored tree `src/browser/vendor/<package>/` must hold exactly one version directory; that directory name is the asset version.
- The manifest `src/browser/vendor/MANIFEST.md` must have a row for `/assets/<package>/` whose `version` column matches the directory version.
- `packaging/deb/usr/share/doc/papyrus/copyright` must cite the same version for the package.

Any mismatch, a missing version directory, a missing manifest row, or a missing citation refuses the build with exit `1`. The check runs on every assembly path, not only with the opt-in flag, and the asset version is independent of the Papyrus package version.

### Package inventory

`dpkg-deb -c` lists exactly two files, plus their directories:

- `/usr/bin/papyrus`, mode `0755`.
- `/usr/share/doc/papyrus/copyright`, mode `0644`.

No man page is packaged. No asset, template, configuration, or other documentation file is packaged: browser assets are embedded at compile time and served from the static asset table, and document types are seed descriptors.

### Inspect the package

```sh
dpkg-deb -I builds/1.0.0/papyrus_1.0.0_amd64.deb
dpkg-deb -c builds/1.0.0/papyrus_1.0.0_amd64.deb
dpkg-deb --fsys-tarfile builds/1.0.0/papyrus_1.0.0_amd64.deb \
  | tar -xO ./usr/share/doc/papyrus/copyright | grep GPL-3.0-or-later
```

Expected metadata: `Package: papyrus`, `Version: <requested version>`, `Architecture: amd64`, `Section: utils`, `Priority: optional`, `Depends: libc6 (>= 2.34)`, `Maintainer`, `Homepage`, and a `Description` naming Papyrus as a project-level backlog and documentation manager. The copyright file declares `GPL-3.0-or-later` (matching `Cargo.toml`) and one section per bundled asset family: KaTeX code MIT, KaTeX fonts OFL-1.1, and Mermaid MIT.

## Install

```sh
sudo dpkg -i builds/<version>/papyrus_<version>_amd64.deb
```

After installation, `papyrus` is available on `PATH` (installed to
`/usr/bin/papyrus`).

### Isolated install (verification only)

Never install the verification package on the host. Use an isolated dpkg database
and install root under the project-local `./.tmp/` directory:

```sh
export PATH="/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"
ROOT="$PWD/.tmp"
mkdir -p "$ROOT/inst" "$ROOT/adm/info" "$ROOT/adm/updates" "$ROOT/adm/parts" "$ROOT/adm/triggers"
: > "$ROOT/adm/status"
: > "$ROOT/adm/available"

fakeroot dpkg --instdir="$ROOT/inst" --admindir="$ROOT/adm" --force-depends \
  -i builds/<version>/papyrus_<version>_amd64.deb
```

- The full `PATH` matters: dpkg needs `ldconfig` and `start-stop-daemon` from
  `/usr/sbin` and `/sbin`.
- `--force-depends` is required because the empty isolated database has no `libc6`
  entry; the host Forky glibc satisfies the runtime dependency.
- dpkg may warn that `/var/log/dpkg.log` is not writable. This is expected and
  confirms the host log is untouched.
- Verify the host was not modified: `dpkg-query -W -f='${Status}' papyrus` must
  report `no packages found matching papyrus` before and after the run.

Checks after install:

```sh
fakeroot stat -c '%U:%G %a %n' "$ROOT/inst/usr/bin/papyrus"   # root:root 755
env PATH="$ROOT/inst/usr/bin:$PATH" sh -c 'command -v papyrus; papyrus --version'
```

## Uninstall

```sh
sudo dpkg -r papyrus
# or, to also remove configuration files:
sudo dpkg --purge papyrus
```

Isolated uninstall:

```sh
export PATH="/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"
ROOT="$PWD/.tmp"
fakeroot dpkg --instdir="$ROOT/inst" --admindir="$ROOT/adm" -r papyrus
env PATH="$ROOT/inst/usr/bin:$PATH" sh -c 'command -v papyrus || echo "not on PATH"'
# or, from an installed state:
fakeroot dpkg --instdir="$ROOT/inst" --admindir="$ROOT/adm" --purge papyrus
rm -rf "$ROOT/inst" "$ROOT/adm"
```

## Smoke test (installed)

Run every command against the installed binary (`$ROOT/inst/usr/bin/papyrus` on
`PATH`) with `--dir` pointing at a scratch project under `./.tmp/`. Exit statuses
follow the existing contract: `0` success, `1` IO/boundary, `2` usage, `3` not found,
`4` validation.

> **Historical checklist.** The long smoke checklist from here through the recorded
> milestone sections predates the Release 1.0 document surface and command changes:
> per-type document commands, `deprecate`/`restore`, `nfreq`, and the generic document
> model. It is retained as verification history, not as a current procedure. For
> current behaviour and migration, use the
> [Release 1.0 upgrade notes](release-1.0-upgrade-notes.md).

### 3.1 CLI baseline and requirement lifecycle

```sh
papyrus --help          # exit 0, lists search and browser
papyrus --version       # exit 0, prints "papyrus <version>"
papyrus --dir <project> init <name> --no-git   # creates papyrus.config.yml + structure
papyrus req create-group --title <title>       # prints REQ-<n>
papyrus req create <title> --group <id>        # prints <group>.<seq>
papyrus req list        # active records by id/title, id order, no deprecated
papyrus req view <id>   # full persisted text; unknown id exits 3
papyrus req edit <id> --title <new>            # updates only supplied fields
papyrus req deprecate <id>                     # moves to deprecated/, hidden from list
papyrus req restore <id>                       # returns with the same id
# same lifecycle for nfreq (create-group, create --group, list, view, edit,
# deprecate, restore)
```

### 3.2 Search

```sh
papyrus search "<contiguous phrase>"   # prints "<id> doc <title>" lines, id order
papyrus search "<words split apart>"   # no phrase match
papyrus search <term> --type doc       # requirement records only
papyrus search <term> --type task      # empty output, exit 0
papyrus search <term> --limit 1        # at most one result line
papyrus --json search <term>           # [{id,type,title,path}, ...]; [] when empty
# a deprecated record is absent and reappears after `req restore`
# usage errors (missing query, unknown --type, --limit 0) exit 2;
# empty/control-character query exits 4; symlink escape exits 1
```

### 3.3 Browser

```sh
papyrus --dir <project> browser --no-open --port <free-port>
# prints exactly: Serving http://127.0.0.1:<free-port>/
```

Then check, over loopback:

- `GET /` and `GET /?type=req` render business requirements; `GET /?type=nfr`
  renders non-functional requirements. Each page renders exactly one category and
  the selected sidebar entry carries `aria-current="page"`.
- `GET /doc/<id>` renders the record view (id, form, document type, title, body).
- Edit round-trip: `GET /doc/<id>/edit` yields the form with its hidden `version`
  token; `POST /doc/<id>` with the documented fields returns `303` with
  `Location: /doc/<id>`; the follow-up `GET` and `papyrus req view <id>` show the
  persisted change.
- `GET /api/requirements?type=req|nfr` and `GET /api/doc/<id>` return the documented
  JSON objects.
- Representative assets return `200` with `Cache-Control: no-store`, non-empty
  bodies, and no CSP header: `/assets/app.css` (`text/css`), `/assets/app.js`
  (`text/javascript`), `/assets/nav-state.js` (`text/javascript`),
  `/assets/katex/katex.min.css` (`text/css`), `/assets/katex/katex.min.js`
  (`text/javascript`), one `/assets/katex/fonts/KaTeX_Main-Regular.woff2`
  (`font/woff2`), and `/assets/mermaid/mermaid.min.js` (`text/javascript`). A
  miss is `404`; a `POST` is `405`.
- 320 px markup hooks: viewport meta; the header `.nav-toggle` present on every
  page with `aria-expanded` and `aria-controls="site-nav"`, defaulting to
  expanded and persisting in `papyrus.nav`; the shared list and page layouts
  introduce no page-level horizontal scrolling at 320, 480, and 720 CSS px.
- Offline rendering and CSP: served HTML/CSS/JS contain no `http://`, `https://`,
  protocol-relative `//`, remote `@import`, remote `url(...)`, `<img>`, or CDN
  reference. Every `url(...)` in every served stylesheet resolves to an entry of
  the asset table, and the only such values are KaTeX's 60 `url(fonts/…)`
  entries; a math page renders offline with its WOFF2 fonts served from
  `/assets/katex/fonts/…` under `Cache-Control: no-store`. Every `text/html`
  response carries exactly one of the two CSP strings fixed in
  `browser-requirements.md` §9.7: strict when `has_diagram` is false, the diagram
  string (only `style-src 'self' 'unsafe-inline'`) when it is true. `/assets/` and
  non-HTML responses carry no CSP; `script-src` never gains `'unsafe-inline'`,
  `'unsafe-eval'`, a nonce, or a hash.
- A missing or empty document type directory renders its empty state (no server
  error).

### 3.4 Offline operation

```sh
unshare -rn env ROOT="$PWD/.tmp" INST="$PWD/.tmp/inst" PROJ="$PWD/.tmp/smoke/project" \
  bash -c 'ip link set lo up; ...'
```

Inside the namespace, prove an outbound TCP connection fails, then repeat the search
and browser view/asset checks over `127.0.0.1`. The browser binds an IP literal, so
no name resolution is required. If `unshare -rn` is unavailable, the offline claim
is supported by loopback-only binding, no external references in served content, and
no outbound-connection code path; record that limitation explicitly.

### 3.5 Browser configuration keys

```sh
papyrus config list                 # includes browser_host, browser_port, browser_open
papyrus config get browser_host     # 127.0.0.1
papyrus config get browser_port     # 6420
papyrus config get browser_open     # true
papyrus config set browser_port <free-port>
# browser without --port uses the configured port; --port overrides it
papyrus config set browser_open false   # suppresses opening without --no-open
papyrus config unset browser_open       # restores true
papyrus config unset browser_port       # restores 6420
# config set browser_port 0 | 70000, config set browser_open maybe
# -> exit 4 and the configuration file is unchanged
```

### 3.6 Embedded assets, digests, notices, and the runtime file dependency

Run the installed binary with its working directory set to an empty directory and a
project directory that contains no asset files. `search` and `browser` must still
work, and the representative `/assets/` paths in §3.3 must be non-empty. The
package contains no asset, template, or documentation files beyond the copyright
file, so the installed binary needs only `/usr/bin/papyrus` and the project
directory.

Asset verification:

1. **Digests.** `cargo test` includes the manifest test, which asserts that the
   manifest row set equals `ASSETS[].path` with no duplicate or missing row and
   recomputes SHA-256 and byte length for every entry. A missing row, an extra
   row, or a byte/digest mismatch fails the build.
2. **Notices.** `dpkg-deb --fsys-tarfile papyrus_<version>_amd64.deb | tar -xO
   ./usr/share/doc/papyrus/copyright` shows the GPL-3.0-or-later block plus one
   section per bundled family (KaTeX MIT, KaTeX fonts OFL-1.1, Mermaid MIT), and
   `dpkg-deb -c` still lists exactly the binary and the copyright file.
3. **Offline rendering.** The §3.3 offline/CSP check passes for a page with math
   and for a page with a Mermaid figure.
4. **Budget.** Measured on 2026-11-16: binary `stat -c %s` = 8,321,888 B
   (7.94 MiB); served KaTeX assets = 558,541 B (CSS 23,335 + JS 275,414 + 20
   WOFF2 259,792, from `src/browser/vendor/MANIFEST.md`); Mermaid =
   3,572,661 B; the pre-Mermaid projection recorded by TASK-148 was 8,302,957 B
   (7.92 MiB). The ceiling is 12 MiB = 12,582,912 B, leaving 4,261,024 B
   (4.06 MiB) of headroom on the measured binary. This is a recorded release
   check; the executable size must remain within the budget, measured with
   `stat -c %s target/release/papyrus`.

## Smoke test results (TASK-4 verification)

Environment: isolated `dpkg --instdir/--admindir` under `fakeroot` on the build host
(Debian package tooling 1.23.11; Rust 1.99.1). Host not modified.

| Check | Result |
|---|---|
| `dpkg -i` (isolated) | PASS |
| `papyrus` on PATH -> `/usr/bin/papyrus` | PASS |
| `papyrus --help` exit 0 | PASS |
| `papyrus --version` -> `papyrus 0.1.0` exit 0 | PASS |
| `init` creates config + structure | PASS |
| `req create-group` prints `REQ-001` | PASS |
| `req create ... --group` prints `REQ-001.001` | PASS |
| `req list` shows active records in id order | PASS |
| `req view <id>` full text; unknown id exit 3 | PASS |
| `req edit <id>` updates supplied field only | PASS |
| `req deprecate <id>` hides from list | PASS |
| `req restore <id>` same id reappears | PASS |
| `nfreq` lifecycle (group/create/list/view/edit/deprecate/restore) | PASS |
| exit code matrix (2/3/4/1) | PASS |
| `dpkg --purge` removes binary; not on PATH | PASS |

`--editor` smoke test is environment-dependent and was verified by code review /
`tests/req_cli.rs::req_editor_body_round_trip` rather than in the isolated install.

## Verification results (TASK-7, search and browser milestone)

- Date: **2026-11-03 (UTC)**.
- Environment: Debian GNU/Linux forky/sid amd64, glibc 2.43-7, dpkg 1.23.11,
  `rustc`/`cargo` 1.99.1, clippy 0.1.99, `fakeroot`, and `unshare -rn` (loopback
  brought up with `ip link set lo up`).
- Isolation: `fakeroot dpkg --instdir=./.tmp/inst --admindir=./.tmp/adm
  --force-depends`; the host reported no installed `papyrus` package before and
  after the run. Scratch directories were removed after verification.
- Package: `papyrus_0.1.0_amd64.deb`, 624048 bytes,
  SHA-256 `4b2c9e8799016a1f98168f4c373100b3efd2f03e93036e648e2e756951dac697`.
- Package metadata: `Package: papyrus`, `Version: 0.1.0`, `Architecture: amd64`,
  `Depends: libc6 (>= 2.34)`, `Section: utils`, `Priority: optional`, copyright
  `GPL-3.0-or-later`.
- Package contents: `/usr/bin/papyrus` (0755, 1688240 bytes) and
  `/usr/share/doc/papyrus/copyright` (0644, 865 bytes) only; no asset, template,
  configuration, or man files.
- Runtime dependency license audit: unchanged from TASK-1/TASK-4; TASK-5, TASK-6,
  and TASK-7 added no runtime dependency.

### Quality gates

| Gate | Result |
|---|---|
| `cargo build --release` (Rust 1.99.1) | PASS |
| `target/release/papyrus --help` lists `search` and `browser` | PASS |
| `cargo test` | PASS — 133 tests (33 unit + 41 `browser_cli` + 20 `cli` + 5 `milestone_smoke` + 11 `req_cli` + 2 `search_browser_smoke` + 21 `search_cli`) |
| `cargo clippy --all-targets -- -D warnings` | PASS — 0 warnings |
| `cargo fmt --check` | PASS |

### Install, uninstall, and smoke checks

| Check | Result |
|---|---|
| Isolated `dpkg -i` | PASS (exit 0) |
| Installed `/usr/bin/papyrus` is `root:root` 0755 | PASS |
| `papyrus` resolves on the isolated `PATH` and runs | PASS |
| Isolated `dpkg -r papyrus` removes the binary; not on `PATH` | PASS |
| Isolated `dpkg --purge papyrus` from installed state removes binary and admin entry | PASS |
| Host package database and filesystem unchanged | PASS |
| `--help` lists `search` and `browser`; `--version` prints `papyrus 0.1.0` | PASS |
| `init`, req/nfreq lifecycle, ids preserved across deprecate/restore | PASS |
| Search: quoted phrase, split words, `--type doc`, `--type task`, `--limit`, `--json`, `[]`, deprecation visibility | PASS |
| Search usage/validation exits (2/4) | PASS |
| Browser: startup line, both categories one at a time, `aria-current`, view | PASS |
| Browser edit round-trip: `303`, `Location`, updated view and CLI view | PASS |
| Browser JSON API: `/api/requirements`, `/api/doc/<id>` | PASS |
| Embedded assets: `/assets/app.css`, `/assets/app.js`, non-empty, `no-store` | PASS |
| 320 px markup hooks and narrow-viewport CSS | PASS |
| No external references in served HTML/CSS/JS; CSP present | PASS |
| Offline (`unshare -rn`, outbound TCP fails): search, view, assets | PASS |
| Config keys: list/get/set/unset, configured port honored, `--port` override, defaults restored, invalid values exit 4 with unchanged file | PASS |
| Embedded assets from an empty working directory and asset-free project | PASS |

### Recorded limitations

- **OQ-1 — no real-GUI browser run.** This headless verification checks served
  markup, asset set, viewport hooks, and the absence of forbidden features. Firefox,
  Edge, and Safari 100+ were not driven in a GUI. NFR-05.001 browser support remains
  covered by TASK-6 (`tests/browser_cli.rs`) and the approved MOCK-0001; schedule a
  manual GUI pass if required.
- **Forky only.** No Debian Trixie or other release was installed or verified, and
  none is claimed (ADR-0003, NFR-03.002).
- **Packaging script precondition.** `scripts/build-deb.sh` was intentionally left
  unchanged (TASK-7 scope). The stale-binary hazard is mitigated by the mandatory
  explicit `cargo build --release` plus the `--help` assertion documented above and
  executed before packaging.

## Smoke test results (TASK-12 verification, backlog-discovery milestone)

- Date: **2026-11-04 (UTC)**.
- Environment: Debian GNU/Linux forky/sid amd64, glibc 2.43, dpkg 1.23.11,
  `rustc`/`cargo` 1.99.1, clippy 0.1.99, `fakeroot`, and `unshare -rn`
  (loopback brought up with `ip link set lo up`).
- Isolation: `fakeroot dpkg --instdir=./.tmp/inst --admindir=./.tmp/adm
  --force-depends`; the host package status was unchanged before and after the
  run (the host already reported `papyrus 0.1.0` installed from prior work and
  was not modified by this verification). Scratch directories were removed after
  verification.
- Package: `papyrus_0.1.0_amd64.deb`, 735840 bytes,
  SHA-256 `69a2152470fbec6a1f110fabad28ee2b5c40229a02c61bac7bd6bb49f422d3db`.
- Package metadata: `Package: papyrus`, `Version: 0.1.0`, `Architecture: amd64`,
  `Depends: libc6 (>= 2.34)`, `Section: utils`, `Priority: optional`, copyright
  `GPL-3.0-or-later`.
- Package contents: `/usr/bin/papyrus` (0755, 2112928 bytes) and
  `/usr/share/doc/papyrus/copyright` (0644, 865 bytes) only; no asset, template,
  configuration, or man files.
- New test file: `tests/backlog_discovery_smoke.rs` (2 tests), passing under
  `cargo test`.

### Quality gates

| Gate | Result |
|---|---|
| `cargo build --release` (Rust 1.99.1) | PASS |
| `target/release/papyrus --help` lists `task`, `milestone`, `release`, `search`, `browser` | PASS |
| `cargo test` | PASS — 160 tests (33 unit + integration suites, including the new `backlog_discovery_smoke` 2 tests) |
| `cargo clippy --all-targets -- -D warnings` | PASS — 0 warnings |
| `cargo fmt --check` | PASS |

### Install, uninstall, and smoke checks

| Check | Result |
|---|---|
| Isolated `dpkg -i` | PASS (exit 0) |
| Installed `/usr/bin/papyrus` is `root:root` 0755 | PASS |
| `papyrus` resolves on the isolated `PATH` and runs | PASS |
| Isolated `dpkg -r papyrus` removes the binary; not on `PATH` | PASS |
| Isolated `dpkg --purge papyrus` from installed state removes binary and admin entry | PASS |
| Host package database and filesystem unchanged | PASS |
| `--help` lists `task`, `milestone`, `release`, `search`, `browser`; `--version` prints `papyrus 0.1.0` | PASS |
| `init`, release/milestone/task lifecycles, ids preserved across archive/restore | PASS |
| Search: quoted phrase, split words, `--type task/milestone/release`, `--limit`, `--json`, `[]`, archival visibility | PASS |
| Browser: startup line, all five categories, `aria-current`, record views | PASS |
| Browser edit round-trip: `303`, `Location`, updated view and CLI view | PASS |
| Browser JSON API: `/api/tasks`, `/api/milestones`, `/api/releases`, `/api/doc/<id>` | PASS |
| Embedded assets: `/assets/app.css`, `/assets/app.js`, non-empty, `no-store` | PASS |
| 320 px markup hooks and narrow-viewport CSS | PASS |
| No external references in served HTML/CSS/JS; CSP present | PASS |
| Offline (`unshare -rn`, outbound TCP fails): search over loopback | PASS |
| Config keys: list/get/set/unset, configured port honored, `--port` override, defaults restored, invalid values exit 4 with unchanged file | PASS |
| Embedded assets from an empty working directory and asset-free project | PASS |

### Recorded limitations

- **OQ-1 — no real-GUI browser run.** This headless verification checks served
  markup, asset set, viewport hooks, and the absence of forbidden features. Firefox,
  Edge, and Safari 100+ were not driven in a GUI. NFR-05.001 browser support remains
  covered by TASK-6/TASK-11 (`tests/browser_cli.rs`) and the approved MOCK-0001/
  MOCK-0002; schedule a manual GUI pass if required.
- **Forky only.** No Debian Trixie or other release was installed or verified, and
  none is claimed (ADR-0003, NFR-03.002).
- **Host baseline.** The build host already reported `papyrus 0.1.0` installed from
  prior work; the isolated verification did not modify it. The host entry is recorded
  as unchanged rather than absent.
- **Packaging script precondition.** `scripts/build-deb.sh` was intentionally left
  unchanged. The stale-binary hazard is mitigated by the mandatory explicit
  `cargo build --release` plus the `--help` assertion executed before packaging.

## Runtime dependency license audit

Reference: TASK-1 license audit (unchanged). TASK-2 to TASK-12 introduced no new
runtime dependencies beyond those already audited. Audit re-run not required.

Scope: runtime dependencies only (crates linked into the distributed binary).
Build-time/proc-macro dependencies are excluded by NFR-06.002 scope.

| Crate | Version | License (SPDX) | Verdict |
|---|---|---|---|
| clap | 4.6.7 | MIT OR Apache-2.0 | Compatible |
| clap_builder | 4.6.7 | MIT OR Apache-2.0 | Compatible |
| clap_lex | 1.1.1 | MIT OR Apache-2.0 | Compatible |
| anstream | 1.0.0 | MIT OR Apache-2.0 | Compatible |
| anstyle | 1.0.14 | MIT OR Apache-2.0 | Compatible |
| anstyle-parse | 1.0.0 | MIT OR Apache-2.0 | Compatible |
| anstyle-query | 1.1.5 | MIT OR Apache-2.0 | Compatible |
| colorchoice | 1.0.5 | MIT OR Apache-2.0 | Compatible |
| is_terminal_polyfill | 1.70.2 | MIT OR Apache-2.0 | Compatible |
| strsim | 0.11.1 | MIT | Compatible |
| utf8parse | 0.2.2 | Apache-2.0 OR MIT | Compatible |
| serde | 1.0.229 | MIT OR Apache-2.0 | Compatible |
| serde_core | 1.0.229 | MIT OR Apache-2.0 | Compatible |
| serde_json | 1.0.154 | MIT OR Apache-2.0 | Compatible |
| yaml_serde | 0.10.7 | MIT OR Apache-2.0 | Compatible |
| libyaml-rs | 0.3.0 | MIT | Compatible |
| indexmap | 2.14.2 | Apache-2.0 OR MIT | Compatible |
| itoa | 1.0.18 | MIT OR Apache-2.0 | Compatible |
| memchr | 2.8.3 | Unlicense OR MIT | Compatible |
| ryu | 1.0.23 | Apache-2.0 OR BSL-1.0 | Compatible |
| zmij | 1.0.23 | MIT | Compatible |

All runtime dependencies are permissively licensed (MIT / Apache-2.0 / Unlicense /
BSL-1.0) and permit linking in a GPL-3.0-or-later application. No incompatible
runtime dependency remains.

Browser assets are not crates: their licences are recorded in
`src/browser/vendor/MANIFEST.md` and in the package `copyright` file, are
compatible with GPL-3.0-or-later (MIT, OFL-1.1), and the NFR-06.002 release gate
is unchanged.

Build-time/proc-macro dependencies (excluded from audit scope): clap_derive, heck,
proc-macro2, quote, syn, unicode-ident.

## Decision-record milestone verification (TASK-16, 2026-11-05)

Verified on Debian Forky (forky/sid), glibc 2.43, dpkg 1.23.11, Rust 1.99.1,
with `papyrus 0.1.0`.

- `cargo build --release`: success.
- `cargo test`: all suites pass (including `decision_records_smoke` 3/3).
- `cargo clippy --all-targets -- -D warnings`: clean.
- `cargo fmt --check`: clean.
- `scripts/build-deb.sh`: produced `papyrus_0.1.0_amd64.deb`.
- Package metadata: `Package: papyrus`, `Version: 0.1.0`, `Architecture: amd64`,
  `Depends: libc6 (>= 2.34)`, Section utils, Priority optional, GPL-3.0-or-later
  copyright. Contents: `/usr/bin/papyrus` (0755) and `/usr/share/doc/papyrus/copyright`
  only; no runtime asset files.
- Isolated install/uninstall via `fakeroot dpkg` with instdir/admindir under
  `./.tmp/` succeeded; host package database untouched; purge removed all files.
- Smoke tests: decision lifecycle (create/list/view/edit/archive), search over
  decision records (default and `--type decision`, `--limit`), browser decision
  category (`/?type=decision`, `/doc/ADR-001`, JSON API `/api/decisions`), offline
  operation over loopback, and browser configuration keys all passed.

## Verification results (TASK-62, CLI milestone)

- Date: 2026-11-06 (UTC).
- Environment: Debian GNU/Linux forky/sid amd64, glibc 2.43, dpkg 1.23.11, cargo 1.99.1, clippy 0.1.99, fakeroot.
- Isolation: fakeroot dpkg --instdir=./.tmp/inst --admindir=./.tmp/adm --force-depends; host package status unchanged.
- Package: papyrus_0.1.0_amd64.deb built with cargo build --release then scripts/build-deb.sh.
- Package contents: /usr/bin/papyrus (0755) and /usr/share/doc/papyrus/copyright only.
- New test file: tests/cli_milestone_smoke.rs (2 tests), passing under cargo test.

### Quality gates

| Gate | Result |
|---|---|
| cargo build --release | PASS |
| cargo test | PASS — 254 tests (36 unit + 218 integration) |
| cargo clippy --all-targets -- -D warnings | PASS — 0 warnings |
| cargo fmt --check | PASS |

### Install, uninstall, and smoke checks

| Check | Result |
|---|---|
| Isolated dpkg -i | PASS (exit 0) |
| Installed /usr/bin/papyrus root:root 0755 | PASS |
| Isolated dpkg -r removes binary | PASS |
| Isolated dpkg --purge | PASS |
| Host package database unchanged | PASS |
| --help lists glossary, ui-spec, ui-mockup, rule, audit, tech-spec | PASS |
| --version prints papyrus 0.1.0 | PASS |
| m-4 management lifecycles (create/list/view/edit/deprecate/restore) | PASS |
| Disabled document types rejected | PASS |
| Search regression over req/task/milestone/release/decision | PASS |
| Browser regression over req/nfr/task/milestone/release/decision | PASS |
| Offline (unshare -rn) search/browser over loopback | PASS |
| Config keys list/get/set/unset | PASS |
| Embedded assets from empty working directory | PASS |

### Recorded limitations

- OQ-1: no real-GUI browser run; markup-level verification only (NFR-05.001).
- Forky only; no other Debian release verified (ADR-0003, NFR-03.002).
- Packaging script does not force rebuild; mitigated by mandatory cargo build --release before packaging.

### TASK-62 package artifact

- File: `papyrus_0.1.0_amd64.deb`
- Size: 858736 bytes
- SHA-256: `e0d9da5f25f717d65dd194f68cba9ed0d487206bf92acf7ec108548ba3b43e92`
- Test count: 254 (36 unit + 218 integration, including `cli_milestone_smoke` 2)

## TASK-33 — Glossary milestone packaging and verification

- Date: **2026-11-06 (UTC)**.
- Environment: Debian GNU/Linux forky/sid amd64, glibc 2.43, dpkg 1.23.11, `rustc`/`cargo` 1.99.1, `fakeroot`.
- Isolation: `fakeroot dpkg --instdir=./.tmp/inst --admindir=./.tmp/adm --force-depends`; the host package status was unchanged before and after the run. Scratch directories were removed after verification.
- Package: `papyrus_0.1.0_amd64.deb`, 865788 bytes.
- Package contents: `/usr/bin/papyrus` (0755) and `/usr/share/doc/papyrus/copyright` only; no asset, template, configuration, or man files.
- New test file: `tests/glossary_terms_smoke.rs` (2 tests), passing under `cargo test`.
- No runtime dependencies added; the TASK-1/TASK-4 runtime dependency license audit remains valid.

### Quality gates

| Gate | Result |
|---|---|
| `cargo build --release` (Rust 1.99.1) | PASS |
| `cargo test` | PASS — 256 tests |
| `cargo clippy --all-targets -- -D warnings` | PASS — 0 warnings |
| `cargo fmt --check` | PASS |

### Install, uninstall, and smoke checks

| Check | Result |
|---|---|
| Isolated `dpkg -i` | PASS (exit 0) |
| Installed `/usr/bin/papyrus` is `root:root` 0755 | PASS |
| `papyrus` resolves on the isolated `PATH` and runs | PASS |
| Isolated `dpkg -r papyrus` removes the binary; not on `PATH` | PASS |
| Isolated `dpkg --purge papyrus` removes binary and admin entry | PASS |
| Host package database and filesystem unchanged | PASS |
| `--help` lists `glossary`, `search`, and `browser`; `--version` prints `papyrus 0.1.0` | PASS |
| Glossary lifecycle (create/list/view/edit/deprecate/restore) | PASS |
| Search over glossary terms (quoted phrase, `--type doc`, `--limit`, `--json`, deprecated hidden) | PASS |
| Browser glossary category (`/?type=glossary`, record view, edit round-trip, JSON API, embedded assets, 320 px hooks, no external references) | PASS |
| `glossary_terms_smoke` (2 tests) | PASS |
| Embedded assets from an empty working directory and asset-free project | PASS |

### Recorded limitations

- **Offline namespace verified via `unshare`.** The offline claim rests on loopback-only binding, no external references in served content, and no outbound-connection code path; the `unshare -rn` namespace check was run and passed.
- **No real-GUI browser run.** Markup- and asset-level verification only (NFR-05.001).
- **Config keys not re-checked on the installed binary.** Browser configuration keys (`browser_host`, `browser_port`, `browser_open`) were not re-verified against the installed package in this run.

### TASK-33 package artifact

- File: `papyrus_0.1.0_amd64.deb`
- Size: 865788 bytes
- Test count: 256 (including `glossary_terms_smoke` 2)

## TASK-37 — UI Specifications milestone packaging and verification

- Date: **2026-11-06 (UTC)**.
- Environment: Debian GNU/Linux forky/sid amd64, glibc 2.43, dpkg 1.23.11, `rustc`/`cargo` 1.99.1, `fakeroot`.
- Isolation: `fakeroot dpkg --instdir=./.tmp/inst37 --admindir=./.tmp/adm37 --force-depends`; host unchanged.
- Package: `papyrus_0.1.0_amd64.deb`, 869420 bytes.
- Contents: `/usr/bin/papyrus` (0755) and copyright only; assets embedded.
- New tests: `tests/ui_specifications_smoke.rs` (2 tests), passing.
- No runtime dependencies added.

### Quality gates

| Gate | Result |
|---|---|
| `cargo build --release` (Rust 1.99.1) | PASS |
| `cargo test` | PASS — 258 tests |
| `cargo clippy --all-targets -- -D warnings` | PASS — 0 warnings |
| `cargo fmt --check` | PASS |

### Install, uninstall, and smoke checks

| Check | Result |
|---|---|
| Isolated `dpkg -i` | PASS |
| Installed binary root:root 0755, on PATH | PASS |
| Isolated `dpkg -r`/`--purge` | PASS |
| UI spec lifecycle (create/list/view/edit/deprecate/restore) | PASS |
| Search over UI specifications (quoted phrase, `--type doc`, `--limit`, `--json`, deprecated hidden) | PASS |
| Browser `/?type=ui-spec`, view, edit, `/api/ui-specs`, assets, 320px hooks | PASS |
| `ui_specifications_smoke` (2 tests) | PASS |

### Recorded limitations

- Offline namespace verified via `unshare`; rests on loopback-only + no external refs.
- No real-GUI browser run (markup/asset-level only).
- Config keys verified on built binary (list/get/set/unset, invalid exits 4); installed-binary recheck deferred.

### TASK-37 package artifact

- File: `papyrus_0.1.0_amd64.deb`
- Size: 869420 bytes
- Test count: 258 (including `ui_specifications_smoke` 2)

## TASK-42 — UI mockups milestone packaging and verification

- Date: **2026-11-07 (UTC)**.
- Environment: Debian GNU/Linux forky/sid amd64, glibc 2.43-7, dpkg 1.23.11,
  `rustc`/`cargo` 1.99.1, clippy 0.1.99, `fakeroot` 2.1.4, `unshare -rn` with
  `ip link set lo up` available.
- Supported release: **Debian Forky only** (ADR-0003, NFR-03.002). Debian Trixie
  and every other Debian release remain **not claimed and not verified**.
- Isolation: `fakeroot dpkg --instdir=./.tmp/task42/inst
  --admindir=./.tmp/task42/adm --force-depends`; the host package status was
  `install ok installed 0.1.0` before and after the run and was not modified.
  Scratch directories were removed after verification.
- Package: `papyrus_0.1.0_amd64.deb`, 886836 bytes,
  SHA-256 `1294fd25ab5fe0081d6970d52670e97159ac17e368adff6776fc62bacf50c216`.
- Package metadata: `Package: papyrus`, `Version: 0.1.0` (from `Cargo.toml`),
  `Architecture: amd64`, `Depends: libc6 (>= 2.34)`, `Section: utils`,
  `Priority: optional`, copyright `GPL-3.0-or-later`.
- Package contents: `/usr/bin/papyrus` (0755, 2771616 bytes) and
  `/usr/share/doc/papyrus/copyright` (0644, 865 bytes) only; no asset, template,
  configuration, or man file. `/assets/app.css` and `/assets/app.js` are compiled
  into the binary and served from memory.
- New test file: `tests/ui_mockups_smoke.rs` (2 tests), passing under `cargo test`.
- No runtime dependencies were added; the TASK-1/TASK-4 runtime dependency
  license audit remains valid (NFR-06.002).

### Quality gates

| Gate | Result |
|---|---|
| `cargo build` (Rust 1.99.1) | PASS |
| `cargo build --release` (Rust 1.99.1) | PASS |
| `target/release/papyrus --help` lists `ui-mockup`, `search`, `browser` | PASS |
| `cargo test` | PASS — 292 tests (54 unit + 238 integration, including `ui_mockups_smoke` 2) |
| `cargo clippy --all-targets -- -D warnings` | PASS — 0 warnings |
| `cargo fmt --check` | PASS |

### Install, uninstall, and smoke checks

| Check | Result |
|---|---|
| Isolated `dpkg -i` | PASS (exit 0) |
| Installed `/usr/bin/papyrus` is `root:root` 0755 | PASS |
| `papyrus` resolves on the isolated `PATH` and runs | PASS |
| Isolated `dpkg -r papyrus` removes the binary; not on that `PATH` | PASS |
| Isolated `dpkg --purge papyrus` removes binary and admin entry | PASS |
| Host package database and filesystem unchanged | PASS |
| `--help` lists `ui-mockup`, `search`, `browser`; `--version` prints `papyrus 0.1.0` | PASS |
| UI mockup lifecycle: create/list/view/edit/deprecate/restore/supersede; IDs preserved | PASS |
| Lifecycle exit codes: unknown id 3, self-supersede 4, unknown successor 3, blocked transitions 4 | PASS |
| Search: quoted phrase, split words no match, `--type doc`, `--limit`, `--json`, empty `[]`, deprecated hidden until restored | PASS |
| Browser: startup line, `?type=ui-mockup` with `aria-current`, view, edit round-trip `303`/`Location`, persisted through UI and CLI | PASS |
| Browser JSON API: `/api/ui-mockups`, `/api/doc/<id>` shapes | PASS |
| Embedded assets: `/assets/app.css`, `/assets/app.js`, non-empty, correct content type, `Cache-Control: no-store` | PASS |
| 320 px markup hooks and narrow-viewport CSS | PASS |
| No external references in served HTML/CSS/JS; CSP present on HTML | PASS |
| Deprecated mockup not listed and `404` on view, edit, POST, and API | PASS |
| Offline (`unshare -rn`, outbound TCP fails): search, list, view, edit, assets over loopback | PASS |
| Config keys: list/get/set/unset for `browser_host`, `browser_port`, `browser_open`; configured port honored; `--port` overrides; defaults restored; invalid values exit 4 with unchanged file | PASS |
| Embedded assets from an empty working directory and asset-free project | PASS |

### Recorded limitations

- **OQ-1 — no real-GUI browser run.** Verification is markup- and asset-level
  only. Firefox, Edge, and Safari 100+ were not driven in a GUI; NFR-05.001
  browser support remains covered by the TASK-40 contract tests and MOCK-0006.
- **Forky only.** No Debian Trixie or other release was installed or verified,
  and none is claimed (ADR-0003, NFR-03.002).
- **Host baseline.** The build host already reported `papyrus 0.1.0` installed
  from prior work; the isolated verification did not modify it.
- **Packaging script precondition.** `scripts/build-deb.sh` was intentionally left
  unchanged. The stale-binary hazard is mitigated by running
  `cargo build --release` first and asserting `--help` lists `ui-mockup` before
  packaging.

### TASK-42 package artifact

- File: `papyrus_0.1.0_amd64.deb`
- Size: 886836 bytes
- SHA-256: `1294fd25ab5fe0081d6970d52670e97159ac17e368adff6776fc62bacf50c216`
- Test count: 292 (54 unit + 238 integration, including `ui_mockups_smoke` 2)

## TASK-46 — Business rules milestone packaging and verification

- Date: **2026-11-07 (CET, +0100)**.
- Environment: Debian GNU/Linux forky/sid amd64, glibc 2.43-7, dpkg 1.23.11,
  `rustc`/`cargo` 1.99.1, clippy 0.1.99, `fakeroot` 2.1.4, `unshare -rn` with
  `ip link set lo up` available.
- Supported release: **Debian Forky only** (ADR-0003, NFR-03.002). Debian Trixie
  and every other Debian release remain **not claimed and not verified**.
- Isolation: `fakeroot dpkg --instdir=./.tmp/task46/inst
  --admindir=./.tmp/task46/adm --force-depends`; the host package status was
  not modified. Scratch directories were removed after verification.
- Package: `papyrus_0.1.0_amd64.deb`, 903368 bytes,
  SHA-256 `a287e67143a4064d36fc66bcf191e8511c4e12cce95e5cf82e42184df8afd724`.
- Package metadata: `Package: papyrus`, `Version: 0.1.0` (from `Cargo.toml`),
  `Architecture: amd64`, `Depends: libc6 (>= 2.34)`, `Section: utils`,
  `Priority: optional`, copyright `GPL-3.0-or-later`.
- Package contents: `/usr/bin/papyrus` (0755, 2828536 bytes) and
  `/usr/share/doc/papyrus/copyright` (0644, 865 bytes) only; no asset, template,
  configuration, or man file. `/assets/app.css` and `/assets/app.js` are compiled
  into the binary and served from memory.
- Business rules have no supersede command (BR-013); lifecycle covers
  create/list/view/edit/deprecate/restore only.
- New test file: `tests/business_rules_smoke.rs` (2 tests), passing under `cargo test`.
- No runtime dependencies were added; the TASK-1/TASK-4 runtime dependency
  license audit remains valid (NFR-06.002).

### Quality gates

| Gate | Result |
|---|---|
| `cargo build` (Rust 1.99.1) | PASS |
| `cargo build --release` (Rust 1.99.1) | PASS |
| `target/release/papyrus --help` lists `rule`, `search`, `browser` | PASS |
| `cargo test` | PASS — 322 tests (59 unit + 263 integration, including `business_rules_smoke` 2) |
| `cargo clippy --all-targets -- -D warnings` | PASS — 0 warnings |
| `cargo fmt --check` | PASS |

### Install, uninstall, and smoke checks

| Check | Result |
|---|---|
| Isolated `dpkg -i` | PASS (exit 0) |
| Installed `/usr/bin/papyrus` is `root:root` 0755 | PASS |
| `papyrus` resolves on the isolated `PATH` and runs | PASS |
| Isolated `dpkg -r papyrus` removes the binary; not on that `PATH` | PASS |
| Isolated `dpkg --purge papyrus` removes binary and admin entry | PASS (verified via separate purge run) |
| Host package database and filesystem unchanged | PASS |
| `--help` lists `rule`, `search`, `browser`; `--version` prints `papyrus 0.1.0` | PASS |
| Business-rule lifecycle: create/list/view/edit/deprecate/restore; IDs preserved | PASS |
| Lifecycle exit codes: unknown id 3, already-deprecated/already-active 4 | PASS |
| Requirements-tag interaction: `--requirement` add, `--no-requirement` remove/all | PASS |
| Search: quoted phrase, split words no match, `--type doc`, `--limit`, `--json`, empty `[]`, deprecated hidden until restored | PASS |
| Browser: startup line, `?type=business-rule` with `aria-current`, view, edit round-trip `303`/`Location`, persisted through UI and CLI | PASS |
| Browser field-preserving edit: unknown sections retained | PASS |
| Browser JSON API: `/api/business-rules`, `/api/business-rule`, `/api/doc/<id>` shapes | PASS |
| Embedded assets: `/assets/app.css`, `/assets/app.js`, non-empty, correct content type | PASS |
| 320 px markup hooks and narrow-viewport CSS | PASS |
| No external references in served HTML/CSS/JS; CSP present on HTML | PASS |
| Deprecated rule not listed and `404` on view, edit, POST, and API | PASS |
| Offline (`unshare -rn`, outbound TCP fails): search over loopback | PASS |
| Config keys: list/get/set/unset for `browser_host`, `browser_port`, `browser_open`; configured port honored; `--port` overrides; defaults restored; invalid values exit 4 with unchanged file | PASS |
| Embedded assets from an empty working directory and asset-free project | PASS |

### Recorded limitations

- **OQ-1 — no real-GUI browser run.** Verification is markup- and asset-level
  only. Firefox, Edge, and Safari 100+ were not driven in a GUI; NFR-05.001
  browser support remains covered by the TASK-25 contract tests.
- **Forky only.** No Debian Trixie or other release was installed or verified,
  and none is claimed (ADR-0003, NFR-03.002).
- **Host baseline.** The build host already reported `papyrus 0.1.0` installed
  from prior work; the isolated verification did not modify it.
- **Packaging script precondition.** `scripts/build-deb.sh` was intentionally left
  unchanged. The stale-binary hazard is mitigated by running
  `cargo build --release` first and asserting `--help` lists `rule` before
  packaging.
- **No supersede.** Business rules have no supersede command (BR-013); the
  lifecycle smoke covers create/list/view/edit/deprecate/restore only.

### TASK-46 package artifact

- File: `papyrus_0.1.0_amd64.deb`
- Size: 903368 bytes
- SHA-256: `a287e67143a4064d36fc66bcf191e8511c4e12cce95e5cf82e42184df8afd724`
- Test count: 322 (59 unit + 263 integration, including `business_rules_smoke` 2)

## TASK-50 — Audit records milestone packaging and verification

- Date: **2026-11-07 (UTC)**.
- Environment: Debian GNU/Linux forky/sid amd64, glibc 2.43-7, dpkg 1.23.11,
  `rustc`/`cargo` 1.99.1, clippy 0.1.99, `fakeroot` 2.1.4, `unshare -rn` with
  `ip link set lo up` available.
- Supported release: **Debian Forky only** (ADR-0003, NFR-03.002). Debian Trixie
  and every other Debian release remain **not claimed and not verified**.
- Isolation: `fakeroot dpkg --instdir=./.tmp/inst
  --admindir=./.tmp/adm --force-depends`; the host package status was
  `install ok installed 0.1.0` before and after the run and was not modified.
  Scratch directories were removed after verification.
- Package: `papyrus_0.1.0_amd64.deb`, 910548 bytes,
  SHA-256 `fcc0fd05c9684ffc11a3b770af9f94eab2f7bb6928cff44335662407de7b605d`.
- Package metadata: `Package: papyrus`, `Version: 0.1.0` (from `Cargo.toml`),
  `Architecture: amd64`, `Depends: libc6 (>= 2.34)`, `Section: utils`,
  `Priority: optional`, copyright `GPL-3.0-or-later`.
- Package contents: `/usr/bin/papyrus` (0755, 2858256 bytes) and
  `/usr/share/doc/papyrus/copyright` (0644, 865 bytes) only; no asset, template,
  configuration, or man file. `/assets/app.css` and `/assets/app.js` are compiled
  into the binary and served from memory.
- Audit records have no supersede command (BR-014); lifecycle covers
  create/list/view/edit/deprecate/restore only. No requirements-tag interaction
  (that is business rules, BR-013). Audit type supports built-in
  security/licence/architecture plus free-form sanitized values.
- New test file: `tests/audit_records_smoke.rs` (2 tests), passing under `cargo test`.
- No runtime dependencies were added; the TASK-1/TASK-4 runtime dependency
  license audit remains valid (NFR-06.002).

### Quality gates

| Gate | Result |
|---|---|
| `cargo build` (Rust 1.99.1) | PASS |
| `cargo build --release` (Rust 1.99.1) | PASS |
| `target/release/papyrus --help` lists `audit`, `search`, `browser` | PASS |
| `cargo test` | PASS — 352 tests (64 unit + 288 integration, including `audit_records_smoke` 2) |
| `cargo clippy --all-targets -- -D warnings` | PASS — 0 warnings |
| `cargo fmt --check` | PASS |

### Install, uninstall, and smoke checks

| Check | Result |
|---|---|
| Isolated `dpkg -i` | PASS (exit 0) |
| Installed `/usr/bin/papyrus` is `root:root` 0755 | PASS |
| `papyrus` resolves on the isolated `PATH` and runs | PASS |
| Isolated `dpkg -r papyrus` removes the binary; not on that `PATH` | PASS |
| Isolated `dpkg --purge papyrus` removes binary and admin entry | PASS |
| Host package database and filesystem unchanged | PASS |
| `--help` lists `audit`, `search`, `browser`; `--version` prints `papyrus 0.1.0` | PASS |
| Audit-record lifecycle: create/list/view/edit/deprecate/restore; IDs preserved | PASS |
| Create supports `--title`, `--content`, `--audit-type` (default security; built-in + free-form sanitized) | PASS |
| Edit preserves omitted fields; audit type shown on view; empty content clears section | PASS |
| Lifecycle exit codes: unknown id 3, already-deprecated/already-active 4 | PASS |
| Search: quoted phrase, split words no match, `--type doc`, `--limit`, `--json`, empty `[]`, deprecated hidden until restored | PASS |
| Browser: startup line, `?type=audit-record` with `aria-current`, view, edit round-trip `303`/`Location`, persisted through UI and CLI | PASS |
| Browser audit type shown (badge next to title/ID plus Audit type row); content as body text without a Content heading | PASS |
| Browser field-preserving edit: unknown sections retained | PASS |
| Browser JSON API: `/api/audit-records`, `/api/audit-record`, `/api/doc/<id>` shapes | PASS |
| Embedded assets: `/assets/app.css`, `/assets/app.js`, non-empty, correct content type | PASS |
| 320 px markup hooks and narrow-viewport CSS | PASS |
| No external references in served HTML/CSS/JS; CSP present on HTML | PASS |
| Deprecated audit not listed and `404` on view, edit, POST, and API | PASS |
| Offline (`unshare -rn`, outbound TCP fails): search, list, view, edit, assets over loopback | PASS |
| Config keys: list/get/set/unset for `browser_host`, `browser_port`, `browser_open`; configured port honored; `--port` overrides; defaults restored; invalid values exit 4 with unchanged file | PASS |
| Embedded assets from an empty working directory and asset-free project | PASS |

### Recorded limitations

- **OQ-1 — no real-GUI browser run.** Verification is markup- and asset-level
  only. Firefox, Edge, and Safari 100+ were not driven in a GUI; NFR-05.001
  browser support remains covered by the TASK-26 contract tests.
- **Forky only.** No Debian Trixie or other release was installed or verified,
  and none is claimed (ADR-0003, NFR-03.002).
- **Host baseline.** The build host already reported `papyrus 0.1.0` installed
  from prior work; the isolated verification did not modify it.
- **Packaging script precondition.** `scripts/build-deb.sh` was intentionally left
  unchanged. The stale-binary hazard is mitigated by running
  `cargo build --release` first and asserting `--help` lists `audit` before
  packaging.
- **No supersede.** Audit records have no supersede command (BR-014); the
  lifecycle smoke covers create/list/view/edit/deprecate/restore only.
- **No requirements-tag interaction.** Audit records carry no requirements tags
  (that is business rules, BR-013); no such step is claimed.

## TASK-59 — Technical specifications milestone packaging and verification

- Date: **2026-11-07 (UTC)**.
- Supported release: **Debian Forky only** (amd64), per ADR-0003 and NFR-03.002.
  Debian Trixie and every other Debian release are **not claimed and not verified**.
- Environment: Debian GNU/Linux forky/sid amd64, glibc 2.43-7, dpkg 1.23.11,
  `rustc`/`cargo` 1.99.1, clippy 0.1.99, `fakeroot` 2.1.4, `unshare -rn` with
  `ip link set lo up` available.
- Isolation: `fakeroot dpkg --instdir=./.tmp/task59/inst
  --admindir=./.tmp/task59/adm --force-depends`; the host package status was
  `install ok installed 0.1.0` before and after the run and was not modified.
  Scratch directories were removed after verification.
- Package: `papyrus_0.1.0_amd64.deb`, 918176 bytes,
  SHA-256 `eaabd7cc16a2bce1ea1c3675d3e955d2abd33f7756fe67133ffa0a3c12eb3498`.
- Package metadata: `Package: papyrus`, `Version: 0.1.0` (from `Cargo.toml`),
  `Architecture: amd64`, `Depends: libc6 (>= 2.34)`, `Section: utils`,
  `Priority: optional`, copyright `GPL-3.0-or-later`.
- Package contents: `/usr/bin/papyrus` (0755, 2888128 bytes) and
  `/usr/share/doc/papyrus/copyright` (0644, 865 bytes) only; no asset, template,
  configuration, or man file. `/assets/app.css` and `/assets/app.js` are compiled
  into the binary and served from memory.
- Technical specifications have no supersede or delete command (BR-017); the
  lifecycle covers create/list/view/edit/deprecate/restore only.
- New test file: `tests/technical_specifications_smoke.rs` (2 tests), passing under
  `cargo test`.
- No runtime dependencies were added; the TASK-1/TASK-4 runtime dependency
  license audit remains valid (NFR-06.002).

### Build and package (TASK-59)

```sh
cargo build --release
./target/release/papyrus --help        # must list tech-spec, search, and browser
./scripts/build-deb.sh                 # unchanged; produces papyrus_0.1.0_amd64.deb
dpkg-deb -I papyrus_0.1.0_amd64.deb
dpkg-deb -c papyrus_0.1.0_amd64.deb
dpkg-deb --fsys-tarfile papyrus_0.1.0_amd64.deb \
  | tar -xO ./usr/share/doc/papyrus/copyright | grep GPL-3.0-or-later
```

### Isolated install and uninstall (TASK-59)

Follow the isolated procedure documented above, using
`./.tmp/task59/{inst,adm}`. Verify:

```sh
fakeroot stat -c '%U:%G %a %n' ./.tmp/task59/inst/usr/bin/papyrus   # root:root 755
env PATH="$PWD/.tmp/task59/inst/usr/bin:$PATH" sh -c \
  'command -v papyrus; papyrus --version'                            # papyrus 0.1.0
fakeroot dpkg --instdir=./.tmp/task59/inst --admindir=./.tmp/task59/adm -r papyrus
env -i PATH="$PWD/.tmp/task59/inst/usr/bin" /bin/sh -c \
  'command -v papyrus || echo "not on isolated PATH"'
fakeroot dpkg --instdir=./.tmp/task59/inst --admindir=./.tmp/task59/adm --purge papyrus
rm -rf ./.tmp/task59
```

The host may already have `papyrus` installed from prior work; record that status
before and after and confirm it is unchanged. Use an isolated-only `PATH` for the
post-removal check when the host has the package installed.

### Extended smoke-test checklist (TASK-59)

Run every command against the installed binary (`./.tmp/task59/inst/usr/bin/papyrus`
on `PATH`) with `--dir` pointing at a scratch project under `./.tmp/task59/`. Exit
statuses follow the existing contract: `0` success, `1` IO/boundary, `2` usage,
`3` not found, `4` validation.

| Area | Checklist item |
|---|---|
| Baseline (3.1) | `--help` exits 0 and lists `tech-spec`, `search`, `browser`; `--version` prints `papyrus 0.1.0` |
| Lifecycle (3.1) | `init`; `tech-spec create <title> --description <text>` prints `TECH-001` and writes `docs/technical/TECH-001.md` with heading, `## Description`, and one trailing newline |
| Lifecycle (3.1) | `list` is active-only and ID-ordered; `view` prints the persisted text and still works for a deprecated ID |
| Lifecycle (3.1) | `edit --title`/`--description` changes only the supplied field; IDs and paths are unchanged |
| Lifecycle (3.1) | `deprecate` moves the file to `deprecated/`; `restore` returns it with the same ID and byte-identical content |
| Lifecycle (3.1) | Missing required arguments exit 2; empty title, control characters, and path separators exit 4; unknown IDs exit 3; already-deprecated/already-active transitions exit 4; a disabled `document_types.tech-spec.enabled` rejects create/list with exit 4 |
| Search (3.2) | Contiguous quoted phrase matches; split words do not match; `--type doc` includes technical specifications; other `--type` values exclude them; `--limit` caps results |
| Search (3.2) | `--json` returns `{id,type,title,path}` objects and `[]` on no match; deprecated records are hidden until restored; a missing directory returns an empty success; `--type tech-spec` exits 2 |
| Browser (3.3) | `browser --no-open --port <free>` prints exactly `Serving http://127.0.0.1:<port>/` |
| Browser (3.3) | `/?type=tech-spec` renders the Technical Specifications category; the sidebar entry follows Audit Records and carries `aria-current="page"`; the count, ID-ordered rows, and first-paragraph preview render |
| Browser (3.3) | `/doc/<id>` shows ID, form, category, title, and description as body text with `white-space: pre-wrap` (no Description heading; `(none)` when empty) and exactly one Edit action |
| Browser (3.3) | Edit round-trip returns `303` to `/doc/<id>`, persists through both the UI and the CLI, and preserves the preamble and unknown sections byte-for-byte |
| Browser (3.3) | `/api/tech-specs`, its `/api/tech-spec` alias, and `/api/doc/<id>` return the documented JSON; `/assets/app.css` and `/assets/app.js` are non-empty with `Cache-Control: no-store` |
| Browser (3.3) | 320 px hooks are present (viewport meta, sidebar toggle with `aria-expanded`/`aria-controls`); served HTML/CSS/JS have no external references and HTML carries the CSP |
| Deprecation (3.3, 3.4) | A deprecated specification is not listed or counted and returns `404` on view, edit, form POST, and API; HTML 404s are identical for deprecated and unknown IDs and carry the help text and one recovery link |
| Offline (3.4) | Inside `unshare -rn` (loopback up), an outbound TCP connection fails; search and browser list/view/edit/assets pass over loopback |
| Config keys (3.5) | `config list/get` expose `browser_host`, `browser_port`, `browser_open`; a configured port is used without `--port` and `--port` overrides it; `browser_open=false` suppresses opening; unset restores defaults; invalid values exit 4 with an unchanged file |
| Assets (3.6) | From an empty working directory and an asset-free project, search and browser work and both embedded assets are non-empty |

### Quality gates

| Gate | Result |
|---|---|
| `cargo build` (Rust 1.99.1) | PASS |
| `cargo build --release` (Rust 1.99.1) | PASS |
| `target/release/papyrus --help` lists `tech-spec`, `search`, `browser` | PASS |
| `cargo test` | PASS — 383 tests (70 unit + 313 integration, including `technical_specifications_smoke` 2) |
| `cargo clippy --all-targets -- -D warnings` | PASS — 0 warnings |
| `cargo fmt --check` | PASS |

### Install, uninstall, and smoke checks

| Check | Result |
|---|---|
| Isolated `dpkg -i` | PASS (exit 0) |
| Installed `/usr/bin/papyrus` is `root:root` 0755 | PASS |
| `papyrus` resolves on the installed `PATH` and runs | PASS |
| Isolated `dpkg -r papyrus` removes the binary; not on the isolated-only `PATH` | PASS |
| Isolated `dpkg --purge papyrus` removes the admin entry; no papyrus files remain | PASS |
| Host package database and filesystem unchanged | PASS |
| `--help` lists `tech-spec`, `search`, `browser`; `--version` prints `papyrus 0.1.0` | PASS |
| Technical-specification lifecycle: create/list/view/edit/deprecate/restore; IDs preserved | PASS |
| Lifecycle exit codes: unknown ID 3; already-deprecated/already-active 4; missing arguments 2; sanitization 4; disabled type 4 | PASS |
| Search: quoted phrase, split words no match, `--type doc`, other `--type` filters, `--limit`, `--json`, empty `[]`, deprecated hidden until restored | PASS |
| Search: missing directory empty success; `--type tech-spec` exits 2 | PASS |
| Browser: startup line, `?type=tech-spec` with `aria-current` and position after Audit Records, count/rows/preview | PASS |
| Browser view: form, category, title, description body without a Description heading; `(none)` for empty; one Edit action | PASS |
| Browser edit round-trip: `303`, `Location`, persisted through UI and CLI | PASS |
| Browser field-preserving edit: preamble and unknown sections byte-identical | PASS |
| Browser JSON API: `/api/tech-specs`, `/api/tech-spec`, `/api/doc/<id>` | PASS |
| Embedded assets: `/assets/app.css`, `/assets/app.js`, non-empty, correct content type, `Cache-Control: no-store` | PASS |
| 320 px markup hooks and narrow-viewport CSS | PASS |
| No external references in served HTML/CSS/JS; CSP present on HTML | PASS |
| Deprecated specification not listed or counted; `404` on view, edit, POST, and API; identical 404 page and single recovery link | PASS |
| Offline (`unshare -rn`, outbound TCP fails): search, list, view, edit, assets over loopback | PASS |
| Config keys: list/get/set/unset; configured port honored; `--port` overrides; `browser_open=false`; defaults restored; invalid values exit 4 with unchanged file | PASS |
| Embedded assets from an empty working directory and asset-free project | PASS |

### Recorded limitations

- **OQ-1 — no real-GUI browser run.** Verification is markup- and asset-level
  only. Firefox, Edge, and Safari 100+ were not driven in a GUI; NFR-05.001
  browser support remains covered by the TASK-57 contract tests and MOCK-0009.
- **Forky only.** No Debian Trixie or other release was installed or verified,
  and none is claimed (ADR-0003, NFR-03.002).
- **Host baseline.** The build host already reported `papyrus 0.1.0` installed
  from prior work; the isolated verification did not modify it. The post-removal
  `PATH` check therefore uses the isolated root only.
- **Packaging script precondition.** `scripts/build-deb.sh` was intentionally left
  unchanged. The stale-binary hazard is mitigated by running
  `cargo build --release` first and asserting `--help` lists `tech-spec` before
  packaging.
- **No supersede or delete.** Technical specifications have no supersede or delete
  command (BR-017); the lifecycle smoke covers create/list/view/edit/deprecate/
  restore only.
- **No dedicated search type.** `--type tech-spec` is not part of the documented
  vocabulary and exits 2; technical specifications are found through `--type doc`
  and default search.
- **`--editor` not exercised.** The multi-line editor route is environment
  dependent and remains covered by `tests/technical_specifications_cli.rs`.

### TASK-59 package artifact

- File: `papyrus_0.1.0_amd64.deb`
- Size: 918176 bytes
- SHA-256: `eaabd7cc16a2bce1ea1c3675d3e955d2abd33f7756fe67133ffa0a3c12eb3498`
- Test count: 383 (70 unit + 313 integration, including `technical_specifications_smoke` 2)

## TASK-20 — Complete content-management milestone packaging and verification

- Date: **2026-11-10 (UTC)**.
- Environment: Debian GNU/Linux forky/sid amd64, glibc 2.43, dpkg 1.23.11,
  `rustc`/`cargo` 1.99.1, clippy 0.1.99, `fakeroot`, `unshare`/`ip` available.
- Supported release: **Debian Forky only** (amd64), per ADR-0003 and NFR-03.002.
  Debian Trixie and every other Debian release are **not claimed and not verified**.
- Isolation: `fakeroot dpkg --instdir=./.tmp/inst --admindir=./.tmp/adm
  --force-depends`; host package status unchanged before and after the run
  (host reports `papyrus 0.1.0` installed from prior work). Scratch directories
  removed after verification.
- Package: `papyrus_0.1.0_amd64.deb`, 1100976 bytes.
- Package metadata: `Package: papyrus`, `Version: 0.1.0`, `Architecture: amd64`,
  `Depends: libc6 (>= 2.34)`, `Section: utils`, `Priority: optional`,
  copyright `GPL-3.0-or-later`.
- Package contents: `/usr/bin/papyrus` (0755) and
  `/usr/share/doc/papyrus/copyright` (0644) only; no asset, template,
  configuration, or man file. `/assets/app.css` and `/assets/app.js` are
  compiled into the binary and served from memory.
- New test file: `tests/content_management_smoke.rs` (4 tests), passing under
  `cargo test`.
- No runtime dependencies added; the TASK-1/TASK-4 runtime dependency license
  audit remains valid (NFR-06.002).

### Quality gates

| Gate | Result |
|---|---|
| `cargo build --release` (Rust 1.99.1) | PASS |
| `target/release/papyrus --help` lists full command set | PASS |
| `cargo test` | PASS — 506 tests |
| `cargo clippy --all-targets -- -D warnings` | PASS — 0 warnings |
| `cargo fmt --check` | PASS |

### Install, uninstall, and smoke checks

| Check | Result |
|---|---|
| Isolated `dpkg -i` | PASS (exit 0) |
| Installed `/usr/bin/papyrus` is `root:root` 0755 | PASS |
| `papyrus` resolves on the isolated `PATH` and runs | PASS |
| Isolated `dpkg -r papyrus` removes the binary; not on `PATH` | PASS |
| Isolated `dpkg --purge papyrus` removes binary and admin entry | PASS |
| Host package database and filesystem unchanged | PASS |
| `--help` lists full command set; `--version` prints `papyrus 0.1.0` | PASS |
| All twelve record families: create/list/view/edit/deprecate-or-archive/restore | PASS |
| ID padding constrained to 1-6 (invalid values exit 4, file unchanged) | PASS (verified on the installed binary) |
| Unified search: quoted phrase, `--type`, `--limit`, `--json`, empty `[]`, deprecation/archival visibility, disabled types, no writes | PASS |
| Browser: startup line, inventory hub, sidebar final order, `aria-current`, per-type lists, record views, edit round-trip | PASS |
| Browser: cross-type ID resolution (single, multi -> 500, deprecated/unknown -> 404) | PASS |
| Browser: JSON API and embedded assets served from the binary | PASS |
| 320 px markup hooks and no external references | PASS |
| Front-matter metadata projection (REQ-06.004) | PASS |
| Configuration, managed-artifacts/lifecycle, and agent-skills browser surfaces removed: `/config`, `/config/edit`, `/lifecycle`, and `/skills` with their `?type=` aliases return `404`; no sidebar entry and no inventory card (ADR-034, TASK-170, TASK-194, TASK-197) | PASS |
| Lazy migration and Backlog.md round-trip | PASS |
| Offline operation over loopback | PASS |
| Browser configuration keys: list/get/set/unset, configured port, `--port` override, defaults restored, invalid values exit 4 | PASS |
| Embedded assets from an empty working directory and asset-free project | PASS |

### Recorded limitations

- **OQ-1 — no real-GUI browser run.** Verification is markup- and asset-level
  only. Firefox, Edge, and Safari 100+ were not driven in a GUI; NFR-05.001
  browser support remains covered by the TASK-4/TASK-19 contract tests and
  MOCK-0012.
- **Forky only.** No Debian Trixie or other release was installed or verified,
  and none is claimed (ADR-0003, NFR-03.002).
- **Host baseline.** The build host already reported `papyrus 0.1.0` installed
  from prior work; the isolated verification did not modify it.
- **Packaging script precondition.** `scripts/build-deb.sh` was intentionally
  left unchanged. The stale-binary hazard is mitigated by running
  `cargo build --release` first and asserting `--help` lists the full command
  set before packaging.
- **Offline namespace.** `unshare -rn` is available; the namespace run was
  executed and passed — an outbound TCP connection failed, and search, browser
  list/view/edit, and assets succeeded over loopback. The offline claim is also
  supported by loopback-only binding and no external references in served
  content.

## TASK-82 — Use cases and test documentation milestone packaging and verification

- Date: **2026-11-11 08:45:12 UTC** (verification started; local CET 09:45:12).
- Environment: Debian GNU/Linux forky/sid amd64, glibc 2.43, dpkg 1.23.11,
  `rustc`/`cargo` 1.99.1, clippy 0.1.99, `rustfmt` (rustfmt 1.99.1), `fakeroot`
  2.1.4, `unshare`/`ip` available.
- Supported release: **Debian Forky only** (amd64), per ADR-0003 and NFR-03.002.
  Debian Trixie and every other Debian release are **not claimed and not verified**.
- Isolation: `fakeroot dpkg --instdir=./.tmp/inst --admindir=./.tmp/adm
  --force-depends`; the host reported `papyrus 0.1.0` installed before and after
  the run and was not modified. Scratch directories were removed after verification.
- Package: `papyrus_0.1.0_amd64.deb`, 1136804 bytes,
  SHA-256 `b5f01d4e33fe100232b39bbf2adfc1c4f2d5767cd64adeec4c11771fa2051592`.
- Package metadata: `Package: papyrus`, `Version: 0.1.0` (from `Cargo.toml`),
  `Architecture: amd64`, `Depends: libc6 (>= 2.34)`, `Section: utils`,
  `Priority: optional`, copyright `GPL-3.0-or-later`.
- Package contents: `/usr/bin/papyrus` (0755, 3817808 bytes) and
  `/usr/share/doc/papyrus/copyright` (0644, 865 bytes) only; no asset, template,
  configuration, or man file. `/assets/app.css` and `/assets/app.js` are compiled
  into the binary and served from memory.
- New test file: `tests/use_cases_test_cases_test_reports_smoke.rs` (2 tests),
  passing under `cargo test`.
- No runtime dependencies were added; the TASK-1/TASK-4 runtime dependency
  license audit remains valid (NFR-06.002). Use cases, test cases, and test
  reports have no supersede or delete command (BR-018/BR-019/BR-020); the
  lifecycle smoke covers create/list/view/edit/deprecate/restore only. There is no
  dedicated `--type use-case`/`--type test-case`/`--type test-report` search
  filter; all three are found through `--type doc` and default search.

### Quality gates

| Gate | Result |
|---|---|
| `cargo build --release` (Rust 1.99.1) | PASS |
| `target/release/papyrus --help` lists `use-case`, `test-case`, `test-report`, `search`, `browser` | PASS |
| `cargo test` | PASS — 565 tests (including `use_cases_test_cases_test_reports_smoke` 2) |
| `cargo clippy --all-targets -- -D warnings` | PASS — 0 warnings |
| `cargo fmt --check` | PASS |

### Install, uninstall, and smoke checks

| Check | Result |
|---|---|
| Isolated `dpkg -i` | PASS (exit 0) |
| Installed `/usr/bin/papyrus` is `root:root` 0755 | PASS |
| `papyrus` resolves on the isolated `PATH` and runs | PASS |
| Isolated `dpkg -r papyrus` removes the binary; not on `PATH` | PASS |
| Isolated `dpkg --purge papyrus` removes the admin entry | PASS |
| Host package database and filesystem unchanged | PASS |
| `--help` lists `use-case`, `test-case`, `test-report`, `search`, `browser`; `--version` prints `papyrus 0.1.0` | PASS |
| Use-case/test-case/test-report lifecycle: create/list/view/edit/deprecate/restore; IDs preserved | PASS |
| Lifecycle exit codes: unknown id 3, already-deprecated/already-active 4, missing args 2, sanitization 4 | PASS |
| Search: quoted phrase, split words no match, `--type doc` includes all three, other `--type` filters exclude them, `--limit`, `--json`, empty `[]`, deprecated hidden until restored | PASS |
| Search: `--type use-case`/`--type test-case`/`--type test-report` exit 2 (undocumented) | PASS |
| Browser: startup line `Serving http://127.0.0.1:<port>/` with `--no-open` | PASS |
| Browser: `?type=use-case`, `?type=test-case`, `?type=test-report` with sidebar order after Technical Specifications and `aria-current="page"` | PASS |
| Browser: list count/rows/preview/metadata; record view form, category, title, description body without a Description heading, applicability section, single Edit | PASS |
| Browser edit round-trip: `303`, `Location`, persisted through UI and CLI | PASS |
| Browser field-preserving write: preamble and unknown sections byte-identical | PASS |
| Browser JSON API: `/api/use-cases`, `/api/use-case`, `/api/test-cases`, `/api/test-case`, `/api/test-reports`, `/api/test-report`, `/api/doc/<id>` | PASS |
| Embedded assets: `/assets/app.css`, `/assets/app.js`, non-empty, correct content type, `Cache-Control: no-store` | PASS |
| 320 px markup hooks and narrow-viewport CSS | PASS |
| No external references in served HTML/CSS/JS; CSP present on HTML | PASS |
| Deprecated records not listed or counted; `404` on view, edit, POST, and API with identical 404 page and single recovery link | PASS |
| Offline (`unshare -rn`, outbound TCP fails): search, list, view, edit, assets over loopback | PASS |
| Config keys: list/get/set/unset for `browser_host`, `browser_port`, `browser_open`; configured port honored; `--port` overrides; defaults restored; invalid values exit 4 with unchanged file | PASS |
| Embedded assets from an empty working directory and asset-free project | PASS |

### Recorded limitations

- **OQ-1 — no real-GUI browser run.** Verification is markup- and asset-level
  only. Firefox, Edge, and Safari 100+ were not driven in a GUI; NFR-05.001
  browser support remains covered by the TASK-81 contract tests and MOCK-0013.
- **Forky only.** No Debian Trixie or other release was installed or verified,
  and none is claimed (ADR-0003, NFR-03.002).
- **Host baseline.** The build host already reported `papyrus 0.1.0` installed
  from prior work; the isolated verification did not modify it.
- **Packaging script precondition.** `scripts/build-deb.sh` was intentionally left
  unchanged. The stale-binary hazard is mitigated by running
  `cargo build --release` first and asserting `--help` lists `use-case`,
  `test-case`, and `test-report` before packaging.
- **No supersede or delete.** Use cases, test cases, and test reports have no
  supersede or delete command (BR-018/BR-019/BR-020); the lifecycle smoke covers
  create/list/view/edit/deprecate/restore only.
- **`--editor` not exercised.** The multi-line editor route is environment
  dependent and remains covered by `tests/use_cases_cli.rs`,
  `tests/test_cases_cli.rs`, and `tests/test_reports_cli.rs`.
- **Offline namespace.** `unshare -rn` is available; the namespace run was
  executed and passed — an outbound TCP connection failed, and search, browser
  list/view/edit, and assets succeeded over loopback.

### TASK-82 package artifact

- File: `papyrus_0.1.0_amd64.deb`
- Size: 1136804 bytes
- SHA-256: `b5f01d4e33fe100232b39bbf2adfc1c4f2d5767cd64adeec4c11771fa2051592`
- Test count: 565 (including `use_cases_test_cases_test_reports_smoke` 2)

## TASK-98 — Release 1.0 native layout, import, relocation, single ID space, and packaging verification

**Supported platform statement.** Debian **Forky amd64** is the supported and verified
release target for this package (ADR-0003, NFR-03.002). Debian Trixie and every other
release are **not claimed or verified**; no other architecture is claimed.

### Environment

| Item | Value |
|---|---|
| Host | Debian GNU/Linux forky/sid, amd64 |
| Date and timezone | 2026-11-13 12:21 CET (UTC+01:00) |
| glibc | 2.43-7 |
| rustc / cargo | 1.99.1 (pinned by `rust-toolchain.toml`) |
| clippy / rustfmt | 0.1.99 / 1.10.0-stable |
| dpkg / dpkg-deb | 1.23.11 |
| Tooling | `fakeroot`, `unshare`, `ip`, `curl` available |
| Package toolchain | `./scripts/build-deb.sh` unchanged |

### Isolation method

- The sample Backlog.md project fixture and every scratch project were created under
  `./.tmp/task98-*` by `tests/release_native_layout_smoke.rs` and removed by its
  `Scratch` guard (also on panic). The installed-binary smoke used `./.tmp/task98/`.
- The repository's own `backlog/` store was never an import source and was not
  rewritten: its content hash (116 files, excluding the TASK-98 ticket metadata
  updated through the Backlog CLI) was unchanged, and `git status --porcelain --
  backlog` reported only the expected TASK-98 ticket file.
- The package was installed in an isolated dpkg root with
  `fakeroot dpkg --instdir=./.tmp/task98/inst --admindir=./.tmp/task98/adm
  --force-depends -i`; the host `dpkg -l papyrus` output was recorded before and
  after and was identical.

### Automated verification

`tests/release_native_layout_smoke.rs` (new, 9 tests):

| Test | Acceptance criterion |
|---|---|
| `t98_01_fixture_inventory_and_repository_store_untouched` | T-98-01 |
| `t98_02_adoption_and_noop_writes_are_byte_identical` | T-98-02 |
| `t98_03_no_backlog_shaped_write_anywhere` | T-98-03 |
| `t98_04_invisible_before_import_readable_after` | T-98-04 |
| `t98_05_single_id_space_on_every_surface` | T-98-05 |
| `t98_06_import_fidelity` | T-98-06 |
| `t98_07_documented_upgrade_path` | T-98-07 |
| `t98_08_relocation_atomicity_and_browser_path` | T-98-08 |
| `t98_09_search_indexes_front_matter_without_writes` | T-98-09 |

Observed results:

- **Adoption / no-op identity** — `t98_02_adoption_and_noop_writes_are_byte_identical`
  asserts both record states explicitly. **Adoption read:** for a legacy task, a
  legacy milestone, and a legacy technical specification, the adopted file must be
  exactly the prepended front-matter block followed by the original bytes, so every
  remaining body byte is compared byte-for-byte; the block must open and close with
  `---` and carry the record's `id`, and nothing beyond the block may be added. The
  legacy title heading, the `## Status`/`## Release` sections, and the content
  sections are still present after the read. **First write:** the first write to the legacy record must
  remove every level-1 title heading and the `## Status`/`## Release` metadata
  section, keep the `## Description` content section and its body, and keep the moved
  front-matter values; the following equal-value write must then be byte-, mode-, and
  mtime-identical, with `updated_date` not refreshed. A record that already carried
  native front matter was read twice and written with equal values without changing
  any byte, mode, `updated_date`, or mtime. A read-only directory failed adoption
  with a non-zero exit and left the file byte-identical.
- **No Backlog-shaped write** — after create, edit, archive, restore, deprecate,
  adopt, and import, every written record path was `<ID>.md` (decisions
  `<ID>-<kebab-slug>.md`); no Backlog stem, `## Status` metadata section, or
  storage-ID key was written. The eight mechanical checks (in a ten-test suite
  including two scanner-coverage tests) in `tests/no_drift.rs` passed in the same
  `cargo test` run.
- **Invisibility / import** — an unimported store copied into an initialised project
  was invisible to CLI list/view/search, their `--json` forms, browser list, JSON
  API, and record views while a requirement record stayed visible. After
  `papyrus import backlog --from <fixture>` every in-scope record was readable by
  its Papyrus ID and the fixture source tree hash was unchanged (before import,
  after import, and after CLI/browser reads).
- **Single ID space** — CLI text, `--json`, board HTML, `/api/tasks`,
  `/api/milestones`, `/api/doc/<ID>`, and rendered links carried only Papyrus IDs;
  no `storage_id`, `display_id`, or `storage_path` field appeared;
  `task view task-1` and `milestone view m-0` exited 3.
- **Import fidelity** — `task-1…task-5` became `TASK-001…TASK-005` in source-number
  order (milestones `m-0…m-2` became `MS-001…MS-003`, archived destinations
  preserved); fields, unknown key, and body markers mapped as specified; AC
  ordinals were dropped with checkbox states kept; unknown section and free text
  were preserved verbatim; `docs=1 drafts=1 decisions=1 config=1 locks=1` were
  reported as excluded and not imported. The status mismatch (`In Progress`) and
  dangling milestone (`m-999`) were warned about and preserved/dropped as specified.
- **Upgrade path** — `init`, a dry run that changed no project or source byte, a real
  import with a saved report, normal CLI/browser use (including a board status change
  persisted to the record), refusal of a second run without `--allow-existing`
  (exit 1), and duplication with new IDs when it was supplied. The source store was
  kept and byte-identical.
- **Relocation** — a multi-segment destination moved all files including unmanaged
  files/subfolders and removed the empty source; the browser configuration view
  (retired later by ADR-034) produced the same result. Conflict, unwritable, and
  out-of-project destinations exited 4, moved nothing, and left the configuration
  file byte-identical.
- **Search** — the front-matter-only query `fm-only-label` (a label present only in
  TASK-001's front matter) returned TASK-001 on the CLI and was visible in the
  browser record view and `/api/doc/TASK-001`; quoted phrases, `--type`, `--limit`,
  `--json`, empty results, archived-hidden, deprecated-hidden, and restored-visible
  behaviour matched the documented search behaviour. The project hash was unchanged
  after the search run. Review note: the product has no browser search endpoint;
  search is the documented CLI surface, and the browser record view/API expose the
  matching metadata.

### Quality gates

| Gate | Result |
|---|---|
| `cargo build --release` (Rust 1.99.1) | PASS — `target/release/papyrus` |
| `target/release/papyrus --help` lists `task`, `import`, `browser`, `search` | PASS |
| `target/release/papyrus --version` | PASS — `papyrus 0.1.0` |
| `cargo test` | PASS — **633 tests** (0 failed, 0 ignored; includes the new 9 and `tests/no_drift.rs`) |
| `cargo clippy --all-targets -- -D warnings` | PASS — 0 warnings |
| `cargo fmt --check` | PASS |

### Package artifact

- File: `papyrus_0.1.0_amd64.deb`
- Size: 1,197,168 bytes
- SHA-256: `96acf96b672522bbd718dab94552e1e817b1743745a11ad42a5d75744a0985b2`
- `dpkg-deb -I`: `Package: papyrus`, `Version: 0.1.0`, `Architecture: amd64`,
  `Depends: libc6 (>= 2.34)`, `Section: utils`, `Priority: optional`,
  `Maintainer: Papyrus Maintainers <maintainers@papyrus.example>`.
- `dpkg-deb -c`: only `/usr/bin/papyrus` (0755) and
  `/usr/share/doc/papyrus/copyright` (0644, 865 bytes) plus their directories; the
  copyright declares `GPL-3.0-or-later`.

### Installed-binary smoke on Debian Forky amd64

| Check | Result |
|---|---|
| Isolated `fakeroot dpkg --instdir/--admindir` install | PASS (exit 0) |
| Installed file owner/mode (fakeroot view) | PASS — `root:root` 0755 |
| Installed binary prints the package version | PASS — `papyrus 0.1.0` |
| Installed `init`, invisibility before import, `import backlog --from` | PASS — 1 task, 1 milestone; source kept |
| Browser board: per-milestone section and configured status columns | PASS — `MS-001` board section, six columns |
| Browser task record view | PASS — HTTP 200, body present |
| Board status change persisted to the record | PASS — HTTP 303, `status: Initiated` in `TASK-001.md` |
| Installed `config set backlog_directory` success | PASS — multi-segment `native/store/`; the unmanaged file and subfolder moved, the empty source was removed, and the new value was persisted |
| Installed `config set backlog_directory` conflict | PASS — exit 4, `destination entry conflict: unmanaged.txt`; nothing moved, both trees intact, configuration byte-identical |
| JSON API `/api/tasks`, `/api/milestones` | PASS — HTTP 200, Papyrus IDs |
| Embedded assets `/assets/app.css`, `/assets/app.js` | PASS — HTTP 200, 14,151 and 7,795 bytes |
| Offline `unshare -rn` (loopback up) | PASS — outbound TCP failed; list, record view, status change (303, `status: Refined`), API, CSS, and JS over loopback succeeded |
| `dpkg -r papyrus` | PASS — binary removed from the isolated root |
| `dpkg --purge papyrus` | PASS — admin entry removed, 0 files left |
| Host package database and filesystem | PASS — unchanged before/after |

### Recorded limitations and findings

- **OQ-1 — no real-GUI browser run.** Verification is HTTP, markup, and asset level;
  Firefox, Edge, and Safari were not driven in a GUI.
- **Forky only.** No Debian Trixie or other release was installed or verified.
- **Package version.** The package keeps `Cargo.toml` version `0.1.0` (OQ-2); the
  release owner decides any bump.
- **Offline namespace.** `unshare -rn` was available and the offline run passed.
- **TASK-99 upgrade notes unavailable at verification time.** The Release 1.0
  upgrade notes (TASK-99) did not exist in the repository when this verification ran
  (TASK-99 was in `To Do`), so the documented one-time import procedure was executed
  from the release specification §6, the import specification §11 contract, and the
  shipped `papyrus import backlog --help` instead of from TASK-99's notes. No TASK-99
  wording could be cross-checked; the TASK-99/TASK-73 owner must confirm the notes,
  once published, against this evidence.
- **Finding F-98-1 (for TASK-73/TASK-99, not edited here).**
  `release-1.0-native-layout-verification-and-packaging.md` §2.2 says a legacy
  record's equal-value write produces byte-identical content. Observed behaviour:
  the first write to a legacy body retires its title heading and metadata section
  (documented by TASK-89 in `task-management.md` and
  `milestone-and-release-management.md`), so that first write is not byte-identical;
  after retirement, equal-value writes are byte-identical. The new test asserts
  adoption-read byte-for-byte body identity, the documented first-write retirement
  (level-1 heading and metadata sections removed, content sections kept), and
  post-retirement identity. **Resolved in the quality-gate wording fix:** AC#2 and
  specification §2.2 now state adoption-read identity, native equal-value write
  identity, and the documented first-write retirement; the strengthened `t98_02`
  assertions match the reworded criteria, and no implementation change was needed.
- **Finding F-98-2 (for TASK-73).** The browser has no search endpoint; the release
  specification's "CLI and browser" search wording is satisfied only by the CLI
  search plus the browser record view/API exposure of the matching metadata.
  **Resolved in the quality-gate wording fix:** AC#9 and specification §8.1/§8.2
  scope search to the CLI with the browser limited to the record view and JSON API;
  no implementation change was needed.

## TASK-117 — Config-defined document types release verification and packaging

**Supported platform statement.** Debian **Forky amd64** is the supported and verified
release target for this package (ADR-0003, NFR-03.002). Debian Trixie and every other
release, and every other architecture, remain **not claimed or verified**.

This section records the m-17 release gate for config-defined document types. It is a
verification and packaging record only: no product source, requirement, business rule,
ADR, README, or upgrade-notes file was changed.

### Environment

| Item | Value |
|---|---|
| Host | Debian GNU/Linux forky/sid, amd64 |
| Date and timezone | 2026-11-14 20:53 CET (UTC+01:00) |
| glibc | 2.43-7 |
| rustc / cargo | 1.99.1 (pinned by `rust-toolchain.toml`) |
| clippy / rustfmt | 0.1.99 / 1.10.0-stable |
| dpkg / dpkg-deb | 1.23.11 |
| Tooling | `fakeroot`, `unshare`, `ip`, `curl` available |
| Package toolchain | `./scripts/build-deb.sh` unchanged |

### Isolation method

- Every CLI, upgrade, browser, and installed-binary fixture lived under
  `./.tmp/task117/`; the two new test files use a `Scratch` guard that removes its
  directory on success and on panic.
- The repository's `backlog/` store was never used as a fixture and was not written.
  `git status --porcelain` before and after the run lists only the TASK-117 artifacts.
- The package was installed in an isolated dpkg root with
  `fakeroot dpkg --instdir=./.tmp/task117/inst --admindir=./.tmp/task117/adm
  --force-depends -i`; the host `dpkg -l papyrus` output was identical before and
  after (the host already carried the packaged `0.1.0` from prior work).
- All `./.tmp/task117/` scratch, including the isolated roots, was removed after the
  verification; the built `.deb` remains.

### Automated verification

`cargo test`: **789 passed, 0 failed, 22 ignored** across 42 test binaries
(299 library unit tests plus integration binaries). The ignored tests are the
pre-existing milestone-flow smoke tests that require an external editor/flow; they
are unrelated to this feature.

Named document-type suites in the same run:

| Suite | Passed | Failed |
|---|---|---|
| `src/lib.rs` unit tests (registry, config, document, search, browser) | 299 | 0 |
| `tests/cli_dispatch.rs` | 9 | 0 |
| `tests/browser_config_types_cli.rs` | 19 | 0 |
| `tests/search_config_types_cli.rs` | 13 | 0 |
| `tests/config_list_keys_cli.rs` | 9 | 0 |
| `tests/cli.rs` | 24 | 0 |
| `tests/no_drift.rs` | 10 | 0 |
| `tests/release_document_types_smoke.rs` (new) | 8 | 0 |
| `tests/document_type_no_drift.rs` (new) | 8 | 0 |

New test names:

- `tests/release_document_types_smoke.rs`: `t117_01_generated_help_and_dispatch_matrix`,
  `t117_02_cli_lifecycle_persistence_and_validation`,
  `t117_03_json_is_accepted_on_every_document_verb`,
  `t117_04_search_front_matter_and_hidden_states`,
  `t117_05_invalid_configuration_fails_closed_and_repairs`,
  `t117_06_upgrade_from_pre_change_configuration`,
  `t117_07_deploy_command_from_configuration_alone`,
  `t117_08_browser_matrix_over_http`.
- `tests/document_type_no_drift.rs`: `d01_no_deprecated_folder_literal_in_src`,
  `d02_no_nfreq_command_in_src`, `d03_no_document_deprecate_or_restore_variants`,
  `d04_no_static_per_type_command_in_cli`,
  `d05_no_slug_literal_dispatch_outside_type_modules`,
  `d06_no_stale_nfreq_or_static_command_block_in_user_docs`,
  `d07_no_static_inventory_in_config_type_specifications`,
  `scanner_flags_planted_violations_and_accepts_allowlisted_lines`.

### CLI matrix (C-01 .. C-26)

Executed against `target/release/papyrus` on a fixture declaring
`note` (`NOTE`, `notes/`, `statuses: [active, archived, review, approved]`,
`default_status: active`).

| Row | Observed |
|---|---|
| C-01 `--help` | exit 0; the eleven default slugs plus `note`; no `deploy`, no `nfreq` |
| C-02 `note --help` | exit 0; create/list/view/edit/archive/delete/status |
| C-03 `req`/`nfr --help` | exit 0; `create-group`; `create --help` shows required `--group` |
| C-04 `ui-mockup --help` | exit 0; `supersede`; no `status` |
| C-05 `nfreq --help` | exit 2 |
| C-06 unknown slug | exit 2 |
| C-07 disabled note | exit 2 for every verb; absent from `--help` |
| C-08 undeclared `deploy` | exit 2 |
| C-09 create | exit 0; prints `NOTE-001`; writes `docs/notes/NOTE-001.md` |
| C-10 list/view | exit 0; ID order; metadata and body match the file |
| C-11 edit | exit 0; only the supplied title changed |
| C-12 status review | exit 0; `status: review`; file stays in place |
| C-13 status archived | exit 4; stderr names `archive`; file hash unchanged |
| C-14 archive | exit 0; moves to `docs/notes/archived/NOTE-001.md`; hidden from list; review discarded |
| C-15 delete without `--force` | prompt; `n` leaves the file, `y` removes it |
| C-16 delete `--force` | exit 0; no prompt; file gone |
| C-17 unknown ID | exit 3 |
| C-18 invalid title | empty, newline/CR, and path separator exit 4 with no write (see F-117-1) |
| C-19 unwritable directory | exit 1, `Permission denied (os error 13)` |
| C-20 `--json` | create/list/view/edit/status/archive/delete each emit one JSON value; errors emit `{"error":{"code","message"}}` on stderr |
| C-21 search | title, `--type doc`, and `--json` find `NOTE-*` as `doc`; archived/disabled hidden |
| C-22 `docs/notes/deprecated/NOTE-900.md` | not listed, viewed, searched, or browsed; inert |
| C-23 manual recovery | move to `docs/notes/` and set `status: active`; list/view work; no `restore` command (exit 2) |
| C-24 invalid config | validating commands exit 4 naming `document_types.note.statuses`; `config get`/`list` still read |
| C-25 repair | valid `config set` writes; invalid merge exits 4 and leaves the file byte-identical (same SHA-256) |
| C-26 `init --force` | rewrites the eleven defaults; `req list` exit 0; undeclared `note` exit 2 |

### Browser matrix (B-01 .. B-19)

HTTP only against `browser --no-open --port <free>` on a `note` fixture with
`active`, `review`, and `approved` records. Rows B-01 .. B-18 are asserted by
`t117_08_browser_matrix_over_http`; B-19 was executed manually.

| Row | Observed |
|---|---|
| B-01 startup | exactly `Serving http://127.0.0.1:<port>/` |
| B-02 list | 200; `Notes` under `Documents`; Status/ID/Title/Created/Updated order; rows match CLI |
| B-03 detail | 200; status read-only; no status control |
| B-04 edit form | 200; no `name="status"`/`name="status_change"`; labels/references present |
| B-05 edit POST | 303 to `/doc/NOTE-003`; persisted via CLI; unknown key/section preserved |
| B-06 no-op save | 303; file byte-identical (SHA-256 unchanged) |
| B-07 stale version | 409; file unchanged |
| B-08 `status=archived` | 422; file unchanged |
| B-09 filter values | exactly `active`, `review`, `approved`; no `archived` |
| B-10 review/approved filter | only matching records |
| B-11 invalid filters | `archived`, `bogus`, `Review`, empty, duplicate each 400 with the error page and no list |
| B-12 no status | all active-directory records, including custom statuses |
| B-13 archived | omitted; detail/edit/POST/API 404; 404 page identical to an unknown ID; deprecated file also 404 |
| B-14 disabled type | after restart: list/detail/edit/API 404; no sidebar entry; `GET /lifecycle` returns `404` (retired per ADR-034), so no page lists `Notes` |
| B-15 `/lifecycle` (retired) | `GET /lifecycle`, `GET /?type=lifecycle`, `GET /skills`, and `GET /?type=skills` return `404`; no lifecycle or agent-skills page is served (ADR-034) |
| B-16 `/api/docs/note` | 200; rows carry id, kind, title, status, labels, references, version, path, metadata |
| B-17 assets | app.css/app.js 200, non-empty, correct type, `Cache-Control: no-store` |
| B-18 CSP/external refs | `default-src 'none'; …` present; no `http://`/`https://` reference in served HTML |
| B-19 offline | `unshare -rn` with loopback up: outbound TCP to 1.1.1.1:80 fails (exit 1); list/detail/edit/API/assets all 200 over loopback |

### Upgrade scenario

A pre-change configuration containing only `document_types.req` and
`document_types.deploy`, an active `REQ-001`, a `docs/deployments/DEP-001.md`, and
`docs/requirements/business/deprecated/REQ-900.md` was exercised:

1. `--help` lists `req` and `deploy`, not `nfreq`.
2. `nfreq --help`, `req deprecate`, and `req restore` exit 2.
3. `req list` omits `REQ-900`; search for its body marker is empty; `deploy list` shows `DEP-001`.
4. Moving `REQ-900.md` into `docs/requirements/business/` and setting `status: active`
   makes `req list`/`req view REQ-900` work with the same ID.
5. Breaking a `statuses` list exits 4 naming `document_types.req.statuses`;
   `config get`/`list` still read; `config set` repairs it.
6. Removing the `deploy` entry makes `deploy --help` exit 2; re-declaring it by
   editing `papyrus.config.yml` only restores the command. The fresh-project check
   prints `DEP-001` for the first `deploy create "Release 1"` with no rebuild and no
   installed file beyond `/usr/bin/papyrus`.

The fixture outside the intentional edits stayed byte-identical. The exact
front-matter-only search query was `fm-only-label` (a label absent from the body);
it returned `NOTE-001 doc Wallet note` and did not change the project hash.

### Package artifact

- File: `papyrus_0.1.0_amd64.deb`
- Size: 1,284,160 bytes
- SHA-256: `6028cc9e79f112d03a0c2e2b1a3d423997920ec6232f723a48ac45516494486d`
- `dpkg-deb -I`: `Package: papyrus`, `Version: 0.1.0`, `Architecture: amd64`,
  `Depends: libc6 (>= 2.34)`, `Section: utils`, `Priority: optional`,
  `Maintainer: Papyrus Maintainers <maintainers@papyrus.example>`.
- `dpkg-deb -c`: only `/usr/bin/papyrus` (0755) and
  `/usr/share/doc/papyrus/copyright` (0644, 865 bytes) plus directories; the
  copyright declares `GPL-3.0-or-later`.

### Installed-binary smoke on Debian Forky amd64

| Check | Result |
|---|---|
| Isolated `fakeroot dpkg --instdir/--admindir` install | PASS (exit 0) |
| Installed file owner/mode (fakeroot view) | PASS — `root:root` 0755 binary, 0644 copyright |
| Installed binary version | PASS — `papyrus 0.1.0` |
| Installed `--help` | PASS — lists the enabled configuration slugs |
| Declare `note` by editing config only | PASS — `--help` lists `note` with no rebuild |
| Installed `note create/list/view/status/archive/delete/search` | PASS — `NOTE-001`, status `review`, archived path, forced delete, search hit |
| Offline `unshare -rn` (loopback up) | PASS — outbound TCP failed; `--help`, `note --help`, `note list` succeeded; browser list/detail/edit/API/assets returned 200 |
| Man page for `note` | Not expected and not asserted (NFR-03.003 gap; ADR-0021 stays Proposed) |
| `dpkg -r papyrus` | PASS — binary and copyright removed from the isolated root |
| `dpkg --purge papyrus` | PASS — no papyrus file and no papyrus admin entry remain |
| Host package database and filesystem | PASS — `dpkg -l papyrus` identical before/after |

### No-drift result

`tests/document_type_no_drift.rs` passed all D-01 .. D-07 checks in the full suite,
including the scanner self-test that plants a violation and an allowlisted migration
line. Historical ADRs, historical specifications, `docs/packaging.md` evidence, and
`backlog/` records are excluded as specified. The live documentation allowlist covers
only removal/migration wording, the inert-folder recovery section, and the legacy
folder note.

### Quality gates

| Gate | Result |
|---|---|
| `cargo build --release` (Rust 1.99.1) | PASS — `target/release/papyrus` |
| `target/release/papyrus --help` lists the enabled slugs before packaging | PASS |
| `cargo test` | PASS — 789 passed, 0 failed, 22 ignored |
| `cargo clippy --all-targets -- -D warnings` | PASS — 0 warnings |
| `cargo fmt --check` | PASS |

### Findings and dispositions

| ID | Finding | Severity | Owner | Disposition |
|---|---|---|---|---|
| G-1 | `config set document_types.<new>.prefix` cannot bootstrap a type; it exits 4 naming the missing `document_types.<new>.directory`, leaves the file byte-identical, and refuses a mapping value. The documented direct edit works. | Low | TASK-111 owner | Accepted documented limitation; not release-blocking. |
| G-2 | `docs/release-1.0-upgrade-notes.md` line 148 says task IDs allocate from active, archived, and deprecated locations; the shipped allocator reads only active and archived. A planted `backlog/deprecated/tasks/TASK-999.md` did not change the next `TASK-005` allocation. | Low | Upgrade-notes owner | Documentation finding; not release-blocking. |
| G-3 | `README.md` says milestones move `active -> archived`; `src/registry.rs` `MILESTONE_STATES` is `["open","closed"]`, and shipped milestone files carry no status while `archive` moves the file. | Low | README owner | Stale wording recorded; not release-blocking for this gate. |
| G-4 | `config list` omits `document_types.<slug>.default_status`; `config get` returns it, `config set`/`unset` validate it, and an invalid value is inspectable and repairable. | Medium | TASK-111 owner | Follow-up; not release-blocking because the repair path works. |
| G-5 | BR-007 documents `papyrus release close <id>` as the only `planned -> released` transition, but `src/release.rs` has no `Close` variant and `papyrus release close REL-001` exits 2; the browser lifecycle text also names a non-existent `papyrus release status <id> released`. The `planned -> released` transition is unreachable. | High | Release owner / BR-007 owner | **Release-blocking** for the m-17 gate unless a follow-up code task is opened or the owner accepts amending BR-007. |
| F-117-1 | `sanitize_name` rejects NUL, CR, LF, empty values, and path separators, but accepts other C0 controls (TAB 0x09, 0x01) in a document title; C-18 exercises the rejected forms. | Low | REQ-01.004 / TASK-110 owner | Follow-up input-validation hardening; not release-blocking. |

### Recorded limitations (G-6)

- No real-GUI browser run (NFR-05.001 limitation); verification is HTTP, markup, and
  asset level.
- Debian Forky amd64 only (ADR-0003, NFR-03.002); no other release or architecture is
  claimed.
- Config-defined slugs have no man page; ADR-0021 remains Proposed and NFR-03.003 is
  not claimed. This is a recorded release risk, not a new claim.
- `scripts/build-deb.sh` does not force a rebuild; the mitigation was
  `cargo build --release` first and asserting `--help` before packaging.
- `dpkg -r` removes both packaged files, so the subsequent `--purge` reports the
  package as already removed and removes the remaining admin entry; the end state is
  the intended one.

### Release gate statement

`release-ready: no` — G-5 is open and release-blocking. G-1, G-2, G-3, G-4, and
F-117-1 are recorded follow-ups or accepted limitations. The quality gates, CLI,
browser, upgrade, package, offline, and no-drift checks pass.

## TASK-149 — Browser assets, offline rendering, and binary budget

**Supported platform statement.** Debian **Forky amd64** is the supported and verified
release target for this package (ADR-0003, NFR-03.002). Debian Trixie and every other
release, and every other architecture, remain **not claimed or verified**.

This section records the m-21 packaging gate for the vendored browser assets
(ADR-0025, ADR-0026, ADR-0027) and the 12 MiB binary budget. It is a verification and
packaging record: no product source, test, packaging, requirement, business rule, ADR,
or README file was changed. Browser-observed rendering is TASK-150.

### Environment

| Item | Value |
|---|---|
| Host | Debian GNU/Linux forky/sid, amd64 |
| Date and timezone | 2026-11-16 10:28 CET (UTC+01:00) |
| glibc | 2.43-7 |
| rustc / cargo | 1.99.1 (pinned by `rust-toolchain.toml`) |
| clippy / rustfmt | 0.1.99 / 1.10.0-stable |
| dpkg / dpkg-deb | 1.23.11 |
| fakeroot | 2.1.4 |
| Tooling | `unshare -rn`, `ip`, `curl` available |
| Package toolchain | `./scripts/build-deb.sh` unchanged |

### Isolation method

- Every fixture, the extracted packaged binary, and the isolated dpkg root lived under
  `./.tmp/task149/`; the directory was removed after the run.
- The stale `papyrus_0.1.0_amd64.deb` present at the start (1,284,160 B, 2026-11-14,
  4,197,544 B binary, 865 B copyright) was deleted before packaging and is not evidence
  for this gate.
- The package was installed with `fakeroot dpkg --instdir=./.tmp/task149/inst
  --admindir=./.tmp/task149/adm --force-depends -i`. The first attempt aborted with
  `2 expected programs not found in PATH`; exporting
  `PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin` fixed it, as the
  Install section already requires.
- The host package database was identical before and after: `dpkg-query -W
  -f='${Status}' papyrus` reported `install ok installed` both times (the host already
  carried the packaged `0.1.0` from earlier work).

### Package artifact

- File: `papyrus_0.1.0_amd64.deb`
- Size: **2,379,712 bytes** (2.27 MiB)
- SHA-256: `b3bc9b729b2b3097a8939a12586896950fab8d738a79f762acb7f56ccf565ead`
- Growth over the stale 1,284,160 B package: **+1,095,552 B (1.04 MiB / 1.10 MB)**,
  slightly above ADR-0026 decision 12's 0.75–0.98 MB estimate. No package-size budget
  exists; recorded only.
- `dpkg-deb -I`: `Package: papyrus`, `Version: 0.1.0` (equal to `Cargo.toml`),
  `Architecture: amd64`, `Depends: libc6 (>= 2.34)`, `Section: utils`,
  `Priority: optional`, `Maintainer: Papyrus Maintainers
  <maintainers@papyrus.example>`, `Homepage`, and a four-line `Description`.
- Packaged binary equals the built binary:
  `dpkg-deb --fsys-tarfile ... | tar -xO ./usr/bin/papyrus | sha256sum` ->
  `2a71c1af6a1afc581540d51043145b731ce81621114f9a1a248d148ac2d5031e`, size
  **8,323,136 bytes** — both equal to `target/release/papyrus`.

### Package contents

| Entry | Mode | Bytes |
|---|---|---|
| `./usr/bin/papyrus` | `0755` | 8,323,136 |
| `./usr/share/doc/papyrus/copyright` | `0644` | 8,000 |

Directories only otherwise. A `grep -Ei '\.(css|js|html|map|woff2?|ttf)$|/etc/|/usr/share/man|/usr/share/papyrus|config'`
over `dpkg-deb -c` printed nothing and `grep -c '^-'` printed `2`. The isolated install
contained exactly the same two files, and `/usr/share/papyrus`, `/etc`, and
`/usr/share/man` were absent.

### Notices

- Packaged copyright: **8,000 bytes**, byte-identical to
  `packaging/deb/usr/share/doc/papyrus/copyright` (`cmp` exit 0, SHA-256
  `4610a7abd140648d1e2c136a38cc150c58904396fbbcf7f60348c02a0ca4fc3c`). Explicitly not
  the stale 865 B file.
- `Format:` Debian copyright header, then exactly **four** `Files:` paragraphs:
  `*` (GPL-3.0-or-later), `katex.min.js` + `katex.min.css` (MIT),
  `mermaid.min.js` (MIT-Mermaid), and `fonts/*.woff2` (OFL-1.1). Each identifier
  carries its full licence text; the Mermaid holder (`2014-2022 Knut Sveidqvist`) and
  the KaTeX font holders (Design Science, Inc., Khan Academy) are present.
- Every family in `src/browser/vendor/MANIFEST.md` — KaTeX code MIT, KaTeX fonts
  OFL-1.1, Mermaid MIT — has a paragraph.

### Assets and digests

| Check | Result |
|---|---|
| `cargo test --lib browser::assets` | PASS — 2 tests (`manifest_covers_every_asset_with_matching_bytes_and_digest`, `sha256_matches_standard_vectors`) |
| Manifest rows vs table entries | 26 = 26 (3 first-party, 22 KaTeX, 1 Mermaid) |
| Digests recomputed on disk (independent of Rust) | **26/26** byte count and SHA-256 match |
| Digests recomputed over HTTP from the packaged binary | **26/26** status, bytes, digest, `Content-Type`, `Content-Length`, `Cache-Control: no-store` |
| `HEAD` on every asset | **26/26** `200` with headers and an empty body |
| Routing edges | `GET`/`HEAD` `/assets/nope`, `/assets/`, `/assets/app.css/` -> `404` (6/6); `PUT`/`DELETE`/`OPTIONS`/`POST` on an asset -> `405`; `?x=1` query -> `200`, identical bytes |
| Offline reference scans (`t143_`, `t24_`, `t40l_`, `t25l_`, `t26k_`, `t57k_`, `t81l_`) | PASS — no `.map`, no `@import`, every `url(` a bare relative `fonts/...` resolving to a served WOFF2 sibling, no `sourceMappingURL` |
| `katex.min.css` font references | 60 `url(` over 20 `@font-face`: 20 `.woff2` (served), 20 `.woff` and 20 `.ttf` (expected `404`, ADR-0025 decision 4) |
| Asset module filesystem-free | PASS — `t143_asset_module_is_filesystem_free`; `grep -nE 'std::fs\|File::open\|PathBuf\|Path' src/browser/assets.rs` prints nothing |

### CSP, width, and navigation (HTTP level)

- Both literals were compared byte-for-byte with `pub const CSP` / `pub const
  CSP_DIAGRAM`. Strict was observed on `/`, `/?type=req`, `/?type=task`,
  `/doc/<no-math>`, `/doc/<no-math>/edit`, `/doc/<math-only>`, `/nope`, a PlantUML-only
  page, and an over-cap fence page (9/9). The retired `/config`, `/lifecycle`, and
  `/skills` routes are `404` and were excluded (ADR-034).
  Diagram was observed only on pages with a renderable `mermaid` figure (3/3).
- No `Content-Security-Policy` on any `/assets/...` response or any
  `application/json` response. In both literals `script-src` is exactly `'self'` with no
  `'unsafe-inline'`, `'unsafe-eval'`, nonce, or hash; `'unsafe-inline'` occurs once, in
  the diagram literal's `style-src`; the literals differ in exactly one token.
- Width: `/?type=task` alone serves `<div class="app app--full">`; every other shell
  page serves `<div class="app">` and no `app--full`. `app.css` contains
  `.app { max-width: 1040px; ... }` and `.app--full { max-width: none; }`; `app.js` and
  `nav-state.js` never mention `app--full`.
- Navigation: every page carries one toggle and one `<nav id="site-nav"
  aria-label="Document types">`, no `data-nav` and no preset collapsed state, and loads
  `/assets/nav-state.js` synchronously (679 B, manifest digest
  `c12c7fe53d844706c7c085f2c9ad6fc6aa784a3ba236895259b486d993bd961d`). `nav-state.js`
  reads `papyrus.nav` before first paint; `app.js` writes it.
- `papyrus check` from the installed binary: clean corpus exit **0** (`no findings`,
  `{"findings":[],"skipped":[],"repaired":[]}`); duplicate-ID corpus exit **4** with one
  `duplicate-id` finding and both sorted locations. The project hash was identical
  before and after both runs — nothing was written.

### Budget arithmetic

| Quantity | Value |
|---|---|
| Binary (`stat -c %s target/release/papyrus`, equal to the packaged binary) | **8,323,136 B** = 7.94 MiB |
| Ceiling (ADR-0027 decision 6) | 12,582,912 B = 12 MiB |
| Headroom | **4,259,776 B** = 4.06 MiB |
| KaTeX weight | 558,541 B (CSS 23,335 + JS 275,414 + 20 WOFF2 259,792) |
| Mermaid weight | 3,572,661 B |
| First-party assets | `app.css` 17,880 B, `app.js` 12,953 B, `nav-state.js` 679 B |
| Vendor weight | 4,131,202 B; all 26 assets 4,162,714 B |
| `.deb` | 2,379,712 B, SHA-256 `b3bc9b729b2b3097a8939a12586896950fab8d738a79f762acb7f56ccf565ead` |

The relationship holds: pre-Mermaid 4,730,296 B + Mermaid 3,572,661 B + the client and
Rust delta -> 8,323,136 B measured.

### Quality gates

| Gate | Result |
|---|---|
| `cargo build --release` (Rust 1.99.1) | PASS — `target/release/papyrus` |
| `cargo build --release --offline` | PASS — no network needed |
| `target/release/papyrus --help` lists `check`, `browser`, `search`, `task`, `milestone`, `release`, `import` before packaging | PASS |
| `cargo test` | PASS — 1,044 passed, 0 failed, 23 ignored |
| `cargo clippy --all-targets -- -D warnings` | PASS — 0 warnings |
| `cargo fmt --check` | PASS |
| Offline `unshare -rn` (loopback up) | PASS — outbound TCP failed (exit 7); list, board, math page, diagram page, 26/26 assets, and `check` succeeded over loopback |
| Installed-binary run from an empty working directory with an asset-free project (`env -i`) | PASS — 26/26 assets |
| No new crate or build step | PASS — `Cargo.lock` 36 packages, no `build.rs`, no `npm`/`node` reference in `scripts/` or `packaging/`, 29 committed vendor files |
| Man page | Absent, as expected (NFR-03.003 unclaimed, ADR-0021 `Proposed`) |

### Findings

| ID | Finding | Severity | Disposition |
|---|---|---|---|
| T149-1 | §3.6 item 4 said the build fails the budget check above 12,582,912 B, but no such automated check exists. Real when first observed; resolved in `676128c`, which reworded item 4 to a recorded release check: "This is a recorded release check; the executable size must remain within the budget, measured with `stat -c %s target/release/papyrus`." The only remaining "fails the build" sentence is item 1 (manifest row or byte/digest mismatch, enforced by the `cargo test` manifest test). | Low | Resolved by the §3.6 rewording; quoted text verified against the current file. |
| T149-2 | The `.deb` grows 1.10 MB over the stale package, slightly above ADR-0026 decision 12's 0.75–0.98 MB estimate. | Informational | No package-size budget exists; recorded only. |

No defect was found in the packaged artifact, the asset contract, the CSP literals, the
width or navigation markup, or `papyrus check`, so no code, test, or packaging change
was required by this gate.

### Recorded limitations

- **Browser-observed behaviour is TASK-150.** This gate asserts packaging and HTTP-level
  facts only. KaTeX and Mermaid rendering with the bundled fonts, the diagram type
  matrix, hostile fixtures, navigation collapse and persistence in a real browser,
  JavaScript-disabled operation, `document.scrollWidth == clientWidth`, and the
  `check --fix` loop are verified in the dated
  `## TASK-150 — Browser runtime verification (2026-11-16)` section below.
- Debian Forky amd64 only (ADR-0003, NFR-03.002); no other release or architecture is
  claimed.
- `git ls-files src/browser/vendor` was not run because the verification session
  permitted only `git add`/`git commit`. The committed-assets claim is supported by
  `git status --porcelain` (no untracked or modified path under `src/browser/vendor`),
  the on-disk listing of 29 vendor files, and a `.gitignore` that covers only `target/`
  and `.tmp/`.
- `scripts/build-deb.sh` still reuses an existing `target/release/papyrus`; the
  mitigation remains `cargo build --release` first, asserting `--help`, and proving the
  packaged SHA-256 equals the built one before recording the package.

### Measurement provenance

- The artifact was built and packaged at 2026-11-16 10:29–10:30 CET from the working
  tree holding commits up to `cc20dd0` plus the then-uncommitted
  `src/browser/app.css`, `src/browser/math.rs`, and `src/browser/vendor/MANIFEST.md`
  changes; `git rev-parse HEAD` reported `676128cf23a612ed4e06723554a35ce21275ad8a`
  when the run started (checklist item 3).
- A concurrent session advanced `HEAD` to `cc20dd0` during the run and, at 10:43 CET,
  started uncommitted page-level diagram-numbering work in `src/browser/math.rs`,
  `src/browser/render.rs`, and `tests/browser_diagram_cli.rs`. Those files belong to
  another ticket and are outside this gate's scope. The binary and `.deb` SHA-256
  values above describe the measured artifact, not a rebuild of that later tree.
- Re-running `cargo test` on the later tree at 10:46 CET passed: **1,046 passed,
  0 failed, 23 ignored**. Any rebuild after further source changes will change the
  binary SHA-256 by design; the measured headroom (about 4.06 MiB) is unaffected by
  small client-side changes.

## TASK-150 — Browser runtime verification (2026-11-16)

**Supported platform statement.** Debian **Forky amd64** is the supported and verified
release target (ADR-0003, NFR-03.002). No other Debian release or architecture is
claimed.

This section is the durable evidence for the end-to-end diagnostics, browser-shell, and
rendering gate. It records measured browser and package behaviour only; no product
source, test, asset, packaging, requirement, business rule, ADR, or specification was
changed by TASK-150. Scratch fixtures and downloaded browsers lived under
`./.tmp/task150/` and were removed after the run.

### Environment and artifact identity

| Item | Value |
|---|---|
| Date and timezone | 2026-11-16T11:20+01:00 (CET) |
| Host | Debian GNU/Linux forky/sid, amd64 (x86_64) |
| rustc / cargo | 1.99.1 (58cf8f9c3 2026-10-13) / 1.99.1 (5f94df478 2026-08-27) |
| HEAD at build | `4a54cc1905ea158473d2ff024c4273ddd2b64c55` |
| Built binary | `target/release/papyrus`, **8,323,296 B**, SHA-256 `f306d2223ebd9404f742baf27b4cb2365959ccab575da7048414a8dfe72418d1` |
| Package | `papyrus_0.1.0_amd64.deb`, **2,379,448 B**, SHA-256 `874d03de512dc8f6cb96e6c362e9beb27c8bfdf43209b8e630fd961b536d27c5` |
| Packaged binary | Same SHA-256 and size as the built binary (`dpkg-deb --fsys-tarfile ... | tar -xO ./usr/bin/papyrus`) |
| Isolated install | `dpkg --instdir=./.tmp/task150/inst --admindir=./.tmp/task150/adm --force-depends`; exactly `./usr/bin/papyrus` (0755) and `./usr/share/doc/papyrus/copyright` (0644) |
| Host dpkg state | `install ok installed` before and after; unchanged |

**Supersession note.** TASK-149 measured binary **8,323,136 B** / SHA-256
`2a71c1af6a1afc581540d51043145b731ce81621114f9a1a248d148ac2d5031e` and `.deb`
**2,379,712 B** / SHA-256 `b3bc9b729b2b3097a8939a12586896950fab8d738a79f762acb7f56ccf565ead`
at `HEAD 676128c`. Commits `a718f53` and `cfd4804` (diagram focus markup and
page-continuous numbering) landed afterwards, so the artifact was rebuilt. The new
binary SHA-256 differs by design and the values above supersede TASK-149's for the
release vehicle. TASK-149's dated section is left intact; its package checks were
re-run as a delta: two-file contents, four `Files:` paragraphs, 8,000 B copyright
(SHA-256 `4610a7abd140648d1e2c136a38cc150c58904396fbbcf7f60348c02a0ca4fc3c`),
26/26 asset digests, 26/26 `HEAD`, budget, offline `unshare -rn`, and absence proofs all
passed. Binary growth over TASK-149: **+160 B**; `.deb` delta **−264 B**.

### Browser matrix

| ID | Browser | Binary | Version | Role |
|---|---|---|---|---|
| M1 | Firefox ESR (installed) | `/usr/lib/firefox-esr/firefox-esr`, SHA-256 `3c87b278879fd1d6a4ee027e0b111afb7d03cc8786c8147ee11a6eb60774c27e` | 153.5.1esr | full checklist; WebDriver BiDi at DPR 1, plus geckodriver 0.36.0 for focus/keyboard |

Supported browser under NFR-05.001 as amended 2026-11-16 is **Firefox 115+ only**.
M2 (older ESR at/above 115), M3 (Edge 130) and M4 (Safari 18.x) are out of scope
and are not executed; no owner-manual matrix and no browser result is claimed for
them. All browser-observed evidence in this section is from M1 (Firefox 153.5.1esr)
only. The Firefox 100 floor probe and the Mermaid CSSStyleSheet shim are obsolete:
the shim was removed (commit 4a54cc1) and DEF-1 is no longer a defect.

### Results by section

| Section | Result | Key measurements |
|---|---|---|
| A. Diagnostics (1–12) | **PASS 12/12** | clean `no findings`/exit 0; dirty exit 4 with both findings in specified order; JSON shape; confirm/cancel/non-TTY/`--force`; idempotent second run; broken configuration followed by static findings; `task list`/`browser`/`search` exit 4 while `config`/`init --force` repair; no-write hashes unchanged |
| B. Shared list and board (13–19) | **PASS** | 17/17 family list routes plus board; one shared table in a labelled scroll region; headers `ID, Title, Status, Release/Group, Created, Updated, Actions`; grouped children ascending with `data-child-of`; empty note state `0 notes` with no table; board `app app--full` 1,428 px = client width at 1,440; capped routes 1,040 px |
| C. Navigation and width (20–30) | **PASS (M1)** | M1 7/7 default and 7/7 collapsed; first-frame probe collapsed; toggle/persistence/history pass; invalid/missing/throwing storage expand; 15/15 JS-disabled samples on M1 and M2; keyboard Enter/Space and visible focus ring pass; M1 matrix **98/98**, max overflow 0; M2 matrix **36/36 no-overflow** (8 samples requested 320 px but Firefox exposes minimum 438 px client width); `.main` content width 540 at 600 px and 992 at 1,440 px collapsed; board 1,380 px collapsed; storage matrix 18/18 on M1 and 18/18 on M2 |
| D. Math (31–39) | **PASS** | NOTE-001 4/4 formulas with annotation = `data-tex`, `.katex-html` `aria-hidden="true"`, 2 display + 2 inline; REQ-001.001 1/1; 3 Woff2 faces requested (KaTeX_Math-Italic 16,740 B, Main-Regular 26,572 B, Size2-Regular 5,508 B), no `.woff`/`.ttf`/external; aborted and throwing-stub runs mark 4/4 and restore source; malformed NOTE-030 renders KaTeX error output; curated fields contain no `data-tex`; CLI `task view`/`search` retain `$t=1$`; CSP listener saw zero script/font/img/default violations |
| E. Diagrams (40–50) | **PASS** | 10/10 Mermaid figures render at 1,440/320/1,920 (flowchart, graph, sequence, class, state, er, gantt, pie, journey, gitGraph); PlantUML 3/3 keep source with exact note; 5 literal fences stay text; unknown + malformed fail closed, `flowchart-elk` renders; over-limit + valid isolation; missing bundle marks 10/10; hostile fixture executes nothing; caps 100,001 B and 2,001 B with exact note; wide diagram viewBox 3,524.33 renders 284 px at 320 and 721 px at 1,920; 10/10 keyboard regions with unique names; all HTTP requests same-origin, Mermaid requested once, 10/10 render |
| F. Security and CSP (51–56) | **PASS** | CSP matrix 16/16 exact; strict/diagram differ only in the `style-src` token; no CSP on assets or `/api/requirements`; browser saw no script/font/img/default violations and Mermaid's 10 inline `<style>` blocks were honoured; hostile math/diagram introduced no script, handler, `javascript:` URL, remote image, or navigation; required headers present, one CSP on HTML, none on the asset |
| G. Package (57–62) | **PASS** | `cargo test` **1,046 passed, 0 failed, 23 ignored**; clippy 0 warnings; fmt clean; package two files only; copyright 8,000 B identical, four `Files:` paragraphs; assets **26/26** bytes/digest/type/length/no-store/empty HEAD; binary 8,323,296 B within 12,582,912 B, headroom **4,259,616 B (4.062 MiB)**; offline build and `unshare -rn` pass; `/usr/share/papyrus`, `/etc`, `/usr/share/man` absent |
| H. Documentation and release (63–67) | **PASS** | README and BR-001/BR-005/BR-016/BR-016 clauses match the observed surface; specification clauses match; this section added; no release milestone or release ticket created by m-21 |

### Recorded limitations

- **Browser scope.** Under NFR-05.001 as amended 2026-11-16, only Firefox 115+ is
  supported. M1 (Firefox 153.5.1esr) is the only browser exercised; no Edge 130,
  Safari 18.x, or older-Firefox result is claimed, and no owner-manual matrix is
  outstanding. AC #3–#6 and #9 are satisfied on the strength of M1 alone.
- **Headless M1 focus.** BiDi reports `document.hasFocus() == false`, so focus-visible
  and keyboard activation were re-run with geckodriver 0.36.0 on the same Firefox
  153.5.1esr binary; Tab, Enter, Space, and a solid focus ring passed there.
- **Historical M2 clamp (no longer in scope).** The earlier M2 (Firefox 102) run
  recorded 8 width-matrix samples at 438 px because geckodriver enforces a 450 px
  minimum outer width. M1 covers exact 320 px; no exact-320 gap remains for the
  supported browser.

### Defects

No open defects. DEF-1 (Firefox 100.0 Mermaid rendering failure) is obsolete under
NFR-05.001 as amended 2026-11-16: the supported floor is now Firefox 115+, the
Mermaid CSSStyleSheet compatibility shim was removed (commit 4a54cc1), and the
Firefox 100 probe is no longer in scope. No product source, test, asset, packaging,
requirement, business rule, ADR, or specification was changed by TASK-150.

### Release vehicle

`backlog milestone list --plain` shows m-21 as the only active milestone with an open
task (`13/14 done`), and `backlog task list --milestone m-21 --plain` shows TASK-150 as
its last open task. No release milestone and no release ticket were created by m-21.
`papyrus release list` on the fixture project returns only fixture records REL-001 and
REL-002 (`planned`), not backlog release records. Decision: release notes and a release
ticket remain out of scope for TASK-150; the owner chooses the vehicle. Reported to
`@orchestrator` in a TASK-150 comment.
