---
id: TECH-072
title: "Process-Global Cache Removal and the Serving Snapshot"
created_date: '2026-11-16T17:46:55Z'
updated_date: '2026-11-21T00:29:39Z'
labels: []
references:
  - ADR-0005
  - ADR-0006
  - ADR-0016
  - ADR-0017
  - ADR-0019
  - ADR-0020
  - ADR-0036
  - ADR-0037
  - REQ-001.004
  - REQ-007.001
  - REQ-007.002
  - REQ-007.003
  - REQ-007.004
  - NFR-007.001
  - NFR-007.002
status: verified
---
# TECH-072 Process-Global Cache Removal and the Serving Snapshot

## Description
## Purpose

No process-global descriptor cache, interning table, or leaked slug survives.
The registry owns every descriptor it publishes, and a command name and record
descriptor are owned values with no `'static` requirement.

The browser has exactly one long-lived cache: the immutable `ServingSnapshot`
defined by ADR-037. It owns the effective `Config`, the `ArtifactRegistry`, the
`BrowserRoutes` table, and the input fingerprint that produced them. It is
invalidated by the configuration file's modification time and byte length,
published by one atomic swap, and never replaced by a failed reload. Record
content is not cached across requests; a request may build one display-only
record snapshot for its own lifetime.

## Current state and target

| Concern | Current | Target |
|---|---|---|
| Descriptor storage | A `OnceLock<Mutex<BTreeMap<..>>>` and leaked per-slug values | Owned by the `ArtifactRegistry`; no process-global map |
| Descriptor lifetime | `&'static` values and `Box::leak` | Owned or `Cow` values with no `'static` bound |
| Command name | A leaked slug string for `clap::Command::new` | An owned `String` from the descriptor |
| Server configuration | Loaded and validated on every handler call | Read from one immutable `ServingSnapshot` |
| Invalidation | No registry or route reload; configuration re-read per request | Configuration fingerprint checked once per request; atomic rebuild on change |
| Record reads | Each reference resolution rescans directories and re-parses | One request-scoped record snapshot for display; writes still read the live file |

## Design

### 1. Ownership model: the registry owns descriptors

- `RecordDescriptor` stays one type. Its configuration-derived fields are owned
  values, or `Cow` values that may borrow a static literal but are not required
  to.
- `TypeDescriptor.record` owns the document record descriptor for each
  configured type, including a user-defined type.
- `ArtifactRegistry::record_descriptor(&self, slug) -> Option<&RecordDescriptor>`
  keeps its signature; borrowing from the registry is the only way to get a
  document record.
- `generic_document_descriptor` and `GENERIC_DESC_CACHE` are deleted. A
  user-defined type builds its record inline from the descriptor and the
  configuration defaults.
- `DocumentModel`, the role field sets, and the field sequence are descriptor
  data. No second static table holds document fields, headings, options, or
  role sets.
- `clap`'s `string` feature is enabled so `Command::new` accepts a `String`.
  `document_command` passes the descriptor's owned slug; no `Box::leak`
  remains. No crate is added; `clap_builder` is already a dependency.
- There is no cache and no static storage for document records: two registries
  built from two configurations each describe only their own type set, and the
  same registry returns field-equal descriptors on every call.

### 2. The serving snapshot

`ServingSnapshot` owns exactly four things:

1. the effective `Config`, loaded and validated per ADR-0019;
2. the `ArtifactRegistry` built from that configuration;
3. the `BrowserRoutes` table derived from that registry;
4. the input fingerprint that produced all three.

Each part is constructed once and is immutable after publication. `ServerState`
holds the current snapshot behind one swap point, a single-writer,
multi-reader cell such as `RwLock<Arc<ServingSnapshot>>`, and holds no second
copy of any part. The former registry, routes, and configuration accessors are
replaced by one snapshot accessor.

A request takes exactly one snapshot handle when it starts and passes it to
routing, handlers, and rendering. Every configuration read, descriptor lookup,
and route lookup in that request resolves through that handle. Because the
snapshot is immutable and shared by `Arc`, a request cannot combine one
configuration version's registry with another version's route table, and two
concurrent requests cannot see a half-written version.

The snapshot is server state only. It is created after the listener binds and
lives until the process exits. No CLI command reads or writes it; the CLI keeps
its per-invocation configuration load and its once-per-process command tree.
Nothing is cached across processes and nothing is persisted.

### 3. Invalidation

