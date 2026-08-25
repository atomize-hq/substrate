**Kind:** architecture
**Stable ID:** `shared-invariant-03`
**Canonical for:** invariant 3 only
**Status:** canonical shared-architecture record
**Authority scope:** exact extracted invariant source body only; no implementation authority
**Source span:** [`../../01-target-architecture.md#3-routing-is-exact-and-fail-closed`](../../01-target-architecture.md#3-routing-is-exact-and-fail-closed) lines 149–151
**Supersedes:** canonical ownership of the extracted source body; source heading remains a compatibility anchor
**Superseded by:** none
**Projection consumers:** [`../README.md`](../README.md), [`../../01-target-architecture.md`](../../01-target-architecture.md), [`../../03-phase-slice-map.md`](../../03-phase-slice-map.md)

# Invariant 3: Routing is exact and fail-closed

### 3. Routing is exact and fail-closed

Every world verb resolves exact session, caller, backend, world id/generation, and task/worker/active-run identity. Backend-only selection ambiguity fails closed. Model-facing callers never supply internal lease, resume, UAA-session, or participant lineage truth.
