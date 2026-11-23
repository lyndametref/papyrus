---
id: ADR-037
title: "In-server configuration and registry snapshot"
created_date: '2026-11-20T19:30:02Z'
updated_date: '2026-11-20T19:30:02Z'
labels: []
references:
  - RE-010
  - ADR-0005
  - ADR-0006
  - ADR-0016
  - ADR-0017
  - ADR-0019
  - ADR-0020
  - ADR-0036
  - REQ-001.003
  - REQ-007.002
  - NFR-007.001
  - NFR-007.002
  - TECH-007
  - TECH-068
  - TECH-070
  - TECH-071
  - TECH-077
superseded_by: null
---
# ADR-037 In-server configuration and registry snapshot

## Status
accepted
## Decision
## Supersession and amendments

1. This ADR supersedes the no-cache clause of ADR-0017 decision 10, and only that clause. The rest of decision 10 stays in force: the registry is plain data built in-process, and it introduces no new dependency, no new process model, no index file, no background scan, and no serialization format. ADR-0017 keeps its accepted status; this is a clause-level supersession, not a record-level one. ADR-036 retired decision 10's descriptor content with the descriptor decisions it superseded; the constraint quoted here is the deployment rule ADR-037 relies on, and ADR-037 changes only its no-cache clause.
2. This ADR amends, without superseding, ADR-0016 decisions 3 and 9, the per-request re-read risk bullet of ADR-0005, the per-request configuration re-read risk bullet of ADR-0006, and ADR-0020 decision 2. Each amended clause carries a dated "Amended by ADR-037" revision note; no decision is renumbered, softened, split, or reopened, and no unamended clause of those records changes.
3. Nothing else is replaced. ADR-0005's transport, routing, rendering, and security baseline, ADR-0006's optimistic-concurrency and atomic-write contract, ADR-0019's fail-closed validation, and ADR-0036's descriptor model remain in force. Record content is still read from disk on every request; this ADR caches configuration, registry, and route metadata only.

## The serving snapshot

4. The browser serves from one immutable `ServingSnapshot`. It owns exactly four things: the effective `Config` (loaded and validated per ADR-0019), the `ArtifactRegistry` built from that configuration (ADR-0017, ADR-0036), the `BrowserRoutes` table derived from that registry (ADR-0020), and the input fingerprint (decision 8) that produced all three. Each part is constructed once; nothing in a snapshot is mutable after publication.
5. `ServerState` holds the current snapshot behind one swap point — a `RwLock<Arc<ServingSnapshot>>` or an equivalent single-writer, multi-reader cell from `std` — and holds no second copy of any part. Today's `ServerState.registry` field, `ServerState.routes` field, and `ServerState::config()` method are replaced by one snapshot accessor; no handler calls `load_effective_config` per request.
6. A request takes exactly one snapshot handle when it starts and passes it to routing, handlers, and rendering. Every configuration read, descriptor lookup, and route lookup in that request resolves through that handle. Because the snapshot is immutable and shared by `Arc`, a request cannot combine the registry of one configuration version with the route table of another, and two concurrent requests cannot see a half-written version.
7. The snapshot is server state only. It is created after the listener binds and lives until the process exits. No CLI command reads or writes it: the CLI keeps its per-invocation configuration load and its once-per-process command tree (ADR-0016 decisions 3 and 9). Nothing is cached across processes and nothing is persisted.

## Invalidation

8. A snapshot records a fingerprint for every input file that can change it. A fingerprint is the file's modification time (seconds and nanoseconds) plus its byte length, read with `std::fs::metadata`; no content is read to compute it.
9. The complete input set is:

   | Input | Source | Fingerprint | Snapshot parts it invalidates |
   |---|---|---|---|
   | Configuration file | The path selected at startup (`--config`, `PAPYRUS_CONFIG`, or the project default), resolved per REQ-001.003 and fixed for the process | mtime + length | `Config`, registry, routes |
   | Registry inputs | The configuration file only: `ArtifactRegistry::build(&Config)` reads no other file, and the built-in seed table is compiled into the binary (ADR-0036 decision 28) | The configuration fingerprint | Registry, routes |
   | Route table | Derived in-process by `BrowserRoutes::build(&ArtifactRegistry)`; it has no file of its own | None; it has no independent fingerprint | Routes, and only together with the registry |

