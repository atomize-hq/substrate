**Kind:** slice row
**Stable ID:** `c3-router-ownership-restoration`
**Canonical for:** extracted C3 slice row only
**Status:** canonical slice row record
**Authority scope:** exact extracted source table header and row only; no schedule, dispatch, or implementation authority
**Source span:** [`../03-phase-slice-map.md`](../03-phase-slice-map.md) line 200
**Supersedes:** canonical ownership of the extracted `C3 — Router ownership restoration` row
**Superseded by:** none
**Projection consumers:** [`track-c-obligations-inbox-auto-attach-and-router-attach.md`](track-c-obligations-inbox-auto-attach-and-router-attach.md), [`../03-phase-slice-map.md`](../03-phase-slice-map.md)

# C3 — Router ownership restoration

> **Authority boundary:** This file owns only the extracted C3 Track C row. It preserves the exact row text below, does not widen worker-control authority, and does not dispatch C3.

| Slice | Goal | Authority owner | Must-read sections | Sibling context | Exact allowed code areas | Explicit non-goals | Exit gate | Regression gates | Prerequisites | Gates deliberately deferred |
|---|---|---|---|---|---|---|---|---|---|---|
| **C3 — Router ownership restoration** | Consume attach-eligible claims, restore one sanctioned host episode, settle the claim, and stop. | RouterAttachTrigger for triggering; HostSessionAuthority for the attach transition. | `02` RouterAttachTrigger row; router responsibilities/non-responsibilities; `04` HostExecutionEpisodeV1. | HostSessionAuthority; SurfaceAdapter; AutoAttachProjection. | `agent_runtime/auto_attach.rs`; bounded router entrypoint; helper launch adapter; router tests. | No prompt replay, approval/answer/fork/continue, or always-running backend assumption. | Router produces one attach outcome per session claim and cannot invoke worker-control verbs; manual reattach coexists without duplicate ownership. | `RG-ATTACH-01`, `RG-ATTACH-02`, `RG-AUTH-01`. | C2 and A1/A2. | Worker-control, receipt, and unrelated policy work. |
