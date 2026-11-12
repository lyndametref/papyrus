# Use Neo4j for Genealogy Storage

## Status

Accepted

## Decision

Use Neo4j as the primary storage for genealogical data. Each animal is a node with properties (id, name, birth date, etc.). Parent-child relationships are directed edges labeled `PARENT_OF`. The genealogical graph is updated only through the backend service layer enforcing domain invariants (e.g., no cycles, valid breeding dates).

Cypher queries handle pedigree depth queries, color inheritance path analysis, and inbreeding coefficient computation natively without recursive application logic.

## Context

The application needs to store and query complex genealogical relationships for an ovine flock: parentage chains, sibling relationships, multi-generation pedigrees, and inbreeding coefficient calculations. Relational databases require recursive CTEs or multiple joins for depth queries, which become slow and cumbersome beyond 3-4 generations. The domain is inherently graph-shaped — animals are nodes, parent-child relationships are edges.

We considered three options:
1. **PostgreSQL with recursive CTEs** — familiar stack, no additional infra, but query complexity and performance degrade with depth.
2. **Neo4j** — native graph storage, Cypher queries map directly to the domain model, excellent for multi-gen traversal.
3. **Document store (MongoDB)** — flexible schema but weak relationship traversal without application-level joins.

## Consequences

**Positive:**
- Pedigree queries of arbitrary depth execute in milliseconds via native graph traversal.
- Domain model maps directly to the data model — animals as nodes, relationships as edges.
- Cypher is readable by domain experts familiar with graph thinking.
- Neo4j's ACID transactions ensure data integrity for critical genealogical updates.

**Negative:**
- Additional infrastructure dependency — Neo4j cluster must be deployed alongside PostgreSQL.
- Team must learn Cypher and graph data modeling patterns.
- Not suitable for non-graph data (health records, observations) — those stay in PostgreSQL.
- Backup and operational tooling must cover both databases consistently.
