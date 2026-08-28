**Kind:** architecture
**Stable ID:** `shared-invariant-08`
**Canonical for:** invariant 8 only
**Status:** canonical shared-architecture record
**Authority scope:** exact extracted invariant source body only; no implementation authority
**Source span:** [`../../01-target-architecture.md#8-world-placement-and-policy-mediation-are-separate-proofs`](../../01-target-architecture.md#8-world-placement-and-policy-mediation-are-separate-proofs) lines 179–186
**Supersedes:** canonical ownership of the extracted source body; source heading remains a compatibility anchor
**Superseded by:** none
**Projection consumers:** [`../README.md`](../README.md), [`../../01-target-architecture.md`](../../01-target-architecture.md)

# Invariant 8: World placement and policy mediation are separate proofs

### 8. World placement and policy mediation are separate proofs

A world-scoped UAA must both:

1. start from a guest-realizable runtime envelope; and
2. route every side-effecting operation through Substrate-owned policy enforcement.

`cwd`, `CODEX_HOME`, `SUBSTRATE_CAGED`, `add_dirs`, or `external_sandbox=true` do not prove operation mediation.