10. The registry and the route table are invalidated together, always. A derived structure whose inputs are the configuration and the registry receives no independent fingerprint, so there is no state in which one of the three parts is current and another is not. A future input file — a configuration include, a schema file, a second configuration source — joins the fingerprint table in the same change that makes the loader read it, and it invalidates the parts derived from it.
11. The fingerprint check runs once per request, before routing. If every fingerprint matches the current snapshot, the request performs only those `stat` calls and no rebuild; no configuration, descriptor, route, or file-content work is repeated. If any fingerprint differs — including a file that has disappeared or become unreadable — the request attempts one reload (decisions 13-20).
12. Invalidation is therefore change-driven, not request-driven. The number of requests does not affect the cost of loading or validating configuration; only a change to an input file does.

## Reload

13. A reload is the startup construction path, in the startup order, with no shortcut: load the selected configuration, validate it fully (ADR-0019), build the registry (ADR-0017 decision 9, ADR-0036), then derive the route table (ADR-0020 decision 2). Every gate that runs at startup runs again, so a reload cannot produce a descriptor, verb set, status model, or route entry that validation or the registry constructor would reject.
14. Publication is one atomic swap. The fully built snapshot replaces the current `Arc` in the swap point in a single write-lock acquisition; there is no per-field update, no in-place mutation, and no partial publication. A request that already holds the previous handle finishes on it; a request that starts after the swap gets the new one. The previous snapshot is dropped when its last handle is released.
15. Reloads are serialized and de-duplicated. A request that observes a changed fingerprint attempts to take the reload lock; under the lock it re-checks the fingerprint and skips the rebuild when the snapshot is already current, so a rebuild started by an earlier request is never repeated. A request that cannot take the lock because a rebuild is in progress does not wait and serves the current snapshot, so one request may see the previous configuration once. The request that rebuilds serves the new snapshot on success and the previous one on failure. A burst of concurrent requests therefore causes one rebuild, not one per connection.
16. Reload is fail-closed, in the sense that matters for serving: a reload error never becomes the served state. If loading, validation, registry construction, or route derivation returns an error, the new snapshot is discarded, the previous snapshot remains current, and the request is served from it with its normal status code and page. The server does not exit, does not serve an error page for a configuration problem, and does not fall back to defaults.
17. The reload failure is reported through the existing diagnostics path: one `warning: ...` line on standard error, the same channel and form the browser already uses for per-request diagnostics and startup warnings (TECH-007 section 11). The line names the failing input and the error, and the request log under `--debug` is unchanged. No new HTTP status, header, page, endpoint, JSON member, or exit code is introduced, and the browser sends no configuration error content to the client.
18. The failing fingerprint is remembered. A broken configuration is reported once per distinct input state and is not re-reported on every request; when the file changes again — including when it is fixed — the fingerprint differs, the reload is retried, and a new warning is emitted only if the retry also fails.
19. Startup has no last-known-good snapshot, so fail-closed serving does not apply to it. The first snapshot must build or `papyrus browser` aborts with the documented outcome: exit 1 for a missing configuration, exit 4 for an invalid one (ADR-0016 decision 3). Decisions 16-18 govern only reloads after a snapshot exists.
20. Settings consumed before the listener binds — host, port, open — are read from the startup configuration only. A reload changes pages, navigation, lists, and routes, not the bound socket, the concurrency cap, or the `Host`/`Origin` allowlist built from the bind values. A change to `browser_host`, `browser_port`, or `browser_open` takes effect at the next `papyrus browser` start. This limitation already exists today; this ADR makes it explicit rather than changing it.

## Request-scoped record snapshot

