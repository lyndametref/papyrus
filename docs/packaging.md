# Papyrus Build, Packaging, and Installation

## Supported platforms

- Architecture: `amd64`.
- Supported Debian release: **Debian Forky only** (per ADR-0003; supersedes ADR-0002).
- Debian Trixie and any other Debian release are not claimed or verified as supported.
  Adding a release to the supported set requires an explicit verification pass and a
  follow-up ADR/requirement update (ADR-0003, NFR-03.002).

## Prerequisites

- Rust 1.99.1 via rustup (pinned by `rust-toolchain.toml`).
- Cargo 1.99.1.
- `dpkg-deb` (Debian package tooling).

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

- `--help` / `-h`: print help and exit `0`.
- `--version`: print `papyrus <version>` and exit `0`.
- unknown flags/arguments: print usage to stderr and exit `2`.

## Packaging

```sh
./scripts/build-deb.sh
```

This produces `papyrus_<version>_amd64.deb` in the project root (for example
`papyrus_0.1.0_amd64.deb`), where `<version>` is taken from `Cargo.toml`.

Inspect the package:

```sh
dpkg-deb -I papyrus_0.1.0_amd64.deb
dpkg-deb -c papyrus_0.1.0_amd64.deb
```

## Install

```sh
sudo dpkg -i papyrus_<version>_amd64.deb
```

After installation, `papyrus` is available on `PATH` (installed to `/usr/bin/papyrus`).

For verification, the package may be installed in an isolated environment (container/chroot/VM
or `dpkg --instdir/--admindir` under `fakeroot`) so the host is never modified:

```sh
mkdir -p /tmp/scratchpad/inst /tmp/scratchpad/adm
# initialize an empty dpkg database under admindir
mkdir -p /tmp/scratchpad/adm/info /tmp/scratchpad/adm/updates /tmp/scratchpad/adm/parts /tmp/scratchpad/adm/triggers
touch /tmp/scratchpad/adm/status /tmp/scratchpad/adm/available
fakeroot dpkg --instdir=/tmp/scratchpad/inst --admindir=/tmp/scratchpad/adm \
  --force-depends -i papyrus_<version>_amd64.deb
```

## Uninstall

```sh
sudo dpkg -r papyrus
# or, to also remove configuration files:
sudo dpkg --purge papyrus
```

Isolated uninstall:

```sh
fakeroot dpkg --instdir=/tmp/scratchpad/inst --admindir=/tmp/scratchpad/adm --purge papyrus
```

## Smoke test (installed)

Run against the installed binary (isolated Forky-equivalent environment):

```sh
papyrus --help          # exit 0, prints usage
papyrus --version       # exit 0, prints "papyrus <version>"
papyrus --dir <tmp> init <name>   # creates papyrus.config.yml + structure
papyrus req create-group --title <title>   # prints REQ-<n>
papyrus req create <title> --group <id>    # prints <group>.<seq>
papyrus req list       # active records by id/title, id order, no deprecated
papyrus req view <id>  # full persisted text; unknown id exits 3
papyrus req edit <id>  # updates only supplied fields
papyrus req deprecate <id>  # moves to deprecated/, hidden from list
papyrus req restore <id>    # returns with same id, reappears in list
# same lifecycle for nfreq (create-group, create --group, list, view, edit,
# deprecate, restore)
```

Exit statuses (TASK-2 contract): 0 success, 1 IO/boundary, 2 usage, 3 not found, 4 validation.

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

## Verification results (TASK-4)

- Date: 2026-11-02
- Environment: build host with Rust 1.99.1 (rustup), `dpkg-deb` 1.23.11 (amd64);
  isolated install via `fakeroot dpkg --instdir/--admindir` (host untouched).
- `cargo build --release`: PASS
- `cargo test`: PASS (41 tests: 5 unit + 20 cli + 5 milestone_smoke + 11 req_cli)
- `cargo clippy --all-targets -- -D warnings`: PASS (0 warnings)
- `cargo fmt --check`: PASS
- `./scripts/build-deb.sh`: PASS -> `papyrus_0.1.0_amd64.deb`
- Package metadata: `Package: papyrus`, `Architecture: amd64`, `Version: 0.1.0`,
  copyright `GPL-3.0-or-later`, `Depends: libc6 (>= 2.34)`: PASS
- Install/uninstall (isolated): PASS
- Smoke test (installed): PASS (see table above)
- Overall: PASS

Supported release claim: Debian Forky only (amd64). No Trixie or later release is
claimed or verified (ADR-0003, NFR-03.002).

## Runtime dependency license audit

Reference: TASK-1 license audit (unchanged). No new runtime dependencies were introduced by TASK-2/TASK-3 beyond those already audited (yaml_serde and serde_json were already audited). Audit re-run not required.

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

All runtime dependencies are permissively licensed (MIT / Apache-2.0 / Unlicense / BSL-1.0)
and permit linking in a GPL-3.0-or-later application. No incompatible runtime dependency
remains.

Build-time/proc-macro dependencies (excluded from audit scope): clap_derive,
heck, proc-macro2, quote, syn, unicode-ident.
