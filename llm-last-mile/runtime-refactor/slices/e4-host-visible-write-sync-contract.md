**Kind:** slice row
**Stable ID:** `e4-host-visible-write-sync-contract`
**Canonical for:** extracted E4 slice row only
**Status:** canonical slice row record
**Authority scope:** exact extracted source table header and row only; no schedule, dispatch, or implementation authority
**Source span:** [`../03-phase-slice-map.md`](../03-phase-slice-map.md) line 223
**Supersedes:** canonical ownership of the extracted `E4 — Host-visible write/sync contract` row
**Superseded by:** none
**Projection consumers:** [`track-e-dispatch-policy-and-config-projection.md`](track-e-dispatch-policy-and-config-projection.md), [`../03-phase-slice-map.md`](../03-phase-slice-map.md)

# E4 — Host-visible write/sync contract

> **Authority boundary:** This file owns only the extracted E4 Track E row. It preserves the exact row text below, does not broaden policy merely to make sync pass, and does not dispatch E4.

| Slice | Goal | Must-read docs | Sibling context | Allowed code areas | Explicit non-goals | Exit gate | Regression gates |
|---|---|---|---|---|---|---|---|
| **E4 — Host-visible write/sync contract** | Freeze and prove when world writes are immediately host-visible, isolated, or explicitly reconciled under narrowed policy. | `05` host-visible sync row; `04` monotonicity and snapshot rules; workspace overlay model sync boundary | E1–E3; world-service fs enforcement; broker | `world-service` fs/overlay execution; relevant `crates/world*`; workspace-sync code/tests/docs | No silent best-effort copying; no treating command success as host-visible proof; no policy broadening to make sync pass. | Matrix proof covers `host_visible=true`, full isolation, narrowed write allowlists, retained turns, denied writes, and explicit reconciliation semantics. | `RG-SYNC-01`, `RG-POLICY-02`, `RG-UAA-03` |
