**Kind:** track index
**Stable ID:** `track-c-obligations-inbox-auto-attach-and-router-attach`
**Canonical for:** Track C navigation only
**Status:** non-authoritative navigation
**Authority scope:** exact extracted Track C table only; no schedule, dispatch, or implementation authority
**Source span:** [`../03-phase-slice-map.md`](../03-phase-slice-map.md) lines 194–201
**Supersedes:** canonical ownership of the extracted Track C table
**Superseded by:** none
**Projection consumers:** [`README.md`](README.md), [`../03-phase-slice-map.md`](../03-phase-slice-map.md)

# Track C — Obligations, inbox, auto-attach, and router attach

> **Authority boundary:** This file is a navigation index only. The completed C1 family remains at its existing D6 owner; C2 and C3 remain later independent slices, undispatched here, and do not widen B4 or worker-control authority.

| Slice | Goal | Authority owner | Must-read sections | Sibling context | Exact allowed code areas | Explicit non-goals | Exit gate | Regression gates | Prerequisites | Gates deliberately deferred |
|---|---|---|---|---|---|---|---|---|---|---|
| [**C1 — canonical family row moved**](../b3-1-c1/slice-and-task.md#c1-packet-row) | [Canonical C1 packet row](../b3-1-c1/slice-and-task.md#c1-packet-row) | — | — | — | — | — | — | — | — | — |
| [**C2 — Inbox and auto-attach projections**](c2-inbox-and-auto-attach-projections.md) | Make inbox and attach eligibility pure projections over obligation state and effective policy. | InboxProjection and AutoAttachProjection. | `02` projection rows; notification inbox design; auto-attach trigger design. | ObligationLedger; CompatibilityReadModel; SteeringPolicyEngine. | `agent_runtime/{host_inbox,obligation_ledger,auto_attach,state_store}.rs`; `host_inbox_materialization.rs`; projection tests. | No host process launch, no worker action, and no removal of compatibility ingress without migration proof. | Deleting/rebuilding projections does not lose obligation truth; session-coalesced claim is deterministic; wrong-host and policy denial fail closed. | Projection clauses of `RG-OBL-02`, `RG-ATTACH-01`, `RG-BASE-03`. | A1, A2/A3, C1, and B4; B4 transitively supplies B2.2/B3.2/E2. | C3 router launch and unrelated worker controls. |
| [**C3 — Router ownership restoration**](c3-router-ownership-restoration.md) | Consume attach-eligible claims, restore one sanctioned host episode, settle the claim, and stop. | RouterAttachTrigger for triggering; HostSessionAuthority for the attach transition. | `02` RouterAttachTrigger row; router responsibilities/non-responsibilities; `04` HostExecutionEpisodeV1. | HostSessionAuthority; SurfaceAdapter; AutoAttachProjection. | `agent_runtime/auto_attach.rs`; bounded router entrypoint; helper launch adapter; router tests. | No prompt replay, approval/answer/fork/continue, or always-running backend assumption. | Router produces one attach outcome per session claim and cannot invoke worker-control verbs; manual reattach coexists without duplicate ownership. | `RG-ATTACH-01`, `RG-ATTACH-02`, `RG-AUTH-01`. | C2 and A1/A2. | Worker-control, receipt, and unrelated policy work. |
