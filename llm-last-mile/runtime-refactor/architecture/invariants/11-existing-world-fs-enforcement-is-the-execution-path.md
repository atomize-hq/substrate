**Kind:** architecture
**Stable ID:** `shared-invariant-11`
**Canonical for:** invariant 11 only
**Status:** canonical shared-architecture record
**Authority scope:** exact extracted invariant source body only; no implementation authority
**Source span:** [`../../01-target-architecture.md#11-existing-world_fs-enforcement-is-the-execution-path`](../../01-target-architecture.md#11-existing-world_fs-enforcement-is-the-execution-path) lines 203–205
**Supersedes:** canonical ownership of the extracted source body; source heading remains a compatibility anchor
**Superseded by:** none
**Projection consumers:** [`../README.md`](../README.md), [`../../01-target-architecture.md`](../../01-target-architecture.md)

# Invariant 11: Existing world_fs enforcement is the execution path

### 11. Existing `world_fs` enforcement is the execution path

Dispatch narrowing uses restricted `PolicyPatch.world_fs`, canonical finalization, `PolicySnapshotV3`, and the existing world-service overlay/full-isolation/Landlock/caged/network machinery. Do not create a parallel filesystem sandbox.