21. A request may build one record snapshot for its own lifetime: a map from record ID to the record it resolves to, covering the enabled descriptors' active directories. It is built on first use, so a request that resolves no IDs does not build it. The first lookup lists each enabled descriptor's directory once (file names only, no parsing); after that an ID lookup is one map read plus, at most, one file parse, and each resolved record is parsed once and memoized for the rest of the request. It is local to the request, not held in `ServerState`, not shared with another connection, and dropped when the response is written.
22. The request snapshot serves title and reference resolution and related-record display: `browser::records::resolve_id` currently scans every enabled descriptor directory once per ID and re-parses the match, so a page with N references performs N directory scans and N parses; with the request snapshot the page performs one directory listing per enabled type and one parse per distinct record, regardless of N. Milestone, release, dependency, requirement-reference, group, and superseded-by lookups all read it.
23. The request snapshot is not a cache of record content across requests. ADR-0005's rule that the server keeps no in-memory copy of record content between requests, and ADR-0006's save-then-reread rule, are unchanged: every request re-reads the records it displays, and a successful save is still proven by the redirect target's fresh read.
24. Nothing written and no staleness decision comes from the request snapshot. A `POST` resolves its target ID through the snapshot (the ID set only), then re-reads the live file with `record::store::read_document`, computes the version token from the live bytes (ADR-0006), applies the patch to that live document, and writes through `record::store::write_atomic`. The snapshot never supplies bytes for a write and never supplies or validates a version token. The optimistic-concurrency contract is identical to the one before this ADR.
25. Archived records and disabled types stay invisible to the snapshot by construction: only enabled descriptors' active directories are indexed (ADR-0020 decisions 7 and 8), so an archived or unknown ID resolves exactly as it does today and keeps the same 404 page.

## Why the cache does not reintroduce the staleness risk ADR-0017 avoided

26. ADR-0017 decision 10 refused a cache because the only freshness rule it could offer was the process lifetime: a registry cached at startup, or on first use, could outlive a configuration edit and keep serving a descriptor, verb set, status model, or route that the validated configuration no longer contained. The risk was not "any memory of configuration"; it was "memory with no invalidation rule and no revalidation gate".
27. This ADR removes both halves of that condition. Every input is fingerprinted, so a change is detected by the next request rather than never. Every rebuild runs the same validate-then-build path as startup, so the snapshot can never contain a descriptor that configuration validation would reject, a partially built registry, or a route for a type that does not exist. Publication is atomic, so no request mixes two versions, and a failed reload never replaces a good snapshot.
28. Freshness granularity is the same as the per-request re-read it replaces. Today a configuration edit is picked up by the next `load_effective_config` call, which is at least once per request; with this ADR it is picked up by the next fingerprint check, which is also once per request. An edit made after the check in request N is seen in request N+1 under both regimes, so the staleness window does not grow.
29. Two differences are intentional and both are stricter. A single request uses one configuration: today two `load_effective_config` calls inside one request can return different versions if the file changes in between; now the request holds one immutable snapshot. And a record edit is still seen immediately, because record content is not part of the snapshot (decisions 21-25). The cache covers only data whose only writer is the project's configuration file, and it is invalidated by exactly that file's metadata.
30. The residual limit is mtime granularity, not policy: on a filesystem whose timestamps are coarse, an edit that keeps both the modification time and the byte length unchanged could go undetected until the next change. The length component narrows that case, it is the same class of limit ADR-0006 already accepts for its version token, and a content hash would remove it at the cost of reading the file on every request — which would restore the per-request I/O this ADR removes.

## No new dependency or process model

31. The snapshot uses the standard library only: `std::fs::metadata`, `std::sync::RwLock`, `std::sync::Arc`, and the existing configuration, registry, and route constructors. There is no new crate (RE-010 D3), no file watcher, no background thread, no timer, no index file, no serialization format, and no second parser of configuration or records.
32. Reload work runs on the request thread that observes the change. The browser's one-request-per-connection, 16-connection model (ADR-0005) is unchanged; a rebuild occupies the observing connection, not a process-wide resource.