A snapshot records a fingerprint for every input file that can change it. A
fingerprint is the file's modification time (seconds and nanoseconds) plus its
byte length, read with `std::fs::metadata`; no content is read.

| Input | Source | Fingerprint | Snapshot parts it invalidates |
|---|---|---|---|
| Configuration file | The path selected at startup, fixed for the process | mtime + length | `Config`, registry, routes |
| Registry inputs | The configuration file only; the seed table is compiled into the binary | The configuration fingerprint | Registry, routes |
| Route table | Derived in-process from the registry; no file of its own | None | Routes, only together with the registry |

The registry and the route table are invalidated together, always. A derived
structure whose inputs are the configuration and the registry receives no
independent fingerprint, so there is no state in which one of the three parts is
current and another is not. A future input file joins the fingerprint table in
the same change that makes the loader read it.

The fingerprint check runs once per request, before routing. If every
fingerprint matches the current snapshot, the request performs only those
`stat` calls and no rebuild. If any fingerprint differs, including a file that
has disappeared or become unreadable, the request attempts one reload.
Invalidation is change-driven, not request-driven: the number of requests does
not affect the cost of loading or validating configuration.

### 4. Reload and fail-closed publication

A reload is the startup construction path, in the startup order, with no
shortcut: load the selected configuration, validate it fully, build the
registry, then derive the route table. Every gate that runs at startup runs
again.

Publication is one atomic swap. The fully built snapshot replaces the current
`Arc` in a single write-lock acquisition; there is no per-field update, no
in-place mutation, and no partial publication. A request that already holds the
previous handle finishes on it; a request that starts after the swap gets the
new one. The previous snapshot is dropped when its last handle is released.

Reloads are serialized and de-duplicated. A request that observes a changed
fingerprint takes the reload lock, re-checks the fingerprint under the lock, and
skips the rebuild when the snapshot is already current. A request that cannot
take the lock serves the current snapshot without waiting.

Reload is fail-closed in the sense that matters for serving: a reload error
never becomes the served state. If loading, validation, registry construction,
or route derivation fails, the new snapshot is discarded, the previous snapshot
remains current, and the request is served from it with its normal status code
and page. The server does not exit, does not serve an error page for a
configuration problem, and does not fall back to defaults. The failure is
reported as one `warning: ...` line on standard error; no new HTTP status,
header, page, endpoint, JSON member, or exit code is introduced, and no
configuration error content is sent to the client. A broken configuration is
reported once per distinct input state; a later change retries the reload and
emits a new warning only if the retry also fails.

Startup has no last-known-good snapshot, so fail-closed serving does not apply
to it. The first snapshot must build or `papyrus browser` aborts: exit 1 for a
missing configuration and exit 4 for an invalid one.

Settings consumed before the listener binds — host, port, and open — are read
from the startup configuration only. A reload changes pages, navigation, lists,
and routes, not the bound socket, the concurrency cap, or the `Host`/`Origin`
allowlist. A change to `browser_host`, `browser_port`, or `browser_open` takes
effect at the next browser start.

### 5. Request-scoped record snapshot

A request may build one record snapshot for its own lifetime: a map from record
ID to the record it resolves to, covering the enabled descriptors' active
directories. It is built on first use, so a request that resolves no IDs does
not build it. The first lookup lists each enabled descriptor's directory once
(file names only, no parsing); after that an ID lookup is one map read plus, at
most, one file parse, and each resolved record is parsed once and memoized for
the rest of the request. It is local to the request, not held in `ServerState`,
not shared with another connection, and dropped when the response is written.

The request snapshot serves title and reference resolution and related-record
display: milestones, releases, dependencies, requirement references, groups,
and superseded-by lookups all read it. A page with N references performs one
directory listing per enabled type and one parse per distinct record,
regardless of N.

The request snapshot is not a cache of record content across requests. Every
request re-reads the records it displays, and a successful save is proven by
the redirect target's fresh read. Nothing written and no staleness decision
comes from the request snapshot. A `POST` resolves its target ID through the
snapshot (the ID set only), then re-reads the live file with
`record::store::read_document`, computes the version token from the live bytes,
applies the patch to that live document, and writes through
`record::store::write_atomic`. The snapshot never supplies bytes for a write
and never supplies or validates a version token. The optimistic-concurrency
contract is unchanged.

Archived records and disabled types stay invisible by construction: only
enabled descriptors' active directories are indexed, so an archived or unknown
ID resolves exactly as before and keeps the same not-found page.

