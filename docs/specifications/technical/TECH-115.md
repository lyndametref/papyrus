---
id: TECH-115
title: "Latency Criteria"
created_date: '2026-11-27T17:13:35Z'
updated_date: '2026-11-27T19:16:45Z'
labels: []
references:
  - NFR-007
  - NFR-007.001
  - NFR-007.002
status: verified
---
# TECH-115 Latency Criteria

## Description
Covers the latency targets and the shared measurement method for browser routes at 1,000 active records. Server transport, caching, and page content are separate specifications.

## Latency reduction strategy

- The serving snapshot is built once per configuration generation and reused; a request performs one `stat` fingerprint check and rebuilds only when the configuration changed.
- Each request may build one record snapshot: it lists each enabled active directory once, maps canonical file stems to paths, and parses a matching record at most once.
- Full and front-matter-only parses use separate caches; a record already parsed in full is reused for front-matter-only reads within the request.
- The inventory page parses front matter only, once per listed file, and derives every card count from that single scan.
- List and board pages build their indexes from the same request snapshot, so a record parsed for a card, title, milestone, or reference is not parsed again.
- Static assets are served from embedded bytes; each first-party `ETag` is computed once per process, and a conditional request returns `304` without a body.
- No page, fragment, or record payload is cached across requests; caching is limited to the serving snapshot and the request-scoped record snapshot.

## Shared measurement method

- The corpus is 1,000 active records across the enabled document types, including a `grouped-record` group with 200 members and a `formatted-record` type with 50 records.
- Reference conditions are 4 vCPU, 8 GiB RAM, an SSD, Linux, one server process, loopback HTTP, and a release build with LTO and strip.
- Each route is measured with 10 warm-up requests followed by 100 measured sequential requests against one running server.
- The reported figures are the server-observed p50 and p95.
- The first request after a cold start and any request that triggers a configuration reload are excluded.
- The run and the measured p95 are recorded with the release.
- Every target is server-observed latency only; no target applies to client rendering time.

## Inventory page

- The document-type inventory page, at route `/`, is served with a p95 latency below 250 ms (NFR-007.001).

## View route

- A document view page is served with a p95 latency below 100 ms (NFR-007.002).
- The view run measures 100 requests per document model and includes one record with an unresolved reference and one member of the 200-member group (NFR-007.002).

## Edit route

- A document edit page is served with a p95 latency below 100 ms (NFR-007.002).
- The edit run measures 100 requests per document model and includes one record with an unresolved reference and one member of the 200-member group (NFR-007.002).