## Downstream constraints

1. One snapshot owns the configuration, registry, and route table. `ServerState` holds no second copy of any of them, and no handler loads configuration per request.
2. A reload builds in the order configuration, validation, registry, routes, and publishes one fully built snapshot atomically. No partially reloaded snapshot may be observed or served.
3. A reload that fails keeps the last known good snapshot serving and reports one `warning: ...` line on standard error. No new status code, header, page, endpoint, or exit code is added.
4. The fingerprint is modification time plus length of each input file. A derived structure with no input file of its own gets no independent fingerprint, so the registry and the route table can never be invalidated separately.
5. Record content is never cached across requests. The request-scoped record snapshot lives for one request, is discarded with the response, and supplies no write content and no version token; every write re-reads the live record through `record::store`.
6. No new dependency, background thread, file watcher, index file, or serialization format is introduced, and nothing is cached across processes.
7. Startup fail-closed behaviour is unchanged: the first snapshot must build, with exit 1 for a missing configuration and exit 4 for an invalid one.
## Context
MS-031 AC-10 forbids code changes in this milestone, so this ADR records the contract the refactor will implement. RE-010 D11 already fixed the direction — "Keep the mtime config/registry cache" — and D3 forbids a new crate.

The delivered browser keeps no cache at all. `ServerState` in `src/browser/mod.rs` holds a startup `ArtifactRegistry` and a startup `BrowserRoutes`, but its `config()` method calls `commands::load_effective_config` on every use, and nearly every handler uses it: error pages, the inventory page, list pages, detail and edit pages, and every `POST` handler. `load_effective_config` re-reads `papyrus.config.yml` and re-runs the ADR-0019 validation each time, while the `registry` and `routes` fields never notice a configuration edit. The two halves of the server's type model therefore already have opposite freshness rules: the configuration is reloaded far more often than it can change, and the registry and route table never reload at all.

Two further costs are per record rather than per request. `browser::records::build_index` walks a descriptor's directory and parses every file it lists. `browser::records::resolve_id` walks every enabled descriptor's directory once per looked-up ID and re-parses the match, and a view page calls it once for a related milestone or release, once per dependency, and once per reference, so a reference-heavy page performs one full directory scan and parse per resolved ID. NFR-007.001 (inventory p95 under 250 ms at 1,000 records) and NFR-007.002 (view and edit p95 under 100 ms at 1,000 records) are the performance targets, and NFR-007.002 already excludes requests that trigger a configuration reload from its measurement.

Forces:

- A configuration edit must take effect without restarting the browser; the per-request load is the only mechanism that provides this today.
- The type set, verbs, status models, and routes must never disagree with the validated configuration (ADR-0019, ADR-0036, ADR-0020).
- A broken configuration must not take a working server down, must not be served, and must not silently fall back to defaults.
- A request must not mix two configuration versions, which the current repeated `load_effective_config` calls can do.
- Record content must stay uncached across requests (ADR-0005, ADR-0006), and a write must keep reading the live record for its version token (ADR-0006).
- No new dependency, background thread, watcher, index file, or serialization format (RE-010 D3; the surviving clauses of ADR-0017 decision 10).
- Startup fail-closed behaviour is fixed by ADR-0016 decision 3 and must not change.
- Configuration loading orchestration currently lives in `src/commands` and is called from the browser; the snapshot must not become a second loader with different validation.

Alternatives considered:

