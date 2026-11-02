# Papyrus Build, Packaging, and Installation

## Supported platforms

- Architecture: `amd64`.
- Supported Debian releases: **Debian Trixie and Forky only**.
- Later Debian releases are not advertised as supported (NFR-03.002 open question).

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
sudo dpkg -i papyrus_0.1.0_amd64.deb
```

After installation, `papyrus` is available on `PATH` (installed to `/usr/bin/papyrus`).

## Uninstall

```sh
sudo dpkg -r papyrus
# or, to also remove configuration files:
sudo dpkg --purge papyrus
```

## Smoke test (installed)

```sh
papyrus --help
papyrus --version
```

Both must exit `0`.

## Runtime dependency license audit

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

All runtime dependencies are permissively licensed (MIT / Apache-2.0) and permit
linking in a GPL-3.0-or-later application. No incompatible runtime dependency
remains.

Build-time/proc-macro dependencies (excluded from audit scope): clap_derive,
heck, proc-macro2, quote, syn, unicode-ident.
