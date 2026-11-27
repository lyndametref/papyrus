---
id: TECH-115
title: "Latency Criteria"
created_date: '2026-11-27T17:13:35Z'
updated_date: '2026-11-27T17:51:47Z'
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