| Option | Outcome | Reason |
|---|---|---|
| One immutable snapshot of configuration, registry, and routes, invalidated by input modification time and swapped atomically (this ADR) | Selected | Keeps configuration edits live, gives the registry and routes the same freshness as the configuration, and removes the per-request load and per-record config re-reads without adding a dependency. |
| Keep the per-request `load_effective_config` and add no cache | Rejected | The status quo: configuration is re-read and re-validated far more often than it can change, and the registry and routes still never track an edit. |
| Cache at startup only, with no invalidation | Rejected | Exactly the staleness ADR-0017 decision 10 avoided: a configuration edit would leave the server serving a type set the configuration no longer declares until restart. |
| Rebuild configuration, registry, and routes on every request | Rejected | Moves the cost from per record to per request without removing it, and still lets one request observe two versions. |
| Invalidate the route table separately from the registry | Rejected | The table is a pure function of the registry; separate invalidation can only produce a mixed state. |
| File-watcher thread or inotify | Rejected | Adds a background thread and an OS-specific surface for a check that costs one `stat` per input per request. |
| Content-hash fingerprints | Deferred | Removes the coarse-timestamp residual at the cost of reading the configuration file on every request, which is the I/O the snapshot exists to avoid. |
| Cache parsed record content across requests | Rejected | Contradicts ADR-0005's no-cache rule and ADR-0006's save-then-reread and live-token contract. |
## Consequences
**Positive:**

- Configuration is loaded and validated once per change instead of at least once per request and once per record, which is the largest single contributor to inventory and list latency at a few hundred to 1,000 records (NFR-007.001, NFR-007.002).
- A configuration edit takes effect without a restart, and it now moves the registry and the route table with it. Today a configuration edit is visible to code paths that read `Config` and invisible to code paths that read the startup registry, so one page can already mix two type models.
- One request uses one consistent configuration, registry, and route table.
- A detail page resolves N references with one directory pass per enabled type instead of one pass per reference, so reference-heavy pages stop scaling with the number of references.
- A broken configuration can no longer take a running server down or make every page fail: the last known good snapshot keeps serving and the failure is reported once on stderr.
- The snapshot is one object with one construction order, so the registry cannot be rebuilt without the routes, or the routes without the registry.

**Negative:**

- An invalid configuration now produces a warning on stderr and stale-but-valid pages instead of a failing page. That is the intended fail-closed behaviour, but a user who edits the configuration and watches only the browser has no in-page signal; the diagnostics channel is stderr (decision 17).
- A rebuild happens on a request thread, so the request that observes a configuration change pays the load, validation, registry, and route cost. The cost is bounded by configuration size and occurs once per change; NFR-007.002 already excludes such requests.
- Handler signatures grow one parameter (the snapshot handle), touching every browser handler. The change is mechanical but broad.
- `browser_host`, `browser_port`, and `browser_open` no longer even appear to be live: a reload cannot rebind the socket, so those edits take effect only at the next start (decision 20).
- The request-scoped record snapshot can differ from disk within one request if a record is edited between snapshot construction and render. It is display-only by decision 24, but it is a new window a reviewer must understand.

**Risks and mitigations:**

- *Mixed snapshot observed*: publication is one `Arc` swap and snapshots are immutable; readers hold their handle for the request. A test asserts that every part of a served page comes from one snapshot generation.
- *Repeated failed reloads*: reloads are serialized, and the failing fingerprint is remembered, so a broken configuration costs one attempt per distinct input state, not one per request.
- *Coarse timestamp misses an edit*: the length component narrows the case; the residual is documented (decision 30) and is the same class ADR-0006 accepts for its version token.
- *Snapshot used for a write*: the request snapshot is display-only; a contract test asserts the version token and the written bytes come from a live `record::store::read_document`, and a stale token still returns 409.
- *Rebuild cost lands on one request*: bounded by configuration size, once per change, and de-duplicated across concurrent requests.
- *Request snapshot memory*: one file-name entry per active record of every enabled type plus one parsed record per distinct resolved ID, held only for the request and built only when an ID is resolved; the current code already scans the same directories per reference, so peak work does not grow at 1,000 records.
- *Diagnostics lost in `--json` mode*: the warning stays on stderr, which is where the browser already writes per-request diagnostics and warnings in both modes (TECH-007 section 11).
- *Registry and route divergence on reload*: both are built in one function in one order and published together, and there is no accessor for one without the other.
