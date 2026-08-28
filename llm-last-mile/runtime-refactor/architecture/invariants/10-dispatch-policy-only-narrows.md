**Kind:** architecture
**Stable ID:** `shared-invariant-10`
**Canonical for:** invariant 10 only
**Status:** canonical shared-architecture record
**Authority scope:** exact extracted invariant source body only; no implementation authority
**Source span:** [`../../01-target-architecture.md#10-dispatch-policy-only-narrows`](../../01-target-architecture.md#10-dispatch-policy-only-narrows) lines 192–201
**Supersedes:** canonical ownership of the extracted source body; source heading remains a compatibility anchor
**Superseded by:** none
**Projection consumers:** [`../README.md`](../README.md), [`../../01-target-architecture.md`](../../01-target-architecture.md)

# Invariant 10: Dispatch policy only narrows

### 10. Dispatch policy only narrows

```text
effective_turn_policy =
  current parent policy
  AND retained-worker capability cap, when present
  AND dispatch/turn narrowing patch, when present
```

Accepted work uses an immutable `PolicySnapshotV3`. Parent policy changes affect future acceptance; they do not silently mutate active work.
