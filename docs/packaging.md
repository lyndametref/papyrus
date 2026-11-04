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

## Packaging

> **Mandatory build step.** `scripts/build-deb.sh` reuses an existing
> `target/release/papyrus` and rebuilds only when the binary is missing. Always run
> `cargo build --release` immediately before packaging, and confirm the fresh binary
> exposes the current feature set, otherwise a stale binary can be shipped.

```sh
cargo build --release
./target/release/papyrus --help        # must list search and browser
./scripts/build-deb.sh
```

The script produces `papyrus_<version>_amd64.deb` in the project root (for example
`papyrus_0.1.0_amd64.deb`), where `<version>` is taken from `Cargo.toml`. The package
version is not bumped by packaging; release versioning belongs to release
orchestration.

Inspect the package:

```sh
dpkg-deb -I papyrus_0.1.0_amd64.deb
dpkg-deb -c papyrus_0.1.0_amd64.deb
dpkg-deb --fsys-tarfile papyrus_0.1.0_amd64.deb \
  | tar -xO ./usr/share/doc/papyrus/copyright | grep GPL-3.0-or-later
```

Expected metadata and contents:

- `Package: papyrus`, `Version: <Cargo version>`, `Architecture: amd64`,
  `Section: utils`, `Priority: optional`, `Depends: libc6 (>= 2.34)`,
  `Maintainer`, `Homepage`, and a `Description` naming Papyrus as a project-level
  backlog and documentation manager.
- Copyright file in Debian copyright format declaring `GPL-3.0-or-later`
  (matches `Cargo.toml`; NFR-06.001).
- Only `/usr/bin/papyrus` (mode `0755`) and
  `/usr/share/doc/papyrus/copyright` (mode `0644`), plus their directories. No
  `.css`, `.js`, `.html`, template, configuration, or man page is packaged: the
  browser assets are compiled into the binary with `include_str!` and served from
  memory (`/assets/app.css`, `/assets/app.js`).

## Install

```sh
sudo dpkg -i papyrus_<version>_amd64.deb
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
  -i papyrus_<version>_amd64.deb
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
- `GET /assets/app.css` (`text/css`) and `GET /assets/app.js`
  (`text/javascript`) return `200` with `Cache-Control: no-store` and non-empty
  bodies.
- 320 px markup hooks: viewport meta, sidebar toggle with `aria-expanded` and
  `aria-controls="type-list"`, responsive footer hooks, and the narrow-viewport
  rules in the served CSS.
- No external references: served HTML/CSS/JS contain no `http://`, `https://`,
  protocol-relative remote reference, remote `@import`, `url(...)`, `<img>`, web
  font, or CDN reference; HTML responses carry the documented
  `Content-Security-Policy`.
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

### 3.6 Embedded assets and no runtime file dependency

Run the installed binary with its working directory set to an empty directory and a
project directory that contains no asset files. `search` and `browser` must still
work, and `/assets/app.css` and `/assets/app.js` must be non-empty. The package
contains no asset, template, or documentation files beyond the copyright file, so
the installed binary needs only `/usr/bin/papyrus` and the project directory.

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

Build-time/proc-macro dependencies (excluded from audit scope): clap_derive, heck,
proc-macro2, quote, syn, unicode-ident.
