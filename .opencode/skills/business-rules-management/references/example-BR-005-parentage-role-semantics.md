# BR-005: Parentage Role Semantics

## Description

Parentage is represented as child-to-parent relationships between individuals with sire (male parent) and dam (female parent) role semantics. The MVP does not enforce additional cardinality constraints beyond role semantics.

## Rationale

Based on requirements REQ-01.005 — Sex, color pattern, and lineage information, and REQ-02 — Genealogy and Parentage Graph, this rule defines parentage as a directed relationship from child to sire and dam. Sex-typed roles (sire = male, dam = female) provide clarity in genealogy views. The MVP deliberately avoids enforcing cardinality constraints (e.g., a maximum number of sires per lamb) to handle real-world scenarios where parentage may be uncertain or disputed.

## Applicability

Applies at individual creation and when modifying parentage links. Sire and dam are optional fields. A sire must reference a male Individual; a dam must reference a female Individual.