### 6. Call sites

- `ServerState` exposes one snapshot accessor; no handler loads configuration
  per request.
- Routing, list, view, edit, conflict, not-found, JSON, and asset handlers take
  the snapshot handle and resolve descriptors and routes through it.
- A reload rebuilds configuration, registry, and routes in one function and
  publishes one snapshot; there is no accessor for one part without the others.
- The request record snapshot is used only by display resolution. No write path
  reads it for content or for a version token.
- The CLI keeps its per-invocation configuration load and its owned command
  tree; it does not use the serving snapshot.

## Behaviour preservation

- **Help and usage text.** Byte-identical: same command names, order, about
  lines, verbs, options, and exit codes. The help golden suite passes without
  regeneration.
- **Dispatch.** A config-defined slug still resolves through
  `dispatch -> registry.descriptor(name) -> document::run`, and its record
  descriptor carries the same slug, prefix, directory, status key, and stem
  matching.
- **Record I/O and migration.** `resolve_read_directories`, `matches_stem`,
  `configured_prefixes`, and `declared_statuses` keep their bodies; the record
  values are field-identical to the removed static values, so front-matter
  seeding, adoption, archive paths, and migration output do not change. Legacy
  `deprecated/` folders stay inert.
- **Disabled types.** Still absent from the command tree and still skipped by
  migration.
- **Errors.** Validation and exit-code mapping (REQ-001.004) are untouched.

## Test plan

1. **Leak guard.** A production-line source scan fails on `Box::leak`,
   `mem::forget`, `GENERIC_DESC_CACHE`, `generic_document_descriptor`, or
   `OnceLock`/`OnceCell`/`LazyLock` in the descriptor and command-tree modules.
2. **Double build.** In one process, registry A and registry B are built from
   different configurations; each tree and descriptor matches its own registry,
   repeated calls are field-equal, and no cross-registry state exists.
3. **Snapshot generation.** A test asserts that every part of a served page
   comes from one snapshot generation, that a request holds one handle, and
   that concurrent requests cannot observe a partially built snapshot.
4. **Invalidation.** A configuration edit is picked up by the next request; an
   unchanged file causes only the fingerprint check; two concurrent requests
   cause one rebuild; an edit to `browser_host`, `browser_port`, or
   `browser_open` does not rebind the socket.
5. **Fail-closed reload.** An invalid configuration keeps the previous snapshot
   serving, emits one warning, adds no status code or page, and retries only
   when the input changes again.
6. **Request snapshot.** A reference-heavy page performs one directory listing
   per enabled type and one parse per distinct record; a stale version token
   still returns `409`; the written bytes and token come from a live read.
7. **Existing suites.** Help golden, dispatch, document, no-drift, typed CLI,
   migration, native-layout, search, and browser suites pass unchanged.
8. **Quality gates.** `cargo build`, `cargo test`, and
   `cargo clippy --all-targets -- -D warnings` are clean.

## Assumptions

- No requirement or business rule changes; the descriptor and snapshot
  contracts preserve record behaviour.
- Enabling clap's `string` feature is acceptable and adds no crate.
- Descriptor identity required by tests is field equality, not pointer
  equality.
- The snapshot uses only the standard library: `std::fs::metadata`,
  `std::sync::RwLock`, and `std::sync::Arc`.

## Risks and mitigations

| Risk | Mitigation |
|---|---|
| Dropping `Copy` breaks a value use at a distance | The compiler catches every value use; a clone is semantically identical |
| `Cow` comparisons silently change matching | `Cow<str> == &str` exists; stem-matching and descriptor assertions run unchanged |
| A mixed snapshot is observed | Publication is one `Arc` swap and snapshots are immutable; a generation test covers every page part |
| Repeated failed reloads | Reloads are serialized and the failing fingerprint is remembered; one attempt per distinct input state |
| A record response becomes stale within a request | The request snapshot is display-only; writes re-read the live file and compute the token from live bytes |
| A process-global cache reappears | Source guard plus the double-build test |
| Coarse timestamps miss an edit | The byte-length component narrows the case; the residual is the same class ADR-0006 accepts |

## Out of scope

- Changing descriptor content, the registry model, or record behaviour.
- Adding any crate; only the clap feature flag changes.
- Introducing any cache, static map, or interning table for descriptors or
  slugs outside the serving snapshot.
- Optimising per-call registry construction in migration.
- Editing ADRs, requirements, or business rules.
