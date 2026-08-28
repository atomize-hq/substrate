**Kind:** slice row
**Stable ID:** `e1-restricted-world-fs-narrowing`
**Canonical for:** extracted E1 slice row only
**Status:** canonical slice row record
**Authority scope:** exact extracted source table header and row only; no schedule, dispatch, or implementation authority
**Source span:** [`../03-phase-slice-map.md`](../03-phase-slice-map.md) line 220
**Supersedes:** canonical ownership of the extracted `E1 — Restricted world_fs narrowing` row
**Superseded by:** none
**Projection consumers:** [`track-e-dispatch-policy-and-config-projection.md`](track-e-dispatch-policy-and-config-projection.md), [`../03-phase-slice-map.md`](../03-phase-slice-map.md)

# E1 — Restricted world_fs narrowing

> **Authority boundary:** This file owns only the extracted E1 Track E row. It preserves the exact row text below, does not authorize policy implementation, and does not dispatch E1.

| Slice | Goal | Must-read docs | Sibling context | Allowed code areas | Explicit non-goals | Exit gate | Regression gates |
|---|---|---|---|---|---|---|---|
| **E1 — Restricted world_fs narrowing** | Accept a request-scoped restricted `PolicyPatch.world_fs`, validate path-containment monotonicity, and materialize a canonical narrowed snapshot. | `02` EffectivePolicyResolver + NarrowingPatch rows; `04` narrowing contract/rules; steering policy matrix capability section | WorldCommandExecutionBroker; receipt acceptance; agent inventory overlay logic | `crates/broker`; `execution/{policy_model,policy_snapshot,agent_inventory}.rs`; transport API policy types; resolver tests | No new filesystem policy model; no broadening dimensions; no receipt/manifests yet. | Gate=false rejects; gate=true accepts only narrowing; file-under-directory containment works; escapes/symlinks/broadening fail closed. | `RG-POLICY-01`, `RG-POLICY-02` |
